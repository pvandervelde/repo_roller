# Template: Nested Variables

## Purpose

This template is used for **integration testing** to verify that nested variable substitution works correctly.

## Test Coverage

- **Test File**: `crates/integration_tests/tests/variable_substitution_edge_cases.rs`
- **Test Function**: `test_nested_variable_substitution()`
- **Validates**:
  - Variables can reference other variables
  - Nested substitution resolves correctly
  - Multiple levels of nesting supported
  - No infinite recursion

## Template Contents

- `config.toml` - Configuration with nested variable definitions
- `README.md` - This file
- `greeting.txt` - File using nested variables

## Variable Structure

The template defines variables that reference each other:

```toml
[variables]
first_name = "John"
last_name = "Doe"
full_name = "{{first_name}} {{last_name}}"
greeting = "Hello, {{full_name}}!"
```

When a user provides `first_name = "Alice"` and `last_name = "Smith"`,
the final greeting should be: "Hello, Alice Smith!"

## Usage

This template is automatically used by the integration test suite when testing nested variable substitution.

**Note**: This is a test fixture template and should not be used for creating production repositories.
