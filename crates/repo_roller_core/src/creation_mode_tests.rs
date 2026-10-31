//! Tests for `CreationMode`, `RepositoryCreationRequest.creation_mode`, and
//! the builder's `.creation_mode()` setter + build()-time validation.
//!
//! See docs/spec/interfaces/fork-mirror-operations.md §4 and RCA-004/005/006.

use super::*;
use crate::SourceRepositoryReference;

fn source() -> SourceRepositoryReference {
    SourceRepositoryReference {
        owner: OrganizationName::new("upstream-org").unwrap(),
        name: RepositoryName::new("upstream-repo").unwrap(),
    }
}

// ============================================================================
// Tier 1: Specification tests — CreationMode shape
// ============================================================================

#[test]
fn test_creation_mode_default_is_standard() {
    assert_eq!(CreationMode::default(), CreationMode::Standard);
}

#[test]
fn test_creation_mode_fork_serde_roundtrip() {
    let mode = CreationMode::Fork { source: source() };
    let json = serde_json::to_string(&mode).expect("serialize");
    let roundtripped: CreationMode = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(mode, roundtripped);
}

#[test]
fn test_creation_mode_mirror_preserve_history_true_serde_roundtrip() {
    let mode = CreationMode::Mirror {
        source: source(),
        preserve_history: true,
    };
    let json = serde_json::to_string(&mode).expect("serialize");
    let roundtripped: CreationMode = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(mode, roundtripped);
}

#[test]
fn test_creation_mode_mirror_preserve_history_false_serde_roundtrip() {
    let mode = CreationMode::Mirror {
        source: source(),
        preserve_history: false,
    };
    let json = serde_json::to_string(&mode).expect("serialize");
    let roundtripped: CreationMode = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(mode, roundtripped);
}

/// Stub-killing: a buggy `Mirror` implementation that ignores
/// `preserve_history` (e.g. always `true`) would make these two modes
/// compare equal; they must not.
#[test]
fn test_creation_mode_mirror_preserve_history_true_and_false_are_distinct() {
    let history = CreationMode::Mirror {
        source: source(),
        preserve_history: true,
    };
    let source_only = CreationMode::Mirror {
        source: source(),
        preserve_history: false,
    };
    assert_ne!(history, source_only);
}

#[test]
fn test_creation_mode_standard_fork_and_mirror_are_pairwise_distinct() {
    let standard = CreationMode::Standard;
    let fork = CreationMode::Fork { source: source() };
    let mirror = CreationMode::Mirror {
        source: source(),
        preserve_history: true,
    };

    assert_ne!(standard, fork);
    assert_ne!(standard, mirror);
    assert_ne!(fork, mirror);
}

#[test]
fn test_creation_mode_wire_tag_is_type_snake_case() {
    let json = serde_json::to_string(&CreationMode::Standard).unwrap();
    assert!(json.contains("\"type\":\"standard\""), "got: {json}");

    let json = serde_json::to_string(&CreationMode::Fork { source: source() }).unwrap();
    assert!(json.contains("\"type\":\"fork\""), "got: {json}");

    let json = serde_json::to_string(&CreationMode::Mirror {
        source: source(),
        preserve_history: true,
    })
    .unwrap();
    assert!(json.contains("\"type\":\"mirror\""), "got: {json}");
}

// ============================================================================
// Tier 1: RepositoryCreationRequest / builder wiring
// ============================================================================

#[test]
fn test_builder_defaults_creation_mode_to_standard() {
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("my-repo").unwrap(),
        OrganizationName::new("my-org").unwrap(),
    )
    .content_strategy(ContentStrategy::Empty)
    .build();

    assert_eq!(request.creation_mode, CreationMode::Standard);
}

#[test]
fn test_builder_creation_mode_setter_applies_fork_mode() {
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("my-repo").unwrap(),
        OrganizationName::new("my-org").unwrap(),
    )
    .creation_mode(CreationMode::Fork { source: source() })
    .build();

    assert_eq!(
        request.creation_mode,
        CreationMode::Fork { source: source() }
    );
}

// ============================================================================
// Tier 2: Adversarial — build()-time content_strategy/creation_mode validation
// ============================================================================

