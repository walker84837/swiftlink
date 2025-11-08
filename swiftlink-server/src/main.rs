use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, web};
use clap::{Parser, ValueHint};
use env_logger::Target;
use log::{LevelFilter, error, info, warn};
use rand::{Rng, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use std::{
    borrow::Cow,
    fs,
    path::PathBuf,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use url::Url;

use thiserror::Error;

type SwiftlinkResult<T> = Result<T, ServerError>;

#[derive(Debug, Error)]
enum ServerError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}

fn generate_random_code(code_size: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(code_size)
        .map(char::from)
        .collect()
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
    /// (Optional) 10‐character alphanumeric bearer token for DELETE.
    /// If omitted, we generate one at startup and log it.
    bearer_token: Option<String>,
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
                bearer_token: None,
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

#[derive(Serialize)]
struct InfoResponse {
    code: String,
    created_at: i64,
    url: String,
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

/// Initialize the database (create the links table)
async fn init_db<T>(db_pool: &sqlx::Pool<sqlx::Postgres>) -> SwiftlinkResult<()>
where
    ServerError: From<T>,
    T: From<sqlx::Error>,
{
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS links (
            code TEXT PRIMARY KEY,
            url TEXT NOT NULL,
            created_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT
        )
        "#
    )
    .execute(db_pool)
    .await
    .map_err(|e| <sqlx::Error as Into<T>>::into(e))?;

    Ok(())
}

/// Validate URL and returns an error message if invalid.
/// Returns Ok(()) if valid.
fn validate_url(input: &str) -> Result<(), &'static str> {
    match Url::parse(input) {
        Ok(url) => {
            if url.has_host() {
                Ok(())
            } else {
                Err("URL must have a host")
            }
        }
        Err(_) => Err("Invalid URL"),
    }
}

/// Checks if the URL already exists in the database.
/// Returns Ok(Some(existing_code)) if found, Ok(None) if not found,
/// or Err(response) if a database error occurs.
async fn check_existing_url(
    db_pool: &sqlx::Pool<sqlx::Postgres>,
    url: &str,
) -> Result<Option<String>, HttpResponse> {
    match sqlx::query_scalar!("SELECT code FROM links WHERE url = $1", url)
        .fetch_optional(db_pool)
        .await
    {
        Ok(result) => Ok(result),
        Err(e) => {
            error!("Error checking for existing URL: {:?}", e);
            Err(HttpResponse::InternalServerError().body("Error creating link"))
        }
    }
}

/// Inserts a new link into the database.
async fn insert_new_link(
    db_pool: &sqlx::Pool<sqlx::Postgres>,
    url: &str,
    code: &str,
    created_at: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO links (code, url, created_at) VALUES ($1, $2, $3)",
        code,
        url,
        created_at
    )
    .execute(db_pool)
    .await
    .map(|_| ())
}

/// Handles the unique constraint conflict by fetching the existing link code.
/// Returns Ok(response) if successful, or Err(response) if a database error occurs.
async fn handle_unique_conflict(
    db_pool: &sqlx::Pool<sqlx::Postgres>,
    url: &str,
) -> Result<HttpResponse, HttpResponse> {
    match sqlx::query_scalar!("SELECT code FROM links WHERE url = $1", url)
        .fetch_one(db_pool)
        .await
    {
        Ok(existing_code) => {
            info!("URL inserted concurrently: {} -> {}", existing_code, url);
            Ok(HttpResponse::Ok().json(CreateLinkResponse {
                code: existing_code,
                url: url.to_string(),
            }))
        }
        Err(e) => {
            error!("Error fetching existing URL after conflict: {:?}", e);
            Err(HttpResponse::InternalServerError().body("Error creating link"))
        }
    }
}

