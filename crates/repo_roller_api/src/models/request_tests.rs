//! Tests for HTTP request types

use super::*;

/// Test CreateRepositoryRequest deserialization with all fields
#[test]
fn test_create_repository_request_full() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo",
        "template": "rust-library",
        "visibility": "private",
        "team": "platform",
        "repositoryType": "library",
        "variables": {
            "project_name": "MyLib"
        }
    }"#;

    let req: CreateRepositoryRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.organization, "myorg");
    assert_eq!(req.name, "my-repo");
    assert_eq!(req.template, "rust-library");
    assert_eq!(req.visibility, Some("private".to_string()));
    assert_eq!(req.team, Some("platform".to_string()));
    assert_eq!(req.repository_type, Some("library".to_string()));
    assert_eq!(
        req.variables.get("project_name"),
        Some(&"MyLib".to_string())
    );
}

/// Test CreateRepositoryRequest deserialization with minimal fields
#[test]
fn test_create_repository_request_minimal() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo",
        "template": "rust-library"
    }"#;

    let req: CreateRepositoryRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.organization, "myorg");
    assert_eq!(req.name, "my-repo");
    assert_eq!(req.template, "rust-library");
    assert_eq!(req.visibility, None);
    assert_eq!(req.team, None);
    assert_eq!(req.repository_type, None);
    assert!(req.variables.is_empty());
}

/// Test that unknown fields are rejected (deny_unknown_fields)
#[test]
fn test_create_repository_request_rejects_unknown_fields() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo",
        "template": "rust-library",
        "unknownField": "value"
    }"#;

    let result = serde_json::from_str::<CreateRepositoryRequest>(json);
    assert!(result.is_err(), "Should reject unknown fields");
}

/// Test ValidateRepositoryNameRequest deserialization
#[test]
fn test_validate_repository_name_request() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo"
    }"#;

    let req: ValidateRepositoryNameRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.organization, "myorg");
    assert_eq!(req.name, "my-repo");
}

/// Test PreviewConfigurationRequest deserialization
#[test]
fn test_preview_configuration_request() {
    let json = r#"{
        "template": "rust-library",
        "team": "platform",
        "repositoryType": "library"
    }"#;

    let req: PreviewConfigurationRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.template, "rust-library");
    assert_eq!(req.team, Some("platform".to_string()));
    assert_eq!(req.repository_type, Some("library".to_string()));
}

/// Test PreviewConfigurationRequest with minimal fields
#[test]
fn test_preview_configuration_request_minimal() {
    let json = r#"{
        "template": "rust-library"
    }"#;

    let req: PreviewConfigurationRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.template, "rust-library");
    assert_eq!(req.team, None);
    assert_eq!(req.repository_type, None);
}

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
