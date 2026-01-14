# Content Providers Interface Specification

**Status**: Interface Design Complete
**Version**: 1.0
**Last Updated**: 2026-01-10

## Overview

This specification defines the `ContentProvider` trait and its implementations for generating repository content during creation. Content providers follow the Strategy pattern, allowing the repository creation workflow to support multiple content generation strategies without modification.

## Architecture Context

**Layer**: Core Domain
**Module Path**: `crates/repo_roller_core/src/content_providers.rs`
**Dependencies**:

- `RepositoryCreationRequest` (request.rs)
- `TemplateConfig` (config_manager crate)
- `MergedConfiguration` (config_manager crate)
- `TemplateFetcher` (template_engine crate)
- `TempDir` (temp_dir crate)

**Architectural Pattern**: Strategy Pattern

```
create_repository()
    ↓
ContentProvider::provide_content()
    ↓
Returns TempDir with content
    ↓
Git init & commit
    ↓
Push to GitHub
```

## Use Cases

### Use Case 1: Template-Based Repository Creation (Current Behavior)

- **Actor**: Developer creating repository from template
- **Goal**: Create repository with files from template repository
- **Content**: Template files with variable substitution
- **Provider**: `TemplateBasedContentProvider`

### Use Case 2: Empty Repository Creation

- **Actor**: Developer needing blank slate repository
- **Goal**: Create repository with no files, using template settings
- **Content**: No files (empty directory)
- **Provider**: `ZeroContentProvider`
- **Example**: Creating repository for later migration, using GitHub Actions template settings

### Use Case 3: Custom Initialization

- **Actor**: Developer needing minimal initialized repository
- **Goal**: Create repository with selected initialization files
- **Content**: README.md and/or .gitignore
- **Provider**: `CustomInitContentProvider`

## ContentProvider Trait

### Trait Definition

```rust
#[async_trait::async_trait]
pub trait ContentProvider: Send + Sync {
    async fn provide_content(
        &self,
        request: &RepositoryCreationRequest,
        template_config: Option<&config_manager::TemplateConfig>,
        template_source: &str,
        merged_config: &config_manager::MergedConfiguration,
    ) -> RepoRollerResult<TempDir>;
}
```

### Method: `provide_content`

#### Purpose

Creates and populates a temporary directory with repository content according to the provider's strategy.

#### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `request` | `&RepositoryCreationRequest` | Repository creation request with name, owner, variables, content strategy |
| `template_config` | `Option<&TemplateConfig>` | Template configuration (Some if template provided, None otherwise) |
| `template_source` | `&str` | Template source identifier (e.g., "org/repo", empty string if no template) |
| `merged_config` | `&MergedConfiguration` | Merged organization configuration providing variables and settings |

#### Returns

| Case | Return Value | Description |
|------|--------------|-------------|
| Success | `Ok(TempDir)` | Temporary directory with repository content |
| Failure | `Err(RepoRollerError)` | Content generation failed |

#### Error Conditions

| Error Type | Variant | Condition |
|------------|---------|-----------|
| `ValidationError` | `MissingRequiredField` | Required parameter missing for strategy |
| `ValidationError` | `InvalidFieldValue` | Invalid parameter value for strategy |
| `TemplateError` | `FetchFailed` | Template file fetching failed |
| `TemplateError` | `SubstitutionFailed` | Variable substitution failed |
| `SystemError` | `Internal` | Temporary directory creation failed |
| `SystemError` | `FileSystem` | File operations failed |

#### Behavior Specification

- **Temporary Directory**: Returned `TempDir` is automatically cleaned up when dropped
- **Content Readiness**: Content is ready for Git initialization (no `.git` directory)
- **File State**: Files are in final form (variables substituted, content processed)
- **Empty Directory**: Valid to return empty `TempDir` (for `ZeroContentProvider`)

#### Performance Expectations

- **Template fetching**: Network-dependent (GitHub API)
- **Variable substitution**: Proportional to file count and size
- **Custom init files**: Fast (<100ms)

## TemplateBasedContentProvider

### Purpose

Fetches and processes template files from a source repository. This is the current/default behavior.

### Behavior

1. **Fetch**: Retrieves template files from source repository via `TemplateFetcher`
2. **Copy**: Copies template files to temporary directory
3. **Substitute**: Performs variable substitution in all files
4. **Augment**: Creates additional files (README.md, .gitignore) if not provided by template

### Constructor

```rust
impl<'a> TemplateBasedContentProvider<'a> {
    pub fn new(fetcher: &'a dyn TemplateFetcher) -> Self
}
```

#### Parameters

- `fetcher`: Template fetcher implementation for retrieving files from GitHub

### Requirements

| Requirement | Validation | Error |
|-------------|------------|-------|
| `template_config` must be `Some` | Check at method start | `ValidationError::MissingRequiredField` |
| `template_source` must not be empty | Check length > 0 | `ValidationError::InvalidFieldValue` |
| Template must exist in source | Fetcher validates | `TemplateError::FetchFailed` |

### Template File Processing

#### Variable Substitution

Uses `template_engine::TemplateProcessor` for variable substitution:

