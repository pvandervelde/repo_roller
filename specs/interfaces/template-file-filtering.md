# Template File Filtering Interface

**Status**: Interface Design Complete
**Related Task**: Task 7.0 - Template File Exclusion
**Layer**: Configuration Management / Template Processing

---

## Overview

This interface defines how template authors control which files from template repositories are included or excluded during repository creation. File filtering is configured entirely at the template level, giving template authors full control over what gets processed.

**Key Principles**:

- **Single configuration location** - All filtering in `.reporoller/template.toml` only
- **Template-level control** - No default exclusions except `.reporoller/` directory
- **Explicit over implicit** - Clear, predictable filtering behavior
- **Simple glob patterns** - Standard glob syntax for includes and excludes

---

## Architectural Context

### Responsibilities (RDD)

**Configuration Layer** (`config_manager`):

- **Knows**: Template filtering configuration structure
- **Does**: Loads and validates filtering patterns from template repositories

**Template Engine** (`template_engine`):

- **Knows**: Glob pattern matching, file filtering logic
- **Does**: Applies filtering patterns during template processing

**Orchestrator** (`repo_roller_core`):

- **Knows**: How to wire configuration to template engine
- **Does**: Passes filtering config from templates to processing requests

### Dependencies

```
TemplateConfig (config_manager)
    ↓ contains
TemplatingConfig (template_engine)
    ↓ used by
TemplateProcessingRequest (template_engine)
    ↓ processed by
TemplateProcessor (template_engine)
```

---

## Types and Interfaces

### Core Type: `TemplatingConfig`

**Location**: `crates/template_engine/src/lib.rs` (already exists)

```rust
/// Configuration for controlling which files are processed during template rendering.
///
/// This structure allows fine-grained control over which files in a template
/// repository should be processed for variable substitution. Files can be included
/// or excluded based on glob patterns.
///
/// ## Pattern Matching
///
/// - Patterns follow standard glob syntax (`*`, `?`, `**`, etc.)
/// - Patterns are applied relative to the template repository root
/// - Exclude patterns take precedence over include patterns
/// - If no include patterns are specified, all files are included by default
///
/// ## Examples
///
/// ```rust
/// use template_engine::TemplatingConfig;
///
/// // Process all Rust and markdown files, but skip test files
/// let config = TemplatingConfig {
///     include_patterns: vec![
///         "**/*.rs".to_string(),
///         "**/*.md".to_string(),
///         "Cargo.toml".to_string(),
///     ],
///     exclude_patterns: vec![
///         "**/*_test.rs".to_string(),
///         "**/*_tests.rs".to_string(),
///         "target/**".to_string(),
///     ],
/// };
///
/// // Process everything except binary files and build artifacts
/// let liberal_config = TemplatingConfig {
///     include_patterns: vec!["**/*".to_string()],
///     exclude_patterns: vec![
///         "**/*.exe".to_string(),
///         "**/*.dll".to_string(),
///         "**/*.so".to_string(),
///         "**/target/**".to_string(),
///         "**/node_modules/**".to_string(),
///     ],
/// };
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TemplatingConfig {
    /// Glob patterns for files that should be processed.
    ///
    /// If empty, all files are included by default (unless excluded).
    pub include_patterns: Vec<String>,

    /// Glob patterns for files that should be skipped.
    ///
    /// Exclude patterns take precedence over include patterns.
    pub exclude_patterns: Vec<String>,
}
```

### Extended Type: `TemplateConfig`

**Location**: `crates/config_manager/src/template_config.rs` (needs update)

```rust
/// Template-specific configuration embedded in template repositories.
///
/// Defines the configuration requirements and defaults for repositories
/// created from this template. Templates have the highest precedence in
/// the configuration hierarchy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateConfig {
    /// Template metadata (required).
    pub template: TemplateMetadata,

    /// Repository type specification (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_type: Option<RepositoryTypeSpec>,

    /// Template variables (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<String, TemplateVariable>>,

    /// Repository feature settings (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<RepositorySettings>,

    /// Pull request configuration (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pull_requests: Option<PullRequestSettings>,

    /// Branch protection settings (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_protection: Option<BranchProtectionSettings>,

    /// Template-specific labels (additive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<LabelConfig>>,

    /// Template-specific webhooks (additive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhooks: Option<Vec<WebhookConfig>>,

    /// Template-specific environments (additive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environments: Option<Vec<EnvironmentConfig>>,

    /// Template-specific GitHub Apps (additive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_apps: Option<Vec<GitHubAppConfig>>,

    /// Default visibility for repositories created from this template (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_visibility: Option<RepositoryVisibility>,

    /// File filtering configuration for template processing (optional).
    ///
    /// Controls which files from the template repository are processed and
    /// included in the created repository. If not specified, all files are
    /// processed (except the `.reporoller/` directory which is always excluded).
    ///
    /// ## TOML Format
    ///
    /// ```toml
    /// [templating]
    /// include_patterns = ["**/*.rs", "**/*.toml", "**/*.md"]
    /// exclude_patterns = ["target/**", "*.log", "tmp/**"]
    /// ```
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use config_manager::TemplateConfig;
    /// use template_engine::TemplatingConfig;
    ///
    /// let toml = r#"
    ///     [template]
    ///     name = "rust-service"
    ///     description = "Rust microservice template"
    ///     author = "Platform Team"
    ///     tags = ["rust", "service"]
    ///
    ///     [templating]
    ///     include_patterns = ["**/*.rs", "Cargo.toml", "README.md"]
    ///     exclude_patterns = ["target/**", "*.bak"]
    /// "#;
    ///
    /// let config: TemplateConfig = toml::from_str(toml).expect("Parse failed");
    /// assert!(config.templating.is_some());
    /// ```
    #[serde(skip_serializing_if = "Option::is_none")]
    pub templating: Option<TemplatingConfig>,
}
```

## Configuration Format

### TOML Configuration (`.reporoller/template.toml`)

**Single Configuration Location**: All filtering configured in template TOML.

```toml
# .reporoller/template.toml

