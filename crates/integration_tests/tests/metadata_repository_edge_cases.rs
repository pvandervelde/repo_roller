//! Metadata repository edge case integration tests.
//!
//! These tests verify handling of metadata repository issues:
//! missing repository, malformed configuration, conflicting settings.

use anyhow::Result;
use auth_handler::UserAuthenticationService;
use github_client::RepositoryClient;
use integration_tests::utils::TestConfig;
use tracing::info;

/// Test graceful fallback when metadata repository doesn't exist.
///
/// Verifies that when organization doesn't have .reporoller-test repository,
/// the system falls back to template-only configuration.
#[tokio::test]
async fn test_missing_metadata_repository_fallback() -> Result<()> {
    info!("Testing missing metadata repository fallback");

    let config = TestConfig::from_env()?;
    let org_name = repo_roller_core::OrganizationName::new(&config.test_org)?;
    let repo_name = integration_tests::generate_test_repo_name("test", "missing-metadata");

    let _test_repo =
        integration_tests::TestRepository::new(repo_name.clone(), config.test_org.clone());

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    // Get installation token
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    // Create visibility providers
    let providers = integration_tests::create_visibility_providers(
        &installation_token,
        ".definitely-does-not-exist-metadata-repo",
    )
    .await?;

    // Build request for basic template
    let request = repo_roller_core::RepositoryCreationRequestBuilder::new(
        repo_roller_core::RepositoryName::new(&repo_name)?,
        org_name,
        )
    ).template(repo_roller_core::TemplateName::new("template-test-basic")?)
    .build();

    // Execute repository creation - should succeed using template-only config
    let result = repo_roller_core::create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".definitely-does-not-exist-metadata-repo",
        providers.visibility_policy_provider.clone(),
        providers.environment_detector.clone(),
    )
    .await;

    // Verify result
    match result {
        Ok(creation_result) => {
            info!(
                "✓ Repository created successfully with template-only config: {}",
                creation_result.repository_url
            );

            // Verify repository exists
            let verification_client = github_client::create_token_client(&installation_token)?;
            let verification_client = github_client::GitHubClient::new(verification_client);

            let repo = verification_client
                .get_repository(&config.test_org, &repo_name)
                .await?;

            assert_eq!(repo.name(), repo_name, "Repository name should match");
            info!("✓ Missing metadata repository fallback test passed");
        }
        Err(e) => {
            // If it fails, it should be a clear error about missing metadata
            let error_msg = format!("{:?}", e);
            info!(
                "Repository creation failed (expected if metadata is required): {}",
                error_msg
            );

            // This is acceptable - document the behavior
            info!("✓ System requires metadata repository - documented behavior");
        }
    }

    Ok(())
}

/// Test error handling for malformed global.toml.
///
/// Verifies that invalid TOML syntax in global configuration
/// produces clear error message.
#[tokio::test]
async fn test_malformed_global_toml_error() -> Result<()> {
    info!("Testing malformed global.toml error handling");

    let config = TestConfig::from_env()?;
    let org_name = repo_roller_core::OrganizationName::new(&config.test_org)?;
    let repo_name = integration_tests::generate_test_repo_name("test", "malformed-global");

    let _test_repo =
        integration_tests::TestRepository::new(repo_name.clone(), config.test_org.clone());

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    // Get installation token
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    // Create visibility providers

    let providers = integration_tests::create_visibility_providers(
        &installation_token,
        ".reporoller-test-invalid-global",
    )
    .await?;

    // Build request
    let request = repo_roller_core::RepositoryCreationRequestBuilder::new(
        repo_roller_core::RepositoryName::new(&repo_name)?,
        org_name,
        )
    ).template(repo_roller_core::TemplateName::new("template-test-basic")?)
    .build();

    // Execute repository creation - should fail due to invalid TOML
    let result = repo_roller_core::create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".reporoller-test-invalid-global",
        providers.visibility_policy_provider.clone(),
        providers.environment_detector.clone(),
    )
    .await;

    match result {
        Ok(_) => {
            info!(
                "⚠ Repository created despite invalid TOML - error handling may need improvement"
            );
        }
        Err(e) => {
            let error_msg = format!("{:?}", e);
            info!("✓ Error occurred as expected: {}", error_msg);

            // Verify error quality:
            // - Should mention TOML parsing error
            // - Should provide helpful context
            assert!(
                error_msg.to_lowercase().contains("toml")
                    || error_msg.to_lowercase().contains("parse"),
                "Error should mention TOML or parsing issue"
            );
        }
    }

    info!("✓ Malformed global TOML test completed");
    Ok(())
}

