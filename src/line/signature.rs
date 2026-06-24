use base64::{Engine as _, engine::general_purpose::STANDARD};
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// 驗證 LINE Webhook 的 X-Line-Signature header。
/// 若 secret 為空（開發環境），直接通過。
pub fn verify(secret: &str, body: &[u8], signature: &str) -> bool {
    if secret.is_empty() {
        return true;
    }

    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(body);
    let digest = mac.finalize().into_bytes();
    let expected = STANDARD.encode(digest);

    // 使用常數時間比較，防止 timing attack
    constant_time_eq(expected.as_bytes(), signature.as_bytes())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}
