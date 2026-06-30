use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::NoContent,
};
use tracing::{error, info};

use crate::app::AppState;
use crate::line::{signature::verify, webhook::LinePayload};
use crate::locale::{LocaleParam, parse_locale};
use crate::service::dispatch_events;

/// LINE Webhook 入口（依語系使用對應 LINE Channel 設定）。
pub async fn webhook_handler(
    State(state): State<Arc<AppState>>,
    Path(LocaleParam { locale }): Path<LocaleParam>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<NoContent, StatusCode> {
    let locale = parse_locale(&locale)?;
    let locale_config = state.config.locale(locale);

    let signature = headers
        .get("x-line-signature")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !state.config.skip_line_signature_verify
        && !verify(&locale_config.line_channel_secret, &body, signature)
    {
        error!(
            locale = locale.code(),
            reason = "invalid_signature",
            "webhook rejected"
        );
        return Err(StatusCode::BAD_REQUEST);
    }

    let payload: LinePayload = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            error!(
                locale = locale.code(),
                reason = "invalid_json",
                error = %e,
                "webhook rejected"
            );
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    info!(
        locale = locale.code(),
        events = payload.events.len(),
        "webhook received"
    );

    dispatch_events(state, locale, payload.events);

    Ok(NoContent)
}
