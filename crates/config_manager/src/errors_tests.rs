//! Tests for the fork/mirror-related `ConfigurationError` variants:
//! `AmbiguousTemplateDesignation`, `TemplateNotDesignated`, `CapabilityDisabled`,
//! `SourceOwnerNotAllowed`.
//!
//! See docs/spec/interfaces/fork-mirror-operations.md §1-2 and RCF-005/RCF-006.

use super::*;

// ============================================================================
// Tier 1: Specification tests — Display formatting
// ============================================================================

#[test]
fn test_capability_disabled_display_names_the_capability() {
    let error = ConfigurationError::CapabilityDisabled {
        capability: "fork".to_string(),
    };
    let message = error.to_string();
    assert!(
        message.contains("fork"),
        "message should name the disabled capability, got: {message}"
    );
}

#[test]
fn test_capability_disabled_display_names_mirror_capabilities_distinctly() {
    let fork = ConfigurationError::CapabilityDisabled {
        capability: "fork".to_string(),
    }
    .to_string();
    let history_mirror = ConfigurationError::CapabilityDisabled {
        capability: "source_history_mirror".to_string(),
    }
    .to_string();
    let source_only_mirror = ConfigurationError::CapabilityDisabled {
        capability: "source_only_mirror".to_string(),
    }
    .to_string();

    assert_ne!(fork, history_mirror);
    assert_ne!(history_mirror, source_only_mirror);
    assert_ne!(fork, source_only_mirror);
}

#[test]
fn test_template_not_designated_display_names_template_and_expected_designation() {
    let error = ConfigurationError::TemplateNotDesignated {
        template: "my-fork-template".to_string(),
        expected_designation: "Fork".to_string(),
        actual_designation: None,
    };
    let message = error.to_string();
    assert!(message.contains("my-fork-template"), "got: {message}");
    assert!(message.contains("Fork"), "got: {message}");
}

#[test]
fn test_template_not_designated_display_includes_actual_designation_when_present() {
    let error = ConfigurationError::TemplateNotDesignated {
        template: "wrong-template".to_string(),
        expected_designation: "Fork".to_string(),
        actual_designation: Some("SourceOnlyMirror".to_string()),
    };
    let message = error.to_string();
    assert!(
        message.contains("SourceOnlyMirror"),
        "message should surface the template's actual designation for diagnosis, got: {message}"
    );
}

#[test]
fn test_ambiguous_template_designation_display_lists_all_conflicting_templates() {
    let error = ConfigurationError::AmbiguousTemplateDesignation {
        org: "acme".to_string(),
        designation: "Fork".to_string(),
        templates: vec!["fork-a".to_string(), "fork-b".to_string()],
    };
    let message = error.to_string();
    assert!(message.contains("acme"), "got: {message}");
    assert!(message.contains("fork-a"), "got: {message}");
    assert!(message.contains("fork-b"), "got: {message}");
}

#[test]
fn test_source_owner_not_allowed_display_names_owner_and_capability() {
    let error = ConfigurationError::SourceOwnerNotAllowed {
        owner: "evil-org".to_string(),
        capability: "fork".to_string(),
    };
    let message = error.to_string();
    assert!(message.contains("evil-org"), "got: {message}");
    assert!(message.contains("fork"), "got: {message}");
}

#[test]
fn test_source_owner_not_allowed_display_names_each_capability_distinctly() {
    let fork = ConfigurationError::SourceOwnerNotAllowed {
        owner: "evil-org".to_string(),
        capability: "fork".to_string(),
    }
    .to_string();
    let history_mirror = ConfigurationError::SourceOwnerNotAllowed {
        owner: "evil-org".to_string(),
        capability: "source_history_mirror".to_string(),
    }
    .to_string();
    let source_only_mirror = ConfigurationError::SourceOwnerNotAllowed {
        owner: "evil-org".to_string(),
        capability: "source_only_mirror".to_string(),
    }
    .to_string();

    assert_ne!(fork, history_mirror);
    assert_ne!(history_mirror, source_only_mirror);
    assert_ne!(fork, source_only_mirror);
}

