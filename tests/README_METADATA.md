# Test Metadata Repository

This document describes the test metadata repository used for integration testing of the organization settings system.

## Repository Details

- **Organization**: glitchgrove
- **Repository**: `.reporoller-test`
- **Purpose**: Test configuration data for integration tests
- **Visibility**: Private

## Repository Structure

```
.reporoller-test/
├── global/
│   ├── defaults.toml           # Organization-wide default settings
│   └── standard-labels.toml    # Standard labels for all repositories
├── teams/
│   ├── platform/
│   │   ├── config.toml         # Platform team configuration overrides
│   │   └── labels.toml         # Platform team-specific labels
│   └── backend/
│       ├── config.toml         # Backend team configuration overrides
│       └── labels.toml         # Backend team-specific labels
├── types/
│   ├── library/
│   │   └── config.toml         # Library repository type configuration
│   └── service/
│       └── config.toml         # Service repository type configuration
└── README.md                   # Repository documentation
```

## Configuration Hierarchy

The metadata repository demonstrates the four-level configuration hierarchy:

1. **Global** (`global/defaults.toml`): Organization-wide defaults
2. **Repository Type** (`types/*/config.toml`): Type-specific settings
3. **Team** (`teams/*/config.toml`): Team-specific overrides
4. **Template**: Defined in RepoRoller CLI configuration

### Global Defaults

Applies to all repositories in the organization:

- Issues: Enabled (overridable)
- Projects: Disabled (overridable)
- Discussions: Enabled (overridable)
- Wiki: Disabled (overridable)
- Security: Enabled (NOT overridable)
- Pull requests: Squash and rebase allowed
- Branch protection: Require 1 review (at least)

### Team Configurations

#### Platform Team

- **Discussions**: Disabled (different from global)
- **Wiki**: Enabled (different from global)
- **Approving reviews**: 2 required
- **Custom properties**: team=platform, criticality=high
- **Labels**: platform-infrastructure, security-critical, performance

#### Backend Team

- **Projects**: Enabled (different from global)
- **Auto-merge**: Enabled
- **Custom properties**: team=backend, service_tier=tier-2
- **Labels**: api-breaking, database, microservice

### Repository Types

#### Library Type

- **Wiki**: Disabled
- **Discussions**: Enabled
- **Code owner reviews**: Required
- **Custom properties**: repo_type=library, visibility=public

#### Service Type

- **Wiki**: Enabled
- **Discussions**: Disabled
- **Approving reviews**: 2 required
- **Custom properties**: repo_type=service, deployment_type=kubernetes
- **Environments**: development, staging, production (with wait timers)

## Standard Labels

All repositories receive these standard labels from `global/standard-labels.toml`:

- `bug`: Something isn't working (red)
- `enhancement`: New feature or request (cyan)
- `documentation`: Documentation improvements (blue)
- `good-first-issue`: Good for newcomers (purple)
- `help-wanted`: Extra attention needed (green)
- `question`: Further information requested (pink)
- `wontfix`: This will not be worked on (white)
- `duplicate`: Issue or PR already exists (gray)
- `invalid`: This doesn't seem right (yellow)

## Maintenance

### Creating/Recreating the Repository

Use the provided PowerShell script:

```powershell
# Create repository
.\tests\create-test-metadata-repository.ps1

# Force recreate repository
.\tests\create-test-metadata-repository.ps1 -Force

# Verbose logging
.\tests\create-test-metadata-repository.ps1 -Verbose
```

### Verification

Verify the repository structure:

```bash
# List repository contents
gh api repos/glitchgrove/.reporoller-test/contents --jq '.[].name'

# View global defaults
gh api repos/glitchgrove/.reporoller-test/contents/global/defaults.toml --jq '.content' | base64 -d

# View team configuration
gh api repos/glitchgrove/.reporoller-test/contents/teams/platform/config.toml --jq '.content' | base64 -d
```

## Integration Test Usage

The integration tests use this metadata repository to verify:

1. **Organization Settings Loading**: Tests load global defaults from `global/defaults.toml`
2. **Team Overrides**: Tests verify team-specific settings override global defaults
3. **Repository Types**: Tests verify type-specific configurations are applied
4. **Custom Properties**: Tests verify custom properties are set via GitHub API
5. **Configuration Hierarchy**: Tests verify the complete precedence chain

### Test Environment Variables

Required for integration tests:

```bash
export GITHUB_APP_ID=1442780
export GITHUB_APP_PRIVATE_KEY="<private key>"
export TEST_ORG=glitchgrove
export METADATA_REPO=.reporoller-test
```

### Example Test Scenarios

```rust
// Test organization settings integration
#[tokio::test]
async fn test_organization_settings_integration() -> Result<()> {
    let config = TestConfig::from_env()?;
    let mut runner = IntegrationTestRunner::new(config).await?;

    // Create repository - should apply global defaults
    let result = runner.run_single_test_scenario(
        TestScenario::OrganizationSettings
    ).await;

    assert!(result.success);
    // Verify repository has settings from global/defaults.toml

    runner.cleanup_test_repositories().await?;
    Ok(())
}
```

## Notes

- This is a **test-only** repository; do not use for production
- Repository is automatically created by test setup scripts
- Configuration format follows RepoRoller metadata repository specification
- Custom properties require GitHub App with appropriate permissions

## Related Documentation

- [Integration Testing Strategy](../specs/testing/integration-testing-strategy.md)
- [Template Data Collection](../specs/testing/template-data-collection.md)
- [Organization Repository Settings Design](../specs/design/organization-repository-settings.md)
