//! Tests for error types
//!
//! This module provides comprehensive test coverage for all error types
//! defined in the errors module, ensuring proper error creation, display
//! formatting, and helper method functionality.

use super::*;

// ============================================================================
// ValidationError Tests
// ============================================================================

#[test]
fn test_validation_error_empty_field() {
    let err = ValidationError::empty_field("test_field");

    assert_eq!(err.to_string(), "Field 'test_field' cannot be empty");

    // Verify the error matches the expected variant
    match err {
        ValidationError::EmptyField { field } => {
            assert_eq!(field, "test_field");
        }
        _ => panic!("Expected EmptyField variant"),
    }
}

#[test]
fn test_validation_error_empty_field_with_string() {
    let field_name = "repository_name".to_string();
    let err = ValidationError::empty_field(field_name);

    assert_eq!(err.to_string(), "Field 'repository_name' cannot be empty");
}

#[test]
fn test_validation_error_too_long() {
    let err = ValidationError::too_long("name", 150, 100);

    assert_eq!(
        err.to_string(),
        "Field 'name' is too long: 150 characters (max: 100)"
    );

    match err {
        ValidationError::TooLong { field, actual, max } => {
            assert_eq!(field, "name");
            assert_eq!(actual, 150);
            assert_eq!(max, 100);
        }
        _ => panic!("Expected TooLong variant"),
    }
}

#[test]
fn test_validation_error_too_long_edge_case() {
    // Test when actual is exactly max + 1
    let err = ValidationError::too_long("description", 101, 100);

    assert!(err.to_string().contains("101 characters"));
    assert!(err.to_string().contains("max: 100"));
}

#[test]
fn test_validation_error_too_short() {
    let err = ValidationError::too_short("password", 3, 8);

    assert_eq!(
        err.to_string(),
        "Field 'password' is too short: 3 characters (min: 8)"
    );

    match err {
        ValidationError::TooShort { field, actual, min } => {
            assert_eq!(field, "password");
            assert_eq!(actual, 3);
            assert_eq!(min, 8);
        }
        _ => panic!("Expected TooShort variant"),
    }
}

#[test]
fn test_validation_error_too_short_zero_length() {
    let err = ValidationError::too_short("username", 0, 1);

    assert!(err.to_string().contains("0 characters"));
    assert!(err.to_string().contains("min: 1"));
}

#[test]
fn test_validation_error_invalid_format() {
    let err = ValidationError::invalid_format("email", "must contain @ symbol");

    assert_eq!(
        err.to_string(),
        "Field 'email' has invalid format: must contain @ symbol"
    );

    match err {
        ValidationError::InvalidFormat { field, reason } => {
            assert_eq!(field, "email");
            assert_eq!(reason, "must contain @ symbol");
        }
        _ => panic!("Expected InvalidFormat variant"),
    }
}

#[test]
fn test_validation_error_invalid_format_complex_reason() {
    let err = ValidationError::invalid_format(
        "repository_name",
        "must start with a letter, contain only lowercase letters, numbers, and hyphens",
    );

    assert!(err.to_string().contains("repository_name"));
    assert!(err.to_string().contains("must start with a letter"));
}

// ============================================================================
// ValidationError Equality and Clone Tests
// ============================================================================

#[test]
fn test_validation_error_equality() {
    let err1 = ValidationError::empty_field("test");
    let err2 = ValidationError::empty_field("test");
    let err3 = ValidationError::empty_field("other");

    assert_eq!(err1, err2);
    assert_ne!(err1, err3);
}

#[test]
fn test_validation_error_clone() {
    let original = ValidationError::too_long("name", 100, 50);
    let cloned = original.clone();

    assert_eq!(original, cloned);
}

#[test]
fn test_validation_error_clone_all_variants() {
    let errors = vec![
        ValidationError::empty_field("field1"),
        ValidationError::too_long("field2", 100, 50),
        ValidationError::too_short("field3", 2, 5),
        ValidationError::invalid_format("field4", "bad format"),
    ];

    for original in errors {
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }
}

// ============================================================================
// ValidationError Debug Format Tests
// ============================================================================

#[test]
fn test_validation_error_debug_format() {
    let err = ValidationError::empty_field("test");
    let debug_output = format!("{:?}", err);

    // Debug output should contain the variant name and field value
    assert!(debug_output.contains("EmptyField"));
    assert!(debug_output.contains("test"));
}

