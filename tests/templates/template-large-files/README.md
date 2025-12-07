# Template: Large Files

## Purpose

This template is used for **integration testing** to verify that the template processing engine can handle large files (>10MB) without memory issues or timeouts.

## Test Coverage

- **Test File**: `crates/integration_tests/tests/template_processing_edge_cases.rs`
- **Test Function**: `test_large_file_processing()`
- **Validates**:
  - Large file transfer without corruption
  - Memory handling for >10MB files
  - Processing performance
  - Correct file size preservation

## Template Contents

- `large-data.json` - 12MB JSON file with test data
- `README.md` - This file
- `config.txt` - Normal sized file for comparison

## Usage

This template is automatically used by the integration test suite when testing large file handling.

**Note**: This is a test fixture template and should not be used for creating production repositories.