[template]
name = "rust-microservice"
description = "Production-ready Rust microservice template"
author = "Platform Team"
tags = ["rust", "service", "backend"]

# File filtering configuration
[templating]
include_patterns = [
    "**/*.rs",           # All Rust source files
    "**/*.toml",         # All TOML configuration files
    "**/*.md",           # All markdown documentation
    "LICENSE",           # License file
    ".gitignore",        # Git ignore rules
]

exclude_patterns = [
    "target/**",         # Rust build output
    "**/*.log",          # Log files
    "**/*.tmp",          # Temporary files
    "**/tmp/**",         # Temporary directories
    "scripts/dev/**",    # Development-only scripts
]
```

**Note**: For long exclusion lists, consider organizing patterns with comments for maintainability:

```toml
[templating]
# Source code only
include_patterns = ["src/**/*.rs", "Cargo.toml"]

# Exclude build artifacts
exclude_patterns = [
    # Build outputs
    "target/**",
    "dist/**",
    "build/**",

    # Logs and temp files
    "**/*.log",
    "**/*.tmp",
    "tmp/**",

    # OS files
    "**/.DS_Store",
    "**/Thumbs.db",
]
```

---

## Pattern Syntax

### Glob Patterns

File filtering uses standard glob syntax with the following features:

| Pattern | Meaning | Example Matches |
|---------|---------|-----------------|
| `*` | Match any characters except `/` | `*.rs` matches `main.rs`, `lib.rs` |
| `**` | Match any characters including `/` | `**/*.rs` matches `src/main.rs`, `tests/unit/test.rs` |
| `?` | Match single character | `test?.rs` matches `test1.rs`, `testa.rs` |
| `[abc]` | Match one character from set | `file[123].txt` matches `file1.txt`, `file2.txt` |
| `[a-z]` | Match character in range | `[a-z]*.rs` matches `main.rs`, `lib.rs` |


### Pattern Resolution Rules

1. **All patterns are relative to template repository root**
   - `src/**/*.rs` matches files in `src/` directory
   - `*.md` matches markdown files in root only
   - `**/*.md` matches markdown files anywhere

2. **Exclude patterns take precedence over include patterns**

   ```toml
   include_patterns = ["**/*.rs"]
   exclude_patterns = ["target/**"]
   # Result: All .rs files EXCEPT those in target/
   ```

3. **Empty include_patterns means "include all"**

   ```toml
   include_patterns = []           # Include everything
   exclude_patterns = ["target/**"] # Except target/
   ```

### Path Separators

- Patterns use forward slashes (`/`) regardless of OS
- Template engine normalizes paths for cross-platform compatibility
- Both `target/**` and `target\\**` work (normalized internally)

---

## Behavioral Specifications

### Filtering Algorithm

```rust
fn should_process_file(
    file_path: &str,
    config: &Option<TemplatingConfig>,
) -> bool {
    match config {
        None => true,  // No config = process all files
        Some(cfg) => {
            // Check exclude patterns first (highest precedence)
            if matches_any_pattern(file_path, &cfg.exclude_patterns) {
                return false;
            }

            // If include patterns specified, file must match at least one
            if !cfg.include_patterns.is_empty() {
                return matches_any_pattern(file_path, &cfg.include_patterns);
            }

            // No include patterns = include all (that aren't excluded)
            true
        }
    }
}
```

### Special File Handling

**Always Excluded** (hardcoded in `GitHubTemplateFetcher`):

- `.git/**` - Git repository metadata
- `.reporoller/**` - Template configuration (not copied to target repository)

**Special Processing**:

- `.gitignore` - Filtered out during fetch, regenerated in target repository
- `.github/**` - **Allowed** - Templates can include GitHub Actions workflows for target repositories

### Error Handling

**Invalid Patterns**:

```rust
// Invalid glob patterns are logged but don't fail processing
if let Err(e) = compile_glob_pattern(pattern) {
    warn!("Invalid pattern '{}' ignored: {}", pattern, e);
    continue;
}
```

**Conflicting Patterns**:

```toml
# This is valid - exclude takes precedence
include_patterns = ["src/**/*.rs"]
exclude_patterns = ["src/generated/**"]
# Result: src/**/*.rs EXCEPT src/generated/**
```

**Empty Results**:

```rust
// If filtering results in zero files, return error
if filtered_files.is_empty() {
    return Err(TemplateError::NoFilesAfterFiltering {
        template_name: template_name.to_string(),
        patterns: config.clone(),
    });
}
```

---

## Integration Points

### 1. Configuration Loading

**File**: `crates/config_manager/src/github_template_repository.rs`

```rust
impl TemplateRepository for GitHubTemplateRepository {
    async fn load_template_config(
        &self,
        org: &str,
        template_name: &str,
    ) -> Result<TemplateConfig, ConfigurationError> {
        // Load .reporoller/template.toml (includes [templating] section)
        let config = self.load_toml_config(org, template_name).await?;

        // templating field is optional and already parsed from TOML
        Ok(config)
    }
}
```

### 2. Processing Request Creation

**File**: `crates/repo_roller_core/src/template_processing.rs`

```rust
pub(crate) async fn process_template_content(
    template: &TemplateConfig,
    merged_config: &MergedConfiguration,
    req: &RepositoryCreationRequest,
    local_repo_path: &TempDir,
) -> Result<(), SystemError> {
    // ... existing code ...

    // Pass templating config from template to processing request
    let processing_request = TemplateProcessingRequest {
        variables: user_variables,
        built_in_variables: all_built_in_variables,
        variable_configs,
        templating_config: template.templating.clone(), // ✅ Use template config
    };

    // ... rest of function ...
}
```

### 3. Template Processing

**File**: `crates/template_engine/src/lib.rs`

```rust
impl TemplateProcessor {
    pub fn process_template(
        &self,
        files: &[(String, Vec<u8>)],
        request: &TemplateProcessingRequest,
        output_dir: &Path,
    ) -> Result<ProcessedTemplate, Error> {
        // ... existing code ...

        for (file_path, content) in files {
            // Apply filtering based on config
            if let Some(ref config) = request.templating_config {
                // Check exclude patterns
                if self.should_exclude_file(file_path, &config.exclude_patterns) {
                    debug!("Excluding file (exclude pattern): {}", file_path);
                    continue;
                }

                // Check include patterns if specified
                if !config.include_patterns.is_empty()
                    && !self.should_include_file(file_path, &config.include_patterns)
                {
                    debug!("Excluding file (not in include): {}", file_path);
                    continue;
                }
            }

            // ... process file ...
        }

        // ... existing code ...
    }
}
```

---

## Testing Strategy

### Unit Tests

**Location**: `crates/config_manager/src/template_config_tests.rs`

```rust
#[test]
fn test_templating_config_deserialization() {
    let toml = r#"
        [template]
        name = "test"
        description = "Test"
        author = "Test"
        tags = []

        [templating]
        include_patterns = ["**/*.rs"]
        exclude_patterns = ["target/**"]
    "#;

    let config: TemplateConfig = toml::from_str(toml).unwrap();
    assert!(config.templating.is_some());

    let templating = config.templating.unwrap();
    assert_eq!(templating.include_patterns.len(), 1);
    assert_eq!(templating.exclude_patterns.len(), 1);
}

