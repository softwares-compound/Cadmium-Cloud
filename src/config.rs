use std::env;

pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
}

pub fn load() -> Config {
    Config {
        host: env::var("CADMIUM_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
        port: env::var("CADMIUM_PORT").unwrap_or_else(|_| "8080".to_string()).parse().expect("Invalid port"),
        database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
    }
}
