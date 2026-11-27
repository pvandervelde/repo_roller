//! Clock-ordering invariant tests for `RateLimiter`.
//!
//! Contract: the limiter takes its state lock FIRST and reads `clock.now()`
//! AFTER, so the expiry queue's order always equals window-start order. Reading
//! the clock before locking lets a slow thread insert a stale (earlier)
//! timestamp behind a later one, which stalls reclamation of expired keys.

use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

/// Clock that records, on every `now()`, whether the limiter state lock is held.
struct LockObservingClock {
    limiter: OnceLock<Arc<RateLimiter>>,
    observations: Mutex<Vec<bool>>,
}

impl LockObservingClock {
    fn limiter_with(config: RateLimitConfig) -> (Arc<RateLimiter>, Arc<Self>) {
        let clock = Arc::new(Self {
            limiter: OnceLock::new(),
            observations: Mutex::new(Vec::new()),
        });
        let l = Arc::new(RateLimiter::with_clock(config, clock.clone()));
        assert!(clock.limiter.set(l.clone()).is_ok());
        (l, clock)
    }
    fn observed(&self) -> Vec<bool> {
        self.observations.lock().unwrap().clone()
    }
}

impl Clock for LockObservingClock {
    fn now(&self) -> Duration {
        let locked = self.limiter.get().is_some_and(|l| l.state_is_locked());
        self.observations.lock().unwrap().push(locked);
        Duration::from_secs(1_000)
    }
}

#[test]
fn state_lock_hook_reports_unlocked_when_idle() {
    let (l, _c) = LockObservingClock::limiter_with(RateLimitConfig::new(true, 5, 60));
    assert!(!l.state_is_locked());
}

#[test]
fn check_reads_clock_exactly_once() {
    let (l, c) = LockObservingClock::limiter_with(RateLimitConfig::new(true, 5, 60));
    l.check("k");
    assert_eq!(c.observed().len(), 1);
}

#[test]
fn check_reads_clock_only_while_holding_state_lock() {
    let (l, c) = LockObservingClock::limiter_with(RateLimitConfig::new(true, 5, 60));
    l.check("k");
    assert_eq!(c.observed(), vec![true]);
}

#[test]
fn check_of_limited_key_reads_clock_only_while_holding_state_lock() {
    let (l, c) = LockObservingClock::limiter_with(RateLimitConfig::new(true, 1, 60));
    l.check("k");
    l.check("k");
    assert_eq!(c.observed(), vec![true, true]);
}

#[test]
fn evict_expired_reads_clock_only_while_holding_state_lock() {
    let (l, c) = LockObservingClock::limiter_with(RateLimitConfig::new(true, 5, 60));
    l.evict_expired();
    assert_eq!(c.observed(), vec![true]);
}

/// Clock whose first reader stalls after reading, to let a second caller race
/// past it. Reads are numbered in the order `now()` is entered.
struct StallingClock {
    calls: AtomicUsize,
}

impl Clock for StallingClock {
    fn now(&self) -> Duration {
        match self.calls.fetch_add(1, Ordering::SeqCst) {
            0 => {
                let t = Duration::from_secs(1_000);
                std::thread::sleep(Duration::from_millis(400));
                t
            }
            1 => Duration::from_secs(1_030),
            _ => Duration::from_secs(1_060),
        }
    }
}

#[test]
fn slow_first_caller_cannot_insert_stale_timestamp_behind_later_one() {
    let l = Arc::new(RateLimiter::with_clock(
        RateLimitConfig::new(true, 5, 60),
        Arc::new(StallingClock {
            calls: AtomicUsize::new(0),
        }),
    ));
    let a = {
        let l = l.clone();
        std::thread::spawn(move || l.check("a"))
    };
    std::thread::sleep(Duration::from_millis(100));
    let b = {
        let l = l.clone();
        std::thread::spawn(move || l.check("b"))
    };
    a.join().unwrap();
    b.join().unwrap();
    // Third check happens at t=1060: "a" (start 1000) has expired and must be
    // reclaimed; "b" (start 1030) is live. Tracked afterwards: b and c.
    l.check("c");
    assert_eq!(l.tracked_keys(), 2);
}
