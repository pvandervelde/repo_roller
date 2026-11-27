//! Property tests for the rate limiter and config parsing.

use super::tests::{cfg_from, is_allowed, limiter, retry_after};
use super::*;
use proptest::prelude::*;

proptest! {
    /// Within one window, exactly min(requests, max) are allowed.
    #[test]
    fn allowed_count_never_exceeds_max(max in 1u32..50, reqs in 0usize..200) {
        let (l, _) = limiter(max, 60);
        let allowed = (0..reqs).filter(|_| is_allowed(l.check("k"))).count();
        prop_assert_eq!(allowed, reqs.min(max as usize));
    }

    /// Any interleaving of users: each user's allowed count is capped independently.
    #[test]
    fn per_user_budgets_are_independent(
        max in 1u32..10,
        seq in proptest::collection::vec(0usize..5, 0..200)
    ) {
        let (l, _) = limiter(max, 60);
        let mut allowed = [0u32; 5];
        let mut sent = [0u32; 5];
        for u in seq {
            sent[u] += 1;
            if is_allowed(l.check(&format!("u{u}"))) { allowed[u] += 1; }
        }
        for u in 0..5 {
            prop_assert_eq!(allowed[u], sent[u].min(max));
        }
    }

    /// Retry-After is always within 1..=window while limited.
    #[test]
    fn retry_after_always_in_bounds(max in 1u32..5, window in 1u64..3600, offset_ms in 0u64..3_600_000) {
        let (l, c) = limiter(max, window);
        for _ in 0..max { l.check("k"); }
        c.advance(Duration::from_millis(offset_ms % (window * 1000)));
        let r = retry_after(l.check("k"));
        prop_assert!(r >= 1 && r <= window, "retry_after {} outside 1..={}", r, window);
    }

    /// After advancing by the advertised Retry-After the next request is allowed.
    #[test]
    fn obeying_retry_after_always_succeeds(max in 1u32..5, window in 1u64..600, offset_ms in 0u64..600_000) {
        let (l, c) = limiter(max, window);
        for _ in 0..max { l.check("k"); }
        c.advance(Duration::from_millis(offset_ms % (window * 1000)));
        let r = retry_after(l.check("k"));
        c.advance(Duration::from_secs(r));
        prop_assert!(is_allowed(l.check("k")));
    }

    /// After eviction past the window nothing is tracked (bounded memory).
    #[test]
    fn eviction_bounds_memory(n in 0usize..300, window in 1u64..100) {
        let (l, c) = limiter(2, window);
        for i in 0..n { l.check(&format!("k{i}")); }
        c.advance(Duration::from_secs(window));
        l.evict_expired();
        prop_assert_eq!(l.tracked_keys(), 0);
    }

    /// Config parsing never panics and always yields usable limits.
    #[test]
    fn config_parsing_never_panics_and_is_always_usable(
        e in ".{0,12}", r in ".{0,24}", w in ".{0,24}"
    ) {
        let c = cfg_from(&[
            ("RATE_LIMIT_ENABLED", &e),
            ("RATE_LIMIT_REQUESTS", &r),
            ("RATE_LIMIT_WINDOW_SECS", &w),
        ]);
        prop_assert!(c.max_requests >= 1);
        prop_assert!(c.window >= Duration::from_secs(1));
    }

    /// Any positive integer values round-trip.
    #[test]
    fn positive_numbers_round_trip(r in 1u32..=u32::MAX, w in 1u64..=10_000_000) {
        let c = cfg_from(&[
            ("RATE_LIMIT_REQUESTS", &r.to_string()),
            ("RATE_LIMIT_WINDOW_SECS", &w.to_string()),
        ]);
        prop_assert_eq!(c.max_requests, r);
        prop_assert_eq!(c.window, Duration::from_secs(w));
    }

    /// Arbitrary keys (unicode, long) never panic the limiter.
    #[test]
    fn arbitrary_keys_do_not_panic(key in ".{0,200}") {
        let (l, _) = limiter(1, 60);
        prop_assert!(is_allowed(l.check(&key)));
        prop_assert!(!is_allowed(l.check(&key)));
    }
}
