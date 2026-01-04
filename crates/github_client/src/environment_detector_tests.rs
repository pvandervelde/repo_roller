//! Tests for GitHub environment detector implementation.

use super::*;
use std::sync::Arc;

// Note: These are unit tests for the detector structure and caching logic.
// Integration tests that hit the real GitHub API are in the integration_tests crate.

#[tokio::test]
async fn test_cache_initialization() {
    let client = Arc::new(Octocrab::builder().build().unwrap());
    let detector = GitHubApiEnvironmentDetector::new(client);

    // Cache should start empty
    let cache = detector.cache.read().await;
    assert_eq!(cache.len(), 0);
}

#[tokio::test]
async fn test_cached_limitations_is_valid_within_ttl() {
    let limitations = PlanLimitations::enterprise();
    let cached = CachedLimitations::new(limitations);

    // Should be valid immediately
    assert!(cached.is_valid(Duration::from_secs(3600)));

    // Should still be valid after a short time
    tokio::time::sleep(Duration::from_millis(10)).await;
    assert!(cached.is_valid(Duration::from_secs(3600)));
}

#[tokio::test]
async fn test_cached_limitations_expires_after_ttl() {
    let limitations = PlanLimitations::enterprise();
    let cached = CachedLimitations::new(limitations);

    // Should not be valid with a TTL of 0
    assert!(!cached.is_valid(Duration::from_secs(0)));
}

#[tokio::test]
async fn test_cached_limitations_clone() {
    let limitations = PlanLimitations::enterprise();
    let cached = CachedLimitations::new(limitations.clone());

    assert_eq!(cached.limitations, limitations);
}

#[tokio::test]
async fn test_detector_cache_ttl_is_one_hour() {
    let client = Arc::new(Octocrab::builder().build().unwrap());
    let detector = GitHubApiEnvironmentDetector::new(client);

    assert_eq!(detector.cache_ttl, Duration::from_secs(3600));
}

#[tokio::test]
async fn test_clear_cache() {
    let client = Arc::new(Octocrab::builder().build().unwrap());
    let detector = GitHubApiEnvironmentDetector::new(client);

    // Manually add something to cache
    {
        let mut cache = detector.cache.write().await;
        cache.insert(
            "test-org".to_string(),
            CachedLimitations::new(PlanLimitations::enterprise()),
        );
    }

    // Verify cache has content
    {
        let cache = detector.cache.read().await;
        assert_eq!(cache.len(), 1);
    }

    // Clear cache
    detector.clear_cache().await;

    // Verify cache is empty
    {
        let cache = detector.cache.read().await;
        assert_eq!(cache.len(), 0);
    }
}
