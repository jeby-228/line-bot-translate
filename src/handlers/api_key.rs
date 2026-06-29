use std::sync::Arc;

use axum::{extract::State, response::Json};
use serde_json::{Value, json};

use crate::app::AppState;
use crate::groq::client;

/// 檢查 Groq API key 是否有效（不回傳 key 本身）。
pub async fn check_api_key(State(state): State<Arc<AppState>>) -> Json<Value> {
    let result = client::check_api_key(&state.http, &state.config.groq_api_key).await;
    Json(json!({
        "groq": {
            "valid": result.valid,
            "message": result.message,
        }
    }))
}
