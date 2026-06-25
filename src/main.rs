mod config;
mod groq;
mod line;

use std::sync::Arc;

use axum::{
    Router,
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
};
use serde_json::{Value, json};
use tower_http::trace::TraceLayer;
use tracing::{error, info};

use config::Config;
use groq::client::translate;
use line::{reply::send_reply, signature::verify, webhook::LinePayload};

// ── AppState ──────────────────────────────────────────────────────────────────

struct AppState {
    config: Config,
    http: reqwest::Client,
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with_target(false)
        .compact()
        .init();

    let config = Config::from_env();
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("Failed to build HTTP client");

    let state = Arc::new(AppState { config, http });

    let app = Router::new()
        .route("/", get(health_check))
        .route("/webhook", post(webhook_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = "0.0.0.0:8000";
    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app).await.expect("Server error");
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// 健康檢查
async fn health_check() -> Json<Value> {
    Json(json!({ "service": "webhook-translate" }))
}

/// LINE Webhook 入口
///
/// 接收 LINE Platform 傳來的事件，對文字訊息進行中↔印尼文翻譯後回覆。
/// 需帶有正確的 `X-Line-Signature` header（開發環境可不帶）。
async fn webhook_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, StatusCode> {
    let signature = headers
        .get("x-line-signature")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !verify(&state.config.line_channel_secret, &body, signature) {
        error!("Invalid LINE signature");
        return Err(StatusCode::BAD_REQUEST);
    }

    let payload: LinePayload = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            error!("Invalid JSON body: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    for event in &payload.events {
        if event.event_type != "message" {
            continue;
        }

        let message = match &event.message {
            Some(m) if m.message_type == "text" => m,
            _ => continue,
        };

        let user_text = match &message.text {
            Some(t) => t.trim().to_string(),
            None => continue,
        };

        let reply_token = match &event.reply_token {
            Some(t) => t.clone(),
            None => continue,
        };

        if user_text.is_empty() || reply_token.is_empty() {
            continue;
        }

        info!("Translating: {:?}", &user_text[..user_text.len().min(100)]);

        let translated = translate(
            &state.http,
            &state.config.groq_api_key,
            &state.config.groq_model,
            &user_text,
        )
        .await;

        info!("Translated: {:?}", &translated[..translated.len().min(100)]);

        if let Err(e) = send_reply(
            &state.http,
            &state.config.line_access_token,
            &reply_token,
            &translated,
        )
        .await
        {
            error!("Failed to send LINE reply: {}", e);
        }
    }

    Ok(Json(json!({ "status": "ok" })))
}
