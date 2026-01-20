# Template File Filtering Guide

**Audience**: Template authors
**Related**: [Template Configuration Reference](template-configuration.md)

---

## Overview

Template file filtering allows you to control which files from your template repository are included when creating new repositories. This is useful for excluding build artifacts, test data, or files that should only exist in the template repository itself.

## Configuration Location

All file filtering is configured in your template's `.reporoller/template.toml` file using the `[templating]` section:

```toml
[templating]
include_patterns = ["**/*.rs", "**/*.toml", "**/*.md"]
exclude_patterns = ["target/**", "**/*.log"]
```

## Pattern Syntax

Patterns use standard **glob syntax**:

- `*` - Matches any characters except path separators (`/`)
- `?` - Matches exactly one character
- `**` - Matches any number of directories (recursive)
- `[abc]` - Matches one character from the set
- `[a-z]` - Matches one character from the range

### Pattern Examples

| Pattern | Matches | Description |
|---------|---------|-------------|
| `*.rs` | `main.rs`, `lib.rs` | Rust files in root directory only |
| `**/*.rs` | `src/main.rs`, `tests/integration.rs` | All Rust files recursively |
| `src/**/*.rs` | `src/lib.rs`, `src/utils/helper.rs` | Rust files under `src/` only |
| `**/*_test.rs` | `utils_test.rs`, `src/lib_test.rs` | Test files with `_test.rs` suffix |
| `*.{rs,toml}` | `main.rs`, `Cargo.toml` | Files with multiple extensions |
| `docs/*.md` | `docs/guide.md` | Markdown files in `docs/` directory only |
| `target/**` | `target/debug/app`, `target/release/` | Everything under `target/` |

## Filtering Rules

### 1. Default Behavior (No `[templating]` section)

If you don't specify a `[templating]` section, **all files are included** except:

- `.reporoller/` directory (always excluded)
- `.git/` directory (never copied from templates)

### 2. Include Patterns

If you specify `include_patterns`:

- **Empty list** (`[]`): Include all files
- **Non-empty list**: Only files matching at least one pattern are included

### 3. Exclude Patterns

Exclude patterns **always take precedence** over include patterns:

- Files matching any exclude pattern are removed, even if they match an include pattern

### 4. Pattern Application Order

```
1. Start with all files in template repository
2. Apply include patterns (if specified)
3. Apply exclude patterns (always removes matches)
4. Remove .reporoller/ directory (always excluded)
5. Result: Files to process
```

## Common Use Cases

### Rust Projects

Exclude build artifacts and IDE files:

```toml
[templating]
include_patterns = [
    "**/*.rs",
    "**/*.toml",
    "**/*.md",
    ".gitignore",
    "LICENSE",
]
exclude_patterns = [
    "target/**",           # Cargo build directory
    "**/*.log",            # Log files
    "**/.DS_Store",        # macOS metadata
    "**/.idea/**",         # IntelliJ IDEA
    "**/.vscode/**",       # VS Code (if not wanted)
]
```

### TypeScript/Node.js Projects

Exclude node_modules and build outputs:

```toml
[templating]
include_patterns = [
    "**/*.ts",
    "**/*.js",
    "**/*.json",
    "**/*.md",
    "**/*.yml",
]
exclude_patterns = [
    "node_modules/**",     # Dependencies
    "dist/**",             # Build output
    "build/**",            # Build output
    "**/*.log",            # Log files
    "**/.next/**",         # Next.js cache
]
```

### Python Projects

Exclude virtual environments and bytecode:

```toml
[templating]
include_patterns = [
    "**/*.py",
    "**/*.txt",            # requirements.txt, etc.
    "**/*.md",
    "**/*.yml",
    "pyproject.toml",
]
exclude_patterns = [
    "venv/**",             # Virtual environment
    ".venv/**",            # Virtual environment
    "**/__pycache__/**",   # Python bytecode
    "**/*.pyc",            # Compiled Python
    "**/*.pyo",            # Optimized Python
    "**/*.egg-info/**",    # Package metadata
    ".pytest_cache/**",    # Pytest cache
]
```

### Documentation-Only Templates

Include only documentation files:

```toml
[templating]
include_patterns = [
    "**/*.md",
    "**/*.rst",
    "docs/**",
    "mkdocs.yml",
    "README.md",
]
exclude_patterns = [
    "**/.*",               # Hidden files
]
```

### Liberal Template (Include Almost Everything)

Use when you want most files but need to exclude specific directories:

```toml
[templating]
# Empty include means "include everything"
include_patterns = []

# Only exclude what you don't want
exclude_patterns = [
    "target/**",
    "node_modules/**",
    "**/*.log",
    "**/tmp/**",
    "**/.DS_Store",
]
```

## Organizing Long Pattern Lists

For templates with many patterns, use TOML's array syntax for readability:

