# Template: Deep Nesting

## Purpose

This template is used for **integration testing** to verify that deeply nested directory structures (>10 levels) are handled correctly during template processing.

## Test Coverage

- **Test File**: `crates/integration_tests/tests/template_processing_edge_cases.rs`
- **Test Function**: `test_deep_directory_nesting()`
- **Validates**:
  - Directory structures >10 levels deep
  - File creation at various nesting levels
  - Path length handling
  - Correct directory hierarchy preservation

## Template Structure

```
level1/
  level2/
    level3/
      level4/
        level5/
          level6/
            level7/
              level8/
                level9/
                  level10/
                    level11/
                      level12/
                        deep-file.txt
```

## Usage

This template is automatically used by the integration test suite when testing deep directory nesting.

**Note**: This is a test fixture template and should not be used for creating production repositories.