#[test]
fn test_validation_error_debug_all_variants() {
    let errors = vec![
        ValidationError::empty_field("field1"),
        ValidationError::too_long("field2", 100, 50),
        ValidationError::too_short("field3", 2, 5),
        ValidationError::invalid_format("field4", "reason"),
    ];

    for err in errors {
        let debug = format!("{:?}", err);
        // All debug outputs should be non-empty and meaningful
        assert!(!debug.is_empty());
    }
}

// ============================================================================
// ValidationError Constructor Tests
// ============================================================================

#[test]
fn test_validation_error_constructors_with_borrowed_strings() {
    // Test that constructors work with &str
    let err1 = ValidationError::empty_field("borrowed");
    let err2 = ValidationError::too_long("borrowed", 10, 5);
    let err3 = ValidationError::too_short("borrowed", 1, 5);
    let err4 = ValidationError::invalid_format("borrowed", "borrowed reason");

    // All should construct successfully
    assert!(err1.to_string().contains("borrowed"));
    assert!(err2.to_string().contains("borrowed"));
    assert!(err3.to_string().contains("borrowed"));
    assert!(err4.to_string().contains("borrowed"));
}

#[test]
fn test_validation_error_constructors_with_owned_strings() {
    // Test that constructors work with String
    let field = "owned".to_string();
    let reason = "owned reason".to_string();

    let err1 = ValidationError::empty_field(field.clone());
    let err2 = ValidationError::too_long(field.clone(), 10, 5);
    let err3 = ValidationError::too_short(field.clone(), 1, 5);
    let err4 = ValidationError::invalid_format(field, reason);

    // All should construct successfully
    assert!(err1.to_string().contains("owned"));
    assert!(err2.to_string().contains("owned"));
    assert!(err3.to_string().contains("owned"));
    assert!(err4.to_string().contains("owned"));
}

// ============================================================================
// ValidationError Display Format Tests
// ============================================================================

#[test]
fn test_validation_error_display_format_completeness() {
    // Verify all fields are included in display output
    let err = ValidationError::TooLong {
        field: "test_field".to_string(),
        actual: 150,
        max: 100,
    };

    let display = err.to_string();
    assert!(display.contains("test_field"));
    assert!(display.contains("150"));
    assert!(display.contains("100"));
}

#[test]
fn test_validation_error_display_vs_debug() {
    let err = ValidationError::empty_field("test");

    let display = err.to_string();
    let debug = format!("{:?}", err);

    // Display should be user-friendly, debug should show structure
    assert!(!display.contains("EmptyField")); // Display hides variant name
    assert!(debug.contains("EmptyField")); // Debug shows variant name
}

// ============================================================================
// Integration Tests - ValidationError with Real Domain Types
// ============================================================================

#[test]
fn test_validation_error_for_repository_name() {
    // Simulate real validation errors for repository names
    let err = ValidationError::too_long("repository_name", 150, 100);
    assert!(err.to_string().contains("repository_name"));

    let err =
        ValidationError::invalid_format("repository_name", "must match pattern ^[a-z][a-z0-9-]*$");
    assert!(err.to_string().contains("repository_name"));
    assert!(err.to_string().contains("pattern"));
}

#[test]
fn test_validation_error_for_organization_name() {
    // Simulate real validation errors for organization names
    let err = ValidationError::empty_field("organization_name");
    assert!(err.to_string().contains("organization_name"));

    let err = ValidationError::too_short("organization_name", 1, 2);
    assert!(err.to_string().contains("organization_name"));
}

#[test]
fn test_validation_error_for_template_name() {
    // Simulate real validation errors for template names
    let err = ValidationError::invalid_format("template_name", "must use kebab-case format");
    assert!(err.to_string().contains("template_name"));
    assert!(err.to_string().contains("kebab-case"));
}

// ============================================================================
// Edge Cases and Boundary Tests
// ============================================================================

#[test]
fn test_validation_error_with_unicode_field_names() {
    let err = ValidationError::empty_field("名前");
    assert!(err.to_string().contains("名前"));
}

#[test]
fn test_validation_error_with_special_characters() {
    let err = ValidationError::invalid_format("field-with-dashes", "must not contain <>&");
    assert!(err.to_string().contains("field-with-dashes"));
    assert!(err.to_string().contains("<>&"));
}

