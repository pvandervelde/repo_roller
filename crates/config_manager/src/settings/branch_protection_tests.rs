//! Tests for BranchProtectionSettings
use super::*;
#[test]
fn test_default_creates_empty() {
    let settings = BranchProtectionSettings::default();
    assert!(settings.default_branch.is_none());
}
