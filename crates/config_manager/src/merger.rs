//! Configuration merging engine.
//!
//! This module implements the hierarchical configuration merging system that
//! combines configuration from multiple sources (Global → Repository Type → Team → Template)
//! according to precedence rules and override policies.
//!
//! # Configuration Hierarchy
//!
//! The merge follows a strict precedence order from lowest to highest:
//! 1. **Global** - Organization-wide defaults
//! 2. **Repository Type** - Type-specific configuration
//! 3. **Team** - Team-specific overrides
//! 4. **Template** - Template-specific settings (highest precedence)
//!
//! # Override Policy Enforcement
//!
//! Settings can be marked as non-overridable using `OverridableValue<T>` with
//! `override_allowed = false`. When a lower-precedence layer prohibits overrides,
//! higher-precedence layers attempting to override that setting will cause an error.
//!
//! # Examples
//!
//! ```rust
//! use config_manager::{ConfigurationMerger, GlobalDefaults, TemplateConfig, TemplateMetadata};
//!
//! let merger = ConfigurationMerger::new();
//! let global = GlobalDefaults::default();
//! let template = TemplateConfig {
//!     template: TemplateMetadata {
//!         name: "example".to_string(),
//!         description: "Example template".to_string(),
//!         author: "Test".to_string(),
//!         tags: vec![],
//!     },
//!     repository: None,
//!     repository_type: None,
//!     pull_requests: None,
//!     branch_protection: None,
//!     labels: None,
//!     webhooks: None,
//!     environments: None,
//!     github_apps: None,
//!     variables: None,
//!     default_visibility: None,
//! };
//!
//! // Merge configurations with precedence rules
//! let merged = merger.merge_configurations(
//!     &global,
//!     None,  // No repository type
//!     None,  // No team config
//!     &template,
//! )?;
//! # Ok::<(), config_manager::ConfigurationError>(())
//! ```
//!
//! See: specs/design/organization-repository-settings.md

use crate::{
    errors::{ConfigurationError, ConfigurationResult},
    global_defaults::GlobalDefaults,
    merged_config::{ConfigurationSource, MergedConfiguration},
    repository_type_config::RepositoryTypeConfig,
    settings::{
        BranchProtectionSettings, CustomProperty, EnvironmentConfig, GitHubAppConfig,
        PullRequestSettings, RepositorySettings, WebhookConfig,
    },
    team_config::TeamConfig,
    template_config::TemplateConfig as NewTemplateConfig,
    OverridableValue,
};

/// Configuration merging engine.
///
/// Implements hierarchical configuration merging with override policy enforcement.
/// This is a stateless component - it takes configuration inputs and produces
/// merged output without maintaining internal state.
///
/// # Examples
///
/// ```rust
/// use config_manager::{
///     ConfigurationMerger, GlobalDefaults, RepositoryTypeConfig,
///     TeamConfig, TemplateConfig, TemplateMetadata
/// };
///
/// let merger = ConfigurationMerger::new();
/// let global_defaults = GlobalDefaults::default();
/// let repository_type_config = RepositoryTypeConfig::default();
/// let team_config = TeamConfig::default();
/// let template_config = TemplateConfig {
///     template: TemplateMetadata {
///         name: "example".to_string(),
///         description: "Example".to_string(),
///         author: "Test".to_string(),
///         tags: vec![],
///     },
///     repository: None,
///     repository_type: None,
///     pull_requests: None,
///     branch_protection: None,
///     labels: None,
///     webhooks: None,
///     environments: None,
///     github_apps: None,
///     variables: None,
///     default_visibility: None,
/// };
///
/// // Merge with all configuration levels
/// let merged = merger.merge_configurations(
///     &global_defaults,
///     Some(&repository_type_config),
///     Some(&team_config),
///     &template_config,
/// )?;
/// # Ok::<(), config_manager::ConfigurationError>(())
/// ```
#[derive(Debug, Clone, Default)]
pub struct ConfigurationMerger {
    // Stateless - no fields needed for now
    // Could add validation rules or policies in the future
}

impl ConfigurationMerger {
    /// Creates a new configuration merger.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::ConfigurationMerger;
    ///
    /// let merger = ConfigurationMerger::new();
    /// ```
    pub fn new() -> Self {
        Self {}
    }

