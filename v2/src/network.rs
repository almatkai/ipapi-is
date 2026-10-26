use crate::domain::{decode, MAX_BYTES};
use serde_json::Value;
use std::time::Duration;

pub fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .https_only(true)
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(15))
        .pool_max_idle_per_host(0)
        .build()
        .map_err(|_| "Cannot initialize HTTPS client".into())
}
// Caller creates only fixed-origin URLs. Kept separate for synthetic HTTP/TLS contract tests.
pub async fn receive(client: &reqwest::Client, url: url::Url) -> Result<Value, String> {
    let mut res = client
        .get(url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|_| "Network request failed or timed out")?;
    match res.status().as_u16() {
        200..=299 => {}
        401 | 403 => return Err("Service authorization failed".into()),
        429 => return Err("Service rate limit reached; revalidate later".into()),
        300..=399 => return Err("Service redirect refused".into()),
        _ => return Err("Service request failed".into()),
    }
    let ct = res
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    if ct != "application/json" && !ct.ends_with("+json") {
        return Err("Service did not return JSON".into());
    }
    if res.content_length().is_some_and(|n| n > MAX_BYTES as u64) {
        return Err("Response exceeds the size limit".into());
    }
    let mut b = Vec::new();
    while let Some(chunk) = res.chunk().await.map_err(|_| "Response transfer failed")? {
        if b.len() + chunk.len() > MAX_BYTES {
            return Err("Response exceeds the size limit".into());
        }
        b.extend_from_slice(&chunk);
    }
    decode(&b)
}
pub async fn lookup(client: &reqwest::Client, input: &str) -> Result<Value, String> {
    let url = crate::domain::query(input)?;
    // Entire operation (including body/decode) owns a deadline; dropping cancels the request.
    tokio::time::timeout(Duration::from_secs(16), receive(client, url))
        .await
        .map_err(|_| "Network request timed out".to_string())?
}
