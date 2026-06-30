use std::sync::Arc;

use tracing::{error, info};

use crate::app::AppState;
use crate::groq::client::translate;
use crate::line::{reply::send_reply, webhook::Event};
use crate::locale::Locale;

/// 將 webhook 事件分派到背景任務處理。
pub fn dispatch_events(state: Arc<AppState>, locale: Locale, events: Vec<Event>) {
    for event in events {
        let event_type = event.event_type.clone();

        if event_type != "message" {
            info!(event_type = %event_type, "skipping unsupported event");
            continue;
        }

        let user_id = event.source.as_ref().and_then(|s| s.user_id.clone());
        let source_type = event.source.as_ref().and_then(|s| s.source_type.clone());
        let message = match event.message {
            Some(m) if m.message_type == "text" => m,
            other => {
                let message_type = other
                    .as_ref()
                    .map(|m| m.message_type.as_str())
                    .unwrap_or("none");
                info!(
                    event_type = %event_type,
                    source_type = source_type.as_deref().unwrap_or("unknown"),
                    user_id = user_id.as_deref().unwrap_or("unknown"),
                    message_type = %message_type,
                    "skipping non-text message"
                );
                continue;
            }
        };

        let message_id = message.id.clone();
        let user_text = match message.text {
            Some(t) => t.trim().to_string(),
            None => continue,
        };

        let reply_token = match event.reply_token {
            Some(t) => t,
            None => continue,
        };

        if user_text.is_empty() || reply_token.is_empty() {
            continue;
        }

        let state = Arc::clone(&state);
        tokio::spawn(async move {
            process_text_message(
                state,
                locale,
                source_type,
                user_id,
                message_id,
                user_text,
                reply_token,
            )
            .await;
        });
    }
}

async fn process_text_message(
    state: Arc<AppState>,
    locale: Locale,
    source_type: Option<String>,
    user_id: Option<String>,
    message_id: Option<String>,
    user_text: String,
    reply_token: String,
) {
    let locale_config = state.config.locale(locale);
    let system_prompt = locale.system_prompt();
    let text_preview = &user_text[..user_text.len().min(100)];

    info!(
        locale = locale.code(),
        source_type = source_type.as_deref().unwrap_or("unknown"),
        user_id = user_id.as_deref().unwrap_or("unknown"),
        message_id = message_id.as_deref().unwrap_or("unknown"),
        text_preview = %text_preview,
        "translating message"
    );

    let translated = translate(
        &state.http,
        &state.config.groq_api_key,
        &state.config.groq_model,
        &system_prompt,
        &user_text,
    )
    .await;

    let translation_preview = &translated[..translated.len().min(100)];
    info!(
        locale = locale.code(),
        source_type = source_type.as_deref().unwrap_or("unknown"),
        user_id = user_id.as_deref().unwrap_or("unknown"),
        message_id = message_id.as_deref().unwrap_or("unknown"),
        translation_preview = %translation_preview,
        "translation complete"
    );

    if let Err(e) = send_reply(
        &state.http,
        &locale_config.line_access_token,
        &reply_token,
        &translated,
    )
    .await
    {
        error!(
            locale = locale.code(),
            source_type = source_type.as_deref().unwrap_or("unknown"),
            user_id = user_id.as_deref().unwrap_or("unknown"),
            message_id = message_id.as_deref().unwrap_or("unknown"),
            error = %e,
            "failed to send LINE reply"
        );
    }
}
