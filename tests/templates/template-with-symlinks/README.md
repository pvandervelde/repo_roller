# Template: Symbolic Links

## Purpose

This template is used for **integration testing** to verify how symbolic links are handled during template processing.

## Test Coverage

- **Test File**: `crates/integration_tests/tests/template_processing_edge_cases.rs`
- **Test Function**: `test_symlink_handling()`
- **Validates**:
  - Symlink detection
  - Expected behavior (skip, resolve, or error)
  - No broken links in created repository
  - Proper handling of symlink cycles

## Template Contents

- `real-file.txt` - Actual file
- `link-to-file.txt` - Symlink to real-file.txt (created by setup script)
- `link-to-dir/` - Symlink to a directory (created by setup script)
- `setup-symlinks.sh` - Script to create symlinks on Unix systems
- `setup-symlinks.ps1` - Script to create symlinks on Windows
- `README.md` - This file

## Setup

Run the appropriate script for your platform:

**Unix/Linux/macOS:**

```bash
./setup-symlinks.sh
```

**Windows (requires admin privileges):**

```powershell
.\setup-symlinks.ps1
```

## Usage

This template is automatically used by the integration test suite when testing symlink handling.

**Note**: This is a test fixture template and should not be used for creating production repositories.
