//! Tests for handlers module

use super::*;

/// Test that health check handler returns proper JSON response
#[tokio::test]
async fn test_health_check_returns_json() {
    let response = health_check().await;

    // Verify structure exists
    assert_eq!(response.0.status, "healthy");
    assert!(response.0.version.is_some());
    assert!(!response.0.timestamp.is_empty());
    assert!(response.0.error.is_none());
}

/// Test that health check includes version from Cargo.toml
#[tokio::test]
async fn test_health_check_includes_version() {
    let response = health_check().await;

    assert_eq!(response.0.version, Some(env!("CARGO_PKG_VERSION").to_string()));
}

/// Test that health check timestamp is valid ISO 8601
#[tokio::test]
async fn test_health_check_timestamp_format() {
    let response = health_check().await;

    // Should be parseable as ISO 8601
    let parsed = chrono::DateTime::parse_from_rfc3339(&response.0.timestamp);
    assert!(parsed.is_ok(), "Timestamp should be valid ISO 8601 format");
}
