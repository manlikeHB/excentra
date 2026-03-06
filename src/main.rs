use axum::{
    Router,
    routing::{get, post},
};
use dotenvy::dotenv;
use excentra::api::handlers::{auth::register_user, health::health};
use excentra::api::types::AppState;
use excentra::engine::exchange::Exchange;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set");
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET should be set");
    let pool = PgPool::connect(&db_url).await?;

    let shared_state = Arc::new(AppState {
        pool,
        exchange: Mutex::new(Exchange::new()),
        jwt_secret,
    });

    let app = Router::new()
        .route("/auth/register", post(register_user))
        .route("/health", get(health))
        .with_state(shared_state);

    let port = 3000;
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    println!("Server listening on port {}", port);
    axum::serve(listener, app).await?;

    Ok(())
}
