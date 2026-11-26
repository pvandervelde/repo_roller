//! Tests for the hard cap on tracked keys (`RateLimiter::with_max_keys`).
//!
//! Contract under test:
//!
//! * `tracked_keys()` never exceeds `max_keys()` (default 10,000).
//! * When a new key arrives at a full limiter, expired keys are reclaimed
//!   first (amortised O(1), no full sweep per insert).
//! * If still full: `FullPolicy::EvictOldest` evicts the key whose window
//!   started earliest and admits the new key; `FullPolicy::FailClosed`
//!   answers `Limited { retry_after_secs >= 1 }` and does NOT insert.
//! * Work per `check` is bounded: `max_check_work()` (entries visited by the
//!   costliest single check) stays a small constant when nothing expires, and
//!   `total_check_work()` is amortised O(1) per check when many keys expire.
//!
//! All tests use `ManualClock`; none sleep.

use super::tests::{is_allowed, retry_after, ManualClock};
use super::*;
use proptest::prelude::*;

/// Small constant bound on entries visited by any one check (no expiries).
const PER_CHECK_WORK_BOUND: usize = 32;

fn capped(
    max_requests: u32,
    window_secs: u64,
    cap: usize,
    policy: FullPolicy,
) -> (RateLimiter, Arc<ManualClock>) {
    let clock = ManualClock::new();
    let l = RateLimiter::with_max_keys(
        RateLimitConfig::new(true, max_requests, window_secs),
        clock.clone(),
        cap,
        policy,
    );
    (l, clock)
}

// ---------------------------------------------------------------------------
// Construction / defaults
// ---------------------------------------------------------------------------

#[test]
fn default_cap_is_ten_thousand_and_policy_is_evict_oldest() {
    let l = RateLimiter::new(RateLimitConfig::default());
    assert_eq!(DEFAULT_MAX_TRACKED_KEYS, 10_000);
    assert_eq!(l.max_keys(), 10_000);
    assert_eq!(l.full_policy(), FullPolicy::EvictOldest);
}

#[test]
fn with_clock_uses_default_cap() {
    let (l, _c) = super::tests::limiter(1, 60);
    assert_eq!(l.max_keys(), 10_000);
}

#[test]
fn with_max_keys_stores_cap_and_policy() {
    let (l, _c) = capped(1, 60, 7, FullPolicy::FailClosed);
    assert_eq!(l.max_keys(), 7);
    assert_eq!(l.full_policy(), FullPolicy::FailClosed);
}

// ---------------------------------------------------------------------------
// EvictOldest
// ---------------------------------------------------------------------------

#[test]
fn evict_oldest_flood_never_exceeds_cap_and_admits_every_key() {
    let (l, clock) = capped(5, 60, 100, FullPolicy::EvictOldest);
    for i in 0..1_000 {
        clock.advance(Duration::from_millis(1));
        assert!(
            is_allowed(l.check(&format!("k{i}"))),
            "key {i} must be admitted"
        );
        assert!(l.tracked_keys() <= 100, "cap exceeded at insert {i}");
    }
    assert_eq!(
        l.tracked_keys(),
        100,
        "a full limiter stays full under a flood"
    );
}

#[test]
fn evict_oldest_drops_least_recently_started_key_and_keeps_the_rest() {
    let (l, clock) = capped(1, 600, 3, FullPolicy::EvictOldest);
    for k in ["a", "b", "c"] {
        assert!(is_allowed(l.check(k)));
        clock.advance(Duration::from_secs(1));
    }
    assert!(
        is_allowed(l.check("d")),
        "new key admitted by evicting oldest"
    );
    assert_eq!(l.tracked_keys(), 3);
    // b and c kept their (exhausted) budgets.
    assert!(!is_allowed(l.check("b")), "b must still be tracked");
    assert!(!is_allowed(l.check("c")), "c must still be tracked");
    assert!(!is_allowed(l.check("d")), "d must be tracked");
    assert_eq!(l.tracked_keys(), 3);
}

#[test]
fn evict_oldest_evicted_key_returns_with_a_fresh_budget() {
    let (l, clock) = capped(1, 600, 2, FullPolicy::EvictOldest);
    assert!(is_allowed(l.check("a")));
    clock.advance(Duration::from_secs(1));
    assert!(is_allowed(l.check("b")));
    clock.advance(Duration::from_secs(1));
    assert!(is_allowed(l.check("c"))); // evicts a
    assert!(
        is_allowed(l.check("a")),
        "a was evicted, so it is a new key with a fresh window"
    );
    assert!(l.tracked_keys() <= 2);
}

