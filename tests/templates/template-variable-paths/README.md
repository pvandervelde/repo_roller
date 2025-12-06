# Template: Variable Paths

## Purpose

This template is used for **integration testing** to verify that variables can be used in file and directory names.

## Test Coverage

- **Test File**: `crates/integration_tests/tests/variable_substitution_edge_cases.rs`
- **Test Function**: `test_variable_substitution_in_filenames()`
- **Validates**:
  - Variables work in file names
  - Variables work in directory names
  - Path construction with variables
  - Special character handling in path substitution

## Template Structure

```
{{project_name}}/
  {{project_name}}_config.json
  src/
    {{project_name}}_main.rs
  tests/
    test_{{project_name}}.rs
README.md
```

## Variable Definition

The template expects a `project_name` variable to be provided by the user.

Example: If `project_name = "MyProject"`, the structure becomes:

```
MyProject/
  MyProject_config.json
  src/
    MyProject_main.rs
  tests/
    test_MyProject.rs
README.md
```

## Usage

This template is automatically used by the integration test suite when testing variable substitution in paths.

**Note**: This is a test fixture template and should not be used for creating production repositories.
