use crate::detectors::{Detectors, Platform, ProfileStatus};
use crate::http::HttpClient;
use anyhow::Result;
use rand::Rng;
use reqwest::header::HeaderMap;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

pub struct Scraper {
    http: Arc<HttpClient>,
    detectors: Arc<Detectors>,
}

impl Scraper {
    pub fn new(http: Arc<HttpClient>, detectors: Arc<Detectors>) -> Self {
        Self { http, detectors }
    }

    async fn jitter_delay() {
        let mut rng = rand::thread_rng();
        let delay_ms = rng.gen_range(500..2000);
        sleep(Duration::from_millis(delay_ms)).await;
    }

    pub async fn get_json(&self, url: &str, headers: HeaderMap) -> Result<Value> {
        Self::jitter_delay().await;

        let response = self.http.get(url, headers).await?;

        if self.detectors.detect_blocking(&response) {
            let domain = url::Url::parse(url)?
                .host_str()
                .unwrap_or("unknown")
                .to_string();
            crate::telemetry::record_rate_limit_hit(&domain);
            anyhow::bail!("Request blocked or rate limited");
        }

        let json: Value = response.json().await?;
        Ok(json)
    }

    pub async fn post_json(&self, url: &str, headers: HeaderMap, body: String) -> Result<Value> {
        Self::jitter_delay().await;

        let response = self.http.post(url, headers, body).await?;

        if self.detectors.detect_blocking(&response) {
            let domain = url::Url::parse(url)?
                .host_str()
                .unwrap_or("unknown")
                .to_string();
            crate::telemetry::record_rate_limit_hit(&domain);
            anyhow::bail!("Request blocked or rate limited");
        }

        let json: Value = response.json().await?;
        Ok(json)
    }

    pub async fn check_profile_exists(
        &self,
        url: &str,
        platform: Platform,
    ) -> Result<ProfileStatus> {
        Self::jitter_delay().await;

        let response = self.http.get(url, HeaderMap::new()).await?;
        let status_code = response.status();
        let body = response.text().await?;

        let status = if status_code.as_u16() == 429 {
            ProfileStatus::RateLimited
        } else if status_code.as_u16() == 404 {
            ProfileStatus::NotFound
        } else if self.detectors.detect_soft_404(&body, platform) {
            ProfileStatus::NotFound
        } else if self.detectors.detect_private(&body, platform) {
            ProfileStatus::Private
        } else if status_code.is_success() {
            ProfileStatus::Exists
        } else {
            ProfileStatus::Error(format!("HTTP {}", status_code))
        };

        Ok(status)
    }

    pub async fn retry_with_backoff<F, Fut, T>(&self, mut f: F, max_retries: u32) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut retries = 0;
        let mut delay_ms = 1000;

        loop {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    retries += 1;
                    if retries >= max_retries {
                        return Err(e);
                    }

                    tracing::warn!("Retry {}/{}: {}", retries, max_retries, e);

                    let jitter = rand::thread_rng().gen_range(0..500);
                    sleep(Duration::from_millis(delay_ms + jitter)).await;

                    delay_ms *= 2;
                    if delay_ms > 30000 {
                        delay_ms = 30000;
                    }
                }
            }
        }
    }

    pub async fn get_raw(&self, url: &str, headers: HeaderMap) -> Result<reqwest::Response> {
        Self::jitter_delay().await;
        self.http.get(url, headers).await
    }

    pub async fn post_raw(
        &self,
        url: &str,
        headers: HeaderMap,
        body: String,
    ) -> Result<reqwest::Response> {
        Self::jitter_delay().await;
        self.http.post(url, headers, body).await
    }
}
