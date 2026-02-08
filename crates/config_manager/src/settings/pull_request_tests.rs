//! Tests for PullRequestSettings

use super::*;

#[test]
fn test_default_creates_empty() {
    let settings = PullRequestSettings::default();
    assert!(settings.allow_merge_commit.is_none());
}