#[test]
fn test_validation_error_with_empty_reason() {
    let err = ValidationError::invalid_format("field", "");
    assert!(err.to_string().contains("field"));
}

#[test]
fn test_validation_error_with_very_long_reason() {
    let long_reason = "a".repeat(1000);
    let err = ValidationError::invalid_format("field", &long_reason);
    assert!(err.to_string().contains(&long_reason));
}

#[test]
fn test_validation_error_length_boundary_values() {
    // Test with max length values
    let err = ValidationError::too_long("field", usize::MAX, 100);
    assert!(err.to_string().contains(&usize::MAX.to_string()));

    let err = ValidationError::too_short("field", 0, usize::MAX);
    assert!(err.to_string().contains(&usize::MAX.to_string()));
}

// ============================================================================
// RepoRollerError Hierarchy Tests
// ============================================================================

#[test]
fn test_repo_roller_error_from_validation_error() {
    let validation_err = ValidationError::empty_field("test_field");
    let repo_err: RepoRollerError = validation_err.into();

    assert!(matches!(repo_err, RepoRollerError::Validation(_)));
    assert!(repo_err.to_string().contains("Validation error"));
}

#[test]
fn test_repo_roller_error_from_repository_error() {
    let repo_specific_err = RepositoryError::AlreadyExists {
        org: "my-org".to_string(),
        name: "my-repo".to_string(),
    };
    let repo_err: RepoRollerError = repo_specific_err.into();

    assert!(matches!(repo_err, RepoRollerError::Repository(_)));
    assert!(repo_err.to_string().contains("Repository error"));
}

#[test]
fn test_repo_roller_error_from_configuration_error() {
    let config_err = ConfigurationError::FileNotFound {
        path: "/path/to/config.toml".to_string(),
    };
    let repo_err: RepoRollerError = config_err.into();

    assert!(matches!(repo_err, RepoRollerError::Configuration(_)));
    assert!(repo_err.to_string().contains("Configuration error"));
}

#[test]
fn test_repo_roller_error_from_template_error() {
    let template_err = TemplateError::TemplateNotFound {
        name: "rust-library".to_string(),
    };
    let repo_err: RepoRollerError = template_err.into();

    assert!(matches!(repo_err, RepoRollerError::Template(_)));
    assert!(repo_err.to_string().contains("Template processing error"));
}

#[test]
fn test_repo_roller_error_from_authentication_error() {
    let auth_err = AuthenticationError::InvalidToken;
    let repo_err: RepoRollerError = auth_err.into();

    assert!(matches!(repo_err, RepoRollerError::Authentication(_)));
    assert!(repo_err.to_string().contains("Authentication error"));
}

#[test]
fn test_repo_roller_error_from_github_error() {
    let github_err = GitHubError::ResourceNotFound {
        resource: "repository".to_string(),
    };
    let repo_err: RepoRollerError = github_err.into();

    assert!(matches!(repo_err, RepoRollerError::GitHub(_)));
    assert!(repo_err.to_string().contains("GitHub API error"));
}

#[test]
fn test_repo_roller_error_from_system_error() {
    let system_err = SystemError::Internal {
        reason: "unexpected failure".to_string(),
    };
    let repo_err: RepoRollerError = system_err.into();

    assert!(matches!(repo_err, RepoRollerError::System(_)));
    assert!(repo_err.to_string().contains("System error"));
}

// ============================================================================
// ValidationError Domain-Specific Tests
// ============================================================================

#[test]
fn test_validation_error_invalid_repository_name() {
    let err = ValidationError::InvalidRepositoryName {
        reason: "contains invalid characters".to_string(),
    };

    assert!(err.to_string().contains("Invalid repository name"));
    assert!(err.to_string().contains("contains invalid characters"));
}

#[test]
fn test_validation_error_invalid_organization_name() {
    let err = ValidationError::InvalidOrganizationName {
        reason: "too long".to_string(),
    };

    assert!(err.to_string().contains("Invalid organization name"));
    assert!(err.to_string().contains("too long"));
}

#[test]
fn test_validation_error_invalid_template_name() {
    let err = ValidationError::InvalidTemplateName {
        reason: "not in kebab-case".to_string(),
    };

    assert!(err.to_string().contains("Invalid template name"));
    assert!(err.to_string().contains("not in kebab-case"));
}