#[test]
fn evict_oldest_uses_restarted_window_start_not_first_insertion_time() {
    // a starts at t0, b at t10; a's window expires at t60 and restarts at t60
    // (newer than b). At t61 a new key must evict b, never the freshly
    // restarted a (a stale expiry-queue entry for a must be ignored).
    let (l, clock) = capped(1, 60, 2, FullPolicy::EvictOldest);
    assert!(is_allowed(l.check("a")));
    clock.advance(Duration::from_secs(10));
    assert!(is_allowed(l.check("b")));
    clock.advance(Duration::from_secs(50)); // t60: a expired, b not
    assert!(is_allowed(l.check("a")), "a restarts at t60");
    clock.advance(Duration::from_secs(1)); // t61
    assert!(is_allowed(l.check("c")));
    assert_eq!(l.tracked_keys(), 2);
    assert!(!is_allowed(l.check("a")), "restarted a must have survived");
    assert!(!is_allowed(l.check("c")));
}

// ---------------------------------------------------------------------------
// FailClosed
// ---------------------------------------------------------------------------

#[test]
fn fail_closed_rejects_new_key_when_full_without_inserting_it() {
    let (l, _c) = capped(2, 60, 3, FullPolicy::FailClosed);
    for k in ["a", "b", "c"] {
        assert!(is_allowed(l.check(k)));
    }
    let d = l.check("d");
    assert!(
        matches!(d, RateLimitDecision::Limited { .. }),
        "expected Limited for new key at cap, got {d:?}"
    );
    assert_eq!(l.tracked_keys(), 3, "rejected key must not be inserted");
}

#[test]
fn fail_closed_rejection_retry_after_is_between_one_and_window_secs() {
    let (l, _c) = capped(2, 60, 1, FullPolicy::FailClosed);
    assert!(is_allowed(l.check("a")));
    let r = retry_after(l.check("b"));
    assert!((1..=60).contains(&r), "retry_after {r} out of [1, 60]");
}

#[test]
fn fail_closed_repeated_attempts_of_the_same_new_key_never_sneak_in() {
    let (l, _c) = capped(5, 60, 1, FullPolicy::FailClosed);
    assert!(is_allowed(l.check("a")));
    for _ in 0..10 {
        assert!(!is_allowed(l.check("b")), "b must never sneak in");
    }
    assert_eq!(l.tracked_keys(), 1);
}

#[test]
fn fail_closed_flood_does_not_touch_existing_keys_budgets() {
    let (l, _c) = capped(2, 60, 50, FullPolicy::FailClosed);
    for i in 0..50 {
        assert!(is_allowed(l.check(&format!("legit{i}"))), "legit{i}");
    }
    for i in 0..500 {
        assert!(
            !is_allowed(l.check(&format!("attacker{i}"))),
            "attacker{i} must be rejected while full"
        );
        assert!(l.tracked_keys() <= 50);
    }
    assert_eq!(l.tracked_keys(), 50);
    // Each legit key used 1 of 2 requests: exactly one more is allowed.
    for i in 0..50 {
        let k = format!("legit{i}");
        assert_eq!(
            l.check(&k),
            RateLimitDecision::Allowed { remaining: 0 },
            "{k} budget was disturbed by the flood"
        );
        assert!(!is_allowed(l.check(&k)), "{k} must now be limited");
    }
}

#[test]
fn fail_closed_existing_key_is_still_served_and_limited_normally_when_full() {
    let (l, _c) = capped(2, 60, 2, FullPolicy::FailClosed);
    assert!(is_allowed(l.check("a")));
    assert!(is_allowed(l.check("b")));
    assert!(is_allowed(l.check("a")));
    assert!(!is_allowed(l.check("a")));
    assert_eq!(l.tracked_keys(), 2);
}

#[test]
fn fail_closed_admits_new_keys_again_once_the_window_has_passed() {
    let (l, clock) = capped(1, 60, 2, FullPolicy::FailClosed);
    assert!(is_allowed(l.check("a")));
    assert!(is_allowed(l.check("b")));
    assert!(!is_allowed(l.check("c")));
    clock.advance(Duration::from_secs(60));
    assert!(is_allowed(l.check("c")), "expired keys must be reclaimed");
    assert!(l.tracked_keys() <= 2);
}

// ---------------------------------------------------------------------------
// Reclamation of expired keys happens before the policy applies
// ---------------------------------------------------------------------------

