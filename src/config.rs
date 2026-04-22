pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub api_version: String,
    pub port: String,
    pub base_url: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_from: String,
    pub frontend_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        // get environmental variables
        let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set");
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET should be set");
        let api_version = std::env::var("API_VERSION").expect("API_VERSION should be set");
        let port = std::env::var("PORT").expect("PORT should be set");
        let base_url = format!("/api/{}", api_version);

        let smtp_host = std::env::var("SMTP_HOST").expect("SMTP_HOST should be set");
        let smtp_port = std::env::var("SMTP_PORT")
            .expect("SMTP_PORT should be set")
            .parse::<u16>()
            .expect("SMTP_PORT must be a valid port number");
        let smtp_from = std::env::var("SMTP_FROM").expect("SMTP_FROM should be set");
        let frontend_url = std::env::var("FRONTEND_URL").expect("FRONTEND_URL should be set");

        Config {
            database_url: db_url,
            jwt_secret,
            api_version,
            port,
            base_url,
            smtp_host,
            smtp_port,
            smtp_from,
            frontend_url,
        }
    }

    // #[cfg(test)]
    pub fn test_config() -> Config {
        let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:password@localhost:5432/excentra_test".to_string()
        });
        let jwt_secret = "some-strong-secret".to_string();
        let api_version = "v1".to_string();
        let port = "5098".to_string();
        let base_url = format!("/api/{}", api_version);

        let smtp_host = "localhost".to_string();
        let smtp_port = 1025;
        let smtp_from = "noreply@excentra.local".to_string();
        let frontend_url = "http://localhost:3000".to_string();

        Config {
            database_url: db_url,
            jwt_secret,
            api_version,
            port,
            base_url,
            smtp_host,
            smtp_port,
            smtp_from,
            frontend_url,
        }
    }
}
