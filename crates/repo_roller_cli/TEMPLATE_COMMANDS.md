# RepoRoller CLI - Template Commands

This guide covers the template inspection and validation commands in RepoRoller CLI.

## Overview

The template commands allow you to discover, inspect, and validate repository templates in your organization without creating repositories. These commands are useful for:

- Discovering available templates before repository creation
- Validating template configurations before use
- Inspecting template variables and requirements
- Troubleshooting template issues

## Commands

### `repo-roller template info`

Get detailed information about a specific template.

**Usage:**

```bash
repo-roller template info --org <ORGANIZATION> --template <TEMPLATE_NAME> [--format <FORMAT>]
```

**Arguments:**

- `--org <ORGANIZATION>`: Organization name (required)
- `--template <TEMPLATE_NAME>`: Template repository name (required)
- `--format <FORMAT>`: Output format - `json` or `pretty` (default: `pretty`)

**Example:**

```bash
# Get template information in pretty format
repo-roller template info --org myorg --template rust-library

# Get template information as JSON
repo-roller template info --org myorg --template rust-library --format json
```

**Output (Pretty Format):**

```
Template: rust-library

Description: Production-ready Rust library template with CI/CD
Author: Platform Team
Tags: rust, library, ci-cd

Repository Type: library (policy: fixed)

Variables (2):
  ✓ project_name [required]
    Human-readable project name
    Example: my-awesome-library

  • service_port [optional]
    Service port number
    Default: 8080
    Example: 3000

Configuration: 3 sections defined
```

**Output (JSON Format):**

```json
{
  "name": "rust-library",
  "description": "Production-ready Rust library template with CI/CD",
  "author": "Platform Team",
  "tags": ["rust", "library", "ci-cd"],
  "repository_type": {
    "type_name": "library",
    "policy": "fixed"
  },
  "variables": [
    {
      "name": "project_name",
      "description": "Human-readable project name",
      "required": true,
      "example": "my-awesome-library"
    },
    {
      "name": "service_port",
      "description": "Service port number",
      "required": false,
      "default_value": "8080",
      "example": "3000"
    }
  ],
  "configuration_sections": 3
}
```

### `repo-roller template validate`

Validate a template's structure and configuration.

**Usage:**

```bash
repo-roller template validate --org <ORGANIZATION> --template <TEMPLATE_NAME> [--format <FORMAT>]
```

**Arguments:**

- `--org <ORGANIZATION>`: Organization name (required)
- `--template <TEMPLATE_NAME>`: Template repository name (required)
- `--format <FORMAT>`: Output format - `json` or `pretty` (default: `pretty`)

**Example:**

```bash
# Validate template in pretty format
repo-roller template validate --org myorg --template rust-library

# Validate template as JSON
repo-roller template validate --org myorg --template rust-library --format json
```

**Output (Pretty Format - Valid Template):**

```
Validating template: rust-library

✓ Template repository accessible
✓ Configuration file found (.reporoller/template.toml)
✓ Template metadata complete
✓ Variables valid (2 defined)
✓ Repository type references valid

Template is VALID

Warnings (1):
  ⚠ best_practice: Consider adding more tags for better discoverability
```

**Output (Pretty Format - Invalid Template):**

```
Validating template: broken-template

✗ Template has ISSUES

Issues (2):
  ✗ template.toml: Missing required field 'name'
  ✗ variables.service_name: Invalid variable name format

Template validation FAILED
```

**Output (JSON Format):**

```json
{
  "template_name": "rust-library",
  "valid": true,
  "issues": [],
  "warnings": [
    {
      "category": "best_practice",
      "message": "Consider adding more tags for better discoverability"
    }
  ]
}
```

## Workflow Examples

### Discover and Use a Template

```bash
# Step 1: List available templates (not implemented yet - use GitHub UI or API)

# Step 2: Get detailed information about a template
repo-roller template info --org myorg --template rust-service

# Step 3: Validate the template
repo-roller template validate --org myorg --template rust-service

# Step 4: Create a repository using the template
repo-roller create \
  --org myorg \
  --repo my-new-service \
  --template rust-service \
  --description "My new service" \
  --variable project_name="My Service" \
  --variable service_port="8080"
```

### Validate All Variables Before Creation

