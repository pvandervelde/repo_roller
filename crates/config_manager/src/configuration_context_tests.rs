//! Tests for configuration context.

use super::*;

// ============================================================================
// ConfigurationContext Creation Tests (Task 5.3)
// ============================================================================

/// Verify ConfigurationContext can be created with required fields only.
#[test]
fn test_context_creation_with_required_fields() {
    let context = ConfigurationContext::new("my-org", "rust-service");

    assert_eq!(context.organization(), "my-org");
    assert_eq!(context.template(), "rust-service");
    assert_eq!(context.team(), None);
    assert_eq!(context.repository_type(), None);
}

/// Verify ConfigurationContext accepts String, &str, and String types.
#[test]
fn test_context_creation_with_different_string_types() {
    let org_string = String::from("my-org");
    let template_str = "rust-service";

    let context = ConfigurationContext::new(org_string, template_str);

    assert_eq!(context.organization(), "my-org");
    assert_eq!(context.template(), "rust-service");
}

/// Verify with_team builder method adds team information.
#[test]
fn test_context_with_team() {
    let context = ConfigurationContext::new("my-org", "rust-service").with_team("backend-team");

    assert_eq!(context.organization(), "my-org");
    assert_eq!(context.template(), "rust-service");
    assert_eq!(context.team(), Some("backend-team"));
    assert_eq!(context.repository_type(), None);
}

/// Verify with_repository_type builder method adds repository type.
#[test]
fn test_context_with_repository_type() {
    let context =
        ConfigurationContext::new("my-org", "rust-service").with_repository_type("library");

    assert_eq!(context.organization(), "my-org");
    assert_eq!(context.template(), "rust-service");
    assert_eq!(context.team(), None);
    assert_eq!(context.repository_type(), Some("library"));
}

/// Verify builder pattern supports chaining both team and repository type.
#[test]
fn test_context_with_team_and_repository_type() {
    let context = ConfigurationContext::new("my-org", "rust-service")
        .with_team("backend-team")
        .with_repository_type("library");

    assert_eq!(context.organization(), "my-org");
    assert_eq!(context.template(), "rust-service");
    assert_eq!(context.team(), Some("backend-team"));
    assert_eq!(context.repository_type(), Some("library"));
}

/// Verify builder pattern supports chaining in any order.
#[test]
fn test_context_builder_chaining_order() {
    let context1 = ConfigurationContext::new("my-org", "rust-service")
        .with_team("backend-team")
        .with_repository_type("library");

    let context2 = ConfigurationContext::new("my-org", "rust-service")
        .with_repository_type("library")
        .with_team("backend-team");

    assert_eq!(context1.team(), context2.team());
    assert_eq!(context1.repository_type(), context2.repository_type());
}

/// Verify created_at timestamp is set correctly.
#[test]
fn test_context_created_at_timestamp() {
    let before = Utc::now();
    let context = ConfigurationContext::new("my-org", "rust-service");
    let after = Utc::now();

    let created_at = context.created_at();

    assert!(
        created_at >= before && created_at <= after,
        "created_at should be between before and after timestamps"
    );
}

/// Verify context implements Debug trait.
#[test]
fn test_context_implements_debug() {
    let context = ConfigurationContext::new("my-org", "rust-service")
        .with_team("backend-team")
        .with_repository_type("library");

    let debug_str = format!("{:?}", context);

    assert!(debug_str.contains("ConfigurationContext"));
    assert!(debug_str.contains("my-org"));
    assert!(debug_str.contains("rust-service"));
    assert!(debug_str.contains("backend-team"));
    assert!(debug_str.contains("library"));
}

/// Verify context implements Clone trait.
#[test]
fn test_context_is_cloneable() {
    let context1 = ConfigurationContext::new("my-org", "rust-service")
        .with_team("backend-team")
        .with_repository_type("library");

    let context2 = context1.clone();

    assert_eq!(context1.organization(), context2.organization());
    assert_eq!(context1.template(), context2.template());
    assert_eq!(context1.team(), context2.team());
    assert_eq!(context1.repository_type(), context2.repository_type());
    assert_eq!(context1.created_at(), context2.created_at());
}

/// Verify context implements PartialEq trait.
#[test]
fn test_context_equality() {
    let context1 = ConfigurationContext::new("my-org", "rust-service")
        .with_team("backend-team")
        .with_repository_type("library");

    let context2 = ConfigurationContext::new("my-org", "rust-service")
        .with_team("backend-team")
        .with_repository_type("library");

    // Note: These won't be equal due to different created_at timestamps
    assert_ne!(
        context1, context2,
        "Different timestamps make contexts unequal"
    );

    // But clones should be equal
    let context3 = context1.clone();
    assert_eq!(context1, context3, "Cloned contexts should be equal");
}

/// Verify getter methods return correct references.
#[test]
fn test_context_getters_return_references() {
    let context = ConfigurationContext::new("my-org", "rust-service")
        .with_team("backend-team")
        .with_repository_type("library");

    // Getters should return &str, not String
    let _org: &str = context.organization();
    let _template: &str = context.template();
    let _team: Option<&str> = context.team();
    let _repo_type: Option<&str> = context.repository_type();

    assert_eq!(context.organization(), "my-org");
    assert_eq!(context.template(), "rust-service");
    assert_eq!(context.team(), Some("backend-team"));
    assert_eq!(context.repository_type(), Some("library"));
}

/// Verify context can be created with empty strings (edge case).
#[test]
fn test_context_with_empty_strings() {
    let context = ConfigurationContext::new("", "");

    assert_eq!(context.organization(), "");
    assert_eq!(context.template(), "");
}

/// Verify context with long strings.
#[test]
fn test_context_with_long_strings() {
    let long_org = "a".repeat(1000);
    let long_template = "b".repeat(1000);
    let long_team = "c".repeat(1000);

    let context = ConfigurationContext::new(&long_org, &long_template).with_team(&long_team);

    assert_eq!(context.organization().len(), 1000);
    assert_eq!(context.template().len(), 1000);
    assert_eq!(context.team().map(|s| s.len()), Some(1000));
}
