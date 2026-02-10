use socialosint::rate_limiter::RateLimiter;
use std::time::Instant;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_creation() {
        let limiter = RateLimiter::new(10);
        assert!(std::mem::size_of_val(&limiter) > 0);
    }

    #[tokio::test]
    async fn test_rate_limiter_default() {
        let limiter = RateLimiter::default();
        assert!(std::mem::size_of_val(&limiter) > 0);
    }

    #[tokio::test]
    async fn test_rate_limiter_acquire() {
        let limiter = RateLimiter::new(5);

        let start = Instant::now();
        limiter.acquire("example.com").await;
        let first_duration = start.elapsed();

        assert!(first_duration.as_millis() < 100);
    }

    #[tokio::test]
    async fn test_rate_limiter_enforces_delay() {
        let limiter = RateLimiter::new(5);

        limiter.acquire("test.com").await;

        let start = Instant::now();
        limiter.acquire("test.com").await;
        let duration = start.elapsed();

        assert!(duration.as_millis() >= 900);
    }

    #[tokio::test]
    async fn test_rate_limiter_different_domains() {
        let limiter = RateLimiter::new(5);

        limiter.acquire("domain1.com").await;

        let start = Instant::now();
        limiter.acquire("domain2.com").await;
        let duration = start.elapsed();

        assert!(duration.as_millis() < 100);
    }

    #[tokio::test]
    async fn test_set_domain_limit() {
        let limiter = RateLimiter::new(5);

        limiter.set_domain_limit("custom.com", 500);

        limiter.acquire("custom.com").await;

        let start = Instant::now();
        limiter.acquire("custom.com").await;
        let duration = start.elapsed();

        assert!(duration.as_millis() >= 450);
    }

    #[tokio::test]
    async fn test_concurrent_requests() {
        use std::sync::Arc;

        let limiter = Arc::new(RateLimiter::new(3));
        let mut handles = vec![];

        for i in 0..5 {
            let limiter_clone = Arc::clone(&limiter);
            let handle = tokio::spawn(async move {
                limiter_clone.acquire(&format!("domain{}.com", i)).await;
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

    }
}
