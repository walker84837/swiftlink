use serde::Deserialize;

/// Server configuration, comprising of base options and database configuration
#[derive(Deserialize)]
pub struct Config {
    /// Base options
    pub base: BaseOptions,
    /// Database configuration details
    pub database: DatabaseConfig,
}

/// Rate limiting configuration
#[derive(Deserialize)]
pub struct RateLimitConfig {
    /// Maximum requests per window (default: 10)
    pub max_requests: Option<u32>,
    /// Time window in seconds (default: 60)
    pub window_seconds: Option<u64>,
    /// Enable/disable rate limiting (default: true)
    pub enabled: Option<bool>,
}

/// Base options, for the web server and core functionality
#[derive(Deserialize)]
pub struct BaseOptions {
    /// Code length for generated short links, default is 6 if not provided
    pub code_size: Option<usize>,
    /// Port for the web server to listen on
    pub port: Option<u16>,
    /// (Optional) 10‐character alphanumeric bearer token for DELETE.
    /// If omitted, we generate one at startup and log it.
    pub bearer_token: Option<String>,
    /// Rate limiting configuration
    pub rate_limit: Option<RateLimitConfig>,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseType {
    #[default]
    Postgres,
    Sqlite,
}

/// Database-specific configuration
#[derive(Deserialize)]
pub struct DatabaseConfig {
    #[serde(default = "DatabaseType::default")]
    pub database_type: DatabaseType,
    pub username: Option<String>,
    pub password: Option<String>,
    /// Optional host (default "localhost")
    pub host: Option<String>,
    /// Optional port (default 5432)
    pub port: Option<u16>,
    /// Optional database name (default "swiftlink_db")
    pub database: Option<String>,
    pub max_connections: Option<u32>,
}

impl DatabaseConfig {
    /// Return a connection string for the database
    ///
    /// # Errors
    ///
    /// Returns an error if the database struct has mandatory fields set as [`Option::None`]
    pub fn database_url(&self) -> Result<String, String> {
        match self.database_type {
            DatabaseType::Postgres => {
                let username = self
                    .username
                    .as_ref()
                    .ok_or("Username must be specified for Postgres")?;

                let password = self
                    .password
                    .as_ref()
                    .ok_or("Password must be specified for Postgres")?;

                let host = self.host.as_deref().unwrap_or("localhost");
                let port = self.port.unwrap_or(5432);
                let database = self.database.as_deref().unwrap_or("swiftlink_db");

                Ok(format!(
                    "postgres://{}:{}@{}:{}/{}",
                    username, password, host, port, database
                ))
            }
            DatabaseType::Sqlite => self
                .database
                .clone()
                .ok_or_else(|| "Database path must be specified for SQLite".to_string()),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base: BaseOptions {
                code_size: Some(6),
                port: Some(8080),
                bearer_token: None,
                rate_limit: Some(RateLimitConfig {
                    max_requests: Some(10),
                    window_seconds: Some(60),
                    enabled: Some(true),
                }),
            },
            database: DatabaseConfig {
                database_type: DatabaseType::Postgres,
                username: Some("postgres".into()),
                password: Some("password".into()),
                host: Some("localhost".into()),
                port: Some(5432),
                database: Some("swiftlink_db".into()),
                max_connections: Some(5),
            },
        }
    }
}
