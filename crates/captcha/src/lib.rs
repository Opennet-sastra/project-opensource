//! Captcha crate: OCR integration and image processing utilities.

use base64::{engine::general_purpose, Engine as _};
use std::time::Duration;

/// Tries to auto-solve a captcha image using an OCR microservice (e.g. localhost:8000/solve).
/// Returns `Some(text)` if successful, or `None` if the service is unreachable or fails.
pub async fn try_solve_ocr(data_uri: &str, endpoint: &str) -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .ok()?;

    let raw_b64 = data_uri.split_once(";base64,").map_or(data_uri, |(_, b)| b);

    let body = serde_json::json!({ "image": raw_b64 });
    let resp = client.post(endpoint).json(&body).send().await.ok()?;
    let json = resp.json::<serde_json::Value>().await.ok()?;

    if json["success"].as_bool().unwrap_or(false) {
        if let Some(text) = json["text"].as_str() {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    None
}

/// Decodes base64 captcha data URI into raw PNG/JPEG bytes.
pub fn decode_captcha_bytes(data_uri: &str) -> Option<Vec<u8>> {
    let raw = data_uri.split_once(";base64,").map_or(data_uri, |(_, b)| b);

    general_purpose::STANDARD.decode(raw).ok()
}
