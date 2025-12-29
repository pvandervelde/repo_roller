# CLI Template Operations Interface

**Architectural Layer**: User Interface (CLI)
**Module Path**: `crates/repo_roller_cli/src/commands/template_cmd.rs`
**Responsibilities** (from RDD):

- **Knows**: Template listing and validation requirements, CLI output formatting patterns
- **Does**: Discovers templates from organization, validates template structure and configuration, formats output for CLI consumption

## Overview

This interface defines CLI-specific operations for template discovery, inspection, and validation. The CLI layer provides a user-friendly interface to template operations, translating business domain types into CLI-appropriate formats.

**Key Capabilities**:

- **Template Discovery**: List all available templates in an organization
- **Template Information**: Display detailed template metadata, variables, and configuration
- **Template Validation**: Verify template structure and configuration correctness
- **Output Formatting**: Support both JSON and human-readable formats

## Dependencies

- **Business Types**: `TemplateConfig`, `TemplateMetadata`, `TemplateVariable` ([template_config.rs](../../crates/config_manager/src/template_config.rs))
- **Business Interfaces**: `MetadataRepositoryProvider` trait ([metadata_provider.rs](../../crates/config_manager/src/metadata_provider.rs))
- **Errors**: `crate::errors::Error` - CLI-specific error type
- **Standard Types**: `Vec`, `String`, `HashMap`

## Type Definitions

### TemplateInfo

CLI-friendly representation of template information for display and JSON output.

```rust
/// Template information for CLI display.
///
/// This is a CLI-specific view that combines template metadata with
/// configuration details in a format suitable for command-line output.
///
/// See: specs/interfaces/cli-template-operations.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInfo {
    /// Template name (repository name).
    pub name: String,

    /// Human-readable description.
    pub description: String,

    /// Template author or owning team.
    pub author: String,

    /// Tags for categorization.
    pub tags: Vec<String>,

    /// Repository type this template creates (if specified).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_type: Option<RepositoryTypeInfo>,

    /// Template variables that users must provide.
    pub variables: Vec<TemplateVariableInfo>,

    /// Number of configuration sections defined.
    pub configuration_sections: usize,
}

/// Repository type information for CLI display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryTypeInfo {
    /// Repository type name.
    pub type_name: String,

    /// Policy: "fixed" or "preferable".
    pub policy: String,
}

/// Template variable information for CLI display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariableInfo {
    /// Variable name.
    pub name: String,

    /// Variable description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Whether variable is required.
    pub required: bool,

    /// Default value (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,

    /// Example value (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
}
```

### TemplateValidationResult

Result of template validation with detailed diagnostics.

```rust
/// Result of template validation.
///
/// Contains validation status and detailed diagnostics about any issues found.
///
/// See: specs/interfaces/cli-template-operations.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateValidationResult {
    /// Template name being validated.
    pub template_name: String,

    /// Overall validation status.
    pub valid: bool,

    /// Validation issues found (empty if valid).
    pub issues: Vec<ValidationIssue>,

    /// Warnings that don't prevent use but should be addressed.
    pub warnings: Vec<ValidationWarning>,
}

/// Individual validation issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Issue severity: "error" or "warning".
    pub severity: String,

    /// Location of issue (e.g., "template.toml", "variables.service_name").
    pub location: String,

    /// Human-readable issue description.
    pub message: String,
}

/// Validation warning (non-blocking).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// Warning category (e.g., "best_practice", "deprecated").
    pub category: String,

    /// Warning message.
    pub message: String,
}
```

## Function Signatures

### list_templates

