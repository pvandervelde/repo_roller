//! Tests for GitHub metadata repository provider.

use super::*;

// Note: Full integration tests with GitHubClient require actual GitHub API access
// or a trait-based abstraction for GitHubClient (future enhancement).
// For now, we test the configuration types and document expected behavior.

#[test]
fn test_metadata_provider_config_explicit() {
    let config = MetadataProviderConfig::explicit("org-metadata");

    match config.discovery {
        DiscoveryConfig::RepositoryName(name) => {
            assert_eq!(name, "org-metadata");
        }
        _ => panic!("Expected RepositoryName discovery config"),
    }
}

#[test]
fn test_metadata_provider_config_by_topic() {
    let config = MetadataProviderConfig::by_topic("reporoller-metadata");

    match config.discovery {
        DiscoveryConfig::Topic(topic) => {
            assert_eq!(topic, "reporoller-metadata");
        }
        _ => panic!("Expected Topic discovery config"),
    }
}

#[test]
fn test_metadata_provider_config_clone() {
    let config = MetadataProviderConfig::explicit("org-metadata");
    let cloned = config.clone();

    // Both should have the same discovery config
    match (&config.discovery, &cloned.discovery) {
        (DiscoveryConfig::RepositoryName(name1), DiscoveryConfig::RepositoryName(name2)) => {
            assert_eq!(name1, name2);
        }
        _ => panic!("Expected matching RepositoryName configs"),
    }
}

#[test]
fn test_metadata_provider_config_debug() {
    let config = MetadataProviderConfig::explicit("test-repo");
    let debug_str = format!("{:?}", config);

    assert!(debug_str.contains("MetadataProviderConfig"));
    assert!(debug_str.contains("test-repo"));
}

// Repository structure validation tests

