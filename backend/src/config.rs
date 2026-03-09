// /backend/src/config.rs

#[derive(Clone)]
pub struct Config {
    pub etoro_base_url: String,
    pub etoro_api_key: String,
    pub etoro_user_key: String,
    pub bind_addr: String,
    pub cors_origin: Option<String>,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("etoro_base_url", &self.etoro_base_url)
            .field("etoro_api_key", &"[REDACTED]")
            .field("etoro_user_key", &"[REDACTED]")
            .field("bind_addr", &self.bind_addr)
            .field("cors_origin", &self.cors_origin)
            .finish()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing environment variable: {0}")]
    MissingVar(&'static str),
}

impl Config {
    pub fn from_env() -> Result<Config, ConfigError> {
        let etoro_base_url = std::env::var("ETORO_BASE_URL")
            .map_err(|_| ConfigError::MissingVar("ETORO_BASE_URL"))?;
        let etoro_api_key = std::env::var("ETORO_API_KEY")
            .map_err(|_| ConfigError::MissingVar("ETORO_API_KEY"))?;
        let etoro_user_key = std::env::var("ETORO_USER_KEY")
            .map_err(|_| ConfigError::MissingVar("ETORO_USER_KEY"))?;
        let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8000".into());
        let cors_origin = std::env::var("CORS_ORIGIN").ok();

        Ok(Self {
            etoro_base_url,
            etoro_api_key,
            etoro_user_key,
            bind_addr,
            cors_origin,
        })
    }
}