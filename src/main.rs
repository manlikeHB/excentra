use axum::{
    Router,
    routing::{get, post},
};
use dotenvy::dotenv;
use excentra::api::handlers::{
    auth::{login_user, register_user},
    health::health,
};
use excentra::api::{handlers::orders::place_order, types::AppState};
use excentra::engine::exchange::Exchange;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set");
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET should be set");
    let api_version = std::env::var("API_VERSION").expect("API_VERSION should be set");
    let port = std::env::var("PORT").expect("PORT should be set");
    let base_url = format!("/api/{}", api_version);

    let pool = PgPool::connect(&db_url).await?;

    let shared_state = Arc::new(AppState {
        pool,
        exchange: Mutex::new(Exchange::new()),
        jwt_secret,
    });

    let auth_router = Router::new()
        .route("/register", post(register_user))
        .route("/login", post(login_user));

    let order_router = Router::new().route("/", post(place_order));

    let api_routes = Router::new()
        .nest("/auth", auth_router)
        .nest("/orders", order_router);

    let app = Router::new()
        .nest(&base_url, api_routes)
        .route("/health", get(health))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    println!("Server listening on port {}", port);
    axum::serve(listener, app).await?;

    Ok(())
}
