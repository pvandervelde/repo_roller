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

```rust
pub struct OrganizationConfiguration {
    pub default_branch: String,
    pub naming_rules: Option<String>, // Regex for repository names
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
