# Releasing RepoRoller

This document describes the release process for RepoRoller maintainers.

## Overview

RepoRoller uses automated release management with GitHub Actions. Releases are created automatically based on conventional commits merged to the `master` branch.

## Automated Release Process

### 1. Normal Development Workflow

1. **Create feature branches** from `master`
2. **Write conventional commits** following the format below
3. **Create pull requests** to merge features to `master`
4. **Merge PRs** to `master` after review

### 2. Release PR Creation

When commits are merged to `master`:

1. **Automatic version calculation**: The `release-pr` workflow analyzes commits since the last release
2. **Changelog generation**: Commits are grouped and formatted into a changelog
3. **Version update**: `Cargo.toml` is updated with the new version
4. **Release PR created**: A PR is automatically created with:
   - Title: `chore(release): prepare release vX.Y.Z`
   - Labels: `release`, `automated`
   - Body: Generated changelog and instructions

### 3. Release PR Review

Maintainers review the release PR:

- ✅ **Verify changelog accuracy**: Check that all changes are documented
- ✅ **Confirm version is correct**: Ensure semver rules were applied correctly
- ✅ **Override version if needed**: Comment `/release <major|minor|patch>` to change version type

### 4. Publishing the Release

When the release PR is merged:

1. **GitHub Release created** with tag `vX.Y.Z`
2. **Container image built** and pushed to `ghcr.io/pvandervelde/repo_roller_api`
3. **CLI binaries built** for Linux, Windows, and macOS
4. **Release assets uploaded** with checksums
5. **Release notes updated** with installation instructions

## Conventional Commits Format

Commit messages must follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Commit Types and Version Bumps

| Type | Version Bump | Description | Example |
|------|--------------|-------------|---------|
| `feat:` | **MINOR** | New feature | `feat(cli): add template validation command` |
| `fix:` | **PATCH** | Bug fix | `fix(api): handle missing configuration file` |
| `<type>!:` or `BREAKING CHANGE:` | **MAJOR** | Breaking change | `feat(config)!: change hierarchy structure` |
| `docs:` | None | Documentation only | `docs: update release guide` |
| `chore:` | None | Maintenance tasks | `chore: update dependencies` |
| `ci:` | None | CI/CD changes | `ci: add release workflow` |
| `test:` | None | Test additions | `test: add integration tests` |
| `refactor:` | None | Code refactoring | `refactor: extract validation logic` |
| `perf:` | None | Performance improvements | `perf: optimize template caching` |
| `style:` | None | Code formatting | `style: run cargo fmt` |

### Breaking Changes

Mark breaking changes in two ways:

1. **Exclamation mark**: `feat(api)!: change endpoint response format`
2. **Footer**:

   ```
   feat(api): change endpoint response format

   BREAKING CHANGE: Response now returns JSON object instead of array.
   Clients must update their parsing logic.
   ```

### Scope

The scope indicates which component changed:

- `cli`: CLI interface changes
- `api`: REST API changes
- `core`: Core business logic
- `config`: Configuration system
- `template`: Template engine
- `github`: GitHub integration
- `auth`: Authentication
- `test`: Test infrastructure
- `docs`: Documentation

### Examples

**Feature (MINOR bump)**:

```
feat(cli): add repository visibility command

Add 'repo-roller visibility get' command to check current
repository visibility settings and policies.
```

**Bug Fix (PATCH bump)**:

```
fix(template): handle missing template variables gracefully

Previously crashed with panic when template variable was undefined.
Now returns clear error message with variable name.

Fixes #123
```

**Breaking Change (MAJOR bump)**:

```
feat(config)!: restructure configuration hierarchy

BREAKING CHANGE: Configuration file structure has changed.
The 'settings' section is now 'repository' and 'overrides'
moved to top level.

Migration required for existing configurations.
See MIGRATION.md for upgrade instructions.
```

**Documentation (no version bump)**:

```
docs(README): add installation instructions

Document how to install repo_roller_cli using various methods.
```

