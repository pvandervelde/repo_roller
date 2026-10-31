//! Tests for configuration module.

// Note: `resolve_organization_configuration` itself is integration-style and
// requires a live GitHub client, a real metadata repository, and (for the
// #255 follow-up "designated template is never auto-discovered" fix) real
// template repositories to exercise its new auto-discovery branch
// meaningfully. By design it constructs its own `GitHubMetadataProvider` /
// `TemplateLoader` / `OrganizationSettingsManager` internally from a bare
// installation token with no injection seam -- per
// docs/spec/interfaces/fork-mirror-operations.md and the #255 follow-up gap
// note ("it must call the settings_manager it already constructs
// internally"), this is intentional and not something this test suite
// should work around with a seam of its own. Comprehensive behavioral
// testing of the full resolution pipeline (including live network calls) is
// done at the integration test level in the integration_tests crate, where
// real (credentialed) GitHub App clients are used -- see
// crates/integration_tests/src/test_runner.rs. Unit tests here focus on
// ensuring the module compiles and exports the expected functions, plus the
// one piece of the #255 follow-up fix that *is* synchronous and
// network-free: `effective_template_name` below.
//
// `effective_template_name` isolates the decision of which template name
// `resolve_organization_configuration` must use to build its
// `ConfigurationContext`, given:
//   (a) the caller's explicit `template_name` (empty string `""` when the
//       caller gave none),
//   (b) the `TemplateDesignation` implied by the request's `CreationMode`
//       (`None` for `CreationMode::Standard`; see
//       `designation_for_creation_mode` in `lib_tests.rs`), and
//   (c) the `ConfigurationResult<Option<String>>` returned by calling
//       `settings_manager.find_designated_template(organization,
//       designation)` -- which `resolve_organization_configuration` must
//       only actually invoke when `template_name.is_empty() &&
//       designation.is_some()`.
//
// `resolve_organization_configuration` MUST delegate to this exact function
// (or logic equivalent to it) to decide the effective template name, and
// MUST propagate any `Err` it returns (notably `AmbiguousTemplateDesignation`)
// rather than swallowing it into the metadata-repository-unavailable
// fallback path (that fallback is for CRF-001 -- "metadata repository
// cannot be discovered" -- a different, legitimate case).
//
// This test file pins down that decision table so the auto-discovery wiring
// (the actual #255 follow-up defect: "load_creation_config only loads a
// template when the caller's request.template is explicitly Some(...)")
// cannot regress to "always ignore designation" or "always prefer discovery
// over an explicit caller choice".

use config_manager::{ConfigurationError, TemplateDesignation};

use super::*;

/// Verify that the configuration module compiles and exports expected functions.
///
/// This test ensures the module's public interface is available.
/// Full behavioral testing is done in integration tests.
#[test]
fn test_configuration_module_compiles() {
    // This test just needs to compile to verify the module structure is correct.
    // The actual functions are tested via integration tests with mocked clients.

    // Verify types are accessible
    let _: Option<config_manager::MergedConfiguration> = None;
}

// ============================================================================
// effective_template_name Tests (RED: #255 follow-up Gap 1 --
// "designated template is never auto-discovered")
// ============================================================================

#[test]
fn test_effective_template_name_explicit_template_wins_over_discovered_name() {
    let result = effective_template_name(
        "explicit-template",
        Some(TemplateDesignation::Fork),
        Ok(Some("discovered-template".to_string())),
    );

    assert_eq!(
        result,
        Ok(Some("explicit-template".to_string())),
        "an explicitly named template must always win over auto-discovery"
    );
}

#[test]
fn test_effective_template_name_explicit_template_wins_even_over_ambiguous_discovery_error() {
    // Adversarial: proves the explicit-template branch short-circuits
    // BEFORE even inspecting the discovery result. A lazy implementation
    // that always evaluates/propagates `discovery_result` regardless of
    // `template_name` would fail this test even though it might pass the
    // "wins over Ok(Some(..))" test above.
    let ambiguous = Err(ConfigurationError::AmbiguousTemplateDesignation {
        org: "acme-corp".to_string(),
        designation: "Fork".to_string(),
        templates: vec!["fork-a".to_string(), "fork-b".to_string()],
    });

    let result = effective_template_name("explicit-template", Some(TemplateDesignation::Fork), ambiguous);

    assert_eq!(result, Ok(Some("explicit-template".to_string())));
}