#[test]
fn test_optional_templating_config() {
    let toml = r#"
        [template]
        name = "test"
        description = "Test"
        author = "Test"
        tags = []
    "#;

    let config: TemplateConfig = toml::from_str(toml).unwrap();
    assert!(config.templating.is_none());  // Optional field
}
```

**Location**: `crates/config_manager/src/ignore_file_loader_tests.rs`

```rust
#[tokio::test]
async fn test_load_ignore_file_success() {
    // Test loading .reporollerignore from GitHub
}

#[tokio::test]
async fn test_load_ignore_file_not_found() {
    // Test when .reporollerignore doesn't exist
    // Should return Ok(None), not an error
}

#[tokio::test]
async fn test_ignore_file_pattern_parsing() {
    // Test parsing gitignore-style patterns
    // Including comments, negation, etc.
}
```

**Location**: `crates/template_engine/src/lib_tests.rs` (already exists)

```rust
#[test]
fn test_process_template_with_filtering() {
    // ✅ Already exists, verify it still works
}

#[test]
fn test_exclude_overrides_include() {
    let config = TemplatingConfig {
        include_patterns: vec!["**/*.rs".to_string()],
        exclude_patterns: vec!["target/**/*.rs".to_string()],
    };

    // src/main.rs should be included
    // target/debug/main.rs should be excluded
}