#[test]
fn test_validation_error_required_field_missing() {
    let err = ValidationError::RequiredFieldMissing {
        field: "repository_name".to_string(),
    };

    assert!(err.to_string().contains("Required field missing"));
    assert!(err.to_string().contains("repository_name"));
}

#[test]
fn test_validation_error_pattern_mismatch() {
    let err = ValidationError::PatternMismatch {
        field: "repo_name".to_string(),
        pattern: "^[a-z]+$".to_string(),
        value: "MyRepo123".to_string(),
    };

    assert!(err.to_string().contains("must match pattern"));
    assert!(err.to_string().contains("^[a-z]+$"));
    assert!(err.to_string().contains("MyRepo123"));
}

#[test]
fn test_validation_error_length_constraint() {
    let err = ValidationError::LengthConstraint {
        field: "description".to_string(),
        min: 10,
        max: 100,
        actual: 150,
    };

    assert!(err
        .to_string()
        .contains("length must be between 10 and 100"));
    assert!(err.to_string().contains("got 150"));
}

#[test]
fn test_validation_error_invalid_option() {
    let err = ValidationError::InvalidOption {
        field: "visibility".to_string(),
        options: vec!["public".to_string(), "private".to_string()],
        value: "internal".to_string(),
    };

    assert!(err.to_string().contains("must be one of"));
    assert!(err.to_string().contains("public"));
    assert!(err.to_string().contains("private"));
    assert!(err.to_string().contains("internal"));
}

// ============================================================================
// RepositoryError Tests
// ============================================================================

#[test]
fn test_repository_error_already_exists() {
    let err = RepositoryError::AlreadyExists {
        org: "my-org".to_string(),
        name: "my-repo".to_string(),
    };

    assert!(err.to_string().contains("already exists"));
    assert!(err.to_string().contains("my-org"));
    assert!(err.to_string().contains("my-repo"));
}

#[test]
fn test_repository_error_not_found() {
    let err = RepositoryError::NotFound {
        org: "test-org".to_string(),
        name: "test-repo".to_string(),
    };

    assert!(err.to_string().contains("not found"));
    assert!(err.to_string().contains("test-org/test-repo"));
}

#[test]
fn test_repository_error_creation_failed() {
    let err = RepositoryError::CreationFailed {
        reason: "network timeout".to_string(),
    };

    assert!(err.to_string().contains("Failed to create repository"));
    assert!(err.to_string().contains("network timeout"));
}

#[test]
fn test_repository_error_push_failed() {
    let err = RepositoryError::PushFailed {
        reason: "authentication failed".to_string(),
    };

    assert!(err.to_string().contains("Failed to push content"));
    assert!(err.to_string().contains("authentication failed"));
}

#[test]
fn test_repository_error_settings_application_failed() {
    let err = RepositoryError::SettingsApplicationFailed {
        setting: "branch protection".to_string(),
        reason: "insufficient permissions".to_string(),
    };

    assert!(err.to_string().contains("Failed to apply settings"));
    assert!(err.to_string().contains("branch protection"));
    assert!(err.to_string().contains("insufficient permissions"));
}

#[test]
fn test_repository_error_operation_timeout() {
    let err = RepositoryError::OperationTimeout { timeout_secs: 30 };

    assert!(err.to_string().contains("timeout"));
    assert!(err.to_string().contains("30 seconds"));
}

// ============================================================================
// ConfigurationError Tests
// ============================================================================

#[test]
fn test_configuration_error_file_not_found() {
    let err = ConfigurationError::FileNotFound {
        path: "/etc/reporoller/config.toml".to_string(),
    };

    assert!(err.to_string().contains("not found"));
    assert!(err.to_string().contains("/etc/reporoller/config.toml"));
}

#[test]
fn test_configuration_error_parse_error() {
    let err = ConfigurationError::ParseError {
        reason: "invalid TOML syntax".to_string(),
    };

    assert!(err.to_string().contains("Failed to parse configuration"));
    assert!(err.to_string().contains("invalid TOML syntax"));
}

#[test]
fn test_configuration_error_invalid_configuration() {
    let err = ConfigurationError::InvalidConfiguration {
        field: "metadata_repository".to_string(),
        reason: "must be a valid repository name".to_string(),
    };

    assert!(err.to_string().contains("Invalid configuration"));
    assert!(err.to_string().contains("metadata_repository"));
    assert!(err.to_string().contains("must be a valid repository name"));
}

