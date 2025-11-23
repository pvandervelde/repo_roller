# Integration Testing Strategy and Configuration Verification

## Overview

This document describes the testing strategy for RepoRoller integration tests, with particular focus on configuration verification.

## Testing Philosophy

### Hybrid Testing Approach

RepoRoller uses a **hybrid testing strategy** combining:

1. **Domain-Level Integration Tests** - Test core business logic with real GitHub infrastructure
2. **REST API Integration Tests** - Test HTTP layer and end-to-end workflows via REST API
3. **Configuration Verification** - Verify that settings were actually applied to GitHub repositories

This hybrid approach ensures:

- Core business logic works correctly (domain tests)
- HTTP layer translates requests/responses correctly (API tests)
- Configuration is actually applied to GitHub (verification tests)
- Complete end-to-end workflows function properly (workflow tests)

## Test Layers

### Layer 1: Domain-Level Integration Tests

**Location**: `crates/integration_tests/tests/organization_settings_scenarios.rs`

**Purpose**: Test core repository creation workflow with organization settings system

**What These Tests Verify**:

- Metadata repository discovery and loading
- Configuration hierarchy merging (Template > Team > Type > Global)
- Repository creation through `repo_roller_core::create_repository()`
- Custom property assignment
- Configuration application to GitHub repositories

**Test Scenarios**:

1. `test_organization_settings_with_global_defaults` - Global configuration only
2. `test_team_configuration_overrides` - Team-specific overrides
3. `test_repository_type_configuration` - Repository type settings
4. `test_configuration_hierarchy_merging` - Complete hierarchy precedence
5. `test_complete_organization_settings_workflow` - Full integration

**Example Test Pattern**:

```rust
#[tokio::test]
#[ignore = "Requires real GitHub infrastructure"]
async fn test_organization_settings_with_global_defaults() {
    // 1. Setup: Load credentials, prepare request
    let request = RepositoryCreationRequestBuilder::new()
        .repository_name("test-repo")
        .organization_name("test-org")
        .template_name("default-template")
        .build()
        .unwrap();

    // 2. Execute: Create repository via domain API
    let result = create_repository(request, &auth_service, &config_manager).await;

    // 3. Verify: Check creation succeeded
    assert!(result.is_ok());

    // 4. Verify Configuration: Check GitHub state
    let verification = verify_repository_configuration(
        &github_client,
        "test-org",
        "test-repo",
        &expected_config
    ).await.unwrap();

    assert!(verification.passed, "Configuration verification failed: {:?}", verification.failures);
}
```

### Layer 2: REST API Integration Tests

**Location**: `crates/integration_tests/tests/rest_api_endpoints.rs`

**Purpose**: Test REST API endpoints with real GitHub infrastructure

**What These Tests Verify**:

- HTTP request/response serialization
- Authentication middleware
- API error handling
- Configuration preview endpoints
- Repository creation via REST API
- Complete API workflows

**Test Categories**:

1. **Organization Settings Endpoints** (10 tests):
   - `test_list_repository_types_success` - List available types
   - `test_get_repository_type_config_success` - Get type configuration
   - `test_get_global_defaults_success` - Get organization defaults
   - `test_preview_configuration_success` - Preview merged configuration
   - `test_validate_organization_success` - Validate metadata repository

2. **Template Discovery Endpoints** (6 tests):
   - `test_list_templates_success` - Discover template repositories
   - `test_get_template_details_success` - Get template configuration
   - `test_validate_template_success` - Validate template structure

3. **Repository Management Endpoints** (3 tests):
   - `test_validate_repository_name_valid` - Name format validation
   - `test_validate_repository_request_valid` - Complete request validation

4. **Authentication Tests** (3 tests):
   - `test_missing_authentication_token` - No token provided
   - `test_invalid_authentication_token` - Invalid token
   - `test_malformed_authentication_header` - Malformed header

5. **Configuration Verification Tests** (8 tests - Task 9a.3):
   - **Preview Tests**:
     - `test_api_configuration_preview_global_defaults` - Global only
     - `test_api_configuration_preview_with_team` - Team overrides
     - `test_api_configuration_preview_with_type` - Type settings
     - `test_api_configuration_preview_complete_hierarchy` - Full hierarchy
   - **Creation Tests**:
     - `test_api_create_repository_with_global_defaults` - Create with global config
     - `test_api_create_repository_with_team_overrides` - Create with team
     - `test_api_create_repository_with_type_configuration` - Create with type
     - `test_api_create_repository_with_complete_hierarchy` - Create with all levels

