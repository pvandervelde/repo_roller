# Test Filtering Repository

This repository is used to test file filtering capabilities in RepoRoller.

## Structure

This repository contains various file types and directories to test filtering:

- **Source code files** (`src/*.rs`) - ✓ Should be included
- **Configuration files** (`*.toml`, `*.json`) - ✓ Should be included
- **Documentation** (`*.md`, `docs/`) - ✓ Should be included
- **Build artifacts** (`target/`) - ✗ Should be excluded (if present)
- **Log files** (`*.log`) - ✗ Should be excluded (if present)
- **Temporary files** (`tmp/`) - ✗ Should be excluded (if present)

## Filtering Configuration

The `.reporoller/template.toml` file contains a `[templating]` section that demonstrates file filtering:

```toml
[templating]
# Include source files, documentation, and configuration
include_patterns = [
    "**/*.rs",           # All Rust source files
    "**/*.toml",         # All TOML configuration files
    "**/*.md",           # All Markdown documentation
    "**/*.json",         # All JSON files
]

# Exclude build artifacts and temporary files
exclude_patterns = [
    "target/**",         # Rust build directory
    "**/*.log",          # Log files
    "**/tmp/**",         # Temporary directories
    "**/.DS_Store",      # macOS metadata
]
```

## Testing Behavior

Integration tests use this template to verify:

1. **Include patterns work** - Only specified file types are processed
2. **Exclude patterns work** - Specified patterns are excluded
3. **Exclude precedence** - Exclude patterns override include patterns
4. **Empty include behavior** - If no includes specified, all files included (except excluded)
5. **`.reporoller/` exclusion** - The `.reporoller/` directory is always excluded

## Usage

This template is used by `crates/integration_tests/tests/template_file_filtering_tests.rs` to verify file filtering functionality.