    /// Merges configurations from all hierarchy levels.
    ///
    /// Combines configuration from global defaults, repository type, team, and template
    /// according to the precedence hierarchy (Template > Team > Repository Type > Global).
    ///
    /// # Arguments
    ///
    /// * `global` - Organization-wide default configuration
    /// * `repository_type` - Optional repository type-specific configuration
    /// * `team` - Optional team-specific configuration overrides
    /// * `template` - Template-specific configuration (highest precedence)
    ///
    /// # Returns
    ///
    /// Returns a `MergedConfiguration` containing the final resolved settings with
    /// source tracking for audit purposes.
    ///
    /// # Errors
    ///
    /// Returns `ConfigurationError` if:
    /// - A higher-precedence layer tries to override a non-overridable setting
    /// - Configuration values are invalid or incompatible
    /// - Business rule validation fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::{ConfigurationMerger, GlobalDefaults, TemplateConfig, TemplateMetadata};
    ///
    /// let merger = ConfigurationMerger::new();
    /// let global = GlobalDefaults::default();
    /// let template = TemplateConfig {
    ///     template: TemplateMetadata {
    ///         name: "example".to_string(),
    ///         description: "Example".to_string(),
    ///         author: "Test".to_string(),
    ///         tags: vec![],
    ///     },
    ///     repository: None,
    ///     repository_type: None,
    ///     pull_requests: None,
    ///     branch_protection: None,
    ///     labels: None,
    ///     webhooks: None,
    ///     environments: None,
    ///     github_apps: None,
    ///     variables: None,
    ///     default_visibility: None,
    /// };
    ///
    /// let merged = merger.merge_configurations(&global, None, None, &template)?;
    /// # Ok::<(), config_manager::ConfigurationError>(())
    /// ```
    pub fn merge_configurations(
        &self,
        global: &GlobalDefaults,
        repository_type: Option<&RepositoryTypeConfig>,
        team: Option<&TeamConfig>,
        template: &NewTemplateConfig,
    ) -> ConfigurationResult<MergedConfiguration> {
        let mut merged = MergedConfiguration::new();
        let mut source_updates: Vec<(String, ConfigurationSource)> = Vec::new();

        // Phase 1: Apply global defaults as baseline
        self.apply_global_defaults(&mut merged, global, &mut source_updates);

        // Phase 2: Apply repository type overrides (if present)
        if let Some(repo_type) = repository_type {
            self.apply_repository_type_overrides(
                &mut merged,
                repo_type,
                global,
                &mut source_updates,
            )?;
        }

        // Phase 3: Apply team overrides (if present)
        if let Some(team_config) = team {
            self.apply_team_overrides(
                &mut merged,
                team_config,
                global,
                repository_type,
                &mut source_updates,
            )?;
        }

        // Phase 4: Apply template configuration (highest precedence)
        self.apply_template_configuration(
            &mut merged,
            template,
            global,
            repository_type,
            team,
            &mut source_updates,
        )?;

        // Apply all source tracking
        for (field_path, source) in source_updates {
            merged.record_source(&field_path, source);
        }

        Ok(merged)
    }

    /// Applies global defaults as the baseline configuration.
    fn apply_global_defaults(
        &self,
        merged: &mut MergedConfiguration,
        global: &GlobalDefaults,
        source_updates: &mut Vec<(String, ConfigurationSource)>,
    ) {
        if let Some(repo_settings) = &global.repository {
            merged.repository = repo_settings.clone();
            self.track_repository_settings_sources(
                &merged.repository,
                source_updates,
                ConfigurationSource::Global,
            );
        }
        if let Some(pr_settings) = &global.pull_requests {
            merged.pull_requests = pr_settings.clone();
            self.track_pull_request_settings_sources(
                &merged.pull_requests,
                source_updates,
                ConfigurationSource::Global,
            );
        }
        if let Some(bp_settings) = &global.branch_protection {
            merged.branch_protection = bp_settings.clone();
            self.track_branch_protection_settings_sources(
                &merged.branch_protection,
                source_updates,
                ConfigurationSource::Global,
            );
        }

        // Merge global collections
        if let Some(webhooks) = &global.webhooks {
            source_updates.extend(self.merge_webhooks(
                &mut merged.webhooks,
                webhooks,
                ConfigurationSource::Global,
            ));
        }
        if let Some(environments) = &global.environments {
            source_updates.extend(self.merge_environments(
                &mut merged.environments,
                environments,
                ConfigurationSource::Global,
            ));
        }
        if let Some(apps) = &global.github_apps {
            source_updates.extend(self.merge_github_apps(
                &mut merged.github_apps,
                apps,
                ConfigurationSource::Global,
            ));
        }
        if let Some(properties) = &global.custom_properties {
            source_updates.extend(self.merge_custom_properties(
                &mut merged.custom_properties,
                properties,
                ConfigurationSource::Global,
            ));
        }
    }

