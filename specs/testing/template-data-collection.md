# RepoRoller Configuration Architecture Specification

## Overview

This specification defines a hierarchical configuration system for RepoRoller that separates concerns across three levels:

1. **Global**: Organization-wide defaults and common labels
2. **Team**: Team-specific overrides and labels
3. **Template**: Template-specific settings and labels

The system uses a configuration hierarchy where Template settings override Team settings, which override Global defaults.

## Architecture Goals

- **Separation of Concerns**: Different types of configuration are managed separately
- **Team Autonomy**: Teams can define their own labels and override global defaults
- **Template Flexibility**: Templates can specify their own requirements and defaults
- **Consistency**: Global defaults ensure baseline consistency across the organization
- **Maintainability**: Clear structure makes it easy to find and modify settings
- **Scalability**: Easy to add new teams or templates without affecting existing configurations

## Authentication & Access Control

The system uses GitHub App authentication with the following characteristics:

- **GitHub App Installation**: The app must be installed in the target organization
- **Metadata Repository Access**: The app requires read access to the metadata repository
- **Template Repository Access**: The app requires read access to template repositories
- **Target Repository Access**: The app requires admin access to create and configure new repositories
- **Cross-Organization Limitations**: Cross-organization templating is not supported in the initial implementation
- **Branch Protection**: Both metadata and template repositories must have protected main branches requiring reviews

## Metadata Repository Discovery

The metadata repository is discovered through one of two methods:

1. **Configuration-Based**: Repository name specified in the app configuration file
2. **Topic-Based**: Repository tagged with the topic `template-metadata` in the organization

The system assumes one app installation per organization. The metadata repository must be hosted within the same organization as the target repository being created.

## Repository Structure

### Metadata Repository

The metadata repository contains shared and team-specific configurations:

```text
repo-config/
├── README.md
├── global/
│   ├── defaults.toml              # Organization-wide defaults
│   └── standard-labels.toml       # Common labels available to all teams
├── teams/
│   ├── {team-name}/
│   │   ├── config.toml            # Team-specific overrides
│   │   └── labels.toml            # Team-specific labels
│   └── ...
└── schemas/
    ├── template-config.schema.json # JSON schema for template configs
    └── team-config.schema.json     # JSON schema for team configs
```

### Template Repositories

Each template repository must:

- Be marked as a template repository on GitHub
- Contain a `.reporoller/` directory with configuration files

```text
{template-name}/
├── .reporoller/
│   ├── template.toml              # Template-specific configuration
│   ├── labels.toml                # Template-specific labels (optional)
│   └── README.md                  # Template documentation
├── {template files...}
└── ...
```

## Configuration File Specifications

### Global Defaults (`global/defaults.toml`)

Organization-wide default repository settings that apply to ALL repositories unless overridden. Each setting can specify whether teams and templates are allowed to override it.

```toml
[repository]
issues = { value = true, override_allowed = true }
projects = { value = false, override_allowed = true }
discussions = { value = true, override_allowed = true }
wiki = { value = true, override_allowed = true }
security_advisories = { value = true, override_allowed = false }  # Security policy - no overrides
vulnerability_reporting = { value = true, override_allowed = false }  # Security policy - no overrides
auto_close_issues = { value = true, override_allowed = true }

[pull_requests]
allow_merge_commit = { value = false, override_allowed = true }
allow_squash_merge = { value = true, override_allowed = true }
allow_rebase_merge = { value = true, override_allowed = true }
delete_branch_on_merge = { value = true, override_allowed = true }
require_conversation_resolution = { value = true, override_allowed = false }  # Quality policy - no overrides
allow_auto_merge = { value = false, override_allowed = true }
merge_commit_title = { value = "PR_TITLE", override_allowed = true }
merge_commit_message = { value = "PR_BODY", override_allowed = true }
squash_merge_commit_title = { value = "PR_TITLE", override_allowed = true }
squash_merge_commit_message = { value = "COMMIT_MESSAGES", override_allowed = true }

[push]
max_branches_per_push = { value = 5, override_allowed = true }
max_tags_per_push = { value = 5, override_allowed = true }

[branch_protection]
default_branch = { value = "main", override_allowed = true }
require_pull_request_reviews = { value = true, override_allowed = false }  # Security policy - no overrides
required_approving_review_count = { value = 1, override_allowed = true }
dismiss_stale_reviews = { value = true, override_allowed = true }
require_code_owner_reviews = { value = false, override_allowed = true }
restrict_pushes = { value = true, override_allowed = false }  # Security policy - no overrides

[actions]
enabled = { value = true, override_allowed = true }
allow_github_owned_actions = { value = true, override_allowed = false }  # Security policy - no overrides
allow_verified_creator_actions = { value = true, override_allowed = true }
allow_specified_actions = { value = false, override_allowed = true }

# GitHub Apps that must be installed on all repositories
[[github_apps]]
app_id = 12345
permissions = { issues = "read", pull_requests = "write" }
override_allowed = false  # Required for security compliance

[[github_apps]]
app_id = 67890
permissions = { actions = "read" }
override_allowed = true  # Teams can choose to enable this app
```

