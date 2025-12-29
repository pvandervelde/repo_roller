# Template Configuration Loading Interface

**Architectural Layer**: Business Logic / Interface Abstraction
**Module Path**: `crates/config_manager/src/template_loader.rs`
**Responsibilities** (from RDD):

- **Knows**: Template repository structure, cache key generation, template configuration schema
- **Does**: Discovers template repositories, loads `.reporoller/template.toml`, caches loaded configurations

**Related Task**: Task 3.0 - Load Template Configuration from Template Repository

## Overview

This interface defines the contract for loading template configurations from template repositories. Templates are stored as GitHub repositories containing a `.reporoller/template.toml` file that defines repository creation settings, required variables, and metadata.

The system provides:

- **Template Discovery**: Finding template repositories by name or topic
- **Configuration Loading**: Reading and parsing `.reporoller/template.toml` files
- **Intelligent Caching**: In-memory caching to reduce GitHub API calls
- **Error Context**: Clear error messages for missing or malformed configurations

## Dependencies

- **Types**: `TemplateConfig`, `TemplateMetadata` (from [template_config.rs](../../crates/config_manager/src/template_config.rs))
- **Errors**: `ConfigurationError` variants for template loading failures
- **GitHub Client**: `GitHubClient` trait for repository access
- **Standard Types**: `HashMap`, `Arc`, `RwLock` for caching

## Architecture Context

```
OrganizationSettingsManager
    ↓ uses
TemplateLoader (this interface)
    ↓ depends on (abstraction)
TemplateRepository trait
    ↑ implemented by
GitHubTemplateRepository (infrastructure)
```

**Clean Architecture Boundaries**:

- **Core Domain**: Template loading orchestration logic
- **Interface/Port**: `TemplateRepository` trait (abstraction for template access)
- **Infrastructure/Adapter**: GitHub-specific implementation

## Interface Definitions

### TemplateRepository Trait

Interface for accessing template repositories. Abstracts away the storage mechanism (GitHub, filesystem, etc.).

```rust
/// Interface for accessing template repositories.
///
/// This trait abstracts the storage mechanism for templates, allowing
/// implementations backed by GitHub, local filesystem, or other sources.
///
/// Implementations must be thread-safe (`Send + Sync`).
#[async_trait]
pub trait TemplateRepository: Send + Sync {
    /// Load template configuration from a template repository.
    ///
    /// Reads and parses the `.reporoller/template.toml` file from the
    /// specified template repository in the given organization.
    ///
    /// # Arguments
    ///
    /// * `org` - GitHub organization or user name
    /// * `template_name` - Name of the template repository
    ///
    /// # Returns
    ///
    /// Returns the parsed `TemplateConfig` structure.
    ///
    /// # Errors
    ///
    /// * `ConfigurationError::TemplateNotFound` - Repository doesn't exist or not accessible
    /// * `ConfigurationError::TemplateConfigurationMissing` - No `.reporoller/template.toml` file
    /// * `ConfigurationError::ParseError` - Invalid TOML syntax
    /// * `ConfigurationError::InvalidConfiguration` - Missing required fields or invalid values
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use async_trait::async_trait;
    /// # use config_manager::{TemplateRepository, TemplateConfig};
    /// # async fn example(repo: &dyn TemplateRepository) {
    /// let config = repo
    ///     .load_template_config("my-org", "rust-service-template")
    ///     .await
    ///     .expect("Failed to load template config");
    ///
    /// println!("Template: {}", config.template.name);
    /// # }
    /// ```
    async fn load_template_config(
        &self,
        org: &str,
        template_name: &str,
    ) -> ConfigurationResult<TemplateConfig>;

    /// Check if a template repository exists and is accessible.
    ///
    /// Verifies that the template repository exists and the current
    /// authentication context has read access.
    ///
    /// # Arguments
    ///
    /// * `org` - GitHub organization or user name
    /// * `template_name` - Name of the template repository
    ///
    /// # Returns
    ///
    /// Returns `true` if the repository exists and is accessible, `false` otherwise.
    ///
    /// # Errors
    ///
    /// Returns errors only for API failures, not for missing repositories.
    async fn template_exists(&self, org: &str, template_name: &str) -> ConfigurationResult<bool>;
}
```

### TemplateLoader Type

Orchestrates template loading with caching support.

```rust
/// Template configuration loader with intelligent caching.
///
/// Loads template configurations from template repositories and caches
/// them to minimize GitHub API calls. Thread-safe for concurrent access.
///
/// # Cache Behavior
///
/// - Cache key: `(organization, template_name)`
/// - No automatic expiration (manual invalidation via `invalidate_cache()`)
/// - Cache persists for application lifetime
/// - Thread-safe concurrent access via `RwLock`
///
/// # Examples
///
/// ```no_run
/// use config_manager::{TemplateLoader, GitHubTemplateRepository};
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let github_repo = Arc::new(GitHubTemplateRepository::new(/* ... */));
/// let loader = TemplateLoader::new(github_repo);
///
/// let config = loader
///     .load_template_configuration("my-org", "rust-service")
///     .await?;
///
/// println!("Loaded template: {}", config.template.name);
/// # Ok(())
/// # }
/// ```
pub struct TemplateLoader {
    /// Template repository implementation
    repository: Arc<dyn TemplateRepository>,