6. **End-to-End Workflow Tests** (2 tests):
   - `test_complete_rest_api_workflow` - Multi-endpoint workflow
   - `test_complete_repository_creation_workflow_dry_run` - Full creation workflow (no actual repository)

**Example API Test Pattern**:

```rust
#[tokio::test]
#[ignore = "Requires real GitHub infrastructure"]
async fn test_api_configuration_preview_with_team() {
    let app = create_router(test_app_state());
    let token = test_token();
    let org = test_org();

    let request_body = json!({
        "template": "my-template",
        "team": "platform-team"
    });

    let request = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/orgs/{}/preview", org))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let preview: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify merged configuration structure
    assert!(preview["mergedConfiguration"].is_object());
    assert!(preview["sources"].is_object());

    // Verify team overrides are present in sources
    // (implementation detail depends on response structure)
}
```

### Layer 3: Configuration Verification Module

**Location**: `crates/integration_tests/src/verification.rs`

**Purpose**: Provide reusable verification helpers for checking GitHub state

**Components**:

#### Verification Types

```rust
/// Results of configuration verification
pub struct ConfigurationVerification {
    pub passed: bool,
    pub settings_verified: bool,
    pub custom_properties_verified: bool,
    pub branch_protection_verified: bool,
    pub labels_verified: bool,
    pub failures: Vec<String>,
}

/// Expected configuration to verify
pub struct ExpectedConfiguration {
    pub repository_settings: Option<ExpectedRepositorySettings>,
    pub custom_properties: Option<HashMap<String, String>>,
    pub branch_protection: Option<ExpectedBranchProtection>,
    pub labels: Option<Vec<String>>,
}
```

#### Verification Functions

1. **`verify_repository_settings()`** - Check repository feature flags
   - Verifies: `has_issues`, `has_wiki`, `has_discussions`, `has_projects`
   - Note: Requires Repository model extension (TODO)

2. **`verify_custom_properties()`** - Check custom properties were set
   - Verifies: All expected properties exist with correct values
   - Allows extra properties (for forward compatibility)

3. **`verify_branch_protection()`** - Check branch protection rules
   - Verifies: Required reviewers, code owner reviews, stale review dismissal
   - Handles branches without protection (returns appropriate error)

4. **`verify_labels()`** - Check labels were created
   - Verifies: All expected labels exist
   - Allows extra labels (GitHub default labels)

5. **`load_expected_configuration()`** - Load expected config from metadata repo
   - TODO: Implement TOML parsing from metadata repository
   - Constructs `ExpectedConfiguration` for verification

#### Usage Example

```rust
// After creating repository, verify configuration was applied
let expected = ExpectedConfiguration {
    custom_properties: Some(hashmap! {
        "team".to_string() => "platform".to_string(),
        "repo_type".to_string() => "library".to_string(),
    }),
    labels: Some(vec![
        "bug".to_string(),
        "enhancement".to_string(),
    ]),
    branch_protection: Some(ExpectedBranchProtection {
        branch: "main".to_string(),
        required_approving_review_count: Some(2),
        require_code_owner_reviews: Some(true),
        dismiss_stale_reviews: Some(true),
    }),
    repository_settings: None, // TODO: Implement when model extended
};

let verification = verify_all_configuration(
    &github_client,
    "my-org",
    "my-repo",
    &expected
).await?;

assert!(verification.passed,
    "Configuration verification failed:\n{:#?}",
    verification.failures
);
```

## Configuration Verification Workflow

### Step-by-Step Verification Process

1. **Create Repository**

   ```rust
   let result = create_repository(request, &auth_service, &config_manager).await?;
   ```

2. **Load Expected Configuration**

   ```rust
   let expected = load_expected_configuration(
       &github_client,
       "my-org",
       ".reporoller",  // metadata repo
       &test_scenario
   ).await?;
   ```

3. **Verify Custom Properties**

   ```rust
   let props_verification = verify_custom_properties(
       &github_client,
       "my-org",
       "my-repo",
       &expected.custom_properties.unwrap()
   ).await?;
   ```

4. **Verify Repository Settings**

   ```rust
   let settings_verification = verify_repository_settings(
       &github_client,
       "my-org",
       "my-repo",
       &expected.repository_settings.unwrap()
   ).await?;
   ```

5. **Verify Branch Protection**

   ```rust
   let protection_verification = verify_branch_protection(
       &github_client,
       "my-org",
       "my-repo",
       &expected.branch_protection.unwrap()
   ).await?;
   ```

6. **Verify Labels**

   ```rust
   let labels_verification = verify_labels(
       &github_client,
       "my-org",
       "my-repo",
       &expected.labels.unwrap()
   ).await?;
   ```

