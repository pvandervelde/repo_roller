//! Test fixtures for integration tests.
//!
//! This module provides pre-configured test data for metadata repositories,
//! configurations, and other test scenarios. Fixtures ensure consistent test
//! data across multiple test cases.

use serde_json::json;
use std::collections::HashMap;

/// Metadata repository fixtures for testing configuration hierarchy.
pub mod metadata_repository {
    use super::*;

    /// Global defaults configuration with baseline settings.
    ///
    /// This represents the lowest precedence level in the configuration hierarchy.
    pub fn global_defaults() -> serde_json::Value {
        json!({
            "repository": {
                "settings": {
                    "has_issues": true,
                    "has_wiki": false,
                    "has_projects": true,
                    "has_discussions": false,
                    "allow_squash_merge": true,
                    "allow_merge_commit": true,
                    "allow_rebase_merge": false,
                    "delete_branch_on_merge": true
                },
                "security": {
                    "security_advisories": {
                        "value": true,
                        "override_allowed": false
                    }
                }
            },
            "labels": [
                {
                    "name": "bug",
                    "color": "FF0000",
                    "description": "Something isn't working"
                },
                {
                    "name": "enhancement",
                    "color": "00FF00",
                    "description": "New feature or request"
                }
            ]
        })
    }

    /// Team configuration that overrides some global settings.
    pub fn team_config(team_name: &str) -> serde_json::Value {
        match team_name {
            "platform" => json!({
                "repository": {
                    "settings": {
                        "has_wiki": true,  // Override global
                        "has_discussions": true  // Override global
                    }
                },
                "labels": [
                    {
                        "name": "team-platform",
                        "color": "0000FF",
                        "description": "Platform team"
                    }
                ]
            }),
            _ => json!({}),
        }
    }

    /// Repository type configuration.
    pub fn repository_type_config(repo_type: &str) -> serde_json::Value {
        match repo_type {
            "service" => json!({
                "repository": {
                    "settings": {
                        "has_projects": false  // Services don't use project boards
                    }
                },
                "labels": [
                    {
                        "name": "service",
                        "color": "FFFF00",
                        "description": "Service repository"
                    }
                ]
            }),
            "library" => json!({
                "repository": {
                    "settings": {
                        "has_wiki": true  // Libraries have documentation wikis
                    }
                },
                "labels": [
                    {
                        "name": "library",
                        "color": "FF00FF",
                        "description": "Library repository"
                    }
                ]
            }),
            _ => json!({}),
        }
    }

    /// Template configuration with highest precedence.
    pub fn template_config(template_name: &str) -> serde_json::Value {
        match template_name {
            "rust-service" => json!({
                "repository": {
                    "settings": {
                        "has_issues": false  // Override global
                    }
                },
                "labels": [
                    {
                        "name": "rust",
                        "color": "00FFFF",
                        "description": "Rust programming language"
                    }
                ]
            }),
            _ => json!({}),
        }
    }

    /// Configuration with fixed values that cannot be overridden.
    pub fn fixed_value_config() -> serde_json::Value {
        json!({
            "repository": {
                "security": {
                    "security_advisories": {
                        "value": true,
                        "policy": "Fixed"
                    },
                    "vulnerability_alerts": {
                        "value": true,
                        "policy": "Fixed"
                    }
                }
            }
        })
    }

    /// Configuration with override protection disabled.
    pub fn protected_config() -> serde_json::Value {
        json!({
            "repository": {
                "settings": {
                    "has_wiki": {
                        "value": false,
                        "override_allowed": false
                    }
                }
            }
        })
    }

    /// Malformed TOML configuration for error testing.
    pub fn malformed_toml() -> String {
        r#"
[repository.settings]
has_issues = true
has_wiki = false
# Missing closing bracket
has_projects = true
has_discussions
        "#
        .to_string()
    }

    /// Invalid configuration structure.
    pub fn invalid_structure() -> serde_json::Value {
        json!({
            "repository": {
                "settings": {
                    "invalid_field": "this should not exist",
                    "has_issues": "not a boolean"  // Wrong type
                }
            }
        })
    }
}

/// Template variable fixtures for testing substitution.
pub mod template_variables {
    use super::*;

