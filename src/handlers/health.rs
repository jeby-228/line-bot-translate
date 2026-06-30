use axum::response::Json;
use serde_json::{Value, json};

/// 健康檢查
pub async fn health_check() -> Json<Value> {
    Json(json!({
        "service": "webhook-translate",
        "version": "v1",
        "locales": ["th", "id"],
        "routes": {
            "translations": "POST /v1/{locale}/translations",
            "webhook": "POST /v1/{locale}/webhook",
            "check_api_key": "GET /v1/{locale}/check-api-key",
        }
    }))
}
