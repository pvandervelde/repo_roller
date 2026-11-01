//! RED-phase test suite for fork/mirror `creation_mode` handling in
//! `http_create_repository_request_to_domain`.
//!
//! See `docs/spec/interfaces/fork-mirror-operations.md` §8a.
//!
//! ## Spec gap flagged by this file (see also `request_fork_mirror_tests.rs`)
//!
//! §8a states: "(`SourceRepositoryReference`'s fields deserialize via
//! `OrganizationName`/`RepositoryName`'s existing `Deserialize` impls — no
//! new HTTP-layer validation code needed for the nested fields themselves,
//! since those branded types already validate on deserialize.)"
//!
//! This is not accurate for the current implementation:
//! `OrganizationName`/`RepositoryName` (`repo_roller_core/src/repository.rs`)
//! derive `Deserialize` directly on their inner `String` via
//! `#[derive(..., Deserialize)]`. The derived impl does **not** call
//! `OrganizationName::new()`/`RepositoryName::new()` — it constructs the
//! branded type directly from whatever string was in the JSON. Validation
//! only happens through the `::new()` constructor, which the derive macro
//! does not use.
//!
//! Consequence: a caller can submit
//! `"creationMode": {"type":"fork","source":{"owner":"invalid owner with spaces","name":"x"}}`
//! and it will deserialize successfully into an `OrganizationName` holding
//! the invalid string verbatim. If `http_create_repository_request_to_domain`
//! does nothing beyond "pass `creation_mode` straight to
//! `builder.creation_mode(...)`" (the literal text of §8a), this invalid,
//! unvalidated owner reaches the domain layer — and from there,
//! `repo_roller_core`/`github_client` — without ever going through
//! `ApiError::validation_error`. This violates the stated testing
//! requirement: "invalid nested `source.owner`/`source.name` →
//! `ApiError::validation_error`".
//!
//! The tests below therefore require the Coder to add explicit
//! re-validation of `source.owner.as_str()` /`source.name.as_str()` (e.g.
//! via `OrganizationName::new(...)`/`RepositoryName::new(...)`) inside
//! `http_create_repository_request_to_domain` when `creation_mode` carries a
//! `Fork`/`Mirror` source — mirroring the existing `name`/`owner`/`template`
//! validation pattern already in this function (lines ~41-56). This is
//! reported as a spec gap to the architect; these tests encode the required
//! (not merely literal) behaviour.

use super::*;
use repo_roller_core::{CreationMode, OrganizationName, RepositoryName, SourceRepositoryReference};
use std::collections::HashMap;

fn valid_source() -> SourceRepositoryReference {
    SourceRepositoryReference {
        owner: OrganizationName::new("upstream-org").unwrap(),
        name: RepositoryName::new("some-repo").unwrap(),
    }
}

fn base_http_req() -> CreateRepositoryRequest {
    CreateRepositoryRequest {
        organization: "myorg".to_string(),
        name: "my-repo".to_string(),
        template: None,
        visibility: None,
        team: None,
        repository_type: None,
        variables: HashMap::new(),
        content_strategy: repo_roller_core::ContentStrategy::Template,
        creation_mode: None,
        teams: HashMap::new(),
        collaborators: HashMap::new(),
    }
}

// ============================================================================
// Tier 1: Specification tests — creation_mode absent/Standard behaves as before
// ============================================================================

#[test]
fn test_translation_with_absent_creation_mode_defaults_domain_to_standard() {
    let mut http_req = base_http_req();
    http_req.template = Some("rust-library".to_string());
    http_req.creation_mode = None;

    let result = http_create_repository_request_to_domain(http_req, "actor".to_string());
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result.err());
    assert_eq!(result.unwrap().creation_mode, CreationMode::Standard);
}

#[test]
fn test_translation_with_explicit_standard_creation_mode_behaves_like_absent() {
    let mut http_req = base_http_req();
    http_req.template = Some("rust-library".to_string());
    http_req.creation_mode = Some(CreationMode::Standard);

    let result = http_create_repository_request_to_domain(http_req, "actor".to_string());
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result.err());
    assert_eq!(result.unwrap().creation_mode, CreationMode::Standard);
}

/// Regression guard: adding `creation_mode` handling must not disturb the
/// pre-existing Standard-mode validation (Template strategy requires a
/// template name).
#[test]
fn test_translation_standard_mode_template_strategy_without_template_still_rejected() {
    let mut http_req = base_http_req();
    http_req.creation_mode = Some(CreationMode::Standard);
    http_req.content_strategy = repo_roller_core::ContentStrategy::Template;
    http_req.template = None;

    let result = http_create_repository_request_to_domain(http_req, "actor".to_string());
    assert!(result.is_err());
}

