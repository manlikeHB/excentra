use axum::{
    Router,
    routing::{delete, get, post},
};
use dotenvy::dotenv;
use excentra::{
    api::handlers::{
        asset::{add_asset, get_all_assets},
        orderbook::get_orderbook,
        orders::get_order_by_id,
        trading_pairs::{get_active_trading_pairs, get_all_trading_pairs},
        ws::ws_handler,
    },
    db::queries as db_queries,
    services::{assets::AssetService, orderbook::OrderBookService},
};
use excentra::{
    api::handlers::{
        auth::{login_user, register_user},
        balances::{deposit, get_balances},
        health::health,
    },
    services::orders::OrderService,
};
use excentra::{
    api::handlers::{trades::get_recent_trades, trading_pairs::add_trading_pair},
    engine::exchange::Exchange,
    services::{trades::TradeService, trading_pair::TradingPairService},
};
use excentra::{
    api::{
        handlers::orders::{cancel_order, get_orders, place_order},
        types::AppState,
    },
    config::Config,
};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // get config
    let config = Config::from_env();

    // init db pool
    let pool = PgPool::connect(&config.database_url).await?;

    // load trading pairs and resting orders into exchange
    let pairs = db_queries::get_all_trading_pairs(&pool).await?;
    let mut exchange = Exchange::new();

    for pair in pairs {
        exchange.add_trading_pair(pair.id);

        let open_orders = db_queries::get_open_orders_by_pair(&pool, pair.id).await?;
        let order_book = exchange.get_order_book_mut(pair.id)?;

        for order in open_orders {
            order_book.add_limit_order(order.into())?;
        }
    }

    let (tx, _) = broadcast::channel(1000);

    // app state
    let exchange = Arc::new(Mutex::new(exchange));
    let shared_state = Arc::new(AppState {
        pool: pool.clone(),
        order_service: OrderService::new(pool.clone(), exchange.clone(), tx.clone()),
        trading_pair_service: TradingPairService::new(pool.clone(), exchange.clone()),
        trade_service: TradeService::new(pool.clone()),
        asset_service: AssetService::new(pool.clone()),
        order_book_service: OrderBookService::new(exchange.clone()),
        ws_sender: tx,
        jwt_secret: config.jwt_secret,
    });

    // Router & routes
    let auth_router = Router::new()
        .route("/register", post(register_user))
        .route("/login", post(login_user));

    let order_router = Router::new()
        .route("/", post(place_order).get(get_orders))
        .route("/{id}", delete(cancel_order).get(get_order_by_id));

    let balance_router = Router::new()
        .route("/deposit", post(deposit))
        .route("/", get(get_balances));

    let pair_router = Router::new()
        .route("/", get(get_active_trading_pairs).post(add_trading_pair))
        .route("/all", get(get_all_trading_pairs));

    let trades_router = Router::new().route("/{symbol}", get(get_recent_trades));

    let asset_router = Router::new().route("/", post(add_asset).get(get_all_assets));

    let orderbook_router = Router::new().route("/{symbol}", get(get_orderbook));

    let ws_router = Router::new().route("/", get(ws_handler));

    let api_routes = Router::new()
        .nest("/auth", auth_router)
        .nest("/orders", order_router)
        .nest("/balances", balance_router)
        .nest("/pairs", pair_router)
        .nest("/trades", trades_router)
        .nest("/assets", asset_router)
        .nest("/orderbook", orderbook_router)
        .nest("/ws", ws_router);

    let app = Router::new()
        .nest(&config.base_url, api_routes)
        .route("/health", get(health))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", config.port)).await?;
    println!("Server listening on port {}", config.port);
    axum::serve(listener, app).await?;

    Ok(())
}