#[test]
fn expired_keys_are_reclaimed_before_policy_for_both_policies() {
    for policy in [FullPolicy::EvictOldest, FullPolicy::FailClosed] {
        let (l, clock) = capped(1, 60, 3, policy);
        for k in ["a", "b", "c"] {
            assert!(is_allowed(l.check(k)));
        }
        clock.advance(Duration::from_secs(60));
        assert!(is_allowed(l.check("d")), "{policy:?}: d admitted");
        assert!(l.tracked_keys() <= 3, "{policy:?}");
    }
}

#[test]
fn only_expired_keys_are_reclaimed_live_keys_keep_their_budget() {
    for policy in [FullPolicy::EvictOldest, FullPolicy::FailClosed] {
        let (l, clock) = capped(1, 60, 3, policy);
        assert!(is_allowed(l.check("old")));
        clock.advance(Duration::from_secs(30));
        assert!(is_allowed(l.check("b")));
        assert!(is_allowed(l.check("c")));
        clock.advance(Duration::from_secs(30)); // old expired, b/c live
        assert!(is_allowed(l.check("d")), "{policy:?}: d admitted");
        assert!(!is_allowed(l.check("b")), "{policy:?}: b must be retained");
        assert!(!is_allowed(l.check("c")), "{policy:?}: c must be retained");
        assert!(!is_allowed(l.check("d")), "{policy:?}: d tracked");
        assert_eq!(l.tracked_keys(), 3, "{policy:?}");
    }
}

#[test]
fn restarted_key_is_not_reclaimed_by_its_stale_expiry_entry() {
    for policy in [FullPolicy::EvictOldest, FullPolicy::FailClosed] {
        let (l, clock) = capped(1, 60, 2, policy);
        assert!(is_allowed(l.check("a"))); // t0
        clock.advance(Duration::from_secs(10));
        assert!(is_allowed(l.check("b"))); // t10
        clock.advance(Duration::from_secs(50)); // t60
        assert!(is_allowed(l.check("a"))); // a restarts at t60
        clock.advance(Duration::from_secs(10)); // t70: b expired, a live
        assert!(
            is_allowed(l.check("c")),
            "{policy:?}: c admitted via b's slot"
        );
        assert!(
            !is_allowed(l.check("a")),
            "{policy:?}: restarted a must keep its budget"
        );
    }
}

#[test]
fn disabled_limiter_tracks_nothing() {
    let clock = ManualClock::new();
    let l = RateLimiter::with_max_keys(
        RateLimitConfig::new(false, 1, 60),
        clock,
        2,
        FullPolicy::FailClosed,
    );
    for i in 0..10 {
        assert!(is_allowed(l.check(&format!("k{i}"))));
    }
    assert_eq!(l.tracked_keys(), 0);
}

#[test]
fn cap_of_one_holds_for_both_policies() {
    let (e, _c) = capped(1, 60, 1, FullPolicy::EvictOldest);
    assert!(is_allowed(e.check("a")));
    assert!(is_allowed(e.check("b")));
    assert_eq!(e.tracked_keys(), 1);
    let (f, _c) = capped(1, 60, 1, FullPolicy::FailClosed);
    assert!(is_allowed(f.check("a")));
    assert!(!is_allowed(f.check("b")));
    assert_eq!(f.tracked_keys(), 1);
}

