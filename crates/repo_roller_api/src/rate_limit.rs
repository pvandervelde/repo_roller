//! Application-level rate limiting (Task #284).
//!
//! Per-user fixed-window rate limiting for state-changing (POST) requests.
//!
//! # Contract
//!
//! * Fixed window per key: the window starts at the key's first allowed
//!   request; at most `max_requests` are allowed in `[start, start + window)`;
//!   at `start + window` exactly, a fresh window begins.
//! * Rejected requests never extend or restart a window.
//! * `Limited.retry_after_secs` = ceil(seconds until window end), minimum 1.
//! * Config env vars: `RATE_LIMIT_ENABLED`, `RATE_LIMIT_REQUESTS`,
//!   `RATE_LIMIT_WINDOW_SECS`. Invalid values fall back to defaults (never panic).
//! * Middleware runs after auth, keyed by `AuthContext::user_login`, applies to
//!   POST requests only, answers 429 + `Retry-After` + JSON `ErrorResponse`
//!   with code `RateLimitExceeded`.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

use axum::{
    extract::{Request, State},
    http::{header, HeaderValue, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

use crate::errors::{ErrorDetails, ErrorResponse};
use crate::middleware::AuthContext;
use crate::AppState;

/// Default maximum requests per window.
pub const DEFAULT_MAX_REQUESTS: u32 = 10;
/// Default window length in seconds.
pub const DEFAULT_WINDOW_SECS: u64 = 60;

/// Fixed key used when the authenticated principal has no resolvable login.
/// All such principals share one bucket rather than bypassing the limiter.
pub const UNKNOWN_USER_KEY: &str = "<unknown-user>";

/// Default maximum token-exchange requests per client address per window.
pub const DEFAULT_AUTH_MAX_REQUESTS: u32 = 20;
/// Default token-exchange window length in seconds.
pub const DEFAULT_AUTH_WINDOW_SECS: u64 = 60;
/// Key used for token-exchange requests that carry no `ConnectInfo`.
pub const NO_CLIENT_ADDR_KEY: &str = "addr:none";

/// Expired entries are swept every this many checks.
const EVICT_EVERY_N_CHECKS: u64 = 256;
/// When the tracked-key count reaches this, expired entries are swept before
/// inserting a new key.
const MAX_TRACKED_KEYS: usize = 10_000;

/// Rate-limit configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RateLimitConfig {
    /// Master switch. When false, nothing is ever limited.
    pub enabled: bool,
    /// Max requests per key per window (always >= 1).
    pub max_requests: u32,
    /// Window length.
    pub window: Duration,
}

impl RateLimitConfig {
    /// Explicit constructor (used by tests and `from_env_vars`).
    pub fn new(enabled: bool, max_requests: u32, window_secs: u64) -> Self {
        Self {
            enabled,
            max_requests,
            window: Duration::from_secs(window_secs),
        }
    }

    /// Build from a variable lookup (testable without mutating process env).
    /// Invalid/missing values fall back to defaults; never panics.
    ///
    /// `RATE_LIMIT_ENABLED` only disables on `false`/`0` (case-insensitive);
    /// anything else fails closed to enabled. `RATE_LIMIT_REQUESTS` and
    /// `RATE_LIMIT_WINDOW_SECS` must be positive integers.
    pub fn from_env_vars(lookup: impl Fn(&str) -> Option<String>) -> Self {
        let enabled = match lookup("RATE_LIMIT_ENABLED") {
            Some(v) => {
                let v = v.trim();
                !(v.eq_ignore_ascii_case("false") || v == "0")
            }
            None => true,
        };
        let max_requests = lookup("RATE_LIMIT_REQUESTS")
            .and_then(|v| v.trim().parse::<u32>().ok())
            .filter(|n| *n >= 1)
            .unwrap_or(DEFAULT_MAX_REQUESTS);
        let window_secs = lookup("RATE_LIMIT_WINDOW_SECS")
            .and_then(|v| v.trim().parse::<u64>().ok())
            .filter(|n| *n >= 1)
            .unwrap_or(DEFAULT_WINDOW_SECS);
        Self::new(enabled, max_requests, window_secs)
    }

    /// Config for the token-exchange (public) limiter. Reads
    /// `RATE_LIMIT_ENABLED`, `RATE_LIMIT_AUTH_REQUESTS`,
    /// `RATE_LIMIT_AUTH_WINDOW_SECS`. STUB (TDD red).
    pub fn auth_from_env_vars(_lookup: impl Fn(&str) -> Option<String>) -> Self {
        Self::default()
    }

    /// Token-exchange config from the process environment.
    pub fn auth_from_env() -> Self {
        Self::auth_from_env_vars(|k| std::env::var(k).ok())
    }

    /// Build from the process environment.
    pub fn from_env() -> Self {
        Self::from_env_vars(|k| std::env::var(k).ok())
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self::new(true, DEFAULT_MAX_REQUESTS, DEFAULT_WINDOW_SECS)
    }
}

/// Monotonic time source (duration since an arbitrary fixed origin).
pub trait Clock: Send + Sync {
    /// Current monotonic time.
    fn now(&self) -> Duration;
}

/// Production clock.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Duration {
        static ORIGIN: OnceLock<Instant> = OnceLock::new();
        ORIGIN.get_or_init(Instant::now).elapsed()
    }
}

/// Outcome of a limiter check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitDecision {
    /// Request permitted; `remaining` further requests allowed in this window.
    Allowed { remaining: u32 },
    /// Request rejected; retry after this many whole seconds (>= 1).
    Limited { retry_after_secs: u64 },
}

/// Mutable limiter state guarded by one mutex.
#[derive(Default)]
struct LimiterState {
    /// key -> (window start, requests allowed in window)
    windows: HashMap<String, (Duration, u32)>,
    /// Checks performed; drives periodic eviction.
    checks: u64,
}

