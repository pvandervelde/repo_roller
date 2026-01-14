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
        template: Some(template.clone()),
        variables: variables.clone(),
        visibility: None,
        content_strategy: ContentStrategy::Template,
    };

    assert_eq!(request.name, name);
    assert_eq!(request.owner, owner);
    assert_eq!(request.template, Some(template));
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
        template: Some(TemplateName::new("rust-library").unwrap()),
        variables: variables.clone(),
        visibility: None,
        content_strategy: ContentStrategy::Template,
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
        template: Some(TemplateName::new("rust-library").unwrap()),
        content_strategy: ContentStrategy::Template,
        variables: HashMap::new(),
        visibility: None,
    };

    let cloned = request.clone();

    assert_eq!(request.name, cloned.name);
    assert_eq!(request.owner, cloned.owner);
    assert_eq!(request.template, cloned.template);
    assert_eq!(request.content_strategy, cloned.content_strategy);
    assert_eq!(request.variables, cloned.variables);
}

/// Verify that RepositoryCreationRequest has Debug output.
#[test]
fn test_repository_creation_request_debug() {
    let request = RepositoryCreationRequest {
        name: RepositoryName::new("my-repo").unwrap(),
        owner: OrganizationName::new("my-org").unwrap(),
        template: Some(TemplateName::new("rust-library").unwrap()),
        content_strategy: ContentStrategy::Template,
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
        name,                     // RepositoryName
        owner,                    // OrganizationName
        template: Some(template), // Option<TemplateName>
        content_strategy: ContentStrategy::Template,
        variables: HashMap::new(),
        visibility: None,
    };

    // Verify we can access the values
    assert_eq!(request.name.as_str(), "my-repo");
    assert_eq!(request.owner.as_str(), "my-org");
    assert_eq!(request.template.as_ref().unwrap().as_str(), "rust-library");
}

/// Verify that RepositoryCreationRequest can handle empty variables.
#[test]
fn test_repository_creation_request_empty_variables() {
    let request = RepositoryCreationRequest {
        name: RepositoryName::new("my-repo").unwrap(),
        owner: OrganizationName::new("my-org").unwrap(),
        template: Some(TemplateName::new("rust-library").unwrap()),
        content_strategy: ContentStrategy::Template,
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
        template: Some(TemplateName::new("valid-template").unwrap()),
        content_strategy: ContentStrategy::Template,
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
    )
    .template(TemplateName::new("rust-library").unwrap())
    .build();

    assert_eq!(request.name.as_str(), "my-repo");
    assert_eq!(request.owner.as_str(), "my-org");
    assert_eq!(request.template.as_ref().unwrap().as_str(), "rust-library");
    assert!(request.variables.is_empty());
}

/// Verify that builder can add a single variable.
#[test]
fn test_builder_single_variable() {
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("my-repo").unwrap(),
        OrganizationName::new("my-org").unwrap(),
    )
    .template(TemplateName::new("rust-library").unwrap())
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
    )
    .template(TemplateName::new("rust-library").unwrap())
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
    )
    .template(TemplateName::new("rust-library").unwrap())
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
    )
    .template(TemplateName::new("rust-library").unwrap())
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
    )
    .template(TemplateName::new("rust-library").unwrap())
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
    )
    .template(TemplateName::new("rust-library").unwrap())
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
    )
    .template(TemplateName::new("rust-library").unwrap());

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
    )
    .template(TemplateName::new("rust-library").unwrap())
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

// ============================================================================
// ContentStrategy Tests (Task 6.9)
// ============================================================================

/// Test builder with Empty content strategy and no template.
///
/// Verifies that Empty strategy works without a template, using
/// organization defaults for settings.
#[test]
fn test_builder_empty_strategy_without_template() {
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("empty-repo").unwrap(),
        OrganizationName::new("myorg").unwrap(),
    )
    .content_strategy(ContentStrategy::Empty)
    .build();

    assert_eq!(request.name.as_ref(), "empty-repo");
    assert_eq!(request.owner.as_ref(), "myorg");
    assert!(request.template.is_none());
    assert_eq!(request.content_strategy, ContentStrategy::Empty);
}

/// Test builder with Empty content strategy and template.
///
/// Verifies that Empty strategy can use a template for settings
/// while creating no files.
#[test]
fn test_builder_empty_strategy_with_template() {
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("empty-repo").unwrap(),
        OrganizationName::new("myorg").unwrap(),
    )
    .template(TemplateName::new("github-actions").unwrap())
    .content_strategy(ContentStrategy::Empty)
    .build();

    assert_eq!(
        request.template.as_ref().unwrap().as_ref(),
        "github-actions"
    );
    assert_eq!(request.content_strategy, ContentStrategy::Empty);
}

