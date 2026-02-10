use crate::cli::Args;
use crate::detectors::Detectors;
use crate::http::HttpClient;
use crate::logger::Logger;
use crate::rate_limiter::RateLimiter;
use crate::scraper::Scraper;
use crate::tui::{Status, TuiManager};
use crate::{instagram, linkedin, pwndb, twitter};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::Arc;

#[derive(Clone)]
pub struct Core {
    pub http: Arc<HttpClient>,
    pub scraper: Arc<Scraper>,
    pub detectors: Arc<Detectors>,
    pub rate_limiter: Arc<RateLimiter>,
    pub logger: Arc<Logger>,
    pub tui: Arc<TuiManager>,
}

impl Core {
    pub fn new(proxy: Option<String>) -> Result<Self> {
        let http = Arc::new(HttpClient::new(proxy)?);
        let detectors = Arc::new(Detectors::new());
        let rate_limiter = Arc::new(RateLimiter::new(10));
        let scraper = Arc::new(Scraper::new(http.clone(), detectors.clone()));
        let logger = Arc::new(Logger::new());
        let tui = Arc::new(TuiManager::new());

        Ok(Self {
            http,
            scraper,
            detectors,
            rate_limiter,
            logger,
            tui,
        })
    }
}

#[derive(Debug, Deserialize)]
struct Credentials {
    instagram: Option<InstagramCreds>,
    linkedin: Option<LinkedInCreds>,
}

