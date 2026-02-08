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

    /// Target type: "branch", "tag", or "push"
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
///
/// # Actor Types
///
/// - `OrganizationAdmin`: Organization administrators
/// - `RepositoryRole`: Repository-level roles (admin, maintain, write)
/// - `Team`: Specific teams (use team ID)
/// - `Integration`: GitHub Apps
/// - `DeployKey`: Deploy keys
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BypassActorConfig {
    /// Actor ID
    pub actor_id: u64,

    /// Actor type: "OrganizationAdmin", "RepositoryRole", "Team", "Integration", or "DeployKey"
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

impl RulesetConfig {
    /// Converts configuration to domain type.
    ///
    /// Maps TOML-friendly configuration to GitHub API domain types.
    pub fn to_domain_ruleset(&self) -> github_client::RepositoryRuleset {
        use github_client::{
            BypassActor, BypassActorType, BypassMode, RefNameCondition, RepositoryRuleset, Rule,
            RulesetConditions, RulesetEnforcement, RulesetTarget,
        };

        // Convert target
        let target = match self.target.as_str() {
            "tag" => RulesetTarget::Tag,
            "push" => RulesetTarget::Push,
            _ => RulesetTarget::Branch,
        };

        // Convert enforcement
        let enforcement = match self.enforcement.as_str() {
            "disabled" => RulesetEnforcement::Disabled,
            "evaluate" => RulesetEnforcement::Evaluate,
            _ => RulesetEnforcement::Active,
        };

        // Convert bypass actors
        let bypass_actors: Vec<BypassActor> = self
            .bypass_actors
            .iter()
            .map(|ba| BypassActor {
                actor_id: ba.actor_id,
                actor_type: match ba.actor_type.as_str() {
                    "RepositoryRole" => BypassActorType::RepositoryRole,
                    "Team" => BypassActorType::Team,
                    "Integration" => BypassActorType::Integration,
                    "DeployKey" => BypassActorType::DeployKey,
                    _ => BypassActorType::OrganizationAdmin,
                },
                bypass_mode: match ba.bypass_mode.as_str() {
                    "pull_request" => BypassMode::PullRequest,
                    _ => BypassMode::Always,
                },
            })
            .collect();

        // Convert conditions
        let conditions = self.conditions.as_ref().map(|c| RulesetConditions {
            ref_name: RefNameCondition {
                include: c.ref_name.include.clone(),
                exclude: c.ref_name.exclude.clone(),
            },
        });

        // Convert rules
        let rules: Vec<Rule> = self
            .rules
            .iter()
            .map(|r| match r {
                RuleConfig::Creation => Rule::Creation,
                RuleConfig::Update => Rule::Update,
                RuleConfig::Deletion => Rule::Deletion,
                RuleConfig::RequiredLinearHistory => Rule::RequiredLinearHistory,
                RuleConfig::RequiredSignatures => Rule::RequiredSignatures,
                RuleConfig::PullRequest {
                    dismiss_stale_reviews_on_push,
                    require_code_owner_review,
                    require_last_push_approval,
                    required_approving_review_count,
                    required_review_thread_resolution,
                    allowed_merge_methods,
                } => {
                    use github_client::{MergeMethod, PullRequestParameters};

                    Rule::PullRequest {
                        parameters: PullRequestParameters {
                            dismiss_stale_reviews_on_push: *dismiss_stale_reviews_on_push,
                            require_code_owner_review: *require_code_owner_review,
                            require_last_push_approval: *require_last_push_approval,
                            required_approving_review_count: *required_approving_review_count,
                            required_review_thread_resolution: *required_review_thread_resolution,
                            allowed_merge_methods: allowed_merge_methods.as_ref().map(|methods| {
                                methods
                                    .iter()
                                    .filter_map(|m| match m.as_str() {
                                        "merge" => Some(MergeMethod::Merge),
                                        "squash" => Some(MergeMethod::Squash),
                                        "rebase" => Some(MergeMethod::Rebase),
                                        _ => None,
                                    })
                                    .collect()
                            }),
                        },
                    }
                }
                RuleConfig::RequiredStatusChecks {
                    required_status_checks,
                    strict_required_status_checks_policy,
                } => {
                    use github_client::{RequiredStatusChecksParameters, StatusCheck};

                    Rule::RequiredStatusChecks {
                        parameters: RequiredStatusChecksParameters {
                            required_status_checks: required_status_checks
                                .iter()
                                .map(|sc| StatusCheck {
                                    context: sc.context.clone(),
                                    integration_id: sc.integration_id,
                                })
                                .collect(),
                            strict_required_status_checks_policy:
                                *strict_required_status_checks_policy,
                        },
                    }
                }
                RuleConfig::NonFastForward => Rule::NonFastForward,
            })
            .collect();

        RepositoryRuleset {
            id: None, // Will be assigned by GitHub
            name: self.name.clone(),
            target,
            enforcement,
            bypass_actors,
            conditions,
            rules,
        }
    }
}
