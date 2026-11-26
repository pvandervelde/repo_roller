//! Tests for the rate_limit module: config parsing and limiter logic.
//!
//! All limiter tests use an injected `ManualClock`; none sleep.

use super::*;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Helpers (shared with rate_limit_proptests.rs and the router-level tests)
// ---------------------------------------------------------------------------

/// Deterministic clock advanced explicitly by tests.
pub(crate) struct ManualClock(Mutex<Duration>);

impl ManualClock {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self(Mutex::new(Duration::from_secs(1_000))))
    }
    pub(crate) fn advance(&self, d: Duration) {
        *self.0.lock().unwrap() += d;
    }
}

impl Clock for ManualClock {
    fn now(&self) -> Duration {
        *self.0.lock().unwrap()
    }
}

pub(crate) fn limiter(max: u32, window_secs: u64) -> (RateLimiter, Arc<ManualClock>) {
    let clock = ManualClock::new();
    let l = RateLimiter::with_clock(RateLimitConfig::new(true, max, window_secs), clock.clone());
    (l, clock)
}

pub(crate) fn is_allowed(d: RateLimitDecision) -> bool {
    matches!(d, RateLimitDecision::Allowed { .. })
}

pub(crate) fn retry_after(d: RateLimitDecision) -> u64 {
    match d {
        RateLimitDecision::Limited { retry_after_secs } => retry_after_secs,
        other => panic!("expected Limited, got {other:?}"),
    }
}

pub(crate) fn cfg_from(pairs: &[(&str, &str)]) -> RateLimitConfig {
    let map: HashMap<String, String> = pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    RateLimitConfig::from_env_vars(|k| map.get(k).cloned())
}

// ---------------------------------------------------------------------------
// Config parsing
// ---------------------------------------------------------------------------

mod config_tests {
    use super::*;

    #[test]
    fn explicit_values_are_stored() {
        let c = RateLimitConfig::new(false, 7, 30);
        assert!(!c.enabled);
        assert_eq!(c.max_requests, 7);
        assert_eq!(c.window, Duration::from_secs(30));
    }

    #[test]
    fn unset_env_yields_safe_defaults_enabled_and_nonzero() {
        let c = cfg_from(&[]);
        assert!(c.enabled, "rate limiting must be on by default");
        assert!(c.max_requests >= 1);
        assert!(c.window >= Duration::from_secs(1));
        assert_eq!(c, RateLimitConfig::default());
    }

    #[test]
    fn valid_values_are_parsed() {
        let c = cfg_from(&[
            ("RATE_LIMIT_ENABLED", "true"),
            ("RATE_LIMIT_REQUESTS", "25"),
            ("RATE_LIMIT_WINDOW_SECS", "120"),
        ]);
        assert!(c.enabled);
        assert_eq!(c.max_requests, 25);
        assert_eq!(c.window, Duration::from_secs(120));
    }

    #[test]
    fn requests_and_window_are_read_independently() {
        let c = cfg_from(&[("RATE_LIMIT_REQUESTS", "3")]);
        assert_eq!(c.max_requests, 3);
        assert_eq!(c.window, RateLimitConfig::default().window);
        let c = cfg_from(&[("RATE_LIMIT_WINDOW_SECS", "5")]);
        assert_eq!(c.window, Duration::from_secs(5));
        assert_eq!(c.max_requests, RateLimitConfig::default().max_requests);
    }

    #[test]
    fn enabled_false_variants_disable() {
        for v in ["false", "FALSE", "False", "0"] {
            let c = cfg_from(&[("RATE_LIMIT_ENABLED", v)]);
            assert!(!c.enabled, "'{v}' must disable rate limiting");
        }
    }

    #[test]
    fn enabled_true_variants_enable() {
        for v in ["true", "TRUE", "1"] {
            let c = cfg_from(&[("RATE_LIMIT_ENABLED", v)]);
            assert!(c.enabled, "'{v}' must enable rate limiting");
        }
    }

