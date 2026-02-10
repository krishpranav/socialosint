use crate::core::Core;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

const API_BASE_URL: &str = "https://www.linkedin.com/voyager/api";
const AUTH_BASE_URL: &str = "https://www.linkedin.com";

#[derive(Debug, Serialize, Deserialize)]
pub struct LinkedInProfile {
    pub public_id: String,
    pub urn_id: String,
    pub distance: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContactInfo {
    pub email_address: Option<String>,
    pub phone_numbers: Vec<PhoneNumber>,
    pub twitter: Vec<TwitterHandle>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PhoneNumber {
    pub number: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwitterHandle {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Company {
    pub urn_id: String,
    pub name: String,
    pub headline: Option<String>,
    pub subline: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UserResult {
    pub user: String,
    pub user_id: String,
    pub email: String,
}

pub struct LinkedInAPI {
    core: Core,
    email: String,
    password: String,
    csrf_token: Option<String>,
}

impl LinkedInAPI {
    pub fn new(core: Core, email: String, password: String) -> Self {
        Self {
            core,
            email,
            password,
            csrf_token: None,
        }
    }

    fn user_agent() -> &'static str {
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_13_5) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/66.0.3359.181 Safari/537.36"
    }

    pub async fn authenticate(&mut self) -> Result<bool> {
        let auth_url = format!("{}/uas/authenticate", AUTH_BASE_URL);

        let session_url = format!("{}/uas/login", AUTH_BASE_URL);
        self.core
            .http
            .raw_client()
            .get(&session_url)
            .header("User-Agent", Self::user_agent())
            .send()
            .await?;

        let payload = json!({
            "session_key": self.email,
            "session_password": self.password,
        });

        let response = self
            .core
            .http
            .raw_client()
            .post(&auth_url)
            .header("User-Agent", Self::user_agent())
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(false);
        }

        let csrf_token = response
            .cookies()
            .find(|c| c.name() == "JSESSIONID")
            .map(|c| c.value().trim_matches('"').to_string());

        let json: serde_json::Value = response.json().await?;
        if json["login_result"] != "PASS" {
            return Ok(false);
        }

        self.csrf_token = csrf_token;
        Ok(true)
    }

    pub async fn search_people(&self, query: Option<&str>) -> Result<Vec<LinkedInProfile>> {
        let search_query = query.unwrap_or("");
        let url = format!(
            "{}/search/blended?keywords={}&origin=GLOBAL_SEARCH_HEADER",
            API_BASE_URL, search_query
        );

        let response = self
            .core
            .http
            .raw_client()
            .get(&url)
            .header("User-Agent", Self::user_agent())
            .header("x-li-lang", "en_US")
            .header("x-restli-protocol-version", "2.0.0")
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        let mut profiles = Vec::new();

        if let Some(elements) = json["elements"].as_array() {
            for element in elements {
                if element["type"] == "PROFILE" {
                    if let Some(profile_data) = element["profile"].as_object() {
                        let public_id = profile_data["publicIdentifier"]
                            .as_str()
                            .unwrap_or("")
                            .to_string();
                        let urn_id = profile_data["entityUrn"].as_str().unwrap_or("").to_string();

                        profiles.push(LinkedInProfile {
                            public_id,
                            urn_id,
                            distance: None,
                        });
                    }
                }
            }
        }

        Ok(profiles)
    }

    pub async fn search_companies(&self, query: &str) -> Result<Vec<Company>> {
        let url = format!(
            "{}/search/blended?keywords={}&origin=GLOBAL_SEARCH_HEADER",
            API_BASE_URL, query
        );

        let response = self
            .core
            .http
            .raw_client()
            .get(&url)
            .header("User-Agent", Self::user_agent())
            .header("x-li-lang", "en_US")
            .header("x-restli-protocol-version", "2.0.0")
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        let mut companies = Vec::new();

        if let Some(elements) = json["elements"].as_array() {
            for element in elements {
                if element["type"] == "COMPANY" {
                    if let Some(company_data) = element["company"].as_object() {
                        let name = company_data["name"].as_str().unwrap_or("").to_string();
                        let urn_id = company_data["entityUrn"].as_str().unwrap_or("").to_string();

                        companies.push(Company {
                            urn_id,
                            name,
                            headline: None,
                            subline: None,
                        });
                    }
                }
            }
        }

        Ok(companies)
    }

    pub async fn get_profile_contact_info(&self, public_id: &str) -> Result<ContactInfo> {
        let url = format!(
            "{}/identity/profiles/{}/profileContactInfo",
            API_BASE_URL, public_id
        );

        let response = self
            .core
            .http
            .raw_client()
            .get(&url)
            .header("User-Agent", Self::user_agent())
            .header("x-li-lang", "en_US")
            .header("x-restli-protocol-version", "2.0.0")
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;

        let email_address = json["emailAddress"].as_str().map(|s| s.to_string());

        Ok(ContactInfo {
            email_address,
            phone_numbers: Vec::new(),
            twitter: Vec::new(),
        })
    }
}

pub async fn extract_emails_from_users(
    api: &LinkedInAPI,
    profiles: Vec<LinkedInProfile>,
    core: &Core,
) -> Vec<UserResult> {
    let mut results = Vec::new();

    for profile in profiles {
        match api.get_profile_contact_info(&profile.public_id).await {
            Ok(contact_info) => {
                if let Some(email) = contact_info.email_address {
                    core.logger.good(&format!(
                        "User: {} UserID: {} Email: {}",
                        profile.public_id, profile.urn_id, email
                    ));

                    results.push(UserResult {
                        user: profile.public_id,
                        user_id: profile.urn_id,
                        email,
                    });
                }
            }
            Err(_) => {
                continue;
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }

    results
}
