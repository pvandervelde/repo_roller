# Template: Files Without Extensions

## Purpose

This template is used for **integration testing** to verify that files without file extensions are processed correctly.

## Test Coverage

- **Test File**: `crates/integration_tests/tests/template_processing_edge_cases.rs`
- **Test Function**: `test_files_without_extensions()`
- **Validates**:
  - Extensionless files are processed
  - Variable substitution works in extensionless files
  - Common extensionless files (Dockerfile, Makefile, LICENSE) handled correctly

## Template Contents

- `Dockerfile` - Docker container definition
- `Makefile` - Build automation file
- `LICENSE` - License file
- `CHANGELOG` - Change log
- `AUTHORS` - Contributors list
- `README.md` - This file

## Usage

This template is automatically used by the integration test suite when testing extensionless file processing.

**Note**: This is a test fixture template and should not be used for creating production repositories.
