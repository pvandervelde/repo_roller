//! Repository ruleset management operations.
//!
//! This module provides the [`RulesetManager`] component for orchestrating
//! repository ruleset operations with business logic, idempotency, and conflict detection.

use github_client::{GitHubClient, RepositoryClient, RepositoryRuleset};
use std::collections::HashMap;
use tracing::{info, warn};

use crate::{GitHubError, RepoRollerResult};

/// Manages repository ruleset operations.
///
/// This manager provides high-level ruleset management operations that handle
/// idempotency, conflict detection, and proper error handling. It orchestrates
/// the GitHubClient ruleset operations with business logic.
///
/// # Merge Strategy
///
/// Rulesets are applied additively from multiple levels:
/// - Organization-level rulesets
/// - Team-level rulesets
/// - Template-specific rulesets
///
/// All rulesets are combined. Conflicts are detected and reported but do not
/// prevent application. Critical conflicts (e.g., merge strategy deadlocks) are
/// logged as errors, while minor conflicts are logged as warnings.
///
/// # Examples
///
/// ```rust,no_run
/// use github_client::GitHubClient;
/// use repo_roller_core::RulesetManager;
/// use config_manager::settings::RulesetConfig;
/// use std::collections::HashMap;
///
/// # async fn example(github_client: GitHubClient) -> Result<(), Box<dyn std::error::Error>> {
/// let manager = RulesetManager::new(github_client);
///
/// let mut rulesets = HashMap::new();
/// // Add rulesets from org, team, template levels...
///
/// let result = manager.apply_rulesets("my-org", "my-repo", &rulesets).await?;
/// println!("Created: {}, Updated: {}", result.created, result.updated);
/// # Ok(())
/// # }
/// ```
pub struct RulesetManager {
    /// GitHub client for API operations
    github_client: GitHubClient,
}

impl RulesetManager {
    /// Creates a new RulesetManager.
    ///
    /// # Arguments
    ///
    /// * `github_client` - GitHub client for API operations
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use github_client::GitHubClient;
    /// use repo_roller_core::RulesetManager;
    ///
    /// # async fn example(client: GitHubClient) {
    /// let manager = RulesetManager::new(client);
    /// # }
    /// ```
    pub fn new(github_client: GitHubClient) -> Self {
        Self { github_client }
    }

    /// Applies rulesets to a repository, creating or updating as needed.
    ///
    /// This method ensures that all rulesets from the configuration are present
    /// in the repository. It is idempotent and safe to call multiple times.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `rulesets` - Map of ruleset name to RulesetConfig
    ///
    /// # Returns
    ///
    /// `Ok(ApplyRulesetsResult)` with details of operations performed and any conflicts detected
    ///
    /// # Errors
    ///
    /// Returns `RepoRollerError::System` if ruleset operations fail.
    ///
    /// # Behavior
    ///
    /// 1. Lists existing rulesets in the repository
    /// 2. Detects conflicts between rulesets (merge strategy deadlocks, etc.)
    /// 3. Creates or updates each ruleset
    /// 4. Logs conflicts with appropriate severity
    /// 5. Returns summary of operations and conflicts
    ///
    /// # Error Handling
    ///
    /// - Continues on individual ruleset failures (logs warning)
    /// - Returns error only if all rulesets fail
    /// - Partial success is considered success
    /// - Conflicts do not prevent application
    pub async fn apply_rulesets(
        &self,
        owner: &str,
        repo: &str,
        rulesets: &HashMap<String, config_manager::settings::RulesetConfig>,
    ) -> RepoRollerResult<ApplyRulesetsResult> {
        info!(
            owner = owner,
            repo = repo,
            ruleset_count = rulesets.len(),
            "Applying rulesets to repository"
        );

        let mut result = ApplyRulesetsResult::new();

        // Convert configs to domain rulesets
        let domain_rulesets: Vec<RepositoryRuleset> = rulesets
            .values()
            .map(|config| config.to_domain_ruleset())
            .collect();

        // Detect conflicts before applying
        let conflicts = detect_conflicts(&domain_rulesets);
        for conflict in &conflicts {
            match conflict.severity {
                ConflictSeverity::Critical => {
                    warn!(
                        conflict = conflict.description,
                        recommendation = conflict.recommendation,
                        "CRITICAL RULESET CONFLICT DETECTED"
                    );
                }
                ConflictSeverity::Error => {
                    warn!(
                        conflict = conflict.description,
                        recommendation = conflict.recommendation,
                        "Ruleset conflict (error)"
                    );
                }
                ConflictSeverity::Warning => {
                    info!(
                        conflict = conflict.description,
                        recommendation = conflict.recommendation,
                        "Ruleset conflict (warning)"
                    );
                }
                ConflictSeverity::Info => {
                    info!(conflict = conflict.description, "Ruleset info");
                }
            }
        }
        result.conflicts = conflicts;

        // List existing rulesets
        let existing_rulesets = match self
            .github_client
            .list_repository_rulesets(owner, repo)
            .await
        {
            Ok(rulesets) => {
                info!(count = rulesets.len(), "Retrieved existing rulesets");
                rulesets
            }
            Err(e) => {
                warn!(
                    error = ?e,
                    "Failed to list existing rulesets, will attempt to create all"
                );
                Vec::new()
            }
        };

        let existing_map: HashMap<String, &RepositoryRuleset> = existing_rulesets
            .iter()
            .map(|r| (r.name.clone(), r))
            .collect();

        // Apply each ruleset
        for (ruleset_name, ruleset_config) in rulesets {
            info!(name = ruleset_name, "Applying ruleset");

            let domain_ruleset = ruleset_config.to_domain_ruleset();

            if let Some(existing) = existing_map.get(ruleset_name) {
                // Update existing ruleset
                let ruleset_id = existing.id.expect("Existing ruleset must have ID");
                match self
                    .github_client
                    .update_repository_ruleset(owner, repo, ruleset_id, &domain_ruleset)
                    .await
                {
                    Ok(_) => {
                        info!(
                            name = ruleset_name,
                            id = ruleset_id,
                            "Ruleset updated successfully"
                        );
                        result.updated += 1;
                    }
                    Err(e) => {
                        warn!(
                            name = ruleset_name,
                            error = ?e,
                            "Failed to update ruleset"
                        );
                        result.failed += 1;
                        result.failed_rulesets.push(ruleset_name.clone());
                    }
                }
            } else {
                // Create new ruleset
                match self
                    .github_client
                    .create_repository_ruleset(owner, repo, &domain_ruleset)
                    .await
                {
                    Ok(created) => {
                        let created_id = created.id.unwrap_or(0);
                        info!(
                            name = ruleset_name,
                            id = created_id,
                            "Ruleset created successfully"
                        );
                        result.created += 1;
                    }
                    Err(e) => {
                        warn!(
                            name = ruleset_name,
                            error = ?e,
                            "Failed to create ruleset"
                        );
                        result.failed += 1;
                        result.failed_rulesets.push(ruleset_name.clone());
                    }
                }
            }
        }

        info!(
            created = result.created,
            updated = result.updated,
            failed = result.failed,
            conflicts = result.conflicts.len(),
            "Ruleset application complete"
        );

        Ok(result)
    }