7. **Assert All Verifications Passed**

   ```rust
   assert!(props_verification.passed);
   assert!(settings_verification.passed);
   assert!(protection_verification.passed);
   assert!(labels_verification.passed);
   ```

## Test Infrastructure Requirements

### Environment Variables

All integration tests require these environment variables:

```bash
# Required for all tests
GITHUB_TOKEN=<github_app_installation_token>
TEST_ORG=<organization_name>

# Required for specific test scenarios
TEST_TEMPLATE=<template_repository_name>
TEST_TEAM=<team_slug>
TEST_REPOSITORY_TYPE=<repository_type_name>

# Optional
METADATA_REPOSITORY_NAME=<custom_metadata_repo_name>  # defaults to ".reporoller"
TEST_ORG_NO_METADATA=<org_without_metadata_repo>
```

### GitHub App Permissions

The GitHub App used for testing must have these permissions:

**Repository Permissions**:

- `administration`: Read & Write (for repository settings)
- `contents`: Read & Write (for repository creation)
- `custom_properties`: Read & Write (for custom property management)
- `metadata`: Read (for repository information)

**Organization Permissions**:

- `administration`: Read (for organization settings)
- `custom_properties`: Read & Write (for custom property definitions)
- `members`: Read (for team information)

### Test Metadata Repository

Tests require a metadata repository with this structure:

```
.reporoller/
├── global.toml              # Organization-wide defaults
├── teams/
│   └── platform-team.toml   # Team-specific configuration
└── types/
    └── library.toml         # Repository type configuration
```

**Setup Script**: `tests/create-test-metadata-repository.ps1`
**Documentation**: `tests/README_METADATA.md`

## Running Integration Tests

### Run All Integration Tests

```bash
# Run all tests (requires GitHub infrastructure)
cargo test -p integration_tests --test '*' -- --ignored

# Run specific test file
cargo test -p integration_tests --test organization_settings_scenarios -- --ignored
cargo test -p integration_tests --test rest_api_endpoints -- --ignored
```

### Run Specific Test Scenarios

```bash
# Run only global defaults test
cargo test -p integration_tests --test organization_settings_scenarios test_organization_settings_with_global_defaults -- --ignored

# Run only API preview tests
cargo test -p integration_tests --test rest_api_endpoints test_api_configuration_preview -- --ignored

# Run verification workflow tests
cargo test -p integration_tests --test rest_api_endpoints test_complete -- --ignored
```

### Test Output

Tests provide detailed output:

- ✅ Success indicators for each verification step
- ❌ Failure details with specific mismatches
- 📊 Configuration sources showing hierarchy
- 🔍 Verification failure reasons

## Adding New Configuration Verification Tests

### Checklist for New Tests

When adding tests that create repositories:

1. **Load Expected Configuration**
   - Either hardcode expected values for the test
   - Or load from metadata repository using `load_expected_configuration()`

2. **Create Repository**
   - Use domain API or REST API depending on test layer

3. **Verify Creation Succeeded**
   - Check that repository was created
   - Capture repository metadata

4. **Verify Configuration Applied**
   - Use verification functions from `verification` module
   - Check all relevant configuration aspects:
     - Custom properties
     - Repository settings
     - Branch protection
     - Labels

5. **Assert Verification Passed**
   - Fail test if any verification step fails
   - Include detailed failure information

6. **Clean Up** (Optional)
   - Delete test repository if desired
   - Note: Tests currently use timestamped names to avoid conflicts

### Example: Adding a New Verification Test

```rust
#[tokio::test]
#[ignore = "Requires real GitHub infrastructure - CREATES REPOSITORY"]
async fn test_my_new_configuration_scenario() {
    // 1. Setup
    let github_client = create_github_client().await;
    let repo_name = format!("test-scenario-{}", timestamp());

    // 2. Define expected configuration
    let expected = ExpectedConfiguration {
        custom_properties: Some(hashmap! {
            "my_property".to_string() => "my_value".to_string(),
        }),
        // ... other expected values
    };

    // 3. Create repository via API/domain
    let result = create_repository_somehow().await;
    assert!(result.is_ok());

    // 4. Verify custom properties
    let verification = verify_custom_properties(
        &github_client,
        "test-org",
        &repo_name,
        &expected.custom_properties.unwrap()
    ).await.unwrap();

    // 5. Assert verification passed
    assert!(
        verification.passed,
        "Custom properties verification failed: {:?}",
        verification.failures
    );

    // 6. Verify other aspects (settings, protection, labels)
    // ...
}
```

## Common Testing Patterns

### Pattern 1: Configuration Hierarchy Testing

Test that each level of the hierarchy correctly overrides lower levels:

