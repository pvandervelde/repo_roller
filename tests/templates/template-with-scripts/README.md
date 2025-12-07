# Template: Executable Scripts

## Purpose

This template is used for **integration testing** to verify that file permissions (especially executable bits) are preserved during template processing.

## Test Coverage

- **Test File**: `crates/integration_tests/tests/template_processing_edge_cases.rs`
- **Test Function**: `test_executable_permissions_preserved()`
- **Validates**:
  - Executable bit preservation
  - Script file handling
  - Permission metadata
  - Platform-specific behavior

## Template Contents

- `script.sh` - Executable shell script (chmod +x)
- `script.py` - Executable Python script (chmod +x)
- `regular-file.txt` - Non-executable file for comparison
- `setup-permissions.sh` - Script to set executable bits on Unix
- `README.md` - This file

## Setup

Run the setup script to set executable permissions:

**Unix/Linux/macOS:**

```bash
chmod +x setup-permissions.sh
./setup-permissions.sh
```

## Usage

This template is automatically used by the integration test suite when testing permission preservation.

**Note**: GitHub may not preserve all Unix file permissions. This test documents actual behavior.

**Note**: This is a test fixture template and should not be used for creating production repositories.
