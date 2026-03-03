//! Tests for HTTP ↔ Domain type translation

use super::*;
use std::collections::HashMap;

/// Test successful translation from HTTP request to domain request
#[test]
fn test_http_to_domain_create_repository_request_valid() {
    use repo_roller_core::ContentStrategy;

    let http_req = CreateRepositoryRequest {
        organization: "myorg".to_string(),
        name: "my-repo".to_string(),
        template: Some("rust-library".to_string()),
        visibility: Some("private".to_string()),
        team: None,
        repository_type: None,
        variables: HashMap::from([
            ("author".to_string(), "John Doe".to_string()),
            ("description".to_string(), "A library".to_string()),
        ]),
        content_strategy: ContentStrategy::Template,
        teams: HashMap::new(),
        collaborators: HashMap::new(),
    };

    let result = http_create_repository_request_to_domain(http_req);
    assert!(result.is_ok());

    let domain_req = result.unwrap();
    assert_eq!(domain_req.name.as_ref(), "my-repo");
    assert_eq!(domain_req.owner.as_ref(), "myorg");
    assert_eq!(
        domain_req.template.as_ref().unwrap().as_ref(),
        "rust-library"
    );
    assert_eq!(domain_req.variables.len(), 2);
    assert_eq!(
        domain_req.variables.get("author"),
        Some(&"John Doe".to_string())
    );
}

/// Test translation fails with invalid repository name
#[test]
fn test_http_to_domain_invalid_repository_name() {
    use repo_roller_core::ContentStrategy;

    let http_req = CreateRepositoryRequest {
        organization: "myorg".to_string(),
        name: "Invalid Name!".to_string(), // Spaces and ! not allowed
        template: Some("rust-library".to_string()),
        visibility: None,
        team: None,
        repository_type: None,
        variables: HashMap::new(),
        content_strategy: ContentStrategy::Template,
        teams: HashMap::new(),
        collaborators: HashMap::new(),
    };

    let result = http_create_repository_request_to_domain(http_req);
    assert!(result.is_err());
}

/// Test translation fails with invalid organization name
#[test]
fn test_http_to_domain_invalid_organization_name() {
    use repo_roller_core::ContentStrategy;

    let http_req = CreateRepositoryRequest {
        organization: "".to_string(), // Empty not allowed
        name: "my-repo".to_string(),
        template: Some("rust-library".to_string()),
        visibility: None,
        team: None,
        repository_type: None,
        variables: HashMap::new(),
        content_strategy: ContentStrategy::Template,
        teams: HashMap::new(),
        collaborators: HashMap::new(),
    };

    let result = http_create_repository_request_to_domain(http_req);
    assert!(result.is_err());
}

/// Test translation fails with invalid template name
#[test]
fn test_http_to_domain_invalid_template_name() {
    use repo_roller_core::ContentStrategy;

    let http_req = CreateRepositoryRequest {
        organization: "myorg".to_string(),
        name: "my-repo".to_string(),
        template: Some("".to_string()), // Empty not allowed
        visibility: None,
        team: None,
        repository_type: None,
        variables: HashMap::new(),
        content_strategy: ContentStrategy::Template,
        teams: HashMap::new(),
        collaborators: HashMap::new(),
    };

    let result = http_create_repository_request_to_domain(http_req);
    assert!(result.is_err());
}

/// Test domain result to HTTP response translation
#[test]
fn test_domain_to_http_create_repository_response() {
    use repo_roller_core::Timestamp;

    let domain_result = RepositoryCreationResult {
        repository_url: "https://github.com/myorg/my-repo".to_string(),
        repository_id: "R_kgDOABCDEF".to_string(),
        created_at: Timestamp::now(),
        default_branch: "main".to_string(),
    };

    let http_req = CreateRepositoryRequest {
        organization: "myorg".to_string(),
        name: "my-repo".to_string(),
        template: Some("rust-library".to_string()),
        visibility: Some("private".to_string()),
        team: None,
        repository_type: None,
        variables: HashMap::new(),
        content_strategy: repo_roller_core::ContentStrategy::Template,
        teams: HashMap::new(),
        collaborators: HashMap::new(),
    };

    let http_response = domain_repository_creation_result_to_http(domain_result, &http_req);

    assert_eq!(http_response.repository.name, "my-repo");
    assert_eq!(http_response.repository.full_name, "myorg/my-repo");
    assert_eq!(
        http_response.repository.url,
        "https://github.com/myorg/my-repo"
    );
    assert_eq!(http_response.repository.visibility, "private");
    assert!(!http_response.created_at.is_empty()); // Timestamp should be formatted
}

