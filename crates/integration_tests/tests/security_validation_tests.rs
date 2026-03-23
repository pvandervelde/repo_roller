//! Security and input validation integration tests.
//!
//! These tests verify that the system properly handles malicious or malformed inputs
//! and protects against common security vulnerabilities.

use anyhow::Result;
use auth_handler::UserAuthenticationService;
use repo_roller_core::{
    OrganizationName, RepositoryCreationRequestBuilder, RepositoryName, TemplateName,
};
use std::collections::HashMap;
use std::path::Path;
use tracing::info;

/// Test repository name security validation.
///
/// Verifies that repository names with potentially dangerous patterns
/// are rejected before reaching GitHub API.
#[tokio::test]
async fn test_repository_name_injection_protection() -> Result<()> {
    info!("Testing repository name injection protection");

    // Test: SQL injection patterns
    let sql_injection_names = vec![
        "test'; DROP TABLE repos;--",
        "test' OR '1'='1",
        "test\"; DELETE FROM users;--",
    ];

    for name in sql_injection_names {
        let result = RepositoryName::new(name);
        assert!(
            result.is_err(),
            "SQL injection pattern should be rejected: {}",
            name
        );
    }

    // Test: Shell metacharacters
    let shell_injection_names = vec![
        "test; rm -rf /",
        "test`whoami`",
        "test$(cat /etc/passwd)",
        "test && malicious-command",
    ];

    for name in shell_injection_names {
        let result = RepositoryName::new(name);
        assert!(
            result.is_err(),
            "Shell injection pattern should be rejected: {}",
            name
        );
    }

    // Test: Path traversal attempts
    let path_traversal_names = vec!["../../../etc/passwd", "test/../../../home", "..\\..\\..\\"];

    for name in path_traversal_names {
        let result = RepositoryName::new(name);
        assert!(
            result.is_err(),
            "Path traversal pattern should be rejected: {}",
            name
        );
    }

    // Test: Null bytes
    let null_byte_names = vec!["test\0malicious", "repo\x00name"];

    for name in null_byte_names {
        let result = RepositoryName::new(name);
        assert!(
            result.is_err(),
            "Null byte pattern should be rejected: {}",
            name
        );
    }

    // Test: Very long names (GitHub limit is 100 characters)
    let very_long_name = "a".repeat(101);
    let result = RepositoryName::new(&very_long_name);
    assert!(result.is_err(), "Very long name should be rejected");

    // Test: Unicode homoglyphs (could be used for phishing)
    let homoglyph_names = vec![
        "аdmin", // Cyrillic 'а' looks like Latin 'a'
        "teѕt",  // Cyrillic 'ѕ' looks like Latin 's'
    ];

    for name in homoglyph_names {
        let result = RepositoryName::new(name);
        // Note: These might be valid per GitHub's rules, but we should log
        // and potentially warn about them
        info!("Unicode homoglyph test: {} -> {:?}", name, result);
    }

    info!("✓ Repository name injection protection tests passed");
    Ok(())
}

/// Test organization name security validation.
///
/// Verifies that organization names are properly validated against
/// malicious patterns.
#[tokio::test]
async fn test_organization_name_validation_edge_cases() -> Result<()> {
    info!("Testing organization name validation edge cases");

    // Test: Empty organization name
    let result = OrganizationName::new("");
    assert!(
        result.is_err(),
        "Empty organization name should be rejected"
    );

    // Test: Only special characters
    let special_char_names = vec!["!!!", "@@@", "###", "***"];
    for name in special_char_names {
        let result = OrganizationName::new(name);
        assert!(
            result.is_err(),
            "Organization name with only special characters should be rejected: {}",
            name
        );
    }

    // Test: Organization name with spaces
    let spaced_names = vec!["my org", "test organization", "a b c"];
    for name in spaced_names {
        let result = OrganizationName::new(name);
        assert!(
            result.is_err(),
            "Organization name with spaces should be rejected: {}",
            name
        );
    }

    // Test: Very long organization names
    let very_long_org = "organization_".repeat(10);
    let result = OrganizationName::new(&very_long_org);
    assert!(
        result.is_err(),
        "Very long organization name should be rejected"
    );

    // Test: Organization name that looks like URL
    let url_like_names = vec!["http://evil.com", "https://phishing.com", "ftp://bad.com"];
    for name in url_like_names {
        let result = OrganizationName::new(name);
        assert!(
            result.is_err(),
            "URL-like organization name should be rejected: {}",
            name
        );
    }

    info!("✓ Organization name validation edge case tests passed");
    Ok(())
}

