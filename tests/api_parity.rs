#[cfg(test)]
mod api_parity {
    use socialosint::core::Core;

    #[test]
    fn test_core_initialization_parity() {
        let core = Core::new(None);
        assert!(core.is_ok());

        let core_with_proxy = Core::new(Some("socks5://127.0.0.1:9050".to_string()));
        assert!(core_with_proxy.is_ok());
    }

    #[test]
    fn test_logger_output_format_parity() {
        use socialosint::logger::Logger;

        let logger = Logger::new();
        logger.info("test");
        logger.good("test");
        logger.bad("test");
        logger.debug("test");
        logger.field("key", "value");
    }

    #[test]
    fn test_tui_status_enum_parity() {
        use socialosint::tui::Status;

        let _pending = Status::Pending;
        let _in_progress = Status::InProgress;
        let _success = Status::Success;
        let _failed = Status::Failed;
    }

    #[test]
    fn test_rate_limiter_default_parity() {
        use socialosint::rate_limiter::RateLimiter;

        let limiter = RateLimiter::default();
        drop(limiter);

        let custom_limiter = RateLimiter::new(5);
        drop(custom_limiter);
    }

    #[test]
    fn test_detectors_default_parity() {
        use socialosint::detectors::Detectors;

        let detectors = Detectors::default();
        drop(detectors);
    }
}
