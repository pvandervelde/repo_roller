//! GitHub API mocking utilities for integration tests.
//!
//! This module provides WireMock-based mocking of GitHub API endpoints
//! for testing error scenarios, rate limiting, and network failures.

use anyhow::Result;
use serde_json::json;
use wiremock::{
    matchers::{method, path, path_regex},
    Mock, MockServer, ResponseTemplate,
};

/// GitHub API mock server builder.
///
/// Provides a fluent interface for setting up GitHub API mocks.
pub struct GitHubMockServer {
    server: MockServer,
}

impl GitHubMockServer {
    /// Create a new mock server instance.
    pub async fn new() -> Result<Self> {
        let server = MockServer::start().await;
        Ok(Self { server })
    }

    /// Get the base URI for the mock server.
    pub fn uri(&self) -> String {
        self.server.uri()
    }

    /// Mock a rate limit exceeded response.
    pub async fn mock_rate_limit_exceeded(&self) {
        Mock::given(method("GET"))
            .and(path_regex(r"^/.*"))
            .respond_with(
                ResponseTemplate::new(429)
                    .set_body_json(json!({
                        "message": "API rate limit exceeded for user",
                        "documentation_url": "https://docs.github.com/rest/overview/resources-in-the-rest-api#rate-limiting"
                    }))
                    .insert_header("X-RateLimit-Limit", "5000")
                    .insert_header("X-RateLimit-Remaining", "0")
                    .insert_header("X-RateLimit-Reset", "1640000000"),
            )
            .mount(&self.server)
            .await;
    }

    /// Mock a repository not found response.
    pub async fn mock_repository_not_found(&self, owner: &str, repo: &str) {
        Mock::given(method("GET"))
            .and(path(format!("/repos/{}/{}", owner, repo)))
            .respond_with(
                ResponseTemplate::new(404).set_body_json(json!({
                    "message": "Not Found",
                    "documentation_url": "https://docs.github.com/rest/reference/repos#get-a-repository"
                })),
            )
            .mount(&self.server)
            .await;
    }

    /// Mock a permission denied (403 Forbidden) response.
    pub async fn mock_permission_denied(&self, path_pattern: &str) {
        Mock::given(method("GET"))
            .and(path_regex(path_pattern))
            .respond_with(
                ResponseTemplate::new(403).set_body_json(json!({
                    "message": "Resource not accessible by integration",
                    "documentation_url": "https://docs.github.com/rest/reference/repos"
                })),
            )
            .mount(&self.server)
            .await;
    }

