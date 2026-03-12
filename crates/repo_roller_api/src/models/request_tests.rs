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
    assert_eq!(req.template, Some("rust-library".to_string()));
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
    assert_eq!(req.template, Some("rust-library".to_string()));
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
    assert_eq!(req.template, Some("rust-library".to_string()));
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

/// Test CreateRepositoryRequest with optional template (no template provided)
#[test]
fn test_create_repository_request_without_template() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo"
    }"#;

    let req: CreateRepositoryRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.organization, "myorg");
    assert_eq!(req.name, "my-repo");
    assert_eq!(req.template, None);
    assert!(req.variables.is_empty());
}

/// Test CreateRepositoryRequest with contentStrategy empty
#[test]
fn test_create_repository_request_with_empty_strategy() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo",
        "template": "rust-library",
        "contentStrategy": {
            "type": "empty"
        }
    }"#;

    let req: CreateRepositoryRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.organization, "myorg");
    assert_eq!(req.name, "my-repo");
    assert_eq!(req.template, Some("rust-library".to_string()));

    use repo_roller_core::ContentStrategy;
    assert_eq!(req.content_strategy, ContentStrategy::Empty);
}

/// Test CreateRepositoryRequest with contentStrategy custom_init
#[test]
fn test_create_repository_request_with_custom_init_strategy() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo",
        "contentStrategy": {
            "type": "custom_init",
            "include_readme": true,
            "include_gitignore": false
        }
    }"#;

    let req: CreateRepositoryRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.organization, "myorg");
    assert_eq!(req.name, "my-repo");
    assert_eq!(req.template, None);

    use repo_roller_core::ContentStrategy;
    assert_eq!(
        req.content_strategy,
        ContentStrategy::CustomInit {
            include_readme: true,
            include_gitignore: false,
        }
    );
}

/// Test CreateRepositoryRequest with template strategy (explicit)
#[test]
fn test_create_repository_request_with_template_strategy_explicit() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo",
        "template": "rust-library",
        "contentStrategy": {
            "type": "template"
        }
    }"#;

    let req: CreateRepositoryRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.template, Some("rust-library".to_string()));

    use repo_roller_core::ContentStrategy;
    assert_eq!(req.content_strategy, ContentStrategy::Template);
}

/// Test CreateRepositoryRequest defaults to Template strategy when not specified
#[test]
fn test_create_repository_request_defaults_to_template_strategy() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo",
        "template": "rust-library"
    }"#;

    let req: CreateRepositoryRequest = serde_json::from_str(json).unwrap();

    use repo_roller_core::ContentStrategy;
    assert_eq!(req.content_strategy, ContentStrategy::Template);
}

/// Test CreateRepositoryRequest with empty strategy and no template
#[test]
fn test_create_repository_request_empty_strategy_without_template() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo",
        "contentStrategy": {
            "type": "empty"
        }
    }"#;

    let req: CreateRepositoryRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.template, None);

    use repo_roller_core::ContentStrategy;
    assert_eq!(req.content_strategy, ContentStrategy::Empty);
}

/// Test CreateRepositoryRequest with custom_init both flags true
#[test]
fn test_create_repository_request_custom_init_both_files() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo",
        "template": "github-actions",
        "contentStrategy": {
            "type": "custom_init",
            "include_readme": true,
            "include_gitignore": true
        }
    }"#;

    let req: CreateRepositoryRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.template, Some("github-actions".to_string()));

    use repo_roller_core::ContentStrategy;
    assert_eq!(
        req.content_strategy,
        ContentStrategy::CustomInit {
            include_readme: true,
            include_gitignore: true,
        }
    );
}

// ============================================================================
// Teams and Collaborators JSON Deserialization Tests
// ============================================================================

/// Test that teams and collaborators fields are absent by default (empty maps).
#[test]
fn test_create_repository_request_teams_and_collaborators_default_empty() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo"
    }"#;

    let req: CreateRepositoryRequest = serde_json::from_str(json).unwrap();
    assert!(req.teams.is_empty());
    assert!(req.collaborators.is_empty());
}

/// Test that teams field is correctly deserialized from JSON.
#[test]
fn test_create_repository_request_deserializes_teams() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo",
        "teams": {
            "platform": "write",
            "security": "admin"
        }
    }"#;

    let req: CreateRepositoryRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.teams.len(), 2);
    assert_eq!(req.teams.get("platform"), Some(&"write".to_string()));
    assert_eq!(req.teams.get("security"), Some(&"admin".to_string()));
}

/// Test that collaborators field is correctly deserialized from JSON.
#[test]
fn test_create_repository_request_deserializes_collaborators() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo",
        "collaborators": {
            "alice": "write",
            "bob": "read"
        }
    }"#;

    let req: CreateRepositoryRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.collaborators.len(), 2);
    assert_eq!(req.collaborators.get("alice"), Some(&"write".to_string()));
    assert_eq!(req.collaborators.get("bob"), Some(&"read".to_string()));
}

/// Test that both teams and collaborators can be specified together.
#[test]
fn test_create_repository_request_deserializes_teams_and_collaborators_together() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo",
        "teams": {
            "devs": "write"
        },
        "collaborators": {
            "carol": "maintain"
        }
    }"#;

    let req: CreateRepositoryRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.teams.len(), 1);
    assert_eq!(req.collaborators.len(), 1);
    assert_eq!(req.teams.get("devs"), Some(&"write".to_string()));
    assert_eq!(
        req.collaborators.get("carol"),
        Some(&"maintain".to_string())
    );
}

/// Test that empty teams object deserializes to an empty map.
#[test]
fn test_create_repository_request_empty_teams_object() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo",
        "teams": {}
    }"#;

    let req: CreateRepositoryRequest = serde_json::from_str(json).unwrap();
    assert!(req.teams.is_empty());
}

/// Test that permission strings are preserved as raw strings during deserialization.
///
/// The HTTP model stores permission values as plain strings; validation and
/// conversion to AccessLevel happens in the translation layer.
#[test]
fn test_create_repository_request_teams_stores_raw_permission_strings() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo",
        "teams": {
            "t1": "none",
            "t2": "read",
            "t3": "triage",
            "t4": "write",
            "t5": "maintain",
            "t6": "admin"
        }
    }"#;

    let req: CreateRepositoryRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.teams.len(), 6);
    assert_eq!(req.teams.get("t1"), Some(&"none".to_string()));
    assert_eq!(req.teams.get("t2"), Some(&"read".to_string()));
    assert_eq!(req.teams.get("t3"), Some(&"triage".to_string()));
    assert_eq!(req.teams.get("t4"), Some(&"write".to_string()));
    assert_eq!(req.teams.get("t5"), Some(&"maintain".to_string()));
    assert_eq!(req.teams.get("t6"), Some(&"admin".to_string()));
}