/// Stub-killing: the same owner rejected for two different capabilities (or
/// two different owners rejected for the same capability) must not compare
/// equal — rules out a buggy `PartialEq` derive or a constructor that
/// ignores one of its arguments.
#[test]
fn test_source_owner_not_allowed_errors_for_different_owners_are_not_equal() {
    let evil = ConfigurationError::SourceOwnerNotAllowed {
        owner: "evil-org".to_string(),
        capability: "fork".to_string(),
    };
    let other = ConfigurationError::SourceOwnerNotAllowed {
        owner: "other-org".to_string(),
        capability: "fork".to_string(),
    };
    assert_ne!(evil, other);
}

#[test]
fn test_source_owner_not_allowed_errors_for_different_capabilities_are_not_equal() {
    let fork = ConfigurationError::SourceOwnerNotAllowed {
        owner: "evil-org".to_string(),
        capability: "fork".to_string(),
    };
    let mirror = ConfigurationError::SourceOwnerNotAllowed {
        owner: "evil-org".to_string(),
        capability: "source_history_mirror".to_string(),
    };
    assert_ne!(fork, mirror);
}

#[test]
fn test_source_owner_not_allowed_is_cloneable() {
    let error = ConfigurationError::SourceOwnerNotAllowed {
        owner: "evil-org".to_string(),
        capability: "fork".to_string(),
    };
    let cloned = error.clone();
    assert_eq!(error, cloned);
}

/// Security: the message must name the rejected owner/capability but must
/// not leak internal configuration structure or the full allow-list
/// contents (which would partially disclose configuration meant to stay
/// internal to the organization's administrators).
#[test]
fn test_source_owner_not_allowed_message_does_not_leak_internal_structure() {
    let error = ConfigurationError::SourceOwnerNotAllowed {
        owner: "evil-org".to_string(),
        capability: "source_only_mirror".to_string(),
    };
    let message = error.to_string();
    for forbidden in ["override_allowed", "OverridableValue", "token", "secret"] {
        assert!(
            !message.to_lowercase().contains(&forbidden.to_lowercase()),
            "message should not leak '{forbidden}', got: {message}"
        );
    }
}

// ============================================================================
// Tier 2: Adversarial — equality, cloning, and information hygiene
// ============================================================================

/// Stub-killing: two `CapabilityDisabled` errors for different capabilities
/// must not compare equal (rules out a buggy `PartialEq` derive that ignores
/// the field, or a constructor that ignores its argument).
#[test]
fn test_capability_disabled_errors_for_different_capabilities_are_not_equal() {
    let fork = ConfigurationError::CapabilityDisabled {
        capability: "fork".to_string(),
    };
    let mirror = ConfigurationError::CapabilityDisabled {
        capability: "source_history_mirror".to_string(),
    };
    assert_ne!(fork, mirror);
}

#[test]
fn test_configuration_error_variants_are_cloneable() {
    let error = ConfigurationError::TemplateNotDesignated {
        template: "t".to_string(),
        expected_designation: "Fork".to_string(),
        actual_designation: None,
    };
    let cloned = error.clone();
    assert_eq!(error, cloned);
}

/// Security: error messages must name the capability/template but must not
/// leak internal configuration structure (field names like `override_allowed`,
/// raw TOML, or token-like strings) per the Security Rules constraint that
/// `CapabilityDisabled`/`TemplateNotDesignated` messages name the
/// capability/template only.
#[test]
fn test_capability_disabled_message_does_not_leak_internal_structure() {
    let error = ConfigurationError::CapabilityDisabled {
        capability: "source_only_mirror".to_string(),
    };
    let message = error.to_string();
    for forbidden in ["override_allowed", "OverridableValue", "token", "secret"] {
        assert!(
            !message.to_lowercase().contains(&forbidden.to_lowercase()),
            "message should not leak '{forbidden}', got: {message}"
        );
    }
}

#[test]
fn test_template_not_designated_message_does_not_leak_internal_structure() {
    let error = ConfigurationError::TemplateNotDesignated {
        template: "t".to_string(),
        expected_designation: "Fork".to_string(),
        actual_designation: Some("SourceOnlyMirror".to_string()),
    };
    let message = error.to_string();
    for forbidden in ["override_allowed", "OverridableValue", "token", "secret"] {
        assert!(
            !message.to_lowercase().contains(&forbidden.to_lowercase()),
            "message should not leak '{forbidden}', got: {message}"
        );
    }
}