    /// Configuration cache: (org, template_name) -> TemplateConfig
    cache: Arc<RwLock<HashMap<TemplateCacheKey, TemplateConfig>>>,

    /// Cache statistics for monitoring
    stats: Arc<RwLock<CacheStatistics>>,
}

impl TemplateLoader {
    /// Create a new template loader.
    ///
    /// # Arguments
    ///
    /// * `repository` - Template repository implementation
    pub fn new(repository: Arc<dyn TemplateRepository>) -> Self;

    /// Load template configuration with caching.
    ///
    /// Checks cache first, loads from repository on cache miss.
    /// Updates cache and statistics on successful load.
    ///
    /// # Arguments
    ///
    /// * `org` - GitHub organization or user name
    /// * `template_name` - Name of the template repository
    ///
    /// # Returns
    ///
    /// Returns the loaded or cached `TemplateConfig`.
    ///
    /// # Errors
    ///
    /// See `TemplateRepository::load_template_config` for error conditions.
    ///
    /// # Thread Safety
    ///
    /// Safe for concurrent calls. Cache access is synchronized.
    pub async fn load_template_configuration(
        &self,
        org: &str,
        template_name: &str,
    ) -> ConfigurationResult<TemplateConfig>;

    /// Invalidate cached template configuration.
    ///
    /// Removes a specific template from cache, forcing next load
    /// to fetch fresh configuration from repository.
    ///
    /// # Arguments
    ///
    /// * `org` - GitHub organization or user name
    /// * `template_name` - Name of the template repository
    ///
    /// # Returns
    ///
    /// Returns `true` if entry was cached and removed, `false` if not cached.
    pub fn invalidate_cache(&self, org: &str, template_name: &str) -> bool;

    /// Clear all cached template configurations.
    ///
    /// Removes all entries from cache. Useful for testing or
    /// when forcing a complete refresh.
    pub fn clear_cache(&self);

    /// Get cache statistics for monitoring.
    ///
    /// Returns snapshot of cache performance metrics.
    pub fn cache_statistics(&self) -> CacheStatistics;

    /// Check if a template exists and is accessible.
    ///
    /// Does not use cache; always checks with repository.
    ///
    /// # Arguments
    ///
    /// * `org` - GitHub organization or user name
    /// * `template_name` - Name of the template repository
    pub async fn template_exists(
        &self,
        org: &str,
        template_name: &str,
    ) -> ConfigurationResult<bool>;
}
```

### Supporting Types

```rust
/// Cache key for template configurations.
///
/// Uniquely identifies a template by organization and name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TemplateCacheKey {
    organization: String,
    template_name: String,
}

/// Cache performance statistics.
///
/// Tracks cache effectiveness for monitoring and optimization.
#[derive(Debug, Clone, Copy, Default)]
pub struct CacheStatistics {
    /// Total number of load requests
    pub total_requests: u64,

