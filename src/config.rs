use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub line_access_token: String,
    pub line_channel_secret: String,
    pub skip_line_signature_verify: bool,
    pub groq_api_key: String,
    pub groq_model: String,
}

impl Config {
    pub fn from_env() -> Self {
        let line_access_token =
            env::var("LINE_ACCESS_TOKEN").expect("LINE_ACCESS_TOKEN must be set");
        let skip_line_signature_verify = env_flag("SKIP_LINE_SIGNATURE_VERIFY");
        let line_channel_secret = env::var("LINE_CHANNEL_SECRET").unwrap_or_default();

        if !skip_line_signature_verify && line_channel_secret.is_empty() {
            panic!(
                "LINE_CHANNEL_SECRET must be set in production; \
                 set SKIP_LINE_SIGNATURE_VERIFY=true only for local dev or tests"
            );
        }

        let groq_api_key = env::var("GROQ_API_KEY").expect("GROQ_API_KEY must be set");
        let groq_model =
            env::var("GROQ_MODEL").unwrap_or_else(|_| "llama-3.1-8b-instant".to_string());

        Self {
            line_access_token,
            line_channel_secret,
            skip_line_signature_verify,
            groq_api_key,
            groq_model,
        }
    }
}

fn env_flag(name: &str) -> bool {
    env::var(name)
        .map(|v| matches!(v.as_str(), "true" | "1" | "yes"))
        .unwrap_or(false)
}
