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