## Version Override

If the automatic version calculation is incorrect, maintainers can override it:

### Using Comment Commands

On the release PR, comment:

- `/release major` - Force MAJOR version bump (breaking changes)
- `/release minor` - Force MINOR version bump (new features)
- `/release patch` - Force PATCH version bump (bug fixes only)

The workflow will validate the override and update the PR if valid.

### Override Rules

Version overrides must follow semantic versioning:

- ✅ **Can increase**: PATCH → MINOR, PATCH → MAJOR, MINOR → MAJOR
- ❌ **Cannot decrease**: MAJOR → MINOR, MINOR → PATCH
- ❌ **Cannot stay same**: MINOR → MINOR (already calculated as MINOR)

**Examples**:

- If calculated as MINOR, you can override to MAJOR ✅
- If calculated as MAJOR, you cannot override to MINOR ❌
- If calculated as PATCH, you can override to MINOR or MAJOR ✅

## Manual Release (Fallback)

If automated workflows fail, you can create a release manually:

### 1. Create Release Tag

```bash
# Checkout master branch
git checkout master
git pull origin master

# Create and push tag
VERSION="0.2.0"
git tag -a "v${VERSION}" -m "Release v${VERSION}"
git push origin "v${VERSION}"
```

### 2. Build Container Image

```bash
cd crates/repo_roller_api

# Build image
docker build -t "ghcr.io/pvandervelde/repo_roller_api:v${VERSION}" .

# Log in to GHCR
echo $GITHUB_TOKEN | docker login ghcr.io -u USERNAME --password-stdin

# Push image
docker push "ghcr.io/pvandervelde/repo_roller_api:v${VERSION}"

# Tag and push as latest (if stable release)
docker tag "ghcr.io/pvandervelde/repo_roller_api:v${VERSION}" "ghcr.io/pvandervelde/repo_roller_api:latest"
docker push "ghcr.io/pvandervelde/repo_roller_api:latest"
```

### 3. Build CLI Binaries

```bash
# Build for Linux
cargo build --release --package repo_roller_cli --target x86_64-unknown-linux-gnu
tar czf "repo-roller-v${VERSION}-x86_64-linux.tar.gz" \
  -C target/x86_64-unknown-linux-gnu/release repo_roller_cli \
  README.md LICENSE

# Build for Windows (requires cross-compilation setup)
cargo build --release --package repo_roller_cli --target x86_64-pc-windows-msvc
zip "repo-roller-v${VERSION}-x86_64-windows.zip" \
  target/x86_64-pc-windows-msvc/release/repo_roller_cli.exe \
  README.md LICENSE

# Build for macOS (requires macOS or cross-compilation)
cargo build --release --package repo_roller_cli --target x86_64-apple-darwin
tar czf "repo-roller-v${VERSION}-x86_64-macos.tar.gz" \
  -C target/x86_64-apple-darwin/release repo_roller_cli \
  README.md LICENSE
```

### 4. Generate Checksums

```bash
sha256sum repo-roller-v${VERSION}-*.tar.gz > checksums.sha256
sha256sum repo-roller-v${VERSION}-*.zip >> checksums.sha256
```

### 5. Create GitHub Release

1. Go to <https://github.com/pvandervelde/repo_roller/releases>
2. Click "Draft a new release"
3. Enter tag: `vX.Y.Z`
4. Enter title: `Release vX.Y.Z`
5. Add changelog in description
6. Upload binary archives and checksums
7. Mark as pre-release if version is 0.x.x
8. Click "Publish release"

## Hotfix Releases

For urgent fixes that need immediate release:

### 1. Create Hotfix Branch

```bash
# Branch from latest release tag
git checkout -b hotfix/v0.2.1 v0.2.0

# Or branch from master if already merged
git checkout -b hotfix/v0.2.1 master
```

### 2. Apply Fix

```bash
# Make changes
# Commit with fix: type
git commit -m "fix(critical): resolve security vulnerability

Fixes CVE-2024-XXXXX by updating dependency and validating input.
"
```

