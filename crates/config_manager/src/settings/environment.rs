//! Environment configuration.

use serde::{Deserialize, Serialize};

/// Environment configuration.
///
/// Defines a deployment environment with protection rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Environment name
    pub name: String,

    /// Protection rules for the environment
    pub protection_rules: Option<EnvironmentProtectionRules>,

    /// Deployment branch policy
    pub deployment_branch_policy: Option<DeploymentBranchPolicy>,
}

/// Environment protection rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvironmentProtectionRules {
    /// Required reviewers (user/team names)
    pub required_reviewers: Option<Vec<String>>,

    /// Wait timer in minutes before deployment can proceed
    pub wait_timer: Option<i32>,
}

/// Deployment branch policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeploymentBranchPolicy {
    /// Whether to use protected branches only
    pub protected_branches: bool,

    /// Custom branch patterns (if not using protected branches)
    pub custom_branch_patterns: Option<Vec<String>>,
}

#[cfg(test)]
#[path = "environment_tests.rs"]
mod tests;