/// Test builder with CustomInit strategy and README only.
///
/// Verifies that CustomInit can create README.md without .gitignore.
#[test]
fn test_builder_custom_init_readme_only() {
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("quick-start").unwrap(),
        OrganizationName::new("myorg").unwrap(),
    )
    .content_strategy(ContentStrategy::CustomInit {
        include_readme: true,
        include_gitignore: false,
    })
    .build();

    assert_eq!(request.name.as_ref(), "quick-start");
    assert!(request.template.is_none());
    assert_eq!(
        request.content_strategy,
        ContentStrategy::CustomInit {
            include_readme: true,
            include_gitignore: false,
        }
    );
}

/// Test builder with CustomInit strategy and gitignore only.
///
/// Verifies that CustomInit can create .gitignore without README.md.
#[test]
fn test_builder_custom_init_gitignore_only() {
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("my-project").unwrap(),
        OrganizationName::new("myorg").unwrap(),
    )
    .content_strategy(ContentStrategy::CustomInit {
        include_readme: false,
        include_gitignore: true,
    })
    .build();

    assert_eq!(
        request.content_strategy,
        ContentStrategy::CustomInit {
            include_readme: false,
            include_gitignore: true,
        }
    );
}

/// Test builder with CustomInit strategy and both files.
///
/// Verifies that CustomInit can create both README.md and .gitignore.
#[test]
fn test_builder_custom_init_both_files() {
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("full-init").unwrap(),
        OrganizationName::new("myorg").unwrap(),
    )
    .content_strategy(ContentStrategy::CustomInit {
        include_readme: true,
        include_gitignore: true,
    })
    .build();

    assert_eq!(
        request.content_strategy,
        ContentStrategy::CustomInit {
            include_readme: true,
            include_gitignore: true,
        }
    );
}

/// Test builder with CustomInit strategy and template.
///
/// Verifies that CustomInit can use template settings while
/// only creating custom initialization files.
#[test]
fn test_builder_custom_init_with_template() {
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("hybrid-repo").unwrap(),
        OrganizationName::new("myorg").unwrap(),
    )
    .template(TemplateName::new("rust-library").unwrap())
    .content_strategy(ContentStrategy::CustomInit {
        include_readme: true,
        include_gitignore: false,
    })
    .build();

    assert_eq!(request.template.as_ref().unwrap().as_ref(), "rust-library");
    assert_eq!(
        request.content_strategy,
        ContentStrategy::CustomInit {
            include_readme: true,
            include_gitignore: false,
        }
    );
}

/// Test builder with CustomInit strategy with both false.
///
/// Verifies that CustomInit with both options false creates
/// empty repository (equivalent to Empty strategy).
#[test]
fn test_builder_custom_init_both_false() {
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("minimal-repo").unwrap(),
        OrganizationName::new("myorg").unwrap(),
    )
    .content_strategy(ContentStrategy::CustomInit {
        include_readme: false,
        include_gitignore: false,
    })
    .build();

    assert_eq!(
        request.content_strategy,
        ContentStrategy::CustomInit {
            include_readme: false,
            include_gitignore: false,
        }
    );
}

/// Test ContentStrategy::default() returns Template.
///
/// Verifies backward compatibility - default strategy is Template.
#[test]
fn test_content_strategy_default() {
    let strategy = ContentStrategy::default();
    assert_eq!(strategy, ContentStrategy::Template);
}