### 3. Create PR and Merge

- Create PR to `master`
- Get expedited review
- Merge immediately after approval

### 4. Release Process

The automated release workflow will:

- Detect the `fix:` commit
- Calculate PATCH version bump
- Create release PR
- Publish release when merged

For even faster release, manually trigger workflow:

```bash
gh workflow run release-pr.yml
```

## Version Rollback

If a release has critical issues:

### DO NOT Delete the Release

Deleting releases breaks existing installations and is bad practice.

### Instead

1. **Create hotfix** with the fix
2. **Release new version** (e.g., 0.2.1 fixes issues in 0.2.0)
3. **Update problematic release notes**:

   ```markdown
   ⚠️ **DEPRECATED**: This version has critical issues.
   Please upgrade to v0.2.1 immediately.

   See [release notes for v0.2.1](../v0.2.1) for details.
   ```

## Pre-Release Versions

Versions `0.x.x` are automatically marked as pre-releases:

- Not tagged as `latest` in container registry
- GitHub release marked as "Pre-release"
- Indicates API is not yet stable

Once the project reaches `1.0.0`, it's considered stable and follows strict semver.

## Monitoring Releases

### Release Metrics

Track release quality in GitHub:

- Release frequency (aim for regular cadence)
- Failed workflow runs (investigate and fix)
- Manual interventions (minimize automation gaps)

### Alerts

Monitor for:

- ❌ Release workflow failures
- ❌ Container build failures
- ❌ Binary build failures
- ⚠️ Unusual release frequency

## Troubleshooting

### Release PR Not Created

**Symptoms**: Commits merged to master but no release PR appeared.

**Possible causes**:

- No version-affecting commits (only docs/chore/test)
- Commits already included in previous release
- Workflow file has syntax errors

**Resolution**:

1. Check workflow runs: <https://github.com/pvandervelde/repo_roller/actions>
2. Verify commit messages follow conventional commits
3. Manually trigger workflow: `gh workflow run release-pr.yml`

### Version Calculation Incorrect

**Symptoms**: Release PR has wrong version number.

**Resolution**:

1. Review commit history since last tag
2. Use version override: `/release <type>` on PR
3. Check `conventional_commits_next_version` configuration

### Container Build Failed

**Symptoms**: Release publication workflow fails at container build.

**Impact**: **Release is rolled back** (tag deleted, release removed).

**Resolution**:

1. Check workflow logs for build errors
2. Test build locally: `docker build -f crates/repo_roller_api/Dockerfile .`
3. Fix Dockerfile or dependencies
4. Manually trigger release after fix

### Binary Build Failed

**Symptoms**: Some binaries missing from release.

**Impact**: **Release published** but incomplete.

**Resolution**:

1. Check which platforms failed
2. Build missing binaries manually
3. Upload to release: `gh release upload vX.Y.Z binary.tar.gz`

### Stale Release Branches

**Symptoms**: Old release branches still exist after new release.

**Resolution**:
Workflow automatically cleans up stale branches. If manual cleanup needed:

```bash
git push origin --delete release/0.1.0
```

## Release Checklist

Before merging release PR:

- [ ] Changelog includes all notable changes
- [ ] Version follows semantic versioning
- [ ] No pending security issues
- [ ] CI/CD pipelines passing
- [ ] Documentation updated if needed
- [ ] Migration guide provided for breaking changes

After release:

- [ ] Container image pulls successfully
- [ ] Binaries download and run correctly
- [ ] GitHub release created with correct tag
- [ ] Checksums verified
- [ ] Documentation site updated (if applicable)

## Questions or Issues?

If you encounter problems with the release process:

1. Check this guide for common issues
2. Review workflow logs in GitHub Actions
3. Create an issue with `release` label
4. Contact maintainers for emergency releases

---

**Remember**: The automated release process is designed to make releases easy and consistent. Trust the automation, but always verify the results.