#[test]
fn exactly_cap_keys_are_all_admitted_without_eviction() {
    for policy in [FullPolicy::EvictOldest, FullPolicy::FailClosed] {
        let (l, _c) = capped(1, 60, 10, policy);
        for i in 0..10 {
            assert!(is_allowed(l.check(&format!("k{i}"))), "{policy:?} k{i}");
        }
        assert_eq!(l.tracked_keys(), 10, "{policy:?}");
        for i in 0..10 {
            assert!(
                !is_allowed(l.check(&format!("k{i}"))),
                "{policy:?} k{i} kept"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Bounded work (instrumented, not wall-clock)
// ---------------------------------------------------------------------------

#[test]
fn work_is_instrumented_and_nonzero() {
    let (l, _c) = capped(1, 60, 10, FullPolicy::EvictOldest);
    l.check("a");
    assert!(
        l.max_check_work() >= 1,
        "a check must visit at least one entry"
    );
    assert!(l.total_check_work() >= 1);
}

#[test]
fn evict_oldest_flood_has_constant_per_check_work() {
    let (l, clock) = capped(5, 60, 1_000, FullPolicy::EvictOldest);
    for i in 0..10_000 {
        clock.advance(Duration::from_millis(1));
        l.check(&format!("k{i}"));
        assert!(l.tracked_keys() <= 1_000);
    }
    assert!(
        l.max_check_work() <= PER_CHECK_WORK_BOUND,
        "a single check visited {} entries (cap 1000): O(n) work per insert",
        l.max_check_work()
    );
}

#[test]
fn fail_closed_flood_has_constant_per_check_work() {
    let (l, clock) = capped(5, 60, 1_000, FullPolicy::FailClosed);
    for i in 0..10_000 {
        clock.advance(Duration::from_millis(1));
        l.check(&format!("k{i}"));
        assert!(l.tracked_keys() <= 1_000);
    }
    assert!(
        l.max_check_work() <= PER_CHECK_WORK_BOUND,
        "a single check visited {} entries (cap 1000): O(n) work per insert",
        l.max_check_work()
    );
}

#[test]
fn periodic_checks_on_existing_keys_never_trigger_a_full_sweep() {
    // 1000 live keys, then 5000 hits on them; the old "sweep every 256
    // checks" behaviour would visit all 1000 entries on some check.
    let (l, _c) = capped(100_000, 60, 1_000, FullPolicy::FailClosed);
    for i in 0..1_000 {
        l.check(&format!("k{i}"));
    }
    for i in 0..5_000 {
        l.check(&format!("k{}", i % 1_000));
    }
    assert!(
        l.max_check_work() <= PER_CHECK_WORK_BOUND,
        "got {}",
        l.max_check_work()
    );
}

#[test]
fn mass_expiry_reclamation_is_amortised_constant_per_check() {
    for policy in [FullPolicy::EvictOldest, FullPolicy::FailClosed] {
        let (l, clock) = capped(1, 60, 1_000, policy);
        let mut checks = 0usize;
        for round in 0..5 {
            for i in 0..1_000 {
                assert!(is_allowed(l.check(&format!("r{round}k{i}"))), "{policy:?}");
                checks += 1;
            }
            clock.advance(Duration::from_secs(60));
        }
        assert!(l.tracked_keys() <= 1_000);
        assert!(
            l.total_check_work() <= 8 * checks,
            "{policy:?}: total work {} for {checks} checks is not amortised O(1)",
            l.total_check_work()
        );
    }
}

// ---------------------------------------------------------------------------
// Properties
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum Step {
    Check(u8),
    Advance(u32),
}

fn step_strategy() -> impl Strategy<Value = Step> {
    prop_oneof![
        3 => (0u8..40).prop_map(Step::Check),
        1 => (0u32..30_000).prop_map(Step::Advance),
    ]
}

proptest! {
    #[test]
    fn tracked_keys_never_exceeds_cap_under_any_sequence(
        cap in 1usize..12,
        fail_closed in any::<bool>(),
        steps in proptest::collection::vec(step_strategy(), 1..200),
    ) {
        let policy = if fail_closed { FullPolicy::FailClosed } else { FullPolicy::EvictOldest };
        let (l, clock) = capped(2, 20, cap, policy);
        for s in steps {
            match s {
                Step::Check(k) => { l.check(&format!("k{k}")); }
                Step::Advance(ms) => clock.advance(Duration::from_millis(u64::from(ms))),
            }
            prop_assert!(l.tracked_keys() <= cap);
        }
    }

    #[test]
    fn fail_closed_never_allows_more_than_max_per_fixed_window(
        cap in 1usize..8,
        steps in proptest::collection::vec(step_strategy(), 1..300),
    ) {
        let window = Duration::from_secs(20);
        let max = 2u32;
        let (l, clock) = capped(max, 20, cap, FullPolicy::FailClosed);
        let mut now = Duration::ZERO;
        // model: key -> (window start, allowed in window)
        let mut model: std::collections::HashMap<u8, (Duration, u32)> = Default::default();
        for s in steps {
            match s {
                Step::Advance(ms) => {
                    let d = Duration::from_millis(u64::from(ms));
                    clock.advance(d);
                    now += d;
                }
                Step::Check(k) => {
                    if is_allowed(l.check(&format!("k{k}"))) {
                        let e = model.entry(k).or_insert((now, 0));
                        if now - e.0 >= window {
                            *e = (now, 0);
                        }
                        e.1 += 1;
                        prop_assert!(
                            e.1 <= max,
                            "key {} allowed {} times in one window (premature eviction?)",
                            k,
                            e.1
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn fail_closed_rejection_never_grows_tracked_keys(
        cap in 1usize..8,
        n in 1usize..60,
    ) {
        let (l, _c) = capped(3, 60, cap, FullPolicy::FailClosed);
        for i in 0..n {
            let before = l.tracked_keys();
            let d = l.check(&format!("k{i}"));
            if !is_allowed(d) {
                prop_assert_eq!(l.tracked_keys(), before);
                prop_assert!(retry_after(d) >= 1);
            }
        }
    }
}
