//! Tests for organization settings commands.

use super::*;

// ============================================================================
// Command Parsing Tests
// ============================================================================

/// Verify ListTypes command can be constructed with required arguments.
#[test]
fn test_list_types_command_construction() {
    let cmd = OrgSettingsCommands::ListTypes {
        org: "test-org".to_string(),
        format: "pretty".to_string(),
    };

    match cmd {
        OrgSettingsCommands::ListTypes { org, format } => {
            assert_eq!(org, "test-org");
            assert_eq!(format, "pretty");
        }
        _ => panic!("Expected ListTypes variant"),
    }
}

/// Verify ShowType command can be constructed with required arguments.
#[test]
fn test_show_type_command_construction() {
    let cmd = OrgSettingsCommands::ShowType {
        org: "test-org".to_string(),
        type_name: "library".to_string(),
        format: "json".to_string(),
    };

    match cmd {
        OrgSettingsCommands::ShowType {
            org,
            type_name,
            format,
        } => {
            assert_eq!(org, "test-org");
            assert_eq!(type_name, "library");
            assert_eq!(format, "json");
        }
        _ => panic!("Expected ShowType variant"),
    }
}

/// Verify ShowMerged command can be constructed with all arguments.
#[test]
fn test_show_merged_command_construction_with_all_args() {
    let cmd = OrgSettingsCommands::ShowMerged {
        org: "test-org".to_string(),
        template: "rust-lib".to_string(),
        team: Some("platform".to_string()),
        repo_type: Some("library".to_string()),
        format: "pretty".to_string(),
    };

    match cmd {
        OrgSettingsCommands::ShowMerged {
            org,
            template,
            team,
            repo_type,
            format,
        } => {
            assert_eq!(org, "test-org");
            assert_eq!(template, "rust-lib");
            assert_eq!(team, Some("platform".to_string()));
            assert_eq!(repo_type, Some("library".to_string()));
            assert_eq!(format, "pretty");
        }
        _ => panic!("Expected ShowMerged variant"),
    }
}

/// Verify ShowMerged command works with optional arguments omitted.
#[test]
fn test_show_merged_command_construction_without_optional_args() {
    let cmd = OrgSettingsCommands::ShowMerged {
        org: "test-org".to_string(),
        template: "rust-lib".to_string(),
        team: None,
        repo_type: None,
        format: "json".to_string(),
    };

    match cmd {
        OrgSettingsCommands::ShowMerged {
            org,
            template,
            team,
            repo_type,
            format,
        } => {
            assert_eq!(org, "test-org");
            assert_eq!(template, "rust-lib");
            assert_eq!(team, None);
            assert_eq!(repo_type, None);
            assert_eq!(format, "json");
        }
        _ => panic!("Expected ShowMerged variant"),
    }
}

/// Verify ShowGlobal command can be constructed with required arguments.
#[test]
fn test_show_global_command_construction() {
    let cmd = OrgSettingsCommands::ShowGlobal {
        org: "test-org".to_string(),
        format: "pretty".to_string(),
    };

    match cmd {
        OrgSettingsCommands::ShowGlobal { org, format } => {
            assert_eq!(org, "test-org");
            assert_eq!(format, "pretty");
        }
        _ => panic!("Expected ShowGlobal variant"),
    }
}

// ============================================================================
// Command Execution Tests (Implementation)
// ============================================================================

/// Verify list_types attempts authentication (fails without keyring).
#[tokio::test]
async fn test_list_types_returns_auth_error_without_credentials() {
    let result = list_types("test-org", "pretty").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Auth(msg) => {
            assert!(msg.contains("keyring") || msg.contains("app ID"));
        }
        e => panic!("Expected Auth error, got: {:?}", e),
    }
}

/// Verify show_type attempts authentication (fails without keyring).
#[tokio::test]
async fn test_show_type_returns_auth_error_without_credentials() {
    let result = show_type("test-org", "library", "json").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Auth(msg) => {
            assert!(msg.contains("keyring") || msg.contains("app ID"));
        }
        e => panic!("Expected Auth error, got: {:?}", e),
    }
}