    /// Number of requests served from cache
    pub cache_hits: u64,

    /// Number of requests that required repository load
    pub cache_misses: u64,

    /// Current number of cached templates
    pub cached_entries: usize,
}

impl CacheStatistics {
    /// Calculate cache hit ratio (0.0 to 1.0).
    ///
    /// Returns 0.0 if no requests have been made.
    pub fn hit_ratio(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.cache_hits as f64 / self.total_requests as f64
        }
    }
}
```

## Error Conditions

### New Error Variants

These variants extend `ConfigurationError` (defined in [error-types.md](error-types.md)):

```rust
/// Template-specific configuration errors.
pub enum ConfigurationError {
    // ... existing variants ...

    /// Template repository not found or not accessible.
    ///
    /// Occurs when:
    /// - Repository doesn't exist
    /// - Repository is private and not accessible with current credentials
    /// - Organization doesn't exist
    #[error("Template repository not found: {org}/{template}")]
    TemplateNotFound {
        org: String,
        template: String,
    },

    /// Template repository exists but has no `.reporoller/template.toml` file.
    ///
    /// Occurs when repository is valid but missing configuration file.
    #[error("Template configuration file missing in {org}/{template}: expected .reporoller/template.toml")]
    TemplateConfigurationMissing {
        org: String,
        template: String,
    },
}
```

### Error Context

All errors include:

- Organization name
- Template name
- Operation being performed
- Underlying cause (when applicable)

## Integration with OrganizationSettingsManager

The `OrganizationSettingsManager` will use `TemplateLoader` instead of creating stub configurations:

```rust
// Current code (line ~299 in organization_settings_manager.rs):
// TODO: Load actual template configuration from template repository
let template_config = crate::template_config::TemplateConfig { /* stub */ };

// After Task 3.0 implementation:
let template_config = self
    .template_loader
    .load_template_configuration(context.organization(), context.template())
    .await
    .map_err(|e| {
        warn!("Failed to load template configuration: {}", e);
        e
    })?;
```

## Testing Requirements

### Unit Tests

**TemplateLoader Tests** (`template_loader_tests.rs`):

1. **Cache Hit**: Verify second load uses cache (no repository call)
2. **Cache Miss**: Verify first load calls repository
3. **Cache Invalidation**: Verify invalidated entry is reloaded
4. **Clear Cache**: Verify all entries removed
5. **Statistics Tracking**: Verify hit/miss counting
6. **Concurrent Access**: Verify thread-safe cache access
7. **Error Propagation**: Verify repository errors propagate correctly

**Mock TemplateRepository** for unit tests:

```rust
struct MockTemplateRepository {
    templates: HashMap<(String, String), TemplateConfig>,
    call_count: Arc<Mutex<usize>>,
}
```

### Integration Tests

**Against Real GitHub** (`integration_tests/template_loading_tests.rs`):

1. **Load Real Template**: Load from glitchgrove template repositories
2. **Missing Template**: Verify error for non-existent template
3. **Missing Config File**: Verify error for repo without `.reporoller/template.toml`
4. **Invalid TOML**: Verify parse error handling
5. **Cache Effectiveness**: Verify multiple loads use cache
6. **Template Exists Check**: Verify existence checking

**Test Templates in glitchgrove**:

- Use existing `template-test-basic` repository
- Add `.reporoller/template.toml` to test templates
- Create `template-no-config` repository for negative testing

### Contract Tests

Verify all `TemplateRepository` implementations satisfy the contract:

```rust
async fn test_template_repository_contract<T: TemplateRepository>(repo: T) {
    // Verify load succeeds for valid template
    // Verify errors for missing template
    // Verify errors for missing config file
}
```

## Performance Constraints

- **Cache Lookup**: O(1) hash map lookup, < 1ms
- **First Load**: Depends on GitHub API (typically < 500ms)
- **Cached Load**: < 1ms (memory access only)
- **Cache Memory**: ~10KB per cached template configuration

**Monitoring**:

- Log cache hit ratio periodically
- Warn if cache hit ratio < 50% (indicates cache ineffectiveness)
- Track repository load times

## Security Considerations

### Sensitive Data

- **No secrets in template configs**: Templates define structure, not credentials
- **Logging**: Safe to log template names and organizations (public information)

### Access Control

- Template loading uses GitHub API authentication
- Private templates require appropriate access tokens
- Access denied errors mapped to `TemplateNotFound` (don't leak existence)

### Resource Limits

- **Cache Size**: No automatic eviction (monitor memory usage)
- **Load Timeout**: 30 seconds for GitHub API calls (GitHub client responsibility)
- **Concurrent Loads**: No artificial limit (GitHub API rate limits apply)

## Implementation Notes

### File Path

Template configuration must be at **exact path**:

- `.reporoller/template.toml` (lowercase, exact spelling)
- Use GitHub Contents API to fetch file
- Handle 404 as `TemplateConfigurationMissing`

### TOML Parsing

```rust
use serde::Deserialize;