#[derive(Debug, Deserialize)]
struct InstagramCreds {
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct LinkedInCreds {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct OutputResult {
    user: String,
    #[serde(rename = "userID")]
    user_id: String,
    email: String,
}

pub async fn run(args: Args) -> Result<()> {
    let start_time = std::time::Instant::now();

    let proxy = if args.pwndb {
        Some(args.tor_proxy.clone())
    } else {
        None
    };
    let core = Core::new(proxy)?;

    let creds = read_credentials(&args.credentials)?;

    if let Some(output) = &args.output {
        if !output.exists() {
            fs::File::create(output)?;
        }
    }

    if args.pwndb {
        core.logger
            .info("PwnDB enabled - ensure Tor service is running");
    }

    let mut all_results = Vec::new();

    if args.instagram {
        core.tui.update_instagram_status(Status::InProgress);

        match instagram_parameters(&args, &creds, &core).await {
            Ok(results) => {
                core.tui.update_instagram_count(results.len());
                core.tui.update_instagram_status(if results.is_empty() {
                    Status::Failed
                } else {
                    Status::Success
                });
                all_results.extend(results.into_iter().map(|r| (r.user, r.user_id, r.email)));
            }
            Err(e) => {
                core.logger.bad(&format!("Instagram error: {}", e));
                core.tui.update_instagram_status(Status::Failed);
            }
        }
    }

    if args.linkedin {
        core.tui.update_linkedin_status(Status::InProgress);

        match linkedin_parameters(&args, &creds, &core).await {
            Ok(results) => {
                core.tui.update_linkedin_count(results.len());
                core.tui.update_linkedin_status(if results.is_empty() {
                    Status::Failed
                } else {
                    Status::Success
                });
                all_results.extend(results.into_iter().map(|r| (r.user, r.user_id, r.email)));
            }
            Err(e) => {
                core.logger.bad(&format!("LinkedIn error: {}", e));
                core.tui.update_linkedin_status(Status::Failed);
            }
        }
    }

    if args.twitter {
        core.tui.update_twitter_status(Status::InProgress);

        match twitter_parameters(&args, &core).await {
            Ok(results) => {
                core.tui.update_twitter_count(results.len());
                core.tui.update_twitter_status(if results.is_empty() {
                    Status::Failed
                } else {
                    Status::Success
                });
                all_results.extend(results.into_iter().map(|r| (r.user, r.user_id, r.email)));
            }
            Err(e) => {
                core.logger.bad(&format!("Twitter error: {}", e));
                core.tui.update_twitter_status(Status::Failed);
            }
        }
    }

    if let Some(output) = &args.output {
        save_results(output, &all_results, &core)?;
    }

    if args.pwndb && !all_results.is_empty() {
        match pwndb::find_leaks(all_results.clone(), &core).await {
            Ok(leaks) => {
                pwndb::save_results_pwndb(leaks, &core)?;
            }
            Err(e) => {
                core.logger.bad(&format!("PwnDB error: {}", e));
            }
        }
    } else if args.pwndb && all_results.is_empty() {
        core.logger.info("No emails were found to search.");
    }

    let elapsed = start_time.elapsed().as_secs_f64();
    core.tui.display_summary(elapsed);

    Ok(())
}

async fn instagram_parameters(
    args: &Args,
    creds: &Credentials,
    core: &Core,
) -> Result<Vec<instagram::UserResult>> {
    let ig_creds = creds
        .instagram
        .as_ref()
        .context("Instagram credentials not found in credentials.json")?;

    let mut api = instagram::InstagramAPI::new(
        core.clone(),
        ig_creds.username.clone(),
        ig_creds.password.clone(),
    );

    if !api.login().await? {
        anyhow::bail!("Can't Login to Instagram!");
    }

    core.logger.good("Successful login to Instagram!\n");

    let mut results = Vec::new();

    if let Some(query) = &args.info {
        let locations = api.search_location(query).await?;
        for location in locations {
            if let Some(loc) = location["location"].as_object() {
                let name = loc["name"].as_str().unwrap_or("Unknown");
                let pk = loc["pk"].as_u64().unwrap_or(0);
                core.logger
                    .good(&format!("City: {} Search ID: {}", name, pk));
            }
        }
    }

    if let Some(hashtag) = &args.hashtag_ig {
        let items = api.get_hashtag_feed(hashtag).await?;
        let users = extract_users_from_items(items);
        let user_results = fetch_user_profiles(&api, users).await;
        results.extend(instagram::extract_emails_from_users(user_results, core));
    }

    if let Some(location_id) = &args.location {
        let items = api.get_location_feed(location_id).await?;
        let users = extract_users_from_items(items);
        let user_results = fetch_user_profiles(&api, users).await;
        results.extend(instagram::extract_emails_from_users(user_results, core));
    }

    if let Some(target) = &args.target_ig {
        match api.search_username(target).await {
            Ok(user_response) => {
                let user_pk = user_response.user.pk.to_string();
                results.extend(instagram::extract_emails_from_users(
                    vec![user_response.user],
                    core,
                ));

                if args.followers_ig {
                    let followers = api.get_user_followers(&user_pk, None).await?;
                    results.extend(instagram::extract_emails_from_users(followers, core));
                }
                if args.followings_ig {
                    let followings = api.get_user_followings(&user_pk, None).await?;
                    results.extend(instagram::extract_emails_from_users(followings, core));
                }
            }
            Err(e) => {
                core.logger.info(&format!(
                    "The user has a private profile or doesn't exist: {}",
                    e
                ));
            }
        }
    }

    if let Some(query) = &args.search_users_ig {
        let users = api.search_users(query).await?;
        results.extend(instagram::extract_emails_from_users(users, core));
    }

    let mut seen = HashSet::new();
    results.retain(|r| seen.insert(r.email.clone()));

    Ok(results)
}

async fn linkedin_parameters(
    args: &Args,
    creds: &Credentials,
    core: &Core,
) -> Result<Vec<linkedin::UserResult>> {
    let ln_creds = creds
        .linkedin
        .as_ref()
        .context("LinkedIn credentials not found in credentials.json")?;

    let mut api = linkedin::LinkedInAPI::new(
        core.clone(),
        ln_creds.email.clone(),
        ln_creds.password.clone(),
    );

    if !api.authenticate().await? {
        anyhow::bail!("Can't Login to LinkedIn!");
    }

    core.logger.good("Successful login to LinkedIn!\n");

    let mut results = Vec::new();

    if let Some(query) = &args.search_companies {
        let companies = api.search_companies(query).await?;

        for company in &companies {
            core.logger.good(&format!(
                "Name: {} company ID: {} Number of employees: {}",
                company.name,
                company.urn_id,
                company.subline.as_ref().unwrap_or(&String::new())
            ));
        }

        if args.employees {
            for company in companies {
                let employees = api.search_people(Some(&company.name)).await?;
                let user_results = linkedin::extract_emails_from_users(&api, employees, core).await;
                results.extend(user_results);
            }
        }
    }

    if let Some(query) = &args.search_users_in {
        let users = api.search_people(Some(query)).await?;

        for user in &users {
            core.logger
                .good(&format!("User: {} userID: {}", user.public_id, user.urn_id));
        }

        if args.pwndb {
            let user_results = linkedin::extract_emails_from_users(&api, users, core).await;
            results.extend(user_results);
        }
    }

    if let Some(target) = &args.target_in {
        if let Ok(contact_info) = api.get_profile_contact_info(target).await {
            if let Some(email) = contact_info.email_address {
                results.push(linkedin::UserResult {
                    user: target.clone(),
                    user_id: "Not-Found".to_string(),
                    email,
                });
            }
        }
    }

    let mut seen = HashSet::new();
    results.retain(|r| seen.insert(r.email.clone()));

    Ok(results)
}

async fn twitter_parameters(args: &Args, core: &Core) -> Result<Vec<twitter::UserResult>> {
    core.logger.good("Using Twitter!\n");

    let api = twitter::TwitterAPI::new(core.clone());
    let mut results = Vec::new();

    if let Some(target) = &args.target_tw {
        let tweets = api.get_tweets(Some(target), None, args.limit).await?;
        results.extend(twitter::extract_emails_from_tweets(tweets, core));
    }

    if let Some(hashtag) = &args.hashtag_tw {
        let tweets = api.get_tweets(None, Some(hashtag), args.limit).await?;
        results.extend(twitter::extract_emails_from_tweets(tweets, core));
    }

    let mut seen = HashSet::new();
    results.retain(|r| seen.insert(r.email.clone()));

    Ok(results)
}

fn read_credentials(path: &Path) -> Result<Credentials> {
    let content = fs::read_to_string(path).context("Failed to read credentials file")?;
    let creds: Credentials = serde_json::from_str(&content).context("Incorrect JSON format")?;
    Ok(creds)
}

fn save_results(path: &Path, results: &[(String, String, String)], core: &Core) -> Result<()> {
    use std::io::Write;

    core.logger.info("Writing the file...");

    let existing_content = fs::read_to_string(path).unwrap_or_default();
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;

    for (user, user_id, email) in results {
        if !existing_content.contains(email) {
            writeln!(file, "{}:{}:{}", user, user_id, email)?;
        }
    }

    core.logger.good("Correctly saved information...\n");

    Ok(())
}

fn extract_users_from_items(items: Vec<serde_json::Value>) -> Vec<String> {
    items
        .iter()
        .filter_map(|item| item["user"]["username"].as_str().map(|s| s.to_string()))
        .collect()
}

async fn fetch_user_profiles(
    api: &instagram::InstagramAPI,
    usernames: Vec<String>,
) -> Vec<instagram::InstagramUser> {
    let mut users = Vec::new();

    for username in usernames {
        if let Ok(response) = api.search_username(&username).await {
            users.push(response.user);
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    users
}
