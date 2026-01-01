# Release Automation Interface Specification

**Architectural Layer**: Infrastructure (CI/CD Automation)
**Module Path**: `.github/workflows/release-*.yml`
**Responsibilities**:

- Knows: Version numbers, changelog format, semver rules, release artifacts
- Does: Tracks changes, generates changelogs, creates releases, publishes artifacts

## Overview

This specification defines the automated release management system for RepoRoller. The system automatically manages versioning, changelog generation, and artifact publication using GitHub Actions workflows.

## Dependencies

- **GitHub Actions**: Workflow automation platform
- **Git History**: Commit messages following conventional commits
- **GitHub Releases API**: Release creation and asset management
- **GitHub Container Registry**: Container image publishing
- **GitHub Release Assets**: Binary distribution

## Architecture

### Release Workflow Components

```
Merge to Master
    ↓
Release PR Creation/Update
    ↓
Version Calculation (semver)
    ↓
Changelog Generation
    ↓
[Manual Review & Approval]
    ↓
Release PR Merge
    ↓
GitHub Release Creation
    ↓
Artifact Publishing
    - Container Image (GHCR)
    - CLI Binaries (GitHub Releases)
```

### Versioning Strategy

**Semantic Versioning (semver)**:

- **MAJOR**: Breaking changes (incompatible API changes)
- **MINOR**: New features (backward-compatible additions)
- **PATCH**: Bug fixes (backward-compatible fixes)

**Version Calculation from Conventional Commits**:

- `feat:` → MINOR version bump
- `fix:` → PATCH version bump
- `BREAKING CHANGE:` or `!` → MAJOR version bump
- Other types (`docs:`, `chore:`, etc.) → No version bump (accumulate for next release)

## Types and Configuration

### Release Configuration

```toml
# .github/release-config.toml (optional future enhancement)

[versioning]
# Version calculation strategy
strategy = "conventional-commits"  # or "manual"
initial_version = "0.1.0"

[changelog]
# Changelog generation settings
exclude_types = ["chore", "ci", "test"]
group_by_type = true
show_commit_links = true

[artifacts]
# Artifacts to publish
container_image = true
cli_binaries = true
binary_targets = ["x86_64-unknown-linux-gnu", "x86_64-pc-windows-msvc", "x86_64-apple-darwin"]

[pull_request]
# Release PR settings
title_template = "chore(release): prepare release v{{version}}"
branch_name = "release-pr"
labels = ["release", "automated"]
```

## Workflows

### 1. Release PR Management Workflow

**File**: `.github/workflows/release-pr.yml`

**Trigger**: Push to `master` branch

**Purpose**: Create or update release PR with changelog and version bumps

**Steps**:

1. **Checkout repository** with full history
2. **Calculate next version** from conventional commits since last release
3. **Generate changelog** from commit messages
4. **Update version numbers** in:
   - `Cargo.toml` (workspace version)
   - All crate `Cargo.toml` files
   - Any version constants in code
5. **Create or update release PR**:
   - Branch: `release-pr` (force push to update)
   - Title: `chore(release): prepare release v{version}`
   - Body: Generated changelog
   - Labels: `release`, `automated`
6. **Handle existing release PR**:
   - If exists: Update with new version and accumulated changes
   - If doesn't exist: Create new PR

**Version Override Support**:

Maintainers can comment on release PR to override version:

- `/release major` → Force MAJOR version bump
- `/release minor` → Force MINOR version bump
- `/release patch` → Force PATCH version bump

Comment triggers workflow rerun with override applied.

- Version bumps must still follow semver rules; e.g., cannot downgrade version.
- Bumping a version only for larger versions is allowed (e.g., from PATCH to MINOR). Bumping a smaller
  version is not allowed (e.g., from MINOR to PATCH). Additionally bumping a version to the same level is not allowed
  (e.g., from MINOR to MINOR).
- For invalid overrides, the workflow should comment with an error message explaining the issue.

### 2. Release Publication Workflow

**File**: `.github/workflows/release-publish.yml`

**Trigger**: Release PR merged to `master`

**Purpose**: Create GitHub release and publish artifacts

**Steps**:

1. **Checkout repository**
2. **Extract version** from merged PR or `Cargo.toml`
3. **Create GitHub Release**:
   - Tag: `v{version}`
   - Title: `Release v{version}`
   - Body: Changelog from release PR
   - Draft: false
   - Prerelease: Detect from version (e.g., `0.x.x` or `-alpha`, `-beta`)
4. **Build and publish container image**:
   - Build `repo_roller_api` Docker image
   - Tag: `ghcr.io/pvandervelde/repo_roller_api:v{version}`
   - Tag: `ghcr.io/pvandervelde/repo_roller_api:latest` (if not prerelease)
   - Push to GitHub Container Registry
5. **Build and publish CLI binaries**:
   - Build for Linux (x86_64-unknown-linux-gnu)
   - Build for Windows (x86_64-pc-windows-msvc)
   - Build for macOS (x86_64-apple-darwin)
   - Create archives (tar.gz for Unix, zip for Windows)
   - Upload as release assets