    /// Applies repository type-specific overrides.
    fn apply_repository_type_overrides(
        &self,
        merged: &mut MergedConfiguration,
        repo_type: &RepositoryTypeConfig,
        global: &GlobalDefaults,
        source_updates: &mut Vec<(String, ConfigurationSource)>,
    ) -> ConfigurationResult<()> {
        // Merge settings with override validation
        if let Some(override_repo) = &repo_type.repository {
            if let Some(base_repo) = &global.repository {
                source_updates.extend(self.merge_repository_settings(
                    &mut merged.repository,
                    override_repo,
                    base_repo,
                    ConfigurationSource::RepositoryType,
                )?);
            } else {
                merged.repository = override_repo.clone();
                self.track_repository_settings_sources(
                    &merged.repository,
                    source_updates,
                    ConfigurationSource::RepositoryType,
                );
            }
        }
        if let Some(override_pr) = &repo_type.pull_requests {
            if let Some(base_pr) = &global.pull_requests {
                source_updates.extend(self.merge_pull_request_settings(
                    &mut merged.pull_requests,
                    override_pr,
                    base_pr,
                    ConfigurationSource::RepositoryType,
                )?);
            } else {
                merged.pull_requests = override_pr.clone();
                self.track_pull_request_settings_sources(
                    &merged.pull_requests,
                    source_updates,
                    ConfigurationSource::RepositoryType,
                );
            }
        }
        if let Some(override_bp) = &repo_type.branch_protection {
            if let Some(base_bp) = &global.branch_protection {
                source_updates.extend(self.merge_branch_protection_settings(
                    &mut merged.branch_protection,
                    override_bp,
                    base_bp,
                    ConfigurationSource::RepositoryType,
                )?);
            } else {
                merged.branch_protection = override_bp.clone();
                self.track_branch_protection_settings_sources(
                    &merged.branch_protection,
                    source_updates,
                    ConfigurationSource::RepositoryType,
                );
            }
        }

        // Merge collections additively
        if let Some(webhooks) = &repo_type.webhooks {
            source_updates.extend(self.merge_webhooks(
                &mut merged.webhooks,
                webhooks,
                ConfigurationSource::RepositoryType,
            ));
        }
        if let Some(environments) = &repo_type.environments {
            source_updates.extend(self.merge_environments(
                &mut merged.environments,
                environments,
                ConfigurationSource::RepositoryType,
            ));
        }
        if let Some(apps) = &repo_type.github_apps {
            source_updates.extend(self.merge_github_apps(
                &mut merged.github_apps,
                apps,
                ConfigurationSource::RepositoryType,
            ));
        }
        if let Some(properties) = &repo_type.custom_properties {
            source_updates.extend(self.merge_custom_properties(
                &mut merged.custom_properties,
                properties,
                ConfigurationSource::RepositoryType,
            ));
        }

        Ok(())
    }

