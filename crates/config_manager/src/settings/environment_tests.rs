//! Tests for EnvironmentConfig
use super::*;
#[test]
fn test_environment_creation() {
    let env = EnvironmentConfig {
        name: "production".to_string(),
        protection_rules: None,
        deployment_branch_policy: None,
    };
    assert_eq!(env.name, "production");
}