### Standard Labels (`global/standard-labels.toml`)

Common labels available to all teams. Teams can reference these by name.

```toml
[bug]
color = "d73a4a"
description = "Something isn't working"

[enhancement]
color = "a2eeef"
description = "New feature or request"

[documentation]
color = "0075ca"
description = "Improvements or additions to documentation"

# ... additional standard labels
```

### Team Configuration (`teams/{team-name}/config.toml`)

Team-specific configuration overrides. Any setting here overrides the global default.

```toml
[repository]
discussions = false  # Override global default
projects = true      # Override global default

[pull_requests]
required_approving_review_count = 2  # Override global default
require_code_owner_reviews = true
allow_auto_merge = true  # Enable auto-merge for this team

[push]
max_branches_per_push = 10  # Allow more branch updates for this team

[branch_protection]
additional_protected_patterns = ["release/*", "hotfix/*"]

# Team-specific webhooks
[[webhooks]]
url = "https://team.example.com/webhook"
content_type = "json"
events = ["push", "pull_request"]
active = true

# Team-specific GitHub Apps
[[github_apps]]
app_id = 11111
permissions = { contents = "read", issues = "write" }

# Team-specific environments
[[environments]]
name = "staging"
protection_rules = { required_reviewers = ["@team-leads"], wait_timer = 0 }
deployment_branch_policy = { protected_branches = true }

[[environments]]
name = "production"
protection_rules = { required_reviewers = ["@team-leads", "@security-team"], wait_timer = 300 }
deployment_branch_policy = { protected_branches = true, custom_branch_policies = ["release/*"] }
```

### Team Labels (`teams/{team-name}/labels.toml`)

Team-specific labels in addition to standard labels.

```toml
[priority-critical]
color = "b60205"
description = "Critical priority - immediate attention required"

[area-api]
color = "1d76db"
description = "API related changes"

# ... additional team-specific labels
```

### Template Configuration (`.reporoller/template.toml`)

Template-specific metadata and configuration.

```toml
[template]
name = "rust-microservice"
description = "Production-ready Rust microservice with observability"
author = "Platform Team"
tags = ["rust", "microservice", "backend", "api"]

[repository]
# Template-specific overrides
wiki = false
security_advisories = true

[pull_requests]
required_approving_review_count = 2
require_code_owner_reviews = true
allow_auto_merge = true
merge_commit_title = "MERGE_MESSAGE"
squash_merge_commit_title = "PR_TITLE"

[push]
max_branches_per_push = 3  # Restrict for microservices

[branch_protection]
require_status_checks = true
required_status_checks = ["build", "test", "security-scan", "lint"]
require_up_to_date_before_merge = true

# Template-specific webhooks for monitoring
[[webhooks]]
url = "https://monitoring.example.com/webhook/{{service_name}}"
content_type = "json"
events = ["push", "release"]
active = true

# Custom properties for service classification
[[custom_properties]]
name = "service_type"
value = "microservice"

[[custom_properties]]
name = "tech_stack"
value = "rust"

[[custom_properties]]
name = "team"
value = "{{team_name}}"

# Deployment environments for microservices
[[environments]]
name = "development"
protection_rules = { required_reviewers = ["@{{team_name}}"], wait_timer = 0 }
deployment_branch_policy = { protected_branches = false }

[[environments]]
name = "production"
protection_rules = { required_reviewers = ["@{{team_name}}", "@platform-team"], wait_timer = 600 }
deployment_branch_policy = { protected_branches = true, custom_branch_policies = ["main"] }

# Required GitHub Apps for microservices
[[github_apps]]
app_id = 55555  # Deployment app
permissions = { actions = "write", deployments = "write" }

[[github_apps]]
app_id = 66666  # Security scanning app
permissions = { security_events = "write", contents = "read" }

[variables]
service_name = { description = "Name of the microservice", example = "user-service" }
service_port = { description = "Port the service runs on", example = "8080", default = "8080" }
database_type = { description = "Database type", options = ["postgresql", "mysql", "sqlite"], default = "postgresql" }
team_name = { description = "Owning team name", example = "backend-team" }

[templating]
include_patterns = [
    "**/*.toml.template",
    "**/*.md.template",
    "src/**/*.rs",
    "Dockerfile"
]
exclude_patterns = [".git/**", "target/**", ".reporoller/**"]

[actions]
enable_workflows = ["ci.yml", "security-scan.yml", "deploy.yml"]

[[actions.create_issues]]
title = "Setup monitoring and alerting"
body = "Configure monitoring dashboards and alerting for the new service"
labels = ["enhancement", "monitoring", "priority-high"]
assignees = ["@{{team_name}}"]

[[actions.create_issues]]
title = "Review and update security configurations"
body = "Audit security settings and update as needed for production readiness"
labels = ["security", "priority-medium"]
assignees = ["@security-team"]
```

