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
        visibility: None,
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
        visibility: None,
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
        visibility: None,
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
        visibility: None,
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
        visibility: None,
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
        visibility: None,
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
        visibility: None,
    };

    assert_eq!(request.name.as_str(), "valid-repo");
}

// ============================================================================
// RepositoryCreationRequestBuilder Tests
// ============================================================================

/// Verify that builder can create a basic request with no variables.
#[test]
fn test_builder_basic() {
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("my-repo").unwrap(),
        OrganizationName::new("my-org").unwrap(),
        TemplateName::new("rust-library").unwrap(),
    )
    .build();

    assert_eq!(request.name.as_str(), "my-repo");
    assert_eq!(request.owner.as_str(), "my-org");
    assert_eq!(request.template.as_str(), "rust-library");
    assert!(request.variables.is_empty());
}

/// Verify that builder can add a single variable.
#[test]
fn test_builder_single_variable() {
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("my-repo").unwrap(),
        OrganizationName::new("my-org").unwrap(),
        TemplateName::new("rust-library").unwrap(),
    )
    .variable("project_name", "MyProject")
    .build();

    assert_eq!(request.variables.len(), 1);
    assert_eq!(request.variables.get("project_name").unwrap(), "MyProject");
}

/// Verify that builder can chain multiple variables.
#[test]
fn test_builder_multiple_variables_chained() {
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("my-repo").unwrap(),
        OrganizationName::new("my-org").unwrap(),
        TemplateName::new("rust-library").unwrap(),
    )
    .variable("project_name", "MyProject")
    .variable("author", "John Doe")
    .variable("license", "MIT")
    .build();

    assert_eq!(request.variables.len(), 3);
    assert_eq!(request.variables.get("project_name").unwrap(), "MyProject");
    assert_eq!(request.variables.get("author").unwrap(), "John Doe");
    assert_eq!(request.variables.get("license").unwrap(), "MIT");
}

/// Verify that builder can add variables in bulk.
#[test]
fn test_builder_bulk_variables() {
    let mut vars = HashMap::new();
    vars.insert("project_name".to_string(), "MyProject".to_string());
    vars.insert("author".to_string(), "John Doe".to_string());

    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("my-repo").unwrap(),
        OrganizationName::new("my-org").unwrap(),
        TemplateName::new("rust-library").unwrap(),
    )
    .variables(vars)
    .build();

    assert_eq!(request.variables.len(), 2);
    assert_eq!(request.variables.get("project_name").unwrap(), "MyProject");
    assert_eq!(request.variables.get("author").unwrap(), "John Doe");
}

/// Verify that builder can mix individual and bulk variables.
#[test]
fn test_builder_mixed_variables() {
    let mut bulk_vars = HashMap::new();
    bulk_vars.insert("author".to_string(), "John Doe".to_string());
    bulk_vars.insert("license".to_string(), "MIT".to_string());

    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("my-repo").unwrap(),
        OrganizationName::new("my-org").unwrap(),
        TemplateName::new("rust-library").unwrap(),
    )
    .variable("project_name", "MyProject")
    .variables(bulk_vars)
    .variable("version", "0.1.0")
    .build();

    assert_eq!(request.variables.len(), 4);
    assert_eq!(request.variables.get("project_name").unwrap(), "MyProject");
    assert_eq!(request.variables.get("author").unwrap(), "John Doe");
    assert_eq!(request.variables.get("license").unwrap(), "MIT");
    assert_eq!(request.variables.get("version").unwrap(), "0.1.0");
}

/// Verify that builder overwrites variables with same key.
#[test]
fn test_builder_variable_overwrite() {
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("my-repo").unwrap(),
        OrganizationName::new("my-org").unwrap(),
        TemplateName::new("rust-library").unwrap(),
    )
    .variable("author", "First Author")
    .variable("author", "Second Author")
    .build();

    assert_eq!(request.variables.len(), 1);
    assert_eq!(request.variables.get("author").unwrap(), "Second Author");
}

/// Verify that builder is cloneable for reuse.
#[test]
fn test_builder_clone() {
    let builder = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("my-repo").unwrap(),
        OrganizationName::new("my-org").unwrap(),
        TemplateName::new("rust-library").unwrap(),
    )
    .variable("project_name", "MyProject");

    let builder_clone = builder.clone();

    let request1 = builder.variable("author", "Author 1").build();
    let request2 = builder_clone.variable("author", "Author 2").build();

    // Both should have project_name, but different authors
    assert_eq!(request1.variables.get("project_name").unwrap(), "MyProject");
    assert_eq!(request1.variables.get("author").unwrap(), "Author 1");

    assert_eq!(request2.variables.get("project_name").unwrap(), "MyProject");
    assert_eq!(request2.variables.get("author").unwrap(), "Author 2");
}

/// Verify that builder has Debug output.
#[test]
fn test_builder_debug() {
    let builder = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("my-repo").unwrap(),
        OrganizationName::new("my-org").unwrap(),
        TemplateName::new("rust-library").unwrap(),
    );

    let debug_output = format!("{:?}", builder);
    assert!(debug_output.contains("RepositoryCreationRequestBuilder"));
}

/// Verify that builder accepts Into<String> for ergonomics.
#[test]
fn test_builder_into_string() {
    let key = String::from("project_name");
    let value = String::from("MyProject");

    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("my-repo").unwrap(),
        OrganizationName::new("my-org").unwrap(),
        TemplateName::new("rust-library").unwrap(),
    )
    .variable(&key, &value) // &str
    .variable(key.clone(), value.clone()) // String
    .build();

    // Should have one entry (second overwrites first)
    assert_eq!(request.variables.len(), 1);
}

