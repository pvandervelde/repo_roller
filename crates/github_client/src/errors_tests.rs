use super::*;
use std::error::Error as StdError;

#[test]
fn test_api_error() {
    let error = Error::ApiError();

    // Test error message
    assert_eq!(error.to_string(), "API request failed");

    // Test error source
    assert!(error.source().is_none());
}

#[test]
fn test_auth_error() {
    let error = Error::AuthError("Invalid credentials".to_string());

    // Test error message
    assert_eq!(
        error.to_string(),
        "Authentication failed: Invalid credentials"
    );

    // Test error source
    assert!(error.source().is_none());
}

#[test]
fn test_invalid_response_error() {
    let error = Error::InvalidResponse;

    // Test error message
    assert_eq!(error.to_string(), "Invalid response format");

    // Test error source
    assert!(error.source().is_none());
}

#[test]
fn test_rate_limit_exceeded_error() {
    let error = Error::RateLimitExceeded;

    // Test error message
    assert_eq!(error.to_string(), "Rate limit exceeded");

    // Test error source
    assert!(error.source().is_none());
}

#[test]
fn test_error_is_send_sync() {
    // This test verifies that Error implements Send and Sync traits
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Error>();
}
