use axum::{
    Router,
    routing::{delete, get, post},
};
use dotenvy::dotenv;
use excentra::{
    api::{
        handlers::{
            asset::{add_asset, get_all_assets},
            auth::{login_user, logout, refresh_token, register_user},
            balances::{deposit, get_balances},
            health::health,
            orderbook::get_orderbook,
            orders::{cancel_order, get_order_by_id, get_orders, place_order},
            ticker::{get_all_tickers, get_ticker},
            trades::{get_recent_trades_for_a_pair, get_trade_history},
            trading_pairs::{
                add_trading_pair, get_active_trading_pairs, get_all_trading_pairs, get_trading_pair,
            },
            users::{get_user, update_user},
            ws::ws_handler,
        },
        types::AppState,
    },
    config::Config,
    db::queries as db_queries,
    engine::exchange::Exchange,
    services::{
        assets::AssetService, auth::AuthService, orderbook::OrderBookService, orders::OrderService,
        price_seed::PriceSeedService, ticker::TickerService, trades::TradeService,
        trading_pair::TradingPairService, users::UserService,
    },
};
use sqlx::PgPool;
use std::{
    sync::{Arc, atomic::AtomicU64},
    time::Instant,
};
use tokio::sync::{Mutex, broadcast};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{self, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,tower_http=debug")),
        )
        .init();

    // get config
    let config = Config::from_env();

    // init db pool
    let pool = PgPool::connect(&config.database_url).await?;

    // load trading pairs and resting orders into exchange
    let pairs = db_queries::get_active_trading_pairs(&pool).await?;
    let mut total_orders = 0;
    let mut exchange = Exchange::new();

    for pair in pairs {
        exchange.add_trading_pair(pair.id);

        let open_orders = db_queries::get_open_orders_by_pair(&pool, pair.id).await?;
        let order_book = exchange.get_order_book_mut(pair.id)?;
        total_orders += open_orders.len();

        for order in open_orders {
            order_book.add_limit_order(order.into())?;
        }
    }

    tracing::info!(order_count = total_orders, "Orders loaded into engine");

    let (tx, _) = broadcast::channel(1000);

    let orders_processed = Arc::new(AtomicU64::new(0));

    // app state
    let exchange = Arc::new(Mutex::new(exchange));
    let shared_state = Arc::new(AppState {
        pool: pool.clone(),
        order_service: OrderService::new(
            pool.clone(),
            exchange.clone(),
            tx.clone(),
            orders_processed,
        ),
        trading_pair_service: TradingPairService::new(pool.clone(), exchange.clone()),
        trade_service: TradeService::new(pool.clone()),
        asset_service: AssetService::new(pool.clone()),
        order_book_service: OrderBookService::new(exchange.clone()),
        ws_sender: tx.clone(),
        ticker_service: TickerService::new(pool.clone(), tx.clone()),
        ws_connections: Arc::new(AtomicU64::new(0)),
        started_at: Instant::now(),
        auth_service: AuthService::new(pool.clone(), config.jwt_secret),
        user_service: UserService::new(pool.clone()),
        base_url: config.base_url.to_owned(),
    });

    // Router & routes
    let auth_router = Router::new()
        .route("/register", post(register_user))
        .route("/login", post(login_user))
        .route("/refresh", post(refresh_token))
        .route("/logout", post(logout));

    let order_router = Router::new()
        .route("/", post(place_order).get(get_orders))
        .route("/{id}", delete(cancel_order).get(get_order_by_id));

    let balance_router = Router::new()
        .route("/deposit", post(deposit))
        .route("/", get(get_balances));

    let pair_router = Router::new()
        .route("/", get(get_all_trading_pairs).post(add_trading_pair))
        .route("/active", get(get_active_trading_pairs))
        .route("/{symbol}", get(get_trading_pair));

    let trades_router = Router::new()
        .route("/{symbol}", get(get_recent_trades_for_a_pair))
        .route("/me", get(get_trade_history));

    let asset_router = Router::new().route("/", post(add_asset).get(get_all_assets));

    let orderbook_router = Router::new().route("/{symbol}", get(get_orderbook));

    let ticker_router = Router::new()
        .route("/{symbol}", get(get_ticker))
        .route("/", get(get_all_tickers));

    let user_router = Router::new().route("/me", get(get_user).patch(update_user));

    let api_routes = Router::new()
        .nest("/auth", auth_router)
        .nest("/orders", order_router)
        .nest("/balances", balance_router)
        .nest("/pairs", pair_router)
        .nest("/trades", trades_router)
        .nest("/assets", asset_router)
        .nest("/orderbook", orderbook_router)
        .nest("/ticker", ticker_router)
        .nest("/users", user_router);

    let app = Router::new()
        .nest(&config.base_url, api_routes)
        .route("/health", get(health))
        .route("/ws", get(ws_handler))
        .with_state(shared_state.clone())
        .layer(TraceLayer::new_for_http());

    let ticker_state = shared_state.clone();
    tokio::spawn(async move {
        ticker_state.ticker_service.run().await;
    });

    // seed price
    let price_seed_service =
        PriceSeedService::new(pool.clone(), exchange.clone(), reqwest::Client::new());
    price_seed_service.seed_prices().await?;

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", config.port)).await?;
    tracing::info!(port = %config.port, "Server listening");
    axum::serve(listener, app).await?;

    Ok(())
}
