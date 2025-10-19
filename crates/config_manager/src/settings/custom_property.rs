//! GitHub custom properties.

use serde::{Deserialize, Serialize};

/// GitHub custom property.
///
/// Represents a custom property that can be set on a repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomProperty {
    /// Property name
    pub property_name: String,

    /// Property value (can be various types)
    pub value: CustomPropertyValue,
}

/// Custom property value types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CustomPropertyValue {
    /// String value
    String(String),

    /// Single select value
    SingleSelect(String),

    /// Multi-select values
    MultiSelect(Vec<String>),

    /// Boolean value
    Boolean(bool),
}

#[cfg(test)]
#[path = "custom_property_tests.rs"]
mod tests;
