use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tweet {
    pub username: String,
    pub user_id: String,
    pub tweet: String,
}

#[derive(Debug, Clone)]
pub struct UserResult {
    pub user: String,
    pub user_id: String,
    pub email: String,
}

pub struct TwitterAPI {
    client: reqwest::Client,
}

impl TwitterAPI {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .cookie_store(true)
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    pub async fn get_tweets(
        &self,
        username: Option<&str>,
        hashtag: Option<&str>,
        limit: usize,
    ) -> Result<Vec<Tweet>> {
        let query = if let Some(user) = username {
            format!("from:{}", user)
        } else if let Some(tag) = hashtag {
            format!("#{}", tag)
        } else {
            return Ok(Vec::new());
        };

        crate::logger::info(&format!(
            "Searching Twitter for '{}' (limit: {})",
            query, limit
        ));

        let mut tweets = Vec::new();

        let search_url = format!(
            "https://api.twitter.com/2/tweets/search/recent?query={}&max_results={}",
            urlencoding::encode(&query),
            limit.min(100)
        );

        match self.try_twitter_api_search(&search_url).await {
            Ok(api_tweets) => {
                tweets.extend(api_tweets);
            }
            Err(_) => {
                crate::logger::bad("Twitter API access requires authentication. Trying alternative scraping method...");

                match self.try_nitter_scrape(username, hashtag, limit).await {
                    Ok(nitter_tweets) => {
                        tweets.extend(nitter_tweets);
                    }
                    Err(_) => {
                        crate::logger::bad(
                            "Alternative scraping failed. Twitter data collection requires:",
                        );
                        crate::logger::info("1. Twitter API credentials (Bearer token)");
                        crate::logger::info("2. Or a working Nitter instance");
                        crate::logger::info("3. Or a third-party scraping service");

                        return Ok(Vec::new());
                    }
                }
            }
        }

        crate::logger::good(&format!("Collected {} tweets", tweets.len()));
        Ok(tweets)
    }

    async fn try_twitter_api_search(&self, url: &str) -> Result<Vec<Tweet>> {
        let bearer_token =
            std::env::var("TWITTER_BEARER_TOKEN").context("TWITTER_BEARER_TOKEN not set")?;

        let response = self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {}", bearer_token))
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Twitter API returned status: {}", response.status());
        }

        let data: Value = response.json().await?;
        let mut tweets = Vec::new();

        if let Some(tweet_data) = data["data"].as_array() {
            for tweet in tweet_data {
                if let (Some(id), Some(text)) = (tweet["id"].as_str(), tweet["text"].as_str()) {
                    let author_id = tweet["author_id"].as_str().unwrap_or("unknown");

                    tweets.push(Tweet {
                        username: author_id.to_string(),
                        user_id: id.to_string(),
                        tweet: text.to_string(),
                    });
                }
            }
        }

