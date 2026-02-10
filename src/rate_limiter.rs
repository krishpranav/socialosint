use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::time::sleep;

struct DomainLimiter {
    last_request: Instant,
    min_delay_ms: u64,
}

pub struct RateLimiter {
    limiters: Arc<DashMap<String, DomainLimiter>>,
    global_semaphore: Arc<Semaphore>,
}

impl RateLimiter {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            limiters: Arc::new(DashMap::new()),
            global_semaphore: Arc::new(Semaphore::new(max_concurrent)),
        }
    }

    pub async fn acquire(&self, domain: &str) {
        let _permit = self.global_semaphore.acquire().await.unwrap();

        let min_delay_ms = 1000;

        let mut limiter = self
            .limiters
            .entry(domain.to_string())
            .or_insert(DomainLimiter {
                last_request: Instant::now() - Duration::from_secs(10),
                min_delay_ms,
            });

        let elapsed = limiter.last_request.elapsed();
        if elapsed < Duration::from_millis(min_delay_ms) {
            let wait_time = Duration::from_millis(min_delay_ms) - elapsed;
            tracing::debug!("Rate limiting {}: waiting {:?}", domain, wait_time);
            sleep(wait_time).await;
        }

        limiter.last_request = Instant::now();
    }

    pub fn set_domain_limit(&self, domain: &str, min_delay_ms: u64) {
        self.limiters
            .entry(domain.to_string())
            .and_modify(|limiter| limiter.min_delay_ms = min_delay_ms)
            .or_insert(DomainLimiter {
                last_request: Instant::now() - Duration::from_secs(10),
                min_delay_ms,
            });
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(10)
    }
}
