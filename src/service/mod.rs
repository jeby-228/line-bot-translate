use std::sync::Arc;

use tracing::{error, info};

use crate::app::AppState;
use crate::groq::client::translate;
use crate::line::{reply::send_reply, webhook::Event};

/// 將 webhook 事件分派到背景任務處理。
pub fn dispatch_events(state: Arc<AppState>, events: Vec<Event>) {
    for event in events {
        if event.event_type != "message" {
            continue;
        }

        let message = match event.message {
            Some(m) if m.message_type == "text" => m,
            _ => continue,
        };

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
            process_text_message(state, user_text, reply_token).await;
        });
    }
}

async fn process_text_message(state: Arc<AppState>, user_text: String, reply_token: String) {
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
