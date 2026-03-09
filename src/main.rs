use axum::{
    Router,
    routing::{get, post},
};
use dotenvy::dotenv;
use excentra::api::handlers::{
    auth::{login_user, register_user},
    balances::{deposit, get_balances},
    health::health,
};
use excentra::api::{handlers::orders::place_order, types::AppState};
use excentra::db::queries as db_queries;
use excentra::engine::exchange::Exchange;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // get environmental variables
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set");
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET should be set");
    let api_version = std::env::var("API_VERSION").expect("API_VERSION should be set");
    let port = std::env::var("PORT").expect("PORT should be set");
    let base_url = format!("/api/{}", api_version);

    // init db pool
    let pool = PgPool::connect(&db_url).await?;

    // load trading pairs into exchange
    let pairs = db_queries::get_all_trading_pairs(&pool).await?;
    let mut exchange = Exchange::new();
    for pair in pairs {
        exchange.add_trading_pair(pair.id);
    }

    // app state
    let shared_state = Arc::new(AppState {
        pool,
        exchange: Mutex::new(exchange),
        jwt_secret,
    });

    // Router & routes
    let auth_router = Router::new()
        .route("/register", post(register_user))
        .route("/login", post(login_user));

    let order_router = Router::new().route("/", post(place_order));

    let balance_router = Router::new()
        .route("/deposit", post(deposit))
        .route("/", get(get_balances));

    let api_routes = Router::new()
        .nest("/auth", auth_router)
        .nest("/orders", order_router)
        .nest("/balances", balance_router);

    let app = Router::new()
        .nest(&base_url, api_routes)
        .route("/health", get(health))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    println!("Server listening on port {}", port);
    axum::serve(listener, app).await?;

    Ok(())
}