### Template Labels (`.reporoller/labels.toml`)

Optional template-specific labels.

```toml
[service-config]
color = "1d76db"
description = "Service configuration changes"

[monitoring]
color = "0052cc"
description = "Monitoring and observability"

[api-breaking]
color = "b60205"
description = "Breaking API changes"
```

## Rust Implementation Requirements

### Configuration Structs

The implementation must define the following Rust structs:

#### Override-aware Settings

```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OverridableValue<T> {
    pub value: T,
    pub override_allowed: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RepositorySettings {
    pub issues: Option<OverridableValue<bool>>,
    pub projects: Option<OverridableValue<bool>>,
    pub discussions: Option<OverridableValue<bool>>,
    pub wiki: Option<OverridableValue<bool>>,
    pub pages: Option<OverridableValue<bool>>,
    pub security_advisories: Option<OverridableValue<bool>>,
    pub vulnerability_reporting: Option<OverridableValue<bool>>,
    pub auto_close_issues: Option<OverridableValue<bool>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PullRequestSettings {
    pub allow_merge_commit: Option<OverridableValue<bool>>,
    pub allow_squash_merge: Option<OverridableValue<bool>>,
    pub allow_rebase_merge: Option<OverridableValue<bool>>,
    pub delete_branch_on_merge: Option<OverridableValue<bool>>,
    pub allow_auto_merge: Option<OverridableValue<bool>>,
    pub merge_commit_title: Option<OverridableValue<String>>,
    pub merge_commit_message: Option<OverridableValue<String>>,
    pub squash_merge_commit_title: Option<OverridableValue<String>>,
    pub squash_merge_commit_message: Option<OverridableValue<String>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PushSettings {
    pub max_branches_per_push: Option<OverridableValue<u32>>,
    pub max_tags_per_push: Option<OverridableValue<u32>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct WebhookConfig {
    pub url: String,
    pub content_type: Option<String>,
    pub secret: Option<String>,
    pub ssl_verification: Option<bool>,
    pub events: Vec<String>,
    pub active: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CustomProperty {
    pub name: String,
    pub value: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct EnvironmentConfig {
    pub name: String,
    pub protection_rules: Option<EnvironmentProtectionRules>,
    pub deployment_branch_policy: Option<DeploymentBranchPolicy>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct EnvironmentProtectionRules {
    pub required_reviewers: Option<Vec<String>>,
    pub wait_timer: Option<u32>,
    pub prevent_self_review: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DeploymentBranchPolicy {
    pub protected_branches: bool,
    pub custom_branch_policies: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GitHubAppConfig {
    pub app_id: u64,
    pub installation_id: Option<u64>,
    pub permissions: Option<HashMap<String, String>>,
    pub override_allowed: Option<bool>,
}
```

#### Enhanced Variable Definition

```rust
#[derive(Deserialize, Debug, Clone)]
pub struct VariableDefinition {
    pub description: String,
    pub example: Option<String>,
    pub default: Option<String>,
    pub options: Option<Vec<String>>,
    pub required: Option<bool>,
    pub pattern: Option<String>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
}
```

#### Rate Limiting Configuration

```rust
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
        }
    }
}
```

