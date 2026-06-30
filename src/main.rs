mod app;
mod config;
mod groq;
mod handlers;
mod line;
mod locale;
mod service;

use std::{env, sync::Arc};

use axum::{
    Router,
    routing::{get, post},
};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{Level, info};

use app::AppState;
use config::Config;
use handlers::{check_api_key, health_check, translate_handler, webhook_handler};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with_target(false)
        .with_ansi(false)
        .with_writer(std::io::stdout)
        .compact()
        .init();

    info!(
        rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "(unset)".into()),
        "tracing initialized"
    );

    let config = Config::from_env();
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("Failed to build HTTP client");

    let state = Arc::new(AppState { config, http });

    let v1 = Router::new()
        .route("/{locale}/translations", post(translate_handler))
        .route("/{locale}/webhook", post(webhook_handler))
        .route("/{locale}/check-api-key", get(check_api_key));

    let app = Router::new()
        .route("/", get(health_check))
        .nest("/v1", v1)
        .with_state(state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        );

    let port = env::var("PORT").unwrap_or_else(|_| "8000".into());
    let addr = format!("0.0.0.0:{port}");
    info!("Starting server on {addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app).await.expect("Server error");
}
