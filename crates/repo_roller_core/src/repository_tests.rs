//! Tests for RepositoryName and OrganizationName

use super::*;

#[test]
fn test_repository_name_valid() {
    assert!(RepositoryName::new("my-repo").is_ok());
    assert!(RepositoryName::new("my_repo").is_ok());
    assert!(RepositoryName::new("my.repo").is_ok());
    assert!(RepositoryName::new("MyRepo123").is_ok());
}

#[test]
fn test_repository_name_invalid() {
    assert!(RepositoryName::new(".starts-with-dot").is_err());
    assert!(RepositoryName::new("-starts-with-dash").is_err());
    assert!(RepositoryName::new("").is_err());
    assert!(RepositoryName::new("a".repeat(101)).is_err());
    assert!(RepositoryName::new("invalid space").is_err());
}

#[test]
fn test_organization_name_valid() {
    assert!(OrganizationName::new("my-org").is_ok());
    assert!(OrganizationName::new("MyOrg").is_ok());
    assert!(OrganizationName::new("org123").is_ok());
}

#[test]
fn test_organization_name_invalid() {
    assert!(OrganizationName::new("-starts-with-dash").is_err());
    assert!(OrganizationName::new("ends-with-dash-").is_err());
    assert!(OrganizationName::new("double--dash").is_err());
    assert!(OrganizationName::new("").is_err());
    assert!(OrganizationName::new("a".repeat(40)).is_err());
    assert!(OrganizationName::new("invalid_underscore").is_err());
}
