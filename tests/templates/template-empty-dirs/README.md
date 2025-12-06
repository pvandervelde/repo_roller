# Template: Empty Directories

## Purpose

This template is used for **integration testing** to verify handling of empty directories during template processing.

## Test Coverage

- **Test File**: `crates/integration_tests/tests/template_processing_edge_cases.rs`
- **Test Function**: `test_empty_directory_handling()`
- **Validates**:
  - Empty directories are preserved (via .gitkeep)
  - Directory structure maintained
  - Expected behavior documented

## Template Contents

- `empty-dir-1/.gitkeep` - Empty directory with .gitkeep
- `empty-dir-2/.gitkeep` - Another empty directory
- `nested/empty/.gitkeep` - Nested empty directory
- `README.md` - This file

## Note on Git and Empty Directories

Git does not track empty directories. To preserve directory structure, we use `.gitkeep` files.
This is a common convention in Git repositories.

## Usage

This template is automatically used by the integration test suite when testing empty directory handling.

**Note**: This is a test fixture template and should not be used for creating production repositories.
