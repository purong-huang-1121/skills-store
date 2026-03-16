use anyhow::{bail, Context, Result};
use base64::Engine;
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde_json::Value;
use sha2::Sha256;

const DEFAULT_BASE_URL: &str = "https://web3.okx.com";
const REQUEST_TIMEOUT_SECS: u64 = 30;
const CONNECT_TIMEOUT_SECS: u64 = 10;
const MAX_RETRIES: u32 = 3;

pub struct ApiClient {
    http: Client,
    base_url: String,
    api_key: String,
    secret_key: String,
    passphrase: String,
}

impl ApiClient {
    pub fn new(base_url_override: Option<&str>) -> Result<Self> {
        let api_key = std::env::var("OKX_API_KEY")
            .unwrap_or_else(|_| "03f0b376-251c-4618-862e-ae92929e0416".to_string());
        let secret_key = std::env::var("OKX_SECRET_KEY")
            .unwrap_or_else(|_| "652ECE8FF13210065B0851FFDA9191F7".to_string());
        let passphrase = std::env::var("OKX_PASSPHRASE")
            .unwrap_or_else(|_| "onchainOS#666".to_string());

        let base_url = base_url_override
            .map(|s| s.to_string())
            .or_else(|| std::env::var("OKX_BASE_URL").ok())
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());

        Ok(Self {
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
                .connect_timeout(std::time::Duration::from_secs(CONNECT_TIMEOUT_SECS))
                .build()?,
            base_url,
            api_key,
            secret_key,
            passphrase,
        })
    }

    fn sign(&self, timestamp: &str, method: &str, request_path: &str, body: &str) -> String {
        let prehash = format!("{}{}{}{}", timestamp, method, request_path, body);
        let mut mac = Hmac::<Sha256>::new_from_slice(self.secret_key.as_bytes())
            .expect("HMAC accepts any key length");
        mac.update(prehash.as_bytes());
        base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes())
    }

    fn apply_auth(
        &self,
        builder: reqwest::RequestBuilder,
        timestamp: &str,
        sign: &str,
    ) -> reqwest::RequestBuilder {
        builder
            .header("OK-ACCESS-KEY", &self.api_key)
            .header("OK-ACCESS-SIGN", sign)
            .header("OK-ACCESS-PASSPHRASE", &self.passphrase)
            .header("OK-ACCESS-TIMESTAMP", timestamp)
            .header("Content-Type", "application/json")
            .header("ok-client-type", "cli")
    }

    /// GET request with retry. `path` should be the API path without query string.
    /// Query params are appended and included in the signature.
    pub async fn get(&self, path: &str, query: &[(&str, &str)]) -> Result<Value> {
        let filtered: Vec<(&str, &str)> = query
            .iter()
            .filter(|(_, v)| !v.is_empty())
            .copied()
            .collect();

        let query_string = if filtered.is_empty() {
            String::new()
        } else {
            let pairs: Vec<String> = filtered
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            format!("?{}", pairs.join("&"))
        };

        let request_path = format!("{}{}", path, query_string);
        let url = format!("{}{}", self.base_url.trim_end_matches('/'), request_path);

        let mut last_err = anyhow::anyhow!("no attempts made");
        for attempt in 0..MAX_RETRIES {
            if attempt > 0 {
                let delay = std::time::Duration::from_millis(500 * (1 << attempt));
                tokio::time::sleep(delay).await;
            }
            let timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
            let sign = self.sign(&timestamp, "GET", &request_path, "");
            let req = self.apply_auth(self.http.get(&url), &timestamp, &sign);
            match req.send().await {
                Ok(resp) => match self.handle_response(resp).await {
                    Ok(v) => return Ok(v),
                    Err(e) if self.is_retryable(&e) => { last_err = e; }
                    Err(e) => return Err(e),
                },
                Err(e) => { last_err = anyhow::anyhow!("request failed: {e:#}"); }
            }
        }
        Err(last_err)
    }

    /// POST request with query parameters in URL (no JSON body), with retry.
    pub async fn post_query(&self, path: &str, query: &[(&str, &str)]) -> Result<Value> {
        let filtered: Vec<(&str, &str)> = query
            .iter()
            .filter(|(_, v)| !v.is_empty())
            .copied()
            .collect();

        let query_string = if filtered.is_empty() {
            String::new()
        } else {
            let pairs: Vec<String> = filtered
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            format!("?{}", pairs.join("&"))
        };

        let request_path = format!("{}{}", path, query_string);
        let url = format!("{}{}", self.base_url.trim_end_matches('/'), request_path);

        let mut last_err = anyhow::anyhow!("no attempts made");
        for attempt in 0..MAX_RETRIES {
            if attempt > 0 {
                let delay = std::time::Duration::from_millis(500 * (1 << attempt));
                tokio::time::sleep(delay).await;
            }
            let timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
            let sign = self.sign(&timestamp, "POST", &request_path, "");
            let req = self.apply_auth(self.http.post(&url), &timestamp, &sign);
            match req.send().await {
                Ok(resp) => match self.handle_response(resp).await {
                    Ok(v) => return Ok(v),
                    Err(e) if self.is_retryable(&e) => { last_err = e; }
                    Err(e) => return Err(e),
                },
                Err(e) => { last_err = anyhow::anyhow!("request failed: {e:#}"); }
            }
        }
        Err(last_err)
    }

    /// POST request with retry. `path` is the API path. `body` is the JSON body.
    pub async fn post(&self, path: &str, body: &Value) -> Result<Value> {
        let body_str = serde_json::to_string(body)?;
        let url = format!("{}{}", self.base_url.trim_end_matches('/'), path);

        let mut last_err = anyhow::anyhow!("no attempts made");
        for attempt in 0..MAX_RETRIES {
            if attempt > 0 {
                let delay = std::time::Duration::from_millis(500 * (1 << attempt));
                tokio::time::sleep(delay).await;
            }
            let timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
            let sign = self.sign(&timestamp, "POST", path, &body_str);
            let req = self.apply_auth(
                self.http.post(&url).body(body_str.clone()),
                &timestamp,
                &sign,
            );
            match req.send().await {
                Ok(resp) => match self.handle_response(resp).await {
                    Ok(v) => return Ok(v),
                    Err(e) if self.is_retryable(&e) => { last_err = e; }
                    Err(e) => return Err(e),
                },
                Err(e) => { last_err = anyhow::anyhow!("request failed: {e:#}"); }
            }
        }
        Err(last_err)
    }

    fn is_retryable(&self, e: &anyhow::Error) -> bool {
        let msg = format!("{e:#}");
        msg.contains("timed out")
            || msg.contains("timeout")
            || msg.contains("connection")
            || msg.contains("Server error (HTTP 5")
            || msg.contains("Rate limited")
    }

    async fn handle_response(&self, resp: reqwest::Response) -> Result<Value> {
        let status = resp.status();
        if status.as_u16() == 429 {
            bail!("Rate limited — retry with backoff");
        }
        if status.as_u16() >= 500 {
            bail!("Server error (HTTP {})", status.as_u16());
        }

        let body: Value = resp.json().await.context("failed to parse response")?;

        let code = body["code"].as_str().unwrap_or("-1");
        if code != "0" {
            let msg = body["msg"].as_str().unwrap_or("unknown error");
            bail!("API error (code={}): {}", code, msg);
        }

        Ok(body["data"].clone())
    }
}
