//! Repository feature settings.
//!
//! Controls which GitHub repository features are enabled.

use crate::OverridableValue;
use serde::{Deserialize, Serialize};

/// Repository feature settings with override controls.
///
/// Each field represents a GitHub repository feature that can be enabled or disabled.
/// When used in GlobalDefaults, each field is wrapped in `OverridableValue` to control
/// whether teams and templates can override the setting.
///
/// # Examples
///
/// ```rust
/// use config_manager::{OverridableValue, settings::RepositorySettings};
///
/// let settings = RepositorySettings {
///     issues: Some(OverridableValue::allowed(true)),
///     wiki: Some(OverridableValue::fixed(false)), // Policy: no wikis
///     ..Default::default()
///};
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RepositorySettings {
    /// Enable issue tracking
    pub issues: Option<OverridableValue<bool>>,

    /// Enable project boards
    pub projects: Option<OverridableValue<bool>>,

    /// Enable discussions
    pub discussions: Option<OverridableValue<bool>>,

    /// Enable wiki
    pub wiki: Option<OverridableValue<bool>>,

    /// Enable GitHub Pages
    pub pages: Option<OverridableValue<bool>>,

    /// Enable security advisories
    pub security_advisories: Option<OverridableValue<bool>>,

    /// Enable private vulnerability reporting
    pub vulnerability_reporting: Option<OverridableValue<bool>>,

    /// Automatically close stale issues
    pub auto_close_issues: Option<OverridableValue<bool>>,
}

#[cfg(test)]
#[path = "repository_tests.rs"]
mod tests;
