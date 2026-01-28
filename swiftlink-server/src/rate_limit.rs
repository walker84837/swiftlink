use actix_web::body::EitherBody;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, HttpResponse};
use futures_util::future::{LocalBoxFuture, Ready, ready};
use governor::{Quota, RateLimiter, clock::QuantaClock, state::keyed::DefaultKeyedStateStore};
use log::warn;
use std::net::IpAddr;
use std::num::NonZeroU32;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

type SharedRateLimiter =
    Arc<RateLimiter<(IpAddr, String), DefaultKeyedStateStore<(IpAddr, String)>, QuantaClock>>;

/// Rate limiting middleware using in-memory storage
#[derive(Clone)]
pub struct RateLimitMiddleware {
    // Key is (IP, Route) to support per-route limits
    limiter: SharedRateLimiter,
    enabled: bool,
    trust_proxy_headers: bool,
}

impl RateLimitMiddleware {
    pub fn new(
        max_requests: u32,
        window_seconds: u64,
        enabled: bool,
        trust_proxy_headers: bool,
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

        Self {
            limiter,
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
    type Error = Error;
    type InitError = ();
    type Transform = RateLimitMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitMiddlewareService {
            service: Rc::new(service),
            limiter: self.limiter.clone(),
            enabled: self.enabled,
            trust_proxy_headers: self.trust_proxy_headers,
        }))
    }
}

pub struct RateLimitMiddlewareService<S> {
    service: Rc<S>,
    limiter: SharedRateLimiter,
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

    fn poll_ready(
        &self,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let limiter = self.limiter.clone();
        let enabled = self.enabled;
        let trust_proxy_headers = self.trust_proxy_headers;

        Box::pin(async move {
            if !enabled {
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

            if limiter.check_key(&key).is_err() {
                let response = HttpResponse::TooManyRequests().json(serde_json::json!({
                    "error": "Rate limit exceeded",
                    "message": "Too many requests. Please try again later."
                }));
                return Ok(req.into_response(response).map_into_right_body());
            }

            service.call(req).await.map(|res| res.map_into_left_body())
        })
    }
}

/// Extract real client IP from request, supporting reverse proxies if configured
fn extract_client_ip(req: &ServiceRequest, trust_proxy_headers: bool) -> Option<IpAddr> {
    if trust_proxy_headers {
        // Try X-Forwarded-For header first (most comprehensive)
        if let Some(forwarded_str) = req
            .headers()
            .get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
        {
            // X-Forwarded-For can contain multiple IPs, take the first one
            let first_ip = forwarded_str.split(',').next()?.trim();
            if let Ok(ip) = first_ip.parse() {
                return Some(normalize_ip(ip));
            }
        }

        // Try Forwarded header (RFC 7239)
        if let Some(forwarded_str) = req.headers().get("forwarded").and_then(|h| h.to_str().ok()) {
            // Forwarded header format: for=192.0.2.1;proto=http
            for part in forwarded_str.split(',') {
                let part = part.trim();
                if part.to_lowercase().starts_with("for=") {
                    let ip_part = &part[4..];
                    // Handle parameters separated by semicolon (e.g. for=1.2.3.4;proto=https)
                    let ip_part = ip_part.split(';').next().unwrap_or(ip_part);
                    // Remove any quotes, brackets (for IPv6) and whitespace
                    let ip_str = ip_part
                        .trim()
                        .trim_matches('"')
                        .trim_start_matches('[')
                        .trim_end_matches(']');

                    if let Ok(ip) = ip_str.parse() {
                        return Some(normalize_ip(ip));
                    }
                }
            }
        }
    }

    // Fallback to direct connection IP
    req.peer_addr().map(|addr| normalize_ip(addr.ip()))
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
        let mw = RateLimitMiddleware::new(2, 60, true, false);

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
        let mw = RateLimitMiddleware::new(1, 60, true, false);

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
        let mw = RateLimitMiddleware::new(1, 60, true, false);

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
}