        Ok(tweets)
    }

    async fn try_nitter_scrape(
        &self,
        username: Option<&str>,
        hashtag: Option<&str>,
        limit: usize,
    ) -> Result<Vec<Tweet>> {
        let nitter_instances = vec![
            "https://nitter.net",
            "https://nitter.poast.org",
            "https://nitter.privacydev.net",
        ];

        for instance in nitter_instances {
            let url = if let Some(user) = username {
                format!("{}/{}", instance, user)
            } else if let Some(tag) = hashtag {
                format!("{}/search?q=%23{}", instance, tag)
            } else {
                continue;
            };

            match self.scrape_nitter_page(&url, limit).await {
                Ok(tweets) if !tweets.is_empty() => {
                    crate::logger::good(&format!("Successfully scraped from {}", instance));
                    return Ok(tweets);
                }
                _ => continue,
            }
        }

        anyhow::bail!("All Nitter instances failed")
    }

    async fn scrape_nitter_page(&self, url: &str, limit: usize) -> Result<Vec<Tweet>> {
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Nitter returned status: {}", response.status());
        }

        let html = response.text().await?;
        let mut tweets = Vec::new();

        let tweet_regex = Regex::new(r#"<div class="tweet-content[^"]*">([^<]+)</div>"#)?;
        let username_regex = Regex::new(r#"<a class="username"[^>]*>@([^<]+)</a>"#)?;

        let tweet_texts: Vec<String> = tweet_regex
            .captures_iter(&html)
            .map(|cap| cap[1].to_string())
            .take(limit)
            .collect();

        let usernames: Vec<String> = username_regex
            .captures_iter(&html)
            .map(|cap| cap[1].to_string())
            .take(limit)
            .collect();

        for (idx, (text, user)) in tweet_texts.iter().zip(usernames.iter()).enumerate() {
            tweets.push(Tweet {
                username: user.clone(),
                user_id: idx.to_string(),
                tweet: text.clone(),
            });
        }

        Ok(tweets)
    }

    pub async fn get_followers(&self, username: &str, limit: usize) -> Result<Vec<String>> {
        crate::logger::info(&format!("Getting followers of user: {}", username));

        let bearer_token = match std::env::var("TWITTER_BEARER_TOKEN") {
            Ok(token) => token,
            Err(_) => {
                crate::logger::bad("TWITTER_BEARER_TOKEN not set - cannot fetch followers");
                return Ok(Vec::new());
            }
        };

        let user_id = self.get_user_id(username, &bearer_token).await?;

        let url = format!(
            "https://api.twitter.com/2/users/{}/followers?max_results={}",
            user_id,
            limit.min(1000)
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", bearer_token))
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Twitter API returned status: {}", response.status());
        }

        let data: Value = response.json().await?;
        let mut followers = Vec::new();

        if let Some(users) = data["data"].as_array() {
            for user in users {
                if let Some(username) = user["username"].as_str() {
                    followers.push(username.to_string());
                }
            }
        }

        crate::logger::good(&format!("Found {} followers", followers.len()));
        Ok(followers)
    }

    pub async fn get_followings(&self, username: &str, limit: usize) -> Result<Vec<String>> {
        crate::logger::info(&format!("Getting followings of user: {}", username));

        let bearer_token = match std::env::var("TWITTER_BEARER_TOKEN") {
            Ok(token) => token,
            Err(_) => {
                crate::logger::bad("TWITTER_BEARER_TOKEN not set - cannot fetch followings");
                return Ok(Vec::new());
            }
        };

        let user_id = self.get_user_id(username, &bearer_token).await?;

        let url = format!(
            "https://api.twitter.com/2/users/{}/following?max_results={}",
            user_id,
            limit.min(1000)
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", bearer_token))
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Twitter API returned status: {}", response.status());
        }

        let data: Value = response.json().await?;
        let mut followings = Vec::new();

        if let Some(users) = data["data"].as_array() {
            for user in users {
                if let Some(username) = user["username"].as_str() {
                    followings.push(username.to_string());
                }
            }
        }

        crate::logger::good(&format!("Found {} followings", followings.len()));
        Ok(followings)
    }

    async fn get_user_id(&self, username: &str, bearer_token: &str) -> Result<String> {
        let url = format!("https://api.twitter.com/2/users/by/username/{}", username);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", bearer_token))
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get user ID for {}", username);
        }

        let data: Value = response.json().await?;

        data["data"]["id"]
            .as_str()
            .map(|s| s.to_string())
            .context("User ID not found in response")
    }
}

pub fn extract_email_from_tweet(tweet: &str) -> Option<String> {
    let email_regex = Regex::new(r"([a-zA-Z0-9._-]+@[a-zA-Z0-9._-]+\.[a-zA-Z]{2,15})").unwrap();

    if let Some(captures) = email_regex.captures(tweet) {
        return Some(captures[1].to_string());
    }

    None
}

pub fn extract_emails_from_tweets(tweets: Vec<Tweet>) -> Vec<UserResult> {
    let mut results = Vec::new();

    for tweet in tweets {
        if let Some(email) = extract_email_from_tweet(&tweet.tweet) {
            crate::logger::good(&format!(
                "Username: {} UserID: {} Email: {}",
                tweet.username, tweet.user_id, email
            ));

            results.push(UserResult {
                user: tweet.username,
                user_id: tweet.user_id,
                email,
            });
        }
    }

    results.sort_by(|a, b| a.email.cmp(&b.email));
    results.dedup_by(|a, b| a.email == b.email);

    results
}