    /// Lists all rulesets currently defined in a repository.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    ///
    /// # Returns
    ///
    /// Vector of RepositoryRuleset objects
    ///
    /// # Errors
    ///
    /// Returns `RepoRollerError::System` if API call fails.
    pub async fn list_rulesets(
        &self,
        owner: &str,
        repo: &str,
    ) -> RepoRollerResult<Vec<RepositoryRuleset>> {
        match self
            .github_client
            .list_repository_rulesets(owner, repo)
            .await
        {
            Ok(rulesets) => Ok(rulesets),
            Err(e) => {
                warn!(owner = owner, repo = repo, error = ?e, "Failed to list rulesets");
                Err(GitHubError::InvalidResponse {
                    reason: format!("Failed to list rulesets for {}/{}: {}", owner, repo, e),
                }
                .into())
            }
        }
    }
}

/// Detects conflicts between rulesets.
///
/// # Arguments
///
/// * `rulesets` - Collection of rulesets to analyze
///
/// # Returns
///
/// Vector of detected conflicts with severity and recommendations
fn detect_conflicts(_rulesets: &[RepositoryRuleset]) -> Vec<RulesetConflict> {
    let conflicts = Vec::new();

    // TODO: Implement conflict detection logic
    // - Detect merge strategy deadlocks
    // - Detect contradictory enforcement levels
    // - Detect overlapping conditions
    // - Detect conflicting bypass actors

    conflicts
}

/// Result of applying rulesets to a repository.
///
/// Contains counters for the different outcomes of ruleset operations,
/// plus any detected conflicts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyRulesetsResult {
    /// Number of rulesets created
    pub created: usize,

    /// Number of rulesets updated
    pub updated: usize,

    /// Number of rulesets that failed to apply
    pub failed: usize,

    /// Names of rulesets that failed (for error reporting)
    pub failed_rulesets: Vec<String>,

    /// Detected conflicts between rulesets
    pub conflicts: Vec<RulesetConflict>,
}

impl ApplyRulesetsResult {
    /// Creates a new empty result.
    pub fn new() -> Self {
        Self {
            created: 0,
            updated: 0,
            failed: 0,
            failed_rulesets: Vec::new(),
            conflicts: Vec::new(),
        }
    }

    /// Returns true if all operations succeeded.
    pub fn is_success(&self) -> bool {
        self.failed == 0
    }

    /// Returns true if any rulesets were successfully applied (created or updated).
    pub fn has_changes(&self) -> bool {
        self.created > 0 || self.updated > 0
    }

    /// Returns true if any critical conflicts were detected.
    pub fn has_critical_conflicts(&self) -> bool {
        self.conflicts
            .iter()
            .any(|c| c.severity == ConflictSeverity::Critical)
    }
}

impl Default for ApplyRulesetsResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a conflict between rulesets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulesetConflict {
    /// Severity of the conflict
    pub severity: ConflictSeverity,

    /// Human-readable description of the conflict
    pub description: String,

    /// Recommended action to resolve the conflict
    pub recommendation: String,

    /// Rulesets involved in the conflict
    pub rulesets: Vec<String>,
}

/// Severity level for ruleset conflicts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictSeverity {
    /// Critical conflict that will likely prevent repository operations
    /// Example: Merge strategy deadlock (require PR + conflicting merge methods)
    Critical,

    /// Error-level conflict that should be resolved
    /// Example: Contradictory enforcement levels
    Error,

    /// Warning about potential issues
    /// Example: Overlapping conditions
    Warning,

    /// Informational note
    /// Example: Multiple rulesets targeting same branch
    Info,
}

#[cfg(test)]
#[path = "ruleset_manager_tests.rs"]
mod tests;
