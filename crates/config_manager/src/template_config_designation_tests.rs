//! Tests for `TemplateDesignation` and the `TemplateConfig.designation` field.
//!
//! Covers RCA-004/005/006 (fork/mirror template designation) and the
//! "Template Discovery" acceptance criteria in `.llm/tasks.md`: a template
//! self-declares at most one special role via `designation`, independent of
//! (but mutually exclusive with a `Fixed` policy on) `repository_type`.
//!
//! See docs/spec/interfaces/fork-mirror-operations.md §1.

use super::*;

// ============================================================================
// Test Helpers
// ============================================================================

fn minimal_metadata() -> TemplateMetadata {
    TemplateMetadata {
        name: "test-template".to_string(),
        description: "Test template".to_string(),
        author: "Test Author".to_string(),
        tags: vec![],
    }
}

/// Builds a minimal valid `TemplateConfig` with the given designation.
fn template_with_designation(designation: Option<TemplateDesignation>) -> TemplateConfig {
    TemplateConfig {
        template: minimal_metadata(),
        repository_type: None,
        variables: None,
        repository: None,
        pull_requests: None,
        branch_protection: None,
        labels: None,
        webhooks: None,
        environments: None,
        github_apps: None,
        rulesets: None,
        default_visibility: None,
        templating: None,
        notifications: None,
        permissions: None,
        teams: None,
        collaborators: None,
        naming_rules: None,
        designation,
        creation_capabilities: None,
    }
}

// ============================================================================
// Tier 1: Specification tests — TemplateDesignation serde round trip
// ============================================================================

#[test]
fn test_template_designation_serde_roundtrip_fork() {
    let value = TemplateDesignation::Fork;
    let json = serde_json::to_string(&value).expect("serialize");
    let roundtripped: TemplateDesignation = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(roundtripped, TemplateDesignation::Fork);
}

#[test]
fn test_template_designation_serde_roundtrip_source_history_mirror() {
    let value = TemplateDesignation::SourceHistoryMirror;
    let json = serde_json::to_string(&value).expect("serialize");
    let roundtripped: TemplateDesignation = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(roundtripped, TemplateDesignation::SourceHistoryMirror);
}

#[test]
fn test_template_designation_serde_roundtrip_source_only_mirror() {
    let value = TemplateDesignation::SourceOnlyMirror;
    let json = serde_json::to_string(&value).expect("serialize");
    let roundtripped: TemplateDesignation = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(roundtripped, TemplateDesignation::SourceOnlyMirror);
}

/// Stub-killing: a hardcoded serializer that always emits "fork" would pass
/// the Fork round-trip but fail here for the other two variants' exact wire
/// format (snake_case, per docs/spec/interfaces/fork-mirror-operations.md §1).
#[test]
fn test_template_designation_toml_wire_format_is_snake_case() {
    assert_eq!(
        serde_json::to_string(&TemplateDesignation::Fork).unwrap(),
        "\"fork\""
    );
    assert_eq!(
        serde_json::to_string(&TemplateDesignation::SourceHistoryMirror).unwrap(),
        "\"source_history_mirror\""
    );
    assert_eq!(
        serde_json::to_string(&TemplateDesignation::SourceOnlyMirror).unwrap(),
        "\"source_only_mirror\""
    );
}

#[test]
fn test_template_designation_deserializes_from_toml_scalar() {
    #[derive(serde::Deserialize)]
    struct Wrapper {
        designation: TemplateDesignation,
    }

    let toml = "designation = \"source_only_mirror\"\n";
    let parsed: Wrapper = toml::from_str(toml).expect("parse");
    assert_eq!(parsed.designation, TemplateDesignation::SourceOnlyMirror);
}

// ============================================================================
// Tier 1: TemplateConfig.designation field
// ============================================================================

#[test]
fn test_template_config_designation_field_absent_parses_as_none() {
    let toml = r#"
        [template]
        name = "no-designation"
        description = "desc"
        author = "author"
        tags = []
    "#;

    let config: TemplateConfig = toml::from_str(toml).expect("parse");
    assert_eq!(config.designation, None);
}

#[test]
fn test_template_config_designation_field_parses_fork_from_toml() {
    let toml = r#"
        designation = "fork"

        [template]
        name = "fork-settings"
        description = "Settings applied to every fork"
        author = "Platform Team"
        tags = []
    "#;

    let config: TemplateConfig = toml::from_str(toml).expect("parse");
    assert_eq!(config.designation, Some(TemplateDesignation::Fork));
}

#[test]
fn test_template_config_designation_field_roundtrips_through_json() {
    let config = template_with_designation(Some(TemplateDesignation::SourceHistoryMirror));
    let json = serde_json::to_string(&config).expect("serialize");
    let roundtripped: TemplateConfig = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(
        roundtripped.designation,
        Some(TemplateDesignation::SourceHistoryMirror)
    );
}

/// Mirrors the `skip_serializing_if = "Option::is_none"` convention used by
/// every other optional field on `TemplateConfig` (see `repository_type`,
/// `variables`, etc.) — `designation` must not appear in serialized output
/// when absent, so existing templates without a designation are unaffected.
#[test]
fn test_template_config_designation_omitted_from_output_when_none() {
    let config = template_with_designation(None);
    let json = serde_json::to_string(&config).expect("serialize");
    assert!(
        !json.contains("designation"),
        "designation key should be omitted when None, got: {json}"
    );
}

#[test]
fn test_template_config_designation_present_in_output_when_some() {
    let config = template_with_designation(Some(TemplateDesignation::Fork));
    let json = serde_json::to_string(&config).expect("serialize");
    assert!(
        json.contains("\"designation\":\"fork\""),
        "designation key should be present when Some, got: {json}"
    );
}

// ============================================================================
// Tier 2: Adversarial — variant discrimination and derives
// ============================================================================

/// Stub-killing: an implementation that collapses all three designations
/// into a single internal representation (e.g. `bool`) would fail this
/// pairwise-inequality check.
#[test]
fn test_template_designation_variants_are_pairwise_distinct() {
    assert_ne!(
        TemplateDesignation::Fork,
        TemplateDesignation::SourceHistoryMirror
    );
    assert_ne!(
        TemplateDesignation::Fork,
        TemplateDesignation::SourceOnlyMirror
    );
    assert_ne!(
        TemplateDesignation::SourceHistoryMirror,
        TemplateDesignation::SourceOnlyMirror
    );
}

#[test]
fn test_template_designation_is_copy() {
    let value = TemplateDesignation::Fork;
    let copied = value;
    // Using `value` after assigning `copied` only compiles if `Copy` holds.
    assert_eq!(value, copied);
}

#[test]
fn test_template_config_clone_preserves_designation() {
    let config = template_with_designation(Some(TemplateDesignation::SourceOnlyMirror));
    let cloned = config.clone();
    assert_eq!(
        cloned.designation,
        Some(TemplateDesignation::SourceOnlyMirror)
    );
}