    /// Applies team-specific overrides.
    fn apply_team_overrides(
        &self,
        merged: &mut MergedConfiguration,
        team: &TeamConfig,
        global: &GlobalDefaults,
        repository_type: Option<&RepositoryTypeConfig>,
        source_updates: &mut Vec<(String, ConfigurationSource)>,
    ) -> ConfigurationResult<()> {
        // Determine base configuration for override validation
        let default_repo = RepositorySettings::default();
        let default_pr = PullRequestSettings::default();
        let default_bp = BranchProtectionSettings::default();

        let base_repo = repository_type
            .and_then(|rt| rt.repository.as_ref())
            .or(global.repository.as_ref())
            .unwrap_or(&default_repo);
        let base_pr = repository_type
            .and_then(|rt| rt.pull_requests.as_ref())
            .or(global.pull_requests.as_ref())
            .unwrap_or(&default_pr);
        let base_bp = repository_type
            .and_then(|rt| rt.branch_protection.as_ref())
            .or(global.branch_protection.as_ref())
            .unwrap_or(&default_bp);

        // Merge settings with override validation
        if let Some(override_repo) = &team.repository {
            source_updates.extend(self.merge_repository_settings(
                &mut merged.repository,
                override_repo,
                base_repo,
                ConfigurationSource::Team,
            )?);
        }
        if let Some(override_pr) = &team.pull_requests {
            source_updates.extend(self.merge_pull_request_settings(
                &mut merged.pull_requests,
                override_pr,
                base_pr,
                ConfigurationSource::Team,
            )?);
        }
        if let Some(override_bp) = &team.branch_protection {
            source_updates.extend(self.merge_branch_protection_settings(
                &mut merged.branch_protection,
                override_bp,
                base_bp,
                ConfigurationSource::Team,
            )?);
        }

        // Merge collections additively
        if let Some(webhooks) = &team.webhooks {
            source_updates.extend(self.merge_webhooks(
                &mut merged.webhooks,
                webhooks,
                ConfigurationSource::Team,
            ));
        }
        if let Some(environments) = &team.environments {
            source_updates.extend(self.merge_environments(
                &mut merged.environments,
                environments,
                ConfigurationSource::Team,
            ));
        }
        if let Some(apps) = &team.github_apps {
            source_updates.extend(self.merge_github_apps(
                &mut merged.github_apps,
                apps,
                ConfigurationSource::Team,
            ));
        }
        if let Some(properties) = &team.custom_properties {
            source_updates.extend(self.merge_custom_properties(
                &mut merged.custom_properties,
                properties,
                ConfigurationSource::Team,
            ));
        }

        Ok(())
    }

    /// Applies template configuration (highest precedence).
    fn apply_template_configuration(
        &self,
        merged: &mut MergedConfiguration,
        template: &NewTemplateConfig,
        global: &GlobalDefaults,
        repository_type: Option<&RepositoryTypeConfig>,
        team: Option<&TeamConfig>,
        source_updates: &mut Vec<(String, ConfigurationSource)>,
    ) -> ConfigurationResult<()> {
        // Determine base configuration for override validation (most recent layer)
        let default_repo = RepositorySettings::default();
        let default_pr = PullRequestSettings::default();
        let default_bp = BranchProtectionSettings::default();

        let base_repo = team
            .and_then(|t| t.repository.as_ref())
            .or(repository_type.and_then(|rt| rt.repository.as_ref()))
            .or(global.repository.as_ref())
            .unwrap_or(&default_repo);
        let base_pr = team
            .and_then(|t| t.pull_requests.as_ref())
            .or(repository_type.and_then(|rt| rt.pull_requests.as_ref()))
            .or(global.pull_requests.as_ref())
            .unwrap_or(&default_pr);
        let base_bp = team
            .and_then(|t| t.branch_protection.as_ref())
            .or(repository_type.and_then(|rt| rt.branch_protection.as_ref()))
            .or(global.branch_protection.as_ref())
            .unwrap_or(&default_bp);

        // Merge settings with override validation
        if let Some(override_repo) = &template.repository {
            source_updates.extend(self.merge_repository_settings(
                &mut merged.repository,
                override_repo,
                base_repo,
                ConfigurationSource::Template,
            )?);
        }
        if let Some(override_pr) = &template.pull_requests {
            source_updates.extend(self.merge_pull_request_settings(
                &mut merged.pull_requests,
                override_pr,
                base_pr,
                ConfigurationSource::Template,
            )?);
        }
        if let Some(override_bp) = &template.branch_protection {
            source_updates.extend(self.merge_branch_protection_settings(
                &mut merged.branch_protection,
                override_bp,
                base_bp,
                ConfigurationSource::Template,
            )?);
        }

        // Merge collections additively
        if let Some(webhooks) = &template.webhooks {
            source_updates.extend(self.merge_webhooks(
                &mut merged.webhooks,
                webhooks,
                ConfigurationSource::Template,
            ));
        }
        if let Some(environments) = &template.environments {
            source_updates.extend(self.merge_environments(
                &mut merged.environments,
                environments,
                ConfigurationSource::Template,
            ));
        }
        if let Some(apps) = &template.github_apps {
            source_updates.extend(self.merge_github_apps(
                &mut merged.github_apps,
                apps,
                ConfigurationSource::Template,
            ));
        }

        Ok(())
    }