```rust
/// List all available templates for an organization.
///
/// Discovers template repositories and loads their configurations
/// to provide summary information.
///
/// # Arguments
///
/// * `org` - Organization name
/// * `provider` - Metadata repository provider (authenticated)
///
/// # Returns
///
/// Returns a vector of `TemplateInfo` objects, one per discovered template.
/// Returns empty vector if no templates found.
///
/// # Errors
///
/// * `Error::Auth` - Authentication failed or GitHub App not configured
/// * `Error::Config` - Failed to load template configurations
/// * `Error::GitHub` - GitHub API errors during template discovery
///
/// # Example
///
/// ```no_run
/// # use repo_roller_cli::commands::template_cmd::{list_templates, TemplateInfo};
/// # use config_manager::MetadataRepositoryProvider;
/// # use std::sync::Arc;
/// # async fn example(provider: Arc<dyn MetadataRepositoryProvider>) -> Result<(), Box<dyn std::error::Error>> {
/// let templates = list_templates("myorg", provider).await?;
/// for template in templates {
///     println!("{}: {}", template.name, template.description);
/// }
/// # Ok(())
/// # }
/// ```
///
/// See: specs/interfaces/cli-template-operations.md
pub async fn list_templates(
    org: &str,
    provider: Arc<dyn MetadataRepositoryProvider>,
) -> Result<Vec<TemplateInfo>, Error>
```

### get_template_info

```rust
/// Get detailed information about a specific template.
///
/// Loads the complete template configuration and formats it for CLI display.
///
/// # Arguments
///
/// * `org` - Organization name
/// * `template_name` - Template repository name
/// * `provider` - Metadata repository provider (authenticated)
///
/// # Returns
///
/// Returns complete `TemplateInfo` for the specified template.
///
/// # Errors
///
/// * `Error::Auth` - Authentication failed
/// * `Error::Config` - Template not found or configuration invalid
/// * `Error::GitHub` - GitHub API errors
///
/// # Example
///
/// ```no_run
/// # use repo_roller_cli::commands::template_cmd::{get_template_info, TemplateInfo};
/// # use config_manager::MetadataRepositoryProvider;
/// # use std::sync::Arc;
/// # async fn example(provider: Arc<dyn MetadataRepositoryProvider>) -> Result<(), Box<dyn std::error::Error>> {
/// let info = get_template_info("myorg", "rust-library", provider).await?;
/// println!("Template: {} ({})", info.name, info.description);
/// println!("Variables: {}", info.variables.len());
/// # Ok(())
/// # }
/// ```
///
/// See: specs/interfaces/cli-template-operations.md
pub async fn get_template_info(
    org: &str,
    template_name: &str,
    provider: Arc<dyn MetadataRepositoryProvider>,
) -> Result<TemplateInfo, Error>
```

### validate_template

```rust
/// Validate a template's structure and configuration.
///
/// Performs comprehensive validation including:
/// - Template repository accessibility
/// - `.reporoller/template.toml` existence and parse validity
/// - Required metadata fields presence
/// - Variable definition completeness
/// - Repository type reference validity (if type specified)
/// - Configuration consistency
///
/// # Arguments
///
/// * `org` - Organization name
/// * `template_name` - Template repository name
/// * `provider` - Metadata repository provider (authenticated)
///
/// # Returns
///
/// Returns `TemplateValidationResult` with validation status and any issues found.
///
/// # Errors
///
/// * `Error::Auth` - Authentication failed
/// * `Error::GitHub` - GitHub API errors (network, permissions)
///
/// Note: Template configuration errors are returned in the validation result,
/// not as function errors.
///
/// # Example
///
/// ```no_run
/// # use repo_roller_cli::commands::template_cmd::{validate_template, TemplateValidationResult};
/// # use config_manager::MetadataRepositoryProvider;
/// # use std::sync::Arc;
/// # async fn example(provider: Arc<dyn MetadataRepositoryProvider>) -> Result<(), Box<dyn std::error::Error>> {
/// let result = validate_template("myorg", "rust-library", provider).await?;
/// if result.valid {
///     println!("✓ Template is valid");
/// } else {
///     println!("✗ Validation failed:");
///     for issue in result.issues {
///         println!("  - {}: {}", issue.location, issue.message);
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// See: specs/interfaces/cli-template-operations.md
pub async fn validate_template(
    org: &str,
    template_name: &str,
    provider: Arc<dyn MetadataRepositoryProvider>,
) -> Result<TemplateValidationResult, Error>
```

## Translation Functions

### TemplateConfig to TemplateInfo

```rust
/// Convert business domain `TemplateConfig` to CLI `TemplateInfo`.
///
/// # Arguments
///
/// * `config` - Template configuration from business domain
///
/// # Returns
///
/// Returns CLI-friendly `TemplateInfo` representation.
///
/// See: specs/interfaces/cli-template-operations.md
fn template_config_to_info(config: TemplateConfig) -> TemplateInfo
```

## Validation Rules

### Template Validation Checks

**Required Elements**:

- Template repository must be accessible
- `.reporoller/template.toml` must exist
- `[template]` section with name, description, author must be present
- Template name in TOML must match repository name

**Variable Validation**:

- Variable names must be valid identifiers (alphanumeric + underscore)
- Required variables cannot have default values
- Example values should be provided for complex variables

**Repository Type Validation**:

- If `repository_type` specified, type must exist in organization configuration
- Policy must be "fixed" or "preferable"

**Configuration Validation**:

- No conflicting settings between sections
- All referenced labels, webhooks, apps are properly defined

### Warning Conditions

**Best Practice Warnings**:

- Template has no description or empty description
- No tags defined for categorization
- No variables defined (template not customizable)
- Required variables without examples

**Deprecation Warnings**:

- Template configuration uses deprecated fields
- Template references deprecated repository types

## Output Formatting

### JSON Format

```json
{
  "name": "rust-library",
  "description": "Rust library template with CI/CD",
  "author": "Platform Team",
  "tags": ["rust", "library", "cargo"],
  "repository_type": {
    "type_name": "library",
    "policy": "fixed"
  },
  "variables": [
    {
      "name": "project_name",
      "description": "Human-readable project name",
      "required": true,
      "default_value": null,
      "example": "my-awesome-library"
    }
  ],
  "configuration_sections": 3
}
```

### Pretty Format (Human-Readable)

```
Template: rust-library
Author: Platform Team
Description: Rust library template with CI/CD
Tags: rust, library, cargo

