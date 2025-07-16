# Integration Test Templates Specification

This document specifies the test template repositories required for RepoRoller integration testing.

## Overview

The integration tests require four specific template repositories to validate different aspects of RepoRoller functionality. These templates should be created in the `pvandervelde` GitHub organization and must be publicly accessible.

## Required Template Repositories

### 1. test-basic

**Repository**: `https://github.com/pvandervelde/test-basic`
**Purpose**: Basic repository creation with minimal template functionality
**Test Scenario**: BasicCreation

#### Structure

```
test-basic/
├── README.md
├── .gitignore
├── src/
│   └── main.rs
└── Cargo.toml
```

#### Files

**README.md**

```markdown
# Test Basic Template

This is a basic test template for RepoRoller integration testing.

## Purpose

This template validates basic repository creation functionality without complex features.

## Usage

This template is used by RepoRoller integration tests to verify:
- Basic file copying
- Repository structure creation
- Minimal template processing
```

**.gitignore**

```
target/
*.log
.env
```

**src/main.rs**

```rust
fn main() {
    println!("Hello from test-basic template!");
}
```

**Cargo.toml**

```toml
[package]
name = "test-basic"
version = "0.1.0"
edition = "2021"

[dependencies]
```

### 2. test-variables

**Repository**: `https://github.com/pvandervelde/test-variables`
**Purpose**: Variable substitution testing
**Test Scenario**: VariableSubstitution

#### Structure

```
test-variables/
├── README.md
├── .gitignore
├── src/
│   └── main.rs
├── Cargo.toml
└── config.yml
```

#### Files

**README.md**

```markdown
# {{project_name}}

{{project_description}}

## Author

Created by {{author_name}} ({{author_email}})

## Version

Version: {{version}}

## License

{{license}}
```

**.gitignore**

```
target/
*.log
.env
{{custom_ignore}}
```

**src/main.rs**

```rust
fn main() {
    println!("Hello from {{project_name}}!");
    println!("Version: {{version}}");
    println!("Author: {{author_name}}");
}
```

**Cargo.toml**

```toml
[package]
name = "{{project_name}}"
version = "{{version}}"
edition = "2021"
authors = ["{{author_name}} <{{author_email}}>"]
description = "{{project_description}}"
license = "{{license}}"

[dependencies]
```

**config.yml**

```yaml
project:
  name: "{{project_name}}"
  version: "{{version}}"
  author: "{{author_name}}"
  email: "{{author_email}}"
  description: "{{project_description}}"
  license: "{{license}}"

settings:
  debug: {{debug_mode}}
  environment: "{{environment}}"
```

#### Expected Variables

- `project_name`: Name of the project
- `project_description`: Description of the project
- `author_name`: Author's name
- `author_email`: Author's email
- `version`: Project version (default: "0.1.0")
- `license`: License type (default: "MIT")
- `custom_ignore`: Additional gitignore patterns
- `debug_mode`: Boolean for debug mode (default: true)
- `environment`: Environment name (default: "development")

### 3. test-filtering

**Repository**: `https://github.com/pvandervelde/test-filtering`
**Purpose**: File filtering based on include/exclude patterns
**Test Scenario**: FileFiltering

#### Structure

```
test-filtering/
├── README.md
├── .gitignore
├── src/
│   ├── main.rs
│   ├── lib.rs
│   └── utils.rs
├── tests/
│   ├── integration_test.rs
│   └── unit_test.rs
├── docs/
│   ├── api.md
│   └── guide.md
├── examples/
│   ├── basic.rs
│   └── advanced.rs
├── scripts/
│   ├── build.sh
│   └── deploy.sh
├── config/
│   ├── development.yml
│   ├── production.yml
│   └── test.yml
├── Cargo.toml
└── LICENSE
```

#### Files

**README.md**

```markdown
# Test Filtering Template

This template contains various file types to test RepoRoller's file filtering capabilities.

## Structure

- `src/` - Source code files
- `tests/` - Test files
- `docs/` - Documentation files
- `examples/` - Example files
- `scripts/` - Build and deployment scripts
- `config/` - Configuration files
```

**src/main.rs**

```rust
mod utils;

fn main() {
    println!("Test filtering template");
    utils::helper_function();
}
```

**src/lib.rs**

```rust
pub mod utils;

pub fn library_function() {
    println!("Library function");
}
```

**src/utils.rs**

```rust
pub fn helper_function() {
    println!("Helper function");
}
```

**tests/integration_test.rs**

```rust
#[test]
fn integration_test() {
    assert_eq!(2 + 2, 4);
}
```

**tests/unit_test.rs**

