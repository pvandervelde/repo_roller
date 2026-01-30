# Template Processing Interfaces

**Architectural Layer**: Business Interface (trait) + Infrastructure (file system + Handlebars implementation)
**Crate**: `template_engine`
**Responsibilities**:

- **Knows**: Variable substitution rules, template file formats
- **Does**: Fetches template files, processes variables, validates templates

---

## Overview

Template processing handles fetching template files from source repositories and performing variable substitution to generate final repository content.

## Current State

**Existing Traits** (already defined):

- `TemplateFetcher` - Fetches template files from source

**Existing Types**:

- `TemplateProcessor` - Performs variable substitution
- `TemplateProcessingRequest` - Input for processing
- `VariableConfig` - Variable validation rules

**Status**: ✅ Generally well-designed

**TODO**: Add type safety with branded types

---

## Existing Interface: TemplateFetcher

```rust
#[async_trait]
pub trait TemplateFetcher: Send + Sync {
    async fn fetch_template_files(&self, source_repo: &str)
        -> Result<Vec<(String, Vec<u8>)>, TemplateEngineError>;
}
```

**Status**: ✅ Well-designed

**TODO**: Change `source_repo: &str` to a typed `TemplateSource` or URL type

---

## Template Processor

```rust
pub struct TemplateProcessor {
    // Handlebars registry
}

impl TemplateProcessor {
    pub fn new() -> Result<Self, TemplateEngineError>;

    pub fn process_template(
        &self,
        files: &[(String, Vec<u8>)],
        request: &TemplateProcessingRequest,
        base_path: &Path,
    ) -> Result<ProcessedTemplate, TemplateEngineError>;

    pub fn generate_built_in_variables(
        &self,
        params: &BuiltInVariablesParams,
    ) -> HashMap<String, String>;
}
```

**Status**: ✅ Well-designed

**TODO**: Migrate `BuiltInVariablesParams` to use branded types

---

## Types

### TemplateProcessingRequest

```rust
pub struct TemplateProcessingRequest {
    pub variables: HashMap<String, String>,
    pub built_in_variables: HashMap<String, String>,
    pub variable_configs: HashMap<String, VariableConfig>,
    pub templating_config: Option<TemplatingConfig>,
}
```

### BuiltInVariablesParams

```rust
pub struct BuiltInVariablesParams<'a> {
    pub repo_name: &'a str, // TODO: Change to &'a RepositoryName
    pub org_name: &'a str,  // TODO: Change to &'a OrganizationName
    pub template_name: &'a str, // TODO: Change to &'a TemplateName
    pub template_repo: &'a str,
    pub user_login: &'a str,
    pub user_name: &'a str,
    pub default_branch: &'a str,
}
```

### ProcessedTemplate

```rust
pub struct ProcessedTemplate {
    pub files: Vec<(String, Vec<u8>)>,
}
```

---

## Variable Configuration

### VariableConfig

```rust
pub struct VariableConfig {
    pub description: Option<String>,
    pub example: Option<String>,
    pub required: bool,
    pub pattern: Option<String>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub options: Option<Vec<String>>,
    pub default: Option<String>,
}
```

**Status**: ✅ Comprehensive validation support

---

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum TemplateEngineError {
    #[error("Template fetch failed: {0}")]
    FetchError(String),

    #[error("Variable substitution failed: {0}")]
    SubstitutionError(String),

    #[error("Variable validation failed: {0}")]
    ValidationError(String),
}
```

---

## Implementation Notes

- Uses Handlebars for variable substitution
- Supports `{{variable}}` syntax
- Validates all required variables before processing
- Handles binary files (no substitution)
- Supports conditional blocks and loops

---

## Migration Path

1. Add branded type parameters to `BuiltInVariablesParams`
2. Update template fetcher to use typed source references
3. Add comprehensive error context
4. Improve variable validation error messages

---

## Related Specifications

- `specs/interfaces/repository-domain.md` - Uses template engine
- `specs/interfaces/shared-types.md` - Type definitions
- `specs/interfaces/error-types.md` - Error hierarchy
