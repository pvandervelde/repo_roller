//! Tests for InstallationId and GitHubToken

use super::*;

#[test]
fn test_installation_id() {
    let id = InstallationId::new(12345);
    assert_eq!(id.as_u64(), 12345);
    assert_eq!(id.to_string(), "12345");
}

#[test]
fn test_github_token_valid() {
    assert!(GitHubToken::new("ghp_1234567890").is_ok());
}

#[test]
fn test_github_token_invalid() {
    assert!(GitHubToken::new("").is_err());
    assert!(GitHubToken::new("short").is_err());
}

#[test]
fn test_github_token_security() {
    let token = GitHubToken::new("ghp_secret_token_value").unwrap();
    let debug_output = format!("{:?}", token);
    assert!(!debug_output.contains("secret"));
    assert!(debug_output.contains("REDACTED"));

    let display_output = format!("{}", token);
    assert_eq!(display_output, "[REDACTED]");
}