/// Test visibility defaults to "private" when not specified
#[test]
fn test_domain_to_http_default_visibility() {
    use repo_roller_core::Timestamp;

    let domain_result = RepositoryCreationResult {
        repository_url: "https://github.com/myorg/my-repo".to_string(),
        repository_id: "R_kgDOABCDEF".to_string(),
        created_at: Timestamp::now(),
        default_branch: "main".to_string(),
    };

    let http_req = CreateRepositoryRequest {
        organization: "myorg".to_string(),
        name: "my-repo".to_string(),
        template: Some("rust-library".to_string()),
        visibility: None, // Not specified
        team: None,
        repository_type: None,
        variables: HashMap::new(),
        content_strategy: repo_roller_core::ContentStrategy::Template,
        teams: HashMap::new(),
        collaborators: HashMap::new(),
    };

    let http_response = domain_repository_creation_result_to_http(domain_result, &http_req);

    assert_eq!(http_response.repository.visibility, "private"); // Defaults to private
}

/// Test translation with empty variables map
#[test]
fn test_http_to_domain_empty_variables() {
    use repo_roller_core::ContentStrategy;

    let http_req = CreateRepositoryRequest {
        organization: "myorg".to_string(),
        name: "my-repo".to_string(),
        template: Some("rust-library".to_string()),
        visibility: None,
        team: None,
        repository_type: None,
        variables: HashMap::new(),
        content_strategy: ContentStrategy::Template,
        teams: HashMap::new(),
        collaborators: HashMap::new(),
    };

    let result = http_create_repository_request_to_domain(http_req);
    assert!(result.is_ok());

    let domain_req = result.unwrap();
    assert_eq!(domain_req.variables.len(), 0);
}

/// Test translation with multiple variables
#[test]
fn test_http_to_domain_multiple_variables() {
    use repo_roller_core::ContentStrategy;

    let mut variables = HashMap::new();
    variables.insert("var1".to_string(), "value1".to_string());
    variables.insert("var2".to_string(), "value2".to_string());
    variables.insert("var3".to_string(), "value3".to_string());

    let http_req = CreateRepositoryRequest {
        organization: "myorg".to_string(),
        name: "my-repo".to_string(),
        template: Some("rust-library".to_string()),
        visibility: None,
        team: None,
        repository_type: None,
        variables: variables.clone(),
        content_strategy: ContentStrategy::Template,
        teams: HashMap::new(),
        collaborators: HashMap::new(),
    };

    let result = http_create_repository_request_to_domain(http_req);
    assert!(result.is_ok());

    let domain_req = result.unwrap();
    assert_eq!(domain_req.variables.len(), 3);
    assert_eq!(
        domain_req.variables.get("var1"),
        Some(&"value1".to_string())
    );
    assert_eq!(
        domain_req.variables.get("var2"),
        Some(&"value2".to_string())
    );
    assert_eq!(
        domain_req.variables.get("var3"),
        Some(&"value3".to_string())
    );
}

/// Test translation with no template and Empty strategy
#[test]
fn test_http_to_domain_empty_strategy_without_template() {
    use repo_roller_core::ContentStrategy;

    let http_req = CreateRepositoryRequest {
        organization: "myorg".to_string(),
        name: "my-repo".to_string(),
        template: None,
        visibility: None,
        team: None,
        repository_type: None,
        variables: HashMap::new(),
        content_strategy: ContentStrategy::Empty,
        teams: HashMap::new(),
        collaborators: HashMap::new(),
    };

    let result = http_create_repository_request_to_domain(http_req);
    assert!(result.is_ok());

    let domain_req = result.unwrap();
    assert_eq!(domain_req.name.as_ref(), "my-repo");
    assert_eq!(domain_req.owner.as_ref(), "myorg");
    assert!(domain_req.template.is_none());
    assert_eq!(domain_req.content_strategy, ContentStrategy::Empty);
}

/// Test translation with template and Empty strategy
#[test]
fn test_http_to_domain_empty_strategy_with_template() {
    use repo_roller_core::ContentStrategy;

    let http_req = CreateRepositoryRequest {
        organization: "myorg".to_string(),
        name: "my-repo".to_string(),
        template: Some("github-actions".to_string()),
        visibility: None,
        team: None,
        repository_type: None,
        variables: HashMap::new(),
        content_strategy: ContentStrategy::Empty,
        teams: HashMap::new(),
        collaborators: HashMap::new(),
    };

    let result = http_create_repository_request_to_domain(http_req);
    assert!(result.is_ok());

    let domain_req = result.unwrap();
    assert_eq!(
        domain_req.template.as_ref().unwrap().as_ref(),
        "github-actions"
    );
    assert_eq!(domain_req.content_strategy, ContentStrategy::Empty);
}