#[test]
fn test_translation_fork_creation_mode_translates_to_domain_fork() {
    let mut http_req = base_http_req();
    http_req.creation_mode = Some(CreationMode::Fork {
        source: valid_source(),
    });

    let result = http_create_repository_request_to_domain(http_req, "actor".to_string());
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result.err());
    assert_eq!(
        result.unwrap().creation_mode,
        CreationMode::Fork {
            source: valid_source()
        }
    );
}

#[test]
fn test_translation_mirror_history_creation_mode_translates_to_domain_mirror() {
    let mut http_req = base_http_req();
    http_req.creation_mode = Some(CreationMode::Mirror {
        source: valid_source(),
        preserve_history: true,
    });

    let result = http_create_repository_request_to_domain(http_req, "actor".to_string());
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result.err());
    assert_eq!(
        result.unwrap().creation_mode,
        CreationMode::Mirror {
            source: valid_source(),
            preserve_history: true,
        }
    );
}

#[test]
fn test_translation_mirror_source_only_creation_mode_translates_to_domain_mirror() {
    let mut http_req = base_http_req();
    http_req.creation_mode = Some(CreationMode::Mirror {
        source: valid_source(),
        preserve_history: false,
    });

    let result = http_create_repository_request_to_domain(http_req, "actor".to_string());
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result.err());
    assert_eq!(
        result.unwrap().creation_mode,
        CreationMode::Mirror {
            source: valid_source(),
            preserve_history: false,
        }
    );
}

/// Stub-killing: a buggy implementation that ignores `preserve_history`
/// would make Fork/Mirror tests pass independently but fail this direct
/// distinctness check.
#[test]
fn test_translation_mirror_preserve_history_true_and_false_are_distinct_after_translation() {
    let mut http_req_true = base_http_req();
    http_req_true.creation_mode = Some(CreationMode::Mirror {
        source: valid_source(),
        preserve_history: true,
    });
    let mut http_req_false = base_http_req();
    http_req_false.creation_mode = Some(CreationMode::Mirror {
        source: valid_source(),
        preserve_history: false,
    });

    let domain_true =
        http_create_repository_request_to_domain(http_req_true, "actor".to_string()).unwrap();
    let domain_false =
        http_create_repository_request_to_domain(http_req_false, "actor".to_string()).unwrap();

    assert_ne!(domain_true.creation_mode, domain_false.creation_mode);
}

/// Template is optional for Fork — translation must not require it (unlike
/// Standard + Template strategy).
#[test]
fn test_translation_fork_without_template_succeeds() {
    let mut http_req = base_http_req();
    http_req.template = None;
    http_req.creation_mode = Some(CreationMode::Fork {
        source: valid_source(),
    });

    let result = http_create_repository_request_to_domain(http_req, "actor".to_string());
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result.err());
}

#[test]
fn test_translation_fork_with_optional_template_succeeds() {
    let mut http_req = base_http_req();
    http_req.template = Some("fork-settings".to_string());
    http_req.creation_mode = Some(CreationMode::Fork {
        source: valid_source(),
    });

    let result = http_create_repository_request_to_domain(http_req, "actor".to_string());
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result.err());
    assert_eq!(result.unwrap().template.unwrap().as_ref(), "fork-settings");
}

// ============================================================================
// Tier 1/2: content_strategy + fork/mirror creation_mode validation
// ============================================================================

#[test]
fn test_translation_fork_with_explicit_empty_content_strategy_is_rejected() {
    let mut http_req = base_http_req();
    http_req.creation_mode = Some(CreationMode::Fork {
        source: valid_source(),
    });
    http_req.content_strategy = repo_roller_core::ContentStrategy::Empty;

    let result = http_create_repository_request_to_domain(http_req, "actor".to_string());
    assert!(
        result.is_err(),
        "fork creation_mode with explicit non-default content_strategy must be rejected"
    );
}

#[test]
fn test_translation_fork_with_explicit_custom_init_content_strategy_is_rejected() {
    let mut http_req = base_http_req();
    http_req.creation_mode = Some(CreationMode::Fork {
        source: valid_source(),
    });
    http_req.content_strategy = repo_roller_core::ContentStrategy::CustomInit {
        include_readme: true,
        include_gitignore: false,
    };

    let result = http_create_repository_request_to_domain(http_req, "actor".to_string());
    assert!(result.is_err());
}

