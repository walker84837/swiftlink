use actix_web::{
    Error, HttpResponse,
    body::EitherBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
};
use futures_util::future::{LocalBoxFuture, Ready, ready};
use governor::{Quota, RateLimiter, clock::QuantaClock, state::keyed::DefaultKeyedStateStore};
use log::warn;

use std::{
    collections::HashMap,
    net::IpAddr,
    num::NonZeroU32,
    rc::Rc,
    sync::{Arc, Mutex},
    task::{Context, Poll},
    time::{Duration, Instant},
};

/// A key (IP address, route) used by the rate limiter to identify a client.
/// The combination is used to scope rate limits per-route.
type RateKey = (IpAddr, String);

/// Shared, thread-safe rate limiter instance. This enforces request rate limits keyed by [`RateKey`].
///
/// Rate limits apply per unique [`RateKey`] tuple.
type SharedRateLimiter = Arc<RateLimiter<RateKey, DefaultKeyedStateStore<RateKey>, QuantaClock>>;

/// LRU cache entry for rate limiting state
#[derive(Clone)]
struct LruEntry {
    last_accessed: Instant,
    /// Whether this entry is actively being used
    active: bool,
}

/// Bounded LRU cache for rate limiter state entries
///
/// This prevents memory exhaustion attacks by limiting the number of unique
/// clients that can be tracked simultaneously. When the limit is reached,
/// the least recently used inactive entries are evicted to make room.
#[derive(Clone)]
struct BoundedStateStore {
    max_entries: usize,
    entries: Arc<Mutex<HashMap<RateKey, LruEntry>>>,
}

impl BoundedStateStore {
    fn new(max_entries: usize) -> Self {
        Self {
            max_entries,
            entries: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Check if we can accommodate a new entry, evicting oldest if necessary
    fn can_add_entry(&self, key: &RateKey) -> bool {
        let mut entries = self.entries.lock().expect("lock is held by current thread");

        // If entry already exists, just update access time
        if let Some(entry) = entries.get_mut(key) {
            entry.last_accessed = Instant::now();
            entry.active = true;
            return true;
        }

        // If we have space, add new entry
        if entries.len() < self.max_entries {
            entries.insert(
                key.clone(),
                LruEntry {
                    last_accessed: Instant::now(),
                    active: true,
                },
            );
            return true;
        }

        // Find and evict least recently used inactive entry
        if let Some((oldest_key, _)) = entries
            .iter()
            .filter(|(_, entry)| !entry.active)
            .min_by_key(|(_, entry)| entry.last_accessed)
            .map(|(k, v)| (k.clone(), v))
        {
            entries.remove(&oldest_key);
            entries.insert(
                key.clone(),
                LruEntry {
                    last_accessed: Instant::now(),
                    active: true,
                },
            );
            true
        } else {
            // No inactive entries to evict, at capacity
            false
        }
    }

    /// Mark entry as inactive (no longer being rate limited)
    fn mark_inactive(&self, key: &RateKey) {
        if let Ok(mut entries) = self.entries.lock()
            && let Some(entry) = entries.get_mut(key)
        {
            entry.active = false;
        }
    }
}

/// Rate limiting middleware using bounded in-memory storage
#[derive(Clone)]
pub struct RateLimitMiddleware {
    limiter: SharedRateLimiter,
    state_store: BoundedStateStore,
    enabled: bool,
    trust_proxy_headers: bool,
}

impl RateLimitMiddleware {
    pub fn new(
        max_requests: u32,
        window_seconds: u64,
        enabled: bool,
        trust_proxy_headers: bool,
        max_tracked_clients: Option<usize>,
    ) -> Self {
        let max_requests_nz =
            NonZeroU32::new(max_requests.max(1)).expect("max_requests must be > 0");
        let window_duration = Duration::from_secs(window_seconds.max(1));

        // Governor uses a token-bucket model:
        // - allow_burst(max_requests) allows full window bursts
        // - replenish_interval controls steady-state rate
        // Calculate replenish interval: window / max_requests
        let replenish_interval_nanos = window_duration.as_nanos() / max_requests_nz.get() as u128;
        let replenish_interval = Duration::from_nanos(replenish_interval_nanos as u64);

        let quota = Quota::with_period(replenish_interval)
            .expect("Replenish interval must be valid")
            .allow_burst(max_requests_nz);

        let limiter = Arc::new(RateLimiter::new(
            quota,
            DefaultKeyedStateStore::default(),
            QuantaClock::default(),
        ));

        let state_store = BoundedStateStore::new(max_tracked_clients.unwrap_or(10000));

        Self {
            limiter,
            state_store,
            enabled,
            trust_proxy_headers,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = RateLimitMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitMiddlewareService {
            service: Rc::new(service),
            limiter: self.limiter.clone(),
            state_store: self.state_store.clone(),
            enabled: self.enabled,
            trust_proxy_headers: self.trust_proxy_headers,
        }))
    }
}

/// Internal service that wraps the inner service and enforces rate limiting
pub struct RateLimitMiddlewareService<S> {
    service: Rc<S>,
    limiter: SharedRateLimiter,
    state_store: BoundedStateStore,
    enabled: bool,
    trust_proxy_headers: bool,
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let limiter = self.limiter.clone();
        let state_store = self.state_store.clone();
        let enabled = self.enabled;
        let trust_proxy_headers = self.trust_proxy_headers;

