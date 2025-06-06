// main.rs
mod error;
mod quote;
mod templates;
mod web;
mod api;

// standard library imports
use std::borrow::Cow;
use std::str::FromStr;
use std::sync::Arc;
use std::error::Error; 
use std::path::PathBuf; 

// tokio imports
use tokio::{net::TcpListener, sync::RwLock};

// axum imports
use axum::{
    http::{StatusCode, Method},
    response::IntoResponse,
    routing::get, // Keep get for routing::get
    Router,
};

use clap::Parser;

use sqlx::{
    migrate::MigrateDatabase,
    sqlite::{SqliteConnectOptions, SqlitePool},
    ConnectOptions,
};

use tower_http::{
    cors::{Any, CorsLayer},
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_swagger_ui::SwaggerUi;
use log::LevelFilter; 

// --- GLOBAL STRUCTS AND FUNCTIONS ---

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, name = "init-from")]
    init_from: Option<PathBuf>, 
    #[arg(long, env = "DATABASE_URL")]
    db_uri: Option<String>,
    #[arg(short, long, default_value = "3000", env = "PORT")]
    port: u16,
}

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
}

fn get_db_uri_from_args_or_env(args_db_uri: Option<String>) -> Cow<'static, str> {
    if let Some(uri) = args_db_uri {
        uri.into()
    } else if let Ok(uri) = std::env::var("DATABASE_URL") {
        uri.into()
    } else {
        "sqlite:db/quotes.db".into()
    }
}

// helper function to get database pool
async fn get_db_pool(db_uri: &str) -> Result<SqlitePool, Box<dyn Error>> {
    if !sqlx::Sqlite::database_exists(db_uri).await.unwrap_or(false) {
        if let Some(colon_idx) = db_uri.rfind(':') {
            let path_part = &db_uri[colon_idx + 1..];
            if let Some(last_slash_idx) = path_part.rfind('/') {
                let dir_to_create = &path_part[..last_slash_idx];
                if !dir_to_create.is_empty() {
                    tokio::fs::create_dir_all(dir_to_create).await?;
                }
            }
        }
        sqlx::Sqlite::create_database(db_uri).await?;
    }

    let connect_options = SqliteConnectOptions::from_str(db_uri)?
        .create_if_missing(true)
        .log_statements(LevelFilter::Debug);

    let db_pool = SqlitePool::connect_with(connect_options).await?;
    sqlx::migrate!("./migrations").run(&db_pool).await?;
    Ok(db_pool)
}


#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::get_quote_api,
        crate::api::get_random_quote_api
    ),
    components(
        schemas(crate::quote::Quote, crate::quote::JsonQuote)
    ),
    tags(
        (name = "quote_server", description = "Quote API for NBA enthusiasts")
    )
)]
struct ApiDoc;

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Oops! Page not found.").into_response()
}

// --- MAIN APPLICATION ENTRY POINT ---
#[tokio::main]
async fn main() {
    if let Err(err) = run_app().await {
        eprintln!("quote_server: error: {:#}", err);
        std::process::exit(1);
    }
}

// --- MAIN RUN_APP FUNCTION ---
async fn run_app() -> Result<(), Box<dyn Error>> {
    // Initialize tracing (logging)
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "quote_server=debug,tower_http=debug".into()))
        .with(fmt::layer())
        .init();

    // parse command line arguments
    let args = Args::parse();

    // get database URI
    let db_uri = get_db_uri_from_args_or_env(args.db_uri);

    // get database pool
    let db_pool = get_db_pool(&db_uri).await?;

    // create shared application state
    let shared_state = Arc::new(RwLock::new(AppState { db: db_pool }));

    // initialize database with data from file if specified
    if let Some(quotes_path) = args.init_from {
        tracing::info!("Initializing database from {:?}", quotes_path);
        let app_reader = shared_state.read().await;
        match crate::quote::read_quotes_from_file(&quotes_path) {
            Ok(quotes) => {
                for jq_item in quotes {
                    let quote_data = jq_item.to_quote();
                    let mut tx = app_reader.db.begin().await?;
                    let insert_res = sqlx::query!(
                        "INSERT OR IGNORE INTO quotes (id, quote, author) VALUES (?, ?, ?)",
                        quote_data.id, quote_data.quote, quote_data.author
                    )
                    .execute(&mut *tx)
                    .await;

                    if let Err(e) = insert_res {
                        tracing::error!("Failed to insert quote {}: {}", quote_data.id, e);
                        tx.rollback().await?;
                        continue;
                    }
                    if let Err(e) = tx.commit().await {
                        tracing::error!("Failed to commit transaction for quote {}: {}", quote_data.id, e);
                    }
                }
                tracing::info!("Database initialized successfully.");
            }
            Err(e) => {
                tracing::error!("Failed to initialize database from file: {}", e);
                return Err(Box::new(e));
            }
        }
    }

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_origin(Any);

    // tracing layer for HTTP requests
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
        .on_response(DefaultOnResponse::new().level(tracing::Level::INFO));

    // openAPI documentation setup
    let openapi_document: utoipa::openapi::OpenApi = ApiDoc::openapi();

    // --- Axum Router Setup for Server-Side Rendered UI and API ---
    let app = Router::new()
        // web UI route for the main page
        .route("/", get(crate::web::get_main_page_handler))
        .nest_service("/static", tower_http::services::ServeDir::new("assets/static"))

        // your existing API routes
        .nest("/api/v1", api::router())
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi_document.clone()))
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc")) 
        .fallback(handler_404) // using the global handler_404
        .layer(cors)
        .layer(trace_layer)
        .with_state(shared_state);

    // start the server
    let listener = TcpListener::bind(&format!("127.0.0.1:{}", args.port)).await?;
    tracing::info!("Quote server listening on http://127.0.0.1:{}", args.port);
    axum::serve(listener, app).await?;

    Ok(())
}