#[test]
fn test_configuration_error_override_not_permitted() {
    let err = ConfigurationError::OverrideNotPermitted {
        setting: "security.require_2fa".to_string(),
        reason: "security policy prevents override".to_string(),
    };

    assert!(err.to_string().contains("override not permitted"));
    assert!(err.to_string().contains("security.require_2fa"));
    assert!(err
        .to_string()
        .contains("security policy prevents override"));
}

#[test]
fn test_configuration_error_required_config_missing() {
    let err = ConfigurationError::RequiredConfigMissing {
        key: "github.app_id".to_string(),
    };

    assert!(err.to_string().contains("Required configuration missing"));
    assert!(err.to_string().contains("github.app_id"));
}

#[test]
fn test_configuration_error_hierarchy_resolution_failed() {
    let err = ConfigurationError::HierarchyResolutionFailed {
        reason: "circular dependency detected".to_string(),
    };

    assert!(err
        .to_string()
        .contains("Configuration hierarchy resolution failed"));
    assert!(err.to_string().contains("circular dependency detected"));
}

#[test]
fn test_configuration_error_metadata_repository_not_found() {
    let err = ConfigurationError::MetadataRepositoryNotFound {
        org: "acme-corp".to_string(),
    };

    assert!(err.to_string().contains("Metadata repository not found"));
    assert!(err.to_string().contains("acme-corp"));
}

// ============================================================================
// TemplateError Tests
// ============================================================================

#[test]
fn test_template_error_not_found() {
    let err = TemplateError::TemplateNotFound {
        name: "rust-microservice".to_string(),
    };

    assert!(err.to_string().contains("Template not found"));
    assert!(err.to_string().contains("rust-microservice"));
}

#[test]
fn test_template_error_fetch_failed() {
    let err = TemplateError::FetchFailed {
        reason: "repository is private".to_string(),
    };

    assert!(err.to_string().contains("Failed to fetch template"));
    assert!(err.to_string().contains("repository is private"));
}

#[test]
fn test_template_error_syntax_error() {
    let err = TemplateError::SyntaxError {
        file: "config.toml.template".to_string(),
        reason: "unclosed variable reference".to_string(),
    };

    assert!(err.to_string().contains("Template syntax error"));
    assert!(err.to_string().contains("config.toml.template"));
    assert!(err.to_string().contains("unclosed variable reference"));
}

#[test]
fn test_template_error_substitution_failed() {
    let err = TemplateError::SubstitutionFailed {
        variable: "PROJECT_NAME".to_string(),
        reason: "variable not provided".to_string(),
    };

    assert!(err.to_string().contains("Variable substitution failed"));
    assert!(err.to_string().contains("PROJECT_NAME"));
    assert!(err.to_string().contains("variable not provided"));
}

#[test]
fn test_template_error_required_variable_missing() {
    let err = TemplateError::RequiredVariableMissing {
        variable: "ORGANIZATION".to_string(),
    };

    assert!(err
        .to_string()
        .contains("Required template variable missing"));
    assert!(err.to_string().contains("ORGANIZATION"));
}

#[test]
fn test_template_error_processing_timeout() {
    let err = TemplateError::ProcessingTimeout { timeout_secs: 60 };

    assert!(err.to_string().contains("timeout"));
    assert!(err.to_string().contains("60 seconds"));
}

#[test]
fn test_template_error_security_violation() {
    let err = TemplateError::SecurityViolation {
        reason: "template attempts to access sensitive files".to_string(),
    };

    assert!(err.to_string().contains("Security violation"));
    assert!(err
        .to_string()
        .contains("template attempts to access sensitive files"));
}

#[test]
fn test_template_error_path_traversal_attempt() {
    let err = TemplateError::PathTraversalAttempt {
        path: "../../../etc/passwd".to_string(),
    };

    assert!(err.to_string().contains("Path traversal attempt detected"));
    assert!(err.to_string().contains("../../../etc/passwd"));
}

// ============================================================================
// AuthenticationError Tests
// ============================================================================

#[test]
fn test_authentication_error_invalid_token() {
    let err = AuthenticationError::InvalidToken;

    assert!(err.to_string().contains("Invalid or expired token"));
}

