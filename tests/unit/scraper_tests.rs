use socialosint::detectors::{Detectors, Platform, ProfileStatus};
use socialosint::http::HttpClient;
use socialosint::scraper::Scraper;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;
    use reqwest::header::HeaderMap;

    fn create_test_scraper() -> Scraper {
        let http = Arc::new(HttpClient::new(None).unwrap());
        let detectors = Arc::new(Detectors::new());
        Scraper::new(http, detectors)
    }

    #[tokio::test]
    async fn test_scraper_creation() {
        let scraper = create_test_scraper();
        assert!(std::mem::size_of_val(&scraper) > 0);
    }

    #[tokio::test]
    async fn test_get_json_success() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"status": "ok", "data": [1,2,3]}"#)
            .create_async()
            .await;

        let scraper = create_test_scraper();
        let url = format!("{}/api/data", server.url());

        let result = scraper.get_json(&url, HeaderMap::new()).await;
        assert!(result.is_ok());

        let json = result.unwrap();
        assert_eq!(json["status"], "ok");
        assert_eq!(json["data"][0], 1);
    }

    #[tokio::test]
    async fn test_post_json_success() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/api/create")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(r#"{"created": true, "id": 123}"#)
            .create_async()
            .await;

        let scraper = create_test_scraper();
        let url = format!("{}/api/create", server.url());

        let result = scraper
            .post_json(&url, HeaderMap::new(), r#"{"name": "test"}"#.to_string())
            .await;
        assert!(result.is_ok());

        let json = result.unwrap();
        assert_eq!(json["created"], true);
        assert_eq!(json["id"], 123);
    }

    #[tokio::test]
    async fn test_get_json_blocked() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", "/blocked")
            .with_status(403)
            .create_async()
            .await;

        let scraper = create_test_scraper();
        let url = format!("{}/blocked", server.url());

        let result = scraper.get_json(&url, HeaderMap::new()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("blocked"));
    }

    #[tokio::test]
    async fn test_check_profile_exists_found() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", "/user/john")
            .with_status(200)
            .with_body("User profile for john")
            .create_async()
            .await;

        let scraper = create_test_scraper();
        let url = format!("{}/user/john", server.url());

        let result = scraper
            .check_profile_exists(&url, Platform::Instagram)
            .await;
        assert!(result.is_ok());

        match result.unwrap() {
            ProfileStatus::Exists => {}
            _ => panic!("Expected ProfileStatus::Exists"),
        }
    }

    #[tokio::test]
    async fn test_check_profile_not_found() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", "/user/nonexistent")
            .with_status(404)
            .create_async()
            .await;

        let scraper = create_test_scraper();
        let url = format!("{}/user/nonexistent", server.url());

        let result = scraper
            .check_profile_exists(&url, Platform::Instagram)
            .await;
        assert!(result.is_ok());

        match result.unwrap() {
            ProfileStatus::NotFound => {}
            _ => panic!("Expected ProfileStatus::NotFound"),
        }
    }

    #[tokio::test]
    async fn test_check_profile_private() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", "/user/private")
            .with_status(200)
            .with_body("This account is private")
            .create_async()
            .await;

        let scraper = create_test_scraper();
        let url = format!("{}/user/private", server.url());

        let result = scraper
            .check_profile_exists(&url, Platform::Instagram)
            .await;
        assert!(result.is_ok());

        match result.unwrap() {
            ProfileStatus::Private => {}
            _ => panic!("Expected ProfileStatus::Private"),
        }
    }

    #[tokio::test]
    async fn test_check_profile_rate_limited() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", "/user/test")
            .with_status(429)
            .create_async()
            .await;

        let scraper = create_test_scraper();
        let url = format!("{}/user/test", server.url());

        let result = scraper
            .check_profile_exists(&url, Platform::Instagram)
            .await;
        assert!(result.is_ok());

        match result.unwrap() {
            ProfileStatus::RateLimited => {}
            _ => panic!("Expected ProfileStatus::RateLimited"),
        }
    }

    #[tokio::test]
    async fn test_retry_with_backoff_success() {
        let scraper = create_test_scraper();
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));

        let counter_clone = counter.clone();
        let result = scraper
            .retry_with_backoff(
                move || {
                    let c = counter_clone.clone();
                    async move {
                        let attempts = c.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                        if attempts < 3 {
                            Err(anyhow::anyhow!("Temporary error"))
                        } else {
                            Ok("Success".to_string())
                        }
                    }
                },
                5,
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success");
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_with_backoff_max_retries() {
        let scraper = create_test_scraper();
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));

        let counter_clone = counter.clone();
        let result = scraper
            .retry_with_backoff(
                move || {
                    let c = counter_clone.clone();
                    async move {
                        c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        Err::<String, _>(anyhow::anyhow!("Persistent error"))
                    }
                },
                3,
            )
            .await;

        assert!(result.is_err());
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_get_raw() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", "/raw")
            .with_status(200)
            .with_body("Raw response")
            .create_async()
            .await;

        let scraper = create_test_scraper();
        let url = format!("{}/raw", server.url());

        let result = scraper.get_raw(&url, HeaderMap::new()).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.status(), 200);
    }

    #[tokio::test]
    async fn test_post_raw() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/raw")
            .with_status(201)
            .with_body("Created")
            .create_async()
            .await;

        let scraper = create_test_scraper();
        let url = format!("{}/raw", server.url());

        let result = scraper
            .post_raw(&url, HeaderMap::new(), "test data".to_string())
            .await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.status(), 201);
    }
}