    /// Merges repository settings with override policy validation.
    ///
    /// # Arguments
    ///
    /// * `target` - The target settings to update
    /// * `override_settings` - The settings attempting to override
    /// * `base_settings` - The base settings with override policies
    /// * `source` - The configuration source for audit trail
    ///
    /// # Errors
    ///
    /// Returns `ConfigurationError::OverrideNotAllowed` if attempting to override
    /// a setting that prohibits overrides.
    fn merge_repository_settings(
        &self,
        target: &mut RepositorySettings,
        override_settings: &RepositorySettings,
        base_settings: &RepositorySettings,
        source: ConfigurationSource,
    ) -> ConfigurationResult<Vec<(String, ConfigurationSource)>> {
        let mut source_updates = Vec::new();

        // Merge issues
        if let Some(override_value) = &override_settings.issues {
            if let Some(base_value) = &base_settings.issues {
                self.validate_override("repository.issues", base_value, &override_value.value)?;
            }
            target.issues = Some(override_value.clone());
            source_updates.push(("repository.issues".to_string(), source));
        }

        // Merge projects
        if let Some(override_value) = &override_settings.projects {
            if let Some(base_value) = &base_settings.projects {
                self.validate_override("repository.projects", base_value, &override_value.value)?;
            }
            target.projects = Some(override_value.clone());
            source_updates.push(("repository.projects".to_string(), source));
        }

        // Merge discussions
        if let Some(override_value) = &override_settings.discussions {
            if let Some(base_value) = &base_settings.discussions {
                self.validate_override(
                    "repository.discussions",
                    base_value,
                    &override_value.value,
                )?;
            }
            target.discussions = Some(override_value.clone());
            source_updates.push(("repository.discussions".to_string(), source));
        }

        // Merge wiki
        if let Some(override_value) = &override_settings.wiki {
            if let Some(base_value) = &base_settings.wiki {
                self.validate_override("repository.wiki", base_value, &override_value.value)?;
            }
            target.wiki = Some(override_value.clone());
            source_updates.push(("repository.wiki".to_string(), source));
        }

        // Merge pages
        if let Some(override_value) = &override_settings.pages {
            if let Some(base_value) = &base_settings.pages {
                self.validate_override("repository.pages", base_value, &override_value.value)?;
            }
            target.pages = Some(override_value.clone());
            source_updates.push(("repository.pages".to_string(), source));
        }

        // Merge security_advisories
        if let Some(override_value) = &override_settings.security_advisories {
            if let Some(base_value) = &base_settings.security_advisories {
                self.validate_override(
                    "repository.security_advisories",
                    base_value,
                    &override_value.value,
                )?;
            }
            target.security_advisories = Some(override_value.clone());
            source_updates.push(("repository.security_advisories".to_string(), source));
        }

        // Merge vulnerability_reporting
        if let Some(override_value) = &override_settings.vulnerability_reporting {
            if let Some(base_value) = &base_settings.vulnerability_reporting {
                self.validate_override(
                    "repository.vulnerability_reporting",
                    base_value,
                    &override_value.value,
                )?;
            }
            target.vulnerability_reporting = Some(override_value.clone());
            source_updates.push(("repository.vulnerability_reporting".to_string(), source));
        }

        // Merge auto_close_issues
        if let Some(override_value) = &override_settings.auto_close_issues {
            if let Some(base_value) = &base_settings.auto_close_issues {
                self.validate_override(
                    "repository.auto_close_issues",
                    base_value,
                    &override_value.value,
                )?;
            }
            target.auto_close_issues = Some(override_value.clone());
            source_updates.push(("repository.auto_close_issues".to_string(), source));
        }

        Ok(source_updates)
    }

