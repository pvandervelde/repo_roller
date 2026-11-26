//! Application-level rate limiting (Task #284).
//!
//! TDD RED PHASE: only types, constructors and signatures exist. Behaviour
//! (`from_env_vars`, `check`, `evict_expired`, `tracked_keys`,
//! `SystemClock::now`, `rate_limit_middleware`) is stubbed and MUST be
//! implemented by the Coder.
//!
//! # Contract (assumed by the tests)
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

#![allow(dead_code, unused_variables)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::AppState;

/// Default maximum requests per window.
pub const DEFAULT_MAX_REQUESTS: u32 = 10;
/// Default window length in seconds.
pub const DEFAULT_WINDOW_SECS: u64 = 60;

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
    pub fn from_env_vars(lookup: impl Fn(&str) -> Option<String>) -> Self {
        todo!("TDD stub")
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
        todo!("TDD stub")
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

/// Per-key fixed-window limiter.
pub struct RateLimiter {
    config: RateLimitConfig,
    clock: Arc<dyn Clock>,
    state: Mutex<HashMap<String, (Duration, u32)>>,
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
            state: Mutex::new(HashMap::new()),
        }
    }

    /// The active configuration.
    pub fn config(&self) -> &RateLimitConfig {
        &self.config
    }

    /// Record one request for `key` and decide. Always `Allowed` if disabled.
    pub fn check(&self, key: &str) -> RateLimitDecision {
        todo!("TDD stub")
    }

    /// Drop every key whose window has ended. Returns number removed.
    pub fn evict_expired(&self) -> usize {
        todo!("TDD stub")
    }

    /// Number of keys currently tracked.
    pub fn tracked_keys(&self) -> usize {
        todo!("TDD stub")
    }
}

/// Axum middleware enforcing the limit (must run after `auth_middleware`).
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    next.run(request).await
}

#[cfg(test)]
#[path = "rate_limit_tests.rs"]
pub(crate) mod tests;

#[cfg(test)]
#[path = "rate_limit_proptests.rs"]
mod proptests;
