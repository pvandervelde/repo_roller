//! Repository ruleset configuration settings.
//!
//! Defines ruleset configuration for TOML-based hierarchical configuration system.
//! Maps to GitHub repository ruleset types while being TOML-friendly.

use serde::{Deserialize, Serialize};

#[cfg(test)]
#[path = "ruleset_tests.rs"]
mod tests;

/// Repository ruleset configuration.
///
/// Defines a governance ruleset that applies to branches or tags.
///
/// # Examples
///
/// ```toml
/// [[rulesets]]
/// name = "main-protection"
/// target = "branch"
/// enforcement = "active"
///
/// [rulesets.conditions.ref_name]
/// include = ["refs/heads/main"]
///
/// [[rulesets.rules]]
/// type = "deletion"
///
/// [[rulesets.rules]]
/// type = "pull_request"
/// required_approving_review_count = 2
/// require_code_owner_review = true
/// allowed_merge_methods = ["squash"]
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RulesetConfig {
    /// Ruleset name
    pub name: String,

    /// Target type: "branch" or "tag"
    #[serde(default = "default_target")]
    pub target: String,

    /// Enforcement level: "active", "disabled", or "evaluate"
    #[serde(default = "default_enforcement")]
    pub enforcement: String,

    /// Actors who can bypass this ruleset
    #[serde(default)]
    pub bypass_actors: Vec<BypassActorConfig>,

    /// Conditions for when this ruleset applies
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditions: Option<RulesetConditionsConfig>,

    /// Rules in this ruleset
    pub rules: Vec<RuleConfig>,
}

fn default_target() -> String {
    "branch".to_string()
}

fn default_enforcement() -> String {
    "active".to_string()
}

/// Actor who can bypass a ruleset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BypassActorConfig {
    /// Actor ID
    pub actor_id: u64,

    /// Actor type: "OrganizationAdmin", "RepositoryRole", "Team", or "Integration"
    pub actor_type: String,

    /// Bypass mode: "always" or "pull_request"
    #[serde(default = "default_bypass_mode")]
    pub bypass_mode: String,
}

fn default_bypass_mode() -> String {
    "always".to_string()
}

/// Conditions for when a ruleset applies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RulesetConditionsConfig {
    /// Reference name patterns
    pub ref_name: RefNameConditionConfig,
}

/// Reference name condition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefNameConditionConfig {
    /// Patterns to include
    pub include: Vec<String>,

    /// Patterns to exclude
    #[serde(default)]
    pub exclude: Vec<String>,
}

/// A rule within a ruleset (TOML-friendly representation).
///
/// Uses untagged enum with discriminator field for TOML compatibility.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RuleConfig {
    /// Prevent creation of matching refs
    Creation,

    /// Prevent updates to matching refs
    Update,

    /// Prevent deletion of matching refs
    Deletion,

    /// Require linear history (no merge commits)
    RequiredLinearHistory,

    /// Require signed commits
    RequiredSignatures,

    /// Pull request requirements
    PullRequest {
        /// Dismiss stale reviews when new commits are pushed
        #[serde(skip_serializing_if = "Option::is_none")]
        dismiss_stale_reviews_on_push: Option<bool>,

        /// Require code owner review
        #[serde(skip_serializing_if = "Option::is_none")]
        require_code_owner_review: Option<bool>,

        /// Require last push approval
        #[serde(skip_serializing_if = "Option::is_none")]
        require_last_push_approval: Option<bool>,

        /// Required approving review count
        #[serde(skip_serializing_if = "Option::is_none")]
        required_approving_review_count: Option<u32>,

        /// Required review thread resolution
        #[serde(skip_serializing_if = "Option::is_none")]
        required_review_thread_resolution: Option<bool>,

        /// Allowed merge methods: "merge", "squash", "rebase"
        #[serde(skip_serializing_if = "Option::is_none")]
        allowed_merge_methods: Option<Vec<String>>,
    },

    /// Required status checks
    RequiredStatusChecks {
        /// Required status check contexts
        required_status_checks: Vec<StatusCheckConfig>,

        /// Require branches to be up to date before merging
        #[serde(skip_serializing_if = "Option::is_none")]
        strict_required_status_checks_policy: Option<bool>,
    },

    /// Non-fast-forward updates
    NonFastForward,
}

/// A required status check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusCheckConfig {
    /// Status check context
    pub context: String,

    /// Integration ID (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integration_id: Option<u64>,
}