/// Test ContentStrategy serialization for Template.
#[test]
fn test_content_strategy_serialize_template() {
    let strategy = ContentStrategy::Template;
    let json = serde_json::to_string(&strategy).unwrap();
    assert_eq!(json, r#"{"type":"template"}"#);
}

/// Test ContentStrategy serialization for Empty.
#[test]
fn test_content_strategy_serialize_empty() {
    let strategy = ContentStrategy::Empty;
    let json = serde_json::to_string(&strategy).unwrap();
    assert_eq!(json, r#"{"type":"empty"}"#);
}

/// Test ContentStrategy serialization for CustomInit.
#[test]
fn test_content_strategy_serialize_custom_init() {
    let strategy = ContentStrategy::CustomInit {
        include_readme: true,
        include_gitignore: false,
    };
    let json = serde_json::to_string(&strategy).unwrap();
    assert!(json.contains(r#""type":"custom_init""#));
    assert!(json.contains(r#""include_readme":true"#));
    assert!(json.contains(r#""include_gitignore":false"#));
}

/// Test ContentStrategy deserialization for Template.
#[test]
fn test_content_strategy_deserialize_template() {
    let json = r#"{"type":"template"}"#;
    let strategy: ContentStrategy = serde_json::from_str(json).unwrap();
    assert_eq!(strategy, ContentStrategy::Template);
}

/// Test ContentStrategy deserialization for Empty.
#[test]
fn test_content_strategy_deserialize_empty() {
    let json = r#"{"type":"empty"}"#;
    let strategy: ContentStrategy = serde_json::from_str(json).unwrap();
    assert_eq!(strategy, ContentStrategy::Empty);
}

/// Test ContentStrategy deserialization for CustomInit.
#[test]
fn test_content_strategy_deserialize_custom_init() {
    let json = r#"{"type":"custom_init","include_readme":true,"include_gitignore":false}"#;
    let strategy: ContentStrategy = serde_json::from_str(json).unwrap();
    assert_eq!(
        strategy,
        ContentStrategy::CustomInit {
            include_readme: true,
            include_gitignore: false,
        }
    );
}

/// Test ContentStrategy Clone trait.
#[test]
fn test_content_strategy_clone() {
    let strategy = ContentStrategy::CustomInit {
        include_readme: true,
        include_gitignore: true,
    };
    let cloned = strategy.clone();
    assert_eq!(strategy, cloned);
}

/// Test ContentStrategy Debug trait.
#[test]
fn test_content_strategy_debug() {
    let strategy = ContentStrategy::Empty;
    let debug_str = format!("{:?}", strategy);
    assert!(debug_str.contains("Empty"));
}

/// Test RepositoryCreationRequest with Empty strategy validates correctly.
#[test]
fn test_request_empty_strategy_validation() {
    let request = RepositoryCreationRequest {
        name: RepositoryName::new("test-repo").unwrap(),
        owner: OrganizationName::new("test-org").unwrap(),
        template: None,
        variables: HashMap::new(),
        visibility: None,
        content_strategy: ContentStrategy::Empty,
    };

    // Should not panic or error - Empty strategy doesn't require template
    assert_eq!(request.content_strategy, ContentStrategy::Empty);
}

/// Test RepositoryCreationRequest with CustomInit strategy validates correctly.
#[test]
fn test_request_custom_init_strategy_validation() {
    let request = RepositoryCreationRequest {
        name: RepositoryName::new("test-repo").unwrap(),
        owner: OrganizationName::new("test-org").unwrap(),
        template: None,
        variables: HashMap::new(),
        visibility: None,
        content_strategy: ContentStrategy::CustomInit {
            include_readme: true,
            include_gitignore: true,
        },
    };

    // Should not panic or error - CustomInit strategy doesn't require template
    assert_eq!(
        request.content_strategy,
        ContentStrategy::CustomInit {
            include_readme: true,
            include_gitignore: true,
        }
    );
}

/// Test builder defaults to Template strategy when not specified.
#[test]
fn test_builder_defaults_to_template_strategy() {
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("test-repo").unwrap(),
        OrganizationName::new("test-org").unwrap(),
    )
    .template(TemplateName::new("rust-lib").unwrap())
    .build();

    // Default strategy should be Template
    assert_eq!(request.content_strategy, ContentStrategy::Template);
}

/// Test builder with Empty strategy and variables.
///
/// Verifies that variables can be passed even with Empty strategy
/// (might be used for settings substitution).
#[test]
fn test_builder_empty_strategy_with_variables() {
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("test-repo").unwrap(),
        OrganizationName::new("test-org").unwrap(),
    )
    .content_strategy(ContentStrategy::Empty)
    .variable("key1", "value1")
    .variable("key2", "value2")
    .build();

    assert_eq!(request.content_strategy, ContentStrategy::Empty);
    assert_eq!(request.variables.len(), 2);
    assert_eq!(request.variables.get("key1"), Some(&"value1".to_string()));
}

/// Test builder with CustomInit strategy and variables.
///
/// Verifies that variables can be passed with CustomInit strategy.
#[test]
fn test_builder_custom_init_strategy_with_variables() {
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("test-repo").unwrap(),
        OrganizationName::new("test-org").unwrap(),
    )
    .content_strategy(ContentStrategy::CustomInit {
        include_readme: true,
        include_gitignore: true,
    })
    .variable("project_name", "MyProject")
    .build();

    assert_eq!(
        request.content_strategy,
        ContentStrategy::CustomInit {
            include_readme: true,
            include_gitignore: true,
        }
    );
    assert_eq!(request.variables.len(), 1);
}