#[test]
fn test_translation_mirror_history_with_explicit_empty_content_strategy_is_rejected() {
    let mut http_req = base_http_req();
    http_req.creation_mode = Some(CreationMode::Mirror {
        source: valid_source(),
        preserve_history: true,
    });
    http_req.content_strategy = repo_roller_core::ContentStrategy::Empty;

    let result = http_create_repository_request_to_domain(http_req, "actor".to_string());
    assert!(result.is_err());
}

#[test]
fn test_translation_mirror_source_only_with_explicit_custom_init_content_strategy_is_rejected() {
    let mut http_req = base_http_req();
    http_req.creation_mode = Some(CreationMode::Mirror {
        source: valid_source(),
        preserve_history: false,
    });
    http_req.content_strategy = repo_roller_core::ContentStrategy::CustomInit {
        include_readme: false,
        include_gitignore: true,
    };

    let result = http_create_repository_request_to_domain(http_req, "actor".to_string());
    assert!(result.is_err());
}

/// The default `content_strategy` (absent from JSON ⇒ `Template`) is
/// compatible with Fork/Mirror and must not be rejected — mirrors the
/// domain builder's own treatment of `Template` as the "no-op" default.
#[test]
fn test_translation_fork_with_default_template_content_strategy_succeeds() {
    let mut http_req = base_http_req();
    http_req.creation_mode = Some(CreationMode::Fork {
        source: valid_source(),
    });
    http_req.content_strategy = repo_roller_core::ContentStrategy::Template;

    let result = http_create_repository_request_to_domain(http_req, "actor".to_string());
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result.err());
}

#[test]
fn test_translation_mirror_source_only_with_default_template_content_strategy_succeeds() {
    let mut http_req = base_http_req();
    http_req.creation_mode = Some(CreationMode::Mirror {
        source: valid_source(),
        preserve_history: false,
    });
    http_req.content_strategy = repo_roller_core::ContentStrategy::Template;

    let result = http_create_repository_request_to_domain(http_req, "actor".to_string());
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result.err());
}

/// Regression: Standard mode with a non-default content_strategy must
/// continue to be accepted (this validation is scoped to Fork/Mirror only).
#[test]
fn test_translation_standard_mode_with_explicit_empty_content_strategy_still_accepted() {
    let mut http_req = base_http_req();
    http_req.creation_mode = Some(CreationMode::Standard);
    http_req.content_strategy = repo_roller_core::ContentStrategy::Empty;

    let result = http_create_repository_request_to_domain(http_req, "actor".to_string());
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result.err());
}

#[test]
fn test_translation_standard_mode_with_explicit_custom_init_content_strategy_still_accepted() {
    let mut http_req = base_http_req();
    http_req.creation_mode = None; // absent == Standard
    http_req.content_strategy = repo_roller_core::ContentStrategy::CustomInit {
        include_readme: true,
        include_gitignore: true,
    };

    let result = http_create_repository_request_to_domain(http_req, "actor".to_string());
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result.err());
}

/// Verifies the exact field name used in the validation error, per spec:
/// `ApiError::validation_error("content_strategy", "not valid with a
/// fork/mirror creation_mode")`. `ApiError` has no `PartialEq`/`Display`, so
/// this inspects the `Debug` representation (which surfaces the wrapped
/// `anyhow::Error`'s message).
#[test]
fn test_translation_fork_content_strategy_conflict_error_mentions_content_strategy_field() {
    let mut http_req = base_http_req();
    http_req.creation_mode = Some(CreationMode::Fork {
        source: valid_source(),
    });
    http_req.content_strategy = repo_roller_core::ContentStrategy::Empty;

    let result = http_create_repository_request_to_domain(http_req, "actor".to_string());
    let err = result.expect_err("must be rejected");
    let debug_repr = format!("{:?}", err);
    assert!(
        debug_repr.contains("content_strategy") || debug_repr.contains("contentStrategy"),
        "error should reference the content_strategy field, got: {debug_repr}"
    );
}