6. **Update release notes**:
   - Add download links for binaries
   - Add container image pull command
   - Add checksums for verification

### 3. Version Comment Handler Workflow

**File**: `.github/workflows/release-comment.yml`

**Trigger**: Issue comment on release PR matching `/release (major|minor|patch)`

**Purpose**: Handle maintainer version override requests

**Steps**:

1. **Validate comment**:
   - Check PR is release PR (label: `release`)
   - Check commenter has write permissions
   - Extract version override command
2. **Trigger release PR workflow** with override parameter
3. **Add reaction** to comment (👍 for success, 👎 for failure)
4. **Add comment** with result (new version, what was changed)

## Conventional Commits Format

### Commit Message Structure

```
<type>(<scope>): <description>

[optional body]

[optional footer with BREAKING CHANGE]
```

### Supported Types

**Version-Affecting**:

- `feat`: New feature (MINOR bump)
- `fix`: Bug fix (PATCH bump)
- Breaking change marker (MAJOR bump):
  - `BREAKING CHANGE:` in footer, or
  - `!` after type/scope: `feat!:` or `feat(api)!:`

**Non-Version-Affecting** (accumulate for changelog):

- `docs`: Documentation changes
- `style`: Code style/formatting
- `refactor`: Code refactoring without feature/fix
- `perf`: Performance improvements
- `test`: Test additions/changes
- `build`: Build system changes
- `ci`: CI/CD changes
- `chore`: Maintenance tasks

### Examples

**PATCH release** (0.1.0 → 0.1.1):

```
fix(cli): handle missing configuration file gracefully

Previously crashed with panic, now returns proper error message.

Fixes #123
```

**MINOR release** (0.1.0 → 0.2.0):

```
feat(api): add repository visibility endpoint

Add GET /api/repositories/{org}/{repo}/visibility endpoint
to check repository visibility status and policies.
```

**MAJOR release** (0.1.0 → 1.0.0):

```
feat(config)!: change configuration hierarchy structure

BREAKING CHANGE: Configuration file structure has changed.
Migration required for existing configurations.

- Rename 'settings' section to 'repository'
- Move 'overrides' to top level
- Add 'metadata' section for version tracking

See MIGRATION.md for upgrade instructions.
```

## Changelog Format

### Generated Changelog Structure

```markdown
# Changelog

## [Version Number] - YYYY-MM-DD

### Breaking Changes

- Description of breaking change with commit link

### Features

- New feature description with commit link
- Another feature with commit link

### Bug Fixes

- Bug fix description with commit link
- Another fix with commit link

### Documentation

- Documentation changes (if significant)

### Other Changes

- Refactoring, style, test changes (collapsed or omitted)

**Full Changelog**: https://github.com/pvandervelde/repo_roller/compare/v0.1.0...v0.2.0
```

## Artifact Publishing

### Container Image

**Repository**: `ghcr.io/pvandervelde/repo_roller_api`

**Tags**:

- `v{version}` - Specific version (e.g., `v0.2.0`)
- `v{major}.{minor}` - Minor version (e.g., `v0.2`)
- `v{major}` - Major version (e.g., `v0`)
- `latest` - Latest stable release (not applied to prereleases)

**Build Process**:

1. Use multi-stage Dockerfile from `crates/repo_roller_api/Dockerfile`
2. Build with release optimizations
3. Tag with version and latest (if stable)
4. Push to GHCR with GitHub token authentication

**Pull Command**:

```bash
docker pull ghcr.io/pvandervelde/repo_roller_api:v0.2.0
# or
docker pull ghcr.io/pvandervelde/repo_roller_api:latest
```

### CLI Binaries

**Targets**:

- **Linux**: `x86_64-unknown-linux-gnu`
  - Archive: `repo-roller-v{version}-x86_64-linux.tar.gz`
  - Binary: `repo-roller`
- **Windows**: `x86_64-pc-windows-msvc`
  - Archive: `repo-roller-v{version}-x86_64-windows.zip`
  - Binary: `repo-roller.exe`
- **macOS**: `x86_64-apple-darwin`
  - Archive: `repo-roller-v{version}-x86_64-macos.tar.gz`
  - Binary: `repo-roller`

**Build Process**:

1. Cross-compile using `cross` or native toolchains
2. Strip binaries for size reduction
3. Create archives with binary and README
4. Generate SHA256 checksums
5. Upload to GitHub Release as assets

**Installation**:

```bash
# Linux/macOS
curl -L https://github.com/pvandervelde/repo_roller/releases/download/v0.2.0/repo-roller-v0.2.0-x86_64-linux.tar.gz | tar xz
sudo mv repo-roller /usr/local/bin/

# Windows (PowerShell)
Invoke-WebRequest -Uri https://github.com/pvandervelde/repo_roller/releases/download/v0.2.0/repo-roller-v0.2.0-x86_64-windows.zip -OutFile repo-roller.zip
Expand-Archive repo-roller.zip
Move-Item repo-roller\repo-roller.exe C:\Windows\System32\
```

## Error Handling

### Release PR Workflow Errors

**No commits since last release**:

- Skip PR creation/update
- Log message: "No changes to release"
- Exit gracefully

**Version calculation failure**:

- Log error with commit history details
- Create issue: "Release version calculation failed"
- Notify maintainers

**Changelog generation failure**:

- Fall back to simple commit list
- Log warning
- Continue with release PR

**PR creation/update failure**:

- Retry up to 3 times with exponential backoff
- Create issue if all retries fail
- Include error details in issue

### Release Publication Errors

**Container build failure**:

- Fail workflow (block release)
- Log detailed build error
- Rollback: Delete created release tag
- Notify maintainers

**Binary build failure**:

- Continue with successful targets
- Mark release as "incomplete" in notes
- Create issue for failed targets
- Notify maintainers

**Asset upload failure**:

- Retry up to 3 times
- Log detailed error
- Create issue if persistent
- Release remains published (manual asset upload possible)

## Security Considerations

### Token Permissions

**Release PR Workflow** needs:

- `contents: write` - Create/update release PR branch
- `pull-requests: write` - Create/update PR
- `actions: read` - Trigger other workflows

**Release Publication Workflow** needs:

- `contents: write` - Create release and tags
- `packages: write` - Push to GHCR
- `actions: read` - Access build artifacts

### Secrets Management

**Required Secrets**:

- `GITHUB_TOKEN` - Automatically provided by GitHub Actions
- No additional secrets needed (uses GitHub's built-in authentication)

### Supply Chain Security

**Container Image**:

- Base image pinned with SHA256 digest
- Multi-stage build (minimal attack surface)
- Vulnerability scanning in CI
- Signed with cosign (future enhancement)

**CLI Binaries**:

- Built in GitHub Actions (reproducible)
- SHA256 checksums provided
- Code signing (future enhancement)

## Testing Strategy

### Pre-Release Testing

**Dry Run Mode**:

```yaml
# Test release workflow without creating PR
workflow_dispatch:
  inputs:
    dry_run:
      description: 'Dry run (no PR creation)'
      required: false
      default: 'false'
```

**Integration Tests**:

- Test version calculation with various commit scenarios
- Test changelog generation with different commit types
- Test version override command parsing
- Test artifact build process

### Release Validation

**Post-Release Checks**:

1. Verify GitHub release created
2. Verify container image pullable
3. Verify CLI binaries downloadable
4. Verify checksums match
5. Smoke test: Run CLI `--version` command
6. Smoke test: Start API container

## Operational Procedures

### Manual Release Process

If automated release fails, manual fallback:

1. **Create release tag**:

   ```bash
   git tag -a v0.2.0 -m "Release v0.2.0"
   git push origin v0.2.0
   ```

2. **Build container manually**:

   ```bash
   cd crates/repo_roller_api
   docker build -t ghcr.io/pvandervelde/repo_roller_api:v0.2.0 .
   docker push ghcr.io/pvandervelde/repo_roller_api:v0.2.0
   ```

3. **Build binaries manually**:

   ```bash
   cargo build --release --package repo_roller_cli
   # Repeat for each target platform
   ```

4. **Create GitHub release** via web UI
5. **Upload assets** manually

### Hotfix Release Process

For urgent fixes bypassing normal process:

1. Create hotfix branch from release tag
2. Apply fix and commit with `fix:` type
3. Create PR directly to master
4. After merge, manually trigger release workflow
5. Release PR will show only hotfix changes

### Version Rollback

If release has critical issues:

1. **Do NOT delete release** (breaks user installations)
2. Create new patch release with fix
3. Update release notes of problematic version with deprecation notice
4. Mark problematic version as "yanked" in documentation

## Monitoring and Metrics

### Release Metrics

**Track**:

- Release frequency (releases per month)
- Time from commit to release
- Manual intervention rate
- Release failure rate
- Artifact download counts

**Alerts**:

- Release workflow failure
- Artifact publication failure
- Unusual release frequency (too fast/slow)

## Future Enhancements

**Planned Improvements**:

1. **Automated Testing**: Run full test suite before release PR creation
2. **Code Signing**: Sign binaries and container images
3. **Release Notes Enhancement**: Auto-generate contributor credits
4. **Dependency Updates**: Auto-update dependencies in release PR
5. **Multiple Channels**: Support stable/beta/alpha release channels
6. **Homebrew Formula**: Auto-update Homebrew formula on release
7. **Cargo Publish**: Publish crates to crates.io (when ready)
8. **Release Schedule**: Support scheduled releases (e.g., monthly)

## References

- **Conventional Commits**: <https://www.conventionalcommits.org/>
- **Semantic Versioning**: <https://semver.org/>
- **GitHub Actions**: <https://docs.github.com/en/actions>
- **GitHub Container Registry**: <https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry>
- **Cross-Compilation**: <https://github.com/cross-rs/cross>

## Related Specifications

- [Error Handling](./error-types.md) - Error types and handling
- [API Interface](./api-endpoints.md) - REST API versioning
- [CLI Interface](./cli-commands.md) - CLI version output
- [Constraints](../constraints.md) - Implementation constraints
- [Vocabulary](../vocabulary.md) - Version naming conventions
