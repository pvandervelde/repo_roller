//! Application-level rate limiting (Task #284).
//!
//! Per-user fixed-window rate limiting for all requests except the safe
//! methods `GET`, `HEAD` and `OPTIONS`.
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
//! * Middleware runs after auth, keyed by [`principal_key`], applies to
//!   every method except GET/HEAD/OPTIONS, answers 429 + `Retry-After` + JSON `ErrorResponse`
//!   with code `RateLimitExceeded`. Requests with no principal identifier are
//!   rejected (401), never pooled.
//! * Each limiter tracks at most a fixed number of keys (default
//!   [`DEFAULT_MAX_TRACKED_KEYS`]); see [`FullPolicy`] for what happens at the cap.
//! * The public token endpoint has its own limiter keyed by client IP
//!   (`auth_rate_limit_middleware`); env vars `RATE_LIMIT_AUTH_REQUESTS`,
//!   `RATE_LIMIT_AUTH_WINDOW_SECS`.

use std::collections::{HashMap, VecDeque};
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

use axum::{
    extract::{ConnectInfo, Request, State},
    http::{header, HeaderValue, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

use crate::errors::{ErrorDetails, ErrorResponse};
use crate::middleware::{AuthContext, AuthError};
use crate::AppState;

/// Default maximum requests per window.
pub const DEFAULT_MAX_REQUESTS: u32 = 10;
/// Default window length in seconds.
pub const DEFAULT_WINDOW_SECS: u64 = 60;

/// Default maximum token-exchange requests per client address per window.
pub const DEFAULT_AUTH_MAX_REQUESTS: u32 = 20;
/// Default token-exchange window length in seconds.
pub const DEFAULT_AUTH_WINDOW_SECS: u64 = 60;
/// Key used for token-exchange requests that carry no `ConnectInfo`.
pub const NO_CLIENT_ADDR_KEY: &str = "addr:none";

/// `scope` label value for rejections by the per-user limiter.
pub const SCOPE_USER: &str = "user";
/// `scope` label value for rejections by the token-endpoint (client IP) limiter.
pub const SCOPE_AUTH_IP: &str = "auth_ip";

/// Legacy sweep interval, kept only for the sweep tests: expired keys are now
/// reclaimed incrementally on every check, which is at least as eager.
#[cfg(test)]
const EVICT_EVERY_N_CHECKS: u64 = 256;
/// Default hard cap on the number of keys a limiter tracks.
pub const DEFAULT_MAX_TRACKED_KEYS: usize = 10_000;

/// What a limiter does when it is at its key cap and reclamation of expired
/// keys freed nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FullPolicy {
    /// Evict the key whose window started earliest and admit the new key.
    EvictOldest,
    /// Reject the new key with `Limited` without inserting it (fail closed).
    FailClosed,
}

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
        let enabled = Self::parse_enabled(&lookup);
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
    /// `RATE_LIMIT_AUTH_WINDOW_SECS`. Same parsing rules as
    /// [`from_env_vars`](Self::from_env_vars); defaults are
    /// `DEFAULT_AUTH_MAX_REQUESTS` / `DEFAULT_AUTH_WINDOW_SECS`.
    pub fn auth_from_env_vars(lookup: impl Fn(&str) -> Option<String>) -> Self {
        let enabled = Self::parse_enabled(&lookup);
        let max_requests = lookup("RATE_LIMIT_AUTH_REQUESTS")
            .and_then(|v| v.trim().parse::<u32>().ok())
            .filter(|n| *n >= 1)
            .unwrap_or(DEFAULT_AUTH_MAX_REQUESTS);
        let window_secs = lookup("RATE_LIMIT_AUTH_WINDOW_SECS")
            .and_then(|v| v.trim().parse::<u64>().ok())
            .filter(|n| *n >= 1)
            .unwrap_or(DEFAULT_AUTH_WINDOW_SECS);
        Self::new(enabled, max_requests, window_secs)
    }

    /// `RATE_LIMIT_ENABLED` only disables on `false`/`0` (case-insensitive);
    /// anything else fails closed to enabled.
    fn parse_enabled(lookup: &impl Fn(&str) -> Option<String>) -> bool {
        match lookup("RATE_LIMIT_ENABLED") {
            Some(v) => {
                let v = v.trim();
                !(v.eq_ignore_ascii_case("false") || v == "0")
            }
            None => true,
        }
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
///
/// Invariant: every key in `windows` has exactly one entry in `expiry` whose
/// start equals the key's current window start. `expiry` is ordered by window
/// start (the clock is monotonic, so entries are pushed in non-decreasing
/// order). Entries whose start differs from the key's current start (or whose
/// key is gone) are stale and are skipped when popped.
#[derive(Default)]
struct LimiterState {
    /// key -> (window start, requests allowed in window)
    windows: HashMap<Arc<str>, (Duration, u32)>,
    /// Window starts in insertion (= expiry) order.
    expiry: VecDeque<(Duration, Arc<str>)>,
    /// Entries visited by the costliest single `check`.
    max_work: usize,
    /// Entries visited across all `check` calls.
    total_work: usize,
}

impl LimiterState {
    /// Pop the front expiry entry. Returns `None` when the queue is empty,
    /// otherwise whether the entry was current (and its key was removed from
    /// `windows`) as opposed to stale.
    fn pop_front_entry(&mut self) -> Option<bool> {
        let (start, key) = self.expiry.pop_front()?;
        let current = self.windows.get(&key).is_some_and(|(s, _)| *s == start);
        if current {
            self.windows.remove(&key);
        }
        Some(current)
    }

    /// Reclaim every key whose window has ended. Amortised O(1) per check:
    /// each window start is pushed and popped once. Returns
    /// `(keys removed, entries visited)`.
    fn reclaim_expired(&mut self, now: Duration, window: Duration) -> (usize, usize) {
        let mut removed = 0;
        let mut visited = 0;
        while let Some((start, _)) = self.expiry.front() {
            if now.saturating_sub(*start) < window {
                break;
            }
            visited += 1;
            if self.pop_front_entry() == Some(true) {
                removed += 1;
            }
        }
        (removed, visited)
    }

    /// Evict the key with the oldest current window start, skipping stale
    /// entries. Returns entries visited.
    fn evict_oldest(&mut self) -> usize {
        let mut visited = 0;
        while let Some(current) = self.pop_front_entry() {
            visited += 1;
            if current {
                break;
            }
        }
        visited
    }

    /// Whole seconds (>= 1) until the earliest-ending live window ends.
    fn retry_after_for_full(&self, now: Duration, window: Duration) -> u64 {
        let remaining = self
            .expiry
            .front()
            .map_or(window, |(start, _)| (*start + window).saturating_sub(now));
        ceil_secs(remaining)
    }
}

/// Ceiling of `d` in whole seconds, minimum 1.
fn ceil_secs(d: Duration) -> u64 {
    (d.as_secs() + u64::from(d.subsec_nanos() > 0)).max(1)
}

/// Per-key fixed-window limiter with a hard cap on tracked keys.
pub struct RateLimiter {
    config: RateLimitConfig,
    clock: Arc<dyn Clock>,
    state: Mutex<LimiterState>,
    max_keys: usize,
    full_policy: FullPolicy,
}

impl RateLimiter {
    /// Limiter using the system clock.
    pub fn new(config: RateLimitConfig) -> Self {
        Self::with_clock(config, Arc::new(SystemClock))
    }

    /// Limiter using the system clock, the default key cap and
    /// [`FullPolicy::FailClosed`] (for unauthenticated, attacker-keyed scopes).
    pub fn new_fail_closed(config: RateLimitConfig) -> Self {
        Self::with_max_keys(
            config,
            Arc::new(SystemClock),
            DEFAULT_MAX_TRACKED_KEYS,
            FullPolicy::FailClosed,
        )
    }

    /// Limiter with an injected clock, the default key cap and
    /// [`FullPolicy::EvictOldest`].
    pub fn with_clock(config: RateLimitConfig, clock: Arc<dyn Clock>) -> Self {
        Self::with_max_keys(
            config,
            clock,
            DEFAULT_MAX_TRACKED_KEYS,
            FullPolicy::EvictOldest,
        )
    }

    /// Limiter with an injected clock, an explicit hard cap on tracked keys
    /// (clamped to at least 1) and the policy applied when the cap is reached.
    pub fn with_max_keys(
        config: RateLimitConfig,
        clock: Arc<dyn Clock>,
        max_keys: usize,
        full_policy: FullPolicy,
    ) -> Self {
        Self {
            config,
            clock,
            state: Mutex::new(LimiterState::default()),
            max_keys: max_keys.max(1),
            full_policy,
        }
    }

    /// Hard cap on tracked keys.
    pub fn max_keys(&self) -> usize {
        self.max_keys
    }

    /// Policy applied when the cap is reached.
    pub fn full_policy(&self) -> FullPolicy {
        self.full_policy
    }

    /// Largest number of map/queue entries visited by any single `check`
    /// (instrumentation for the bounded-work tests).
    pub fn max_check_work(&self) -> usize {
        self.lock().max_work
    }

    /// Total entries visited across all `check` calls (instrumentation).
    pub fn total_check_work(&self) -> usize {
        self.lock().total_work
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
    /// Memory is bounded by the key cap (`max_keys`). Each check first
    /// reclaims expired keys from the front of an expiry-ordered queue
    /// (amortised O(1): every window start is queued and dequeued once). A new
    /// key arriving at a full limiter, after reclamation, is handled per
    /// [`FullPolicy`]: `EvictOldest` drops the key with the oldest window start
    /// and admits it; `FailClosed` answers `Limited` (retry-after = time until
    /// the earliest window ends, >= 1) without inserting it.
    pub fn check(&self, key: &str) -> RateLimitDecision {
        if !self.config.enabled {
            return RateLimitDecision::Allowed {
                remaining: self.config.max_requests,
            };
        }
        let now = self.clock.now();
        let mut st = self.lock();

        // One unit for the map lookup, plus queue entries visited.
        let (_, reclaimed_visits) = st.reclaim_expired(now, self.config.window);
        let mut work = 1 + reclaimed_visits;
        let decision = self.decide(&mut st, key, now, &mut work);
        st.max_work = st.max_work.max(work);
        st.total_work = st.total_work.saturating_add(work);
        decision
    }

    fn decide(
        &self,
        st: &mut LimiterState,
        key: &str,
        now: Duration,
        work: &mut usize,
    ) -> RateLimitDecision {
        let window = self.config.window;
        let max = self.config.max_requests;

        if let Some((start, count)) = st.windows.get_mut(key) {
            // Expired windows were reclaimed above, so this window is live.
            if *count >= max {
                return RateLimitDecision::Limited {
                    retry_after_secs: ceil_secs((*start + window).saturating_sub(now)),
                };
            }
            *count += 1;
            return RateLimitDecision::Allowed {
                remaining: max - *count,
            };
        }

        if st.windows.len() >= self.max_keys {
            match self.full_policy {
                FullPolicy::FailClosed => {
                    return RateLimitDecision::Limited {
                        retry_after_secs: st.retry_after_for_full(now, window),
                    };
                }
                FullPolicy::EvictOldest => {
                    *work += st.evict_oldest();
                    if st.windows.len() >= self.max_keys {
                        // Unreachable while the queue invariant holds; never
                        // exceed the cap regardless.
                        return RateLimitDecision::Limited {
                            retry_after_secs: ceil_secs(window),
                        };
                    }
                }
            }
        }

        let key: Arc<str> = Arc::from(key);
        st.windows.insert(Arc::clone(&key), (now, 1));
        st.expiry.push_back((now, key));
        RateLimitDecision::Allowed {
            remaining: max.saturating_sub(1),
        }
    }

    /// Drop every key whose window has ended. Returns number removed.
    pub fn evict_expired(&self) -> usize {
        let now = self.clock.now();
        self.lock().reclaim_expired(now, self.config.window).0
    }

    /// Test hook: whether the state lock is currently held (by any thread).
    #[cfg(test)]
    pub(crate) fn state_is_locked(&self) -> bool {
        matches!(
            self.state.try_lock(),
            Err(std::sync::TryLockError::WouldBlock)
        )
    }

    /// Number of keys currently tracked.
    pub fn tracked_keys(&self) -> usize {
        self.lock().windows.len()
    }
}

/// Limiter key for an authenticated principal, or `None` when the context
/// carries no stable identifier (the request must then be rejected).
///
/// Keys are namespaced so a login can never collide with a principal id:
/// `login:<lowercased login>` for users (GitHub logins are case-insensitive)
/// and `sub:<principal id>` for loginless principals. An empty login is not an
/// identifier.
pub fn principal_key(ctx: &AuthContext) -> Option<String> {
    if let Some(login) = ctx.user_login.as_deref().filter(|l| !l.is_empty()) {
        return Some(format!("login:{}", login.to_lowercase()));
    }
    ctx.principal_id
        .as_deref()
        .filter(|p| !p.is_empty())
        .map(|p| format!("sub:{p}"))
}

/// Limiter key for the client of a public request, derived from the
/// connection address only.
///
/// Uses the peer IP from `ConnectInfo<SocketAddr>` (port dropped, IPv4-mapped
/// IPv6 canonicalised to IPv4). IPv4 clients are keyed `ip:a.b.c.d`; IPv6
/// clients by their /64 prefix, `addr:v6/<16 hex digits>`, because a single
/// subscriber typically controls a whole /64 and could otherwise mint
/// unlimited keys. Forwarding headers (`X-Forwarded-For`, ...)
/// are deliberately never consulted: they are client-controlled and would let
/// an attacker evade the limit or exhaust another client's budget. Requests
/// without `ConnectInfo` share the fail-closed `NO_CLIENT_ADDR_KEY` bucket.
pub fn client_key(request: &Request) -> String {
    match request.extensions().get::<ConnectInfo<SocketAddr>>() {
        Some(ConnectInfo(addr)) => match addr.ip().to_canonical() {
            IpAddr::V4(v4) => format!("ip:{v4}"),
            IpAddr::V6(v6) => {
                let o = v6.octets();
                let prefix = u64::from_be_bytes([o[0], o[1], o[2], o[3], o[4], o[5], o[6], o[7]]);
                format!("addr:v6/{prefix:016x}")
            }
        },
        None => NO_CLIENT_ADDR_KEY.to_string(),
    }
}

/// Build the standard `429` response and count the rejection under `scope`.
fn reject(state: &AppState, scope: &str, retry_after_secs: u64) -> Response {
    state
        .rate_limit_rejections
        .with_label_values(&[scope])
        .inc();
    tracing::warn!(
        scope,
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

/// Per-client-address limiter for the public `POST /api/v1/auth/token`.
///
/// Only `POST` requests are limited. The bucket key is [`client_key`]; a
/// rejection is a `429` (see [`rate_limit_middleware`]) counted under
/// `scope="auth_ip"`. Uses its own limiter (`AppState::auth_rate_limiter`),
/// separate from the per-user budget.
pub async fn auth_rate_limit_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    if request.method() != Method::POST {
        return next.run(request).await;
    }
    match state.auth_rate_limiter.check(&client_key(&request)) {
        RateLimitDecision::Allowed { .. } => next.run(request).await,
        RateLimitDecision::Limited { retry_after_secs } => {
            reject(&state, SCOPE_AUTH_IP, retry_after_secs)
        }
    }
}

/// `true` for the methods that never consume rate-limit budget.
fn is_safe_method(method: &Method) -> bool {
    matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS)
}

/// Axum middleware enforcing the per-user limit (must run after
/// `auth_middleware`).
///
/// Every method except `GET`, `HEAD` and `OPTIONS` is limited (see
/// [`is_safe_method`]). The bucket key is [`principal_key`].
/// A request whose context carries no identifier (or no `AuthContext` at all)
/// is rejected with `401` instead of being pooled into a shared bucket.
/// Rejections return `429` with a `Retry-After` header and the standard JSON
/// error body, are logged at WARN (without the user identity) and counted in
/// `rate_limit_rejections_total{scope="user"}`.
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    if is_safe_method(request.method()) {
        return next.run(request).await;
    }

    let Some(ctx) = request.extensions().get::<AuthContext>() else {
        tracing::warn!("Rate limiter reached without an AuthContext; rejecting");
        return AuthError::MissingToken.into_response();
    };
    let Some(key) = principal_key(ctx) else {
        tracing::warn!("Authenticated request has no principal identifier; rejecting");
        return AuthError::MissingPrincipalIdentifier.into_response();
    };

    match state.rate_limiter.check(&key) {
        RateLimitDecision::Allowed { .. } => next.run(request).await,
        RateLimitDecision::Limited { retry_after_secs } => {
            reject(&state, SCOPE_USER, retry_after_secs)
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

#[cfg(test)]
#[path = "rate_limit_cap_tests.rs"]
mod cap_tests;

#[cfg(test)]
#[path = "rate_limit_scope_tests.rs"]
mod scope_tests;

#[cfg(test)]
#[path = "rate_limit_clock_order_tests.rs"]
mod clock_order_tests;
