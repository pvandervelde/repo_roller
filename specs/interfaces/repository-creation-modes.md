# Repository Creation Modes Specification

**Status**: Interface Design Complete
**Version**: 1.0
**Last Updated**: 2026-01-10

## Overview

This specification defines the content generation strategy system for repository creation. It introduces the `ContentStrategy` enum that determines how repository content is generated, and documents the changes to `RepositoryCreationRequest` to support optional templates.

## Architecture Context

**Layer**: Core Domain
**Module Path**: `crates/repo_roller_core/src/request.rs`
**Dependencies**:

- `TemplateName` (template.rs)
- `OrganizationName` (repository.rs)
- `RepositoryName` (repository.rs)
- `RepositoryVisibility` (visibility.rs)

## ContentStrategy Enum

### Type Definition

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentStrategy {
    /// Fetch and process files from template repository
    Template,

    /// Create no files (empty repository)
    #[serde(rename = "empty")]
    Empty,

    /// Create custom initialization files
    #[serde(rename = "custom_init")]
    CustomInit {
        /// Create README.md file
        include_readme: bool,

        /// Create .gitignore file
        include_gitignore: bool,
    },
}
```

### Default Behavior

```rust
impl Default for ContentStrategy {
    fn default() -> Self {
        Self::Template
    }
}
```

**Rationale**: Template-based creation is the primary use case and maintains backward compatibility.

### Serialization Format

#### JSON Examples

**Template Strategy**:

```json
{
  "type": "template"
}
```

**Empty Strategy**:

```json
{
  "type": "empty"
}
```

**Custom Init Strategy**:

```json
{
  "type": "custom_init",
  "include_readme": true,
  "include_gitignore": true
}
```

### Variants

#### Template

**Purpose**: Fetch and process files from template repository (current/default behavior).

**Requirements**:

- `template_name` must be `Some` with valid template
- Template must exist in organization

**Behavior**:

- Fetches template files from GitHub
- Copies files to local repository
- Performs variable substitution
- Creates additional files if not in template

**Content Provider**: `TemplateBasedContentProvider`

#### Empty

**Purpose**: Create repository with no files.

**Requirements**:

- None (works with or without template)

**Behavior**:

- Creates empty temporary directory
- No files generated
- Repository will have no initial commit content

**Content Provider**: `ZeroContentProvider`

**Use Cases**:

- Migration scenarios (content added later)
- Blank slate repositories
- Using template for settings only

#### CustomInit

**Purpose**: Create repository with selected initialization files.

**Requirements**:

- At least one of `include_readme` or `include_gitignore` should be true (not enforced)

**Behavior**:

- Creates temporary directory
- Generates README.md if `include_readme` is true
- Generates .gitignore if `include_gitignore` is true

**Content Provider**: `CustomInitContentProvider`

**Use Cases**:

- Quick repository setup with minimal files
- Custom workflows requiring specific initialization

## RepositoryCreationRequest Changes

### Template Field Optionality

**Change**: `template` field changes from `TemplateName` to `Option<TemplateName>`

**Before**:

```rust
pub struct RepositoryCreationRequest {
    pub name: RepositoryName,
    pub owner: OrganizationName,
    pub template: TemplateName,  // Required
    pub visibility: Option<RepositoryVisibility>,
    pub variables: HashMap<String, String>,
}
```

**After**:

```rust
pub struct RepositoryCreationRequest {
    pub name: RepositoryName,
    pub owner: OrganizationName,
    pub template: Option<TemplateName>,  // Optional
    pub visibility: Option<RepositoryVisibility>,
    pub variables: HashMap<String, String>,
    pub content_strategy: ContentStrategy,
}
```

### New Field: content_strategy

**Type**: `ContentStrategy`
**Default**: `ContentStrategy::Template`
**Purpose**: Determines content generation strategy

### Updated Type Definition

```rust
/// Request for creating a new repository.
///
/// Contains all information needed to create a repository including:
/// - Repository identification (name, owner)
/// - Optional template for content and settings
/// - Content generation strategy
/// - Visibility preference
/// - Template variables for substitution
///
/// Use [`RepositoryCreationRequestBuilder`] to construct instances.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryCreationRequest {
    /// Repository name (validated, unique within organization)
    pub name: RepositoryName,

    /// Organization/owner name (validated)
    pub owner: OrganizationName,

    /// Optional template name for content and settings.
    ///
    /// When `Some`, template is loaded for:
    /// - File content (if ContentStrategy::Template)
    /// - Repository settings
    /// - Default values for variables
    ///
    /// When `None`:
    /// - Organization defaults used for settings
    /// - Only Empty or CustomInit strategies valid
    pub template: Option<TemplateName>,

    /// Visibility preference for the repository.
    ///
    /// If provided, this is the user's visibility preference.
    /// Final visibility is determined by the VisibilityResolver based on:
    /// - Organization policy (highest priority)
    /// - User preference (this field)
    /// - Template default
    /// - System default (Private)
    pub visibility: Option<RepositoryVisibility>,

    /// Template variables for substitution.
    ///
    /// Variables are used during template processing to customize content.
    /// Variable values are validated against template variable configurations.
    pub variables: HashMap<String, String>,

    /// Content generation strategy.
    ///
    /// Determines how repository content is generated:
    /// - Template: Fetch and process template files (default)
    /// - Empty: Create no files
    /// - CustomInit: Create selected initialization files
    ///
    /// See [`ContentStrategy`] for details.
    pub content_strategy: ContentStrategy,
}
```

## RepositoryCreationRequestBuilder Changes

### Updated Builder

```rust
pub struct RepositoryCreationRequestBuilder {
    name: Option<RepositoryName>,
    owner: Option<OrganizationName>,
    template: Option<TemplateName>,  // Now truly optional
    visibility: Option<RepositoryVisibility>,
    variables: Option<HashMap<String, String>>,
    content_strategy: Option<ContentStrategy>,
}
```

### Constructor Change

**Before**:

```rust
pub fn new(
    name: RepositoryName,
    owner: OrganizationName,
    template: TemplateName,  // Required
) -> Self
```

**After**:

```rust
pub fn new(
    name: RepositoryName,
    owner: OrganizationName,
) -> Self
```

**Rationale**: Template is now optional, so it's not required in constructor.

### New Builder Method: template()

```rust
/// Set the template for content and settings.
///
/// When provided, the template is loaded for file content (if using Template strategy)
/// and repository settings.
///
/// # Examples
///
/// ```
/// # use repo_roller_core::*;
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let request = RepositoryCreationRequestBuilder::new(
///     RepositoryName::new("my-repo")?,
///     OrganizationName::new("my-org")?,
/// )
/// .template(TemplateName::new("rust-service")?)
/// .build();
/// # Ok(())
/// # }
/// ```
pub fn template(mut self, template: TemplateName) -> Self {
    self.template = Some(template);
    self
}
```

### New Builder Method: content_strategy()

```rust
/// Set the content generation strategy.
///
/// Determines how repository content is generated. Defaults to Template strategy.
///
/// # Examples
///
/// ```
/// # use repo_roller_core::*;
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Empty repository
/// let request = RepositoryCreationRequestBuilder::new(
///     RepositoryName::new("my-repo")?,
///     OrganizationName::new("my-org")?,
/// )
/// .template(TemplateName::new("github-actions")?)
/// .content_strategy(ContentStrategy::Empty)
/// .build();
///
/// // Custom initialization
/// let request = RepositoryCreationRequestBuilder::new(
///     RepositoryName::new("my-repo")?,
///     OrganizationName::new("my-org")?,
/// )
/// .content_strategy(ContentStrategy::CustomInit {
///     include_readme: true,
///     include_gitignore: true,
/// })
/// .build();
/// # Ok(())
/// # }
/// ```
pub fn content_strategy(mut self, strategy: ContentStrategy) -> Self {
    self.content_strategy = Some(strategy);
    self
}
```

### Updated build() Method

```rust
pub fn build(self) -> RepositoryCreationRequest {
    let name = self.name.expect("name is required");
    let owner = self.owner.expect("owner is required");
    let content_strategy = self.content_strategy.unwrap_or_default();

    // Validation: Template strategy requires template
    if matches!(content_strategy, ContentStrategy::Template) && self.template.is_none() {
        panic!("ContentStrategy::Template requires template to be set. Use .template() method.");
    }

    RepositoryCreationRequest {
        name,
        owner,
        template: self.template,
        visibility: self.visibility,
        variables: self.variables.unwrap_or_default(),
        content_strategy,
    }
}
```

## Valid Combinations Matrix

| template | ContentStrategy | Valid | Behavior |
|----------|-----------------|-------|----------|
| `Some("template")` | `Template` | ✅ | Fetch template, process files (current behavior) |
| `Some("template")` | `Empty` | ✅ | Use template settings, no files |
| `Some("template")` | `CustomInit` | ✅ | Use template settings, custom files only |
| `None` | `Template` | ❌ | **INVALID** - Validation error at build() |
| `None` | `Empty` | ✅ | Use org defaults, no files |
| `None` | `CustomInit` | ✅ | Use org defaults, custom files only |

## Validation Rules

### Build-Time Validation

Enforced in `RepositoryCreationRequestBuilder::build()`:

```rust
// Rule 1: Template strategy requires template name
if matches!(content_strategy, ContentStrategy::Template) && self.template.is_none() {
    panic!("ContentStrategy::Template requires template to be set");
}
```

### Runtime Validation

Enforced in `create_repository()` function:

```rust
// Validate template exists when provided
if let Some(ref template_name) = request.template {
    // Load template configuration (will error if not found)
    let template = metadata_provider
        .load_template_configuration(request.owner.as_ref(), template_name.as_ref())
        .await?;
}

// Validate strategy matches template presence
match &request.content_strategy {
    ContentStrategy::Template => {
        request.template.as_ref().ok_or_else(|| {
            RepoRollerError::Validation(ValidationError::InvalidFieldValue {
                field: "content_strategy".to_string(),
                value: "Template".to_string(),
                reason: "ContentStrategy::Template requires template name".to_string(),
            })
        })?;
    }
    _ => { /* Empty and CustomInit work with or without template */ }
}
```

## Configuration Resolution Without Template

When `template` is `None`, configuration resolution follows this fallback hierarchy:

1. **Organization Defaults**: Loaded from `.reporoller-test/global/defaults.toml`
2. **Repository Type Defaults**: Not applicable (no repository type without template)
3. **System Defaults**: Built-in defaults from `config_manager`

### Required Org Defaults

The organization defaults must provide sensible values for:

**Repository Features**:

- `issues`: Default true
- `projects`: Default false
- `discussions`: Default false
- `wiki`: Default false
- `pages`: Default false
- `security_advisories`: Default true
- `vulnerability_reporting`: Default true

**Pull Request Settings**:

- `required_approving_review_count`: Default 1
- `allow_merge_commit`: Default true
- `allow_squash_merge`: Default true
- `allow_rebase_merge`: Default false
- `allow_auto_merge`: Default false
- `delete_branch_on_merge`: Default true

**Branch Protection**:

- `require_conversation_resolution`: Default true
- `require_pull_request_reviews`: Default true

### Implementation Note

The `config_manager` crate should ensure organization defaults are comprehensive enough to create repositories without templates.

## Migration Guide

### For Existing Code

**Before** (required template):

```rust
let request = RepositoryCreationRequestBuilder::new(
    RepositoryName::new("my-repo")?,
    OrganizationName::new("my-org")?,
    TemplateName::new("rust-service")?,  // Required parameter
)
.build();
```

**After** (optional template):

```rust
let request = RepositoryCreationRequestBuilder::new(
    RepositoryName::new("my-repo")?,
    OrganizationName::new("my-org")?,
)
.template(TemplateName::new("rust-service")?)  // Optional method call
.build();
```

### For CLI Interface

**Current**:

```bash
repo-roller create my-repo --org my-org --template rust-service
```

**New Options**:

```bash
# Template-based (default)
repo-roller create my-repo --org my-org --template rust-service

# Empty repository with template settings
repo-roller create my-repo --org my-org --template rust-service --empty

# Empty repository with org defaults
repo-roller create my-repo --org my-org --empty

# Custom initialization
repo-roller create my-repo --org my-org --init-readme --init-gitignore
repo-roller create my-repo --org my-org --template rust-service --init-readme
```

### For API Interface

**Request Body**:

```json
{
  "name": "my-repo",
  "owner": "my-org",
  "template": "rust-service",
  "content_strategy": {
    "type": "custom_init",
    "include_readme": true,
    "include_gitignore": true
  }
}
```

## Backward Compatibility

### Breaking Changes

1. **Constructor signature**: `RepositoryCreationRequestBuilder::new()` no longer requires `template` parameter
2. **Field type**: `RepositoryCreationRequest.template` is now `Option<TemplateName>`

### Mitigation

These are **compile-time breaking changes** that require code updates:

```rust
// Old code (will not compile)
let builder = RepositoryCreationRequestBuilder::new(name, owner, template);

// New code
let builder = RepositoryCreationRequestBuilder::new(name, owner)
    .template(template);
```

### Compatibility Strategy

- Update all call sites in the same PR as interface changes
- Provide migration guide in PR description
- Update integration tests to use new API
- Update CLI to support new options

## Examples

### Template-Based Repository (Current Behavior)

```rust
let request = RepositoryCreationRequestBuilder::new(
    RepositoryName::new("my-service")?,
    OrganizationName::new("my-org")?,
)
.template(TemplateName::new("rust-service")?)
.content_strategy(ContentStrategy::Template)  // Default, can be omitted
.variable("author", "Jane Doe")
.build();
```

### Empty Repository with Template Settings

```rust
let request = RepositoryCreationRequestBuilder::new(
    RepositoryName::new("my-actions-repo")?,
    OrganizationName::new("my-org")?,
)
.template(TemplateName::new("github-actions")?)
.content_strategy(ContentStrategy::Empty)
.build();

// Uses template settings (e.g., issues disabled, wiki disabled)
// but creates no files
```

### Empty Repository with Org Defaults

```rust
let request = RepositoryCreationRequestBuilder::new(
    RepositoryName::new("blank-slate")?,
    OrganizationName::new("my-org")?,
)
.content_strategy(ContentStrategy::Empty)
.build();

// No template specified, uses org defaults for all settings
```

### Custom Initialization

```rust
let request = RepositoryCreationRequestBuilder::new(
    RepositoryName::new("quick-start")?,
    OrganizationName::new("my-org")?,
)
.content_strategy(ContentStrategy::CustomInit {
    include_readme: true,
    include_gitignore: true,
})
.build();

// Creates README.md and .gitignore, uses org defaults for settings
```

### Custom Init with Template Settings

```rust
let request = RepositoryCreationRequestBuilder::new(
    RepositoryName::new("quick-start")?,
    OrganizationName::new("my-org")?,
)
.template(TemplateName::new("rust-library")?)
.content_strategy(ContentStrategy::CustomInit {
    include_readme: true,
    include_gitignore: false,
})
.build();

// Creates README.md only, uses rust-library template for settings
```

## Testing Requirements

### Unit Tests

Test `ContentStrategy`:

- Serialization/deserialization for all variants
- Default value is Template
- Clone, Debug, PartialEq implementations

Test `RepositoryCreationRequestBuilder`:

- build() succeeds with valid combinations
- build() panics with Template strategy + no template
- template() method sets value correctly
- content_strategy() method sets value correctly

### Integration Tests

- Create repository with Empty strategy + template
- Create repository with Empty strategy + no template
- Create repository with CustomInit + both files
- Create repository with CustomInit + README only
- Create repository with CustomInit + gitignore only
- Verify settings are applied correctly in all cases

## Related Specifications

- [Content Providers](content-providers.md) - ContentProvider trait and implementations
- [Request Types](request-types.md) - RepositoryCreationRequest details
- [Configuration System](../design/organization-repository-settings.md) - Configuration resolution

## Change History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-01-10 | Initial specification with ContentStrategy enum |
