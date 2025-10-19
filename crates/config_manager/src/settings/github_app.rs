//! GitHub App configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// GitHub App configuration.
///
/// Defines a GitHub App that should be installed on the repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubAppConfig {
    /// GitHub App ID
    pub app_id: u64,

    /// Permissions required for this app
    pub permissions: HashMap<String, String>,
}

#[cfg(test)]
#[path = "github_app_tests.rs"]
mod tests;
