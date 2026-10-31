//! Property-based tests for `SourceRepositoryReference` and
//! `CreationMode`-driven builder construction.
//!
//! See docs/spec/interfaces/fork-mirror-operations.md §3-4.

use super::*;
use crate::SourceRepositoryReference;
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
    /// For any pair of strings that independently satisfy the
    /// `OrganizationName`/`RepositoryName` grammar, constructing a
    /// `SourceRepositoryReference` never fails, and serializing then
    /// deserializing it always reproduces an equal value.
    #[test]
    fn prop_source_repository_reference_roundtrips_for_any_valid_owner_and_name(
        owner_str in valid_org_name_strategy(),
        name_str in valid_repo_name_strategy(),
    ) {
        let owner = OrganizationName::new(&owner_str).expect("generated owner must be valid");
        let name = RepositoryName::new(&name_str).expect("generated name must be valid");

        let reference = SourceRepositoryReference { owner, name };

        let json = serde_json::to_string(&reference).expect("serialize");
        let roundtripped: SourceRepositoryReference =
            serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(reference, roundtripped);
    }

    /// For any valid source owner/name pair, building a request with
    /// `CreationMode::Fork` and no explicit `content_strategy`/`template`
    /// never panics — the builder must treat Fork/Mirror as independent of
    /// the Standard-mode "Template strategy requires a template" rule for
    /// every possible source reference, not just hand-picked examples.
    #[test]
    fn prop_fork_mode_builder_never_panics_regardless_of_source_reference(
        owner_str in valid_org_name_strategy(),
        name_str in valid_repo_name_strategy(),
    ) {
        let source = SourceRepositoryReference {
            owner: OrganizationName::new(&owner_str).unwrap(),
            name: RepositoryName::new(&name_str).unwrap(),
        };

        let request = RepositoryCreationRequestBuilder::new(
            RepositoryName::new("target-repo").unwrap(),
            OrganizationName::new("target-org").unwrap(),
        )
        .creation_mode(CreationMode::Fork { source: source.clone() })
        .build();

        prop_assert_eq!(request.creation_mode, CreationMode::Fork { source });
    }

    /// Symmetric property for the source+history mirror mode.
    #[test]
    fn prop_history_mirror_mode_builder_never_panics_regardless_of_source_reference(
        owner_str in valid_org_name_strategy(),
        name_str in valid_repo_name_strategy(),
    ) {
        let source = SourceRepositoryReference {
            owner: OrganizationName::new(&owner_str).unwrap(),
            name: RepositoryName::new(&name_str).unwrap(),
        };

        let request = RepositoryCreationRequestBuilder::new(
            RepositoryName::new("target-repo").unwrap(),
            OrganizationName::new("target-org").unwrap(),
        )
        .creation_mode(CreationMode::Mirror { source: source.clone(), preserve_history: true })
        .build();

        prop_assert_eq!(
            request.creation_mode,
            CreationMode::Mirror { source, preserve_history: true }
        );
    }
}
