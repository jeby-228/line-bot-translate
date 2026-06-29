use axum::response::Json;
use serde_json::{Value, json};

/// 健康檢查
pub async fn health_check() -> Json<Value> {
    Json(json!({"service": "webhook-translate" }))
}
