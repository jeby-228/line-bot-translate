use anyhow::Result;
use serde::Serialize;
use tracing::{error, info};

const LINE_REPLY_URL: &str = "https://api.line.me/v2/bot/message/reply";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ReplyRequest<'a> {
    reply_token: &'a str,
    messages: [TextMessage<'a>; 1],
}

#[derive(Serialize)]
struct TextMessage<'a> {
    #[serde(rename = "type")]
    message_type: &'a str,
    text: &'a str,
}

pub async fn send_reply(
    http: &reqwest::Client,
    access_token: &str,
    reply_token: &str,
    text: &str,
) -> Result<()> {
    let payload = ReplyRequest {
        reply_token,
        messages: [TextMessage {
            message_type: "text",
            text,
        }],
    };

    let resp = http
        .post(LINE_REPLY_URL)
        .bearer_auth(access_token)
        .json(&payload)
        .send()
        .await?;

    let status = resp.status();
    if status.is_success() {
        info!("LINE reply sent: status={}", status.as_u16());
    } else {
        let body = resp.text().await.unwrap_or_default();
        error!(
            "LINE reply failed: status={} body={}",
            status.as_u16(),
            body
        );
    }

    Ok(())
}