        // Create a future that will forward the request to the inner service
        Box::pin(async move {
            // If rate limiting is disabled, return early
            if !enabled {
                // Forward the request to the inner service unmodified
                return service.call(req).await.map(|res| res.map_into_left_body());
            }

            let Some(ip) = extract_client_ip(&req, trust_proxy_headers) else {
                warn!(
                    "Rate limit skipped: could not determine client IP ({} {})",
                    req.method(),
                    req.path()
                );
                return service.call(req).await.map(|res| res.map_into_left_body());
            };

            // Use (IP, Route Pattern) as the key to prevent path param abuse
            let route_key = req
                .match_pattern()
                .unwrap_or_else(|| req.path().to_string());

            let key = (ip, route_key);

            // Check if we can track this client (bounded memory usage)
            if !state_store.can_add_entry(&key) {
                warn!(
                    "Rate limit skipped: too many tracked clients ({} {})",
                    req.method(),
                    req.path()
                );
                return service.call(req).await.map(|res| res.map_into_left_body());
            }

            if limiter.check_key(&key).is_err() {
                state_store.mark_inactive(&key);
                let response = HttpResponse::TooManyRequests().json(serde_json::json!({
                    "error": "Rate limit exceeded",
                    "message": "Too many requests. Please try again later."
                }));
                return Ok(req.into_response(response).map_into_right_body());
            }

            state_store.mark_inactive(&key);
            service.call(req).await.map(|res| res.map_into_left_body())
        })
    }
}

/// Extract real client IP from request, supporting reverse proxies if configured
///
/// See:
/// - <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Forwarded-For>
/// - <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Forwarded>
fn extract_client_ip(req: &ServiceRequest, trust_proxy_headers: bool) -> Option<IpAddr> {
    if !trust_proxy_headers {
        // Get the direct peer IP
        return req.peer_addr().map(|addr| normalize_ip(addr.ip()));
    }

    // Try X-Forwarded-For header first
    if let Some(ip) = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(str::trim)
        .and_then(|ip_str| ip_str.parse().ok())
    {
        return Some(normalize_ip(ip));
    }

    // Try Forwarded header (RFC 7239)
    if let Some(forwarded_str) = req.headers().get("forwarded").and_then(|h| h.to_str().ok())
        && let Some(ip) = forwarded_str
            .split(',')
            .map(str::trim)
            .filter_map(extract_for_param)
            .next()
    {
        return Some(normalize_ip(ip));
    }

    // Fallback to direct peer IP if no forwarded headers are valid
    req.peer_addr().map(|addr| normalize_ip(addr.ip()))
}

/// Extracts the client IP from the `for=...` value in the HTTP `Forwarded` field
fn extract_for_param(forwarded_element: &str) -> Option<IpAddr> {
    for part in forwarded_element.split(';').map(str::trim) {
        // Lowercase for comparison, but keep original for parsing
        let part_lower = part.to_lowercase();
        if part_lower.starts_with("for=") {
            // Use original 'part' to preserve casing for parsing
            let stripped = &part[4..]; // skip "for="
            let ip_str = stripped
                .trim_matches('"')
                .trim_start_matches('[')
                .trim_end_matches(']')
                .trim();
            if let Ok(ip) = ip_str.parse() {
                return Some(ip);
            }
        }
    }
    None
}