/// Regression guard: the pre-existing rule ("`ContentStrategy::Template`
/// requires a template") must still apply unchanged for `CreationMode::Standard`.
#[test]
#[should_panic(expected = "ContentStrategy::Template requires template to be set")]
fn test_builder_standard_mode_without_template_still_panics_on_default_content_strategy() {
    RepositoryCreationRequestBuilder::new(
        RepositoryName::new("my-repo").unwrap(),
        OrganizationName::new("my-org").unwrap(),
    )
    .build();
}

/// Stub-killing / key regression test: a lazy implementation that keeps the
/// old "Template strategy requires a template" rule unconditional would
/// panic here too — but Fork must bypass `ContentStrategy` entirely and
/// succeed even with no template and no explicit `content_strategy` call
/// (AC: "works correctly when no template is configured").
#[test]
fn test_builder_fork_mode_without_template_and_without_content_strategy_call_does_not_panic() {
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("my-repo").unwrap(),
        OrganizationName::new("my-org").unwrap(),
    )
    .creation_mode(CreationMode::Fork { source: source() })
    .build();

    assert_eq!(
        request.creation_mode,
        CreationMode::Fork { source: source() }
    );
    assert_eq!(request.template, None);
}

#[test]
fn test_builder_mirror_history_mode_without_template_does_not_panic() {
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("my-repo").unwrap(),
        OrganizationName::new("my-org").unwrap(),
    )
    .creation_mode(CreationMode::Mirror {
        source: source(),
        preserve_history: true,
    })
    .build();

    assert_eq!(request.template, None);
}

#[test]
fn test_builder_mirror_source_only_mode_without_template_does_not_panic() {
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("my-repo").unwrap(),
        OrganizationName::new("my-org").unwrap(),
    )
    .creation_mode(CreationMode::Mirror {
        source: source(),
        preserve_history: false,
    })
    .build();

    assert_eq!(request.template, None);
}

/// A template may optionally be supplied alongside Fork/Mirror — that is
/// validated against the template's `designation` at creation time (requires
/// a config lookup), not at builder time, so `build()` must not panic merely
/// because a template is present.
#[test]
fn test_builder_fork_mode_with_template_does_not_panic_at_build_time() {
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("my-repo").unwrap(),
        OrganizationName::new("my-org").unwrap(),
    )
    .creation_mode(CreationMode::Fork { source: source() })
    .template(TemplateName::new("fork-settings").unwrap())
    .build();

    assert!(request.template.is_some());
}

#[test]
#[should_panic(expected = "content_strategy is not used with CreationMode::Fork")]
fn test_builder_fork_mode_with_explicit_empty_content_strategy_panics() {
    RepositoryCreationRequestBuilder::new(
        RepositoryName::new("my-repo").unwrap(),
        OrganizationName::new("my-org").unwrap(),
    )
    .creation_mode(CreationMode::Fork { source: source() })
    .content_strategy(ContentStrategy::Empty)
    .build();
}

#[test]
#[should_panic(expected = "content_strategy is not used with CreationMode::Fork")]
fn test_builder_fork_mode_with_explicit_custom_init_content_strategy_panics() {
    RepositoryCreationRequestBuilder::new(
        RepositoryName::new("my-repo").unwrap(),
        OrganizationName::new("my-org").unwrap(),
    )
    .creation_mode(CreationMode::Fork { source: source() })
    .content_strategy(ContentStrategy::CustomInit {
        include_readme: true,
        include_gitignore: false,
    })
    .build();
}

#[test]
#[should_panic(expected = "content_strategy is not used with CreationMode::Fork/Mirror")]
fn test_builder_mirror_history_mode_with_explicit_empty_content_strategy_panics() {
    RepositoryCreationRequestBuilder::new(
        RepositoryName::new("my-repo").unwrap(),
        OrganizationName::new("my-org").unwrap(),
    )
    .creation_mode(CreationMode::Mirror {
        source: source(),
        preserve_history: true,
    })
    .content_strategy(ContentStrategy::Empty)
    .build();
}

#[test]
#[should_panic(expected = "content_strategy is not used with CreationMode::Fork/Mirror")]
fn test_builder_mirror_source_only_mode_with_explicit_custom_init_content_strategy_panics() {
    RepositoryCreationRequestBuilder::new(
        RepositoryName::new("my-repo").unwrap(),
        OrganizationName::new("my-org").unwrap(),
    )
    .creation_mode(CreationMode::Mirror {
        source: source(),
        preserve_history: false,
    })
    .content_strategy(ContentStrategy::CustomInit {
        include_readme: false,
        include_gitignore: true,
    })
    .build();
}