/// Test template name security validation.
///
/// Verifies that template names are properly validated to prevent
/// malicious template references.
#[tokio::test]
async fn test_template_name_validation_security() -> Result<()> {
    info!("Testing template name validation security");

    // Test: Path traversal in template names
    let path_traversal_templates = vec!["../../etc/passwd", "../../../secret", "..\\..\\..\\"];

    for name in path_traversal_templates {
        let result = TemplateName::new(name);
        assert!(
            result.is_err(),
            "Path traversal in template name should be rejected: {}",
            name
        );
    }

    // Test: Absolute paths
    let absolute_paths = vec!["/etc/passwd", "/root/.ssh/id_rsa", "C:\\Windows\\System32"];

    for name in absolute_paths {
        let result = TemplateName::new(name);
        assert!(
            result.is_err(),
            "Absolute path in template name should be rejected: {}",
            name
        );
    }

    // Test: Shell metacharacters
    let shell_templates = vec!["template`whoami`", "test$(evil)", "template;rm -rf /"];

    for name in shell_templates {
        let result = TemplateName::new(name);
        assert!(
            result.is_err(),
            "Shell metacharacters in template name should be rejected: {}",
            name
        );
    }

    info!("✓ Template name validation security tests passed");
    Ok(())
}

/// Test that variable values don't allow code injection.
///
/// Verifies that template variable values are stored and rendered literally —
/// they are not executed as shell commands, not re-evaluated as template syntax,
/// and not interpreted as HTML.
#[tokio::test]
async fn test_template_variable_injection_protection() -> Result<()> {
    info!("Testing template variable injection protection");

    use template_engine::{TemplateProcessingRequest, TemplateProcessor};

    let processor = TemplateProcessor::new().expect("Failed to create template processor");

    // --- Template syntax must not be double-expanded --------------------------
    // If a variable value contains Handlebars syntax such as {{other}}, it must
    // appear literally in the rendered output and not trigger a second round of
    // template expansion.
    let template_injection_values = vec!["{{nested}}", "{% code %}", "${variable}"];

    for value in template_injection_values {
        let mut variables = HashMap::new();
        variables.insert("description".to_string(), value.to_string());

        let files = vec![(
            "README.md".to_string(),
            "# Description: {{description}}".to_string().into_bytes(),
        )];

        let request = TemplateProcessingRequest {
            variables,
            built_in_variables: HashMap::new(),
            variable_configs: HashMap::new(),
            templating_config: None,
        };

        let result = processor.process_template(&files, &request, Path::new("."));
        assert!(
            result.is_ok(),
            "Template processing should succeed for value: {}",
            value
        );

        let processed = result.unwrap();
        let readme = processed
            .files
            .iter()
            .find(|(path, _)| path == "README.md")
            .map(|(_, content)| String::from_utf8_lossy(content).into_owned())
            .expect("README.md should be present in output");

        // The variable value must appear literally — not expanded again
        assert!(
            readme.contains(value),
            "Output must contain the literal value '{}', got: {}",
            value,
            readme
        );
    }

    // --- Shell command syntax in variables must be preserved literally --------
    // Shell metacharacters inside variable values must never be executed.
    // The template engine writes files; it is not a shell interpreter. These
    // values should pass through unchanged.
    let shell_injection_values = vec![
        "`rm -rf /`",
        "$(malicious command)",
        "; malicious-command",
        "test && echo pwned",
    ];

    for value in shell_injection_values {
        let mut variables = HashMap::new();
        variables.insert("command".to_string(), value.to_string());

        let files = vec![(
            "script.sh".to_string(),
            // Triple-brace {{{command}}} disables HTML escaping so the raw value
            // passes through verbatim. The security property (no shell execution)
            // is guaranteed by the Handlebars processing model, not by encoding.
            b"#!/bin/bash\n# Command: {{{command}}}".to_vec(),
        )];

        let request = TemplateProcessingRequest {
            variables,
            built_in_variables: HashMap::new(),
            variable_configs: HashMap::new(),
            templating_config: None,
        };

        let result = processor.process_template(&files, &request, Path::new("."));
        assert!(
            result.is_ok(),
            "Template processing should succeed for shell value: {}",
            value
        );

        let processed = result.unwrap();
        let script = processed
            .files
            .iter()
            .find(|(path, _)| path == "script.sh")
            .map(|(_, content)| String::from_utf8_lossy(content).into_owned())
            .expect("script.sh should be present in output");

        // The value must appear literally — not interpreted as a shell command
        assert!(
            script.contains(value),
            "Output must contain the literal shell value '{}', got: {}",
            value,
            script
        );
    }

    // --- HTML/XSS payloads in variables must be HTML-encoded in rendered output ----
    // Handlebars double-brace {{variable}} HTML-encodes output, so characters such
    // as `<` and `>` become `&lt;` and `&gt;`. This means injected script tags
    // will NOT execute when the output is used in an HTML context.
    // We verify both that (a) the rendered output does NOT contain the raw `<script>`
    // tag, and (b) the HTML-encoded form IS present, confirming encoding is active.
    let xss_values = vec![
        (
            "<script>alert('XSS')</script>",
            "&lt;script&gt;",
            "<script>",
        ),
        (
            "<img src=x onerror=alert('XSS')>",
            "&lt;img src=x",
            "<img src=x",
        ),
        ("'; DROP TABLE users; --", "&#x27;; DROP TABLE", "'; DROP"),
    ];

    for (value, expected_encoded, not_expected_raw) in xss_values {
        let mut variables = HashMap::new();
        variables.insert("description".to_string(), value.to_string());

        let files = vec![(
            "README.md".to_string(),
            "# Description: {{description}}".to_string().into_bytes(),
        )];

        let request = TemplateProcessingRequest {
            variables,
            built_in_variables: HashMap::new(),
            variable_configs: HashMap::new(),
            templating_config: None,
        };

        let result = processor.process_template(&files, &request, Path::new("."));
        assert!(
            result.is_ok(),
            "Template processing should succeed for XSS value: {}",
            value
        );

        let processed = result.unwrap();
        let readme = processed
            .files
            .iter()
            .find(|(path, _)| path == "README.md")
            .map(|(_, content)| String::from_utf8_lossy(content).into_owned())
            .expect("README.md should be present in output");

        // The raw dangerous string must NOT appear in output
        assert!(
            !readme.contains(not_expected_raw),
            "Output must NOT contain raw value '{}' — Handlebars should have HTML-encoded it. Got: {}",
            not_expected_raw,
            readme
        );
        // The HTML-encoded form MUST appear, confirming encoding is active
        assert!(
            readme.contains(expected_encoded),
            "Output must contain HTML-encoded form '{}'. Got: {}",
            expected_encoded,
            readme
        );
    }

    // --- Request builder accepts injection-like values without panicking ------
    // (Separate from rendering — verifies the value is stored in the domain
    // object verbatim before any template processing occurs)
    let risky_values = vec![
        "<script>alert('XSS')</script>",
        "<img src=x onerror=alert('XSS')>",
        "'; DROP TABLE users; --",
    ];

    for value in risky_values {
        let result = RepositoryCreationRequestBuilder::new(
            RepositoryName::new("test-repo")?,
            OrganizationName::new("test-org")?,
        )
        .template(TemplateName::new("test-template")?)
        .variable("description", value)
        .build();

        assert!(
            result.variables.contains_key("description"),
            "Variable should be stored"
        );
        assert_eq!(
            result.variables["description"], value,
            "Variable value must be preserved verbatim in the request"
        );
    }

    info!("✓ Template variable injection protection tests passed");
    Ok(())
}

