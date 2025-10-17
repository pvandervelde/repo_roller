//! Tests for RepositoryCreationRequest

use super::*;
use std::collections::HashMap;

// ============================================================================
// RepositoryCreationRequest Tests
// ============================================================================

/// Verify that RepositoryCreationRequest can be created with all required fields.
#[test]
fn test_repository_creation_request_creation() {
    let name = RepositoryName::new("my-repo").unwrap();
    let owner = OrganizationName::new("my-org").unwrap();
    let template = TemplateName::new("rust-library").unwrap();
    let variables = HashMap::new();

    let request = RepositoryCreationRequest {
        name: name.clone(),
        owner: owner.clone(),
        template: template.clone(),
        variables: variables.clone(),
    };

    assert_eq!(request.name, name);
    assert_eq!(request.owner, owner);
    assert_eq!(request.template, template);
    assert_eq!(request.variables, variables);
}

/// Verify that RepositoryCreationRequest works with template variables.
#[test]
fn test_repository_creation_request_with_variables() {
    let mut variables = HashMap::new();
    variables.insert("project_name".to_string(), "MyProject".to_string());
    variables.insert("author".to_string(), "John Doe".to_string());

    let request = RepositoryCreationRequest {
        name: RepositoryName::new("my-repo").unwrap(),
        owner: OrganizationName::new("my-org").unwrap(),
        template: TemplateName::new("rust-library").unwrap(),
        variables: variables.clone(),
    };

    assert_eq!(request.variables.len(), 2);
    assert_eq!(request.variables.get("project_name").unwrap(), "MyProject");
    assert_eq!(request.variables.get("author").unwrap(), "John Doe");
}

/// Verify that RepositoryCreationRequest is cloneable.
#[test]
fn test_repository_creation_request_clone() {
    let request = RepositoryCreationRequest {
        name: RepositoryName::new("my-repo").unwrap(),
        owner: OrganizationName::new("my-org").unwrap(),
        template: TemplateName::new("rust-library").unwrap(),
        variables: HashMap::new(),
    };

    let cloned = request.clone();

    assert_eq!(request.name, cloned.name);
    assert_eq!(request.owner, cloned.owner);
    assert_eq!(request.template, cloned.template);
    assert_eq!(request.variables, cloned.variables);
}

/// Verify that RepositoryCreationRequest has Debug output.
#[test]
fn test_repository_creation_request_debug() {
    let request = RepositoryCreationRequest {
        name: RepositoryName::new("my-repo").unwrap(),
        owner: OrganizationName::new("my-org").unwrap(),
        template: TemplateName::new("rust-library").unwrap(),
        variables: HashMap::new(),
    };

    let debug_output = format!("{:?}", request);
    assert!(debug_output.contains("RepositoryCreationRequest"));
    assert!(debug_output.contains("my-repo"));
    assert!(debug_output.contains("my-org"));
    assert!(debug_output.contains("rust-library"));
}

/// Verify that RepositoryCreationRequest uses branded types for type safety.
#[test]
fn test_repository_creation_request_type_safety() {
    // This test verifies compile-time type safety by attempting to use the types
    let name = RepositoryName::new("my-repo").unwrap();
    let owner = OrganizationName::new("my-org").unwrap();
    let template = TemplateName::new("rust-library").unwrap();

    // These types should not be interchangeable
    let request = RepositoryCreationRequest {
        name,     // RepositoryName
        owner,    // OrganizationName
        template, // TemplateName
        variables: HashMap::new(),
    };

    // Verify we can access the values
    assert_eq!(request.name.as_str(), "my-repo");
    assert_eq!(request.owner.as_str(), "my-org");
    assert_eq!(request.template.as_str(), "rust-library");
}

/// Verify that RepositoryCreationRequest can handle empty variables.
#[test]
fn test_repository_creation_request_empty_variables() {
    let request = RepositoryCreationRequest {
        name: RepositoryName::new("my-repo").unwrap(),
        owner: OrganizationName::new("my-org").unwrap(),
        template: TemplateName::new("rust-library").unwrap(),
        variables: HashMap::new(),
    };

    assert!(request.variables.is_empty());
}

/// Verify that RepositoryCreationRequest validates names through branded types.
#[test]
fn test_repository_creation_request_validates_names() {
    // Invalid repository name (starts with dash)
    assert!(RepositoryName::new("-invalid").is_err());

    // Invalid organization name (ends with dash)
    assert!(OrganizationName::new("invalid-").is_err());

    // Invalid template name (uppercase not allowed)
    assert!(TemplateName::new("Invalid-Template").is_err());

    // Valid request can be created with valid names
    let request = RepositoryCreationRequest {
        name: RepositoryName::new("valid-repo").unwrap(),
        owner: OrganizationName::new("valid-org").unwrap(),
        template: TemplateName::new("valid-template").unwrap(),
        variables: HashMap::new(),
    };

    assert_eq!(request.name.as_str(), "valid-repo");
}