Repository Type:
  Type: library
  Policy: fixed (cannot be overridden)

Variables (1):
  ✓ project_name [required]
    Human-readable project name
    Example: my-awesome-library

Configuration: 3 sections defined
```

### Validation Result Pretty Format

```
Validating template: rust-library

✓ Template repository accessible
✓ Configuration file found (.reporoller/template.toml)
✓ Template metadata complete
✓ Variables valid (1 defined)
✓ Repository type references valid

Template is VALID

Warnings (1):
  ⚠ Best Practice: Consider adding more tags for better discoverability
```

## Error Handling

### Authentication Errors

```rust
Error::Auth("GitHub App credentials not configured. Run 'repo-roller auth setup'.")
```

### Template Not Found

```rust
Error::Config("Template 'rust-library' not found in organization 'myorg'")
```

### Configuration Parse Error

```rust
Error::Config("Failed to parse template.toml: missing required field 'template.name' at line 5")
```

### GitHub API Errors

```rust
Error::GitHub("GitHub API rate limit exceeded. Retry after: 2024-12-29T15:30:00Z")
```

## Implementation Notes

### Caching Considerations

- Template list should be cached for short duration (5 minutes) in CLI context
- Individual template info can be cached based on template repository commit SHA
- Validation results should not be cached (always run fresh)

### Performance Requirements

- `list_templates()` should complete in <5 seconds for organizations with 50 templates
- `get_template_info()` should complete in <2 seconds per template
- `validate_template()` should complete in <3 seconds per template

### Security Considerations

- Never display GitHub tokens in output (even in JSON mode)
- Sanitize error messages to prevent information leakage
- Validate all user input (org names, template names) before use

### Accessibility

- Use clear, actionable error messages
- Color-coded output should degrade gracefully (no color mode)
- Table formatting should be consistent and readable
- JSON output should be valid and parseable by standard tools

## Testing Requirements

### Unit Tests

- Test `template_config_to_info()` translation with various configurations
- Test validation logic with valid and invalid templates
- Test output formatting (JSON and pretty) with sample data
- Test error handling paths

### Integration Tests

- Test against real templates in glitchgrove organization
- Verify authentication flow end-to-end
- Test with templates missing various configuration elements
- Verify validation catches real configuration errors

### Test Scenarios

1. **Happy Path**: List templates, get info, validate valid template
2. **No Templates**: Organization with no templates
3. **Invalid Template**: Template with malformed configuration
4. **Missing Configuration**: Template without `.reporoller/template.toml`
5. **Network Errors**: Handle GitHub API failures gracefully
6. **Auth Failures**: Handle missing or invalid credentials

## Future Enhancements

**Potential additions** (not in current scope):

- Template comparison (`template diff <template1> <template2>`)
- Template usage statistics (how many repos created from template)
- Template update checking (notify if template has updates)
- Template scaffolding (create new template from scratch)
- Batch validation (validate all templates in organization)

## Related Specifications

- [Template Loading Interface](template-loading.md) - Business layer template configuration loading
- [Configuration Interfaces](configuration-interfaces.md) - Configuration hierarchy and merging
- [API Response Types](api-response-types.md) - Similar types for REST API (compare for consistency)
- [Shared Types](shared-types.md) - Core domain types

---

**Implementation Status**: ✅ Interface design complete, ready for implementation
**Last Updated**: December 29, 2024
