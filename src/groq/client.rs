use serde::{Deserialize, Serialize};
use tracing::{error, info};

use super::sanitize::{SAFE_REJECT_MESSAGE, extract_translation};

const GROQ_API_URL: &str = "https://api.groq.com/openai/v1/chat/completions";
const MAX_INPUT_CHARS: usize = 500;

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    temperature: f32,
    response_format: ResponseFormat,
    messages: Vec<ChatMessage<'a>>,
}

#[derive(Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    format_type: &'static str,
}

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: AssistantMessage,
}

#[derive(Deserialize)]
struct AssistantMessage {
    content: String,
}

/// 呼叫 Groq Chat Completions API，回傳翻譯後文字。
pub async fn translate(
    http: &reqwest::Client,
    api_key: &str,
    model: &str,
    system_prompt: &str,
    user_text: &str,
) -> String {
    let safe_text = if user_text.len() > MAX_INPUT_CHARS {
        &user_text[..MAX_INPUT_CHARS]
    } else {
        user_text
    };

    let request = ChatRequest {
        model,
        temperature: 0.3,
        response_format: ResponseFormat {
            format_type: "json_object",
        },
        messages: vec![
            ChatMessage {
                role: "system",
                content: system_prompt.to_string(),
            },
            ChatMessage {
                role: "user",
                content: format!("<user_input>\n{}\n</user_input>", safe_text),
            },
        ],
    };

    let resp = match http
        .post(GROQ_API_URL)
        .bearer_auth(api_key)
        .json(&request)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            error!("Groq HTTP error: {}", e);
            return format!(
                "Groq 連線失敗: {}",
                &e.to_string()[..e.to_string().len().min(100)]
            );
        }
    };

    let status = resp.status();
    if status.as_u16() == 401 {
        return "【錯誤】API Key 無效，請檢查 GROQ_API_KEY。".to_string();
    }

    let body = match resp.text().await {
        Ok(b) => b,
        Err(e) => {
            error!("Groq response read error: {}", e);
            return SAFE_REJECT_MESSAGE.to_string();
        }
    };

    if !status.is_success() {
        if body.contains("model_not_found") {
            return "【錯誤】模型名稱已過期，請更新 GROQ_MODEL 環境變數。".to_string();
        }
        error!(
            "Groq API error: status={} body={}",
            status.as_u16(),
            &body[..body.len().min(200)]
        );
        return format!("Groq 報錯: {}", &body[..body.len().min(100)]);
    }

    let chat_resp: ChatResponse = match serde_json::from_str(&body) {
        Ok(r) => r,
        Err(e) => {
            error!(
                "Groq response parse error: {} body={}",
                e,
                &body[..body.len().min(200)]
            );
            return SAFE_REJECT_MESSAGE.to_string();
        }
    };

    let raw = chat_resp
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .unwrap_or_default();

    info!("Groq raw output: {:?}", &raw[..raw.len().min(200)]);
    extract_translation(&raw)
}

const GROQ_MODELS_URL: &str = "https://api.groq.com/openai/v1/models";

#[derive(Debug, Serialize)]
pub struct ApiKeyCheck {
    pub valid: bool,
    pub message: String,
}

/// 驗證 Groq API key 是否有效。測試用 dummy key 不會發出外部請求。
pub async fn check_api_key(http: &reqwest::Client, api_key: &str) -> ApiKeyCheck {
    if api_key == "test-dummy-key" {
        return ApiKeyCheck {
            valid: false,
            message: "test dummy key (skipped live validation)".into(),
        };
    }

    let resp = match http.get(GROQ_MODELS_URL).bearer_auth(api_key).send().await {
        Ok(r) => r,
        Err(e) => {
            error!("Groq API key check HTTP error: {}", e);
            let err_msg = e.to_string();
            return ApiKeyCheck {
                valid: false,
                message: format!("connection failed: {}", &err_msg[..err_msg.len().min(100)]),
            };
        }
    };

    match resp.status().as_u16() {
        200 => ApiKeyCheck {
            valid: true,
            message: "ok".into(),
        },
        401 => ApiKeyCheck {
            valid: false,
            message: "invalid API key".into(),
        },
        status => {
            let body = resp.text().await.unwrap_or_default();
            error!(
                "Groq API key check failed: status={} body={}",
                status,
                &body[..body.len().min(200)]
            );
            ApiKeyCheck {
                valid: false,
                message: format!("groq returned {status}"),
            }
        }
    }
}
