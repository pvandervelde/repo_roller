//! Tests for PushSettings
use super::*;
#[test]
fn test_default_creates_empty() {
    let settings = PushSettings::default();
    assert!(settings.max_branches_per_push.is_none());
}