impl LimiterState {
    fn evict(&mut self, now: Duration, window: Duration) -> usize {
        let before = self.windows.len();
        self.windows
            .retain(|_, (start, _)| now.saturating_sub(*start) < window);
        before - self.windows.len()
    }
}

/// Per-key fixed-window limiter.
pub struct RateLimiter {
    config: RateLimitConfig,
    clock: Arc<dyn Clock>,
    state: Mutex<LimiterState>,
}

impl RateLimiter {
    /// Limiter using the system clock.
    pub fn new(config: RateLimitConfig) -> Self {
        Self::with_clock(config, Arc::new(SystemClock))
    }

    /// Limiter with an injected clock.
    pub fn with_clock(config: RateLimitConfig, clock: Arc<dyn Clock>) -> Self {
        Self {
            config,
            clock,
            state: Mutex::new(LimiterState::default()),
        }
    }

    /// The active configuration.
    pub fn config(&self) -> &RateLimitConfig {
        &self.config
    }

    fn lock(&self) -> MutexGuard<'_, LimiterState> {
        // A poisoned lock only means another thread panicked mid-update; the
        // map is still structurally valid, so keep limiting rather than fail.
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Record one request for `key` and decide. Always `Allowed` if disabled.
    ///
    /// Expired entries are evicted automatically every
    /// `EVICT_EVERY_N_CHECKS` checks and whenever a new key is inserted while
    /// `MAX_TRACKED_KEYS` or more keys are tracked, bounding memory.
    pub fn check(&self, key: &str) -> RateLimitDecision {
        if !self.config.enabled {
            return RateLimitDecision::Allowed {
                remaining: self.config.max_requests,
            };
        }
        let now = self.clock.now();
        let window = self.config.window;
        let max = self.config.max_requests;
        let mut st = self.lock();

        st.checks = st.checks.wrapping_add(1);
        if st.checks.is_multiple_of(EVICT_EVERY_N_CHECKS) {
            st.evict(now, window);
        }

        if let Some((start, count)) = st.windows.get_mut(key) {
            let elapsed = now.saturating_sub(*start);
            if elapsed >= window {
                *start = now;
                *count = 1;
                return RateLimitDecision::Allowed {
                    remaining: max.saturating_sub(1),
                };
            }
            if *count >= max {
                let remaining = window - elapsed;
                let secs = remaining.as_secs() + u64::from(remaining.subsec_nanos() > 0);
                return RateLimitDecision::Limited {
                    retry_after_secs: secs.max(1),
                };
            }
            *count += 1;
            return RateLimitDecision::Allowed {
                remaining: max - *count,
            };
        }

        if st.windows.len() >= MAX_TRACKED_KEYS {
            st.evict(now, window);
        }
        st.windows.insert(key.to_string(), (now, 1));
        RateLimitDecision::Allowed {
            remaining: max.saturating_sub(1),
        }
    }

    /// Drop every key whose window has ended. Returns number removed.
    pub fn evict_expired(&self) -> usize {
        let now = self.clock.now();
        self.lock().evict(now, self.config.window)
    }

    /// Number of keys currently tracked.
    pub fn tracked_keys(&self) -> usize {
        self.lock().windows.len()
    }
}

/// Limiter key for an authenticated principal, or `None` when the context
/// carries no stable identifier (the request must then be rejected).
///
/// STUB (TDD red).
pub fn principal_key(ctx: &AuthContext) -> Option<String> {
    ctx.user_login.clone()
}

/// Limiter key for the client of a public request, derived from the
/// connection address only. STUB (TDD red).
pub fn client_key(_request: &Request) -> String {
    NO_CLIENT_ADDR_KEY.to_string()
}

/// Per-client-address limiter for `POST /api/v1/auth/token`. STUB (TDD red).
pub async fn auth_rate_limit_middleware(
    State(_state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    next.run(request).await
}

/// Axum middleware enforcing the limit (must run after `auth_middleware`).
///
/// Only `POST` requests are limited. The bucket key is the authenticated
/// user's login (`AuthContext::user_login`); principals without a login share
/// the fixed `UNKNOWN_USER_KEY` bucket. Rejections return `429` with a
/// `Retry-After` header and the standard JSON error body, are logged at WARN
/// (without the user identity) and counted in `rate_limit_rejections_total`.
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    if request.method() != Method::POST {
        return next.run(request).await;
    }

    let key = request
        .extensions()
        .get::<AuthContext>()
        .and_then(|ctx| ctx.user_login.as_deref())
        .unwrap_or(UNKNOWN_USER_KEY);

    match state.rate_limiter.check(key) {
        RateLimitDecision::Allowed { .. } => next.run(request).await,
        RateLimitDecision::Limited { retry_after_secs } => {
            state.rate_limit_rejections.inc();
            tracing::warn!(
                retry_after_secs,
                "Rate limit exceeded; rejecting request with 429"
            );
            let body = ErrorResponse {
                error: ErrorDetails {
                    code: "RateLimitExceeded".to_string(),
                    message: format!("Too many requests. Retry after {retry_after_secs} seconds."),
                    details: None,
                },
            };
            let mut response = (StatusCode::TOO_MANY_REQUESTS, Json(body)).into_response();
            response
                .headers_mut()
                .insert(header::RETRY_AFTER, HeaderValue::from(retry_after_secs));
            response
        }
    }
}

#[cfg(test)]
#[path = "rate_limit_tests.rs"]
pub(crate) mod tests;

#[cfg(test)]
#[path = "rate_limit_auth_tests.rs"]
mod auth_tests;

#[cfg(test)]
#[path = "rate_limit_proptests.rs"]
mod proptests;
