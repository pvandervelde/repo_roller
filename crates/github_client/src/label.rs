//! Label domain types.
//!
//! This module contains types representing GitHub issue and pull request labels.

use serde::{Deserialize, Serialize};

#[cfg(test)]
#[path = "label_tests.rs"]
mod tests;

/// Represents a label on a pull request or issue.
///
/// This struct contains the essential information about a label
/// that is needed for categorization and filtering.
///
/// # Fields
///
/// * `name` - The name of the label
///
/// # Examples
///
/// ```
/// use github_client::Label;
///
/// let label = Label {
///     name: "bug".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    /// The name of the label
    pub name: String,
}