/// No builder panic must reach the caller: calling the translation function
/// directly (not through `catch_unwind`) and getting back a graceful `Err`
/// is itself the proof — if the implementation instead delegated straight
/// to `builder.content_strategy(..).creation_mode(..).build()` without the
/// up-front check, this test would abort the whole test binary via panic
/// rather than failing gracefully.
#[test]
fn test_translation_fork_content_strategy_conflict_does_not_panic_the_builder() {
    let mut http_req = base_http_req();
    http_req.creation_mode = Some(CreationMode::Mirror {
        source: valid_source(),
        preserve_history: true,
    });
    http_req.content_strategy = repo_roller_core::ContentStrategy::CustomInit {
        include_readme: true,
        include_gitignore: true,
    };

    let result = http_create_repository_request_to_domain(http_req, "actor".to_string());
    assert!(result.is_err());
}

// ============================================================================
// Tier 2: Adversarial — invalid nested `source` fields must not reach the domain
// ============================================================================
//
// See the module-level doc comment above: `OrganizationName`/`RepositoryName`
// deserialize without validation, so these invalid values are constructed
// here exactly as an attacker-controlled HTTP client would produce them —
// via `serde_json::from_str`, not via direct (private-field) construction.

fn http_req_json_with_fork_source(owner_json: &str, name_json: &str) -> String {
    format!(
        r#"{{
            "organization": "myorg",
            "name": "my-repo",
            "creationMode": {{
                "type": "fork",
                "source": {{ "owner": {owner_json}, "name": {name_json} }}
            }}
        }}"#
    )
}

#[test]
fn test_translation_rejects_fork_source_owner_containing_spaces() {
    let json = http_req_json_with_fork_source("\"invalid owner with spaces\"", "\"some-repo\"");
    let http_req: CreateRepositoryRequest =
        serde_json::from_str(&json).expect("HTTP-layer deserialization currently has no field-level validation for nested source fields");

    let result = http_create_repository_request_to_domain(http_req, "actor".to_string());
    assert!(
        result.is_err(),
        "a syntactically invalid source.owner must be rejected as ApiError::validation_error, \
         not silently passed through to the domain layer (see module-level spec-gap note)"
    );
}

#[test]
fn test_translation_rejects_fork_source_owner_with_leading_hyphen() {
    let json = http_req_json_with_fork_source("\"-bad-owner\"", "\"some-repo\"");
    let http_req: CreateRepositoryRequest =
        serde_json::from_str(&json).expect("deserializes at JSON layer");

    let result = http_create_repository_request_to_domain(http_req, "actor".to_string());
    assert!(result.is_err());
}

#[test]
fn test_translation_rejects_fork_source_owner_with_double_hyphen() {
    let json = http_req_json_with_fork_source("\"bad--owner\"", "\"some-repo\"");
    let http_req: CreateRepositoryRequest =
        serde_json::from_str(&json).expect("deserializes at JSON layer");

    let result = http_create_repository_request_to_domain(http_req, "actor".to_string());
    assert!(result.is_err());
}

#[test]
fn test_translation_rejects_fork_source_owner_empty_string() {
    let json = http_req_json_with_fork_source("\"\"", "\"some-repo\"");
    let http_req: CreateRepositoryRequest =
        serde_json::from_str(&json).expect("deserializes at JSON layer");

    let result = http_create_repository_request_to_domain(http_req, "actor".to_string());
    assert!(result.is_err());
}

#[test]
fn test_translation_rejects_fork_source_repo_name_starting_with_dot() {
    let json = http_req_json_with_fork_source("\"upstream-org\"", "\".bad-repo\"");
    let http_req: CreateRepositoryRequest =
        serde_json::from_str(&json).expect("deserializes at JSON layer");

    let result = http_create_repository_request_to_domain(http_req, "actor".to_string());
    assert!(result.is_err());
}

#[test]
fn test_translation_rejects_fork_source_repo_name_empty_string() {
    let json = http_req_json_with_fork_source("\"upstream-org\"", "\"\"");
    let http_req: CreateRepositoryRequest =
        serde_json::from_str(&json).expect("deserializes at JSON layer");

    let result = http_create_repository_request_to_domain(http_req, "actor".to_string());
    assert!(result.is_err());
}

#[test]
fn test_translation_rejects_mirror_source_owner_containing_spaces() {
    let json = format!(
        r#"{{
            "organization": "myorg",
            "name": "my-repo",
            "creationMode": {{
                "type": "mirror",
                "source": {{ "owner": "invalid owner", "name": "some-repo" }},
                "preserveHistory": true
            }}
        }}"#
    );
    let http_req: CreateRepositoryRequest =
        serde_json::from_str(&json).expect("deserializes at JSON layer");

    let result = http_create_repository_request_to_domain(http_req, "actor".to_string());
    assert!(result.is_err());
}