/// Test that secrets are never exposed in error messages.
///
/// Verifies that when errors occur involving sensitive data,
/// the error messages don't leak credentials or private keys.
#[tokio::test]
async fn test_secrets_not_exposed_in_errors() -> Result<()> {
    info!("Testing that secrets are not exposed in errors");

    // Test: Invalid GitHub App private key
    let fake_private_key =
        "-----BEGIN RSA PRIVATE KEY-----\nFAKE_KEY_DATA\n-----END RSA PRIVATE KEY-----";

    let auth_service = auth_handler::GitHubAuthService::new(12345, fake_private_key.to_string());

    // Try to get token with invalid key - should fail
    let result = auth_service
        .get_installation_token_for_org("test-org")
        .await;

    assert!(result.is_err(), "Should fail with invalid private key");

    // Verify error message doesn't contain the private key
    let error_msg = result.unwrap_err().to_string();
    assert!(
        !error_msg.contains("FAKE_KEY_DATA"),
        "Error message should not contain private key data"
    );
    assert!(
        !error_msg.contains("BEGIN RSA PRIVATE KEY"),
        "Error message should not contain key markers"
    );

    // Verify error message is helpful but doesn't leak secrets
    assert!(
        error_msg.to_lowercase().contains("authentication")
            || error_msg.to_lowercase().contains("credential")
            || error_msg.to_lowercase().contains("token"),
        "Error message should indicate authentication issue"
    );

    info!("✓ Secrets exposure in errors tests passed");
    Ok(())
}

/// Test very long input validation.
///
/// Verifies that extremely long inputs are properly rejected
/// to prevent buffer overflow, DoS, or database issues.
#[tokio::test]
async fn test_very_long_input_validation() -> Result<()> {
    info!("Testing very long input validation");

    // Test: Very long repository name
    let very_long_name = "repository_".repeat(100); // 1100 characters
    let result = RepositoryName::new(&very_long_name);
    assert!(
        result.is_err(),
        "Very long repository name should be rejected"
    );

    // Test: Very long organization name
    let very_long_org = "organization_".repeat(100);
    let result = OrganizationName::new(&very_long_org);
    assert!(
        result.is_err(),
        "Very long organization name should be rejected"
    );

    // Test: Very long template name
    let very_long_template = "template_".repeat(100);
    let result = TemplateName::new(&very_long_template);
    assert!(
        result.is_err(),
        "Very long template name should be rejected"
    );

    // Test: Very long variable values (should be allowed but monitored)
    let very_long_value = "value".repeat(1000); // 5000 characters
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("test-repo")?,
        OrganizationName::new("test-org")?,
    )
    .template(TemplateName::new("test-template")?)
    .variable("long_description", &very_long_value)
    .build();

    assert!(
        request.variables.contains_key("long_description"),
        "Long variable values should be accepted (but may be truncated by template)"
    );

    info!("✓ Very long input validation tests passed");
    Ok(())
}
