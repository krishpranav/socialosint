use socialosint::detectors::{Detectors, Platform, ProfileStatus};

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[tokio::test]
    async fn test_detect_blocking_403() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", "/blocked")
            .with_status(403)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/blocked", server.url()))
            .send()
            .await
            .unwrap();

        let detectors = Detectors::new();
        assert!(detectors.detect_blocking(&response));
    }

    #[tokio::test]
    async fn test_detect_blocking_429() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", "/ratelimit")
            .with_status(429)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/ratelimit", server.url()))
            .send()
            .await
            .unwrap();

        let detectors = Detectors::new();
        assert!(detectors.detect_blocking(&response));
    }

    #[tokio::test]
    async fn test_detect_blocking_cloudflare() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", "/cf")
            .with_status(200)
            .with_header("server", "cloudflare")
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/cf", server.url()))
            .send()
            .await
            .unwrap();

        let detectors = Detectors::new();
        assert!(detectors.detect_blocking(&response));
    }

    #[tokio::test]
    async fn test_detect_blocking_normal_request() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", "/normal")
            .with_status(200)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/normal", server.url()))
            .send()
            .await
            .unwrap();

        let detectors = Detectors::new();
        assert!(!detectors.detect_blocking(&response));
    }

    #[test]
    fn test_detect_soft_404_instagram() {
        let detectors = Detectors::new();

        let body = "Sorry, this page isn't available.";
        assert!(detectors.detect_soft_404(body, Platform::Instagram));

        let body = "User not found";
        assert!(detectors.detect_soft_404(body, Platform::Instagram));

        let body = "Welcome to Instagram";
        assert!(!detectors.detect_soft_404(body, Platform::Instagram));
    }

    #[test]
    fn test_detect_soft_404_linkedin() {
        let detectors = Detectors::new();

        let body = "Page not found";
        assert!(detectors.detect_soft_404(body, Platform::LinkedIn));

        let body = "Profile not found";
        assert!(detectors.detect_soft_404(body, Platform::LinkedIn));

        let body = "Welcome to LinkedIn";
        assert!(!detectors.detect_soft_404(body, Platform::LinkedIn));
    }

    #[test]
    fn test_detect_soft_404_twitter() {
        let detectors = Detectors::new();

        let body = "This account doesn't exist";
        assert!(detectors.detect_soft_404(body, Platform::Twitter));

        let body = "User not found";
        assert!(detectors.detect_soft_404(body, Platform::Twitter));

        let body = "Welcome to Twitter";
        assert!(!detectors.detect_soft_404(body, Platform::Twitter));
    }

    #[test]
    fn test_detect_private_instagram() {
        let detectors = Detectors::new();

        let body = "This account is private";
        assert!(detectors.detect_private(body, Platform::Instagram));

        let body = r#"{"is_private":true}"#;
        assert!(detectors.detect_private(body, Platform::Instagram));

        let body = "Public profile";
        assert!(!detectors.detect_private(body, Platform::Instagram));
    }

    #[test]
    fn test_detect_private_linkedin() {
        let detectors = Detectors::new();

        let body = "Sign in to view";
        assert!(detectors.detect_private(body, Platform::LinkedIn));

        let body = "Private profile";
        assert!(detectors.detect_private(body, Platform::LinkedIn));

        let body = "Public profile";
        assert!(!detectors.detect_private(body, Platform::LinkedIn));
    }

    #[test]
    fn test_detect_private_twitter() {
        let detectors = Detectors::new();

        let body = "These Tweets are protected";
        assert!(detectors.detect_private(body, Platform::Twitter));

        let body = "This account is private";
        assert!(detectors.detect_private(body, Platform::Twitter));

        let body = "Public tweets";
        assert!(!detectors.detect_private(body, Platform::Twitter));
    }

    #[test]
    fn test_detect_captcha() {
        let detectors = Detectors::new();

        assert!(detectors.detect_captcha("Please solve this CAPTCHA"));
        assert!(detectors.detect_captcha("reCAPTCHA verification required"));
        assert!(detectors.detect_captcha("hCaptcha challenge"));
        assert!(detectors.detect_captcha("Complete the challenge below"));

        assert!(!detectors.detect_captcha("Normal page content"));
    }

    #[tokio::test]
    async fn test_is_redirect() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", "/redirect")
            .with_status(301)
            .with_header("Location", "/new-location")
            .create_async()
            .await;

        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();
        let response = client
            .get(format!("{}/redirect", server.url()))
            .send()
            .await
            .unwrap();

        let detectors = Detectors::new();
        assert!(detectors.is_redirect(&response));
    }

    #[test]
    fn test_profile_status_variants() {
        let _exists = ProfileStatus::Exists;
        let _private = ProfileStatus::Private;
        let _not_found = ProfileStatus::NotFound;
        let _blocked = ProfileStatus::Blocked;
        let _rate_limited = ProfileStatus::RateLimited;
        let _error = ProfileStatus::Error("test".to_string());
    }

    #[test]
    fn test_platform_variants() {
        let _ig = Platform::Instagram;
        let _ln = Platform::LinkedIn;
        let _tw = Platform::Twitter;
        let _pw = Platform::PwnDB;
    }
}
