use super::*;
use std::error::Error as StdError;

#[test]
fn test_api_error_display() {
    let error = Error::ApiError();

    // Test error message
    assert_eq!(error.to_string(), "API request failed");

    // Test error source
    assert!(error.source().is_none());
}

#[test]
fn test_auth_error_display() {
    let error = Error::AuthError("Invalid credentials".to_string());

    // Test error message
    assert_eq!(
        error.to_string(),
        "Failed to authenticate or initialize GitHub client: Invalid credentials"
    );

    // Test error source
    assert!(error.source().is_none());
}

#[test]
fn test_invalid_response_error_display() {
    let error = Error::InvalidResponse;

    // Test error message
    assert_eq!(error.to_string(), "Invalid response format");

    // Test error source
    assert!(error.source().is_none());
}

#[test]
fn test_rate_limit_exceeded_error_display() {
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

// ============================================================================
// Error::AlreadyExists Tests (RED: task #282 -- new variant signalling that
// `create_org_repository` detected GitHub's own "already exists" 422
// response, distinct from the generic `InvalidResponse` every other
// create_org_repository failure maps to.)
// ============================================================================

#[test]
fn test_already_exists_error_display() {
    let error = Error::AlreadyExists;

    assert_eq!(error.to_string(), "Repository already exists");
    assert!(error.source().is_none());
}

/// Stub-killing: an implementation that aliases the new variant onto an
/// existing one (e.g. reusing `InvalidResponse` or `NotFound` instead of
/// adding a real new variant) would fail this exhaustive-match test, since
/// it would either fail to compile this `match` as written or collapse two
/// semantically distinct cases into the same arm.
#[test]
fn test_already_exists_is_a_distinct_variant_from_not_found_and_invalid_response() {
    let already_exists = Error::AlreadyExists;
    let not_found = Error::NotFound;
    let invalid_response = Error::InvalidResponse;

    assert_ne!(already_exists.to_string(), not_found.to_string());
    assert_ne!(already_exists.to_string(), invalid_response.to_string());

    // Exhaustive match: compilation itself proves `AlreadyExists` is a real,
    // independently-matchable variant (not a `#[error(...)]`-only alias).
    fn classify(e: &Error) -> &'static str {
        match e {
            Error::AlreadyExists => "already_exists",
            Error::ApiError() => "api_error",
            Error::AuthError(_) => "auth_error",
            Error::Deserialization(_) => "deserialization",
            Error::FailedToCreateAccessToken(..) => "failed_to_create_access_token",
            Error::FailedToFindAppInstallation(..) => "failed_to_find_app_installation",
            Error::InvalidResponse => "invalid_response",
            Error::NotFound => "not_found",
            Error::RateLimitExceeded => "rate_limit_exceeded",
        }
    }

    assert_eq!(classify(&already_exists), "already_exists");
    assert_eq!(classify(&not_found), "not_found");
}