```rust
// Test progression: Global → Type → Team → Template
// Each test adds one more level and verifies precedence

#[tokio::test]
async fn test_global_only() { /* verify global settings */ }

#[tokio::test]
async fn test_with_repository_type() { /* verify type overrides global */ }

#[tokio::test]
async fn test_with_team() { /* verify team overrides type */ }

#[tokio::test]
async fn test_with_template() { /* verify template overrides team */ }
```

### Pattern 2: API Workflow Testing

Test complete multi-step workflows through the API:

```rust
#[tokio::test]
async fn test_complete_workflow() {
    // Step 1: Validate organization
    let validation = api_call("/orgs/{org}/validate").await;
    assert!(validation.valid);

    // Step 2: List templates
    let templates = api_call("/orgs/{org}/templates").await;
    assert!(!templates.is_empty());

    // Step 3: Preview configuration
    let preview = api_call("/orgs/{org}/preview").await;
    assert!(preview.merged_configuration.is_some());

    // Step 4: Create repository
    let result = api_call("/repositories").await;
    assert!(result.success);

    // Step 5: Verify configuration applied
    verify_all_configuration().await;
}
```

### Pattern 3: Error Path Testing

Test that configuration errors are caught and reported:

```rust
#[tokio::test]
async fn test_invalid_configuration_rejected() {
    let invalid_config = /* create invalid configuration */;

    let result = create_repository_with_config(invalid_config).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        RepoRollerError::Configuration(err) => {
            assert!(err.to_string().contains("expected error message"));
        }
        _ => panic!("Expected ConfigurationError"),
    }
}
```

## Known Limitations and TODOs

### Current Limitations

1. **Repository Model Extension Needed**
   - `verify_repository_settings()` cannot fully verify settings
   - Repository model lacks `has_issues`, `has_wiki`, `has_discussions`, `has_projects` fields
   - Workaround: Tests document this limitation and skip settings verification

2. **Expected Configuration Loading Not Implemented**
   - `load_expected_configuration()` is a stub
   - Tests currently hardcode expected values
   - TODO: Implement TOML parsing from metadata repository

3. **REST API Tests Don't Verify GitHub State**
   - API creation tests (9a.3) verify API responses
   - They don't yet call verification module to check GitHub
   - TODO: Wire verification module into API creation tests

### Future Enhancements

1. **Extend Repository Model**
   - Add feature flag fields to `github_client::models::Repository`
   - Update `get_repository_settings()` to populate these fields
   - Enable full settings verification

2. **Implement Configuration Loading**
   - Parse metadata repository TOML files
   - Construct `ExpectedConfiguration` from parsed data
   - Enable automatic expected value loading

3. **Add Verification to API Tests**
   - Call verification module from API creation tests
   - Verify actual GitHub state matches expected configuration
   - Complete the verification loop

4. **Add Performance Tests**
   - Measure configuration loading time
   - Measure merge operation performance
   - Test with large metadata repositories

5. **Add Cleanup Helpers**
   - Automatic test repository deletion
   - Cleanup of test custom properties
   - Teardown scripts for test infrastructure

## Troubleshooting

### Tests Fail with "Metadata Repository Not Found"

**Cause**: Metadata repository doesn't exist or isn't configured correctly

**Solution**:

1. Run setup script: `tests/create-test-metadata-repository.ps1`
2. Verify `TEST_ORG` points to correct organization
3. Check GitHub App has access to metadata repository

### Tests Fail with "Permission Denied"

**Cause**: GitHub App lacks required permissions

**Solution**:

1. Check GitHub App permissions match requirements above
2. Verify app is installed on test organization
3. Regenerate installation token if needed

### Verification Fails but Repository Exists

**Cause**: Configuration wasn't applied or applied incorrectly

**Solution**:

1. Check GitHub repository manually to see actual state
2. Compare with expected configuration in test
3. Check logs for configuration application errors
4. Verify metadata repository configuration is valid

### Tests Time Out

**Cause**: GitHub API rate limiting or network issues

**Solution**:

1. Reduce test concurrency (run tests sequentially)
2. Check GitHub API rate limits
3. Verify network connectivity to GitHub
4. Increase test timeout values if needed

## Summary

This testing strategy ensures:

- ✅ **Core logic works** - Domain tests verify business logic
- ✅ **HTTP layer works** - API tests verify REST endpoints
- ✅ **Configuration applies** - Verification tests check GitHub state
- ✅ **Workflows complete** - End-to-end tests verify full scenarios
- ✅ **Errors caught** - Validation tests catch configuration issues

By following this strategy, we prevent false confidence from tests that only check success flags without verifying actual GitHub repository state.
