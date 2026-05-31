// Property-based tests for AccessLevel ordering.
//
// AccessLevel derives `Ord`/`PartialOrd`, so the ordering must satisfy
// the standard mathematical properties of a total order.

use super::AccessLevel;
use proptest::prelude::*;

/// Arbitrary `AccessLevel` strategy.
fn any_access_level() -> impl Strategy<Value = AccessLevel> {
    prop_oneof![
        Just(AccessLevel::None),
        Just(AccessLevel::Read),
        Just(AccessLevel::Triage),
        Just(AccessLevel::Write),
        Just(AccessLevel::Maintain),
        Just(AccessLevel::Admin),
    ]
}

proptest! {
    /// Ordering is reflexive: a ≤ a for every AccessLevel.
    #[test]
    fn prop_access_level_ordering_is_reflexive(
        a in any_access_level(),
    ) {
        prop_assert!(a <= a, "{:?} must satisfy a <= a", a);
    }

    /// Ordering is antisymmetric: if a ≤ b AND b ≤ a then a == b.
    #[test]
    fn prop_access_level_ordering_is_antisymmetric(
        a in any_access_level(),
        b in any_access_level(),
    ) {
        if a <= b && b <= a {
            prop_assert_eq!(a, b, "a <= b and b <= a must imply a == b");
        }
    }

    /// Ordering is transitive: if a ≤ b and b ≤ c then a ≤ c.
    #[test]
    fn prop_access_level_ordering_is_transitive(
        a in any_access_level(),
        b in any_access_level(),
        c in any_access_level(),
    ) {
        if a <= b && b <= c {
            prop_assert!(a <= c, "transitivity violated: {:?} <= {:?} <= {:?} but {:?} > {:?}", a, b, c, a, c);
        }
    }

    /// Ordering is total: for any pair (a, b), a ≤ b OR b ≤ a.
    #[test]
    fn prop_access_level_ordering_is_total(
        a in any_access_level(),
        b in any_access_level(),
    ) {
        prop_assert!(
            a <= b || b <= a,
            "total order violated: {:?} and {:?} are incomparable", a, b
        );
    }

    /// `None` is less than or equal to every other level (it is the minimum).
    #[test]
    fn prop_none_is_minimum_access_level(
        other in any_access_level(),
    ) {
        prop_assert!(
            AccessLevel::None <= other,
            "AccessLevel::None must be the minimum; {:?} is not >= None", other
        );
    }

    /// `Admin` is greater than or equal to every other level (it is the maximum).
    #[test]
    fn prop_admin_is_maximum_access_level(
        other in any_access_level(),
    ) {
        prop_assert!(
            other <= AccessLevel::Admin,
            "AccessLevel::Admin must be the maximum; {:?} is not <= Admin", other
        );
    }
}
