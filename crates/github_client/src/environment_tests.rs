//! Tests for GitHub environment detection types.

use super::*;

#[test]
fn test_plan_limitations_free_plan() {
    let limitations = PlanLimitations::free_plan();

    assert!(limitations.supports_private_repos); // Free plans DO support private repos
    assert!(!limitations.supports_internal_repos);
    assert_eq!(limitations.private_repo_limit, None); // No hard limit on private repo count
    assert!(!limitations.is_enterprise);
}

#[test]
fn test_plan_limitations_paid_plan() {
    let limitations = PlanLimitations::paid_plan();

    assert!(limitations.supports_private_repos);
    assert!(!limitations.supports_internal_repos);
    assert_eq!(limitations.private_repo_limit, None);
    assert!(!limitations.is_enterprise);
}

#[test]
fn test_plan_limitations_enterprise() {
    let limitations = PlanLimitations::enterprise();

    assert!(limitations.supports_private_repos);
    assert!(limitations.supports_internal_repos);
    assert_eq!(limitations.private_repo_limit, None);
    assert!(limitations.is_enterprise);
}

#[test]
fn test_plan_limitations_equality() {
    let plan1 = PlanLimitations::enterprise();
    let plan2 = PlanLimitations::enterprise();
    let plan3 = PlanLimitations::paid_plan();

    assert_eq!(plan1, plan2);
    assert_ne!(plan1, plan3);
}

#[test]
fn test_plan_limitations_clone() {
    let original = PlanLimitations::enterprise();
    let cloned = original.clone();

    assert_eq!(original, cloned);
}