/// Verify show_merged attempts authentication (fails without keyring).
#[tokio::test]
async fn test_show_merged_returns_auth_error_without_credentials() {
    let result = show_merged(
        "test-org",
        "rust-lib",
        Some("platform"),
        Some("library"),
        "pretty",
    )
    .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Auth(msg) => {
            assert!(msg.contains("keyring") || msg.contains("app ID"));
        }
        e => panic!("Expected Auth error, got: {:?}", e),
    }
}

/// Verify show_merged works without optional parameters (fails on auth).
#[tokio::test]
async fn test_show_merged_without_optional_params_returns_auth_error() {
    let result = show_merged("test-org", "rust-lib", None, None, "json").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Auth(msg) => {
            assert!(msg.contains("keyring") || msg.contains("app ID"));
        }
        e => panic!("Expected Auth error, got: {:?}", e),
    }
}

/// Verify show_global attempts authentication (fails without keyring).
#[tokio::test]
async fn test_show_global_returns_auth_error_without_credentials() {
    let result = show_global("test-org", "pretty").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Auth(msg) => {
            assert!(msg.contains("keyring") || msg.contains("app ID"));
        }
        e => panic!("Expected Auth error, got: {:?}", e),
    }
}

// ============================================================================
// Execute Routing Tests
// ============================================================================

/// Verify execute routes ListTypes to list_types handler (fails on auth).
#[tokio::test]
async fn test_execute_routes_list_types() {
    let cmd = OrgSettingsCommands::ListTypes {
        org: "test-org".to_string(),
        format: "pretty".to_string(),
    };

    let result = execute(&cmd).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Auth(msg) => {
            assert!(msg.contains("keyring") || msg.contains("app ID"));
        }
        e => panic!("Expected Auth error from list_types, got: {:?}", e),
    }
}

/// Verify execute routes ShowType to show_type handler (fails on auth).
#[tokio::test]
async fn test_execute_routes_show_type() {
    let cmd = OrgSettingsCommands::ShowType {
        org: "test-org".to_string(),
        type_name: "library".to_string(),
        format: "json".to_string(),
    };

    let result = execute(&cmd).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Auth(msg) => {
            assert!(msg.contains("keyring") || msg.contains("app ID"));
        }
        e => panic!("Expected Auth error from show_type, got: {:?}", e),
    }
}

/// Verify execute routes ShowMerged to show_merged handler (fails on auth).
#[tokio::test]
async fn test_execute_routes_show_merged() {
    let cmd = OrgSettingsCommands::ShowMerged {
        org: "test-org".to_string(),
        template: "rust-lib".to_string(),
        team: Some("platform".to_string()),
        repo_type: Some("library".to_string()),
        format: "pretty".to_string(),
    };

    let result = execute(&cmd).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Auth(msg) => {
            assert!(msg.contains("keyring") || msg.contains("app ID"));
        }
        e => panic!("Expected Auth error from show_merged, got: {:?}", e),
    }
}

/// Verify execute routes ShowGlobal to show_global handler (fails on auth).
#[tokio::test]
async fn test_execute_routes_show_global() {
    let cmd = OrgSettingsCommands::ShowGlobal {
        org: "test-org".to_string(),
        format: "pretty".to_string(),
    };

    let result = execute(&cmd).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Auth(msg) => {
            assert!(msg.contains("keyring") || msg.contains("app ID"));
        }
        e => panic!("Expected Auth error from show_global, got: {:?}", e),
    }
}

// ============================================================================
// Format Validation Tests
// ============================================================================

/// Verify commands accept "json" format.
#[test]
fn test_commands_accept_json_format() {
    let cmd = OrgSettingsCommands::ListTypes {
        org: "test-org".to_string(),
        format: "json".to_string(),
    };

    match cmd {
        OrgSettingsCommands::ListTypes { format, .. } => {
            assert_eq!(format, "json");
        }
        _ => panic!("Expected ListTypes variant"),
    }
}

/// Verify commands accept "pretty" format.
#[test]
fn test_commands_accept_pretty_format() {
    let cmd = OrgSettingsCommands::ShowGlobal {
        org: "test-org".to_string(),
        format: "pretty".to_string(),
    };

    match cmd {
        OrgSettingsCommands::ShowGlobal { format, .. } => {
            assert_eq!(format, "pretty");
        }
        _ => panic!("Expected ShowGlobal variant"),
    }
}