/// Positive control for the negative tests above: a syntactically valid
/// nested source must still be accepted, proving the added re-validation
/// doesn't reject everything indiscriminately.
#[test]
fn test_translation_accepts_fork_source_with_valid_owner_and_name() {
    let json = http_req_json_with_fork_source("\"upstream-org\"", "\"upstream-repo\"");
    let http_req: CreateRepositoryRequest = serde_json::from_str(&json).unwrap();

    let result = http_create_repository_request_to_domain(http_req, "actor".to_string());
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result.err());
}

// ============================================================================
// Tier 3: Property-based contract tests — translation must never panic
// ============================================================================

mod proptests {
    use super::*;
    use proptest::prelude::*;

    fn content_strategy_strategy() -> impl Strategy<Value = repo_roller_core::ContentStrategy> {
        prop_oneof![
            Just(repo_roller_core::ContentStrategy::Template),
            Just(repo_roller_core::ContentStrategy::Empty),
            (any::<bool>(), any::<bool>()).prop_map(|(r, g)| {
                repo_roller_core::ContentStrategy::CustomInit {
                    include_readme: r,
                    include_gitignore: g,
                }
            }),
        ]
    }

    fn creation_mode_strategy() -> impl Strategy<Value = Option<CreationMode>> {
        prop_oneof![
            Just(None),
            Just(Some(CreationMode::Standard)),
            Just(Some(CreationMode::Fork {
                source: valid_source()
            })),
            Just(Some(CreationMode::Mirror {
                source: valid_source(),
                preserve_history: true,
            })),
            Just(Some(CreationMode::Mirror {
                source: valid_source(),
                preserve_history: false,
            })),
        ]
    }

    proptest! {
        /// For any combination of arbitrary (possibly invalid) top-level
        /// `organization`/`name`/`template` strings crossed with every
        /// `creation_mode` variant and every `content_strategy` variant,
        /// `http_create_repository_request_to_domain` must return a
        /// `Result` without panicking. This is the core API-boundary safety
        /// property: handlers must never panic on caller input, and the
        /// domain builder's own guard (`request.rs`) is a `panic!`, not a
        /// `Result`, so this function is the only thing standing between
        /// arbitrary HTTP input and a crashed request handler.
        #[test]
        fn prop_translation_never_panics_for_arbitrary_top_level_fields(
            organization in ".{0,40}",
            name in ".{0,120}",
            template in proptest::option::of(".{0,60}"),
            content_strategy in content_strategy_strategy(),
            creation_mode in creation_mode_strategy(),
        ) {
            let http_req = CreateRepositoryRequest {
                organization,
                name,
                template,
                visibility: None,
                team: None,
                repository_type: None,
                variables: HashMap::new(),
                content_strategy,
                creation_mode,
                teams: HashMap::new(),
                collaborators: HashMap::new(),
            };

            // The specific Ok/Err outcome is not asserted here — only that
            // the call returns normally. A panic inside the function body
            // (e.g. an un-guarded builder.build() reaching its panic!
            // branches) would fail this property automatically.
            let result = http_create_repository_request_to_domain(http_req, "actor".to_string());
            prop_assert!(result.is_ok() || result.is_err());
        }

        /// Same property, exercised through the full JSON deserialization
        /// path with an arbitrary string injected into the nested
        /// `creationMode.source.owner`/`source.name` fields — the exact
        /// attack surface described in the spec-gap note at the top of
        /// this file. Deserialization itself may fail (that's fine and
        /// expected for most generated strings); when it succeeds, the
        /// translation call that follows must still never panic.
        #[test]
        fn prop_translation_never_panics_for_arbitrary_nested_source_fields_via_json(
            owner_raw in "[\\PC]{0,30}",
            name_raw in "[\\PC]{0,30}",
            preserve_history in any::<bool>(),
        ) {
            let owner_json = serde_json::to_string(&owner_raw).unwrap();
            let name_json = serde_json::to_string(&name_raw).unwrap();
            let json = format!(
                r#"{{
                    "organization": "myorg",
                    "name": "my-repo",
                    "creationMode": {{
                        "type": "mirror",
                        "source": {{ "owner": {owner_json}, "name": {name_json} }},
                        "preserveHistory": {preserve_history}
                    }}
                }}"#
            );

            if let Ok(http_req) = serde_json::from_str::<CreateRepositoryRequest>(&json) {
                let result = http_create_repository_request_to_domain(http_req, "actor".to_string());
                prop_assert!(result.is_ok() || result.is_err());
            }
        }
    }
}