    /// Basic variable set for standard templates.
    pub fn basic_variables() -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("project_name".to_string(), "test-project".to_string());
        vars.insert("author".to_string(), "Test Author".to_string());
        vars.insert(
            "description".to_string(),
            "A test project description".to_string(),
        );
        vars
    }

    /// Variables with special characters.
    pub fn special_character_variables() -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("description".to_string(), "Test: <>\"'&${}[]()".to_string());
        vars.insert("author".to_string(), "O'Brien".to_string());
        vars.insert("company".to_string(), "Smith & Jones".to_string());
        vars
    }

    /// Variables with nested references.
    pub fn nested_variables() -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("first_name".to_string(), "John".to_string());
        vars.insert("last_name".to_string(), "Doe".to_string());
        vars.insert(
            "full_name".to_string(),
            "{{first_name}} {{last_name}}".to_string(),
        );
        vars
    }

    /// Very long variable value for stress testing.
    pub fn long_variable_value() -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("description".to_string(), "a".repeat(10_000));
        vars
    }

    /// Variables with Handlebars syntax (should be treated as literal).
    pub fn handlebars_syntax_variables() -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert(
            "description".to_string(),
            "Use {{variable}} syntax in your code".to_string(),
        );
        vars
    }

    /// Invalid variable names for validation testing.
    pub fn invalid_variable_names() -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("invalid-name".to_string(), "value".to_string());
        vars.insert("invalid.name".to_string(), "value".to_string());
        vars.insert("123_starts_with_number".to_string(), "value".to_string());
        vars
    }
}

/// GitHub API mock response fixtures.
pub mod github_api {
    use super::*;

    /// Rate limit exceeded response.
    pub fn rate_limit_response() -> serde_json::Value {
        json!({
            "message": "API rate limit exceeded",
            "documentation_url": "https://docs.github.com/rest/overview/resources-in-the-rest-api#rate-limiting"
        })
    }

    /// Repository not found response.
    pub fn not_found_response() -> serde_json::Value {
        json!({
            "message": "Not Found",
            "documentation_url": "https://docs.github.com/rest/reference/repos#get-a-repository"
        })
    }

    /// Permission denied response.
    pub fn forbidden_response() -> serde_json::Value {
        json!({
            "message": "Resource not accessible by integration",
            "documentation_url": "https://docs.github.com/rest/reference/repos"
        })
    }

    /// Successful repository creation response.
    pub fn repository_created_response(repo_name: &str, owner: &str) -> serde_json::Value {
        json!({
            "id": 123456789,
            "node_id": "MDEwOlJlcG9zaXRvcnkxMjM0NTY3ODk=",
            "name": repo_name,
            "full_name": format!("{}/{}", owner, repo_name),
            "private": false,
            "owner": {
                "login": owner,
                "id": 987654321,
                "type": "Organization"
            },
            "html_url": format!("https://github.com/{}/{}", owner, repo_name),
            "description": "Test repository",
            "created_at": "2024-01-08T12:00:00Z",
            "updated_at": "2024-01-08T12:00:00Z",
            "pushed_at": "2024-01-08T12:00:00Z",
            "default_branch": "main"
        })
    }

    /// Unexpected API response format for error handling.
    pub fn unexpected_response_format() -> serde_json::Value {
        json!({
            "unexpected_field": "this shouldn't be here",
            "missing_required_fields": true
        })
    }
}

/// Error message fixtures for validation testing.
pub mod error_messages {
    /// Expected error messages for different scenarios.
    pub fn expected_messages() -> Vec<(&'static str, Vec<&'static str>)> {
        vec![
            (
                "invalid_repository_name",
                vec!["repository name", "invalid", "format"],
            ),
            (
                "template_not_found",
                vec!["template", "not found", "does not exist"],
            ),
            (
                "authentication_failed",
                vec!["authentication", "credential", "token"],
            ),
            (
                "rate_limit_exceeded",
                vec!["rate limit", "exceeded", "wait"],
            ),
            ("permission_denied", vec!["permission", "access", "denied"]),
        ]
    }

    /// Messages that should never appear in errors (sensitive data).
    pub fn forbidden_patterns() -> Vec<&'static str> {
        vec![
            "BEGIN RSA PRIVATE KEY",
            "END RSA PRIVATE KEY",
            "password",
            "secret",
            "token=",
            "bearer ",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_defaults_has_required_fields() {
        let config = metadata_repository::global_defaults();
        assert!(config["repository"]["settings"]["has_issues"].is_boolean());
        assert!(config["labels"].is_array());
    }

    #[test]
    fn test_basic_variables_returns_hashmap() {
        let vars = template_variables::basic_variables();
        assert!(vars.contains_key("project_name"));
        assert!(vars.contains_key("author"));
        assert!(vars.contains_key("description"));
    }

    #[test]
    fn test_rate_limit_response_format() {
        let response = github_api::rate_limit_response();
        assert!(response["message"].is_string());
        assert!(response["documentation_url"].is_string());
    }

    #[test]
    fn test_malformed_toml_is_invalid() {
        let toml = metadata_repository::malformed_toml();
        let result = toml::from_str::<serde_json::Value>(&toml);
        assert!(result.is_err(), "Malformed TOML should fail to parse");
    }
}