// Parse TOML content
let config: TemplateConfig = toml::from_str(&content)
    .map_err(|e| ConfigurationError::ParseError {
        file_path: ".reporoller/template.toml".to_string(),
        line: e.line_col().map(|(line, _)| line),
        message: e.to_string(),
    })?;
```

### Cache Key Generation

```rust
impl TemplateCacheKey {
    fn new(org: impl Into<String>, template: impl Into<String>) -> Self {
        Self {
            organization: org.into(),
            template_name: template.into(),
        }
    }
}
```

### Thread Safety

- `Arc<RwLock<HashMap>>` for cache allows:
  - Multiple concurrent readers (cache hits)
  - Single writer for updates (cache misses)
  - Safe concurrent invalidation

## Migration Strategy

### Phase 1: Add Interface and Stubs

- Define `TemplateRepository` trait
- Create `TemplateLoader` with basic cache
- Add new error variants
- **All stubs compile, tests skipped**

### Phase 2: Implement GitHub Backend

- Implement `GitHubTemplateRepository`
- Use `GitHubClient::get_file_content()` to read `.reporoller/template.toml`
- Handle 404 errors appropriately

### Phase 3: Integrate with OrganizationSettingsManager

- Replace stub template creation (line ~299)
- Add `TemplateLoader` to manager dependencies
- Update constructor to accept loader

### Phase 4: Testing

- Unit tests with mock repository
- Integration tests against glitchgrove
- Verify cache effectiveness

## Example Usage

### Basic Usage

```rust
use config_manager::{TemplateLoader, GitHubTemplateRepository};
use std::sync::Arc;

// Create loader with GitHub backend
let github_repo = Arc::new(GitHubTemplateRepository::new(github_client));
let loader = TemplateLoader::new(github_repo);

// Load template (first time: GitHub API call)
let config1 = loader
    .load_template_configuration("myorg", "rust-service")
    .await?;

// Load again (served from cache)
let config2 = loader
    .load_template_configuration("myorg", "rust-service")
    .await?;

// Check statistics
let stats = loader.cache_statistics();
println!("Cache hit ratio: {:.1}%", stats.hit_ratio() * 100.0);
```

### With Invalidation

```rust
// Load template
let config = loader.load_template_configuration("myorg", "rust-service").await?;

// Template was updated in GitHub...

// Force refresh by invalidating cache
loader.invalidate_cache("myorg", "rust-service");

// Next load will fetch fresh configuration
let fresh_config = loader.load_template_configuration("myorg", "rust-service").await?;
```

### Checking Template Existence

```rust
// Check if template exists before trying to load
if loader.template_exists("myorg", "new-template").await? {
    let config = loader.load_template_configuration("myorg", "new-template").await?;
    println!("Template found: {}", config.template.name);
} else {
    println!("Template not found or not accessible");
}
```

## Related Specifications

- [Template Configuration Types](../../crates/config_manager/src/template_config.rs)
- [Configuration Error Types](error-types.md)
- [GitHub Client Interface](github-interfaces.md)
- [Organization Settings Manager](configuration-interfaces.md)
- [Template Processing Design](../design/template-processing.md)

## Changelog

- **2024-12-28**: Initial interface specification
