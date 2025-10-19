//! GitHub label configuration.

use serde::{Deserialize, Serialize};

/// GitHub label configuration.
///
/// Defines a label that can be applied to issues and pull requests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelConfig {
    /// Label name.
    pub name: String,

    /// Label color (hex code without #).
    pub color: String,

    /// Label description.
    pub description: String,
}

#[cfg(test)]
#[path = "label_tests.rs"]
mod tests;