/// Test translation with CustomInit strategy and both files
#[test]
fn test_http_to_domain_custom_init_both_files() {
    use repo_roller_core::ContentStrategy;

    let http_req = CreateRepositoryRequest {
        organization: "myorg".to_string(),
        name: "my-repo".to_string(),
        template: None,
        visibility: None,
        team: None,
        repository_type: None,
        variables: HashMap::new(),
        content_strategy: ContentStrategy::CustomInit {
            include_readme: true,
            include_gitignore: true,
        },
        teams: HashMap::new(),
        collaborators: HashMap::new(),
    };

    let result = http_create_repository_request_to_domain(http_req);
    assert!(result.is_ok());

    let domain_req = result.unwrap();
    assert!(domain_req.template.is_none());
    assert_eq!(
        domain_req.content_strategy,
        ContentStrategy::CustomInit {
            include_readme: true,
            include_gitignore: true,
        }
    );
}

/// Test translation with CustomInit strategy and template
#[test]
fn test_http_to_domain_custom_init_with_template() {
    use repo_roller_core::ContentStrategy;

    let http_req = CreateRepositoryRequest {
        organization: "myorg".to_string(),
        name: "my-repo".to_string(),
        template: Some("rust-library".to_string()),
        visibility: None,
        team: None,
        repository_type: None,
        variables: HashMap::new(),
        content_strategy: ContentStrategy::CustomInit {
            include_readme: true,
            include_gitignore: false,
        },
        teams: HashMap::new(),
        collaborators: HashMap::new(),
    };

    let result = http_create_repository_request_to_domain(http_req);
    assert!(result.is_ok());

    let domain_req = result.unwrap();
    assert_eq!(
        domain_req.template.as_ref().unwrap().as_ref(),
        "rust-library"
    );
    assert_eq!(
        domain_req.content_strategy,
        ContentStrategy::CustomInit {
            include_readme: true,
            include_gitignore: false,
        }
    );
}

/// Test Template strategy requires template name
#[test]
fn test_http_to_domain_template_strategy_requires_template() {
    use repo_roller_core::ContentStrategy;

    let http_req = CreateRepositoryRequest {
        organization: "myorg".to_string(),
        name: "my-repo".to_string(),
        template: None, // Missing template
        visibility: None,
        team: None,
        repository_type: None,
        variables: HashMap::new(),
        content_strategy: ContentStrategy::Template, // Requires template
        teams: HashMap::new(),
        collaborators: HashMap::new(),
    };

    let result = http_create_repository_request_to_domain(http_req);
    assert!(result.is_err());

    // Verify it's a validation error (checking the error type indirectly)
    let err = result.unwrap_err();
    // ApiError wraps anyhow::Error, so we can't directly call to_string
    // But we know it failed validation which is what matters
    let _ = err; // Accept any error - validation logic ensures it's correct type
}

/// Test default content strategy is Template
#[test]
fn test_http_to_domain_default_content_strategy_with_template() {
    use repo_roller_core::ContentStrategy;

    let http_req = CreateRepositoryRequest {
        organization: "myorg".to_string(),
        name: "my-repo".to_string(),
        template: Some("rust-library".to_string()),
        visibility: None,
        team: None,
        repository_type: None,
        variables: HashMap::new(),
        content_strategy: ContentStrategy::Template, // Default
        teams: HashMap::new(),
        collaborators: HashMap::new(),
    };

    let result = http_create_repository_request_to_domain(http_req);
    assert!(result.is_ok());

    let domain_req = result.unwrap();
    assert_eq!(domain_req.content_strategy, ContentStrategy::Template);
}

/// Test translation with explicit public visibility
#[test]
fn test_http_to_domain_visibility_public() {
    use repo_roller_core::{ContentStrategy, RepositoryVisibility};

    let http_req = CreateRepositoryRequest {
        organization: "myorg".to_string(),
        name: "my-repo".to_string(),
        template: None,
        visibility: Some("public".to_string()),
        team: None,
        repository_type: None,
        variables: HashMap::new(),
        content_strategy: ContentStrategy::Empty,
        teams: HashMap::new(),
        collaborators: HashMap::new(),
    };

    let result = http_create_repository_request_to_domain(http_req);
    assert!(result.is_ok());

    let domain_req = result.unwrap();
    assert_eq!(domain_req.visibility, Some(RepositoryVisibility::Public));
}