```bash
# Get template info to see all variables
repo-roller template info --org myorg --template rust-service --format json | jq '.variables'

# Prepare variables based on template requirements
# ... collect variable values ...

# Validate template before creating repository
repo-roller template validate --org myorg --template rust-service

# Create repository with all variables
repo-roller create --org myorg --repo my-service --template rust-service \
  --variable project_name="My Service" \
  --variable service_port="8080" \
  --variable database="postgres"
```

### Troubleshoot Template Issues

```bash
# If repository creation fails due to template issues:

# 1. Validate the template
repo-roller template validate --org myorg --template problematic-template

# 2. Check template configuration details
repo-roller template info --org myorg --template problematic-template --format json

# 3. Review validation output for specific issues
#    - Missing required fields
#    - Invalid variable names
#    - Conflicting configuration
```

## When to Use Template Commands vs REST API

### Use CLI Template Commands When

- **Interactive exploration**: Discovering templates and understanding their structure
- **Pre-validation**: Checking templates before repository creation
- **Troubleshooting**: Diagnosing template configuration issues
- **Manual workflows**: Ad-hoc template inspection during development

### Use REST API Template Endpoints When

- **Automation**: Building scripts or applications that work with templates
- **Integration**: Connecting template discovery to other systems
- **Bulk operations**: Processing multiple templates programmatically
- **CI/CD pipelines**: Automated validation as part of deployment

## Interpreting Validation Results

### Validation Status

- **Valid**: Template configuration is correct and can be used for repository creation
- **Invalid**: Template has errors that must be fixed before use

### Issue Severity

- **✗ Error**: Critical issues that prevent template use
- **⚠ Warning**: Non-critical issues that should be addressed for best practices

### Common Issues

| Issue | Cause | Solution |
|-------|-------|----------|
| Missing required field 'name' | Template metadata incomplete | Add `name` field to `[template]` section |
| Invalid variable name | Variable names contain spaces or special characters | Use alphanumeric and underscore only |
| Required variable with default | Logical conflict in variable definition | Remove `default` or make variable optional |
| Repository type not found | Template references non-existent type | Verify type name or create type configuration |

### Common Warnings

| Warning | Meaning | Action |
|---------|---------|--------|
| No description provided | Template lacks description | Add description for documentation |
| No tags defined | Template not categorized | Add tags for discoverability |
| No variables defined | Template has no customization | Consider if variables would be useful |
| Missing example for required variable | Required variable lacks usage example | Add example to help users |

## Troubleshooting

### "No templates found"

**Possible causes:**

1. Organization name is incorrect
2. No template repositories exist in the organization
3. Templates don't have the `.reporoller/template.toml` configuration file
4. GitHub App not installed in the organization

**Solutions:**

- Verify organization name: `repo-roller org-settings list-types --org <ORGANIZATION>`
- Check GitHub App installation in organization settings
- Ensure template repositories have `.reporoller/template.toml` in the default branch

### "Authentication failed"

**Possible causes:**

1. GitHub App credentials not configured
2. Credentials expired or invalid
3. GitHub App not installed for the organization

**Solutions:**

```bash
# Reconfigure authentication
repo-roller auth setup

# Verify GitHub App has correct permissions:
# - Repository: Read
# - Metadata: Read
# - Contents: Read
```

### "Template not found"

**Possible causes:**

1. Template name is incorrect (typo, wrong case)
2. Template repository exists but lacks `.reporoller/template.toml`
3. GitHub App doesn't have access to the repository

**Solutions:**

- Verify template name matches repository name exactly
- Check repository exists: Visit `https://github.com/<org>/<template>`
- Ensure `.reporoller/template.toml` exists in the template repository
- Verify GitHub App installation includes the template repository

### "Failed to parse template configuration"

**Possible causes:**

1. Invalid TOML syntax in `template.toml`
2. Missing required fields
3. Incorrect field types or values

**Solutions:**

- Validate TOML syntax using online validator
- Review template configuration specification
- Check error message for specific line number and issue
- Compare with working template configurations

## Configuration File Location

Templates are discovered from repositories containing `.reporoller/template.toml` files. The metadata repository (default: `.reporoller-test` in glitchgrove organization for testing) defines which repositories are recognized as templates.

## See Also

- [Repository Creation Guide](../README.md#creating-repositories) - Using templates to create repositories
- [Template Configuration Specification](../../specs/interfaces/cli-template-operations.md) - Template file format
- [Organization Settings](../README.md#organization-settings) - Managing repository types and team configurations
- [Authentication Setup](../README.md#authentication) - Configuring GitHub App credentials
