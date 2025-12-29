use super::*;
use serde_json::from_str;

#[test]
fn test_organization_deserialization() {
    let json_str = r#"{
        "name": "test-org"
    }"#;

    let org: Organization = from_str(json_str).expect("Failed to deserialize Organization");

    assert_eq!(org.name, "test-org");
}

#[test]
fn test_repository_creation() {
    let repo = Repository::new(
        "my-repo".to_string(),
        "owner/my-repo".to_string(),
        "MDEwOlJlcG9zaXRvcnkxMjM0NTY3ODk=".to_string(),
        false,
    );

    assert_eq!(repo.name(), "my-repo");
    assert_eq!(repo.node_id(), "MDEwOlJlcG9zaXRvcnkxMjM0NTY3ODk=");
    assert!(!repo.is_private());
}

#[test]
fn test_repository_deserialization() {
    let json_str = r#"{
        "name": "example-repo",
        "full_name": "user/example-repo",
        "node_id": "MDEwOlJlcG9zaXRvcnkxMTExMTExMQ==",
        "private": false
    }"#;

    let repo: Repository = from_str(json_str).expect("Failed to deserialize Repository");

    assert_eq!(repo.name(), "example-repo");
    assert_eq!(repo.node_id(), "MDEwOlJlcG9zaXRvcnkxMTExMTExMQ==");
    assert!(!repo.is_private());
}

#[test]
fn test_repository_deserialization_with_features() {
    let json_str = r#"{
        "name": "feature-repo",
        "full_name": "org/feature-repo",
        "node_id": "node123",
        "private": true,
        "has_issues": true,
        "has_wiki": false,
        "has_projects": true,
        "has_discussions": false
    }"#;

    let repo: Repository = from_str(json_str).expect("Failed to deserialize Repository");

    assert_eq!(repo.name(), "feature-repo");
    assert!(repo.is_private());
    assert_eq!(repo.has_issues(), Some(true));
    assert_eq!(repo.has_wiki(), Some(false));
    assert_eq!(repo.has_projects(), Some(true));
    assert_eq!(repo.has_discussions(), Some(false));
}

#[test]
fn test_repository_url() {
    let repo = Repository::new(
        "my-repo".to_string(),
        "owner/my-repo".to_string(),
        "node123".to_string(),
        false,
    );

    let url = repo.url();
    assert_eq!(url.as_str(), "https://github.com/owner/my-repo.git");
}

#[test]
fn test_repository_private_flag() {
    let public_repo = Repository::new(
        "public-repo".to_string(),
        "owner/public-repo".to_string(),
        "node1".to_string(),
        false,
    );

    let private_repo = Repository::new(
        "private-repo".to_string(),
        "owner/private-repo".to_string(),
        "node2".to_string(),
        true,
    );

    assert!(!public_repo.is_private());
    assert!(private_repo.is_private());
}
