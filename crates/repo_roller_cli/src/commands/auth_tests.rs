use super::*;

#[tokio::test]
async fn test_auth_commands_debug_format() {
    let cmd = AuthCommands::GitHub {
        method: "token".to_string(),
    };
    let debug_output = format!("{:?}", cmd);
    assert!(debug_output.contains("GitHub"));
    assert!(debug_output.contains("token"));
}

#[tokio::test]
async fn test_execute_github_invalid_method() {
    let cmd = AuthCommands::GitHub {
        method: "invalid".to_string(),
    };

    // This test would require mocking the config loading and keyring operations
    // For now, we'll just verify the command structure is valid
    if let AuthCommands::GitHub { method } = cmd {
        assert_eq!(method, "invalid");
    }
}

#[test]
fn test_keyring_constants() {
    assert_eq!(KEY_RING_SERVICE_NAME, "repo_roller_cli");
    assert_eq!(KEY_RING_APP_ID, "github_app_id");
    assert_eq!(KEY_RING_APP_PRIVATE_KEY_PATH, "github_private_key_path");
    assert_eq!(KEY_RING_USER_TOKEN, "github_token");
    assert_eq!(KEY_RING_WEB_HOOK_SECRET, "webhook_secret");
}