### Enhanced Error Handling

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("GitHub API error: {status} - {message}")]
    GitHubApi { status: u16, message: String },

    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    #[error("Team configuration not found: {0}")]
    TeamNotFound(String),

    #[error("Required variable not provided: {0}")]
    MissingVariable(String),

    #[error("Invalid variable value for {name}: {value} (reason: {reason})")]
    InvalidVariable {
        name: String,
        value: String,
        reason: String,
    },

    #[error("Configuration validation failed: {0}")]
    ValidationError(String),

    #[error("Override not allowed for setting: {setting} in {context}")]
    OverrideNotAllowed { setting: String, context: String },    #[error("Rate limit exceeded, retry after {retry_after_seconds} seconds")]
    RateLimitExceeded { retry_after_seconds: u64 },

    #[error("Label validation failed: {label_name} - {reason}")]
    LabelValidationError { label_name: String, reason: String },

    #[error("Webhook configuration invalid: {webhook_url} - {reason}")]
    WebhookValidationError { webhook_url: String, reason: String },

    #[error("Environment configuration invalid: {environment} - {reason}")]
    EnvironmentValidationError { environment: String, reason: String },

    #[error("GitHub App configuration invalid: {app_id} - {reason}")]
    GitHubAppValidationError { app_id: u64, reason: String },

    #[error("Required GitHub App missing: {app_id}")]
    RequiredGitHubAppMissing { app_id: u64 },

    #[error("Custom property validation failed: {property_name} - {reason}")]
    CustomPropertyValidationError { property_name: String, reason: String },
}
```

### Configuration Provider with Caching

```rust
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[async_trait]
pub trait ConfigurationProvider: Send + Sync {
    async fn load_global_defaults(&self) -> Result<GlobalDefaults, ConfigError>;
    async fn load_standard_labels(&self) -> Result<HashMap<String, LabelConfig>, ConfigError>;
    async fn load_team_config(&self, team_name: &str) -> Result<TeamConfig, ConfigError>;
    async fn load_team_labels(&self, team_name: &str) -> Result<HashMap<String, LabelConfig>, ConfigError>;
    async fn discover_templates(&self) -> Result<Vec<String>, ConfigError>;
    async fn load_template_config(&self, template_repo: &str) -> Result<TemplateConfig, ConfigError>;
    async fn load_template_labels(&self, template_repo: &str) -> Result<Option<HashMap<String, LabelConfig>>, ConfigError>;
}