    #[test]
    fn unparseable_enabled_fails_closed_to_enabled() {
        for v in ["maybe", "", "yes please", "2"] {
            let c = cfg_from(&[("RATE_LIMIT_ENABLED", v)]);
            assert!(
                c.enabled,
                "garbage '{v}' must not silently disable limiting"
            );
        }
    }

    #[test]
    fn invalid_requests_fall_back_to_default() {
        let d = RateLimitConfig::default().max_requests;
        for v in ["abc", "-5", "", "1.5", "0", "99999999999999999999"] {
            let c = cfg_from(&[("RATE_LIMIT_REQUESTS", v)]);
            assert_eq!(c.max_requests, d, "'{v}' must fall back to default");
        }
    }

    #[test]
    fn invalid_window_falls_back_to_default() {
        let d = RateLimitConfig::default().window;
        for v in ["abc", "-1", "", "0", "1e3", "99999999999999999999999"] {
            let c = cfg_from(&[("RATE_LIMIT_WINDOW_SECS", v)]);
            assert_eq!(c.window, d, "'{v}' must fall back to default");
        }
    }

    #[test]
    fn invalid_value_does_not_poison_other_valid_values() {
        let c = cfg_from(&[
            ("RATE_LIMIT_REQUESTS", "nope"),
            ("RATE_LIMIT_WINDOW_SECS", "90"),
            ("RATE_LIMIT_ENABLED", "false"),
        ]);
        assert!(!c.enabled);
        assert_eq!(c.window, Duration::from_secs(90));
    }

    #[test]
    fn whitespace_around_numbers_is_tolerated_or_defaulted_never_panics() {
        let c = cfg_from(&[("RATE_LIMIT_REQUESTS", " 4 ")]);
        assert!(c.max_requests == 4 || c.max_requests == RateLimitConfig::default().max_requests);
    }

    #[test]
    fn limiter_exposes_its_config() {
        let (l, _) = limiter(4, 9);
        assert_eq!(l.config(), &RateLimitConfig::new(true, 4, 9));
    }
}

// ---------------------------------------------------------------------------
// Limiter logic
// ---------------------------------------------------------------------------

mod limiter_tests {
    use super::*;

    #[test]
    fn first_n_requests_allowed_then_limited() {
        let (l, _) = limiter(3, 60);
        assert!(is_allowed(l.check("alice")));
        assert!(is_allowed(l.check("alice")));
        assert!(is_allowed(l.check("alice")));
        assert!(
            !is_allowed(l.check("alice")),
            "N+1th request must be limited"
        );
    }

    #[test]
    fn limit_of_one_allows_exactly_one() {
        let (l, _) = limiter(1, 60);
        assert!(is_allowed(l.check("alice")));
        assert!(!is_allowed(l.check("alice")));
    }

    #[test]
    fn remaining_counts_down_to_zero() {
        let (l, _) = limiter(3, 60);
        assert_eq!(l.check("a"), RateLimitDecision::Allowed { remaining: 2 });
        assert_eq!(l.check("a"), RateLimitDecision::Allowed { remaining: 1 });
        assert_eq!(l.check("a"), RateLimitDecision::Allowed { remaining: 0 });
    }

    #[test]
    fn keeps_limiting_on_repeated_rejections() {
        let (l, _) = limiter(1, 60);
        l.check("a");
        for _ in 0..20 {
            assert!(!is_allowed(l.check("a")));
        }
    }

    #[test]
    fn users_are_isolated() {
        let (l, _) = limiter(2, 60);
        l.check("alice");
        l.check("alice");
        assert!(!is_allowed(l.check("alice")));
        assert_eq!(l.check("bob"), RateLimitDecision::Allowed { remaining: 1 });
        assert!(is_allowed(l.check("bob")));
        assert!(!is_allowed(l.check("bob")));
    }

    #[test]
    fn keys_are_case_sensitive_and_exact() {
        let (l, _) = limiter(1, 60);
        assert!(is_allowed(l.check("Alice")));
        assert!(
            is_allowed(l.check("alice")),
            "different key string => different bucket"
        );
        assert!(is_allowed(l.check("alice ")));
    }

    #[test]
    fn empty_key_is_handled_without_panic() {
        let (l, _) = limiter(1, 60);
        assert!(is_allowed(l.check("")));
        assert!(!is_allowed(l.check("")));
    }