- **Built-in variables**: repo_name, org_name, template_name, user_login, user_name, default_branch
- **Config variables**: Extracted from `merged_config` with `config_` prefix
- **User variables**: Provided in `request.variables`

#### Additional Files

Creates if not present in template:

- **README.md**: Basic readme with repository metadata
- **.gitignore**: Common ignore patterns

### Example Usage

```rust
use template_engine::GitHubTemplateFetcher;

let fetcher = GitHubTemplateFetcher::new(github_client);
let provider = TemplateBasedContentProvider::new(&fetcher);

let temp_dir = provider.provide_content(
    &request,
    Some(&template_config),
    "my-org/template-rust-library",
    &merged_config,
).await?;
```

### Integration Notes

- Reuses existing `template_processing::prepare_local_repository()` function
- Maintains backward compatibility with current behavior
- No changes to template processing logic

## ZeroContentProvider

### Purpose

Creates an empty temporary directory with no files. Useful for:

- Migration scenarios where content added separately
- Blank slate repositories for experimentation
- Using template configuration for settings only (no files)

### Behavior

1. **Create**: Creates empty temporary directory
2. **Return**: Returns empty `TempDir`

### Constructor

```rust
impl ZeroContentProvider {
    pub fn new() -> Self
}

impl Default for ZeroContentProvider {
    fn default() -> Self
}
```

### Requirements

| Requirement | Validation | Error |
|-------------|------------|-------|
| None | No validation needed | N/A |

### Parameters Usage

- `template_config`: Ignored (may be Some or None)
- `template_source`: Ignored (typically empty string)
- `merged_config`: Not used for content, but settings still applied to GitHub repository
- `request`: Only used for logging context

### Example Usage

```rust
let provider = ZeroContentProvider::new();

let temp_dir = provider.provide_content(
    &request,
    Some(&template_config),  // Settings used, but no files copied
    "",
    &merged_config,
).await?;

// temp_dir is empty, no files created
```

### Notes

- Even though directory is empty, repository will still have settings applied via `merged_config`
- Template configuration (if provided) is used for repository settings, not content
- When Git initialized, repository will have initial branch but no files

## CustomInitContentProvider

### Purpose

Creates custom initialization files based on user preferences. Generates minimal repository setup with README.md and/or .gitignore.

### Behavior

1. **Create**: Creates empty temporary directory
2. **Generate**: Generates requested initialization files
3. **Return**: Returns `TempDir` with generated files

### Configuration

```rust
pub struct CustomInitOptions {
    pub include_readme: bool,
    pub include_gitignore: bool,
}
```

### Constructor

```rust
impl CustomInitContentProvider {
    pub fn new(options: CustomInitOptions) -> Self
}
```

### Requirements

| Requirement | Validation | Error |
|-------------|------------|-------|
| At least one option true | Recommended but not enforced | N/A (valid to have both false) |

### File Generation

#### README.md Content

**With Template**:

```markdown
# {repo_name}

Repository created using RepoRoller with template '{template_name}'.

**Organization:** {org_name}
**Created:** {current_date}
```

**Without Template**:

```markdown
# {repo_name}

Repository created using RepoRoller.

**Organization:** {org_name}
**Created:** {current_date}
```

#### .gitignore Content

Standard ignore patterns:

```gitignore
# Operating System Files
.DS_Store
.DS_Store?
._*
.Spotlight-V100
.Trashes
ehthumbs.db
Thumbs.db

# Editor and IDE
.vscode/
.idea/
*.swp
*.swo
*~

# Logs and Temporary Files
*.log
*.tmp
*.temp
*.bak

# Build Outputs
target/
dist/
build/
out/
*.o
*.so
*.exe

# Dependencies
node_modules/
vendor/
```

### Example Usage

```rust
let options = CustomInitOptions {
    include_readme: true,
    include_gitignore: true,
};
let provider = CustomInitContentProvider::new(options);

let temp_dir = provider.provide_content(
    &request,
    Some(&template_config),
    "",
    &merged_config,
).await?;

// temp_dir contains README.md and .gitignore
```

### Variable Substitution

Simple substitution for README.md:

- `{repo_name}` → `request.name.as_ref()`
- `{org_name}` → `request.owner.as_ref()`
- `{template_name}` → `request.template.as_ref()` (if present)
- `{current_date}` → `chrono::Utc::now().format("%Y-%m-%d")`

No variable substitution in .gitignore (static content).

## Integration with create_repository()

### Workflow Integration

The `create_repository()` function integrates content providers at Step 5:

```rust
// Step 5: Prepare local repository content based on strategy
let local_repo_path = match &request.content_strategy {
    ContentStrategy::Template => {
        // Validate template is provided
        let template = request.template.as_ref().ok_or_else(|| {
            RepoRollerError::Validation(ValidationError::InvalidFieldValue {
                field: "content_strategy".to_string(),
                value: "Template".to_string(),
                reason: "ContentStrategy::Template requires template name".to_string(),
            })
        })?;

        let provider = TemplateBasedContentProvider::new(&template_fetcher);
        provider.provide_content(
            &request,
            Some(&template_config),
            &template_source,
            &merged_config,
        ).await?
    }

    ContentStrategy::Empty => {
        let provider = ZeroContentProvider::new();
        provider.provide_content(
            &request,
            template_config_option, // May be Some or None
            "",
            &merged_config,
        ).await?
    }

    ContentStrategy::CustomInit { include_readme, include_gitignore } => {
        let options = CustomInitOptions {
            include_readme: *include_readme,
            include_gitignore: *include_gitignore,
        };
        let provider = CustomInitContentProvider::new(options);
        provider.provide_content(
            &request,
            template_config_option, // May be Some or None
            "",
            &merged_config,
        ).await?
    }
};

// Steps 6-10 proceed unchanged with local_repo_path
```

### Configuration Resolution

When template is not provided (`request.template` is `None`):

- Configuration resolution falls back to organization defaults
- `merged_config` still provides settings for GitHub repository
- Content providers operate normally (empty or custom init files)

### Backward Compatibility

- Default `ContentStrategy` is `Template` (existing behavior)
- Existing code continues to work without changes
- New strategies are opt-in via request builder

## Error Handling

### Error Mapping

| Provider Error | RepoRollerError | Context |
|----------------|-----------------|---------|
| TempDir creation failed | `SystemError::Internal` | "Failed to create temporary directory" |
| Template fetch failed | `TemplateError::FetchFailed` | "Failed to fetch template files" |
| File copy failed | `SystemError::FileSystem` | "Failed to copy template files" |
| Variable substitution failed | `TemplateError::SubstitutionFailed` | "Batch variable replacement failed" |
| README creation failed | `SystemError::FileSystem` | "Failed to create README.md" |
| .gitignore creation failed | `SystemError::FileSystem` | "Failed to create .gitignore" |

### Validation Errors

```rust
// TemplateBasedContentProvider validation
if template_config.is_none() {
    return Err(RepoRollerError::Validation(ValidationError::MissingRequiredField {
        field: "template_config".to_string(),
        context: "TemplateBasedContentProvider requires template configuration".to_string(),
    }));
}

if template_source.is_empty() {
    return Err(RepoRollerError::Validation(ValidationError::InvalidFieldValue {
        field: "template_source".to_string(),
        value: "(empty)".to_string(),
        reason: "Template source must be provided for template-based creation".to_string(),
    }));
}
```

## Testing Requirements

### Unit Tests

Each provider must have unit tests covering:

#### TemplateBasedContentProvider

- Successful content generation with valid template
- Template fetch failure handling
- Variable substitution
- Additional file creation
- Empty template_source rejection
- None template_config rejection

#### ZeroContentProvider

- Empty directory creation
- Works with and without template_config
- TempDir cleanup on drop

#### CustomInitContentProvider

- README.md generation (with/without template)
- .gitignore generation
- Both files together
- Neither file (empty dir)
- Variable substitution in README

### Integration Tests

Test against real GitHub infrastructure:

- Template-based repository creation (existing test)
- Empty repository creation with template settings
- Empty repository creation without template
- Custom init repository creation (README only)
- Custom init repository creation (gitignore only)
- Custom init repository creation (both files)

### Test Organization

```
crates/repo_roller_core/src/
├── content_providers.rs
├── content_providers_tests.rs      # Unit tests
└── ...

crates/integration_tests/tests/
├── content_provider_integration_tests.rs  # Integration tests
└── ...
```

## Security Considerations

### Path Traversal Prevention

- `TemplateBasedContentProvider` uses existing `validate_safe_path()` function
- Custom file creation uses `TempDir.path().join()` (safe)
- No user-controlled paths in file creation

### Content Injection

- README.md template uses safe string formatting (no eval)
- .gitignore is static content (no injection risk)
- Template substitution uses `template_engine` crate (handles escaping)

## Performance Characteristics

| Provider | Time Complexity | Notes |
|----------|-----------------|-------|
| `TemplateBasedContentProvider` | O(n) where n = file count | Network-bound (GitHub API) |
| `ZeroContentProvider` | O(1) | Fast (directory creation only) |
| `CustomInitContentProvider` | O(1) | Fast (2 file writes max) |

## Future Enhancements

### Potential Extensions

1. **Language-specific .gitignore**: Support templates for Rust, Node, Python, etc.
2. **LICENSE file generation**: Add license selection with copyright info
3. **CI/CD file generation**: Create basic workflow files
4. **Custom file templates**: User-provided file templates
5. **Template composition**: Combine multiple templates

### Extensibility Points

The trait design allows easy addition of new providers:

```rust
pub struct AdvancedContentProvider { /* ... */ }

#[async_trait::async_trait]
impl ContentProvider for AdvancedContentProvider {
    async fn provide_content(/* ... */) -> RepoRollerResult<TempDir> {
        // New implementation
    }
}
```

## Related Specifications

- [Repository Creation Modes](repository-creation-modes.md) - ContentStrategy enum
- [Template Processing](template-processing.md) - Template file processing
- [Request Types](request-types.md) - RepositoryCreationRequest
- [Error Types](error-types.md) - Error handling

## Change History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-01-10 | Initial specification with three providers |
