//! Tests for ActionSettings
use super::*;
#[test]
fn test_default_creates_empty() {
    let settings = ActionSettings::default();
    assert!(settings.enabled.is_none());
}
