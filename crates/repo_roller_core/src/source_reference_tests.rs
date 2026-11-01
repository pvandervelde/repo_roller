//! Tests for `SourceRepositoryReference`.
//!
//! See docs/spec/interfaces/fork-mirror-operations.md §3 and RCA-004/005/006.
//! No new validation rules are introduced — `SourceRepositoryReference`
//! delegates entirely to the existing `OrganizationName`/`RepositoryName`
//! branded-type validation (lines 89-139 of this file).

use super::*;

// ============================================================================
// Tier 1: Specification tests
// ============================================================================

#[test]
fn test_source_repository_reference_constructs_with_valid_owner_and_name() {
    let owner = OrganizationName::new("upstream-org").expect("valid owner");
    let name = RepositoryName::new("upstream-repo").expect("valid name");

    let reference = SourceRepositoryReference {
        owner: owner.clone(),
        name: name.clone(),
    };

    assert_eq!(reference.owner, owner);
    assert_eq!(reference.name, name);
}

/// `SourceRepositoryReference.owner` may reference an organization outside
/// the one RepoRoller manages (e.g. the upstream of a fork) — the same
/// `OrganizationName` validation rules apply regardless of which
/// organization is being referenced.
#[test]
fn test_source_repository_reference_accepts_an_owner_outside_the_managed_organization() {
    let reference = SourceRepositoryReference {
        owner: OrganizationName::new("some-other-upstream-org").expect("valid"),
        name: RepositoryName::new("their-repo").expect("valid"),
    };

    assert_eq!(reference.owner.as_str(), "some-other-upstream-org");
}

#[test]
fn test_source_repository_reference_is_cloneable_and_comparable() {
    let reference = SourceRepositoryReference {
        owner: OrganizationName::new("acme").unwrap(),
        name: RepositoryName::new("widgets").unwrap(),
    };
    let cloned = reference.clone();
    assert_eq!(reference, cloned);
}

#[test]
fn test_source_repository_reference_serde_roundtrip() {
    let reference = SourceRepositoryReference {
        owner: OrganizationName::new("acme").unwrap(),
        name: RepositoryName::new("widgets").unwrap(),
    };

    let json = serde_json::to_string(&reference).expect("serialize");
    let roundtripped: SourceRepositoryReference = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(reference, roundtripped);
}

// ============================================================================
// Tier 2: Adversarial — validation delegation
// ============================================================================

/// Stub-killing: `SourceRepositoryReference` must not introduce a parallel,
/// looser validation path — invalid organization names must still be
/// rejected by `OrganizationName::new` before a `SourceRepositoryReference`
/// can ever be constructed.
#[test]
fn test_source_repository_reference_owner_rejects_invalid_organization_name() {
    let result = OrganizationName::new("-leading-hyphen");
    assert!(result.is_err());
}

#[test]
fn test_source_repository_reference_name_rejects_invalid_repository_name() {
    let result = RepositoryName::new(".leading-dot");
    assert!(result.is_err());
}

#[test]
fn test_source_repository_reference_distinguishes_owner_from_name() {
    let a = SourceRepositoryReference {
        owner: OrganizationName::new("acme").unwrap(),
        name: RepositoryName::new("widgets").unwrap(),
    };
    let b = SourceRepositoryReference {
        owner: OrganizationName::new("widgets").unwrap(),
        name: RepositoryName::new("acme").unwrap(),
    };

    assert_ne!(
        a, b,
        "swapping owner and name must produce a distinct reference"
    );
}
