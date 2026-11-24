//! Property-based tests for `validate_fork_target_name_matches_source`.
//!
//! Remediates a CRITICAL security finding on top of task #282: GitHub's
//! fork-creation API cannot rename a repository on creation, so a
//! `CreationMode::Fork` request whose `request.name` differs from
//! `source.name` would cause the task #282 existence-check gate
//! (`ensure_repository_does_not_exist`) to check the wrong identifier.
//!
//! This file verifies the core invariant across arbitrary valid
//! owner/name/source-name combinations: for `CreationMode::Fork`, the
//! function returns `Ok` if and only if `request.name == source.name`,
//! regardless of what either string actually is (only hand-picked fixture
//! values in `lib_tests.rs` could coincidentally satisfy a hardcoded or
//! partially-wired implementation; arbitrary generated values cannot).

use super::*;
use proptest::prelude::*;

/// Generates strings that satisfy `OrganizationName`'s validation rules:
/// 1-39 chars, alphanumeric + hyphen, no leading/trailing/double hyphen.
fn valid_org_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9]{0,8}(-[a-z0-9]{1,8}){0,3}"
}

/// Generates strings that satisfy `RepositoryName`'s validation rules:
/// 1-100 chars, alphanumeric/hyphen/underscore/period, no leading '.' or '-'.
fn valid_repo_name_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9._-]{0,20}"
}

proptest! {
    /// For any valid owner/name/source-name combination, a `Fork`-mode
    /// request is accepted (`Ok`) if and only if `request.name` and
    /// `source.name` are exactly equal; otherwise it is rejected with
    /// `ForkTargetNameMismatch` echoing exactly those two strings.
    #[test]
    fn prop_validate_fork_target_name_matches_source_ok_iff_names_match(
        request_owner_str in valid_org_name_strategy(),
        source_owner_str in valid_org_name_strategy(),
        request_name_str in valid_repo_name_strategy(),
        source_name_str in valid_repo_name_strategy(),
    ) {
        let request = RepositoryCreationRequestBuilder::new(
            RepositoryName::new(&request_name_str).expect("generated request name must be valid"),
            OrganizationName::new(&request_owner_str).expect("generated request owner must be valid"),
        )
        .creation_mode(CreationMode::Fork {
            source: SourceRepositoryReference {
                owner: OrganizationName::new(&source_owner_str)
                    .expect("generated source owner must be valid"),
                name: RepositoryName::new(&source_name_str)
                    .expect("generated source name must be valid"),
            },
        })
        .build();

        let result = validate_fork_target_name_matches_source(&request);

        if request_name_str == source_name_str {
            prop_assert!(
                result.is_ok(),
                "matching names must be accepted, got {:?}",
                result
            );
        } else {
            match result {
                Err(RepoRollerError::Configuration(ConfigurationError::ForkTargetNameMismatch {
                    requested,
                    source,
                })) => {
                    prop_assert_eq!(requested, request_name_str.clone());
                    prop_assert_eq!(source, source_name_str.clone());
                }
                other => prop_assert!(
                    false,
                    "expected ForkTargetNameMismatch for mismatched names, got {:?}",
                    other
                ),
            }
        }
    }

    /// Non-Fork modes (`Standard`, both `Mirror` variants) must always
    /// return `Ok` for this guard, regardless of any relationship between
    /// `request.name` and a source name — this guard is Fork-only.
    #[test]
    fn prop_validate_fork_target_name_matches_source_mirror_mode_always_ok(
        request_owner_str in valid_org_name_strategy(),
        source_owner_str in valid_org_name_strategy(),
        request_name_str in valid_repo_name_strategy(),
        source_name_str in valid_repo_name_strategy(),
        preserve_history in proptest::bool::ANY,
    ) {
        let request = RepositoryCreationRequestBuilder::new(
            RepositoryName::new(&request_name_str).expect("generated request name must be valid"),
            OrganizationName::new(&request_owner_str).expect("generated request owner must be valid"),
        )
        .creation_mode(CreationMode::Mirror {
            source: SourceRepositoryReference {
                owner: OrganizationName::new(&source_owner_str)
                    .expect("generated source owner must be valid"),
                name: RepositoryName::new(&source_name_str)
                    .expect("generated source name must be valid"),
            },
            preserve_history,
        })
        .build();

        let result = validate_fork_target_name_matches_source(&request);

        prop_assert!(
            result.is_ok(),
            "Mirror mode must always bypass this Fork-only guard, got {:?}",
            result
        );
    }
}