#[test]
fn test_effective_template_name_standard_mode_ignores_discovered_name() {
    let result = effective_template_name("", None, Ok(Some("ignored-template".to_string())));

    assert_eq!(
        result,
        Ok(None),
        "CreationMode::Standard (designation = None) must never surface a discovered name"
    );
}

#[test]
fn test_effective_template_name_no_designation_returns_none_even_if_discovery_errored() {
    // Even if a (buggy) caller passed an Err discovery_result alongside
    // designation = None, Standard mode must not propagate it -- discovery
    // should never have been attempted in the first place when there is no
    // designation to discover.
    let ambiguous = Err(ConfigurationError::AmbiguousTemplateDesignation {
        org: "acme-corp".to_string(),
        designation: "Fork".to_string(),
        templates: vec!["fork-a".to_string(), "fork-b".to_string()],
    });

    let result = effective_template_name("", None, ambiguous);

    assert_eq!(result, Ok(None));
}

#[test]
fn test_effective_template_name_auto_discovers_designated_template_when_found() {
    let result = effective_template_name(
        "",
        Some(TemplateDesignation::SourceHistoryMirror),
        Ok(Some("history-mirror-settings".to_string())),
    );

    assert_eq!(result, Ok(Some("history-mirror-settings".to_string())));
}

#[test]
fn test_effective_template_name_distinct_designations_can_discover_distinct_names() {
    // Stub-killing: a lazy implementation that ignores `designation`/
    // `discovery_result` and always returns a fixed name would fail here,
    // since the two calls below must surface the two distinct discovered
    // names unchanged.
    let fork_result = effective_template_name(
        "",
        Some(TemplateDesignation::Fork),
        Ok(Some("fork-settings".to_string())),
    );
    let mirror_result = effective_template_name(
        "",
        Some(TemplateDesignation::SourceOnlyMirror),
        Ok(Some("mirror-settings".to_string())),
    );

    assert_eq!(fork_result, Ok(Some("fork-settings".to_string())));
    assert_eq!(mirror_result, Ok(Some("mirror-settings".to_string())));
    assert_ne!(fork_result, mirror_result);
}

#[test]
fn test_effective_template_name_no_designated_template_found_returns_none_not_error() {
    let result = effective_template_name("", Some(TemplateDesignation::SourceOnlyMirror), Ok(None));

    assert_eq!(
        result,
        Ok(None),
        "no designated template existing for the organization is a valid outcome, not an error"
    );
}

#[test]
fn test_effective_template_name_propagates_ambiguous_designation_error() {
    let ambiguous = Err(ConfigurationError::AmbiguousTemplateDesignation {
        org: "acme-corp".to_string(),
        designation: "SourceOnlyMirror".to_string(),
        templates: vec!["mirror-a".to_string(), "mirror-b".to_string()],
    });

    let result = effective_template_name("", Some(TemplateDesignation::SourceOnlyMirror), ambiguous);

    assert!(
        matches!(
            result,
            Err(ConfigurationError::AmbiguousTemplateDesignation { .. })
        ),
        "an ambiguous designation must propagate as an error, not be swallowed: {result:?}"
    );
}

#[test]
fn test_effective_template_name_whitespace_only_name_counts_as_explicit() {
    // Boundary: the contract is literally `template_name.is_empty()`, not
    // `.trim().is_empty()`. A single space must take the explicit-template
    // path (and therefore ignore discovery entirely), not the auto-discovery
    // path.
    let result = effective_template_name(
        " ",
        Some(TemplateDesignation::Fork),
        Ok(Some("discovered-template".to_string())),
    );

    assert_eq!(result, Ok(Some(" ".to_string())));
}

// Integration-level tests (full `resolve_organization_configuration` pipeline,
// including the new auto-discovery wiring exercised against a live
// metadata/template repository) are in crates/integration_tests/tests/
// where we can use real, credentialed:
// - GitHubClient for repository operations
// - MetadataProvider for configuration access
// - Full configuration resolution workflow, including designated-template
//   auto-discovery
// - Configuration application with custom properties
