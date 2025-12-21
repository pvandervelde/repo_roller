//! Tests for test utilities.

use super::*;
use serial_test::serial;

#[test]
fn test_generate_test_repo_name() {
    let name = generate_test_repo_name("test", "basic");
    assert!(name.starts_with("test-repo-roller-"));
    assert!(name.contains("-basic-"));
    // Should include context, timestamp, test name, and random suffix
    assert!(name.len() > 35);
}

#[test]
fn test_generate_e2e_repo_name() {
    let name = generate_test_repo_name("e2e", "api");
    assert!(name.starts_with("e2e-repo-roller-"));
    assert!(name.contains("-api-"));
    assert!(name.len() > 35);
}

#[test]
#[serial]
fn test_get_workflow_context_pr() {
    unsafe {
        std::env::set_var("GITHUB_REF", "refs/pull/456/merge");
    }
    let context = get_workflow_context();
    assert_eq!(context, "pr456");
    unsafe {
        std::env::remove_var("GITHUB_REF");
    }
}

#[test]
#[serial]
fn test_get_workflow_context_main_branch() {
    unsafe {
        std::env::set_var("GITHUB_REF", "refs/heads/main");
    }
    let context = get_workflow_context();
    assert_eq!(context, "main");
    unsafe {
        std::env::remove_var("GITHUB_REF");
    }
}

#[test]
#[serial]
fn test_get_workflow_context_master_branch() {
    unsafe {
        std::env::set_var("GITHUB_REF", "refs/heads/master");
    }
    let context = get_workflow_context();
    assert_eq!(context, "main");
    unsafe {
        std::env::remove_var("GITHUB_REF");
    }
}

#[test]
#[serial]
fn test_get_workflow_context_feature_branch() {
    unsafe {
        std::env::set_var("GITHUB_REF", "refs/heads/feature/new-feature");
    }
    let context = get_workflow_context();
    assert_eq!(context, "feature-new-feature");
    unsafe {
        std::env::remove_var("GITHUB_REF");
    }
}

#[test]
#[serial]
fn test_get_workflow_context_local() {
    unsafe {
        std::env::remove_var("GITHUB_REF");
    }
    let context = get_workflow_context();
    assert_eq!(context, "local");
}
