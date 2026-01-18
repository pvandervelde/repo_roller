# Test Repository Creation Scripts

This directory contains scripts to create GitHub test repositories for RepoRoller integration tests.

## Overview

The integration tests require several test repositories on GitHub:

### Test Template Repositories

1. **test-basic** - Basic repository creation testing
2. **test-variables** - Variable substitution testing
3. **test-filtering** - File filtering testing
4. **test-invalid** - Error handling testing

### Test Metadata Repository

- **.reporoller-test** - Organization settings configuration for integration tests

See [README_METADATA.md](README_METADATA.md) for detailed metadata repository documentation.

## Prerequisites

Before running these scripts, ensure you have:

1. **GitHub CLI installed** - Download from [https://cli.github.com/](https://cli.github.com/)
2. **GitHub CLI authenticated** - Run `gh auth login` and follow the prompts
3. **Git installed** - Required for repository operations
4. **Appropriate permissions** - Write access to the target GitHub organization

## Quick Start

### Creating/Updating Test Infrastructure

**Primary Script** (Most common use case):

```powershell
# Update all template repositories in glitchgrove organization
./tests/create-test-repositories.ps1 -Organization "glitchgrove"
```

This script:

- Creates/updates all 16 template repositories
- Syncs content from `tests/templates/` to GitHub
- Marks repositories as templates
- Handles both creation and updates

### Available Scripts

#### 1. Template Repository Management

**[create-test-repositories.ps1](create-test-repositories.ps1)** - Primary script for template management

```powershell
# Update templates in glitchgrove (recommended)
./tests/create-test-repositories.ps1 -Organization "glitchgrove"

# Force update of existing repositories
./tests/create-test-repositories.ps1 -Organization "glitchgrove" -Force

# Use different organization
./tests/create-test-repositories.ps1 -Organization "myorg"
```

#### 2. Metadata Repository Management

**[create-test-metadata-repository.ps1](create-test-metadata-repository.ps1)** - Creates `.reporoller-test` metadata repository

```powershell
# Create .reporoller-test in glitchgrove
./tests/create-test-metadata-repository.ps1

# Force recreation
./tests/create-test-metadata-repository.ps1 -Force

# Different organization
./tests/create-test-metadata-repository.ps1 -Organization "myorg" -Force
```

**[setup-test-metadata-repos.ps1](setup-test-metadata-repos.ps1)** - Creates multiple test metadata repositories for specialized testing

```powershell
# Create all test metadata repos
./tests/setup-test-metadata-repos.ps1 -Organization "glitchgrove"

# Skip existing repositories
./tests/setup-test-metadata-repos.ps1 -SkipExisting
```

Use this for:

- Testing team-specific configurations
- Repository type hierarchies
- Complex metadata scenarios

#### 3. Cleanup Utilities

**[cleanup-test-repos.ps1](cleanup-test-repos.ps1)** - Removes orphaned test repositories

```powershell
# Preview cleanup (dry run)
./tests/cleanup-test-repos.ps1 -MaxAgeHours 24 -DryRun

# Delete repos older than 24 hours
./tests/cleanup-test-repos.ps1 -MaxAgeHours 24

# Quick cleanup without confirmation
./tests/cleanup-test-repos.ps1 -MaxAgeHours 1 -Force
```

**[cleanup-misnamed-repos.ps1](cleanup-misnamed-repos.ps1)** - Removes incorrectly named repositories

```powershell
# Preview what would be deleted
./tests/cleanup-misnamed-repos.ps1 -DryRun

# Delete misnamed repos older than 7 days
./tests/cleanup-misnamed-repos.ps1 -OlderThanDays 7

# Force delete without confirmation
./tests/cleanup-misnamed-repos.ps1 -Force
```

#### 4. Inspection Utilities

**[identify-test-repos.ps1](identify-test-repos.ps1)** - Lists and categorizes repositories

```powershell
# Analyze glitchgrove repositories
./tests/identify-test-repos.ps1

# Check different organization
./tests/identify-test-repos.ps1 -Organization "myorg"
```

Useful for:

- Understanding repository status
- Identifying misnamed repositories
- Planning cleanup operations

#### 5. Feature-Specific Updates

**[update-templates-visibility.ps1](update-templates-visibility.ps1)** - Updates visibility configuration in template repositories

**[update-metadata-repos-visibility.ps1](update-metadata-repos-visibility.ps1)** - Updates visibility policies in metadata repositories

These are specialized scripts for specific feature testing and should only be used when working on visibility-related features.

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

### Test Repository Naming Convention

All test repositories follow a standardized naming pattern:

**Format**: `{prefix}-repo-roller-{context}-{timestamp}-{test-name}-{random}`

- **Prefix**: `test` (integration tests) or `e2e` (end-to-end tests)
- **Context**: Workflow environment
  - `pr{number}` - Created during PR workflow (e.g., `pr123`)
  - `main` - Created from main/master branch
  - `local` - Created during local development
  - `{branch-name}` - Created from feature branch
- **Timestamp**: Unix timestamp for uniqueness
- **Test name**: Descriptive test identifier
- **Random**: Short random suffix

**Examples**:

- `test-repo-roller-pr456-basic-1703250123-a7f3`
- `e2e-repo-roller-main-api-1703250456-b2e9`
- `test-repo-roller-local-auth-1703250789-c4d1`

### Updating Templates

1. Modify files in `tests/templates/{template-name}/`
2. Run the creation script with `-Force` or `true` parameter
3. Verify integration tests still pass

### Cleanup

#### Automated Cleanup (Two-Tier System)

RepoRoller uses a two-tier cleanup system for test repositories:

##### 1. PR-Based Cleanup (Primary)

Runs automatically when a pull request is closed or merged:

- **Trigger**: `.github/workflows/cleanup-pr-repos.yml`
- **Scope**: Only repositories created by that specific PR (matching `-pr{number}-` pattern)
- **Timing**: Immediate (runs within minutes of PR closure)
- **Reporting**: Posts comment to the closed PR with cleanup results
- **Purpose**: Fast cleanup of PR-specific test artifacts

**What happens**:

1. PR closes (merged or not)
2. Workflow identifies all test repos containing `-pr{number}-` in the name
3. Deletes all matching repositories
4. Comments on PR: "✅ Cleaned up X test repositories"

##### 2. Scheduled Cleanup (Safety Net)

Runs daily as a catch-all for any repositories missed by PR cleanup:

- **Schedule**: Daily at 2 AM UTC (`.github/workflows/cleanup-test-repos.yml`)
- **Scope**: All test repositories older than 24 hours
- **Patterns**: Catches both `test-repo-roller-*` and `e2e-repo-roller-*`
- **Alerting**: Creates an issue if more than 10 orphaned repos are found
- **Purpose**: Safety net for failed PR cleanups, crashed tests, or local development repos

**Manual trigger**:

- Go to Actions → "Cleanup Test Repositories" → "Run workflow"
- Configure max age in hours (default: 24)

**What it catches**:

- Repositories from PRs where cleanup failed
- Local development test repositories
- Test runs that crashed before cleanup
- Repositories from deleted branches

#### Manual Cleanup

Use the cleanup utilities for immediate cleanup:

**Rust cleanup utility** (recommended):

```bash
# Clean up all test repos older than 24 hours
cargo run --package test_cleanup --bin cleanup-orphans -- 24

# Clean up repos from specific PR
cargo run --package test_cleanup --bin cleanup-pr -- 456

# Quick cleanup (1 hour threshold)
cargo run --package test_cleanup --bin cleanup-orphans -- 1
```

**PowerShell script**:

```powershell
# Preview what will be deleted (dry run)
./tests/cleanup-test-repos.ps1 -MaxAgeHours 24 -DryRun

# Delete test repositories older than 24 hours
./tests/cleanup-test-repos.ps1 -MaxAgeHours 24

# Force delete without confirmation
./tests/cleanup-test-repos.ps1 -MaxAgeHours 24 -Force

# Delete very old repos (1 week)
./tests/cleanup-test-repos.ps1 -MaxAgeHours 168 -Force
```

**GitHub CLI** (for template repositories):

```bash
# Delete template repositories
gh repo delete pvandervelde/test-basic --yes
gh repo delete pvandervelde/test-variables --yes
gh repo delete pvandervelde/test-filtering --yes
gh repo delete pvandervelde/test-invalid --yes
```

#### Monitoring and Troubleshooting

**View Cleanup Status**:

- PR cleanups: Check PR comments for "🧹 Test Repository Cleanup" messages
- Scheduled cleanup: Actions → "Cleanup Test Repositories" → Latest run
- Manual runs: Can be triggered anytime from Actions tab

**High Orphan Count Alert**:

If the scheduled cleanup finds more than 10 orphaned repositories, it automatically creates an issue with:

- Number of repositories cleaned
- Possible causes (PR cleanup failures, test crashes)
- Investigation steps
- Link to the cleanup run

**Common Issues**:

1. **PR cleanup didn't run**
   - Check if PR had any CI runs
   - Verify secrets are configured (INTEGRATION_TEST_GITHUB_APP_ID, etc.)
   - Scheduled cleanup will catch these within 24 hours

2. **Too many orphaned repos**
   - Review recent PR cleanup workflow failures
   - Check test logs for crashes before cleanup
   - Verify repository naming follows convention

3. **Local test repos accumulating**
   - Run manual cleanup: `cargo run --package test_cleanup --bin cleanup-orphans -- 0`
   - Or use PowerShell script with short max age

**Best Practices**:

- Monitor the daily cleanup summaries for trends
- Investigate if cleanup consistently finds many repos
- Ensure PR workflows complete successfully
- Use appropriate test contexts (PR, local, etc.)

## Security Considerations

- These are public repositories - don't include sensitive data
- Template repositories are visible to all GitHub users
- Consider using a dedicated test organization for isolation
- Regularly review and update templates for security best practices
- Test repository credentials are stored as GitHub Secrets
- Cleanup uses GitHub App authentication for secure access