/// Normalize IPv6-mapped IPv4 addresses to IPv4
fn normalize_ip(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(ipv6) => {
            if let Some(ipv4) = ipv6.to_ipv4_mapped() {
                IpAddr::V4(ipv4)
            } else {
                IpAddr::V6(ipv6)
            }
        }
        IpAddr::V4(ipv4) => IpAddr::V4(ipv4),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::{self, TestRequest};
    use actix_web::{App, HttpResponse, web};
    use std::str::FromStr;

    #[test]
    fn test_normalize_ip() {
        let v4 = IpAddr::from_str("192.168.1.1").unwrap();
        assert_eq!(normalize_ip(v4), v4);

        let v6 = IpAddr::from_str("2001:db8::1").unwrap();
        assert_eq!(normalize_ip(v6), v6);

        // ::ffff:192.168.1.1
        let mapped = IpAddr::from_str("::ffff:192.168.1.1").unwrap();
        let normalized = normalize_ip(mapped);
        assert!(normalized.is_ipv4());
        assert_eq!(normalized.to_string(), "192.168.1.1");
    }

    #[actix_web::test]
    async fn test_extract_client_ip() {
        // Direct connection
        let req = TestRequest::default()
            .peer_addr("192.168.1.5:12345".parse().unwrap())
            .to_srv_request();
        assert_eq!(
            extract_client_ip(&req, false),
            Some(IpAddr::from_str("192.168.1.5").unwrap())
        );

        // X-Forwarded-For (Trusted)
        let req = TestRequest::default()
            .insert_header(("X-Forwarded-For", "10.0.0.1, 192.168.1.1"))
            .to_srv_request();
        assert_eq!(
            extract_client_ip(&req, true),
            Some(IpAddr::from_str("10.0.0.1").unwrap())
        );

        // X-Forwarded-For (Untrusted)
        let req = TestRequest::default()
            .peer_addr("192.168.1.5:12345".parse().unwrap())
            .insert_header(("X-Forwarded-For", "10.0.0.1, 192.168.1.1"))
            .to_srv_request();
        assert_eq!(
            extract_client_ip(&req, false),
            Some(IpAddr::from_str("192.168.1.5").unwrap())
        );

        // Forwarded (Trusted)
        let req = TestRequest::default()
            .insert_header(("Forwarded", "for=10.0.0.2;proto=http, for=192.168.1.1"))
            .to_srv_request();
        assert_eq!(
            extract_client_ip(&req, true),
            Some(IpAddr::from_str("10.0.0.2").unwrap())
        );

        // Forwarded with IPv6 (Trusted)
        let req = TestRequest::default()
            .insert_header(("Forwarded", "for=\"[2001:db8::1]\""))
            .to_srv_request();
        assert_eq!(
            extract_client_ip(&req, true),
            Some(IpAddr::from_str("2001:db8::1").unwrap())
        );
    }

    #[actix_web::test]
    async fn test_middleware_blocks_excess_requests() {
        // Allow 2 requests per minute
        let mw = RateLimitMiddleware::new(2, 60, true, false, Some(1000));

        let app = test::init_service(
            App::new()
                .wrap(mw)
                .route("/", web::get().to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;

        let ip = "127.0.0.1:12345".parse().unwrap();

        // 1st Request - OK
        let req = TestRequest::get().uri("/").peer_addr(ip).to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // 2nd Request - OK
        let req = TestRequest::get().uri("/").peer_addr(ip).to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // 3rd Request - Blocked
        let req = TestRequest::get().uri("/").peer_addr(ip).to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 429);
    }

    #[actix_web::test]
    async fn test_middleware_per_route_isolation() {
        // Allow 1 request per minute
        let mw = RateLimitMiddleware::new(1, 60, true, false, Some(1000));

        let app = test::init_service(
            App::new()
                .wrap(mw)
                .route(
                    "/a",
                    web::get().to(|| async { HttpResponse::Ok().finish() }),
                )
                .route(
                    "/b",
                    web::get().to(|| async { HttpResponse::Ok().finish() }),
                ),
        )
        .await;

        let ip = "127.0.0.1:12345".parse().unwrap();

        // Request to /a - OK
        let req = TestRequest::get().uri("/a").peer_addr(ip).to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Second request to /a - Blocked
        let req = TestRequest::get().uri("/a").peer_addr(ip).to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 429);

        // Request to /b - OK (different route key)
        let req = TestRequest::get().uri("/b").peer_addr(ip).to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn test_middleware_match_pattern() {
        // Allow 1 request per minute
        let mw = RateLimitMiddleware::new(1, 60, true, false, Some(1000));

        let app = test::init_service(App::new().wrap(mw).route(
            "/user/{id}",
            web::get().to(|| async { HttpResponse::Ok().finish() }),
        ))
        .await;

        let ip = "127.0.0.1:12345".parse().unwrap();

        // Request to /user/1 - OK
        let req = TestRequest::get().uri("/user/1").peer_addr(ip).to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Request to /user/2 - Blocked (same pattern /user/{id})
        let req = TestRequest::get().uri("/user/2").peer_addr(ip).to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 429);
    }

    #[actix_web::test]
    async fn test_bounded_state_store_behavior() {
        // Allow 1 request per minute, but only track 2 clients
        let mw = RateLimitMiddleware::new(1, 60, true, false, Some(2));

        let app = test::init_service(
            App::new()
                .wrap(mw)
                .route("/", web::get().to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;

        // First client - OK
        let req1 = TestRequest::get()
            .uri("/")
            .peer_addr("127.0.0.1:12345".parse().unwrap())
            .to_request();
        let resp1 = test::call_service(&app, req1).await;
        assert_eq!(resp1.status(), 200);

        // Second client - OK
        let req2 = TestRequest::get()
            .uri("/")
            .peer_addr("127.0.0.2:12345".parse().unwrap())
            .to_request();
        let resp2 = test::call_service(&app, req2).await;
        assert_eq!(resp2.status(), 200);

        // Third client - Should skip rate limiting (at capacity) but still be allowed
        let req3 = TestRequest::get()
            .uri("/")
            .peer_addr("127.0.0.3:12345".parse().unwrap())
            .to_request();
        let resp3 = test::call_service(&app, req3).await;
        assert_eq!(resp3.status(), 200); // Should be allowed due to capacity limit
    }
}
