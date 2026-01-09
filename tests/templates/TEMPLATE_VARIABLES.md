# Template Variables Reference

This document lists all the variables required by each test template in the `tests/templates/` directory. Use this as a reference when writing integration tests to ensure all required variables are provided.

## Overview

Templates use Handlebars syntax (`{{variable_name}}`) for variable substitution. When creating repositories from templates that use variables, **all referenced variables must be provided** in the `RepositoryCreationRequestBuilder` or the template processing will fail.

## Template Variable Requirements

### template-test-variables

**Location**: `tests/templates/template-test-variables/`

**Default Visibility**: `public`

**Required Variables**:

| Variable | Type | Description | Example Value |
|----------|------|-------------|---------------|
| `project_name` | string | Project name | `"my-project"` |
| `version` | string | Version number | `"0.1.0"` |
| `author_name` | string | Author's name | `"Integration Test"` |
| `author_email` | string | Author's email | `"test@example.com"` |
| `project_description` | string | Project description | `"A test project"` |
| `license` | string | License identifier | `"MIT"` |
| `license_type` | string | License type (legacy) | `"MIT"` |
| `environment` | string | Environment name | `"test"` |
| `debug_mode` | string | Debug mode flag | `"true"` or `"false"` |

**Files Using Variables**:

- `README.md` - Uses: `project_name`, `project_description`, `author_name`, `author_email`, `version`, `license`
- `Cargo.toml` - Uses: `project_name`, `version`, `author_name`, `author_email`, `project_description`, `license`
- `config.yml` - Uses: `project_name`, `version`, `author_name`, `author_email`, `project_description`, `license`, `debug_mode`, `environment`
- `src/main.rs` - May use project-related variables

**Example Usage**:

```rust
let mut variables = std::collections::HashMap::new();
variables.insert("project_name".to_string(), "my-project".to_string());
variables.insert("version".to_string(), "0.1.0".to_string());
variables.insert("author_name".to_string(), "Integration Test".to_string());
variables.insert("author_email".to_string(), "test@example.com".to_string());
variables.insert("project_description".to_string(), "A test project".to_string());
variables.insert("license".to_string(), "MIT".to_string());
variables.insert("license_type".to_string(), "MIT".to_string());
variables.insert("environment".to_string(), "test".to_string());
variables.insert("debug_mode".to_string(), "true".to_string());

let request = RepositoryCreationRequestBuilder::new(
    repo_name,
    org_name,
    TemplateName::new("template-test-variables")?,
)
.variables(variables)
.build();
```

### template-nested-variables

**Location**: `tests/templates/template-nested-variables/`

**Default Visibility**: Not specified (uses system default)

**Required Variables**:

| Variable | Type | Description | Example Value |
|----------|------|-------------|---------------|
| `first_name` | string | First name | `"John"` |
| `last_name` | string | Last name | `"Doe"` |

**Derived Variables** (automatically calculated):

- `full_name` - Defaults to `"{{first_name}} {{last_name}}"`
- `greeting` - Defaults to `"Hello, {{full_name}}!"`
- `farewell` - Defaults to `"Goodbye, {{full_name}}. Have a great day!"`

**Files Using Variables**:

- `config.toml` - Defines variable relationships
- `greeting.txt` - Uses: `first_name`, `last_name`, `full_name`, `greeting`, `farewell`

**Example Usage**:

```rust
let request = RepositoryCreationRequestBuilder::new(
    repo_name,
    org_name,
    TemplateName::new("template-nested-variables")?,
)
.variable("first_name", "John")
.variable("last_name", "Doe")
.build();
```

### template-variable-paths

**Location**: `tests/templates/template-variable-paths/`

**Default Visibility**: Not specified (uses system default)

**Required Variables**:

| Variable | Type | Description | Example Value | Validation Pattern |
|----------|------|-------------|---------------|-------------------|
| `project_name` | string | Project name (used in file/directory paths) | `"myproject"` | `^[a-zA-Z][a-zA-Z0-9_]*$` |

**Files/Directories Using Variables**:

- `{{project_name}}/` - Directory name contains variable
- `{{project_name}}/{{project_name}}_config.json` - File name contains variable
- `{{project_name}}/src/{{project_name}}_main.rs` - File name contains variable
- `{{project_name}}/tests/test_{{project_name}}.rs` - File name contains variable
- `config.toml` - File content uses variable

**Example Usage**:

```rust
let request = RepositoryCreationRequestBuilder::new(
    repo_name,
    org_name,
    TemplateName::new("template-variable-paths")?,
)
.variable("project_name", "myproject")
.build();
```

### template-test-basic

**Location**: `tests/templates/template-test-basic/`

**Default Visibility**: `private`

**Required Variables**: None

This template does not use any variables. It can be used without providing any variable values.

**Example Usage**:

```rust
let request = RepositoryCreationRequestBuilder::new(
    repo_name,
    org_name,
    TemplateName::new("template-test-basic")?,
)
.build(); // No variables needed
```

### Other Templates

The following templates do not use variables and can be used directly:

- `template-test-filtering` - No variables
- `template-test-invalid` - No variables
- `template-large-files` - No variables
- `template-binary-files` - No variables
- `template-deep-nesting` - No variables
- `template-many-files` - No variables
- `template-unicode-names` - No variables
- `template-empty-dirs` - No variables
- `template-no-extensions` - No variables
- `template-with-dotfiles` - No variables
- `template-with-scripts` - No variables
- `template-with-symlinks` - No variables

## Common Mistakes

### 1. Missing Required Variables

**Problem**: Template fails with error like:

```
Template variable validation failed: template_content - Template rendering failed:
Variable validation failed: unknown - Error rendering "Unnamed template" line 4, col 12:
Failed to access variable in strict mode Some("author_name")
```

**Solution**: Ensure all required variables for the template are provided. Check this document for the complete list.

### 2. Partial Variable Sets

**Problem**: Providing only some variables when the template requires multiple (e.g., only `project_name` and `version` when 9 variables are required).

**Solution**: Always provide the complete set of variables listed for the template. Use the example code blocks in this document as a reference.

### 3. Variable Naming Mismatches

**Problem**: Using `projectName` instead of `project_name`, or `author` instead of `author_name`.

**Solution**: Variable names are case-sensitive and must match exactly. Use the exact names listed in the tables above.

## Testing Guidelines

When writing integration tests that use templates with variables:

1. **Check this document first** to identify all required variables
2. **Provide all required variables** - never provide a partial set
3. **Use consistent test values** - the examples in this document show the standard test values
4. **Add comments** - note which template you're using and link to this doc:

   ```rust
   // Must provide all required variables for template-test-variables
   // See tests/templates/TEMPLATE_VARIABLES.md
   ```

## Updating This Document

When adding new templates with variables or modifying existing templates:

1. **Add the template to this document** with complete variable list
2. **Document all files** that use the variables
3. **Provide working example code** that other developers can copy
4. **Update existing tests** if variable requirements change
5. **Run all integration tests** to ensure nothing breaks

## Variable Validation

The template engine validates variables in strict mode, which means:

- **All referenced variables must be provided** - no defaults for undefined variables
- **Variable names must match exactly** - case-sensitive
- **Empty strings are valid** - but omitting a variable is not
- **Type validation** - some templates may specify type requirements in their `.reporoller/template.toml`

## See Also

- [Template Engine Documentation](../../crates/template_engine/README.md)
- [Integration Test Guidelines](../README.md)
- [Template Configuration Format](../../specs/templates/configuration.md)
