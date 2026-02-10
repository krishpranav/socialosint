use crate::core::Core;
use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LeakResult {
    pub email: String,
    pub password: String,
}

#[derive(Debug)]
pub struct PwnDBResult {
    pub user: String,
    pub user_id: String,
    pub leaks: Vec<LeakResult>,
}

pub struct PwnDBAPI {
    core: Core,
}

impl PwnDBAPI {
    pub fn new(core: Core) -> Self {
        Self { core }
    }

    pub async fn search_leak(&self, email: &str) -> Result<Vec<LeakResult>> {
        let domain = email.split('@').nth(1).unwrap_or("");
        let luser = email.split('@').next().unwrap_or("");

        if domain.is_empty() || luser.is_empty() {
            return Ok(Vec::new());
        }

        let query = format!("luser={} domain={}", luser, domain);

        self.core
            .logger
            .info(&format!("Searching PwnDB for: {}", email));

        let url = "http://pwndb2am4tzkvold.onion/";

        let response = self
            .core
            .http
            .raw_client()
            .post(url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(query)
            .send()
            .await?;

        let text = response.text().await?;
        let leaks = self.parse_pwndb_response(&text);

        if !leaks.is_empty() {
            self.core
                .logger
                .good(&format!("Found {} leaks for {}", leaks.len(), email));
        }

        Ok(leaks)
    }

    pub fn parse_pwndb_response(&self, text: &str) -> Vec<LeakResult> {
        let mut results = Vec::new();

        let luser_regex = Regex::new(r"\[luser\] => ([^\n]+)").unwrap();
        let domain_regex = Regex::new(r"\[domain\] => ([^\n]+)").unwrap();
        let password_regex = Regex::new(r"\[password\] => ([^\n]+)").unwrap();

        let entries: Vec<&str> = text.split("Array").collect();

        for entry in entries {
            let luser = luser_regex
                .captures(entry)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().trim());

            let domain = domain_regex
                .captures(entry)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().trim());

            let password = password_regex
                .captures(entry)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().trim());

            if let (Some(l), Some(d), Some(p)) = (luser, domain, password) {
                let email = format!("{}@{}", l, d);
                results.push(LeakResult {
                    email,
                    password: p.to_string(),
                });
            }
        }

        results
    }

    pub async fn check_haveibeenpwned(&self, email: &str) -> Result<Vec<String>> {
        self.core
            .logger
            .info("Searching information about Leaks found in HaveIBeenPwned");

        let hibp_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        let url = format!(
            "https://haveibeenpwned.com/api/v3/breachedaccount/{}",
            urlencoding::encode(email)
        );

        let response = hibp_client
            .get(&url)
            .header("User-Agent", "SocialOSINT-Rust/1.0")
            .header(
                "hibp-api-key",
                std::env::var("HIBP_API_KEY").unwrap_or_default(),
            )
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    let breaches: Vec<serde_json::Value> = resp.json().await?;
                    let breach_names: Vec<String> = breaches
                        .iter()
                        .filter_map(|b| b["Name"].as_str().map(|s| s.to_string()))
                        .collect();

                    if !breach_names.is_empty() {
                        self.core.logger.good(&format!(
                            "Found {} breaches for {}: {}",
                            breach_names.len(),
                            email,
                            breach_names.join(", ")
                        ));
                    } else {
                        self.core
                            .logger
                            .info(&format!("No breaches found for {}", email));
                    }

                    Ok(breach_names)
                } else if resp.status().as_u16() == 404 {
                    self.core
                        .logger
                        .info(&format!("No breaches found for {}", email));
                    Ok(Vec::new())
                } else if resp.status().as_u16() == 401 {
                    self.core.logger.bad("HIBP API key is missing or invalid. Set HIBP_API_KEY environment variable.");
                    Ok(Vec::new())
                } else if resp.status().as_u16() == 429 {
                    self.core
                        .logger
                        .bad("HIBP rate limit exceeded. Please wait before retrying.");
                    Ok(Vec::new())
                } else {
                    self.core
                        .logger
                        .bad(&format!("HIBP API error: {}", resp.status()));
                    Ok(Vec::new())
                }
            }
            Err(e) => {
                self.core
                    .logger
                    .bad(&format!("Failed to connect to HaveIBeenPwned: {}", e));
                Ok(Vec::new())
            }
        }
    }
}

pub async fn find_leaks(
    emails: Vec<(String, String, String)>,
    core: &Core,
) -> Result<Vec<PwnDBResult>> {
    let api = PwnDBAPI::new(core.clone());
    let mut results = Vec::new();

    for (user, user_id, email) in emails {
        let leaks = api.search_leak(&email).await?;

        if !leaks.is_empty() {
            results.push(PwnDBResult {
                user,
                user_id,
                leaks,
            });
        }

        let _ = api.check_haveibeenpwned(&email).await;

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    Ok(results)
}

pub fn save_results_pwndb(results: Vec<PwnDBResult>, core: &Core) -> Result<()> {
    use std::fs::OpenOptions;
    use std::io::Write;

    core.logger.info("Saving PwnDB results...");

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("PwnDBResults.txt")?;

    for result in results {
        writeln!(file, "User: {} Email: {}", result.user, result.user)?;
        for leak in result.leaks {
            writeln!(
                file,
                "    Leaks found in PwnDB: username: {}, password: {}",
                leak.email.split('@').next().unwrap_or(""),
                leak.password
            )?;
        }
        writeln!(file)?;
    }

    core.logger.good("PwnDB results saved to PwnDBResults.txt");

    Ok(())
}
