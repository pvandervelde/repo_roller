# test_cleanup

Test repository cleanup utilities for RepoRoller.

## Overview

This crate provides utilities for cleaning up test repositories created during RepoRoller's integration and E2E testing. It can be used both programmatically (from test code) and via standalone CLI binaries.

## Features

- **Two cleanup strategies**:
  - Age-based cleanup (orphaned repositories older than X hours)
  - PR-based cleanup (all repositories from a specific pull request)
- **Pattern matching**: Recognizes both `test-repo-roller-*` and `e2e-repo-roller-*` patterns
- **Best-effort deletion**: Logs errors but continues cleanup even if some repos fail
- **GitHub App authentication**: Uses GitHub App for secure, scoped access
- **Detailed logging**: Structured logging with tracing for observability

## CLI Usage

### cleanup-orphans

Cleans up test repositories older than the specified age:

```bash
# Clean up repos older than 24 hours (default for scheduled runs)
cargo run --package test_cleanup --bin cleanup-orphans -- 24

# Clean up repos older than 1 hour
cargo run --package test_cleanup --bin cleanup-orphans -- 1

# Clean up all test repos (0 hours = no age filter)
cargo run --package test_cleanup --bin cleanup-orphans -- 0
```

**Environment variables required**:

- `GITHUB_APP_ID` - GitHub App ID for authentication
- `GITHUB_APP_PRIVATE_KEY` - GitHub App private key (PEM format)
- `TEST_ORG` - Organization name (e.g., "glitchgrove")

### cleanup-pr

Cleans up all test repositories created by a specific pull request:

```bash
# Clean up all repos from PR #456
cargo run --package test_cleanup --bin cleanup-pr -- 456
```

**Environment variables required**: Same as cleanup-orphans

## Programmatic Usage

```rust
use test_cleanup::{CleanupConfig, RepositoryCleanup};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration from environment
    let config = CleanupConfig::from_env()?;

    // Create GitHub client
    let app_client = github_client::create_app_client(
        config.github_app_id,
        &config.github_app_private_key
    ).await?;
    let github_client = github_client::GitHubClient::new(app_client);

    // Create cleanup instance
    let cleanup = RepositoryCleanup::new(github_client, config.test_org);

    // Clean up old repos
    let deleted = cleanup.cleanup_orphaned_repositories(24).await?;
    println!("Deleted {} repositories", deleted.len());

    // Or clean up PR-specific repos
    let pr_deleted = cleanup.cleanup_pr_repositories(456).await?;
    println!("Deleted {} PR repositories", pr_deleted.len());

    Ok(())
}
```

## GitHub Actions Integration

### PR-Based Cleanup

The `cleanup-pr-repos.yml` workflow automatically runs when PRs close:

```yaml
on:
  pull_request:
    types: [closed]
```

It deletes all test repositories matching `-pr{number}-` pattern and posts a comment to the PR with results.

### Scheduled Cleanup (Safety Net)

The `cleanup-test-repos.yml` workflow runs daily at 2 AM UTC:

```yaml
on:
  schedule:
    - cron: "0 2 * * *"
```

It catches any repositories missed by PR cleanup (failed runs, local dev repos, etc.) and alerts if more than 10 orphaned repos are found.

## Repository Naming Convention

This crate recognizes test repositories following the pattern:

```
{prefix}-repo-roller-{context}-{timestamp}-{test-name}-{random}
```

Where:

- **prefix**: `test` or `e2e`
- **context**: `pr{number}`, `main`, `local`, or branch name
- **timestamp**: Unix timestamp
- **test-name**: Test identifier
- **random**: Short random suffix

## Testing

```bash
# Run unit tests
cargo test --package test_cleanup

# Run with logging
RUST_LOG=debug cargo test --package test_cleanup -- --nocapture
```

## License

Licensed under either of Apache License 2.0 or MIT license at your option.