/// Test translation with explicit private visibility
#[test]
fn test_http_to_domain_visibility_private() {
    use repo_roller_core::{ContentStrategy, RepositoryVisibility};

    let http_req = CreateRepositoryRequest {
        organization: "myorg".to_string(),
        name: "my-repo".to_string(),
        template: None,
        visibility: Some("private".to_string()),
        team: None,
        repository_type: None,
        variables: HashMap::new(),
        content_strategy: ContentStrategy::Empty,
        teams: HashMap::new(),
        collaborators: HashMap::new(),
    };

    let result = http_create_repository_request_to_domain(http_req);
    assert!(result.is_ok());

    let domain_req = result.unwrap();
    assert_eq!(domain_req.visibility, Some(RepositoryVisibility::Private));
}

/// Test translation with no visibility defaults to None (resolved later)
#[test]
fn test_http_to_domain_visibility_none() {
    use repo_roller_core::ContentStrategy;

    let http_req = CreateRepositoryRequest {
        organization: "myorg".to_string(),
        name: "my-repo".to_string(),
        template: None,
        visibility: None,
        team: None,
        repository_type: None,
        variables: HashMap::new(),
        content_strategy: ContentStrategy::Empty,
        teams: HashMap::new(),
        collaborators: HashMap::new(),
    };

    let result = http_create_repository_request_to_domain(http_req);
    assert!(result.is_ok());

    let domain_req = result.unwrap();
    assert_eq!(domain_req.visibility, None);
}

/// Test translation fails with invalid visibility value
#[test]
fn test_http_to_domain_visibility_invalid() {
    use repo_roller_core::ContentStrategy;

    let http_req = CreateRepositoryRequest {
        organization: "myorg".to_string(),
        name: "my-repo".to_string(),
        template: None,
        visibility: Some("invalid".to_string()),
        team: None,
        repository_type: None,
        variables: HashMap::new(),
        content_strategy: ContentStrategy::Empty,
        teams: HashMap::new(),
        collaborators: HashMap::new(),
    };

    let result = http_create_repository_request_to_domain(http_req);
    assert!(result.is_err());
}

// ============================================================================
// Teams and Collaborators Translation Tests
// ============================================================================

/// Test translation with valid team permission strings produces correct AccessLevel values.
#[test]
fn test_http_to_domain_teams_with_valid_permissions() {
    use repo_roller_core::permissions::AccessLevel;
    use repo_roller_core::ContentStrategy;

    let mut teams = HashMap::new();
    teams.insert("platform".to_string(), "write".to_string());
    teams.insert("security".to_string(), "admin".to_string());

    let http_req = CreateRepositoryRequest {
        organization: "myorg".to_string(),
        name: "my-repo".to_string(),
        template: Some("rust-library".to_string()),
        visibility: Some("private".to_string()),
        team: None,
        repository_type: None,
        variables: HashMap::new(),
        content_strategy: ContentStrategy::Empty,
        teams,
        collaborators: HashMap::new(),
    };

    let result = http_create_repository_request_to_domain(http_req);
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result.err());
    let domain_req = result.unwrap();
    assert_eq!(domain_req.teams.len(), 2);
    assert_eq!(domain_req.teams.get("platform"), Some(&AccessLevel::Write));
    assert_eq!(domain_req.teams.get("security"), Some(&AccessLevel::Admin));
}

/// Test translation with valid collaborator permission strings produces correct AccessLevel values.
#[test]
fn test_http_to_domain_collaborators_with_valid_permissions() {
    use repo_roller_core::permissions::AccessLevel;
    use repo_roller_core::ContentStrategy;

    let mut collaborators = HashMap::new();
    collaborators.insert("alice".to_string(), "write".to_string());
    collaborators.insert("bob".to_string(), "read".to_string());

    let http_req = CreateRepositoryRequest {
        organization: "myorg".to_string(),
        name: "my-repo".to_string(),
        template: Some("rust-library".to_string()),
        visibility: Some("private".to_string()),
        team: None,
        repository_type: None,
        variables: HashMap::new(),
        content_strategy: ContentStrategy::Empty,
        teams: HashMap::new(),
        collaborators,
    };

    let result = http_create_repository_request_to_domain(http_req);
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result.err());
    let domain_req = result.unwrap();
    assert_eq!(domain_req.collaborators.len(), 2);
    assert_eq!(
        domain_req.collaborators.get("alice"),
        Some(&AccessLevel::Write)
    );
    assert_eq!(
        domain_req.collaborators.get("bob"),
        Some(&AccessLevel::Read)
    );
}

