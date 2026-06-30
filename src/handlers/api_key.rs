use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{Value, json};

use crate::app::AppState;
use crate::groq::client;
use crate::locale::{LocaleParam, parse_locale};

/// 檢查共用 Groq API key 是否有效（依語系路由，結果相同）。
pub async fn check_api_key(
    State(state): State<Arc<AppState>>,
    Path(LocaleParam { locale }): Path<LocaleParam>,
) -> Result<Json<Value>, StatusCode> {
    let locale = parse_locale(&locale)?;
    let result = client::check_api_key(&state.http, &state.config.groq_api_key).await;
    Ok(Json(json!({
        "locale": locale.code(),
        "groq": {
            "valid": result.valid,
            "message": result.message,
        }
    })))
}
