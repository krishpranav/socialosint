use socialosint::core::Core;

#[test]
fn test_core_creation() {
    let core = Core::new(None);
    assert!(core.is_ok(), "Core should be created successfully");
}

#[test]
fn test_core_with_proxy() {
    let core = Core::new(Some("socks5://127.0.0.1:9050".to_string()));
    assert!(core.is_ok(), "Core should be created with proxy");
}

#[tokio::test]
async fn test_http_client_basic() {
    let core = Core::new(None).unwrap();
    let _client = core.http.raw_client();
}

#[test]
fn test_logger_creation() {
    use socialosint::logger::Logger;
    let logger = Logger::new();
    logger.info("Test message");
    logger.good("Success message");
    logger.bad("Error message");
}

#[test]
fn test_tui_creation() {
    use socialosint::tui::{Status, TuiManager};
    let tui = TuiManager::new();
    tui.update_instagram_status(Status::Pending);
    tui.update_instagram_count(0);
}

#[test]
fn test_rate_limiter_creation() {
    use socialosint::rate_limiter::RateLimiter;
    let _limiter = RateLimiter::new(10);
}

#[test]
fn test_detectors_creation() {
    use socialosint::detectors::Detectors;
    let _detectors = Detectors::new();
}
