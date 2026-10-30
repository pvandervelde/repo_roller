//! RED-phase test suite for `CreateRepositoryRequest.creation_mode`
//! (the `creationMode` JSON field).
//!
//! See `docs/spec/interfaces/fork-mirror-operations.md` §8a.
//!
//! ## Interface contract assumed by these tests (not yet implemented)
//!
//! ```ignore
//! #[serde(skip_serializing_if = "Option::is_none")]
//! pub creation_mode: Option<repo_roller_core::CreationMode>,
//! ```
//!
//! added to `CreateRepositoryRequest`, reusing `repo_roller_core::CreationMode`
//! directly (its own `#[serde(tag = "type", rename_all = "snake_case")]`
//! defines the wire shape of the nested object).

use super::*;
use repo_roller_core::{CreationMode, OrganizationName, RepositoryName, SourceRepositoryReference};

fn expected_source(owner: &str, name: &str) -> SourceRepositoryReference {
    SourceRepositoryReference {
        owner: OrganizationName::new(owner).unwrap(),
        name: RepositoryName::new(name).unwrap(),
    }
}

// ============================================================================
// Tier 1: Specification tests — field presence, absence, and defaults
// ============================================================================

/// A request with no `creationMode` field at all must deserialize with
/// `creation_mode: None` — the pre-existing, overwhelming-majority case
/// (Standard creation) must keep working unchanged.
#[test]
fn test_create_repository_request_without_creation_mode_field_yields_none() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo",
        "template": "rust-library"
    }"#;

    let req: CreateRepositoryRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.creation_mode, None);
}

#[test]
fn test_create_repository_request_with_explicit_standard_creation_mode() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo",
        "template": "rust-library",
        "creationMode": { "type": "standard" }
    }"#;

    let req: CreateRepositoryRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.creation_mode, Some(CreationMode::Standard));
}

#[test]
fn test_create_repository_request_with_fork_creation_mode_deserializes() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo",
        "creationMode": {
            "type": "fork",
            "source": { "owner": "upstream-org", "name": "some-repo" }
        }
    }"#;

    let req: CreateRepositoryRequest = serde_json::from_str(json).unwrap();
    assert_eq!(
        req.creation_mode,
        Some(CreationMode::Fork {
            source: expected_source("upstream-org", "some-repo")
        })
    );
}

#[test]
fn test_create_repository_request_with_mirror_history_creation_mode_deserializes() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo",
        "creationMode": {
            "type": "mirror",
            "source": { "owner": "upstream-org", "name": "some-repo" },
            "preserveHistory": true
        }
    }"#;

    let req: CreateRepositoryRequest = serde_json::from_str(json).unwrap();
    assert_eq!(
        req.creation_mode,
        Some(CreationMode::Mirror {
            source: expected_source("upstream-org", "some-repo"),
            preserve_history: true,
        })
    );
}

#[test]
fn test_create_repository_request_with_mirror_source_only_creation_mode_deserializes() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo",
        "creationMode": {
            "type": "mirror",
            "source": { "owner": "upstream-org", "name": "some-repo" },
            "preserveHistory": false
        }
    }"#;

    let req: CreateRepositoryRequest = serde_json::from_str(json).unwrap();
    assert_eq!(
        req.creation_mode,
        Some(CreationMode::Mirror {
            source: expected_source("upstream-org", "some-repo"),
            preserve_history: false,
        })
    );
}

/// Stub-killing: a buggy implementation that hardcodes `preserve_history:
/// true` regardless of the wire value would pass the "true" test above but
/// fail this one.
#[test]
fn test_create_repository_request_mirror_preserve_history_false_is_not_coerced_to_true() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo",
        "creationMode": {
            "type": "mirror",
            "source": { "owner": "upstream-org", "name": "some-repo" },
            "preserveHistory": false
        }
    }"#;

    let req: CreateRepositoryRequest = serde_json::from_str(json).unwrap();
    match req.creation_mode {
        Some(CreationMode::Mirror {
            preserve_history, ..
        }) => assert!(!preserve_history),
        other => panic!("expected Some(Mirror {{ .. }}), got {:?}", other),
    }
}

/// Round-trip: serializing a fork request and re-deserializing it must
/// reproduce an equal value — exercises `skip_serializing_if` alongside the
/// nested tagged enum together, not just one direction.
#[test]
fn test_create_repository_request_creation_mode_fork_serde_roundtrip() {
    let req = CreateRepositoryRequest {
        organization: "myorg".to_string(),
        name: "my-repo".to_string(),
        template: None,
        visibility: None,
        team: None,
        repository_type: None,
        variables: Default::default(),
        content_strategy: repo_roller_core::ContentStrategy::Template,
        creation_mode: Some(CreationMode::Fork {
            source: expected_source("upstream-org", "some-repo"),
        }),
        teams: Default::default(),
        collaborators: Default::default(),
    };

    let json = serde_json::to_string(&req).expect("serialize");
    let roundtripped: CreateRepositoryRequest = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(req.creation_mode, roundtripped.creation_mode);
}

