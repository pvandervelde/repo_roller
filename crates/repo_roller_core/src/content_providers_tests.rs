// Tests for content_providers module
// See specs/interfaces/content-providers.md for complete specification

use super::*;
use crate::{ContentStrategy, OrganizationName, RepositoryName, TemplateName};
use std::collections::HashMap;

// Mock TemplateFetcher for testing
struct MockTemplateFetcher {
    should_succeed: bool,
}

#[async_trait::async_trait]
impl template_engine::TemplateFetcher for MockTemplateFetcher {
    async fn fetch_template_files(&self, _source: &str) -> Result<Vec<(String, Vec<u8>)>, String> {
        if self.should_succeed {
            Ok(vec![])
        } else {
            Err("Mock fetch failed".to_string())
        }
    }
}

// Helper function to create a minimal test request
fn create_test_request() -> RepositoryCreationRequest {
    RepositoryCreationRequest {
        name: RepositoryName::new("test-repo").unwrap(),
        owner: OrganizationName::new("test-org").unwrap(),
        template: Some(TemplateName::new("test-template").unwrap()),
        variables: HashMap::new(),
        visibility: None,
        content_strategy: ContentStrategy::Template,
    }
}

// Helper function to create a minimal test template config
fn create_test_template_config() -> config_manager::TemplateConfig {
    config_manager::TemplateConfig {
        template: config_manager::TemplateMetadata {
            name: "test-template".to_string(),
            description: "Test template".to_string(),
            author: "Test Author".to_string(),
            tags: vec![],
        },
        repository: None,
        repository_type: None,
        pull_requests: None,
        branch_protection: None,
        labels: None,
        webhooks: None,
        environments: None,
        github_apps: None,
        rulesets: None,
        variables: None,
        default_visibility: None,
        templating: None,
        notifications: None,
    }
}

// Helper function to create a minimal merged config
fn create_test_merged_config() -> config_manager::MergedConfiguration {
    config_manager::MergedConfiguration::new()
}

// ============================================================================
// TemplateBasedContentProvider Tests
// ============================================================================

/// Test that TemplateBasedContentProvider requires template_config.
///
/// Assertion: ValidationError if template_config is None.
#[tokio::test]
async fn test_template_provider_requires_template_config() {
    let fetcher = MockTemplateFetcher {
        should_succeed: true,
    };
    let provider = TemplateBasedContentProvider::new(&fetcher);

    let request = create_test_request();
    let merged_config = create_test_merged_config();

    let result = provider
        .provide_content(&request, None, "org/template", &merged_config)
        .await;

    assert!(result.is_err());
    match result {
        Err(RepoRollerError::System(SystemError::Internal { reason })) => {
            assert!(reason.contains("requires template configuration"));
        }
        _ => panic!("Expected SystemError::Internal"),
    }
}

/// Test that TemplateBasedContentProvider requires non-empty template_source.
///
/// Assertion: ValidationError if template_source is empty.
#[tokio::test]
async fn test_template_provider_requires_template_source() {
    let fetcher = MockTemplateFetcher {
        should_succeed: true,
    };
    let provider = TemplateBasedContentProvider::new(&fetcher);

    let request = create_test_request();
    let template = create_test_template_config();
    let merged_config = create_test_merged_config();

    let result = provider
        .provide_content(&request, Some(&template), "", &merged_config)
        .await;

    assert!(result.is_err());
    match result {
        Err(RepoRollerError::System(SystemError::Internal { reason })) => {
            assert!(reason.contains("Template source must be provided"));
        }
        _ => panic!("Expected SystemError::Internal"),
    }
}

// ============================================================================
// ZeroContentProvider Tests
// ============================================================================

/// Test that ZeroContentProvider creates empty directory.
///
/// Assertion: Returns empty TempDir successfully.
#[tokio::test]
async fn test_zero_provider_creates_empty_dir() {
    let provider = ZeroContentProvider::new();

    let request = create_test_request();
    let merged_config = create_test_merged_config();

    let result = provider
        .provide_content(&request, None, "", &merged_config)
        .await;

    assert!(result.is_ok());
    let temp_dir = result.unwrap();

    // Verify directory exists and is empty
    assert!(temp_dir.path().exists());
    let entries: Vec<_> = std::fs::read_dir(temp_dir.path())
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(entries.len(), 0, "Expected empty directory");
}

/// Test that ZeroContentProvider works with None template_config.
///
/// Assertion: Accepts None template_config (ignores it).
#[tokio::test]
async fn test_zero_provider_accepts_none_template() {
    let provider = ZeroContentProvider::new();

    let request = create_test_request();
    let merged_config = create_test_merged_config();

    let result = provider
        .provide_content(&request, None, "", &merged_config)
        .await;

    assert!(result.is_ok());
}

/// Test that ZeroContentProvider works with Some template_config.
///
/// Assertion: Accepts Some template_config but ignores it.
#[tokio::test]
async fn test_zero_provider_accepts_some_template() {
    let provider = ZeroContentProvider::new();

    let request = create_test_request();
    let template = create_test_template_config();
    let merged_config = create_test_merged_config();

    let result = provider
        .provide_content(&request, Some(&template), "org/template", &merged_config)
        .await;

    assert!(result.is_ok());
    let temp_dir = result.unwrap();

    // Verify directory exists and is empty
    assert!(temp_dir.path().exists());
    let entries: Vec<_> = std::fs::read_dir(temp_dir.path())
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(entries.len(), 0, "Expected empty directory");
}

/// Test that ZeroContentProvider::default() works.
///
/// Assertion: Default implementation creates valid provider.
#[tokio::test]
async fn test_zero_provider_default() {
    let provider = ZeroContentProvider;

    let request = create_test_request();
    let merged_config = create_test_merged_config();

    let result = provider
        .provide_content(&request, None, "", &merged_config)
        .await;

    assert!(result.is_ok());
}

