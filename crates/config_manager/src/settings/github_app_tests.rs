//! Tests for GitHubAppConfig
use super::*;
use std::collections::HashMap;
#[test]
fn test_github_app_creation() {
    let app = GitHubAppConfig {
        app_id: 12345,
        permissions: HashMap::new(),
    };
    assert_eq!(app.app_id, 12345);
}
