pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub api_version: String,
    pub port: String,
    pub base_url: String,
    pub resend_api_key: Option<String>, // Optional — None = dev mode
    pub resend_from: String,
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

        let resend_api_key = std::env::var("RESEND_API_KEY")
            .ok()
            .filter(|s| !s.is_empty());
        let resend_from = std::env::var("RESEND_FROM")
            .unwrap_or_else(|_| "noreply@excentra.exchange".to_string());

        let frontend_url = std::env::var("FRONTEND_URL").expect("FRONTEND_URL should be set");

        Config {
            database_url: db_url,
            jwt_secret,
            api_version,
            port,
            base_url,
            resend_api_key,
            resend_from,
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
        let frontend_url = "http://localhost:3000".to_string();

        Config {
            database_url: db_url,
            jwt_secret,
            api_version,
            port,
            base_url,
            resend_api_key: None, // dev mode
            resend_from: "noreply@excentra.local".to_string(),
            frontend_url,
        }
    }
}
