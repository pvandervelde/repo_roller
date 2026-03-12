//! Repository naming rules configuration.
//!
//! Defines the [`RepositoryNamingRulesConfig`] type for specifying constraints
//! on repository names at any level of the configuration hierarchy (organization,
//! repository type, team, or template).
//!
//! # Merge Behaviour
//!
//! Naming rules from all configuration levels are collected **additively** —
//! every rule from global, repository-type, team, and template levels is
//! included in the final [`MergedConfiguration`]. A repository name must
//! satisfy **all** rules in the merged set.
//!
//! # TOML Format
//!
//! Each entry under `[[naming_rules]]` is one set of constraints.  All fields
//! are optional; an entry with no fields is a no-op.
//!
//! ```toml
//! # Enforced at org level — all repos must start with the org prefix
//! [[naming_rules]]
//! description     = "All repositories must use the org prefix"
//! required_prefix = "acme-"
//!
//! # Enforced at repo-type level — service repos need a suffix
//! [[naming_rules]]
//! description     = "Service repositories must end with -svc"
//! required_suffix = "-svc"
//! allowed_pattern = "^[a-z][a-z0-9-]*-svc$"
//!
//! # Prevent use of reserved placeholders
//! [[naming_rules]]
//! description    = "Reserved words must not be used"
//! reserved_words = ["test", "demo", "temp", "tmp"]
//!
//! # Length constraints
//! [[naming_rules]]
//! description = "Repository names must be between 5 and 40 characters"
//! min_length  = 5
//! max_length  = 40
//! ```
//!
//! [`MergedConfiguration`]: crate::merged_config::MergedConfiguration

use serde::{Deserialize, Serialize};

#[cfg(test)]
#[path = "naming_tests.rs"]
mod tests;

/// A single set of naming constraints that a repository name must satisfy.
///
/// All fields are optional.  Any field that is `None` (or an empty `Vec`) is
/// skipped during validation.  When multiple rules are present across
/// configuration levels, **every** rule must be satisfied.
///
/// # Examples
///
/// Require a prefix and validate the overall pattern:
///
/// ```rust
/// use config_manager::settings::RepositoryNamingRulesConfig;
///
/// let rule = RepositoryNamingRulesConfig {
///     description: Some("Services must use the svc- prefix".to_string()),
///     required_prefix: Some("svc-".to_string()),
///     allowed_pattern: Some(r"^svc-[a-z][a-z0-9-]*$".to_string()),
///     ..Default::default()
/// };
///
/// assert_eq!(rule.required_prefix.as_deref(), Some("svc-"));
/// ```
///
/// Forbid specific reserved words:
///
/// ```rust
/// use config_manager::settings::RepositoryNamingRulesConfig;
///
/// let rule = RepositoryNamingRulesConfig {
///     description: Some("Prevent generic placeholder names".to_string()),
///     reserved_words: vec!["test".to_string(), "demo".to_string(), "temp".to_string()],
///     ..Default::default()
/// };
///
/// assert_eq!(rule.reserved_words.len(), 3);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RepositoryNamingRulesConfig {
    /// Human-readable description of this rule set.
    ///
    /// Used in error messages to explain why a repository name is invalid.
    /// Should be written as a positive statement of what is expected, e.g.
    /// "All repositories must start with the team prefix".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Regex pattern that the full repository name must match.
    ///
    /// The pattern is applied to the **entire** name (implicitly anchored).
    /// Use standard regex syntax; the `regex` crate is used for evaluation.
    ///
    /// # Examples
    ///
    /// ```toml
    /// allowed_pattern = "^[a-z][a-z0-9-]*$"
    /// ```
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_pattern: Option<String>,

    /// Regex patterns that the repository name must **not** match.
    ///
    /// If the name matches any of these patterns the name is considered
    /// invalid.  Each entry is an independent pattern.
    ///
    /// # Examples
    ///
    /// ```toml
    /// forbidden_patterns = [".*--.*", ".*__.*"]
    /// ```
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub forbidden_patterns: Vec<String>,

    /// Exact strings that cannot be used as the full repository name.
    ///
    /// Comparison is **case-insensitive**.
    ///
    /// # Examples
    ///
    /// ```toml
    /// reserved_words = ["test", "demo", "temp", "tmp", "scratch"]
    /// ```
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reserved_words: Vec<String>,

    /// Required prefix for the repository name.
    ///
    /// If set, the repository name must begin with this exact string
    /// (case-sensitive).
    ///
    /// # Examples
    ///
    /// ```toml
    /// required_prefix = "acme-"
    /// ```
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_prefix: Option<String>,

    /// Required suffix for the repository name.
    ///
    /// If set, the repository name must end with this exact string
    /// (case-sensitive).
    ///
    /// # Examples
    ///
    /// ```toml
    /// required_suffix = "-lib"
    /// ```
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_suffix: Option<String>,

    /// Minimum allowed length for the repository name.
    ///
    /// Must be at least 1.  GitHub enforces a hard minimum of 1 character.
    /// If both `min_length` and `max_length` are set, `min_length` must be
    /// less than or equal to `max_length`.
    ///
    /// # Examples
    ///
    /// ```toml
    /// min_length = 5
    /// ```
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,

    /// Maximum allowed length for the repository name.
    ///
    /// Must be at most 100 (GitHub's hard limit).  If both `min_length` and
    /// `max_length` are set, `max_length` must be greater than or equal to
    /// `min_length`.
    ///
    /// # Examples
    ///
    /// ```toml
    /// max_length = 40
    /// ```
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
}