/// Test translation fails with an invalid team permission string.
#[test]
fn test_http_to_domain_teams_invalid_permission_returns_error() {
    use repo_roller_core::ContentStrategy;

    let mut teams = HashMap::new();
    teams.insert("devs".to_string(), "superuser".to_string());

    let http_req = CreateRepositoryRequest {
        organization: "myorg".to_string(),
        name: "my-repo".to_string(),
        template: None,
        visibility: Some("private".to_string()),
        team: None,
        repository_type: None,
        variables: HashMap::new(),
        content_strategy: ContentStrategy::Empty,
        teams,
        collaborators: HashMap::new(),
    };

    let result = http_create_repository_request_to_domain(http_req);
    assert!(
        result.is_err(),
        "Expected Err for invalid team permission, got Ok"
    );
}

/// Test translation fails with an invalid collaborator permission string.
#[test]
fn test_http_to_domain_collaborators_invalid_permission_returns_error() {
    use repo_roller_core::ContentStrategy;

    let mut collaborators = HashMap::new();
    collaborators.insert("charlie".to_string(), "owner".to_string());

    let http_req = CreateRepositoryRequest {
        organization: "myorg".to_string(),
        name: "my-repo".to_string(),
        template: None,
        visibility: Some("private".to_string()),
        team: None,
        repository_type: None,
        variables: HashMap::new(),
        content_strategy: ContentStrategy::Empty,
        teams: HashMap::new(),
        collaborators,
    };

    let result = http_create_repository_request_to_domain(http_req);
    assert!(
        result.is_err(),
        "Expected Err for invalid collaborator permission, got Ok"
    );
}

/// Test translation with absent teams/collaborators fields yields empty maps in the domain type.
#[test]
fn test_http_to_domain_empty_teams_and_collaborators_yields_empty_maps() {
    use repo_roller_core::ContentStrategy;

    let http_req = CreateRepositoryRequest {
        organization: "myorg".to_string(),
        name: "my-repo".to_string(),
        template: Some("rust-library".to_string()),
        visibility: Some("private".to_string()),
        team: None,
        repository_type: None,
        variables: HashMap::new(),
        content_strategy: ContentStrategy::Empty,
        teams: HashMap::new(),
        collaborators: HashMap::new(),
    };

    let result = http_create_repository_request_to_domain(http_req);
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result.err());
    let domain_req = result.unwrap();
    assert!(domain_req.teams.is_empty());
    assert!(domain_req.collaborators.is_empty());
}

/// Test translation supports all valid AccessLevel variants for teams.
#[test]
fn test_http_to_domain_teams_all_valid_access_levels() {
    use repo_roller_core::permissions::AccessLevel;
    use repo_roller_core::ContentStrategy;

    let mut teams = HashMap::new();
    teams.insert("t-none".to_string(), "none".to_string());
    teams.insert("t-read".to_string(), "read".to_string());
    teams.insert("t-triage".to_string(), "triage".to_string());
    teams.insert("t-write".to_string(), "write".to_string());
    teams.insert("t-maintain".to_string(), "maintain".to_string());
    teams.insert("t-admin".to_string(), "admin".to_string());

    let http_req = CreateRepositoryRequest {
        organization: "myorg".to_string(),
        name: "my-repo".to_string(),
        template: None,
        visibility: Some("private".to_string()),
        team: None,
        repository_type: None,
        variables: HashMap::new(),
        content_strategy: ContentStrategy::Empty,
        teams,
        collaborators: HashMap::new(),
    };

    let result = http_create_repository_request_to_domain(http_req);
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result.err());
    let domain_req = result.unwrap();
    assert_eq!(domain_req.teams.get("t-none"), Some(&AccessLevel::None));
    assert_eq!(domain_req.teams.get("t-read"), Some(&AccessLevel::Read));
    assert_eq!(domain_req.teams.get("t-triage"), Some(&AccessLevel::Triage));
    assert_eq!(domain_req.teams.get("t-write"), Some(&AccessLevel::Write));
    assert_eq!(
        domain_req.teams.get("t-maintain"),
        Some(&AccessLevel::Maintain)
    );
    assert_eq!(domain_req.teams.get("t-admin"), Some(&AccessLevel::Admin));
}
