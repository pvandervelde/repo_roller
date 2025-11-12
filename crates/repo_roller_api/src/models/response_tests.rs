//! Tests for response models

use super::*;

#[test]
fn test_create_repository_response_serialization() {
    let response = CreateRepositoryResponse {
        repository: RepositoryInfo {
            name: "my-repo".to_string(),
            full_name: "myorg/my-repo".to_string(),
            url: "https://github.com/myorg/my-repo".to_string(),
            visibility: "private".to_string(),
            description: None,
        },
        applied_configuration: serde_json::json!({}),
        created_at: "2025-11-12T10:30:00Z".to_string(),
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"repository\""));
    assert!(json.contains("\"createdAt\""));
}

#[test]
fn test_validation_response_serialization() {
    let response = ValidateRepositoryNameResponse {
        valid: false,
        available: true,
        messages: Some(vec!["Name must be lowercase".to_string()]),
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"valid\":false"));
    assert!(json.contains("\"available\":true"));
}

#[test]
fn test_list_templates_response_serialization() {
    let response = ListTemplatesResponse {
        templates: vec![TemplateSummary {
            name: "rust-library".to_string(),
            description: "Rust library template".to_string(),
            category: Some("rust".to_string()),
            variables: vec!["project_name".to_string()],
        }],
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"rust-library\""));
}