    /// Merges pull request settings with override policy validation.
    fn merge_pull_request_settings(
        &self,
        target: &mut PullRequestSettings,
        override_settings: &PullRequestSettings,
        base_settings: &PullRequestSettings,
        source: ConfigurationSource,
    ) -> ConfigurationResult<Vec<(String, ConfigurationSource)>> {
        let mut source_updates = Vec::new();

        // Merge each field with validation
        if let Some(override_value) = &override_settings.allow_auto_merge {
            if let Some(base_value) = &base_settings.allow_auto_merge {
                self.validate_override(
                    "pull_requests.allow_auto_merge",
                    base_value,
                    &override_value.value,
                )?;
            }
            target.allow_auto_merge = Some(override_value.clone());
            source_updates.push(("pull_requests.allow_auto_merge".to_string(), source));
        }

        if let Some(override_value) = &override_settings.allow_merge_commit {
            if let Some(base_value) = &base_settings.allow_merge_commit {
                self.validate_override(
                    "pull_requests.allow_merge_commit",
                    base_value,
                    &override_value.value,
                )?;
            }
            target.allow_merge_commit = Some(override_value.clone());
            source_updates.push(("pull_requests.allow_merge_commit".to_string(), source));
        }

        if let Some(override_value) = &override_settings.allow_rebase_merge {
            if let Some(base_value) = &base_settings.allow_rebase_merge {
                self.validate_override(
                    "pull_requests.allow_rebase_merge",
                    base_value,
                    &override_value.value,
                )?;
            }
            target.allow_rebase_merge = Some(override_value.clone());
            source_updates.push(("pull_requests.allow_rebase_merge".to_string(), source));
        }

        if let Some(override_value) = &override_settings.allow_squash_merge {
            if let Some(base_value) = &base_settings.allow_squash_merge {
                self.validate_override(
                    "pull_requests.allow_squash_merge",
                    base_value,
                    &override_value.value,
                )?;
            }
            target.allow_squash_merge = Some(override_value.clone());
            source_updates.push(("pull_requests.allow_squash_merge".to_string(), source));
        }

        if let Some(override_value) = &override_settings.delete_branch_on_merge {
            if let Some(base_value) = &base_settings.delete_branch_on_merge {
                self.validate_override(
                    "pull_requests.delete_branch_on_merge",
                    base_value,
                    &override_value.value,
                )?;
            }
            target.delete_branch_on_merge = Some(override_value.clone());
            source_updates.push(("pull_requests.delete_branch_on_merge".to_string(), source));
        }

        if let Some(override_value) = &override_settings.required_approving_review_count {
            if let Some(base_value) = &base_settings.required_approving_review_count {
                self.validate_override(
                    "pull_requests.required_approving_review_count",
                    base_value,
                    &override_value.value,
                )?;
            }
            target.required_approving_review_count = Some(override_value.clone());
            source_updates.push((
                "pull_requests.required_approving_review_count".to_string(),
                source,
            ));
        }

        if let Some(override_value) = &override_settings.require_code_owner_reviews {
            if let Some(base_value) = &base_settings.require_code_owner_reviews {
                self.validate_override(
                    "pull_requests.require_code_owner_reviews",
                    base_value,
                    &override_value.value,
                )?;
            }
            target.require_code_owner_reviews = Some(override_value.clone());
            source_updates.push((
                "pull_requests.require_code_owner_reviews".to_string(),
                source,
            ));
        }

        if let Some(override_value) = &override_settings.require_conversation_resolution {
            if let Some(base_value) = &base_settings.require_conversation_resolution {
                self.validate_override(
                    "pull_requests.require_conversation_resolution",
                    base_value,
                    &override_value.value,
                )?;
            }
            target.require_conversation_resolution = Some(override_value.clone());
            source_updates.push((
                "pull_requests.require_conversation_resolution".to_string(),
                source,
            ));
        }

        Ok(source_updates)
    }

