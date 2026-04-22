use dotenvy::dotenv;
use excentra::{app::build_app, config::Config};

use sqlx::PgPool;
use std::net::SocketAddr;
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
    // Run any pending migrations on startup.
    // safe to run on every startup, including fresh and existing databases.
    sqlx::migrate!().run(&pool).await?;

    let app = build_app(&pool, &config, true).await?;

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port)).await?;
    tracing::info!(port = %config.port, "Server listening");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