```rust
#[test]
fn unit_test() {
    assert!(true);
}
```

**docs/api.md**

```markdown
# API Documentation

This is the API documentation.
```

**docs/guide.md**

```markdown
# User Guide

This is the user guide.
```

**examples/basic.rs**

```rust
fn main() {
    println!("Basic example");
}
```

**examples/advanced.rs**

```rust
fn main() {
    println!("Advanced example");
}
```

**scripts/build.sh**

```bash
#!/bin/bash
cargo build --release
```

**scripts/deploy.sh**

```bash
#!/bin/bash
echo "Deploying application..."
```

**config/development.yml**

```yaml
environment: development
debug: true
```

**config/production.yml**

```yaml
environment: production
debug: false
```

**config/test.yml**

```yaml
environment: test
debug: true
```

**Cargo.toml**

```toml
[package]
name = "test-filtering"
version = "0.1.0"
edition = "2021"

[dependencies]
```

**LICENSE**

```
MIT License

Copyright (c) 2024 Test Template

Permission is hereby granted, free of charge, to any person obtaining a copy...
```

#### Filtering Test Cases

The integration tests should verify:

- **Include patterns**: `src/**/*.rs`, `docs/*.md`, `config/*.yml`
- **Exclude patterns**: `tests/**`, `examples/**`, `scripts/**`
- **Mixed patterns**: Include `src/` but exclude `src/utils.rs`
- **File extension filtering**: Only `.rs` files, only `.md` files

### 4. test-invalid

**Repository**: `https://github.com/pvandervelde/test-invalid`
**Purpose**: Error handling for invalid configurations
**Test Scenario**: ErrorHandling

#### Structure

```
test-invalid/
├── README.md
├── invalid-template.yml
├── src/
│   └── broken.rs
└── malformed.toml
```

#### Files

**README.md**

```markdown
# Test Invalid Template

This template is designed to test error handling in RepoRoller.

## Purpose

This template contains intentionally problematic files to validate:
- Invalid template syntax
- Malformed configuration files
- Missing required variables
- Circular variable references
```

**invalid-template.yml**

```yaml
# This file contains invalid template syntax
project:
  name: "{{missing_variable}}"
  circular_ref: "{{another_circular}}"
  another_circular: "{{circular_ref}}"
  invalid_syntax: "{{unclosed_variable"
  nested_error: "{{valid_var}} and {{invalid_nested"
```

**src/broken.rs**

```rust
// This file contains invalid variable references
fn main() {
    println!("Project: {{undefined_variable}}");
    println!("Circular: {{circular_a}}");
    // {{circular_a}} -> {{circular_b}} -> {{circular_a}}
    let value = "{{malformed_syntax";
}
```

**malformed.toml**

```toml
[package
name = "{{project_name}}"
# Missing closing bracket above
version = "{{undefined_version}}"
invalid_key = {{missing_quotes}}

[dependencies]
# This section has syntax errors
invalid-dep = "{{missing_version}"
```

#### Expected Error Conditions

The integration tests should verify proper error handling for:

- **Undefined variables**: References to variables not provided
- **Circular references**: Variables that reference each other
- **Malformed syntax**: Invalid template syntax
- **Invalid file formats**: Malformed TOML, YAML, etc.
- **Missing required files**: Templates that reference non-existent files

## Template Repository Requirements

### General Requirements

1. **Public repositories** in the `pvandervelde` GitHub organization
2. **MIT License** for all templates
3. **Clear README** explaining the template's purpose
4. **Consistent structure** following the specifications above
5. **Version tags** for stable releases

### Repository Settings

- **Visibility**: Public
- **Template repository**: Enabled
- **Issues**: Enabled for bug reports
- **Wiki**: Disabled
- **Projects**: Disabled
- **Security**: Default branch protection

### Branch Protection

- **Protect main branch**
- **Require pull request reviews**
- **Require status checks**
- **Restrict pushes to main**

## Integration Test Usage

The integration tests will:

1. **Clone templates** from the specified GitHub URLs
2. **Process variables** according to each test scenario
3. **Apply file filtering** based on test patterns
4. **Validate results** against expected outcomes
5. **Clean up** created repositories after testing

## Maintenance

### Updates

- Templates should be updated when RepoRoller functionality changes
- Version tags should be created for major template updates
- Breaking changes should be documented in release notes

### Testing

- Templates should be manually tested with RepoRoller before releases
- Integration tests should validate all template functionality
- Error templates should be verified to produce expected errors

## Security Considerations

- **No sensitive data** in templates
- **No executable scripts** that could be harmful
- **Validate all user inputs** in variable substitution
- **Sanitize file paths** to prevent directory traversal