    /// Merges branch protection settings with override policy validation.
    fn merge_branch_protection_settings(
        &self,
        target: &mut BranchProtectionSettings,
        override_settings: &BranchProtectionSettings,
        base_settings: &BranchProtectionSettings,
        source: ConfigurationSource,
    ) -> ConfigurationResult<Vec<(String, ConfigurationSource)>> {
        let mut source_updates = Vec::new();

        if let Some(override_value) = &override_settings.default_branch {
            if let Some(base_value) = &base_settings.default_branch {
                self.validate_override(
                    "branch_protection.default_branch",
                    base_value,
                    &override_value.value,
                )?;
            }
            target.default_branch = Some(override_value.clone());
            source_updates.push(("branch_protection.default_branch".to_string(), source));
        }

        if let Some(override_value) = &override_settings.require_pull_request_reviews {
            if let Some(base_value) = &base_settings.require_pull_request_reviews {
                self.validate_override(
                    "branch_protection.require_pull_request_reviews",
                    base_value,
                    &override_value.value,
                )?;
            }
            target.require_pull_request_reviews = Some(override_value.clone());
            source_updates.push((
                "branch_protection.require_pull_request_reviews".to_string(),
                source,
            ));
        }

        if let Some(override_value) = &override_settings.require_status_checks {
            if let Some(base_value) = &base_settings.require_status_checks {
                self.validate_override(
                    "branch_protection.require_status_checks",
                    base_value,
                    &override_value.value,
                )?;
            }
            target.require_status_checks = Some(override_value.clone());
            source_updates.push((
                "branch_protection.require_status_checks".to_string(),
                source,
            ));
        }

        if let Some(override_value) = &override_settings.restrict_pushes {
            if let Some(base_value) = &base_settings.restrict_pushes {
                self.validate_override(
                    "branch_protection.restrict_pushes",
                    base_value,
                    &override_value.value,
                )?;
            }
            target.restrict_pushes = Some(override_value.clone());
            source_updates.push(("branch_protection.restrict_pushes".to_string(), source));
        }

        Ok(source_updates)
    }

    /// Merges webhook collections additively.
    ///
    /// All webhooks from all sources are combined.
    fn merge_webhooks(
        &self,
        target: &mut Vec<WebhookConfig>,
        webhooks: &[WebhookConfig],
        source: ConfigurationSource,
    ) -> Vec<(String, ConfigurationSource)> {
        let mut source_updates = Vec::new();

        for webhook in webhooks {
            target.push(webhook.clone());
            source_updates.push(("webhooks".to_string(), source));
        }

        source_updates
    }

    /// Merges environment collections additively.
    ///
    /// All environments from all sources are combined.
    fn merge_environments(
        &self,
        target: &mut Vec<EnvironmentConfig>,
        environments: &[EnvironmentConfig],
        source: ConfigurationSource,
    ) -> Vec<(String, ConfigurationSource)> {
        let mut source_updates = Vec::new();

        for env in environments {
            target.push(env.clone());
            source_updates.push(("environments".to_string(), source));
        }

        source_updates
    }

    /// Merges GitHub App collections additively.
    ///
    /// All GitHub Apps from all sources are combined.
    fn merge_github_apps(
        &self,
        target: &mut Vec<GitHubAppConfig>,
        apps: &[GitHubAppConfig],
        source: ConfigurationSource,
    ) -> Vec<(String, ConfigurationSource)> {
        let mut source_updates = Vec::new();

        for app in apps {
            target.push(app.clone());
            source_updates.push(("github_apps".to_string(), source));
        }

        source_updates
    }

    /// Merges custom property collections additively.
    ///
    /// All custom properties from all sources are combined.
    fn merge_custom_properties(
        &self,
        target: &mut Vec<CustomProperty>,
        properties: &[CustomProperty],
        source: ConfigurationSource,
    ) -> Vec<(String, ConfigurationSource)> {
        let mut source_updates = Vec::new();

        for property in properties {
            target.push(property.clone());
            source_updates.push(("custom_properties".to_string(), source));
        }

        source_updates
    }

