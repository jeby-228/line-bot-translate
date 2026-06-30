use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

use crate::app::AppState;
use crate::groq::client;
use crate::locale::{LocaleParam, parse_locale};

#[derive(Deserialize)]
pub struct TranslateRequest {
    pub text: String,
}

#[derive(Serialize)]
pub struct TranslateResponse {
    pub locale: &'static str,
    pub translation: String,
}

/// 純文字雙向翻譯（中文 ↔ 該語系）。
pub async fn translate_handler(
    State(state): State<Arc<AppState>>,
    Path(LocaleParam { locale }): Path<LocaleParam>,
    Json(body): Json<TranslateRequest>,
) -> Result<Response, StatusCode> {
    let locale = parse_locale(&locale)?;
    let text = body.text.trim();
    if text.is_empty() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "text must not be empty" })),
        )
            .into_response());
    }

    let system_prompt = locale.system_prompt();
    let translation = client::translate(
        &state.http,
        &state.config.groq_api_key,
        &state.config.groq_model,
        &system_prompt,
        text,
    )
    .await;

    Ok(Json(TranslateResponse {
        locale: locale.code(),
        translation,
    })
    .into_response())
}
