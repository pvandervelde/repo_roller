use super::*;
use serde_json::{from_str, to_string};

#[test]
fn test_account_serialization() {
    let account = Account {
        id: 12345,
        login: "my-organization".to_string(),
        account_type: "Organization".to_string(),
        node_id: "MDEyOk9yZ2FuaXphdGlvbjEyMzQ1".to_string(),
    };

    let json_str = to_string(&account).expect("Failed to serialize Account");
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Failed to parse JSON");

    assert_eq!(parsed["id"], 12345);
    assert_eq!(parsed["login"], "my-organization");
    assert_eq!(parsed["type"], "Organization");
    assert_eq!(parsed["node_id"], "MDEyOk9yZ2FuaXphdGlvbjEyMzQ1");
}

#[test]
fn test_account_deserialization() {
    let json_str = r#"{
        "id": 67890,
        "login": "user-account",
        "type": "User",
        "node_id": "MDQ6VXNlcjY3ODkw"
    }"#;

    let account: Account = from_str(json_str).expect("Failed to deserialize Account");

    assert_eq!(account.id, 67890);
    assert_eq!(account.login, "user-account");
    assert_eq!(account.account_type, "User");
    assert_eq!(account.node_id, "MDQ6VXNlcjY3ODkw");
}

#[test]
fn test_installation_serialization() {
    let installation = Installation {
        id: 98765,
        account: Account {
            id: 12345,
            login: "test-org".to_string(),
            account_type: "Organization".to_string(),
            node_id: "MDEyOk9yZ2FuaXphdGlvbjEyMzQ1".to_string(),
        },
        repository_selection: Some("selected".to_string()),
        node_id: "MDIzOkluc3RhbGxhdGlvbjk4NzY1".to_string(),
    };

    let json_str = to_string(&installation).expect("Failed to serialize Installation");
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Failed to parse JSON");

    assert_eq!(parsed["id"], 98765);
    assert_eq!(parsed["account"]["id"], 12345);
    assert_eq!(parsed["account"]["login"], "test-org");
    assert_eq!(parsed["repository_selection"], "selected");
    assert_eq!(parsed["node_id"], "MDIzOkluc3RhbGxhdGlvbjk4NzY1");
}

#[test]
fn test_installation_deserialization() {
    let json_str = r#"{
        "id": 11111,
        "account": {
            "id": 22222,
            "login": "another-org",
            "type": "Organization",
            "node_id": "MDEyOk9yZ2FuaXphdGlvbjIyMjIy"
        },
        "repository_selection": null,
        "node_id": "MDIzOkluc3RhbGxhdGlvbjExMTEx"
    }"#;

    let installation: Installation =
        from_str(json_str).expect("Failed to deserialize Installation");

    assert_eq!(installation.id, 11111);
    assert_eq!(installation.account.id, 22222);
    assert_eq!(installation.account.login, "another-org");
    assert_eq!(installation.repository_selection, None);
    assert_eq!(installation.node_id, "MDIzOkluc3RhbGxhdGlvbjExMTEx");
}