    /// Validates that an override is allowed.
    ///
    /// Checks if a setting can be overridden based on its `override_allowed` flag.
    ///
    /// # Arguments
    ///
    /// * `field_path` - Dot-separated path to the field (e.g., "repository.issues")
    /// * `base_value` - The base value with override policy
    /// * `override_value` - The value attempting to override
    ///
    /// # Errors
    ///
    /// Returns `ConfigurationError::OverrideNotAllowed` if the base value prohibits
    /// overrides and the override value differs from the base.
    fn validate_override<T: PartialEq + std::fmt::Display>(
        &self,
        field_path: &str,
        base_value: &OverridableValue<T>,
        override_value: &T,
    ) -> ConfigurationResult<()> {
        if !base_value.override_allowed && &base_value.value != override_value {
            return Err(ConfigurationError::OverrideNotPermitted {
                setting: field_path.to_string(),
                reason: format!(
                    "Cannot override '{}' with value '{}' - override not allowed by policy",
                    field_path, override_value
                ),
            });
        }
        Ok(())
    }

    /// Tracks source for all non-None repository settings fields.
    fn track_repository_settings_sources(
        &self,
        settings: &RepositorySettings,
        source_updates: &mut Vec<(String, ConfigurationSource)>,
        source: ConfigurationSource,
    ) {
        if settings.issues.is_some() {
            source_updates.push(("repository.issues".to_string(), source));
        }
        if settings.projects.is_some() {
            source_updates.push(("repository.projects".to_string(), source));
        }
        if settings.discussions.is_some() {
            source_updates.push(("repository.discussions".to_string(), source));
        }
        if settings.wiki.is_some() {
            source_updates.push(("repository.wiki".to_string(), source));
        }
        if settings.pages.is_some() {
            source_updates.push(("repository.pages".to_string(), source));
        }
        if settings.security_advisories.is_some() {
            source_updates.push(("repository.security_advisories".to_string(), source));
        }
        if settings.vulnerability_reporting.is_some() {
            source_updates.push(("repository.vulnerability_reporting".to_string(), source));
        }
        if settings.auto_close_issues.is_some() {
            source_updates.push(("repository.auto_close_issues".to_string(), source));
        }
    }

    /// Tracks source for all non-None pull request settings fields.
    fn track_pull_request_settings_sources(
        &self,
        settings: &PullRequestSettings,
        source_updates: &mut Vec<(String, ConfigurationSource)>,
        source: ConfigurationSource,
    ) {
        if settings.allow_auto_merge.is_some() {
            source_updates.push(("pull_requests.allow_auto_merge".to_string(), source));
        }
        if settings.allow_merge_commit.is_some() {
            source_updates.push(("pull_requests.allow_merge_commit".to_string(), source));
        }
        if settings.allow_rebase_merge.is_some() {
            source_updates.push(("pull_requests.allow_rebase_merge".to_string(), source));
        }
        if settings.allow_squash_merge.is_some() {
            source_updates.push(("pull_requests.allow_squash_merge".to_string(), source));
        }
        if settings.delete_branch_on_merge.is_some() {
            source_updates.push(("pull_requests.delete_branch_on_merge".to_string(), source));
        }
        if settings.required_approving_review_count.is_some() {
            source_updates.push((
                "pull_requests.required_approving_review_count".to_string(),
                source,
            ));
        }
        if settings.require_code_owner_reviews.is_some() {
            source_updates.push((
                "pull_requests.require_code_owner_reviews".to_string(),
                source,
            ));
        }
        if settings.require_conversation_resolution.is_some() {
            source_updates.push((
                "pull_requests.require_conversation_resolution".to_string(),
                source,
            ));
        }
    }

    /// Tracks source for all non-None branch protection settings fields.
    fn track_branch_protection_settings_sources(
        &self,
        settings: &BranchProtectionSettings,
        source_updates: &mut Vec<(String, ConfigurationSource)>,
        source: ConfigurationSource,
    ) {
        if settings.default_branch.is_some() {
            source_updates.push(("branch_protection.default_branch".to_string(), source));
        }
        if settings.require_pull_request_reviews.is_some() {
            source_updates.push((
                "branch_protection.require_pull_request_reviews".to_string(),
                source,
            ));
        }
        if settings.require_status_checks.is_some() {
            source_updates.push((
                "branch_protection.require_status_checks".to_string(),
                source,
            ));
        }
        if settings.restrict_pushes.is_some() {
            source_updates.push(("branch_protection.restrict_pushes".to_string(), source));
        }
    }
}

#[cfg(test)]
#[path = "merger_tests.rs"]
mod tests;
