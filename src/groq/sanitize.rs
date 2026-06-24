use once_cell::sync::Lazy;
use regex::Regex;
use tracing::warn;

pub const SAFE_REJECT_MESSAGE: &str = "（這則訊息無法翻譯，請只傳送需要翻譯的文字內容）";

const CODE_SIGNATURES: &[&str] = &[
    "```",
    "#include",
    "typedef ",
    "void ",
    "int main",
    "public static",
    "def ",
    "import ",
    "function ",
    "<?php",
    "struct ",
    "malloc(",
    "printf(",
    "console.log",
    "system.out",
];

static SYMBOL_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[{};]").unwrap());

/// 偵測翻譯結果是否含有程式碼特徵（可能被 prompt injection 污染）。
pub fn looks_like_code(text: &str) -> bool {
    let lowered = text.to_lowercase();
    if CODE_SIGNATURES.iter().any(|sig| lowered.contains(sig)) {
        return true;
    }
    let symbol_count = SYMBOL_RE.find_iter(text).count();
    symbol_count >= 3
}

/// 從 Groq 回傳的 JSON 取出 `translation` 欄位並做輸出端把關。
/// 任何不合法 JSON、空結果或含程式碼特徵的輸出，一律回傳安全訊息。
pub fn extract_translation(raw: &str) -> String {
    let parsed: serde_json::Value = match serde_json::from_str(raw) {
        Ok(v) => v,
        Err(_) => {
            warn!(
                "Translation output not valid JSON (possible injection): {:?}",
                &raw[..raw.len().min(100)]
            );
            return SAFE_REJECT_MESSAGE.to_string();
        }
    };

    let result = match parsed.get("translation").and_then(|v| v.as_str()) {
        Some(s) => s.trim().to_string(),
        None => {
            warn!("Missing 'translation' field in Groq response");
            return SAFE_REJECT_MESSAGE.to_string();
        }
    };

    if result.is_empty() {
        return SAFE_REJECT_MESSAGE.to_string();
    }

    if looks_like_code(&result) {
        warn!(
            "Blocked code-like output (possible prompt injection): {:?}",
            &result[..result.len().min(100)]
        );
        return SAFE_REJECT_MESSAGE.to_string();
    }

    result
}