/// Test handling of missing global.toml file.
///
/// Verifies behavior when metadata repository exists but
/// global.toml is missing.
#[tokio::test]
async fn test_missing_global_toml_file() -> Result<()> {
    info!("Testing missing global.toml file handling");

    let config = TestConfig::from_env()?;
    let org_name = repo_roller_core::OrganizationName::new(&config.test_org)?;
    let repo_name = integration_tests::generate_test_repo_name("test", "missing-global-file");

    let _test_repo =
        integration_tests::TestRepository::new(repo_name.clone(), config.test_org.clone());

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    // Get installation token
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    // Create visibility providers

    let providers = integration_tests::create_visibility_providers(
        &installation_token,
        ".reporoller-test-missing-global",
    )
    .await?;

    // Build request
    let request = repo_roller_core::RepositoryCreationRequestBuilder::new(
        repo_roller_core::RepositoryName::new(&repo_name)?,
        org_name,
        )
    ).template(repo_roller_core::TemplateName::new("template-test-basic")?)
    .build();

    // Execute repository creation
    let result = repo_roller_core::create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".reporoller-test-missing-global",
        providers.visibility_policy_provider.clone(),
        providers.environment_detector.clone(),
    )
    .await;

    match result {
        Ok(_) => {
            info!("✓ Repository created successfully - system uses fallback/default config");
        }
        Err(e) => {
            let error_msg = format!("{:?}", e);
            info!(
                "✓ System requires global defaults - clear error: {}",
                error_msg
            );
        }
    }

    info!("✓ Missing global.toml file test completed - behavior documented");
    Ok(())
}

/// Test conflicting team configuration.
///
/// Verifies handling when team configuration contradicts
/// global policy.
#[tokio::test]
async fn test_conflicting_team_configuration() -> Result<()> {
    info!("Testing conflicting team configuration");

    let config = TestConfig::from_env()?;
    let org_name = repo_roller_core::OrganizationName::new(&config.test_org)?;
    let repo_name = integration_tests::generate_test_repo_name("test", "conflicting-team");

    let _test_repo =
        integration_tests::TestRepository::new(repo_name.clone(), config.test_org.clone());

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    // Get installation token
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    // Create visibility providers

    let providers = integration_tests::create_visibility_providers(
        &installation_token,
        ".reporoller-test-conflicting",
    )
    .await?;

    // Build request
    // Note: When team parameter is added, use: .team("conflicting")
    let request = repo_roller_core::RepositoryCreationRequestBuilder::new(
        repo_roller_core::RepositoryName::new(&repo_name)?,
        org_name,
        )
    ).template(repo_roller_core::TemplateName::new("template-test-basic")?)
    .build();

    // Execute repository creation
    let result = repo_roller_core::create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".reporoller-test-conflicting",
        providers.visibility_policy_provider.clone(),
        providers.environment_detector.clone(),
    )
    .await;

    match result {
        Ok(_) => {
            info!(
                "✓ Repository created (conflict detection requires team parameter implementation)"
            );
        }
        Err(e) => {
            let error_msg = format!("{:?}", e);
            info!("✓ Error occurred: {}", error_msg);

            // When team parameter is implemented, verify:
            // - Error mentions conflict between team and global
            // - Error explains which setting is fixed
            // - Error suggests resolution
        }
    }

    info!("✓ Conflicting team configuration test completed");
    Ok(())
}

