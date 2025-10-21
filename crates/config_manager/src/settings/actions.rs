//! GitHub Actions settings.
//!
//! Controls GitHub Actions permissions and behavior.

use crate::OverridableValue;
use serde::{Deserialize, Serialize};

/// GitHub Actions settings with override controls.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ActionSettings {
    /// Enable GitHub Actions
    pub enabled: Option<OverridableValue<bool>>,

    /// Actions permissions (all, local_only, selected)
    pub allowed_actions: Option<OverridableValue<String>>,

    /// Allow GitHub-owned actions
    pub github_owned_allowed: Option<OverridableValue<bool>>,

    /// Allow verified creator actions
    pub verified_allowed: Option<OverridableValue<bool>>,

    /// List of allowed action patterns
    pub patterns_allowed: Option<Vec<String>>,
}

#[cfg(test)]
#[path = "actions_tests.rs"]
mod tests;
