use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use clap::{Parser, ValueHint};
use env_logger::Target;
use log::{LevelFilter, error, info};
use rand::{Rng, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use std::{fs, path::PathBuf, sync::Arc};
use thiserror::Error;

type SwiftlinkResult<T> = Result<T, ServerError>;

#[derive(Debug, Error)]
enum ServerError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}

/// Server configuration, comprising of base options and database configuration
#[derive(Deserialize)]
struct Config {
    /// Base options
    base: BaseOptions,
    /// Database configuration details
    database: DatabaseConfig,
}

/// Base options, for the web server and core functionality
#[derive(Deserialize)]
struct BaseOptions {
    /// Code length for generated short links, default is 6 if not provided
    code_size: Option<usize>,
    /// Port for the web server to listen on
    port: Option<u16>,
}

/// Database-specific configuration
#[derive(Deserialize)]
struct DatabaseConfig {
    username: String,
    password: String,
    /// Optional host (default "localhost")
    host: Option<String>,
    /// Optional port (default 5432)
    port: Option<u16>,
    /// Optional database name (default "swiftlink_db")
    database: Option<String>,
    max_connections: Option<u32>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base: BaseOptions {
                code_size: Some(6),
                port: Some(8080),
            },
            database: DatabaseConfig {
                username: "postgres".into(),
                password: "password".into(),
                host: Some("localhost".into()),
                port: Some(5432),
                database: Some("swiftlink_db".into()),
                max_connections: Some(5),
            },
        }
    }
}

#[derive(Deserialize)]
struct CreateLinkRequest {
    url: String,
}

#[derive(Serialize)]
struct CreateLinkResponse {
    code: String,
    url: String,
}

#[derive(Clone)]
struct AppState {
    db_pool: sqlx::Pool<sqlx::Postgres>,
    config: Arc<Config>,
}

async fn init_db<T>(db_pool: &sqlx::Pool<sqlx::Postgres>) -> SwiftlinkResult<()>
where
    ServerError: std::convert::From<T>,
    T: std::convert::From<sqlx::Error>,
{
    sqlx::query!(
        r#"
            CREATE TABLE IF NOT EXISTS links (
                code TEXT PRIMARY KEY,
                url TEXT NOT NULL
            )
            "#
    )
    .execute(db_pool)
    .await
    .map_err(|e| <sqlx::Error as Into<T>>::into(e))?;
    info!("Created 'links' table");

    Ok(())
}

/// API Handler: Create a new short link
///
/// Note: Authentication is not yet implemented,
/// but you can later integrate JWT or a Discord-like token format.
async fn create_link(
    state: web::Data<AppState>,
    req: web::Json<CreateLinkRequest>,
) -> impl Responder {
    // Determine code length from config (default to 6)
    let code_size = state.config.base.code_size.unwrap_or(6);
    let code: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(code_size)
        .map(char::from)
        .collect();

    let result = sqlx::query!(
        "INSERT INTO links (code, url) VALUES ($1, $2)",
        code,
        req.url
    )
    .execute(&state.db_pool)
    .await;

    match result {
        Ok(_) => {
            info!("Created link: {} -> {}", code, req.url);
            HttpResponse::Ok().json(CreateLinkResponse {
                code: code.clone(),
                url: req.url.clone(),
            })
        }
        Err(e) => {
            error!("Error inserting link: {:?}", e);
            HttpResponse::InternalServerError().body("Error creating link")
        }
    }
}

/// Handler for redirection: given a code, look up the original URL and redirect.
async fn redirect(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let code = path.into_inner();
    let result = sqlx::query!("SELECT url FROM links WHERE code = $1", code)
        .fetch_one(&state.db_pool)
        .await;

    match result {
        Ok(record) => HttpResponse::Found()
            .append_header(("Location", record.url))
            .finish(),
        Err(e) => {
            error!("Error fetching link: {:?}", e);
            HttpResponse::NotFound().body("Link not found")
        }
    }
}

#[derive(Parser)]
struct Args {
    /// Path to the configuration file
    #[arg(short, long, value_hint = ValueHint::FilePath)]
    config: PathBuf,

    /// Log level
    #[arg(short, long, default_value = "Info")]
    log_level: LevelFilter,
}

#[actix_web::main]
async fn main() -> SwiftlinkResult<()> {
    let args = Args::parse();

    env_logger::builder()
        .filter_level(args.log_level)
        .target(Target::Stderr)
        .init();

    let config: Config = fs::read_to_string(args.config)
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default();
    let config = Arc::new(config);

    let db_config = &config.database;
    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        db_config.username,
        db_config.password,
        db_config.host.as_ref().unwrap_or(&"localhost".to_string()),
        db_config.port.unwrap_or(5432),
        db_config
            .database
            .as_ref()
            .unwrap_or(&"swiftlink_db".to_string()),
    );

    let db_pool = PgPoolOptions::new()
        .max_connections(db_config.max_connections.unwrap_or(5))
        .connect(&database_url)
        .await
        .expect("Failed to create database pool.");

    if let Err(e) = init_db::<ServerError>(&db_pool).await {
        error!("Failed to initialize database: {:?}", e);
        return Err(e);
    }

    let state = web::Data::new(AppState {
        db_pool,
        config: config.clone(),
    });

    let port = config.base.port.unwrap_or(8080);
    info!("Starting server on port {}", port);

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/api/create", web::post().to(create_link))
            .route("/{code}", web::get().to(redirect))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?;

    Ok(())
}
