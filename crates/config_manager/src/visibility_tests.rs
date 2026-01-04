//! Tests for visibility types.
//!
//! These tests verify the basic functionality of visibility policy types
//! defined in the visibility module.

use super::*;

#[test]
fn test_repository_visibility_as_str() {
    assert_eq!(RepositoryVisibility::Public.as_str(), "public");
    assert_eq!(RepositoryVisibility::Private.as_str(), "private");
    assert_eq!(RepositoryVisibility::Internal.as_str(), "internal");
}

#[test]
fn test_repository_visibility_is_private() {
    assert!(!RepositoryVisibility::Public.is_private());
    assert!(RepositoryVisibility::Private.is_private());
    assert!(RepositoryVisibility::Internal.is_private());
}

#[test]
fn test_repository_visibility_serialization() {
    use serde_json;

    let public = RepositoryVisibility::Public;
    assert_eq!(serde_json::to_string(&public).unwrap(), "\"public\"");

    let private = RepositoryVisibility::Private;
    assert_eq!(serde_json::to_string(&private).unwrap(), "\"private\"");

    let internal = RepositoryVisibility::Internal;
    assert_eq!(serde_json::to_string(&internal).unwrap(), "\"internal\"");
}

#[test]
fn test_repository_visibility_deserialization() {
    use serde_json;

    let public: RepositoryVisibility = serde_json::from_str("\"public\"").unwrap();
    assert_eq!(public, RepositoryVisibility::Public);

    let private: RepositoryVisibility = serde_json::from_str("\"private\"").unwrap();
    assert_eq!(private, RepositoryVisibility::Private);

    let internal: RepositoryVisibility = serde_json::from_str("\"internal\"").unwrap();
    assert_eq!(internal, RepositoryVisibility::Internal);
}

#[test]
fn test_visibility_policy_required_allows() {
    let policy = VisibilityPolicy::Required(RepositoryVisibility::Private);

    assert!(policy.allows(RepositoryVisibility::Private));
    assert!(!policy.allows(RepositoryVisibility::Public));
    assert!(!policy.allows(RepositoryVisibility::Internal));
}

#[test]
fn test_visibility_policy_restricted_allows() {
    let policy = VisibilityPolicy::Restricted(vec![RepositoryVisibility::Public]);

    assert!(!policy.allows(RepositoryVisibility::Public));
    assert!(policy.allows(RepositoryVisibility::Private));
    assert!(policy.allows(RepositoryVisibility::Internal));
}

#[test]
fn test_visibility_policy_restricted_multiple() {
    let policy = VisibilityPolicy::Restricted(vec![
        RepositoryVisibility::Public,
        RepositoryVisibility::Internal,
    ]);

    assert!(!policy.allows(RepositoryVisibility::Public));
    assert!(policy.allows(RepositoryVisibility::Private));
    assert!(!policy.allows(RepositoryVisibility::Internal));
}

#[test]
fn test_visibility_policy_unrestricted_allows() {
    let policy = VisibilityPolicy::Unrestricted;

    assert!(policy.allows(RepositoryVisibility::Public));
    assert!(policy.allows(RepositoryVisibility::Private));
    assert!(policy.allows(RepositoryVisibility::Internal));
}

#[test]
fn test_visibility_policy_required_visibility() {
    let required = VisibilityPolicy::Required(RepositoryVisibility::Private);
    assert_eq!(
        required.required_visibility(),
        Some(RepositoryVisibility::Private)
    );

    let restricted = VisibilityPolicy::Restricted(vec![RepositoryVisibility::Public]);
    assert_eq!(restricted.required_visibility(), None);

    let unrestricted = VisibilityPolicy::Unrestricted;
    assert_eq!(unrestricted.required_visibility(), None);
}

#[test]
fn test_policy_constraint_equality() {
    assert_eq!(
        PolicyConstraint::OrganizationRequired,
        PolicyConstraint::OrganizationRequired
    );
    assert_ne!(
        PolicyConstraint::OrganizationRequired,
        PolicyConstraint::OrganizationRestricted
    );
}

#[test]
fn test_visibility_error_display() {
    let error = VisibilityError::PolicyNotFound {
        organization: "test-org".to_string(),
    };
    assert!(error
        .to_string()
        .contains("No visibility policy configured"));

    let error = VisibilityError::PolicyViolation {
        requested: RepositoryVisibility::Public,
        policy: "Required(Private)".to_string(),
    };
    assert!(error.to_string().contains("violates organization policy"));

    let error = VisibilityError::GitHubConstraint {
        requested: RepositoryVisibility::Internal,
        reason: "Requires GitHub Enterprise".to_string(),
    };
    assert!(error.to_string().contains("not available"));

    let error = VisibilityError::EnvironmentDetectionFailed {
        organization: "test-org".to_string(),
        reason: "API error".to_string(),
    };
    assert!(error
        .to_string()
        .contains("Failed to detect environment for organization"));
    assert!(error.to_string().contains("test-org"));
    assert!(error.to_string().contains("API error"));
}
