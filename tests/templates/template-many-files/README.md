# Template: Many Files

## Purpose

This template is used for **integration testing** to verify performance and correctness when processing templates with a large number of files (>1000).

## Test Coverage

- **Test File**: `crates/integration_tests/tests/template_processing_edge_cases.rs`
- **Test Function**: `test_many_files_template()`
- **Validates**:
  - Processing >1000 files without errors
  - Performance within acceptable limits (<5 minutes)
  - No files lost or skipped
  - Correct file count in created repository

## Template Contents

- `README.md` - This file
- `generate-files.ps1` - Script to generate test files
- `files/` - Directory containing generated test files

## Generating Test Files

Run the PowerShell script to generate 1000+ test files:

```powershell
.\generate-files.ps1
```

## Usage

This template is automatically used by the integration test suite when testing many files handling.

**Note**: This is a test fixture template and should not be used for creating production repositories.