pub struct CachedConfigurationProvider {
    inner: Arc<dyn ConfigurationProvider>,
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    cache_ttl: std::time::Duration,
}
```

#### Core Settings Structs

```rust
#[derive(Deserialize, Debug, Clone)]
pub struct RepositorySettings {
    pub issues: Option<bool>,
    pub projects: Option<bool>,
    pub discussions: Option<bool>,
    pub wiki: Option<bool>,
    pub pages: Option<bool>,
    pub security_advisories: Option<bool>,
    pub vulnerability_reporting: Option<bool>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PullRequestSettings {
    pub allow_merge_commit: Option<bool>,
    pub allow_squash_merge: Option<bool>,
    pub allow_rebase_merge: Option<bool>,
    pub delete_branch_on_merge: Option<bool>,
    pub require_conversation_resolution: Option<bool>,
    pub required_approving_review_count: Option<u32>,
    pub require_code_owner_reviews: Option<bool>,
    pub dismiss_stale_reviews: Option<bool>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BranchProtectionSettings {
    pub default_branch: Option<String>,
    pub require_pull_request_reviews: Option<bool>,
    pub required_approving_review_count: Option<u32>,
    pub dismiss_stale_reviews: Option<bool>,
    pub require_code_owner_reviews: Option<bool>,
    pub restrict_pushes: Option<bool>,
    pub require_status_checks: Option<bool>,
    pub required_status_checks: Option<Vec<String>>,
    pub require_up_to_date_before_merge: Option<bool>,
    pub additional_protected_patterns: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ActionSettings {
    pub enabled: Option<bool>,
    pub allow_github_owned_actions: Option<bool>,
    pub allow_verified_creator_actions: Option<bool>,
    pub allow_specified_actions: Option<bool>,
    pub allowed_actions: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct LabelConfig {
    pub color: String,
    pub description: Option<String>,
}
```

#### Configuration Container Structs

```rust
#[derive(Deserialize, Debug, Clone)]
pub struct GlobalDefaults {
    pub repository: Option<RepositorySettings>,
    pub pull_requests: Option<PullRequestSettings>,
    pub push: Option<PushSettings>,
    pub branch_protection: Option<BranchProtectionSettings>,
    pub actions: Option<ActionSettings>,
    pub webhooks: Option<Vec<WebhookConfig>>,
    pub custom_properties: Option<Vec<CustomProperty>>,
    pub environments: Option<Vec<EnvironmentConfig>>,
    pub github_apps: Option<Vec<GitHubAppConfig>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TeamConfig {
    pub repository: Option<RepositorySettings>,
    pub pull_requests: Option<PullRequestSettings>,
    pub push: Option<PushSettings>,
    pub branch_protection: Option<BranchProtectionSettings>,
    pub actions: Option<ActionSettings>,
    pub webhooks: Option<Vec<WebhookConfig>>,
    pub custom_properties: Option<Vec<CustomProperty>>,
    pub environments: Option<Vec<EnvironmentConfig>>,
    pub github_apps: Option<Vec<GitHubAppConfig>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TemplateConfig {
    pub template: TemplateMetadata,
    pub repository: Option<RepositorySettings>,
    pub pull_requests: Option<PullRequestSettings>,
    pub push: Option<PushSettings>,
    pub branch_protection: Option<BranchProtectionSettings>,
    pub actions: Option<ActionSettings>,
    pub webhooks: Option<Vec<WebhookConfig>>,
    pub custom_properties: Option<Vec<CustomProperty>>,
    pub environments: Option<Vec<EnvironmentConfig>>,
    pub github_apps: Option<Vec<GitHubAppConfig>>,
    pub variables: Option<HashMap<String, VariableDefinition>>,
    pub templating: Option<TemplatingConfig>,
    pub actions_config: Option<PostCreationActions>,
}
```

#### Template-Specific Structs

```rust
#[derive(Deserialize, Debug, Clone)]
pub struct TemplateMetadata {
    pub name: String,
    pub description: String,
    pub version: Option<String>,
    pub author: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct VariableDefinition {
    pub description: String,
    pub example: Option<String>,
    pub default: Option<String>,
    pub options: Option<Vec<String>>,
    pub required: Option<bool>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TemplatingConfig {
    pub include_patterns: Option<Vec<String>>,
    pub exclude_patterns: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PostCreationActions {
    pub enable_workflows: Option<Vec<String>>,
    pub create_issues: Option<Vec<IssueTemplate>>,
    pub create_discussions: Option<Vec<DiscussionTemplate>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct IssueTemplate {
    pub title: String,
    pub body: String,
    pub labels: Option<Vec<String>>,
    pub assignees: Option<Vec<String>>,
    pub milestone: Option<String>,
}
```

### Configuration Provider Trait

The implementation must provide a trait for loading configurations:

```rust
pub trait ConfigurationProvider: Send + Sync {
    fn load_global_defaults(&self) -> Result<GlobalDefaults, ConfigError>;
    fn load_standard_labels(&self) -> Result<HashMap<String, LabelConfig>, ConfigError>;
    fn load_team_config(&self, team_name: &str) -> Result<TeamConfig, ConfigError>;
    fn load_team_labels(&self, team_name: &str) -> Result<HashMap<String, LabelConfig>, ConfigError>;
    fn discover_templates(&self) -> Result<Vec<String>, ConfigError>;
    fn load_template_config(&self, template_repo: &str) -> Result<TemplateConfig, ConfigError>;
    fn load_template_labels(&self, template_repo: &str) -> Result<Option<HashMap<String, LabelConfig>>, ConfigError>;
}
```

### Configuration Merger

The implementation must provide a configuration merger that combines all three levels:

```rust
pub struct ConfigurationMerger;

impl ConfigurationMerger {
    pub fn merge_configuration(
        global_defaults: GlobalDefaults,
        global_labels: HashMap<String, LabelConfig>,
        team_config: Option<TeamConfig>,
        team_labels: Option<HashMap<String, LabelConfig>>,
        template_config: TemplateConfig,
        template_labels: Option<HashMap<String, LabelConfig>>,
        variable_values: HashMap<String, String>,
    ) -> Result<MergedConfig, ConfigError>;
}

#[derive(Debug, Clone)]
pub struct MergedConfig {
    pub template: TemplateMetadata,
    pub repository: RepositorySettings,
    pub pull_requests: PullRequestSettings,
    pub push: PushSettings,
    pub branch_protection: BranchProtectionSettings,
    pub actions: ActionSettings,
    pub webhooks: Vec<WebhookConfig>,
    pub custom_properties: Vec<CustomProperty>,
    pub environments: Vec<EnvironmentConfig>,
    pub github_apps: Vec<GitHubAppConfig>,
    pub labels: HashMap<String, LabelConfig>,
    pub variables: HashMap<String, String>,
    pub templating: Option<TemplatingConfig>,
    pub post_creation_actions: Option<PostCreationActions>,
}
```

## Configuration Merging Logic

The merger must implement the following hierarchy (highest to lowest precedence):

1. **Template Configuration**: Settings from `.reporoller/template.toml`
2. **Team Configuration**: Settings from `teams/{team-name}/config.toml`
3. **Global Defaults**: Settings from `global/defaults.toml`

### Label Merging

Labels are merged additively:

1. Start with global standard labels
2. Add team-specific labels (overwrites global if same name)
3. Add template-specific labels (overwrites team/global if same name)

### Field Merging Rules

- For `Option<T>` fields: Use the highest precedence `Some(value)`, fall back to lower precedence
- For `Vec<T>` fields: Merge by concatenating, with higher precedence values appearing first
- For `HashMap<K, V>` fields: Merge by key, with higher precedence values overwriting lower

### GitHub Apps Merging

GitHub Apps have special merging rules to enforce security policies:

1. **Global Apps with `override_allowed = false`**: Must be included in final configuration
2. **Global Apps with `override_allowed = true`**: Can be overridden by team/template settings
3. **Team/Template Apps**: Added to the final configuration if not conflicting with required global apps
4. **Conflict Resolution**: Apps with same `app_id` use highest precedence configuration
5. **Required Apps Validation**: Ensure all non-overridable global apps are present in final configuration

### Override Validation

Before merging, the system validates override permissions:

1. **Check Override Allowed**: Verify that team/template configurations only override settings where `override_allowed = true`
2. **Security Policy Enforcement**: Reject configurations that attempt to override security-critical settings
3. **Error Reporting**: Provide detailed error messages for invalid override attempts

### Complex Field Merging

#### Webhooks

- **Merge Strategy**: Concatenate all webhook configurations from global, team, and template
- **Deduplication**: Remove duplicates based on URL and event combination
- **Variable Substitution**: Apply variable substitution to webhook URLs and secrets

#### Custom Properties

- **Merge Strategy**: Merge by property name, with higher precedence overwriting lower
- **Validation**: Ensure property names are valid according to GitHub requirements

#### Environments

- **Merge Strategy**: Merge by environment name, with higher precedence overwriting lower
- **Protection Rules**: Merge protection rules additively (combine reviewers, use most restrictive timers)
- **Branch Policies**: Use highest precedence branch policy configuration

## Variable Substitution

The system supports variable substitution in template files and configuration with the following characteristics:

### Variable Syntax

- **Standard Variables**: `{{variable_name}}`
- **Escaping**: Use `\{{` to include literal `{{` in template content
- **Undefined Variables**: Left as-is with a warning logged

### Built-in Variables

The system provides the following built-in variables:

```toml
# Available in all templates
{{timestamp}}           # ISO 8601 timestamp of repository creation
{{timestamp_unix}}      # Unix timestamp of repository creation
{{user_login}}          # GitHub username of the person creating the repository
{{user_name}}           # Display name of the person creating the repository
{{org_name}}            # Organization name where repository is being created
{{repo_name}}           # Name of the new repository being created
{{template_name}}       # Name of the template being used
{{template_repo}}       # Full name of the template repository (org/repo)
{{default_branch}}      # Default branch name from configuration
```

### Variable Validation Rules

Template variables support the following validation:

- **Required**: Variables that must be provided (default: false)
- **Options**: Restricted list of allowed values
- **Pattern**: Regex pattern for validation
- **Min/Max Length**: String length constraints
- **Default**: Default value if not provided

Example variable definition:

```toml
[variables]
service_name = {
    description = "Name of the microservice",
    example = "user-service",
    required = true,
    pattern = "^[a-z][a-z0-9-]*[a-z0-9]$",
    min_length = 3,
    max_length = 50
}
## Template Processing

### File Pattern Matching

Template processing uses glob patterns for file inclusion and exclusion:

- **Include Patterns**: Standard glob syntax (e.g., `**/*.rs`, `src/**/*.toml.template`)
- **Exclude Patterns**: Standard glob syntax (e.g., `target/**`, `.git/**`)
- **Precedence**: Exclude patterns override include patterns

### File Type Handling

#### Template Files (`.template` suffix)
- **Processing**: Variable substitution applied to file contents
- **Renaming**: `.template` suffix removed (e.g., `config.toml.template` → `config.toml`)
- **Encoding**: UTF-8 encoding assumed for text processing

#### Regular Text Files
- **Processing**: Variable substitution applied if matching include patterns
- **Preservation**: File names and extensions preserved exactly

#### Binary Files
- **Processing**: Copied without modification
- **Detection**: Files that are not valid UTF-8 are treated as binary

#### Special Files
- **Symlinks**: Copied as-is (symlink preserved)
- **Executable Files**: File permissions preserved during copy
- **Hidden Files**: Processed according to include/exclude patterns

### Template Discovery

Templates are discovered by:

1. **Repository Scanning**: Finding repositories marked as template repositories in the organization
2. **Configuration Validation**: Ensuring `.reporoller/template.toml` exists and is valid
3. **Topic Filtering**: Optionally filtering by repository topics
4. **Access Verification**: Confirming the app has read access to the template repository

## GitHub API Integration

### Supported Configuration Fields

The system supports the following GitHub repository settings with their corresponding API endpoints:

#### Repository Settings (`PATCH /repos/{owner}/{repo}`)
- `has_issues` → `issues`
- `has_projects` → `projects`
- `has_discussions` → `discussions`
- `has_wiki` → `wiki`
- `has_pages` → `pages`
- `security_and_analysis.secret_scanning.status` → `security_advisories`
- `security_and_analysis.secret_scanning_push_protection.status` → `vulnerability_reporting`
- `auto_close_issues` → Auto-close issues when linked pull requests are merged

#### Pull Request Settings (`PATCH /repos/{owner}/{repo}`)
- `allow_merge_commit` → `allow_merge_commit`
- `allow_squash_merge` → `allow_squash_merge`
- `allow_rebase_merge` → `allow_rebase_merge`
- `delete_branch_on_merge` → `delete_branch_on_merge`
- `merge_commit_title` → Default commit message format for merge commits
- `merge_commit_message` → Default commit message content for merge commits
- `squash_merge_commit_title` → Default commit message format for squash merges
- `squash_merge_commit_message` → Default commit message content for squash merges
- `allow_auto_merge` → Enable auto-merge for pull requests

#### Push Settings (`PATCH /repos/{owner}/{repo}`)
- `push_policy.restrict_to_branches` → Limit branch updates per push
- `push_policy.restrict_to_tags` → Limit tag updates per push
- `push_policy.max_branches_per_push` → Maximum branches that can be updated in a single push
- `push_policy.max_tags_per_push` → Maximum tags that can be updated in a single push

#### Branch Protection (`PUT /repos/{owner}/{repo}/branches/{branch}/protection`)
- `required_status_checks` → `require_status_checks`, `required_status_checks`
- `enforce_admins` → `restrict_pushes`
- `required_pull_request_reviews` → Pull request review settings
- `restrictions` → Push restrictions

#### Actions Settings (`PUT /repos/{owner}/{repo}/actions/permissions`)
- `enabled` → Actions enabled/disabled
- `allowed_actions` → Action permission levels

#### Webhooks (`POST /repos/{owner}/{repo}/hooks`)
- `webhooks` → List of webhook configurations with URLs, events, and settings
- Each webhook includes: URL, content type, secret, SSL verification, events list

#### Custom Properties (`PATCH /repos/{owner}/{repo}/custom-properties`)
- `custom_properties` → Key-value pairs for repository-specific custom properties
- Property definitions must exist at organization level before assignment

#### Environments (`PUT /repos/{owner}/{repo}/environments/{environment_name}`)
- `environments` → List of deployment environments with protection rules
- Each environment includes: protection rules, reviewers, deployment branches

#### GitHub Apps Integration (`POST /repos/{owner}/{repo}/installations`)
- `github_apps` → List of GitHub App installations to enable for the repository
- Each app includes: app ID, installation settings, permissions

### Rate Limiting & Retry Logic

#### Rate Limiting Strategy
- **Primary Rate Limit**: GitHub Apps have 5,000 requests per hour per installation
- **Secondary Rate Limits**: Respect `X-RateLimit-Remaining` and `Retry-After` headers
- **Abuse Detection**: Back off exponentially if receiving 403 responses

#### Retry Logic
- **Retry Attempts**: Maximum 3 retries for failed requests
- **Backoff Strategy**: Exponential backoff starting at 1 second
- **Retryable Errors**: 429 (rate limited), 502/503/504 (server errors)
- **Non-Retryable Errors**: 401 (authentication), 403 (forbidden), 404 (not found)

#### Paid Feature Handling
- **Detection**: Monitor for 422 responses indicating unavailable features
- **Behavior**: Log warning and continue with repository creation
- **Documentation**: Clearly document which features require paid plans

## Error Handling & Recovery

### Error Recovery Strategy

#### Repository Creation Failures
- **Approach**: Fail fast and error out - no automatic rollback
- **Rationale**: Human intervention required to assess partial state and decide on cleanup
- **User Communication**: Provide detailed error messages with specific failure points

#### Observability & Monitoring
- **Logging**: Structured logging with correlation IDs for tracing repository creation flows
- **Metrics**: Track success/failure rates, processing times, and API usage
- **Tracing**: Distributed tracing for complex operations spanning multiple services

#### Error Escalation
- **Critical Errors**: Create GitHub issues in the metadata repository for investigation
- **Warning Events**: Log warnings for non-critical failures (e.g., label color validation)
- **User Feedback**: Provide actionable error messages to users

#### Retry Logic Implementation
- **Template Processing**: Retry file processing operations for transient failures
- **API Calls**: Implement exponential backoff for GitHub API failures
- **Configuration Loading**: Retry configuration fetches for network issues

## Label Management

### Label Application Strategy

#### Label Replacement
- **Approach**: Delete all existing labels and replace with merged configuration labels
- **Rationale**: Ensures consistent labeling across repositories created from templates
- **API Calls**: `DELETE /repos/{owner}/{repo}/labels/{name}` followed by `POST /repos/{owner}/{repo}/labels`

#### Label Validation

##### Color Validation
- **Format**: Must be valid 6-character hexadecimal color (without # prefix)
- **Example**: `"ff0000"` for red, `"00ff00"` for green
- **Failure Handling**: Log warning and continue, optionally create issue in source repository

##### Name Validation
- **Length**: Maximum 50 characters
- **Characters**: Alphanumeric, hyphens, underscores, spaces allowed
- **Uniqueness**: Names must be unique within the repository (case-insensitive)
- **Reserved Names**: Avoid GitHub reserved label names

#### Error Handling
- **Color Setting Failures**: Log error and continue with repository creation
- **Invalid Label Source**: Create issue in the repository where the invalid label is defined
- **API Failures**: Retry with exponential backoff for transient failures

## Post-Creation Actions

### Execution Model

#### Timing & Order
- **Sequential Execution**: Actions execute in the order defined in the template configuration
- **Synchronous Processing**: Each action completes before the next begins
- **Timeout Handling**: Individual actions have a 30-second timeout limit

#### Action Types

##### Workflow Enablement
- **GitHub Actions**: Enable specific workflow files in `.github/workflows/`
- **API Call**: `PUT /repos/{owner}/{repo}/actions/workflows/{workflow_id}/enable`
- **Validation**: Verify workflow files exist in the repository

##### Issue Creation
- **Batch Processing**: Create multiple issues from template definitions
- **Variable Substitution**: Apply variable substitution to titles and bodies
- **Label Application**: Apply configured labels to created issues
- **Assignee Resolution**: Resolve team names to individual GitHub usernames

##### Discussion Creation
- **Category Selection**: Use repository's default discussion category
- **Content Processing**: Apply variable substitution to discussion content

### Team Name Resolution

#### Resolution Strategy
- **Team API**: Use GitHub Teams API to resolve `@team-name` to member usernames
- **API Call**: `GET /orgs/{org}/teams/{team_slug}/members`
- **Fallback**: If team resolution fails, leave assignee as team name and log warning
- **Caching**: Cache team member lists for the duration of repository creation

#### Variable Format
- **Team Reference**: `@{{team_name}}` in assignee fields
- **Individual User**: Direct username without @ prefix
- **Mixed Assignment**: Support both team references and individual usernames

## Configuration Validation

### Validation Strategy

#### GitHub Actions Integration
- **Metadata Repository**: GitHub Actions workflows validate global and team configurations
- **Template Repositories**: GitHub Actions workflows validate template configurations
- **Schema Validation**: Use JSON Schema to validate configuration file structure
- **Business Rule Validation**: Custom validation for organizational policies

#### Validation Triggers
- **Pull Request Reviews**: Validation runs on all configuration changes
- **Scheduled Validation**: Periodic validation of all configurations
- **Pre-Creation Validation**: Validate merged configuration before repository creation

## Concurrency & State Management

### Concurrent Repository Creation

#### Locking Strategy
- **Template-Level Locking**: No locking required - templates are read-only during creation
- **Configuration Caching**: Cache configurations for the duration of repository creation
- **Parallel Processing**: Multiple users can create repositories simultaneously from the same template

#### State Consistency
- **Configuration Snapshots**: Use point-in-time configuration snapshots for each creation
- **Change Detection**: Detect configuration changes during active repository creation
- **Graceful Degradation**: Continue with cached configuration if updates fail during creation

#### Resource Management
- **GitHub API Limits**: Distribute API calls across available rate limit quota
- **Processing Queues**: Use queues for handling multiple concurrent repository creations
- **Timeout Handling**: Implement reasonable timeouts for all external operations

## Usage Flow

1. **Template Discovery**: System scans for template repositories
2. **User Selection**: User selects template and team
3. **Configuration Loading**:
   - Load global defaults and labels
   - Load team config and labels (if team specified)
   - Load template config and labels
4. **Configuration Merging**: Merge all configurations according to precedence rules
5. **Variable Collection**: Prompt user for required template variables with validation
6. **Repository Creation**: Create repository with merged configuration
7. **Template Processing**: Process template files with variable substitution
8. **Post-Creation Actions**: Execute workflows, create issues, and discussions
9. **Error Handling**: Log results and create issues for any failures
8. **Post-Creation Actions**: Execute workflows, create issues, etc.

## Testing Requirements

The implementation must include tests for:

### Unit Tests
- Loading each configuration file type with override controls
- Configuration merging with various precedence scenarios and override validation
- Variable substitution with escaping and built-in variables
- Variable validation with all constraint types (pattern, length, options)
- Error handling for missing/invalid configurations
- Template discovery and validation
- Label validation and color format checking
- Rate limiting and retry logic
- Team name resolution for post-creation actions

### Integration Tests
- GitHub API integration with rate limiting
- Template processing with file type handling
- Post-creation action execution
- Configuration caching and invalidation
- Concurrent repository creation scenarios

### End-to-End Tests
- Complete repository creation flow from template discovery to post-creation actions
- Error recovery and partial failure scenarios
- Cross-configuration validation (global, team, template)

## Future Considerations

- Support for JSON configuration files alongside TOML
- Configuration validation using JSON Schema
- Web UI for managing team configurations
- Template versioning and updates
- Audit logging for configuration changes
- Configuration inheritance for nested teams
- Database backend for configuration storage
- Cross-organization template sharing
- Template marketplace and discovery
- Advanced variable substitution with conditionals and loops