#[tokio::test]
async fn test_validate_structure_valid_repository() {
    use chrono::Utc;

    let metadata_repo = MetadataRepository {
        organization: "valid-org".to_string(),
        repository_name: "org-metadata".to_string(),
        discovery_method: DiscoveryMethod::ConfigurationBased {
            repository_name: "org-metadata".to_string(),
        },
        last_updated: Utc::now(),
    };

    // Create a mock provider (would need actual GitHubClient for full test)
    // For now, we can test the validation logic directly

    // Valid repository with normal names should pass validation
    let result = validate_repository_names(&metadata_repo);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_validate_structure_path_traversal_in_org() {
    use chrono::Utc;

    let metadata_repo = MetadataRepository {
        organization: "../etc/passwd".to_string(),
        repository_name: "org-metadata".to_string(),
        discovery_method: DiscoveryMethod::ConfigurationBased {
            repository_name: "org-metadata".to_string(),
        },
        last_updated: Utc::now(),
    };

    let result = validate_repository_names(&metadata_repo);
    assert!(result.is_err());

    match result.unwrap_err() {
        ConfigurationError::InvalidConfiguration { field, reason } => {
            assert_eq!(field, "organization");
            assert!(reason.contains("invalid characters"));
        }
        _ => panic!("Expected InvalidConfiguration error"),
    }
}

#[tokio::test]
async fn test_validate_structure_path_traversal_in_repo() {
    use chrono::Utc;

    let metadata_repo = MetadataRepository {
        organization: "valid-org".to_string(),
        repository_name: "../secrets".to_string(),
        discovery_method: DiscoveryMethod::ConfigurationBased {
            repository_name: "../secrets".to_string(),
        },
        last_updated: Utc::now(),
    };

    let result = validate_repository_names(&metadata_repo);
    assert!(result.is_err());

    match result.unwrap_err() {
        ConfigurationError::InvalidConfiguration { field, reason } => {
            assert_eq!(field, "repository_name");
            assert!(reason.contains("invalid characters"));
        }
        _ => panic!("Expected InvalidConfiguration error"),
    }
}

#[tokio::test]
async fn test_validate_structure_slash_in_org() {
    use chrono::Utc;

    let metadata_repo = MetadataRepository {
        organization: "org/malicious".to_string(),
        repository_name: "org-metadata".to_string(),
        discovery_method: DiscoveryMethod::ConfigurationBased {
            repository_name: "org-metadata".to_string(),
        },
        last_updated: Utc::now(),
    };

    let result = validate_repository_names(&metadata_repo);
    assert!(result.is_err());

    match result.unwrap_err() {
        ConfigurationError::InvalidConfiguration { field, .. } => {
            assert_eq!(field, "organization");
        }
        _ => panic!("Expected InvalidConfiguration error"),
    }
}

#[tokio::test]
async fn test_validate_structure_slash_in_repo() {
    use chrono::Utc;

    let metadata_repo = MetadataRepository {
        organization: "valid-org".to_string(),
        repository_name: "repo/name".to_string(),
        discovery_method: DiscoveryMethod::ConfigurationBased {
            repository_name: "repo/name".to_string(),
        },
        last_updated: Utc::now(),
    };

    let result = validate_repository_names(&metadata_repo);
    assert!(result.is_err());

    match result.unwrap_err() {
        ConfigurationError::InvalidConfiguration { field, .. } => {
            assert_eq!(field, "repository_name");
        }
        _ => panic!("Expected InvalidConfiguration error"),
    }
}

#[tokio::test]
async fn test_validate_structure_hyphen_allowed() {
    use chrono::Utc;

    let metadata_repo = MetadataRepository {
        organization: "my-org".to_string(),
        repository_name: "org-metadata".to_string(),
        discovery_method: DiscoveryMethod::ConfigurationBased {
            repository_name: "org-metadata".to_string(),
        },
        last_updated: Utc::now(),
    };

    let result = validate_repository_names(&metadata_repo);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_validate_structure_underscore_allowed() {
    use chrono::Utc;

    let metadata_repo = MetadataRepository {
        organization: "my_org".to_string(),
        repository_name: "org_metadata".to_string(),
        discovery_method: DiscoveryMethod::ConfigurationBased {
            repository_name: "org_metadata".to_string(),
        },
        last_updated: Utc::now(),
    };

    let result = validate_repository_names(&metadata_repo);
    assert!(result.is_ok());
}

// Helper function for testing validation logic
fn validate_repository_names(repo: &MetadataRepository) -> ConfigurationResult<()> {
    // Security validation: ensure no path traversal in repository/org names
    if repo.organization.contains("..") || repo.organization.contains('/') {
        return Err(ConfigurationError::InvalidConfiguration {
            field: "organization".to_string(),
            reason: "Organization name contains invalid characters".to_string(),
        });
    }

    if repo.repository_name.contains("..") || repo.repository_name.contains('/') {
        return Err(ConfigurationError::InvalidConfiguration {
            field: "repository_name".to_string(),
            reason: "Repository name contains invalid characters".to_string(),
        });
    }

    Ok(())
}

// ============================================================================
// TOML Parsing Tests
// ============================================================================
// Note: Tests that require GitHub API access are covered by integration tests.
// These unit tests focus on TOML parsing logic that can be tested in isolation.

/// Test parsing standard labels from valid TOML.
///
/// Verifies that label configurations can be successfully parsed from
/// TOML content. Note: The `name` field is populated from the map key
/// by `load_standard_labels()`, so this test focuses on color and description.
#[test]
fn test_load_standard_labels_success() {
    let labels_toml = r#"
[bug]
color = "d73a4a"
description = "Something isn't working"

[enhancement]
color = "a2eeef"
description = "New feature or request"

[documentation]
color = "0075ca"
description = "Improvements or additions to documentation"
"#;

    let mut labels: HashMap<String, LabelConfig> =
        toml::from_str(labels_toml).expect("Should parse valid TOML");

    // Simulate what load_standard_labels() does: populate name from map key
    for (name, label) in labels.iter_mut() {
        label.name = name.clone();
    }

    assert_eq!(labels.len(), 3);

    let bug_label = labels.get("bug").expect("bug label should exist");
    assert_eq!(bug_label.name, "bug");
    assert_eq!(bug_label.color, "d73a4a");
    assert_eq!(bug_label.description, "Something isn't working");

    let enhancement_label = labels
        .get("enhancement")
        .expect("enhancement label should exist");
    assert_eq!(enhancement_label.name, "enhancement");
    assert_eq!(enhancement_label.color, "a2eeef");
    assert_eq!(enhancement_label.description, "New feature or request");

    let doc_label = labels
        .get("documentation")
        .expect("documentation label should exist");
    assert_eq!(doc_label.name, "documentation");
    assert_eq!(doc_label.color, "0075ca");
    assert_eq!(
        doc_label.description,
        "Improvements or additions to documentation"
    );
}

/// Test parsing empty standard labels TOML.
///
/// Verifies that empty TOML content (no labels defined) is valid
/// and returns an empty map.
#[test]
fn test_load_standard_labels_empty() {
    let labels_toml = "";

    let result: Result<HashMap<String, LabelConfig>, toml::de::Error> = toml::from_str(labels_toml);
    assert!(result.is_ok());

    let labels = result.unwrap();
    assert!(labels.is_empty(), "Empty TOML should parse to empty map");
}

/// Test parsing standard labels with invalid TOML syntax.
///
/// Verifies that malformed TOML is properly rejected with a parse error.
#[test]
fn test_load_standard_labels_invalid_toml() {
    let invalid_toml = "[bug\ncolor = not a string";

    let result: Result<HashMap<String, LabelConfig>, toml::de::Error> =
        toml::from_str(invalid_toml);
    assert!(result.is_err(), "Invalid TOML should fail to parse");
}

/// Test parsing standard labels with missing required fields.
///
/// Verifies that label definitions missing required fields are rejected.
#[test]
fn test_load_standard_labels_invalid_structure() {
    // Valid TOML but missing required 'description' field
    let invalid_structure = r#"
[bug]
color = "d73a4a"
"#;

    let result: Result<HashMap<String, LabelConfig>, toml::de::Error> =
        toml::from_str(invalid_structure);
    assert!(result.is_err(), "Should fail with missing required fields");
}

// ============================================================================
// Security remediation: `load_template_configuration` must not leak
// secret-shaped template content via free-text `{:?}` logging.
//
// HIGH-severity Security Reviewer finding: prior to remediation, line 626
// logged `tracing::debug!("Successfully loaded template configuration:
// {:?}", config)`, dumping the entire *untrusted* (org-supplied, from
// `.reporoller/template.toml`) `TemplateConfig` at DEBUG level -- including
// any secret-shaped value a template author (accidentally or maliciously)
// placed in e.g. a variable's `default`. This bypasses key-name-based
// redaction entirely, because the redaction layer (a JSON log-line filter)
// never sees the raw `tracing::debug!` call -- and even if it did, `message`
// is not a sensitive field name.
//
// These tests exercise the real trait method end-to-end (a wiremock-backed
// `GitHubClient`, no mocking of `load_template_configuration` itself) and
// capture actual `tracing` output via a real subscriber, per the task's
// requirement to use "a tracing test-capture subscriber."
// ============================================================================

use base64::{engine::general_purpose, Engine as _};
use std::sync::{Arc, Mutex};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// In-memory `tracing_subscriber::fmt::MakeWriter` that captures every
/// formatted line written to it, for later inspection by tests. Mirrors the
/// `CaptureBuffer` pattern used in
/// `crates/repo_roller_api/src/logging_tests.rs`.
#[derive(Clone, Default)]
struct CaptureBuffer(Arc<Mutex<Vec<u8>>>);

impl CaptureBuffer {
    fn contents(&self) -> String {
        String::from_utf8_lossy(
            &self
                .0
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        )
        .to_string()
    }
}

impl std::io::Write for CaptureBuffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for CaptureBuffer {
    type Writer = CaptureBuffer;
    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

/// Builds the base64-encoded GitHub "contents" API response for a single
/// file, matching the shape `GET /repos/{owner}/{repo}/contents/{path}`
/// returns for a file. Mirrors
/// `crates/repo_roller_api/src/handlers_tests.rs::file_content_response`.
fn file_content_response(org: &str, repo: &str, file_path: &str, toml_content: &str) -> serde_json::Value {
    let encoded = general_purpose::STANDARD.encode(toml_content.as_bytes());
    let api_url = format!("https://api.github.com/repos/{org}/{repo}/contents/{file_path}");
    let html_url = format!("https://github.com/{org}/{repo}/blob/main/{file_path}");
    serde_json::json!({
        "name": file_path.split('/').next_back().unwrap_or(file_path),
        "path": file_path,
        "sha": "abc123def456",
        "size": toml_content.len(),
        "type": "file",
        "content": encoded,
        "encoding": "base64",
        "url": api_url,
        "html_url": html_url,
        "git_url": null,
        "download_url": null,
        "_links": {
            "self": api_url,
            "git": null,
            "html": html_url
        }
    })
}

/// A TOML `template.toml` body whose single variable's `default` is a
/// secret-shaped GitHub-token-prefixed string -- the realistic shape of an
/// (accidental or malicious) secret embedded in org-supplied,
/// explicitly-untrusted template content per
/// `docs/spec/security/threat-model.md`.
fn template_toml_with_secret_shaped_variable_default(secret: &str) -> String {
    format!(
        "[template]\n\
         name = \"rust-library\"\n\
         description = \"Rust library template\"\n\
         author = \"Platform Team\"\n\
         tags = []\n\
         \n\
         [variables.api_token]\n\
         description = \"Default API token for local development\"\n\
         default = \"{secret}\"\n"
    )
}

/// Installs a JSON-formatted `tracing` subscriber (level TRACE, so DEBUG
/// events are captured) as the ambient default for the current scope,
/// writing into an in-memory buffer instead of stdout. The returned guard
/// must be held for the duration of the code whose log output is under
/// test; the returned `CaptureBuffer` can be inspected afterwards.
fn install_capturing_subscriber() -> (tracing::subscriber::DefaultGuard, CaptureBuffer) {
    let buffer = CaptureBuffer::default();
    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_max_level(tracing::Level::TRACE)
        .with_writer(buffer.clone())
        .finish();
    let guard = tracing::subscriber::set_default(subscriber);
    (guard, buffer)
}

/// The concrete regression test for the HIGH-severity finding: a
/// secret-shaped value embedded in an org-supplied template's variable
/// `default` must never appear in captured log output after
/// `load_template_configuration` succeeds.
#[tokio::test]
async fn test_load_template_configuration_never_logs_secret_shaped_variable_default() {
    let org = "testorg";
    let template = "rust-library";
    let secret = "ghp_abcdefghij1234567890ABCDEFGHIJ123456";

    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!(
            "/repos/{org}/{template}/contents/.reporoller/template.toml"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(file_content_response(
            org,
            template,
            ".reporoller/template.toml",
            &template_toml_with_secret_shaped_variable_default(secret),
        )))
        .mount(&mock_server)
        .await;

    let client = github_client::create_github_client("fake-token", Some(&mock_server.uri()))
        .expect("client construction with a syntactically valid mock URL must succeed");
    let provider =
        GitHubMetadataProvider::new(client, MetadataProviderConfig::explicit("org-metadata"));

    let (guard, buffer) = install_capturing_subscriber();
    let result = provider.load_template_configuration(org, template).await;
    drop(guard);

    let config = result.expect("template configuration must load successfully from the mock");
    assert_eq!(
        config
            .variables
            .as_ref()
            .and_then(|vars| vars.get("api_token"))
            .and_then(|v| v.default.as_deref()),
        Some(secret),
        "sanity check: the secret must actually be present in the parsed config \
         (otherwise this test would trivially pass without exercising the fix)"
    );

    let output = buffer.contents();
    assert!(
        !output.contains(secret),
        "a secret-shaped variable default from untrusted template content must \
         never appear in emitted log output; got captured log output: {output}"
    );
}

/// Same scenario as above, but with the secret embedded via a `Bearer
/// <jwt>`-shaped default value rather than a `ghp_`-prefixed token, to
/// guard against a fix that only special-cases the exact reviewer example.
#[tokio::test]
async fn test_load_template_configuration_never_logs_bearer_jwt_shaped_variable_default() {
    let org = "testorg";
    let template = "rust-service";
    let jwt = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
    let secret = format!("Bearer {jwt}");

    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!(
            "/repos/{org}/{template}/contents/.reporoller/template.toml"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(file_content_response(
            org,
            template,
            ".reporoller/template.toml",
            &template_toml_with_secret_shaped_variable_default(&secret),
        )))
        .mount(&mock_server)
        .await;

    let client = github_client::create_github_client("fake-token", Some(&mock_server.uri()))
        .expect("client construction with a syntactically valid mock URL must succeed");
    let provider =
        GitHubMetadataProvider::new(client, MetadataProviderConfig::explicit("org-metadata"));

    let (guard, buffer) = install_capturing_subscriber();
    let result = provider.load_template_configuration(org, template).await;
    drop(guard);

    result.expect("template configuration must load successfully from the mock");

    let output = buffer.contents();
    assert!(
        !output.contains(jwt),
        "a Bearer/JWT-shaped variable default from untrusted template content must \
         never appear in emitted log output; got captured log output: {output}"
    );
}

/// Side-effect verification: the fix must not simply delete the success log
/// statement outright -- some bounded, non-secret confirmation that loading
/// succeeded should still be observable. This guards against a
/// "lazy" fix that satisfies the leak tests above by removing all logging
/// at this call site rather than bounding what is logged.
#[tokio::test]
async fn test_load_template_configuration_still_logs_something_on_success() {
    let org = "testorg";
    let template = "rust-library";
    let secret = "ghp_abcdefghij1234567890ABCDEFGHIJ123456";

    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!(
            "/repos/{org}/{template}/contents/.reporoller/template.toml"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(file_content_response(
            org,
            template,
            ".reporoller/template.toml",
            &template_toml_with_secret_shaped_variable_default(secret),
        )))
        .mount(&mock_server)
        .await;

    let client = github_client::create_github_client("fake-token", Some(&mock_server.uri()))
        .expect("client construction with a syntactically valid mock URL must succeed");
    let provider =
        GitHubMetadataProvider::new(client, MetadataProviderConfig::explicit("org-metadata"));

    let (guard, buffer) = install_capturing_subscriber();
    provider
        .load_template_configuration(org, template)
        .await
        .expect("template configuration must load successfully from the mock");
    drop(guard);

    let output = buffer.contents();
    assert!(
        !output.trim().is_empty(),
        "loading a template configuration should still produce some log output \
         (a bounded summary), not silence -- an empty capture suggests logging \
         was deleted rather than made safe"
    );
}
