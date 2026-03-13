# Configuration Management Interfaces

**Architectural Layer**: Business Interface (trait) + Infrastructure (implementation)
**Crate**: `config_manager`
**Responsibilities**:

- **Knows**: Template definitions, variable configurations, hierarchical config resolution
- **Does**: Loads configuration, validates templates, merges config sources

---

## Overview

Configuration management provides template definitions and repository settings. It supports hierarchical configuration (global, organization, repository level) and template variable validation.

## Current State

**Temporary Compatibility Stubs** (added for backward compatibility):

- `Config` struct with templates list
- `TemplateConfig` struct with template settings
- `VariableConfig` struct for variable validation

**TODO**: Replace with proper trait-based design defined below.

---

## Interface Trait

```rust
/// Configuration management service
#[async_trait]
pub trait ConfigurationManager: Send + Sync {
    /// Get template configuration by name
    async fn get_template(&self, name: &TemplateName)
        -> Result<TemplateConfiguration, ConfigError>;

    /// List all available templates
    async fn list_templates(&self) -> Result<Vec<TemplateName>, ConfigError>;

    /// Get organization-specific configuration
    async fn get_org_config(&self, org: &OrganizationName)
        -> Result<OrganizationConfiguration, ConfigError>;
}
```

---

## Types

### TemplateConfiguration

```rust
pub struct TemplateConfiguration {
    pub name: TemplateName,
    pub source_repo: String,
    pub description: Option<String>,
    pub variable_configs: HashMap<String, VariableConfiguration>,
    pub features: Vec<String>,
}
```

### VariableConfiguration

```rust
pub struct VariableConfiguration {
    pub description: Option<String>,
    pub required: bool,
    pub pattern: Option<String>,
    pub default: Option<String>,
    pub example: Option<String>,
}
```

### OrganizationConfiguration

> **Note**: The `OrganizationConfiguration` stub below is superseded by the
> production types implemented in `config_manager`. Refer to the following
> source files for the authoritative definitions:
>
> - `crates/config_manager/src/global_defaults.rs` — `GlobalDefaults` (org-level defaults)
> - `crates/config_manager/src/settings/naming.rs` — `RepositoryNamingRulesConfig`
> - `crates/config_manager/src/merged_config.rs` — `MergedConfiguration`
>
> Naming rules are **not** a single regex string. They are an additive
> collection of `RepositoryNamingRulesConfig` entries, each of which may
> specify any combination of: `allowed_pattern`, `forbidden_patterns`,
> `reserved_words`, `required_prefix`, `required_suffix`, `min_length`, and
> `max_length`. Rules from all hierarchy levels (global → type → team →
> template) are concatenated; every rule in the merged set must be satisfied.

```rust
// Superseded — see global_defaults.rs and settings/naming.rs
pub struct OrganizationConfiguration {
    pub default_branch: String,
    // OUTDATED: was Option<String> (single regex).
    // Actual type: Vec<RepositoryNamingRulesConfig> (additive, multi-field rules).
    pub naming_rules: Vec<RepositoryNamingRulesConfig>,
    pub required_apps: Vec<String>,
}
```

---

## Implementation Notes

- Load from TOML files (file system implementation)
- Future: Load from database or API (different implementations)
- Support environment-specific overrides
- Validate configuration on load

---

## Related Specifications

- `specs/interfaces/repository-domain.md` - Uses configuration
- `specs/interfaces/shared-types.md` - Type definitions