#[test]
fn test_empty_include_means_all() {
    let config = TemplatingConfig {
        include_patterns: vec![],
        exclude_patterns: vec!["*.log".to_string()],
    };

    // All files except .log files
}
```

### Integration Tests

**Location**: `crates/integration_tests/tests/template_file_filtering_tests.rs` (new)

```rust
/// Test that [templating] config in template.toml controls file filtering.
#[tokio::test]
async fn test_filtering_via_toml_config() -> Result<()> {
    // 1. Create repository from template-test-filtering
    // 2. Verify included files are present
    // 3. Verify excluded files are absent
    // 4. Use GitHub API to list repository tree for verification
}

/// Test that .reporollerignore patterns are applied.
#[tokio::test]
async fn test_filtering_via_ignore_file() -> Result<()> {
    // Use template with .reporollerignore
    // Verify patterns from ignore file are applied
}

/// Test that TOML and .reporollerignore patterns merge correctly.
#[tokio::test]
async fn test_filtering_pattern_merging() -> Result<()> {
    // Template with both [templating] and .reporollerignore
    // Verify both sets of patterns are applied
}

/// Test that invalid patterns are handled gracefully.
#[tokio::test]
async fn test_invalid_patterns_logged_not_failed() -> Result<()> {
    // Template with invalid glob pattern
    // Should log warning but not fail repository creation
}
```

### Test Template Updates

**File**: `tests/templates/template-test-filtering/.reporoller/template.toml`

```toml
[template]
name = "template-test-filtering"
description = "Template with file filtering rules for testing"
author = "RepoRoller Test Suite"
tags = ["test", "filtering"]

# Add actual filtering configuration
[templating]
include_patterns = [
    "src/**/*.rs",
    "docs/**/*.md",
    "config/**/*.yml",
    "README.md",
    "Cargo.toml",
]

exclude_patterns = [
    "tests/**",
    "examples/**",
    "scripts/**",
    "target/**",
]
```

---

## Usage Examples

### Example 1: Rust Microservice Template

```toml
# .reporoller/template.toml

[template]
name = "rust-microservice"
description = "Production-ready Rust microservice"
author = "Platform Team"
tags = ["rust", "microservice", "backend"]

[templating]
# Include only production code and configuration
include_patterns = [
    "src/**/*.rs",
    "Cargo.toml",
    "Cargo.lock",
    ".gitignore",
    "README.md",
    "LICENSE",
    "config/**/*.toml",
    ".github/workflows/ci.yml",
    ".github/workflows/release.yml",
]

# Exclude development and build artifacts
exclude_patterns = [
    "target/**",
    "examples/**",
    "scripts/dev/**",
    "**/*.log",
    "**/.DS_Store",
]
```

### Example 2: Documentation Template (Minimal Exclusions)

```toml
# .reporoller/template.toml

[template]
name = "docs-site"
description = "Documentation site template"
author = "Docs Team"
tags = ["documentation", "markdown"]

[templating]
# Include everything except build outputs
include_patterns = []  # Empty = include all

exclude_patterns = [
    "node_modules/**",
    "dist/**",
    "build/**",
    "**/*.log",
]
```

### Example 3: Full-Stack Application with Extensive Exclusions

```toml
# .reporoller/template.toml

[template]
name = "full-stack-app"
description = "Full-stack application template"
author = "Platform Team"
tags = ["fullstack", "typescript", "rust"]

[templating]
include_patterns = [
    "src/**",
    "frontend/**",
    "backend/**",
    "shared/**",
]

