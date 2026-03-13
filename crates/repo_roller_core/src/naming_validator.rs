//! Repository naming rules validation.
//!
//! This module provides [`RepositoryNamingValidator`], a stateless component
//! that validates a repository name against a set of
//! [`RepositoryNamingRulesConfig`] entries produced by the configuration
//! merger.
//!
//! # Validation semantics
//!
//! All rules in the collection are applied in order.  The first rule that
//! fails stops evaluation and returns an error describing which constraint
//! was violated. An empty rule collection is always valid.
//!
//! # Examples
//!
//! ```rust
//! use repo_roller_core::RepositoryNamingValidator;
//! use config_manager::RepositoryNamingRulesConfig;
//!
//! let validator = RepositoryNamingValidator::new();
//!
//! // A simple prefix rule
//! let rules = vec![RepositoryNamingRulesConfig {
//!     required_prefix: Some("acme-".to_string()),
//!     ..Default::default()
//! }];
//!
//! // Valid name
//! assert!(validator.validate("acme-payments", &rules).is_ok());
//!
//! // Invalid name
//! assert!(validator.validate("payments", &rules).is_err());
//! ```

use config_manager::RepositoryNamingRulesConfig;
use regex::Regex;
use tracing::{debug, warn};

use crate::errors::ValidationError;

#[cfg(test)]
#[path = "naming_validator_tests.rs"]
mod tests;

/// Stateless validator for repository naming rules.
///
/// Validates a repository name against a list of `RepositoryNamingRulesConfig`
/// entries.  All rules must be satisfied; the first violation returns an error.
///
/// # Thread safety
///
/// `RepositoryNamingValidator` is a zero-sized type (`Copy + Clone`) and is
/// safe to share across threads.
#[derive(Debug, Clone, Copy, Default)]
pub struct RepositoryNamingValidator;

impl RepositoryNamingValidator {
    /// Creates a new `RepositoryNamingValidator`.
    pub fn new() -> Self {
        Self
    }

    /// Validates `name` against every rule in `rules`.
    ///
    /// Returns `Ok(())` when the name satisfies all rules, or
    /// `Err(ValidationError)` describing the first rule violation.
    ///
    /// # Arguments
    ///
    /// * `name`  – The proposed repository name (already stripped of org prefix).
    /// * `rules` – Slice of naming-rule configs produced by the configuration merger.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::InvalidRepositoryName`] when any rule is violated.
    /// The error message includes the rule description (if set) and the specific
    /// constraint that was broken.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use repo_roller_core::RepositoryNamingValidator;
    /// use config_manager::RepositoryNamingRulesConfig;
    ///
    /// let validator = RepositoryNamingValidator::new();
    /// let rules = vec![RepositoryNamingRulesConfig {
    ///     reserved_words: vec!["test".to_string()],
    ///     ..Default::default()
    /// }];
    ///
    /// assert!(validator.validate("my-service", &rules).is_ok());
    /// assert!(validator.validate("test", &rules).is_err());
    /// ```
    pub fn validate(
        &self,
        name: &str,
        rules: &[RepositoryNamingRulesConfig],
    ) -> Result<(), ValidationError> {
        for rule in rules {
            self.apply_rule(name, rule)?;
        }
        Ok(())
    }

