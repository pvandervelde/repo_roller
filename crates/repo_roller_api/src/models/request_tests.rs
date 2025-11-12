//! Tests for request models

use super::*;

#[test]
fn test_create_repository_request_deserialization() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo",
        "template": "rust-library",
        "variables": {}
    }"#;

    let req: CreateRepositoryRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.organization, "myorg");
    assert_eq!(req.name, "my-repo");
    assert_eq!(req.template, "rust-library");
}

#[test]
fn test_create_repository_request_unknown_field() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo",
        "template": "rust-library",
        "unknown_field": "value"
    }"#;

    let result = serde_json::from_str::<CreateRepositoryRequest>(json);
    assert!(result.is_err());
}

#[test]
fn test_validate_name_request_deserialization() {
    let json = r#"{
        "organization": "myorg",
        "name": "test-repo"
    }"#;

    let req: ValidateRepositoryNameRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.organization, "myorg");
    assert_eq!(req.name, "test-repo");
}

#[test]
fn test_preview_configuration_request_deserialization() {
    let json = r#"{
        "template": "rust-library",
        "team": "platform"
    }"#;

    let req: PreviewConfigurationRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.template, "rust-library");
    assert_eq!(req.team, Some("platform".to_string()));
}
