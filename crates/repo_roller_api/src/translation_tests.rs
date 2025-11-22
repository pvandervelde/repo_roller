//! Tests for HTTP ↔ Domain type translation

use super::*;
use std::collections::HashMap;

/// Test successful translation from HTTP request to domain request
#[test]
fn test_http_to_domain_create_repository_request_valid() {
    let http_req = CreateRepositoryRequest {
        organization: "myorg".to_string(),
        name: "my-repo".to_string(),
        template: "rust-library".to_string(),
        visibility: Some("private".to_string()),
        team: None,
        repository_type: None,
        variables: HashMap::from([
            ("author".to_string(), "John Doe".to_string()),
            ("description".to_string(), "A library".to_string()),
        ]),
    };

    let result = http_create_repository_request_to_domain(http_req);
    assert!(result.is_ok());

    let domain_req = result.unwrap();
    assert_eq!(domain_req.name.as_ref(), "my-repo");
    assert_eq!(domain_req.owner.as_ref(), "myorg");
    assert_eq!(domain_req.template.as_ref(), "rust-library");
    assert_eq!(domain_req.variables.len(), 2);
    assert_eq!(
        domain_req.variables.get("author"),
        Some(&"John Doe".to_string())
    );
}

/// Test translation fails with invalid repository name
#[test]
fn test_http_to_domain_invalid_repository_name() {
    let http_req = CreateRepositoryRequest {
        organization: "myorg".to_string(),
        name: "Invalid Name!".to_string(), // Spaces and ! not allowed
        template: "rust-library".to_string(),
        visibility: None,
        team: None,
        repository_type: None,
        variables: HashMap::new(),
    };

    let result = http_create_repository_request_to_domain(http_req);
    assert!(result.is_err());
}

/// Test translation fails with invalid organization name
#[test]
fn test_http_to_domain_invalid_organization_name() {
    let http_req = CreateRepositoryRequest {
        organization: "".to_string(), // Empty not allowed
        name: "my-repo".to_string(),
        template: "rust-library".to_string(),
        visibility: None,
        team: None,
        repository_type: None,
        variables: HashMap::new(),
    };

    let result = http_create_repository_request_to_domain(http_req);
    assert!(result.is_err());
}

/// Test translation fails with invalid template name
#[test]
fn test_http_to_domain_invalid_template_name() {
    let http_req = CreateRepositoryRequest {
        organization: "myorg".to_string(),
        name: "my-repo".to_string(),
        template: "".to_string(), // Empty not allowed
        visibility: None,
        team: None,
        repository_type: None,
        variables: HashMap::new(),
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
        template: "rust-library".to_string(),
        visibility: Some("private".to_string()),
        team: None,
        repository_type: None,
        variables: HashMap::new(),
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
        template: "rust-library".to_string(),
        visibility: None, // Not specified
        team: None,
        repository_type: None,
        variables: HashMap::new(),
    };

    let http_response = domain_repository_creation_result_to_http(domain_result, &http_req);

    assert_eq!(http_response.repository.visibility, "private"); // Defaults to private
}

/// Test translation with empty variables map
#[test]
fn test_http_to_domain_empty_variables() {
    let http_req = CreateRepositoryRequest {
        organization: "myorg".to_string(),
        name: "my-repo".to_string(),
        template: "rust-library".to_string(),
        visibility: None,
        team: None,
        repository_type: None,
        variables: HashMap::new(),
    };

    let result = http_create_repository_request_to_domain(http_req);
    assert!(result.is_ok());

    let domain_req = result.unwrap();
    assert_eq!(domain_req.variables.len(), 0);
}

/// Test translation with multiple variables
#[test]
fn test_http_to_domain_multiple_variables() {
    let mut variables = HashMap::new();
    variables.insert("var1".to_string(), "value1".to_string());
    variables.insert("var2".to_string(), "value2".to_string());
    variables.insert("var3".to_string(), "value3".to_string());

    let http_req = CreateRepositoryRequest {
        organization: "myorg".to_string(),
        name: "my-repo".to_string(),
        template: "rust-library".to_string(),
        visibility: None,
        team: None,
        repository_type: None,
        variables: variables.clone(),
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