```toml
[templating]

# Group related patterns with comments
include_patterns = [
    # Source code
    "**/*.rs",
    "**/*.toml",

    # Documentation
    "**/*.md",
    "docs/**",

    # Configuration
    "**/*.yml",
    "**/*.json",

    # Special files
    ".gitignore",
    "LICENSE",
    "README.md",
]

exclude_patterns = [
    # Build artifacts
    "target/**",
    "dist/**",
    "build/**",

    # Dependencies
    "node_modules/**",
    "vendor/**",

    # IDE and OS files
    "**/.idea/**",
    "**/.vscode/**",
    "**/.DS_Store",

    # Temporary files
    "**/*.log",
    "**/*.tmp",
    "**/tmp/**",
]
```

## Best Practices

### ✅ Do

1. **Be explicit** - Clearly specify what you want included/excluded
2. **Use comments** - Document why patterns are needed
3. **Test your patterns** - Create test repositories to verify filtering works
4. **Start conservative** - Begin with restrictive includes, relax as needed
5. **Document in README** - Explain filtering behavior to template users

### ❌ Don't

1. **Don't over-filter** - Avoid excluding files users might need
2. **Don't use platform-specific patterns** - Stick to cross-platform globs
3. **Don't exclude `.github/`** - Users may want workflow files from templates
4. **Don't rely on exclude-only** - Combine includes and excludes for clarity
5. **Don't forget `.reporoller/`** - It's always excluded automatically

## Testing Your Filters

### Local Testing

1. Create a test repository using your template:

   ```bash
   repo-roller create my-org test-repo \
     --template my-template \
     --repository-type service
   ```

2. Verify the created repository contains only expected files

3. Check GitHub repository file list to confirm filtering

### Integration Tests

Create integration tests that verify your filtering configuration:

```rust
#[tokio::test]
async fn test_my_template_filtering() -> Result<()> {
    let repo = create_repository_from_template(
        "my-org",
        "test-repo",
        "my-template",
    ).await?;

    // Verify expected files exist
    assert_file_exists(&repo, "src/main.rs");
    assert_file_exists(&repo, "Cargo.toml");

    // Verify excluded files don't exist
    assert_file_not_exists(&repo, "target/debug/app");
    assert_file_not_exists(&repo, "test.log");

    Ok(())
}
```

## Troubleshooting

### "No files were processed after filtering"

**Cause**: Your patterns excluded all files.

**Solution**:

- Check if include patterns are too restrictive
- Verify exclude patterns aren't too broad
- Use empty `include_patterns = []` to include everything first
- Test patterns with `glob` tool or online glob testers

### "Files I wanted are missing"

**Cause**: Include patterns too restrictive or exclude patterns too broad.

**Solution**:

- Add missing file patterns to `include_patterns`
- Remove or narrow down exclude patterns
- Check pattern syntax (missing `**` for recursive matching)

### "Unwanted files are included"

**Cause**: Missing or incorrect exclude patterns.

**Solution**:

- Add specific exclude patterns for unwanted files
- Use `**/*.ext` for all files with extension
- Use `dir/**` to exclude entire directories

### "Pattern not matching expected files"

**Cause**: Incorrect glob syntax or path structure.

**Solution**:

- Remember `*` doesn't cross directory boundaries
- Use `**` for recursive matching
- Test with simpler patterns first
- Check file paths are relative to repository root

## Examples from Real Templates

### Microservice Template

```toml
[templating]
include_patterns = [
    "src/**/*.rs",           # Application source
    "Cargo.toml",
    "Dockerfile",
    ".dockerignore",
    "k8s/**/*.yml",          # Kubernetes manifests
    "**/*.md",               # Documentation
]
exclude_patterns = [
    "target/**",
    "**/*.log",
    "**/.DS_Store",
]
```

### Library Template

```toml
[templating]
include_patterns = [
    "src/**/*.rs",
    "tests/**/*.rs",
    "examples/**/*.rs",
    "benches/**/*.rs",
    "Cargo.toml",
    "README.md",
    "LICENSE",
    ".gitignore",
]
exclude_patterns = [
    "target/**",
    "Cargo.lock",            # Lock file not needed for libraries
]
```

### Full-Stack Application Template

```toml
[templating]
include_patterns = [
    "backend/**/*.rs",       # Rust backend
    "frontend/**/*.ts",      # TypeScript frontend
    "frontend/**/*.tsx",
    "frontend/**/*.json",
    "shared/**",             # Shared types
    "**/*.toml",
    "**/*.json",
    "**/*.yml",
    "**/*.md",
]
exclude_patterns = [
    "target/**",             # Rust build
    "node_modules/**",       # Node dependencies
    "frontend/dist/**",      # Frontend build
    "**/*.log",
]
```

## Related Documentation

- [Template Configuration Reference](template-configuration.md) - Full TOML format reference
- [Template Variables Guide](template-variables.md) - Variable substitution
- [Creating Templates](creating-templates.md) - Complete template authoring guide
- [Testing Templates](testing-templates.md) - How to test your templates

## See Also

- [Glob Pattern Syntax](https://en.wikipedia.org/wiki/Glob_(programming))
- [Template File Filtering Interface Spec](../specs/interfaces/template-file-filtering.md)
- [Integration Test Templates](../tests/templates/) - Real examples
