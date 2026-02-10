use std::time::Instant;

pub fn init() {
    tracing::debug!("Telemetry initialized");
}

pub struct RequestMetrics {
    start: Instant,
    platform: String,
}

impl RequestMetrics {
    pub fn new(platform: &str) -> Self {
        tracing::info!(platform = %platform, "Starting request");
        Self {
            start: Instant::now(),
            platform: platform.to_string(),
        }
    }

    pub fn record_success(self) {
        let duration = self.start.elapsed();
        tracing::info!(
            platform = %self.platform,
            duration_ms = duration.as_millis(),
            "Request succeeded"
        );
    }

    pub fn record_failure(self, error: &str) {
        let duration = self.start.elapsed();
        tracing::error!(
            platform = %self.platform,
            duration_ms = duration.as_millis(),
            error = %error,
            "Request failed"
        );
    }
}

pub fn record_emails_found(platform: &str, count: usize) {
    tracing::info!(platform = %platform, count = count, "Emails found");
}

pub fn record_rate_limit_hit(platform: &str) {
    tracing::warn!(platform = %platform, "Rate limit hit");
}
