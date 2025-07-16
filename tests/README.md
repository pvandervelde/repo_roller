# Test Repository Creation Scripts

This directory contains scripts to create GitHub test template repositories for RepoRoller integration tests.

## Overview

The integration tests require four test template repositories to be available on GitHub:

1. **test-basic** - Basic repository creation testing
2. **test-variables** - Variable substitution testing
3. **test-filtering** - File filtering testing
4. **test-invalid** - Error handling testing

## Prerequisites

Before running these scripts, ensure you have:

1. **GitHub CLI installed** - Download from [https://cli.github.com/](https://cli.github.com/)
2. **GitHub CLI authenticated** - Run `gh auth login` and follow the prompts
3. **Git installed** - Required for repository operations
4. **Appropriate permissions** - Write access to the target GitHub organization

## Scripts

### PowerShell Script (Windows)

```powershell
# Basic usage (creates repos in 'pvandervelde' organization)
./scripts/create-test-repositories.ps1

# Specify different organization
./scripts/create-test-repositories.ps1 -Organization "myorg"

# Force recreation of existing repositories
./scripts/create-test-repositories.ps1 -Force

# Both options
./scripts/create-test-repositories.ps1 -Organization "myorg" -Force
```

### Bash Script (Linux/macOS)

```bash
# Make script executable (Linux/macOS only)
chmod +x scripts/create-test-repositories.sh

# Basic usage (creates repos in 'pvandervelde' organization)
./scripts/create-test-repositories.sh

# Specify different organization
./scripts/create-test-repositories.sh myorg

# Force recreation of existing repositories
./scripts/create-test-repositories.sh pvandervelde true

# Different organization with force
./scripts/create-test-repositories.sh myorg true
```

## What the Scripts Do

1. **Validate Prerequisites**
   - Check GitHub CLI installation and authentication
   - Verify template directories exist and contain files

2. **Process Each Template**
   - Check if repository already exists
   - Optionally remove existing repository (with `-Force` or `true` parameter)
   - Create temporary directory with template files
   - Initialize git repository and make initial commit
   - Create GitHub repository with template flag enabled
   - Push template files to GitHub

3. **Output Results**
   - Display success/failure status for each repository
   - Provide links to created repositories

## Template Structure

The templates are stored locally in `tests/templates/`:

```
tests/templates/
├── test-basic/          # Basic Rust project template
│   ├── README.md
│   ├── .gitignore
│   ├── Cargo.toml
│   └── src/main.rs
├── test-variables/      # Template with variable substitution
│   ├── README.md
│   ├── .gitignore
│   ├── Cargo.toml
│   ├── config.yml
│   └── src/main.rs
├── test-filtering/      # Template with diverse file types
│   ├── README.md
│   ├── .gitignore
│   ├── Cargo.toml
│   ├── src/main.rs
│   ├── docs/guide.md
│   ├── config/settings.json
│   ├── target/debug/main    # Should be filtered out
│   ├── temp/cache.tmp       # Should be filtered out
│   └── logs/application.log # Should be filtered out
└── test-invalid/        # Template with intentional errors
    ├── README.md
    ├── Cargo.toml       # Malformed TOML
    ├── config.json      # Malformed JSON
    └── src/main.rs      # Invalid Rust code
```

## Repository Configuration

Created repositories will be:

- **Public** - Required for template functionality
- **Template enabled** - Allows use as GitHub template
- **Properly tagged** - With descriptive names and descriptions

## Troubleshooting

### GitHub CLI Issues

```bash
# Check if GitHub CLI is installed
gh --version

# Check authentication status
gh auth status

# Re-authenticate if needed
gh auth login
```

### Permission Issues

- Ensure you have write access to the target organization
- For personal repositories, use your GitHub username as organization
- Organization owners may need to grant template creation permissions

### Template Directory Issues

- Verify you're running the script from the repository root
- Check that `tests/templates/` directory exists
- Ensure template directories contain the expected files

## Integration with Tests

Once created, these repositories will be used by:

- `crates/integration_tests/` - Main integration test suite
- `.github/workflows/integration-tests.yml` - CI/CD pipeline
- Local development testing

The integration tests expect these repositories to exist at:

- `https://github.com/{organization}/test-basic`
- `https://github.com/{organization}/test-variables`
- `https://github.com/{organization}/test-filtering`
- `https://github.com/{organization}/test-invalid`

## Maintenance

### Updating Templates

1. Modify files in `tests/templates/{template-name}/`
2. Run the creation script with `-Force` or `true` parameter
3. Verify integration tests still pass

### Cleanup

To remove test repositories:

```bash
# Using GitHub CLI
gh repo delete pvandervelde/test-basic --yes
gh repo delete pvandervelde/test-variables --yes
gh repo delete pvandervelde/test-filtering --yes
gh repo delete pvandervelde/test-invalid --yes
```

## Security Considerations

- These are public repositories - don't include sensitive data
- Template repositories are visible to all GitHub users
- Consider using a dedicated test organization for isolation
- Regularly review and update templates for security best practices
