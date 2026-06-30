use std::env;

use crate::locale::Locale;

#[derive(Debug, Clone)]
pub struct LocaleConfig {
    pub line_access_token: String,
    pub line_channel_secret: String,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub skip_line_signature_verify: bool,
    pub groq_api_key: String,
    pub groq_model: String,
    pub th: LocaleConfig,
    pub id: LocaleConfig,
}

impl Config {
    pub fn from_env() -> Self {
        let skip_line_signature_verify = env_flag("SKIP_LINE_SIGNATURE_VERIFY");
        let th = load_locale_config(Locale::Th);
        let id = load_locale_config(Locale::Id);

        if !skip_line_signature_verify {
            for (locale, cfg) in [(Locale::Th, &th), (Locale::Id, &id)] {
                if cfg.line_channel_secret.is_empty() {
                    panic!(
                        "{}_LINE_CHANNEL_SECRET must be set in production; \
                         set SKIP_LINE_SIGNATURE_VERIFY=true only for local dev or tests",
                        locale.env_prefix()
                    );
                }
            }
        }

        let groq_api_key = env::var("GROQ_API_KEY").expect("GROQ_API_KEY must be set");
        let groq_model =
            env::var("GROQ_MODEL").unwrap_or_else(|_| "llama-3.1-8b-instant".to_string());

        Self {
            skip_line_signature_verify,
            groq_api_key,
            groq_model,
            th,
            id,
        }
    }

    pub fn locale(&self, locale: Locale) -> &LocaleConfig {
        match locale {
            Locale::Th => &self.th,
            Locale::Id => &self.id,
        }
    }
}

fn load_locale_config(locale: Locale) -> LocaleConfig {
    let prefix = locale.env_prefix();
    let access_key = format!("{prefix}_LINE_ACCESS_TOKEN");
    let secret_key = format!("{prefix}_LINE_CHANNEL_SECRET");

    let line_access_token =
        env::var(&access_key).unwrap_or_else(|_| panic!("{access_key} must be set"));

    let line_channel_secret = env::var(&secret_key).unwrap_or_default();

    LocaleConfig {
        line_access_token,
        line_channel_secret,
    }
}

fn env_flag(name: &str) -> bool {
    env::var(name)
        .map(|v| matches!(v.as_str(), "true" | "1" | "yes"))
        .unwrap_or(false)
}
