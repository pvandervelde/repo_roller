//! Configuration setting types for hierarchical configuration system.
//!
//! This module contains all the setting category types used by GlobalDefaults,
//! TeamConfig, RepositoryTypeConfig, and other configuration structures.
//!
//! See: specs/design/organization-repository-settings.md

pub mod actions;
pub mod branch_protection;
pub mod custom_property;
pub mod environment;
pub mod github_app;
pub mod label;
pub mod notifications;
pub mod pull_request;
pub mod push;
pub mod repository;
pub mod ruleset;
pub mod webhook;

// Re-export all types for convenient access
pub use actions::ActionSettings;
pub use branch_protection::BranchProtectionSettings;
pub use custom_property::CustomProperty;
pub use environment::EnvironmentConfig;
pub use github_app::GitHubAppConfig;
pub use label::LabelConfig;
pub use notifications::{NotificationEndpoint, NotificationsConfig};
pub use pull_request::PullRequestSettings;
pub use push::PushSettings;
pub use repository::RepositorySettings;
pub use ruleset::{
    BypassActorConfig, RefNameConditionConfig, RuleConfig, RulesetConditionsConfig, RulesetConfig,
    StatusCheckConfig,
};
pub use webhook::WebhookConfig;
