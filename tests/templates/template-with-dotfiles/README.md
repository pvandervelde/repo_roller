# Template: Dotfiles (Hidden Files)

## Purpose

This template is used for **integration testing** to verify that hidden files (starting with `.`) are processed correctly during template processing.

## Test Coverage

- **Test File**: `crates/integration_tests/tests/template_processing_edge_cases.rs`
- **Test Function**: `test_hidden_files_processing()`
- **Validates**:
  - Hidden files are processed
  - .gitignore is preserved
  - .env.example and similar config files work
  - .github/ directory structure preserved

## Template Contents

- `.gitignore` - Standard gitignore file
- `.env.example` - Example environment variables
- `.editorconfig` - Editor configuration
- `.github/workflows/test.yml` - GitHub Actions workflow
- `README.md` - This file
- `visible-file.txt` - Regular file for comparison

## Usage

This template is automatically used by the integration test suite when testing hidden file processing.

**Note**: This is a test fixture template and should not be used for creating production repositories.
