use crate::core::Core;
use anyhow::{Context, Result};
use hmac::{Hmac, Mac};
use md5;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Sha256;
use uuid::Uuid;

const API_URL: &str = "https://i.instagram.com/api/v1/";
const IG_SIG_KEY: &str = "4f8732eb9ba7d1c8e8897a75d6474d4eb3f5279137431b2aafb71fafe2abe178";
const SIG_KEY_VERSION: &str = "4";

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Serialize, Deserialize)]
pub struct InstagramUser {
    pub username: String,
    pub pk: u64,
    pub public_email: Option<String>,
    pub follower_count: u64,
    pub following_count: u64,
    pub biography: String,
    pub is_private: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub status: String,
    pub user: InstagramUser,
}

#[derive(Debug, Clone)]
pub struct UserResult {
    pub user: String,
    pub user_id: String,
    pub email: String,
    pub private: bool,
}

pub struct InstagramAPI {
    core: Core,
    username: String,
    password: String,
    uuid: String,
    device_id: String,
    username_id: Option<String>,
    token: Option<String>,
    rank_token: Option<String>,
    is_logged_in: bool,
}

impl InstagramAPI {
    pub fn new(core: Core, username: String, password: String) -> Self {
        let combined = format!("{}{}", username, password);
        let hash = format!("{:x}", md5::compute(combined.as_bytes()));
        let device_id = Self::generate_device_id(&hash);
        let uuid = Self::generate_uuid();

        Self {
            core,
            username,
            password,
            uuid,
            device_id,
            username_id: None,
            token: None,
            rank_token: None,
            is_logged_in: false,
        }
    }

    pub fn generate_uuid() -> String {
        Uuid::new_v4().to_string()
    }

    pub fn generate_device_id(seed: &str) -> String {
        format!("android-{}", &seed[0..16])
    }

    fn generate_signature(&self, data: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(IG_SIG_KEY.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(data.as_bytes());
        let result = mac.finalize();
        let signature = hex::encode(result.into_bytes());

        format!(
            "ig_sig_key_version={}&signed_body={}.{}",
            SIG_KEY_VERSION, signature, data
        )
    }

    pub fn user_agent() -> &'static str {
        "Instagram 10.26.0 Android (18/4.3; 320dpi; 720x1280; Xiaomi; HM 1SW; armani; qcom; en_US)"
    }