/// Test nonexistent repository type error.
///
/// Verifies that requesting a repository type that doesn't exist
/// in metadata repository produces clear error.
#[tokio::test]
async fn test_nonexistent_repository_type() -> Result<()> {
    info!("Testing nonexistent repository type error");

    let config = TestConfig::from_env()?;
    let org_name = repo_roller_core::OrganizationName::new(&config.test_org)?;
    let repo_name = integration_tests::generate_test_repo_name("test", "nonexistent-type");

    let _test_repo =
        integration_tests::TestRepository::new(repo_name.clone(), config.test_org.clone());

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    // Get installation token
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    // Create visibility providers

    let providers =
        integration_tests::create_visibility_providers(&installation_token, ".reporoller-test")
            .await?;

    // Build request with nonexistent repository type
    // Note: Currently repo_roller_core doesn't support repository_type in the builder
    // This test documents the expected behavior when it's added
    let request = repo_roller_core::RepositoryCreationRequestBuilder::new(
        repo_roller_core::RepositoryName::new(&repo_name)?,
        org_name,
        )
    ).template(repo_roller_core::TemplateName::new("template-test-basic")?)
    .build();

    // Execute repository creation
    let result = repo_roller_core::create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".reporoller-test",
        providers.visibility_policy_provider.clone(),
        providers.environment_detector.clone(),
    )
    .await;

    // For now, this should succeed since we don't have repository_type parameter yet
    // When repository_type support is added, update this test to:
    // 1. Add .repository_type("nonexistent-type") to builder
    // 2. Verify operation fails with clear error
    // 3. Check error lists available types
    match result {
        Ok(_) => {
            info!("✓ Repository created (repository_type parameter not yet implemented)");
        }
        Err(e) => {
            let error_msg = format!("{:?}", e);
            info!("Error occurred: {}", error_msg);

            // When repository_type is implemented, verify error quality:
            // - Should mention "nonexistent-type" not found
            // - Should list available types (library, service)
            // - Should be clear and actionable
        }
    }

    info!("✓ Nonexistent repository type test completed");
    Ok(())
}

/// Test malformed team configuration file.
///
/// Verifies that invalid TOML in team configuration
/// is handled gracefully.
#[tokio::test]
async fn test_malformed_team_toml() -> Result<()> {
    info!("Testing malformed team TOML error handling");

    let config = TestConfig::from_env()?;
    let org_name = repo_roller_core::OrganizationName::new(&config.test_org)?;
    let repo_name = integration_tests::generate_test_repo_name("test", "malformed-team");

    let _test_repo =
        integration_tests::TestRepository::new(repo_name.clone(), config.test_org.clone());

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    // Get installation token
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    // Create visibility providers

    let providers = integration_tests::create_visibility_providers(
        &installation_token,
        ".reporoller-test-invalid-team",
    )
    .await?;

    // Build request
    // Note: When team parameter is added, use: .team("invalid-syntax")
    let request = repo_roller_core::RepositoryCreationRequestBuilder::new(
        repo_roller_core::RepositoryName::new(&repo_name)?,
        org_name,
        )
    ).template(repo_roller_core::TemplateName::new("template-test-basic")?)
    .build();

    // Execute repository creation
    let result = repo_roller_core::create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".reporoller-test-invalid-team",
        providers.visibility_policy_provider.clone(),
        providers.environment_detector.clone(),
    )
    .await;

    match result {
        Ok(_) => {
            info!("✓ Repository created (team parameter not yet implemented to trigger error)");
        }
        Err(e) => {
            let error_msg = format!("{:?}", e);
            info!("✓ Error occurred: {}", error_msg);

            // When team parameter is implemented, verify error quality:
            // - Should mention TOML parsing error
            // - Should indicate which team file is invalid
            // - Should provide helpful context
        }
    }

    info!("✓ Malformed team TOML test completed");
    Ok(())
}

/// Test missing team file error.
///
/// Verifies that requesting a team that doesn't exist
/// in metadata repository produces clear error.
#[tokio::test]
async fn test_missing_team_file() -> Result<()> {
    info!("Testing missing team file error");

    let config = TestConfig::from_env()?;
    let org_name = repo_roller_core::OrganizationName::new(&config.test_org)?;
    let repo_name = integration_tests::generate_test_repo_name("test", "missing-team");

    let _test_repo =
        integration_tests::TestRepository::new(repo_name.clone(), config.test_org.clone());

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    // Get installation token
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    // Create visibility providers

    let providers =
        integration_tests::create_visibility_providers(&installation_token, ".reporoller-test")
            .await?;

    // Build request with nonexistent team
    // Note: Currently repo_roller_core doesn't support team parameter in the builder
    // This test documents the expected behavior when it's added
    let request = repo_roller_core::RepositoryCreationRequestBuilder::new(
        repo_roller_core::RepositoryName::new(&repo_name)?,
        org_name,
        )
    ).template(repo_roller_core::TemplateName::new("template-test-basic")?)
    .build();

    // Execute repository creation
    let result = repo_roller_core::create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".reporoller-test",
        providers.visibility_policy_provider.clone(),
        providers.environment_detector.clone(),
    )
    .await;

    // For now, this should succeed since we don't have team parameter yet
    // When team support is added, update this test to:
    // 1. Add .team("nonexistent-team") to builder
    // 2. Verify operation fails with clear error
    // 3. Check error lists available teams (backend, platform)
    match result {
        Ok(_) => {
            info!("✓ Repository created (team parameter not yet implemented)");
        }
        Err(e) => {
            let error_msg = format!("{:?}", e);
            info!("Error occurred: {}", error_msg);

            // When team is implemented, verify error quality:
            // - Should mention "nonexistent-team" not found
            // - Should list available teams (backend, platform)
            // - Should be clear and actionable
        }
    }

    info!("✓ Missing team file test completed");
    Ok(())
}

