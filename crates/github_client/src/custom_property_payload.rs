//! GitHub custom property API payload types.
//!
//! This module defines the payload structures for GitHub's custom properties API.
//! The GitHub API uses a different format than our internal `CustomProperty` type,
//! so we need to transform between our domain type and GitHub's expected format.
//!
//! See: https://docs.github.com/en/rest/repos/custom-properties

use serde::{Deserialize, Serialize};

/// Payload for updating repository custom properties via GitHub API.
///
/// This struct represents the expected format for the
/// `PATCH /repos/{owner}/{repo}/custom-properties` endpoint.
///
/// # Examples
///
/// ```
/// use github_client::CustomPropertiesPayload;
/// use serde_json::json;
///
/// let payload = CustomPropertiesPayload {
///     properties: vec![
///         json!({
///             "property_name": "repository_type",
///             "value": "library"
///         })
///     ],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomPropertiesPayload {
    /// Array of custom property values to set on the repository.
    ///
    /// Each property must include `property_name` and `value` fields.
    /// The property definition must already exist at the organization level.
    pub properties: Vec<serde_json::Value>,
}

impl CustomPropertiesPayload {
    /// Create a new custom properties payload from a list of properties.
    ///
    /// # Arguments
    ///
    /// * `properties` - Vector of JSON values representing custom properties
    ///
    /// # Examples
    ///
    /// ```
    /// use github_client::CustomPropertiesPayload;
    /// use serde_json::json;
    ///
    /// let properties = vec![
    ///     json!({
    ///         "property_name": "environment",
    ///         "value": "production"
    ///     })
    /// ];
    ///
    /// let payload = CustomPropertiesPayload::new(properties);
    /// assert_eq!(payload.properties.len(), 1);
    /// ```
    pub fn new(properties: Vec<serde_json::Value>) -> Self {
        Self { properties }
    }
}

#[cfg(test)]
#[path = "custom_property_payload_tests.rs"]
mod tests;