    /// Mock a network timeout (no response).
    pub async fn mock_network_timeout(&self, path_pattern: &str) {
        use std::time::Duration;

        Mock::given(method("GET"))
            .and(path_regex(path_pattern))
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(60)))
            .mount(&self.server)
            .await;
    }

    /// Mock an unexpected response format.
    pub async fn mock_unexpected_response(&self, path_pattern: &str) {
        Mock::given(method("GET"))
            .and(path_regex(path_pattern))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({
                    "unexpected_field": "this shouldn't be here",
                    "missing_required_fields": true
                })),
            )
            .mount(&self.server)
            .await;
    }

    /// Mock a successful repository creation.
    pub async fn mock_create_repository_success(&self, owner: &str, repo_name: &str) {
        Mock::given(method("POST"))
            .and(path(format!("/orgs/{}/repos", owner)))
            .respond_with(
                ResponseTemplate::new(201).set_body_json(json!({
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
                    "default_branch": "main",
                    "has_issues": true,
                    "has_wiki": false,
                    "has_projects": true
                })),
            )
            .mount(&self.server)
            .await;
    }

    /// Mock a repository already exists error (422 Unprocessable Entity).
    pub async fn mock_repository_already_exists(&self, owner: &str, repo_name: &str) {
        Mock::given(method("POST"))
            .and(path(format!("/orgs/{}/repos", owner)))
            .respond_with(
                ResponseTemplate::new(422).set_body_json(json!({
                    "message": "Repository creation failed.",
                    "errors": [
                        {
                            "resource": "Repository",
                            "code": "custom",
                            "field": "name",
                            "message": format!("name already exists on this account: {}", repo_name)
                        }
                    ],
                    "documentation_url": "https://docs.github.com/rest/reference/repos#create-an-organization-repository"
                })),
            )
            .mount(&self.server)
            .await;
    }

    /// Mock a sequence of responses (useful for testing retry logic).
    pub async fn mock_sequence_then_success<F>(
        &self,
        path_pattern: &str,
        failure_count: usize,
        create_failure_response: F,
    ) where
        F: Fn() -> ResponseTemplate,
    {
        // Mount failure responses
        for _ in 0..failure_count {
            Mock::given(method("GET"))
                .and(path_regex(path_pattern))
                .respond_with(create_failure_response())
                .up_to_n_times(1)
                .mount(&self.server)
                .await;
        }

        // Mount success response
        Mock::given(method("GET"))
            .and(path_regex(path_pattern))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
            .mount(&self.server)
            .await;
    }

    /// Mock GitHub file contents API for template loading.
    pub async fn mock_file_contents(
        &self,
        owner: &str,
        repo: &str,
        file_path: &str,
        content: &str,
    ) {
        use base64::{engine::general_purpose, Engine as _};

        let encoded_content = general_purpose::STANDARD.encode(content);

        Mock::given(method("GET"))
            .and(path(format!(
                "/repos/{}/{}/contents/{}",
                owner, repo, file_path
            )))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({
                    "name": file_path.split('/').last().unwrap_or(file_path),
                    "path": file_path,
                    "sha": "abc123",
                    "size": content.len(),
                    "type": "file",
                    "content": encoded_content,
                    "encoding": "base64"
                })),
            )
            .mount(&self.server)
            .await;
    }

    /// Mock directory listing API.
    pub async fn mock_directory_contents(
        &self,
        owner: &str,
        repo: &str,
        dir_path: &str,
        entries: Vec<(&str, &str)>, // (name, type) pairs
    ) {
        let contents: Vec<_> = entries
            .iter()
            .map(|(name, entry_type)| {
                json!({
                    "name": name,
                    "path": format!("{}/{}", dir_path, name),
                    "type": entry_type,
                    "sha": "abc123"
                })
            })
            .collect();

        Mock::given(method("GET"))
            .and(path(format!(
                "/repos/{}/{}/contents/{}",
                owner, repo, dir_path
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(contents))
            .mount(&self.server)
            .await;
    }

    /// Mock search repositories API.
    pub async fn mock_search_repositories(
        &self,
        _query: &str,
        results: Vec<serde_json::Value>,
    ) {
        Mock::given(method("GET"))
            .and(path("/search/repositories"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({
                    "total_count": results.len(),
                    "incomplete_results": false,
                    "items": results
                })),
            )
            .mount(&self.server)
            .await;
    }

    /// Mock authentication (installation token) endpoint.
    pub async fn mock_installation_token(&self, installation_id: u64, token: &str) {
        Mock::given(method("POST"))
            .and(path(format!(
                "/app/installations/{}/access_tokens",
                installation_id
            )))
            .respond_with(
                ResponseTemplate::new(201).set_body_json(json!({
                    "token": token,
                    "expires_at": "2024-12-31T23:59:59Z",
                    "permissions": {
                        "contents": "write",
                        "metadata": "read"
                    }
                })),
            )
            .mount(&self.server)
            .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_server_starts() {
        let server = GitHubMockServer::new().await.unwrap();
        assert!(!server.uri().is_empty());
    }

    #[tokio::test]
    async fn test_mock_rate_limit_exceeded() {
        let server = GitHubMockServer::new().await.unwrap();
        server.mock_rate_limit_exceeded().await;

        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/repos/test/test", server.uri()))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 429);
        let body: serde_json::Value = response.json().await.unwrap();
        assert!(body["message"]
            .as_str()
            .unwrap()
            .contains("rate limit exceeded"));
    }

    #[tokio::test]
    async fn test_mock_repository_not_found() {
        let server = GitHubMockServer::new().await.unwrap();
        server
            .mock_repository_not_found("test-org", "test-repo")
            .await;

        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/repos/test-org/test-repo", server.uri()))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 404);
    }

    #[tokio::test]
    async fn test_mock_file_contents() {
        let server = GitHubMockServer::new().await.unwrap();
        server
            .mock_file_contents("test-org", "test-repo", "README.md", "# Test Content")
            .await;

        let client = reqwest::Client::new();
        let response = client
            .get(format!(
                "{}/repos/test-org/test-repo/contents/README.md",
                server.uri()
            ))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
        let body: serde_json::Value = response.json().await.unwrap();
        assert_eq!(body["name"], "README.md");
        assert_eq!(body["encoding"], "base64");
    }
}