#[test]
fn test_authentication_error_authentication_failed() {
    let err = AuthenticationError::AuthenticationFailed {
        reason: "invalid credentials".to_string(),
    };

    assert!(err.to_string().contains("Authentication failed"));
    assert!(err.to_string().contains("invalid credentials"));
}

#[test]
fn test_authentication_error_insufficient_permissions() {
    let err = AuthenticationError::InsufficientPermissions {
        operation: "create repository".to_string(),
        required: "admin".to_string(),
    };

    assert!(err.to_string().contains("Insufficient permissions"));
    assert!(err.to_string().contains("admin permission required"));
    assert!(err.to_string().contains("create repository"));
}

#[test]
fn test_authentication_error_user_not_found() {
    let err = AuthenticationError::UserNotFound {
        user_id: "user-12345".to_string(),
    };

    assert!(err.to_string().contains("User not found"));
    assert!(err.to_string().contains("user-12345"));
}

#[test]
fn test_authentication_error_organization_access_denied() {
    let err = AuthenticationError::OrganizationAccessDenied {
        org: "secret-corp".to_string(),
    };

    assert!(err.to_string().contains("Organization access denied"));
    assert!(err.to_string().contains("secret-corp"));
}

#[test]
fn test_authentication_error_session_expired() {
    let err = AuthenticationError::SessionExpired;

    assert!(err.to_string().contains("Session expired"));
}

#[test]
fn test_authentication_error_token_refresh_failed() {
    let err = AuthenticationError::TokenRefreshFailed {
        reason: "refresh token invalid".to_string(),
    };

    assert!(err.to_string().contains("Token refresh failed"));
    assert!(err.to_string().contains("refresh token invalid"));
}

// ============================================================================
// GitHubError Tests
// ============================================================================

#[test]
fn test_github_error_api_request_failed() {
    let err = GitHubError::ApiRequestFailed {
        status: 403,
        message: "Forbidden".to_string(),
    };

    assert!(err.to_string().contains("GitHub API request failed"));
    assert!(err.to_string().contains("403"));
    assert!(err.to_string().contains("Forbidden"));
}

#[test]
fn test_github_error_rate_limit_exceeded() {
    let err = GitHubError::RateLimitExceeded {
        reset_at: "2025-10-17T10:30:00Z".to_string(),
    };

    assert!(err.to_string().contains("rate limit exceeded"));
    assert!(err.to_string().contains("2025-10-17T10:30:00Z"));
}

#[test]
fn test_github_error_resource_not_found() {
    let err = GitHubError::ResourceNotFound {
        resource: "organization/repo".to_string(),
    };

    assert!(err.to_string().contains("GitHub resource not found"));
    assert!(err.to_string().contains("organization/repo"));
}

#[test]
fn test_github_error_authentication_failed() {
    let err = GitHubError::AuthenticationFailed {
        reason: "invalid app credentials".to_string(),
    };

    assert!(err.to_string().contains("GitHub authentication failed"));
    assert!(err.to_string().contains("invalid app credentials"));
}

#[test]
fn test_github_error_network_error() {
    let err = GitHubError::NetworkError {
        reason: "connection timeout".to_string(),
    };

    assert!(err
        .to_string()
        .contains("Network error communicating with GitHub"));
    assert!(err.to_string().contains("connection timeout"));
}

#[test]
fn test_github_error_invalid_response() {
    let err = GitHubError::InvalidResponse {
        reason: "unexpected JSON structure".to_string(),
    };

    assert!(err.to_string().contains("Invalid GitHub API response"));
    assert!(err.to_string().contains("unexpected JSON structure"));
}

#[test]
fn test_github_error_app_not_installed() {
    let err = GitHubError::AppNotInstalled {
        org: "my-organization".to_string(),
    };

    assert!(err.to_string().contains("GitHub App not installed"));
    assert!(err.to_string().contains("my-organization"));
}

// ============================================================================
// SystemError Tests
// ============================================================================

#[test]
fn test_system_error_file_system() {
    let err = SystemError::FileSystem {
        operation: "write file".to_string(),
        reason: "disk full".to_string(),
    };

    assert!(err.to_string().contains("File system error"));
    assert!(err.to_string().contains("write file"));
    assert!(err.to_string().contains("disk full"));
}

