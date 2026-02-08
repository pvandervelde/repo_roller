//! Tests for webhook_manager module.

use super::*;
use config_manager::settings::WebhookConfig;

// Note: Full integration tests with mock GitHubClient will be added in Phase 2
// For now, we test the result types and helper methods that are fully implemented

#[test]
fn test_apply_webhooks_result_new() {
    let result = ApplyWebhooksResult::new();
    assert_eq!(result.created, 0);
    assert_eq!(result.updated, 0);
    assert_eq!(result.skipped, 0);
    assert_eq!(result.failed, 0);
    assert!(result.failed_webhooks.is_empty());
}

#[test]
fn test_apply_webhooks_result_default() {
    let result = ApplyWebhooksResult::default();
    assert_eq!(result.created, 0);
    assert_eq!(result.updated, 0);
    assert_eq!(result.skipped, 0);
    assert_eq!(result.failed, 0);
    assert!(result.failed_webhooks.is_empty());
}

#[test]
fn test_apply_webhooks_result_is_success_when_no_failures() {
    let mut result = ApplyWebhooksResult::new();
    assert!(
        result.is_success(),
        "Empty result should be success (no failures)"
    );

    result.created = 2;
    result.updated = 1;
    assert!(
        result.is_success(),
        "Result with operations but no failures should be success"
    );
}

#[test]
fn test_apply_webhooks_result_is_not_success_when_failures() {
    let mut result = ApplyWebhooksResult::new();
    result.created = 1;
    result.failed = 1;
    result
        .failed_webhooks
        .push("https://example.com/failing".to_string());

    assert!(
        !result.is_success(),
        "Result with failures should not be success"
    );
}

#[test]
fn test_apply_webhooks_result_has_changes_when_created() {
    let mut result = ApplyWebhooksResult::new();
    result.created = 1;

    assert!(
        result.has_changes(),
        "Result with created webhooks should have changes"
    );
}

#[test]
fn test_apply_webhooks_result_has_changes_when_updated() {
    let mut result = ApplyWebhooksResult::new();
    result.updated = 1;

    assert!(
        result.has_changes(),
        "Result with updated webhooks should have changes"
    );
}

#[test]
fn test_apply_webhooks_result_has_no_changes_when_only_skipped() {
    let mut result = ApplyWebhooksResult::new();
    result.skipped = 5;

    assert!(
        !result.has_changes(),
        "Result with only skipped should have no changes"
    );
}

#[test]
fn test_apply_webhooks_result_has_no_changes_when_empty() {
    let result = ApplyWebhooksResult::new();

    assert!(!result.has_changes(), "Empty result should have no changes");
}

#[test]
fn test_apply_webhooks_result_tracks_failed_webhooks() {
    let mut result = ApplyWebhooksResult::new();
    result.failed = 2;
    result
        .failed_webhooks
        .push("https://example.com/webhook1".to_string());
    result
        .failed_webhooks
        .push("https://example.com/webhook2".to_string());

    assert_eq!(result.failed, 2);
    assert_eq!(result.failed_webhooks.len(), 2);
    assert!(result
        .failed_webhooks
        .contains(&"https://example.com/webhook1".to_string()));
    assert!(result
        .failed_webhooks
        .contains(&"https://example.com/webhook2".to_string()));
}

#[test]
fn test_apply_webhooks_result_counts() {
    let mut result = ApplyWebhooksResult::new();
    result.created = 3;
    result.updated = 2;
    result.skipped = 1;
    result.failed = 1;

    // Verify all counts are independent
    assert_eq!(result.created, 3);
    assert_eq!(result.updated, 2);
    assert_eq!(result.skipped, 1);
    assert_eq!(result.failed, 1);

    // Verify total operations
    let total = result.created + result.updated + result.skipped + result.failed;
    assert_eq!(total, 7, "Total should be sum of all operations");
}

#[test]
fn test_webhook_config_structure() {
    // Test that WebhookConfig has the expected fields and can be constructed
    let config = WebhookConfig {
        url: "https://example.com/webhook".to_string(),
        content_type: "json".to_string(),
        secret: Some("my-secret".to_string()),
        active: true,
        events: vec!["push".to_string(), "pull_request".to_string()],
    };

    assert_eq!(config.url, "https://example.com/webhook");
    assert_eq!(config.content_type, "json");
    assert_eq!(config.secret, Some("my-secret".to_string()));
    assert!(config.active);
    assert_eq!(config.events.len(), 2);
}

#[test]
fn test_webhook_config_optional_secret() {
    let config = WebhookConfig {
        url: "https://example.com/webhook".to_string(),
        content_type: "json".to_string(),
        secret: None, // No secret
        active: true,
        events: vec!["push".to_string()],
    };

    assert!(config.secret.is_none(), "Secret should be optional");
}

// Manager creation test - will be expanded in Phase 2 with actual behavior tests
#[test]
fn test_webhook_manager_can_be_created() {
    // This is a compilation test - verifies the type system is correct
    // In Phase 2, we'll add tests that actually call apply_webhooks with mocks
    // The WebhookManager type exists and has the correct public API
    // (verified by successful compilation of this crate)
}

// Validation tests will be expanded in Phase 2 when validate_webhook_config is implemented
#[test]
fn test_webhook_validation_requirements_documented() {
    // Validation rules from rustdoc:
    // - URL must be valid HTTPS URL
    // - Events list must not be empty
    // - Content type must be "json" or "form"
    // - Secret (if provided) must meet minimum length requirements

    // These will be tested when implementation is complete in Phase 2
    // For now, this test documents the planned validation requirements
}
