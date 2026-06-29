use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{Json, NoContent},
};
use serde_json::{Value, json};
use tracing::{error, info};

use crate::app::AppState;
use crate::line::{signature::verify, webhook::LinePayload};
use crate::service::dispatch_events;

/// 健康檢查
pub async fn health_check() -> Json<Value> {
    Json(json!({"service": "webhook-translate" }))
}

/// LINE Webhook 入口
///
/// 接收 LINE Platform 傳來的事件，對文字訊息進行中↔印尼文翻譯後回覆。
/// 需帶有正確的 `X-Line-Signature` header（或設定 `SKIP_LINE_SIGNATURE_VERIFY=true` 於開發環境）。
pub async fn webhook_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<NoContent, StatusCode> {
    let signature = headers
        .get("x-line-signature")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !state.config.skip_line_signature_verify
        && !verify(&state.config.line_channel_secret, &body, signature)
    {
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

    info!(events = payload.events.len(), "webhook received");

    dispatch_events(state, payload.events);

    Ok(NoContent)
}
