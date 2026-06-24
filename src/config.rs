use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub line_access_token: String,
    pub line_channel_secret: String,
    pub groq_api_key: String,
    pub groq_model: String,
}

impl Config {
    pub fn from_env() -> Self {
        let line_access_token =
            env::var("LINE_ACCESS_TOKEN").expect("LINE_ACCESS_TOKEN must be set");
        let line_channel_secret =
            env::var("LINE_CHANNEL_SECRET").expect("LINE_CHANNEL_SECRET must be set");
        let groq_api_key = env::var("GROQ_API_KEY").expect("GROQ_API_KEY must be set");
        let groq_model =
            env::var("GROQ_MODEL").unwrap_or_else(|_| "llama-3.1-8b-instant".to_string());

        Self {
            line_access_token,
            line_channel_secret,
            groq_api_key,
            groq_model,
        }
    }
}