exclude_patterns = [
    # Build outputs
    "target/**",
    "dist/**",
    "build/**",
    "out/**",
    "**/*.exe",
    "**/*.dll",
    "**/*.so",

    # Dependencies
    "node_modules/**",
    "vendor/**",

    # IDE files
    ".vscode/**",
    ".idea/**",
    "**/*.iml",

    # Logs and temp
    "**/*.log",
    "logs/**",
    "**/*.tmp",
    "**/*.swp",
    "**/*~",

    # OS files
    "**/.DS_Store",
    "**/Thumbs.db",

    # Test coverage
    "coverage/**",
    "**/*.lcov",
]
```

---

## Error Conditions

### Configuration Errors

```rust
#[derive(Debug, Error)]
pub enum ConfigurationError {
    /// Invalid glob pattern in templating config
    #[error("Invalid glob pattern '{pattern}' in template '{template}': {reason}")]
    InvalidGlobPattern {
        template: String,
        pattern: String,
        reason: String,
    },
}
```

### Template Processing Errors

```rust
#[derive(Debug, Error)]
pub enum TemplateError {
    /// File filtering resulted in zero files
    #[error("No files remaining after applying filtering patterns for template '{template_name}'")]
    NoFilesAfterFiltering {
        template_name: String,
        patterns: TemplatingConfig,
    },
}
```

---

## Migration and Compatibility

### Backward Compatibility

✅ **Fully backward compatible**:

- `templating` field is `Option<TemplatingConfig>` with `skip_serializing_if = "Option::is_none"`
- Existing templates without `[templating]` section continue to work
- No filtering applied when `templating` is `None` (all files processed)

### Migration Path for Existing Templates

**No action required** for existing templates. To add filtering:

- Add `[templating]` section to `.reporoller/template.toml`
- Specify `include_patterns` and/or `exclude_patterns` as needed
- Use comments in TOML arrays to organize long pattern lists

---

## Documentation Requirements

### User-Facing Documentation

1. **Template Author Guide** - Add section on file filtering
   - When to use filtering
   - Pattern syntax and examples
   - Common patterns for different languages
   - Organizing long pattern lists
   - Troubleshooting

2. **Template Configuration Reference** - Document `[templating]` section
   - Field descriptions
   - Pattern syntax
   - Examples for common scenarios
   - TOML array formatting tips

### Developer Documentation

1. **Architecture Documentation** - Update with filtering flow
2. **API Documentation** - Rustdoc for all new types and methods
3. **Testing Guide** - How to test template filtering

---

## Success Criteria

### Functional Requirements

✅ Templates can specify filtering in `.reporoller/template.toml` `[templating]` section
✅ `.reporoller/` directory is always excluded (only default exclusion)
✅ `.github/` directory is allowed (templates can provide workflows for target repositories)
✅ Invalid patterns are logged but don't fail repository creation
✅ Filtering results in at least one file (or error)
✅ Exclude patterns take precedence over include patterns
✅ Empty include_patterns means "include all"

### Quality Requirements

✅ All unit tests pass with new configuration fields
✅ Integration tests verify filtering with real templates
✅ Existing templates without filtering continue to work
✅ No breaking changes to configuration format
✅ Clear error messages for misconfigured filters
✅ Documentation complete with examples

### Performance Requirements

✅ Pattern compilation cached during template processing
✅ Filtering adds < 100ms to repository creation time
✅ Large templates (1000+ files) filter efficiently

---

## Implementation Notes

### Dependency Management

**Add to `config_manager/Cargo.toml`**:

```toml
[dependencies]
template_engine = { path = "../template_engine" }
```

This allows `TemplateConfig` to contain `TemplatingConfig`.

### Pattern Compilation

```rust
// Compile patterns once during template loading
let compiled_patterns = config.exclude_patterns
    .iter()
    .filter_map(|p| {
        match globset::Glob::new(p) {
            Ok(glob) => Some(glob.compile_matcher()),
            Err(e) => {
                warn!("Invalid pattern '{}': {}", p, e);
                None
            }
        }
    })
    .collect::<Vec<_>>();
```

### Security Considerations

- **Path traversal prevention**: Already implemented in `template_file_path()`
- **Pattern injection**: Glob patterns cannot escape template root
- **Resource exhaustion**: Limit pattern complexity (handled by `globset` crate)

---

## Open Questions

None - scope is well-defined:

1. ✅ No default exclusion patterns (user decision)
2. ✅ Template-level configuration only
3. ✅ Support both TOML and `.reporollerignore`
4. ✅ Full documentation required