// ============================================================================
// RepositoryCreationResult Tests
// ============================================================================

/// Verify that RepositoryCreationResult can be created with all required fields.
#[test]
fn test_repository_creation_result_creation() {
    let result = RepositoryCreationResult {
        repository_url: "https://github.com/my-org/my-repo".to_string(),
        repository_id: "R_kgDOABCDEF".to_string(),
        created_at: Timestamp::now(),
        default_branch: "main".to_string(),
    };

    assert_eq!(result.repository_url, "https://github.com/my-org/my-repo");
    assert_eq!(result.repository_id, "R_kgDOABCDEF");
    assert_eq!(result.default_branch, "main");
}

/// Verify that RepositoryCreationResult stores the creation timestamp.
#[test]
fn test_repository_creation_result_timestamp() {
    let timestamp = Timestamp::now();
    let result = RepositoryCreationResult {
        repository_url: "https://github.com/my-org/my-repo".to_string(),
        repository_id: "R_kgDOABCDEF".to_string(),
        created_at: timestamp,
        default_branch: "main".to_string(),
    };

    assert_eq!(result.created_at, timestamp);
}

/// Verify that RepositoryCreationResult is cloneable.
#[test]
fn test_repository_creation_result_clone() {
    let result = RepositoryCreationResult {
        repository_url: "https://github.com/my-org/my-repo".to_string(),
        repository_id: "R_kgDOABCDEF".to_string(),
        created_at: Timestamp::now(),
        default_branch: "main".to_string(),
    };

    let cloned = result.clone();

    assert_eq!(result.repository_url, cloned.repository_url);
    assert_eq!(result.repository_id, cloned.repository_id);
    assert_eq!(result.created_at, cloned.created_at);
    assert_eq!(result.default_branch, cloned.default_branch);
}

/// Verify that RepositoryCreationResult has Debug output.
#[test]
fn test_repository_creation_result_debug() {
    let result = RepositoryCreationResult {
        repository_url: "https://github.com/my-org/my-repo".to_string(),
        repository_id: "R_kgDOABCDEF".to_string(),
        created_at: Timestamp::now(),
        default_branch: "main".to_string(),
    };

    let debug_output = format!("{:?}", result);
    assert!(debug_output.contains("RepositoryCreationResult"));
    assert!(debug_output.contains("https://github.com/my-org/my-repo"));
    assert!(debug_output.contains("R_kgDOABCDEF"));
    assert!(debug_output.contains("main"));
}

/// Verify that RepositoryCreationResult handles various branch names.
#[test]
fn test_repository_creation_result_different_branches() {
    let result_main = RepositoryCreationResult {
        repository_url: "https://github.com/my-org/repo1".to_string(),
        repository_id: "R_kgDOABCDEF".to_string(),
        created_at: Timestamp::now(),
        default_branch: "main".to_string(),
    };

    let result_master = RepositoryCreationResult {
        repository_url: "https://github.com/my-org/repo2".to_string(),
        repository_id: "R_kgDOGHIJKL".to_string(),
        created_at: Timestamp::now(),
        default_branch: "master".to_string(),
    };

    let result_custom = RepositoryCreationResult {
        repository_url: "https://github.com/my-org/repo3".to_string(),
        repository_id: "R_kgDOMNOPQR".to_string(),
        created_at: Timestamp::now(),
        default_branch: "develop".to_string(),
    };

    assert_eq!(result_main.default_branch, "main");
    assert_eq!(result_master.default_branch, "master");
    assert_eq!(result_custom.default_branch, "develop");
}

/// Verify that RepositoryCreationResult handles different GitHub URL formats.
#[test]
fn test_repository_creation_result_url_formats() {
    let https_result = RepositoryCreationResult {
        repository_url: "https://github.com/my-org/my-repo".to_string(),
        repository_id: "R_kgDOABCDEF".to_string(),
        created_at: Timestamp::now(),
        default_branch: "main".to_string(),
    };

    let ssh_result = RepositoryCreationResult {
        repository_url: "git@github.com:my-org/my-repo.git".to_string(),
        repository_id: "R_kgDOABCDEF".to_string(),
        created_at: Timestamp::now(),
        default_branch: "main".to_string(),
    };

    assert!(https_result.repository_url.starts_with("https://"));
    assert!(ssh_result.repository_url.starts_with("git@"));
}

/// Verify that RepositoryCreationResult stores GitHub repository IDs correctly.
#[test]
fn test_repository_creation_result_repository_id() {
    // GitHub uses base64-encoded IDs with prefix
    let result = RepositoryCreationResult {
        repository_url: "https://github.com/my-org/my-repo".to_string(),
        repository_id: "R_kgDOABCDEF".to_string(),
        created_at: Timestamp::now(),
        default_branch: "main".to_string(),
    };

    assert!(result.repository_id.starts_with("R_"));
    assert!(!result.repository_id.is_empty());
}

/// Verify that RepositoryCreationResult can represent successful creation.
#[test]
fn test_repository_creation_result_success_scenario() {
    let timestamp = Timestamp::now();

    let result = RepositoryCreationResult {
        repository_url: "https://github.com/acme-corp/awesome-project".to_string(),
        repository_id: "R_kgDOHXjK7A".to_string(),
        created_at: timestamp,
        default_branch: "main".to_string(),
    };

    // Verify all fields are populated correctly
    assert!(!result.repository_url.is_empty());
    assert!(!result.repository_id.is_empty());
    assert_eq!(result.created_at, timestamp);
    assert!(!result.default_branch.is_empty());
}