#[test]
fn test_system_error_git_operation() {
    let err = SystemError::GitOperation {
        operation: "commit".to_string(),
        reason: "invalid signature".to_string(),
    };

    assert!(err.to_string().contains("Git operation failed"));
    assert!(err.to_string().contains("commit"));
    assert!(err.to_string().contains("invalid signature"));
}

#[test]
fn test_system_error_network() {
    let err = SystemError::Network {
        reason: "DNS resolution failed".to_string(),
    };

    assert!(err.to_string().contains("Network error"));
    assert!(err.to_string().contains("DNS resolution failed"));
}

#[test]
fn test_system_error_serialization() {
    let err = SystemError::Serialization {
        reason: "circular reference detected".to_string(),
    };

    assert!(err.to_string().contains("Serialization error"));
    assert!(err.to_string().contains("circular reference detected"));
}

#[test]
fn test_system_error_deserialization() {
    let err = SystemError::Deserialization {
        reason: "invalid UTF-8 sequence".to_string(),
    };

    assert!(err.to_string().contains("Deserialization error"));
    assert!(err.to_string().contains("invalid UTF-8 sequence"));
}

#[test]
fn test_system_error_internal() {
    let err = SystemError::Internal {
        reason: "unexpected state transition".to_string(),
    };

    assert!(err.to_string().contains("Internal error"));
    assert!(err.to_string().contains("unexpected state transition"));
}

#[test]
fn test_system_error_resource_unavailable() {
    let err = SystemError::ResourceUnavailable {
        resource: "temporary directory".to_string(),
    };

    assert!(err.to_string().contains("Resource unavailable"));
    assert!(err.to_string().contains("temporary directory"));
}

// ============================================================================
// Error Result Type Aliases Tests
// ============================================================================

#[test]
fn test_repo_roller_result_type_ok() {
    let result: RepoRollerResult<i32> = Ok(42);
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_repo_roller_result_type_err() {
    let result: RepoRollerResult<i32> = Err(RepoRollerError::Validation(
        ValidationError::empty_field("test"),
    ));
    assert!(result.is_err());
}

#[test]
fn test_repository_result_type_ok() {
    let result: RepositoryResult<String> = Ok("success".to_string());
    assert_eq!(result.unwrap(), "success");
}

#[test]
fn test_configuration_result_type() {
    let result: ConfigurationResult<()> = Err(ConfigurationError::RequiredConfigMissing {
        key: "api_key".to_string(),
    });
    assert!(result.is_err());
}

#[test]
fn test_template_result_type() {
    let result: TemplateResult<Vec<u8>> = Ok(vec![1, 2, 3]);
    assert_eq!(result.unwrap(), vec![1, 2, 3]);
}

#[test]
fn test_authentication_result_type() {
    let result: AuthenticationResult<bool> = Ok(true);
    assert_eq!(result.unwrap(), true);
}

#[test]
fn test_github_result_type() {
    let result: GitHubResult<String> = Err(GitHubError::ResourceNotFound {
        resource: "test".to_string(),
    });
    assert!(result.is_err());
}

#[test]
fn test_system_result_type() {
    let result: SystemResult<()> = Err(SystemError::Internal {
        reason: "test".to_string(),
    });
    assert!(result.is_err());
}

// ============================================================================
// Error Trait Implementation Tests
// ============================================================================

#[test]
fn test_error_trait_implemented() {
    let err: Box<dyn std::error::Error> =
        Box::new(RepoRollerError::System(SystemError::Internal {
            reason: "test".to_string(),
        }));

    assert!(err.to_string().contains("System error"));
}

#[test]
fn test_debug_format_all_error_types() {
    let errors: Vec<RepoRollerError> = vec![
        ValidationError::empty_field("test").into(),
        RepositoryError::NotFound {
            org: "test".to_string(),
            name: "test".to_string(),
        }
        .into(),
        ConfigurationError::FileNotFound {
            path: "test".to_string(),
        }
        .into(),
        TemplateError::TemplateNotFound {
            name: "test".to_string(),
        }
        .into(),
        AuthenticationError::InvalidToken.into(),
        GitHubError::ResourceNotFound {
            resource: "test".to_string(),
        }
        .into(),
        SystemError::Internal {
            reason: "test".to_string(),
        }
        .into(),
    ];

    for err in errors {
        let debug_output = format!("{:?}", err);
        assert!(!debug_output.is_empty());
    }
}
