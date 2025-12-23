//! Tests for test cleanup utilities.

use super::*;

#[test]
fn test_is_test_repository_integration_prefix() {
    assert!(RepositoryCleanup::is_test_repository(
        "test-repo-roller-pr123-auth-test"
    ));
    assert!(RepositoryCleanup::is_test_repository(
        "test-repo-roller-main-config"
    ));
    assert!(RepositoryCleanup::is_test_repository(
        "test-repo-roller-local-api"
    ));
}

#[test]
fn test_is_test_repository_e2e_prefix() {
    assert!(RepositoryCleanup::is_test_repository(
        "e2e-repo-roller-pr456-workflow"
    ));
    assert!(RepositoryCleanup::is_test_repository(
        "e2e-repo-roller-main-endpoints"
    ));
    assert!(RepositoryCleanup::is_test_repository(
        "e2e-repo-roller-feature-testing"
    ));
}

#[test]
fn test_is_test_repository_non_test_repos() {
    assert!(!RepositoryCleanup::is_test_repository("regular-repo"));
    assert!(!RepositoryCleanup::is_test_repository("my-project"));
    assert!(!RepositoryCleanup::is_test_repository("test-repo")); // Not full pattern
    assert!(!RepositoryCleanup::is_test_repository("e2e-test-suite")); // Not full pattern
    assert!(!RepositoryCleanup::is_test_repository("repo-roller-test")); // Wrong order
}

#[test]
fn test_is_test_repository_edge_cases() {
    // Empty string
    assert!(!RepositoryCleanup::is_test_repository(""));

    // Just the prefix
    assert!(RepositoryCleanup::is_test_repository("test-repo-roller-"));
    assert!(RepositoryCleanup::is_test_repository("e2e-repo-roller-"));

    // Case sensitive
    assert!(!RepositoryCleanup::is_test_repository(
        "Test-repo-roller-main"
    ));
    assert!(!RepositoryCleanup::is_test_repository(
        "E2E-repo-roller-main"
    ));
}
