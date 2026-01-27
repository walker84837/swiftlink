use actix_web::HttpResponse;
use governor::{Quota, RateLimiter, clock::QuantaClock, state::keyed::DefaultKeyedStateStore};
use log::warn;
use std::{net::IpAddr, num::NonZeroU32, sync::Arc, time::Duration};

/// Rate limiting middleware using in-memory storage
#[derive(Clone)]
pub struct RateLimitMiddleware {
    limiter: Arc<RateLimiter<IpAddr, DefaultKeyedStateStore<IpAddr>, QuantaClock>>,
    enabled: bool,
}

impl RateLimitMiddleware {
    pub fn new(max_requests: u32, window_seconds: u64, enabled: bool) -> Self {
        // SAFETY: max_requests is at least 1
        let max_requests = NonZeroU32::new(max_requests.max(1)).unwrap();
        let window = Duration::from_secs(window_seconds.max(1));

        // Calculate nanoseconds per request: converts the time window into per-request timing
        // Formula: (total_window_nanos / max_requests) = nanos_allowed_per_request
        // This ensures the rate limiter allows exactly max_requests within the window duration
        let nanos_per_request = (window.as_nanos() / max_requests.get() as u128) as u64;
        let quota = Quota::with_period(Duration::from_nanos(nanos_per_request))
            .expect("Invalid quota period")
            .allow_burst(max_requests);

        let limiter = Arc::new(RateLimiter::new(
            quota,
            DefaultKeyedStateStore::default(),
            QuantaClock::default(),
        ));

        Self { limiter, enabled }
    }

    pub fn check_rate_limit(&self, client_ip: Option<IpAddr>) -> Result<(), HttpResponse> {
        if !self.enabled {
            return Ok(());
        }

        let Some(ip) = client_ip else {
            warn!("Could not determine client IP for rate limiting. Allowing request.");
            return Ok(());
        };

        self.limiter.check_key(&ip).map_err(|_| {
            HttpResponse::TooManyRequests().json(serde_json::json!({
                "error": "Rate limit exceeded",
                "message": "Too many requests. Please try again later."
            }))
        })
    }
}