    /// Applies a single rule, returning an error on the first violation.
    fn apply_rule(
        &self,
        name: &str,
        rule: &RepositoryNamingRulesConfig,
    ) -> Result<(), ValidationError> {
        let rule_desc = rule
            .description
            .as_deref()
            .unwrap_or("repository naming rule");

        // --- Contradictory length constraints ---------------------------------
        // Detect misconfigured rules (min > max) at validation time rather than
        // silently rejecting every name with a misleading error message.
        if let (Some(min), Some(max)) = (rule.min_length, rule.max_length) {
            if min > max {
                warn!(
                    min = min,
                    max = max,
                    rule = rule_desc,
                    "Naming rule has min_length > max_length; this configuration is invalid"
                );
                return Err(ValidationError::InvalidRepositoryName {
                    reason: format!(
                        "Naming rule '{rule_desc}' is misconfigured: \
                         min_length ({min}) is greater than max_length ({max})"
                    ),
                });
            }
        }

        // --- Length constraints -----------------------------------------------
        if let Some(min) = rule.min_length {
            if name.len() < min {
                debug!(
                    name = name,
                    min = min,
                    len = name.len(),
                    rule = rule_desc,
                    "Repository name too short"
                );
                return Err(ValidationError::InvalidRepositoryName {
                    reason: format!(
                        "Name '{name}' is too short ({} chars); \
                         minimum length is {min} (rule: {rule_desc})",
                        name.len()
                    ),
                });
            }
        }

        if let Some(max) = rule.max_length {
            if name.len() > max {
                debug!(
                    name = name,
                    max = max,
                    len = name.len(),
                    rule = rule_desc,
                    "Repository name too long"
                );
                return Err(ValidationError::InvalidRepositoryName {
                    reason: format!(
                        "Name '{name}' is too long ({} chars); \
                         maximum length is {max} (rule: {rule_desc})",
                        name.len()
                    ),
                });
            }
        }

        // --- Prefix / suffix constraints --------------------------------------
        if let Some(prefix) = &rule.required_prefix {
            if !name.starts_with(prefix.as_str()) {
                debug!(
                    name = name,
                    required_prefix = prefix,
                    rule = rule_desc,
                    "Repository name missing required prefix"
                );
                return Err(ValidationError::InvalidRepositoryName {
                    reason: format!("Name '{name}' must start with '{prefix}' (rule: {rule_desc})"),
                });
            }
        }

        if let Some(suffix) = &rule.required_suffix {
            if !name.ends_with(suffix.as_str()) {
                debug!(
                    name = name,
                    required_suffix = suffix,
                    rule = rule_desc,
                    "Repository name missing required suffix"
                );
                return Err(ValidationError::InvalidRepositoryName {
                    reason: format!("Name '{name}' must end with '{suffix}' (rule: {rule_desc})"),
                });
            }
        }

        // --- Reserved words (case-insensitive exact match) --------------------
        let name_lower = name.to_lowercase();
        for word in &rule.reserved_words {
            if name_lower == word.to_lowercase() {
                debug!(
                    name = name,
                    reserved_word = word,
                    rule = rule_desc,
                    "Repository name is a reserved word"
                );
                return Err(ValidationError::InvalidRepositoryName {
                    reason: format!("Name '{name}' is a reserved word (rule: {rule_desc})"),
                });
            }
        }

        // --- Allowed pattern (whole-name match) -------------------------------
        // TODO: regex patterns are recompiled on every validation call. For the
        // current usage (one call per repository creation) this is acceptable,
        // but a pre-compiled cache or RegexSet should be considered if bulk
        // validation is ever required.
        if let Some(pattern) = &rule.allowed_pattern {
            // Anchor the match so the pattern must cover the full name.
            let anchored = format!("^(?:{pattern})$");
            let re_anchored = Regex::new(&anchored).map_err(|e| {
                warn!(
                    pattern = pattern,
                    rule = rule_desc,
                    error = %e,
                    "allowed_pattern contains an invalid regex; treating as validation failure"
                );
                ValidationError::InvalidRepositoryName {
                    reason: format!(
                        "Invalid allowed_pattern regex '{pattern}' in rule '{rule_desc}': {e}"
                    ),
                }
            })?;
            if !re_anchored.is_match(name) {
                debug!(
                    name = name,
                    pattern = pattern,
                    rule = rule_desc,
                    "Repository name does not match allowed pattern"
                );
                return Err(ValidationError::InvalidRepositoryName {
                    reason: format!(
                        "Name '{name}' does not match required pattern '{pattern}' (rule: {rule_desc})"
                    ),
                });
            }
        }

        // --- Forbidden patterns -----------------------------------------------
        for pattern in &rule.forbidden_patterns {
            let re = Regex::new(pattern).map_err(|e| {
                warn!(
                    pattern = pattern,
                    rule = rule_desc,
                    error = %e,
                    "forbidden_pattern contains an invalid regex; treating as validation failure"
                );
                ValidationError::InvalidRepositoryName {
                    reason: format!(
                        "Invalid forbidden_pattern regex '{pattern}' in rule '{rule_desc}': {e}"
                    ),
                }
            })?;
            if re.is_match(name) {
                debug!(
                    name = name,
                    pattern = pattern,
                    rule = rule_desc,
                    "Repository name matches forbidden pattern"
                );
                return Err(ValidationError::InvalidRepositoryName {
                    reason: format!(
                        "Name '{name}' matches forbidden pattern '{pattern}' (rule: {rule_desc})"
                    ),
                });
            }
        }

        Ok(())
    }
}