    pub async fn login(&mut self) -> Result<bool> {
        if self.is_logged_in {
            return Ok(true);
        }

        let url = format!(
            "{}si/fetch_headers/?challenge_type=signup&guid={}",
            API_URL, self.uuid
        );

        let response = self
            .core
            .http
            .raw_client()
            .get(&url)
            .header("User-Agent", Self::user_agent())
            .send()
            .await?;

        let csrf_token = response
            .cookies()
            .find(|c| c.name() == "csrftoken")
            .map(|c| c.value().to_string())
            .context("No CSRF token")?;

        let login_data = json!({
            "phone_id": self.uuid,
            "_csrftoken": csrf_token,
            "username": self.username,
            "guid": self.uuid,
            "device_id": self.device_id,
            "password": self.password,
            "login_attempt_count": "0"
        });

        let signed_body = self.generate_signature(&login_data.to_string());
        let login_url = format!("{}accounts/login/", API_URL);

        let response = self
            .core
            .http
            .raw_client()
            .post(&login_url)
            .header("User-Agent", Self::user_agent())
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(signed_body)
            .send()
            .await?;

        if response.status().is_success() {
            let json: serde_json::Value = response.json().await?;

            self.username_id = json["logged_in_user"]["pk"]
                .as_u64()
                .map(|id| id.to_string());

            if let Some(uid) = &self.username_id {
                self.rank_token = Some(format!("{}_{}", uid, self.uuid));
            }

            self.token = Some(csrf_token);
            self.is_logged_in = true;

            self.sync_features().await?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn sync_features(&self) -> Result<()> {
        let data = json!({
            "_uuid": self.uuid,
            "_uid": self.username_id,
            "id": self.username_id,
            "_csrftoken": self.token,
            "experiments": ""
        });

        let signed_body = self.generate_signature(&data.to_string());
        let url = format!("{}qe/sync/", API_URL);

        self.core
            .http
            .raw_client()
            .post(&url)
            .header("User-Agent", Self::user_agent())
            .body(signed_body)
            .send()
            .await?;

        Ok(())
    }

    pub async fn search_username(&self, username: &str) -> Result<UserResponse> {
        let url = format!("{}users/{}/usernameinfo/", API_URL, username);

        let response = self
            .core
            .http
            .raw_client()
            .get(&url)
            .header("User-Agent", Self::user_agent())
            .send()
            .await?;

        if response.status().as_u16() == 429 {
            self.core
                .logger
                .bad(&format!("Rate limited for user: {}", username));
        }

        let user_response: UserResponse = response.json().await?;
        Ok(user_response)
    }

    pub async fn search_users(&self, query: &str) -> Result<Vec<InstagramUser>> {
        let url = format!(
            "{}users/search/?ig_sig_key_version={}&is_typeahead=true&query={}&rank_token={}",
            API_URL,
            SIG_KEY_VERSION,
            query,
            self.rank_token.as_ref().unwrap_or(&String::new())
        );

        let response = self
            .core
            .http
            .raw_client()
            .get(&url)
            .header("User-Agent", Self::user_agent())
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        let users: Vec<InstagramUser> = serde_json::from_value(json["users"].clone())?;

        Ok(users)
    }

    pub async fn get_hashtag_feed(&self, hashtag: &str) -> Result<Vec<serde_json::Value>> {
        let url = format!(
            "{}feed/tag/{}/?rank_token={}&ranked_content=true",
            API_URL,
            hashtag,
            self.rank_token.as_ref().unwrap_or(&String::new())
        );

        let response = self
            .core
            .http
            .raw_client()
            .get(&url)
            .header("User-Agent", Self::user_agent())
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        let items: Vec<serde_json::Value> = serde_json::from_value(json["items"].clone())?;

        Ok(items)
    }

    pub async fn search_location(&self, query: &str) -> Result<Vec<serde_json::Value>> {
        let url = format!(
            "{}fbsearch/places/?rank_token={}&query={}",
            API_URL,
            self.rank_token.as_ref().unwrap_or(&String::new()),
            query
        );

        let response = self
            .core
            .http
            .raw_client()
            .get(&url)
            .header("User-Agent", Self::user_agent())
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        let items: Vec<serde_json::Value> = serde_json::from_value(json["items"].clone())?;

        Ok(items)
    }

    pub async fn get_location_feed(&self, location_id: &str) -> Result<Vec<serde_json::Value>> {
        let url = format!(
            "{}feed/location/{}/?rank_token={}&ranked_content=true",
            API_URL,
            location_id,
            self.rank_token.as_ref().unwrap_or(&String::new())
        );

        let response = self
            .core
            .http
            .raw_client()
            .get(&url)
            .header("User-Agent", Self::user_agent())
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        let items: Vec<serde_json::Value> = serde_json::from_value(json["items"].clone())?;

        Ok(items)
    }

    pub async fn get_user_followers(
        &self,
        user_id: &str,
        max_id: Option<String>,
    ) -> Result<Vec<InstagramUser>> {
        let url = if let Some(max_id) = max_id {
            format!(
                "{}friendships/{}/followers/?rank_token={}&max_id={}",
                API_URL,
                user_id,
                self.rank_token.as_ref().unwrap_or(&String::new()),
                max_id
            )
        } else {
            format!(
                "{}friendships/{}/followers/?rank_token={}",
                API_URL,
                user_id,
                self.rank_token.as_ref().unwrap_or(&String::new())
            )
        };

        let response = self
            .core
            .http
            .raw_client()
            .get(&url)
            .header("User-Agent", Self::user_agent())
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        let users: Vec<InstagramUser> = serde_json::from_value(json["users"].clone())?;

        Ok(users)
    }

    pub async fn get_user_followings(
        &self,
        user_id: &str,
        max_id: Option<String>,
    ) -> Result<Vec<InstagramUser>> {
        let url = if let Some(max_id) = max_id {
            format!(
                "{}friendships/{}/following/?ig_sig_key_version={}&rank_token={}&max_id={}",
                API_URL,
                user_id,
                SIG_KEY_VERSION,
                self.rank_token.as_ref().unwrap_or(&String::new()),
                max_id
            )
        } else {
            format!(
                "{}friendships/{}/following/?ig_sig_key_version={}&rank_token={}",
                API_URL,
                user_id,
                SIG_KEY_VERSION,
                self.rank_token.as_ref().unwrap_or(&String::new())
            )
        };

        let response = self
            .core
            .http
            .raw_client()
            .get(&url)
            .header("User-Agent", Self::user_agent())
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        let users: Vec<InstagramUser> = serde_json::from_value(json["users"].clone())?;

        Ok(users)
    }
}

pub fn extract_email_from_bio(bio: &str) -> Option<String> {
    let email_regex =
        regex::Regex::new(r"^[(a-z0-9\_\-\.)]+@[(a-z0-9\_\-\.)]+\.[(a-z)]{2,15}$").unwrap();

    for word in bio.split_whitespace() {
        if email_regex.is_match(word) {
            return Some(word.to_string());
        }
    }

    None
}

pub fn extract_emails_from_users(users: Vec<InstagramUser>, core: &Core) -> Vec<UserResult> {
    let mut results = Vec::new();

    for user in users {
        let email = if let Some(public_email) = user.public_email {
            Some(public_email)
        } else {
            extract_email_from_bio(&user.biography)
        };

        if let Some(email) = email {
            core.logger.good(&format!(
                "Username: {} UserID: {} Email: {} Followers: {} Following: {}",
                user.username, user.pk, email, user.follower_count, user.following_count
            ));

            results.push(UserResult {
                user: user.username,
                user_id: user.pk.to_string(),
                email,
                private: user.is_private,
            });
        }
    }

    results.sort_by(|a, b| a.email.cmp(&b.email));
    results
}
