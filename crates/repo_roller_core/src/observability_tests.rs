//! Unit and adversarial tests for `observability`.

use super::*;

// ---------------------------------------------------------------------
// Tier 1: Specification tests -- span name / attribute-key contract.
// ---------------------------------------------------------------------

#[test]
fn test_span_repository_creation_name_is_repository_creation() {
    assert_eq!(SPAN_REPOSITORY_CREATION, "repository_creation");
}

#[test]
fn test_span_authentication_name_is_authentication() {
    assert_eq!(SPAN_AUTHENTICATION, "authentication");
}

#[test]
fn test_span_configuration_resolution_name_is_configuration_resolution() {
    assert_eq!(SPAN_CONFIGURATION_RESOLUTION, "configuration_resolution");
}

#[test]
fn test_span_template_processing_name_is_template_processing() {
    assert_eq!(SPAN_TEMPLATE_PROCESSING, "template_processing");
}

#[test]
fn test_span_github_repository_creation_name_is_github_repository_creation() {
    assert_eq!(SPAN_GITHUB_REPOSITORY_CREATION, "github_repository_creation");
}

#[test]
fn test_all_span_name_constants_are_pairwise_distinct() {
    let names = [
        SPAN_REPOSITORY_CREATION,
        SPAN_AUTHENTICATION,
        SPAN_CONFIGURATION_RESOLUTION,
        SPAN_TEMPLATE_PROCESSING,
        SPAN_GITHUB_REPOSITORY_CREATION,
    ];
    for i in 0..names.len() {
        for j in 0..names.len() {
            if i != j {
                assert_ne!(
                    names[i], names[j],
                    "span name constants at indices {i} and {j} collide: {:?}",
                    names[i]
                );
            }
        }
    }
}

#[test]
fn test_business_context_attributes_includes_organization_and_requestor_with_template() {
    let attrs = business_context_attributes("my-org", Some("rust-library"), "alice");
    let keys: Vec<&str> = attrs.iter().map(|(k, _)| *k).collect();
    assert_eq!(keys, vec![ATTR_ORGANIZATION, ATTR_TEMPLATE, ATTR_REQUESTOR]);
}

#[test]
fn test_business_context_attributes_omits_template_key_when_none() {
    let attrs = business_context_attributes("my-org", None, "alice");
    let keys: Vec<&str> = attrs.iter().map(|(k, _)| *k).collect();
    assert_eq!(keys, vec![ATTR_ORGANIZATION, ATTR_REQUESTOR]);
}

#[test]
fn test_business_context_attributes_organization_value_matches_sanitized_input() {
    let attrs = business_context_attributes("my-org", None, "alice");
    let org_value = attrs
        .iter()
        .find(|(k, _)| *k == ATTR_ORGANIZATION)
        .map(|(_, v)| v.clone())
        .expect("organization attribute must be present");
    assert_eq!(
        org_value,
        github_client::span_attributes::sanitize_span_attribute_value("my-org")
    );
}

// ---------------------------------------------------------------------
// Tier 2: Adversarial / boundary / stub-killing tests.
// ---------------------------------------------------------------------

#[test]
fn test_business_context_attributes_never_panics_on_empty_strings() {
    let _ = business_context_attributes("", None, "");
}

#[test]
fn test_business_context_attributes_redacts_secret_shaped_organization() {
    let secret_org = format!("ghp_{}", "a".repeat(24));
    let attrs = business_context_attributes(&secret_org, None, "alice");
    let org_value = attrs
        .iter()
        .find(|(k, _)| *k == ATTR_ORGANIZATION)
        .map(|(_, v)| v.clone())
        .expect("organization attribute must be present");
    assert_eq!(
        org_value,
        github_client::span_attributes::SPAN_ATTRIBUTE_REDACTED_MARKER
    );
}

#[test]
fn test_business_context_attributes_redacts_secret_shaped_requestor() {
    let secret_requestor = format!("ghp_{}", "b".repeat(24));
    let attrs = business_context_attributes("my-org", None, &secret_requestor);
    let requestor_value = attrs
        .iter()
        .find(|(k, _)| *k == ATTR_REQUESTOR)
        .map(|(_, v)| v.clone())
        .expect("requestor attribute must be present");
    assert_eq!(
        requestor_value,
        github_client::span_attributes::SPAN_ATTRIBUTE_REDACTED_MARKER
    );
}

#[test]
fn test_business_context_attributes_bounds_pathologically_long_template_name() {
    let huge_template: String = std::iter::repeat('a').take(10_000).collect();
    let attrs = business_context_attributes("my-org", Some(&huge_template), "alice");
    let template_value = attrs
        .iter()
        .find(|(k, _)| *k == ATTR_TEMPLATE)
        .map(|(_, v)| v.clone())
        .expect("template attribute must be present when Some");
    assert!(
        template_value.chars().count()
            <= github_client::span_attributes::MAX_SPAN_ATTRIBUTE_LEN
    );
}

#[test]
fn test_business_context_attributes_distinct_organizations_produce_distinct_values() {
    // Stub-killing: guards against a hardcoded-return implementation that
    // ignores its `organization` parameter.
    let a = business_context_attributes("org-alpha", None, "alice");
    let b = business_context_attributes("org-beta", None, "alice");
    let a_org = a.iter().find(|(k, _)| *k == ATTR_ORGANIZATION).unwrap().1.clone();
    let b_org = b.iter().find(|(k, _)| *k == ATTR_ORGANIZATION).unwrap().1.clone();
    assert_ne!(a_org, b_org);
}

#[test]
fn test_business_context_attributes_distinct_requestors_produce_distinct_values() {
    let a = business_context_attributes("my-org", None, "alice");
    let b = business_context_attributes("my-org", None, "bob");
    let a_req = a.iter().find(|(k, _)| *k == ATTR_REQUESTOR).unwrap().1.clone();
    let b_req = b.iter().find(|(k, _)| *k == ATTR_REQUESTOR).unwrap().1.clone();
    assert_ne!(a_req, b_req);
}

#[test]
fn test_business_context_attributes_returns_exactly_two_entries_without_template() {
    // Stub-killing: guards against an implementation that always includes
    // all three keys regardless of `template`.
    let attrs = business_context_attributes("my-org", None, "alice");
    assert_eq!(attrs.len(), 2);
}

#[test]
fn test_business_context_attributes_returns_exactly_three_entries_with_template() {
    let attrs = business_context_attributes("my-org", Some("basic"), "alice");
    assert_eq!(attrs.len(), 3);
}
