//! Tests for utility functions.

use super::*;

#[test]
fn test_generate_test_repo_name_integration_wrapper() {
    // Test that generate_test_repo_name works (uses test_utils under the hood)
    let name = generate_test_repo_name("test", "basic");
    assert!(name.starts_with("test-repo-roller-"));
    assert!(name.contains("-basic-"));
    assert!(name.len() > 35);
}

#[test]
fn test_generate_integration_test_repo_name() {
    // Test the convenience wrapper
    let name = generate_integration_test_repo_name("auth");
    assert!(name.starts_with("test-repo-roller-"));
    assert!(name.contains("-auth-"));
}

#[test]
fn test_test_repository_creation() {
    let repo = TestRepository::new("test-repo".to_string(), "test-org".to_string());
    assert_eq!(repo.name, "test-repo");
    assert_eq!(repo.owner, "test-org");
    assert_eq!(repo.full_name, "test-org/test-repo");
}

#[test]
fn test_is_test_repository_integration_prefix() {
    assert!(is_test_repository("test-repo-roller-pr123-auth-test"));
    assert!(is_test_repository("test-repo-roller-main-config"));
    assert!(is_test_repository("test-repo-roller-local-api"));
}

#[test]
fn test_is_test_repository_e2e_prefix() {
    assert!(is_test_repository("e2e-repo-roller-pr456-workflow"));
    assert!(is_test_repository("e2e-repo-roller-main-endpoints"));
    assert!(is_test_repository("e2e-repo-roller-feature-testing"));
}

#[test]
fn test_is_test_repository_non_test_repos() {
    assert!(!is_test_repository("regular-repo"));
    assert!(!is_test_repository("my-project"));
    assert!(!is_test_repository("test-repo")); // Not full pattern
    assert!(!is_test_repository("e2e-test-suite")); // Not full pattern
    assert!(!is_test_repository("repo-roller-test")); // Wrong order
}

#[test]
fn test_is_test_repository_edge_cases() {
    // Empty string
    assert!(!is_test_repository(""));

    // Just the prefix
    assert!(is_test_repository("test-repo-roller-"));
    assert!(is_test_repository("e2e-repo-roller-"));

    // Case sensitive
    assert!(!is_test_repository("Test-repo-roller-main"));
    assert!(!is_test_repository("E2E-repo-roller-main"));
}