    #[test]
    fn window_resets_after_expiry() {
        let (l, c) = limiter(2, 60);
        l.check("a");
        l.check("a");
        assert!(!is_allowed(l.check("a")));
        c.advance(Duration::from_secs(60));
        assert_eq!(l.check("a"), RateLimitDecision::Allowed { remaining: 1 });
    }

    #[test]
    fn window_boundary_n_minus_1_still_limited_n_allowed() {
        let (l, c) = limiter(1, 60);
        l.check("a");
        c.advance(Duration::from_millis(59_999));
        assert!(
            !is_allowed(l.check("a")),
            "1ms before window end must still be limited"
        );
        c.advance(Duration::from_millis(1));
        assert!(
            is_allowed(l.check("a")),
            "at exactly window end a new window begins"
        );
    }

    #[test]
    fn well_after_window_is_allowed_with_full_budget() {
        let (l, c) = limiter(3, 10);
        for _ in 0..3 {
            l.check("a");
        }
        c.advance(Duration::from_secs(3600));
        assert_eq!(l.check("a"), RateLimitDecision::Allowed { remaining: 2 });
    }

    #[test]
    fn rejected_requests_do_not_extend_the_window() {
        let (l, c) = limiter(1, 60);
        l.check("a");
        c.advance(Duration::from_secs(30));
        for _ in 0..10 {
            assert!(!is_allowed(l.check("a")));
        }
        c.advance(Duration::from_secs(30));
        assert!(
            is_allowed(l.check("a")),
            "hammering while limited must not push out the reset"
        );
    }

    #[test]
    fn window_starts_at_first_request_not_at_construction() {
        let (l, c) = limiter(1, 60);
        c.advance(Duration::from_secs(500));
        assert!(is_allowed(l.check("a")));
        c.advance(Duration::from_secs(59));
        assert!(!is_allowed(l.check("a")));
    }

    #[test]
    fn user_windows_are_independent_in_time() {
        let (l, c) = limiter(1, 60);
        l.check("early");
        c.advance(Duration::from_secs(40));
        l.check("late");
        c.advance(Duration::from_secs(20)); // early window over, late has 40s left
        assert!(is_allowed(l.check("early")));
        assert!(!is_allowed(l.check("late")));
    }

    #[test]
    fn disabled_limiter_always_allows() {
        let clock = ManualClock::new();
        let l = RateLimiter::with_clock(RateLimitConfig::new(false, 1, 60), clock);
        for _ in 0..50 {
            assert!(is_allowed(l.check("a")));
        }
    }

    #[test]
    fn disabled_limiter_tracks_no_keys() {
        let clock = ManualClock::new();
        let l = RateLimiter::with_clock(RateLimitConfig::new(false, 1, 60), clock);
        l.check("a");
        l.check("b");
        assert_eq!(l.tracked_keys(), 0);
    }