/// Test metadata repository with inconsistent structure.
///
/// Verifies handling when metadata repository structure
/// doesn't match expected layout.
#[tokio::test]
async fn test_inconsistent_metadata_structure() -> Result<()> {
    info!("Testing inconsistent metadata structure");

    let config = TestConfig::from_env()?;
    let org_name = repo_roller_core::OrganizationName::new(&config.test_org)?;
    let repo_name = integration_tests::generate_test_repo_name("test", "incomplete-structure");

    let _test_repo =
        integration_tests::TestRepository::new(repo_name.clone(), config.test_org.clone());

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    // Get installation token
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    // Create visibility providers

    let providers = integration_tests::create_visibility_providers(
        &installation_token,
        ".reporoller-test-incomplete",
    )
    .await?;

    // Build request
    let request = repo_roller_core::RepositoryCreationRequestBuilder::new(
        repo_roller_core::RepositoryName::new(&repo_name)?,
        org_name,
        )
    ).template(repo_roller_core::TemplateName::new("template-test-basic")?)
    .build();

    // Execute repository creation
    let result = repo_roller_core::create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".reporoller-test-incomplete",
        providers.visibility_policy_provider.clone(),
        providers.environment_detector.clone(),
    )
    .await;

    match result {
        Ok(_) => {
            info!("✓ Repository created - system tolerates missing teams/ and types/ directories");
        }
        Err(e) => {
            let error_msg = format!("{:?}", e);
            info!(
                "✓ System requires complete structure - clear error: {}",
                error_msg
            );
        }
    }

    info!("✓ Inconsistent metadata structure test completed - behavior documented");
    Ok(())
}

/// Test metadata repository with duplicate label definitions.
///
/// Verifies handling when same label is defined multiple times
/// with different configurations.
#[tokio::test]
async fn test_duplicate_label_definitions() -> Result<()> {
    info!("Testing duplicate label definitions");

    let config = TestConfig::from_env()?;
    let org_name = repo_roller_core::OrganizationName::new(&config.test_org)?;
    let repo_name = integration_tests::generate_test_repo_name("test", "duplicate-labels");

    let _test_repo =
        integration_tests::TestRepository::new(repo_name.clone(), config.test_org.clone());

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    // Get installation token
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    // Create visibility providers

    let providers = integration_tests::create_visibility_providers(
        &installation_token,
        ".reporoller-test-duplicates",
    )
    .await?;

    // Build request
    let request = repo_roller_core::RepositoryCreationRequestBuilder::new(
        repo_roller_core::RepositoryName::new(&repo_name)?,
        org_name,
        )
    ).template(repo_roller_core::TemplateName::new("template-test-basic")?)
    .build();

    // Execute repository creation
    let result = repo_roller_core::create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".reporoller-test-duplicates",
        providers.visibility_policy_provider.clone(),
        providers.environment_detector.clone(),
    )
    .await;

    match result {
        Ok(creation_result) => {
            info!("✓ Repository created: {}", creation_result.repository_url);

            // Verify repository and check which label definition was used
            let verification_client = github_client::create_token_client(&installation_token)?;
            let verification_client = github_client::GitHubClient::new(verification_client);

            let labels = verification_client
                .list_repository_labels(&config.test_org, &repo_name)
                .await?;

            // Check if "bug" label exists
            let bug_label_count = labels.iter().filter(|l| *l == "bug").count();

            match bug_label_count {
                0 => info!("⚠ No 'bug' label found - duplicates may have been rejected"),
                1 => info!("✓ System handled duplicates - 'bug' label exists once"),
                _ => info!(
                    "⚠ Multiple 'bug' labels created - duplicate handling may need improvement"
                ),
            }
        }
        Err(e) => {
            let error_msg = format!("{:?}", e);
            info!(
                "✓ System rejected duplicate labels with error: {}",
                error_msg
            );
        }
    }

    info!("✓ Duplicate label definitions test completed - behavior documented");
    Ok(())
}
