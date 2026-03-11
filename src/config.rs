pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub api_version: String,
    pub port: String,
    pub base_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        // get environmental variables
        let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set");
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET should be set");
        let api_version = std::env::var("API_VERSION").expect("API_VERSION should be set");
        let port = std::env::var("PORT").expect("PORT should be set");
        let base_url = format!("/api/{}", api_version);

        Config {
            database_url: db_url,
            jwt_secret,
            api_version,
            port,
            base_url,
        }
    }
}