    #[test]
    fn concurrent_checks_never_exceed_the_limit() {
        use std::sync::atomic::{AtomicU32, Ordering};
        let clock = ManualClock::new();
        let l = Arc::new(RateLimiter::with_clock(
            RateLimitConfig::new(true, 50, 60),
            clock,
        ));
        let allowed = Arc::new(AtomicU32::new(0));
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let l = l.clone();
                let allowed = allowed.clone();
                std::thread::spawn(move || {
                    for _ in 0..40 {
                        if is_allowed(l.check("shared")) {
                            allowed.fetch_add(1, Ordering::SeqCst);
                        }
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(allowed.load(Ordering::SeqCst), 50);
    }
}

// ---------------------------------------------------------------------------
// Retry-After calculation
// ---------------------------------------------------------------------------

mod retry_after_tests {
    use super::*;

    #[test]
    fn immediately_after_exhaustion_equals_full_window() {
        let (l, _) = limiter(1, 60);
        l.check("a");
        assert_eq!(retry_after(l.check("a")), 60);
    }

    #[test]
    fn counts_down_as_time_passes() {
        let (l, c) = limiter(1, 60);
        l.check("a");
        c.advance(Duration::from_secs(10));
        assert_eq!(retry_after(l.check("a")), 50);
    }

    #[test]
    fn fractional_seconds_round_up() {
        let (l, c) = limiter(1, 60);
        l.check("a");
        c.advance(Duration::from_millis(10_500));
        assert_eq!(
            retry_after(l.check("a")),
            50,
            "49.5s remaining must round up to 50"
        );
    }

    #[test]
    fn never_zero_even_one_millisecond_before_reset() {
        let (l, c) = limiter(1, 60);
        l.check("a");
        c.advance(Duration::from_millis(59_999));
        assert_eq!(retry_after(l.check("a")), 1);
    }

    #[test]
    fn measured_from_first_request_of_window_and_never_exceeds_window() {
        let (l, c) = limiter(2, 30);
        l.check("a");
        c.advance(Duration::from_secs(7));
        l.check("a");
        let r = retry_after(l.check("a"));
        assert_eq!(r, 23);
        assert!(r <= 30);
    }

    #[test]
    fn honours_configured_window_length() {
        let (l, _) = limiter(1, 5);
        l.check("a");
        assert_eq!(retry_after(l.check("a")), 5);
    }

    #[test]
    fn waiting_the_advertised_retry_after_makes_the_next_request_succeed() {
        let (l, c) = limiter(1, 60);
        l.check("a");
        c.advance(Duration::from_millis(12_300));
        let r = retry_after(l.check("a"));
        c.advance(Duration::from_secs(r));
        assert!(
            is_allowed(l.check("a")),
            "client obeying Retry-After must not be rejected again"
        );
    }
}

// ---------------------------------------------------------------------------
// Eviction / bounded memory
// ---------------------------------------------------------------------------

mod eviction_tests {
    use super::*;

    #[test]
    fn tracked_keys_counts_distinct_keys() {
        let (l, _) = limiter(5, 60);
        assert_eq!(l.tracked_keys(), 0);
        l.check("a");
        l.check("a");
        l.check("b");
        assert_eq!(l.tracked_keys(), 2);
    }

    #[test]
    fn evict_removes_expired_keys_and_reports_count() {
        let (l, c) = limiter(5, 60);
        for i in 0..100 {
            l.check(&format!("user-{i}"));
        }
        assert_eq!(l.tracked_keys(), 100);
        c.advance(Duration::from_secs(60));
        assert_eq!(l.evict_expired(), 100);
        assert_eq!(l.tracked_keys(), 0);
    }

    #[test]
    fn evict_keeps_keys_with_live_windows() {
        let (l, c) = limiter(1, 60);
        l.check("old");
        c.advance(Duration::from_secs(40));
        l.check("fresh");
        c.advance(Duration::from_secs(20));
        assert_eq!(l.evict_expired(), 1);
        assert_eq!(l.tracked_keys(), 1);
        assert!(
            !is_allowed(l.check("fresh")),
            "evicting must not reset a live window"
        );
    }

    #[test]
    fn evict_does_not_remove_key_one_ms_before_expiry() {
        let (l, c) = limiter(1, 60);
        l.check("a");
        c.advance(Duration::from_millis(59_999));
        assert_eq!(l.evict_expired(), 0);
        assert_eq!(l.tracked_keys(), 1);
    }

    #[test]
    fn evict_on_empty_limiter_is_zero() {
        let (l, _) = limiter(1, 60);
        assert_eq!(l.evict_expired(), 0);
    }

    #[test]
    fn evicted_key_gets_a_fresh_full_budget() {
        let (l, c) = limiter(2, 60);
        l.check("a");
        l.check("a");
        c.advance(Duration::from_secs(61));
        l.evict_expired();
        assert_eq!(l.check("a"), RateLimitDecision::Allowed { remaining: 1 });
    }

    #[test]
    fn rejected_key_stays_tracked_until_window_ends() {
        let (l, c) = limiter(1, 60);
        l.check("a");
        l.check("a");
        c.advance(Duration::from_secs(10));
        assert_eq!(l.evict_expired(), 0);
        assert_eq!(l.tracked_keys(), 1);
    }
}
