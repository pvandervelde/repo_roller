//! GitHub label configuration.

use serde::{Deserialize, Serialize};

/// GitHub label configuration.
///
/// Defines a label that can be applied to issues and pull requests.
///
/// When deserializing from TOML where labels are defined as a map
/// (e.g., `[bug]` followed by `color` and `description`), the `name`
/// field should be set from the map key after deserialization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelConfig {
    /// Label name.
    /// 
    /// When deserializing from a TOML map structure, this should be
    /// populated from the map key, not from a TOML field.
    #[serde(default)]
    pub name: String,

    /// Label color (hex code without #).
    pub color: String,

    /// Label description.
    pub description: String,
}

#[cfg(test)]
#[path = "label_tests.rs"]
mod tests;
