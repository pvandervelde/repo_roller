//! Tests for label_manager module.

use super::*;
use config_manager::settings::LabelConfig;
use std::collections::HashMap;

// Note: Full integration tests with mock GitHubClient will be added in Phase 2
// For now, we test the result types and helper methods that are fully implemented

#[test]
fn test_apply_labels_result_new() {
    let result = ApplyLabelsResult::new();
    assert_eq!(result.created, 0);
    assert_eq!(result.updated, 0);
    assert_eq!(result.skipped, 0);
    assert_eq!(result.failed, 0);
    assert!(result.failed_labels.is_empty());
}

#[test]
fn test_apply_labels_result_default() {
    let result = ApplyLabelsResult::default();
    assert_eq!(result.created, 0);
    assert_eq!(result.updated, 0);
    assert_eq!(result.skipped, 0);
    assert_eq!(result.failed, 0);
    assert!(result.failed_labels.is_empty());
}

#[test]
fn test_apply_labels_result_is_success_when_no_failures() {
    let mut result = ApplyLabelsResult::new();
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
fn test_apply_labels_result_is_not_success_when_failures() {
    let mut result = ApplyLabelsResult::new();
    result.created = 2;
    result.failed = 1;
    result.failed_labels.push("failing-label".to_string());

    assert!(
        !result.is_success(),
        "Result with failures should not be success"
    );
}

#[test]
fn test_apply_labels_result_has_changes_when_created() {
    let mut result = ApplyLabelsResult::new();
    result.created = 1;

    assert!(
        result.has_changes(),
        "Result with created labels should have changes"
    );
}

#[test]
fn test_apply_labels_result_has_changes_when_updated() {
    let mut result = ApplyLabelsResult::new();
    result.updated = 1;

    assert!(
        result.has_changes(),
        "Result with updated labels should have changes"
    );
}

#[test]
fn test_apply_labels_result_has_no_changes_when_only_skipped() {
    let mut result = ApplyLabelsResult::new();
    result.skipped = 5;

    assert!(
        !result.has_changes(),
        "Result with only skipped should have no changes"
    );
}

#[test]
fn test_apply_labels_result_has_no_changes_when_empty() {
    let result = ApplyLabelsResult::new();

    assert!(!result.has_changes(), "Empty result should have no changes");
}

#[test]
fn test_apply_labels_result_tracks_failed_labels() {
    let mut result = ApplyLabelsResult::new();
    result.failed = 2;
    result.failed_labels.push("bug".to_string());
    result.failed_labels.push("feature".to_string());

    assert_eq!(result.failed, 2);
    assert_eq!(result.failed_labels.len(), 2);
    assert!(result.failed_labels.contains(&"bug".to_string()));
    assert!(result.failed_labels.contains(&"feature".to_string()));
}

#[test]
fn test_apply_labels_result_counts() {
    let mut result = ApplyLabelsResult::new();
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

// Manager creation test - will be expanded in Phase 2 with actual behavior tests
#[test]
fn test_label_manager_can_be_created() {
    // This is a compilation test - verifies the type system is correct
    // In Phase 2, we'll add tests that actually call apply_labels with mocks
    // For now, verify that the LabelManager type exists and has the right shape

    // Can't easily create a real GitHubClient here without authentication
    // So we just verify the types compile correctly
    fn accepts_label_manager(_manager: LabelManager) {}

    // This function compiles, proving LabelManager is properly defined
}
