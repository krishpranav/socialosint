use anyhow::Result;
use rand::Rng;
use reqwest::header::HeaderMap;
use reqwest::{Client, ClientBuilder, Proxy, Response};
use std::time::Duration;

const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.1 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:121.0) Gecko/20100101 Firefox/121.0",
];

pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub fn new(proxy: Option<String>) -> Result<Self> {
        let mut builder = ClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .cookie_store(true)
            .redirect(reqwest::redirect::Policy::limited(5));

        if let Some(proxy_url) = proxy {
            let proxy = if proxy_url.starts_with("socks5") {
                Proxy::all(&proxy_url)?
            } else {
                Proxy::all(&format!("http://{}", proxy_url))?
            };
            builder = builder.proxy(proxy);
        }

        let client = builder.build()?;

        Ok(Self { client })
    }

    pub fn random_user_agent() -> &'static str {
        let mut rng = rand::thread_rng();
        USER_AGENTS[rng.gen_range(0..USER_AGENTS.len())]
    }

    pub async fn get(&self, url: &str, mut headers: HeaderMap) -> Result<Response> {
        if !headers.contains_key("User-Agent") {
            headers.insert("User-Agent", Self::random_user_agent().parse()?);
        }

        let response = self.client.get(url).headers(headers).send().await?;

        Ok(response)
    }

    pub async fn post(&self, url: &str, mut headers: HeaderMap, body: String) -> Result<Response> {
        if !headers.contains_key("User-Agent") {
            headers.insert("User-Agent", Self::random_user_agent().parse()?);
        }

        if !headers.contains_key("Content-Type") {
            headers.insert("Content-Type", "application/json".parse()?);
        }

        let response = self
            .client
            .post(url)
            .headers(headers)
            .body(body)
            .send()
            .await?;

        Ok(response)
    }

    pub fn raw_client(&self) -> &Client {
        &self.client
    }
}
