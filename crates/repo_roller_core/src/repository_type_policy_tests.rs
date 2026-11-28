//! Tests for server-side enforcement of the template `[repository_type]`
//! policy (`fixed` / `preferable`) -- GitHub issue #288.
//!
//! Target: `resolve_repository_type_policy(request_override, template)`.
//!
//! Design decisions encoded here:
//! - `fixed` + ANY override (even one equal to the template type) is REJECTED
//!   with `ConfigurationError::InvalidConfiguration { field: "repository_type", .. }`.
//! - `preferable` + override => the override wins.
//! - No template, or a template without `[repository_type]` => `Ok(None)`;
//!   an override is not enforced against anything.
//!
//! Note: `create_repository_inner` takes real GitHub clients via
//! `setup_github_clients` (no injectable mock seam), so an end-to-end
//! "fixed+override errors before any mutation" test is not feasible here;
//! ordering (call right after `load_creation_config`, before
//! `validate_creation_mode_capability` / any mutating call) is verified by
//! review.

use super::*;
use config_manager::TemplateConfig;
use proptest::prelude::*;

fn template_with(spec: Option<(&str, &str)>) -> TemplateConfig {
    let mut value = serde_json::json!({
        "template": {
            "name": "tpl",
            "description": "d",
            "author": "a",
            "tags": ["t"]
        }
    });
    if let Some((ty, policy)) = spec {
        value["repository_type"] = serde_json::json!({ "type": ty, "policy": policy });
    }
    serde_json::from_value(value).expect("valid template config")
}

fn assert_repository_type_rejected(
    result: RepoRollerResult<Option<config_manager::RepositoryTypeName>>,
) {
    match result {
        Err(RepoRollerError::Configuration(ConfigurationError::InvalidConfiguration {
            field,
            ..
        })) => assert_eq!(field, "repository_type"),
        other => panic!("expected InvalidConfiguration on repository_type, got {other:?}"),
    }
}

fn name(result: RepoRollerResult<Option<config_manager::RepositoryTypeName>>) -> Option<String> {
    result.expect("expected Ok").map(|n| n.as_str().to_string())
}

// ---- Tier 1: specification matrix ------------------------------------------

#[test]
fn fixed_policy_with_different_override_is_rejected() {
    let tpl = template_with(Some(("service", "fixed")));
    assert_repository_type_rejected(resolve_repository_type_policy(Some("library"), Some(&tpl)));
}

#[test]
fn fixed_policy_with_override_equal_to_template_type_is_still_rejected() {
    let tpl = template_with(Some(("service", "fixed")));
    assert_repository_type_rejected(resolve_repository_type_policy(Some("service"), Some(&tpl)));
}

#[test]
fn fixed_policy_without_override_yields_template_type() {
    let tpl = template_with(Some(("service", "fixed")));
    assert_eq!(
        name(resolve_repository_type_policy(None, Some(&tpl))),
        Some("service".to_string())
    );
}

#[test]
fn preferable_policy_with_override_yields_override() {
    let tpl = template_with(Some(("library", "preferable")));
    assert_eq!(
        name(resolve_repository_type_policy(
            Some("documentation"),
            Some(&tpl)
        )),
        Some("documentation".to_string())
    );
}

#[test]
fn preferable_policy_without_override_yields_template_type() {
    let tpl = template_with(Some(("library", "preferable")));
    assert_eq!(
        name(resolve_repository_type_policy(None, Some(&tpl))),
        Some("library".to_string())
    );
}

#[test]
fn no_template_without_override_yields_none() {
    assert_eq!(name(resolve_repository_type_policy(None, None)), None);
}

#[test]
fn no_template_with_override_yields_none_not_enforced() {
    assert_eq!(
        name(resolve_repository_type_policy(Some("library"), None)),
        None
    );
}

#[test]
fn no_template_with_invalid_override_name_is_rejected() {
    assert!(resolve_repository_type_policy(Some("Not Valid!"), None).is_err());
}

#[test]
fn template_without_spec_with_invalid_override_name_is_rejected() {
    let tpl = template_with(None);
    assert!(resolve_repository_type_policy(Some("Not Valid!"), Some(&tpl)).is_err());
}

#[test]
fn template_without_repository_type_section_yields_none_for_no_override() {
    let tpl = template_with(None);
    assert_eq!(name(resolve_repository_type_policy(None, Some(&tpl))), None);
}

#[test]
fn template_without_repository_type_section_yields_none_even_with_override() {
    let tpl = template_with(None);
    assert_eq!(
        name(resolve_repository_type_policy(Some("library"), Some(&tpl))),
        None
    );
}

// ---- Tier 2: adversarial ---------------------------------------------------

#[test]
fn preferable_override_does_not_return_the_template_type() {
    let tpl = template_with(Some(("library", "preferable")));
    assert_ne!(
        name(resolve_repository_type_policy(Some("service"), Some(&tpl))),
        Some("library".to_string())
    );
}

#[test]
fn fixed_rejection_is_not_a_template_or_validation_error_variant() {
    let tpl = template_with(Some(("service", "fixed")));
    let result = resolve_repository_type_policy(Some("other"), Some(&tpl));
    assert!(matches!(result, Err(RepoRollerError::Configuration(_))));
}

#[test]
fn preferable_with_invalid_override_name_is_rejected() {
    let tpl = template_with(Some(("library", "preferable")));
    assert!(resolve_repository_type_policy(Some("Bad Name!"), Some(&tpl)).is_err());
}

#[test]
fn preferable_with_empty_override_is_rejected() {
    let tpl = template_with(Some(("library", "preferable")));
    assert!(resolve_repository_type_policy(Some(""), Some(&tpl)).is_err());
}

#[test]
fn fixed_policy_error_message_names_the_template_type() {
    let tpl = template_with(Some(("service", "fixed")));
    let err = resolve_repository_type_policy(Some("library"), Some(&tpl)).unwrap_err();
    assert!(err.to_string().contains("service"), "got: {err}");
}

// ---- Tier 3: properties ----------------------------------------------------

fn type_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,20}"
}

proptest! {
    #[test]
    fn prop_fixed_policy_rejects_every_override(tpl_ty in type_name_strategy(), ov in type_name_strategy()) {
        let tpl = template_with(Some((&tpl_ty, "fixed")));
        let rejected = matches!(
            resolve_repository_type_policy(Some(&ov), Some(&tpl)),
            Err(RepoRollerError::Configuration(ConfigurationError::InvalidConfiguration { .. }))
        );
        prop_assert!(rejected);
    }

    #[test]
    fn prop_fixed_policy_without_override_returns_template_type(tpl_ty in type_name_strategy()) {
        let tpl = template_with(Some((&tpl_ty, "fixed")));
        let got = resolve_repository_type_policy(None, Some(&tpl)).unwrap();
        prop_assert_eq!(got.map(|n| n.as_str().to_string()), Some(tpl_ty));
    }

    #[test]
    fn prop_preferable_policy_returns_override_when_given(tpl_ty in type_name_strategy(), ov in type_name_strategy()) {
        let tpl = template_with(Some((&tpl_ty, "preferable")));
        let got = resolve_repository_type_policy(Some(&ov), Some(&tpl)).unwrap();
        prop_assert_eq!(got.map(|n| n.as_str().to_string()), Some(ov));
    }

    #[test]
    fn prop_no_template_never_errors_and_yields_none(ov in proptest::option::of(type_name_strategy())) {
        let got = resolve_repository_type_policy(ov.as_deref(), None).unwrap();
        prop_assert!(got.is_none());
    }
}