/// When `creation_mode` is `None`, it must be omitted from the serialized
/// JSON entirely (`skip_serializing_if = "Option::is_none"`), not emitted as
/// an explicit `null` — this matters for API clients that treat an explicit
/// `"creationMode": null` differently from an absent key.
#[test]
fn test_create_repository_request_with_none_creation_mode_omits_field_from_json() {
    let req = CreateRepositoryRequest {
        organization: "myorg".to_string(),
        name: "my-repo".to_string(),
        template: Some("rust-library".to_string()),
        visibility: None,
        team: None,
        repository_type: None,
        variables: Default::default(),
        content_strategy: repo_roller_core::ContentStrategy::Template,
        creation_mode: None,
        teams: Default::default(),
        collaborators: Default::default(),
    };

    let json = serde_json::to_string(&req).expect("serialize");
    assert!(
        !json.contains("creationMode"),
        "creationMode must be omitted when None, got: {json}"
    );
}

// ============================================================================
// Tier 2: Adversarial — malformed `creationMode` payloads
// ============================================================================

#[test]
fn test_create_repository_request_creation_mode_unknown_type_tag_is_rejected() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo",
        "creationMode": { "type": "clone-via-ftp" }
    }"#;

    let result = serde_json::from_str::<CreateRepositoryRequest>(json);
    assert!(
        result.is_err(),
        "unknown creationMode type tag must be rejected"
    );
}

#[test]
fn test_create_repository_request_creation_mode_fork_missing_source_is_rejected() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo",
        "creationMode": { "type": "fork" }
    }"#;

    let result = serde_json::from_str::<CreateRepositoryRequest>(json);
    assert!(
        result.is_err(),
        "fork creationMode without source must be rejected"
    );
}

#[test]
fn test_create_repository_request_creation_mode_mirror_missing_preserve_history_is_rejected() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo",
        "creationMode": {
            "type": "mirror",
            "source": { "owner": "upstream-org", "name": "some-repo" }
        }
    }"#;

    let result = serde_json::from_str::<CreateRepositoryRequest>(json);
    assert!(
        result.is_err(),
        "mirror creationMode without preserveHistory must be rejected"
    );
}

#[test]
fn test_create_repository_request_creation_mode_source_missing_name_is_rejected() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo",
        "creationMode": {
            "type": "fork",
            "source": { "owner": "upstream-org" }
        }
    }"#;

    let result = serde_json::from_str::<CreateRepositoryRequest>(json);
    assert!(result.is_err(), "source without name must be rejected");
}

#[test]
fn test_create_repository_request_creation_mode_source_missing_owner_is_rejected() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo",
        "creationMode": {
            "type": "fork",
            "source": { "name": "some-repo" }
        }
    }"#;

    let result = serde_json::from_str::<CreateRepositoryRequest>(json);
    assert!(result.is_err(), "source without owner must be rejected");
}

/// `preserveHistory` must be a JSON boolean; a string in its place must be
/// rejected rather than coerced.
#[test]
fn test_create_repository_request_creation_mode_preserve_history_wrong_type_is_rejected() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo",
        "creationMode": {
            "type": "mirror",
            "source": { "owner": "upstream-org", "name": "some-repo" },
            "preserveHistory": "yes"
        }
    }"#;

    let result = serde_json::from_str::<CreateRepositoryRequest>(json);
    assert!(
        result.is_err(),
        "non-boolean preserveHistory must be rejected"
    );
}

/// This is deliberately NOT asserting an error: `OrganizationName`/
/// `RepositoryName` derive `Deserialize` directly on their inner `String`
/// (see `crates/repo_roller_core/src/repository.rs`), which does not invoke
/// `OrganizationName::new()`/`RepositoryName::new()` validation. This means
/// a syntactically-invalid owner/name (spaces, leading hyphen, etc.) inside
/// `creationMode.source` currently deserializes successfully at the JSON
/// layer. This test documents that fact so the gap is visible, and
/// `translation_fork_mirror_tests.rs` verifies the translation layer must
/// still reject such values before they reach the domain layer — see the
/// "spec gap" note in that file and in the Tester's final report.
#[test]
fn test_create_repository_request_creation_mode_source_with_syntactically_invalid_owner_deserializes_at_json_layer(
) {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo",
        "creationMode": {
            "type": "fork",
            "source": { "owner": "invalid owner with spaces", "name": "some-repo" }
        }
    }"#;

    let result = serde_json::from_str::<CreateRepositoryRequest>(json);
    assert!(
        result.is_ok(),
        "documents current (non-validating) Deserialize behaviour for OrganizationName; \
         if this assertion ever flips to Err, update this test and remove the corresponding \
         re-validation in http_create_repository_request_to_domain as redundant"
    );
}