// ============================================================================
// CustomInitContentProvider Tests
// ============================================================================

/// Test that CustomInitContentProvider with include_readme=true creates README.md.
///
/// Assertion: README.md file exists with correct content.
#[tokio::test]
async fn test_custom_init_creates_readme() {
    let options = CustomInitOptions {
        include_readme: true,
        include_gitignore: false,
    };
    let provider = CustomInitContentProvider::new(options);

    let request = create_test_request();
    let merged_config = create_test_merged_config();

    let result = provider
        .provide_content(&request, None, "", &merged_config)
        .await;

    assert!(result.is_ok());
    let temp_dir = result.unwrap();

    // Verify README.md exists
    let readme_path = temp_dir.path().join("README.md");
    assert!(readme_path.exists(), "README.md should exist");

    // Verify README content includes repository name and organization
    let content = std::fs::read_to_string(&readme_path).unwrap();
    assert!(
        content.contains("test-repo"),
        "README should contain repo name"
    );
    assert!(
        content.contains("test-org"),
        "README should contain organization"
    );
}

/// Test that CustomInitContentProvider with include_gitignore=true creates .gitignore.
///
/// Assertion: .gitignore file exists with common patterns.
#[tokio::test]
async fn test_custom_init_creates_gitignore() {
    let options = CustomInitOptions {
        include_readme: false,
        include_gitignore: true,
    };
    let provider = CustomInitContentProvider::new(options);

    let request = create_test_request();
    let merged_config = create_test_merged_config();

    let result = provider
        .provide_content(&request, None, "", &merged_config)
        .await;

    assert!(result.is_ok());
    let temp_dir = result.unwrap();

    // Verify .gitignore exists
    let gitignore_path = temp_dir.path().join(".gitignore");
    assert!(gitignore_path.exists(), ".gitignore should exist");

    // Verify .gitignore content includes common patterns
    let content = std::fs::read_to_string(&gitignore_path).unwrap();
    assert!(content.contains(".DS_Store"), "Should contain OS patterns");
    assert!(
        content.contains("node_modules/"),
        "Should contain dependency patterns"
    );
    assert!(content.contains("target/"), "Should contain build patterns");
}

/// Test that CustomInitContentProvider with both options creates both files.
///
/// Assertion: Both README.md and .gitignore exist.
#[tokio::test]
async fn test_custom_init_creates_both_files() {
    let options = CustomInitOptions {
        include_readme: true,
        include_gitignore: true,
    };
    let provider = CustomInitContentProvider::new(options);

    let request = create_test_request();
    let merged_config = create_test_merged_config();

    let result = provider
        .provide_content(&request, None, "", &merged_config)
        .await;

    assert!(result.is_ok());
    let temp_dir = result.unwrap();

    // Verify both files exist
    assert!(
        temp_dir.path().join("README.md").exists(),
        "README.md should exist"
    );
    assert!(
        temp_dir.path().join(".gitignore").exists(),
        ".gitignore should exist"
    );

    // Verify directory has exactly 2 entries
    let entries: Vec<_> = std::fs::read_dir(temp_dir.path())
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(entries.len(), 2, "Expected exactly 2 files");
}

/// Test that CustomInitContentProvider with both false creates empty directory.
///
/// Assertion: Valid to have both false (empty directory).
#[tokio::test]
async fn test_custom_init_both_false_creates_empty() {
    let options = CustomInitOptions {
        include_readme: false,
        include_gitignore: false,
    };
    let provider = CustomInitContentProvider::new(options);

    let request = create_test_request();
    let merged_config = create_test_merged_config();

    let result = provider
        .provide_content(&request, None, "", &merged_config)
        .await;

    assert!(result.is_ok());
    let temp_dir = result.unwrap();

    // Verify directory is empty
    let entries: Vec<_> = std::fs::read_dir(temp_dir.path())
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(entries.len(), 0, "Expected empty directory");
}

/// Test README.md content with template information.
///
/// Assertion: README includes template name when template provided.
#[tokio::test]
async fn test_readme_content_with_template() {
    let options = CustomInitOptions {
        include_readme: true,
        include_gitignore: false,
    };
    let provider = CustomInitContentProvider::new(options);

    let request = create_test_request();
    let template = create_test_template_config();
    let merged_config = create_test_merged_config();

    let result = provider
        .provide_content(&request, Some(&template), "org/template", &merged_config)
        .await;

    assert!(result.is_ok());
    let temp_dir = result.unwrap();

    let readme_path = temp_dir.path().join("README.md");
    let content = std::fs::read_to_string(&readme_path).unwrap();

    // Verify template name is mentioned
    assert!(
        content.contains("test-template"),
        "README should mention template name"
    );
    assert!(
        content.contains("RepoRoller"),
        "README should mention RepoRoller"
    );
}

/// Test README.md content without template information.
///
/// Assertion: README works correctly when no template provided.
#[tokio::test]
async fn test_readme_content_without_template() {
    let options = CustomInitOptions {
        include_readme: true,
        include_gitignore: false,
    };
    let provider = CustomInitContentProvider::new(options);

    let mut request = create_test_request();
    request.template = None; // No template
    let merged_config = create_test_merged_config();

    let result = provider
        .provide_content(&request, None, "", &merged_config)
        .await;

    assert!(result.is_ok());
    let temp_dir = result.unwrap();

    let readme_path = temp_dir.path().join("README.md");
    let content = std::fs::read_to_string(&readme_path).unwrap();

    // Verify basic content without template reference
    assert!(content.contains("test-repo"));
    assert!(content.contains("test-org"));
    assert!(content.contains("RepoRoller"));
}
