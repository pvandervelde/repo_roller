//! Repository ruleset domain types.
//!
//! This module contains types representing GitHub repository rulesets and their rules.
//! Rulesets provide a way to enforce repository governance policies on branches and tags.
//!
//! See: https://docs.github.com/en/rest/repos/rules

use serde::{Deserialize, Serialize};

#[cfg(test)]
#[path = "ruleset_tests.rs"]
mod tests;

/// Represents a repository ruleset.
///
/// Rulesets define governance rules that apply to branches or tags in a repository.
///
/// # Examples
///
/// ```rust
/// use github_client::{RepositoryRuleset, RulesetTarget, RulesetEnforcement};
///
/// let ruleset = RepositoryRuleset {
///     id: None,
///     name: "main-protection".to_string(),
///     target: RulesetTarget::Branch,
///     enforcement: RulesetEnforcement::Active,
///     bypass_actors: vec![],
///     conditions: None,
///     rules: vec![],
///     node_id: None,
///     source: None,
///     source_type: None,
///     created_at: None,
///     updated_at: None,
///     _links: None,
/// };
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RepositoryRuleset {
    /// Ruleset ID (None for creation, Some for updates)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,

    /// Ruleset name
    pub name: String,

    /// Target type (branch or tag)
    pub target: RulesetTarget,

    /// Enforcement level
    pub enforcement: RulesetEnforcement,

    /// Actors who can bypass this ruleset
    #[serde(default)]
    pub bypass_actors: Vec<BypassActor>,

    /// Conditions for when this ruleset applies
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditions: Option<RulesetConditions>,

    /// Rules in this ruleset
    ///
    /// Note: GitHub's LIST rulesets endpoint does not include rules in the response.
    /// Use GET /repos/{owner}/{repo}/rules/{ruleset_id} to fetch full ruleset details.
    #[serde(default)]
    pub rules: Vec<Rule>,

    /// Node ID (GitHub's global node identifier)
    /// Returned by LIST and GET endpoints, not used for CREATE
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,

    /// Source of the ruleset (repository or organization name)
    /// Example: "owner/repo" or "org-name"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// Type of source (Repository or Organization)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_type: Option<String>,

    /// Timestamp when ruleset was created (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,

    /// Timestamp when ruleset was last updated (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,

    /// Links to related resources
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _links: Option<serde_json::Value>,
}

/// Target type for a ruleset.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RulesetTarget {
    /// Ruleset applies to branches
    Branch,
    /// Ruleset applies to tags
    Tag,
    /// Ruleset applies to pushes
    Push,
}

/// Enforcement level for a ruleset.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RulesetEnforcement {
    /// Ruleset is disabled
    Disabled,
    /// Ruleset is active and enforced
    Active,
    /// Ruleset is in evaluation mode (logs only, doesn't block)
    Evaluate,
}

/// Actor who can bypass a ruleset.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BypassActor {
    /// Actor ID
    pub actor_id: u64,

    /// Actor type
    pub actor_type: BypassActorType,

    /// Bypass mode
    pub bypass_mode: BypassMode,
}

/// Type of actor that can bypass a ruleset.
///
/// # Repository Roles
///
/// The `RepositoryRole` variant encompasses multiple specific roles:
/// - Repository admin
/// - Maintain role
/// - Write role
///
/// The specific role is determined by the `actor_id` field, which references
/// the role ID in GitHub's role system.
///
/// # Special Bypass Modes
///
/// - Use `Team` with organization-level team IDs to allow specific teams
/// - Use `Integration` for GitHub Apps
/// - Deploy keys are represented via `DeployKey`
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum BypassActorType {
    /// Organization admin role
    OrganizationAdmin,
    /// Repository-level role (admin, maintain, write)
    /// The actor_id specifies which repository role
    RepositoryRole,
    /// Team (use team ID as actor_id)
    Team,
    /// Integration (GitHub App)
    Integration,
    /// Deploy key
    DeployKey,
}

/// Mode for bypassing a ruleset.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BypassMode {
    /// Always allow bypass
    Always,
    /// Require pull request
    #[serde(rename = "pull_request")]
    PullRequest,
}

/// Conditions for when a ruleset applies.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RulesetConditions {
    /// Reference name patterns
    pub ref_name: RefNameCondition,
}

/// Reference name condition.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RefNameCondition {
    /// Patterns to include
    pub include: Vec<String>,

    /// Patterns to exclude
    #[serde(default)]
    pub exclude: Vec<String>,
}

/// A rule within a ruleset.
///
/// Rules define specific requirements or restrictions.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Rule {
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
        /// Pull request parameters
        parameters: PullRequestParameters,
    },

    /// Required status checks
    RequiredStatusChecks {
        /// Required status check parameters
        parameters: RequiredStatusChecksParameters,
    },

    /// Non-fast-forward updates
    NonFastForward,
}

/// Parameters for pull request rules.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PullRequestParameters {
    /// Dismiss stale reviews when new commits are pushed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dismiss_stale_reviews_on_push: Option<bool>,

    /// Require code owner review
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_code_owner_review: Option<bool>,

    /// Require last push approval
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_last_push_approval: Option<bool>,

    /// Required approving review count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_approving_review_count: Option<u32>,

    /// Required review thread resolution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_review_thread_resolution: Option<bool>,

    /// Allowed merge methods
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_merge_methods: Option<Vec<MergeMethod>>,
}

/// Parameters for required status checks.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequiredStatusChecksParameters {
    /// Required status checks
    pub required_status_checks: Vec<StatusCheck>,

    /// Require branches to be up to date before merging
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict_required_status_checks_policy: Option<bool>,
}

/// A required status check.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StatusCheck {
    /// Status check context
    pub context: String,

    /// Integration ID (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integration_id: Option<u64>,
}

/// Allowed merge methods for pull requests.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MergeMethod {
    /// Merge commit
    Merge,
    /// Squash merge
    Squash,
    /// Rebase merge
    Rebase,
}