async fn delete_link(
    state: web::Data<AppState>,
    path: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    let configured_token = match &state.config.base.bearer_token {
        Some(tok) => tok.clone(),
        None => {
            error!("Bearer token was somehow not set in configuration.");
            return HttpResponse::InternalServerError().body("Server misconfiguration");
        }
    };

    let auth_header = match req.headers().get("Authorization") {
        Some(hv) => hv.to_str().unwrap_or(""),
        None => "",
    };

    let expected_prefix = "Bearer ";
    if !auth_header.starts_with(expected_prefix) {
        return HttpResponse::Unauthorized().body("Missing or invalid authorization header");
    }
    let provided_token = &auth_header[expected_prefix.len()..];
    if provided_token != configured_token {
        return HttpResponse::Unauthorized().body("Invalid bearer token");
    }

    let code_to_delete: String = path.into_inner();
    let result = sqlx::query!("DELETE FROM links WHERE code = $1", code_to_delete)
        .execute(&state.db_pool)
        .await;
    info!("Deleting code: {code_to_delete}");

    match result {
        Ok(res) => {
            if res.rows_affected() == 0 {
                HttpResponse::NotFound().body("Link not found")
            } else {
                HttpResponse::Ok().body("Link deleted")
            }
        }
        Err(e) => {
            warn!("Error deleting link: {:?}", e);
            HttpResponse::InternalServerError().body("Error deleting link")
        }
    }
}

/// API Handler: Create a new short link
///
/// The main handler calls helper functions for input validation,
/// existing URL check, insertions etc. This makes error handling and
/// code readability better.
async fn create_link(
    state: web::Data<AppState>,
    req: web::Json<CreateLinkRequest>,
) -> impl Responder {
    // Input Validation
    if let Err(e) = validate_url(&req.url) {
        return HttpResponse::BadRequest().body(e);
    }

    // Check if URL is already present in the DB
    match check_existing_url(&state.db_pool, &req.url).await {
        Ok(Some(existing_code)) => {
            info!("URL already exists: {} -> {}", existing_code, req.url);
            return HttpResponse::Ok().json(CreateLinkResponse {
                code: existing_code,
                url: req.url.clone(),
            });
        }
        Ok(None) => {} // Continue to create new link
        Err(err_response) => return err_response,
    }

    // Generate new link data
    let code_size = state.config.base.code_size.unwrap_or(6);
    let code = generate_random_code(code_size);
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time error")
        .as_secs() as i64;

    // Try inserting the new link into the database
    let result = insert_new_link(&state.db_pool, &req.url, &code, created_at).await;
    match result {
        Ok(_) => {
            info!("Created link: {} -> {} at {}", code, req.url, created_at);
            HttpResponse::Ok().json(CreateLinkResponse {
                code,
                url: req.url.clone(),
            })
        }
        Err(e) => {
            // Check if the error is a duplicate key error (unique constraint violation)
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.code() == Some(Cow::Borrowed("23505")) {
                    return match handle_unique_conflict(&state.db_pool, &req.url).await {
                        Ok(response) => response,
                        Err(err_response) => err_response,
                    };
                }
            }
            error!("Error inserting link: {:?}", e);
            HttpResponse::InternalServerError().body("Error creating link")
        }
    }
}

/// API Handler: Get link info (given a code)
async fn get_link_info(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let code = path.into_inner();
    let result = sqlx::query!("SELECT url, created_at FROM links WHERE code = $1", code)
        .fetch_one(&state.db_pool)
        .await;

    match result {
        Ok(record) => HttpResponse::Ok().json(InfoResponse {
            code,
            created_at: record.created_at,
            url: record.url,
        }),
        Err(e) => {
            error!("Error fetching info for code {}: {:?}", code, e);
            HttpResponse::NotFound().body("Link not found")
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

/// Command-line arguments structure.
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

    let mut raw_config: Config = fs::read_to_string(&args.config)
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default();

    if raw_config.base.bearer_token.is_none() {
        let generated = generate_random_code(10);
        info!(
            "No bearer_token set in config.toml; generated one: {}",
            generated
        );
        raw_config.base.bearer_token = Some(generated);
    }

    let config = Arc::new(raw_config);

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
            .route("/api/info/{code}", web::get().to(get_link_info))
            .route("/{code}", web::delete().to(delete_link))
            .route("/{code}", web::get().to(redirect))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?;

    Ok(())
}
