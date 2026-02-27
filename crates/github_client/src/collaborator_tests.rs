//! Serde round-trip and deserialization tests for [`Collaborator`].

use super::*;

// ── Collaborator serde tests ──────────────────────────────────────────────────

#[test]
fn collaborator_deserializes_from_json() {
    let json = r#"{"id": 11223344, "login": "octocat"}"#;
    let collaborator: Collaborator = serde_json::from_str(json).unwrap();
    assert_eq!(collaborator.id, 11223344);
    assert_eq!(collaborator.login, "octocat");
}

#[test]
fn collaborator_round_trips_via_serde() {
    let original = Collaborator {
        id: 42,
        login: "devuser".to_string(),
    };
    let json = serde_json::to_string(&original).unwrap();
    let restored: Collaborator = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn collaborator_deserializes_ignoring_extra_fields() {
    // GitHub API responds with many extra fields; we should ignore them gracefully.
    let json = r#"{
        "id": 99,
        "login": "extrauser",
        "avatar_url": "https://avatars.githubusercontent.com/u/99?v=4",
        "gravatar_id": "",
        "url": "https://api.github.com/users/extrauser",
        "permissions": {"pull": true, "push": true, "admin": false},
        "role_name": "write"
    }"#;
    let collaborator: Collaborator = serde_json::from_str(json).unwrap();
    assert_eq!(collaborator.id, 99);
    assert_eq!(collaborator.login, "extrauser");
}

#[test]
fn collaborator_fails_on_missing_required_fields() {
    let json = r#"{"id": 1}"#; // missing "login"
    let result: Result<Collaborator, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

#[test]
fn collaborator_equality_is_value_based() {
    let a = Collaborator {
        id: 1,
        login: "alice".to_string(),
    };
    let b = Collaborator {
        id: 1,
        login: "alice".to_string(),
    };
    let c = Collaborator {
        id: 2,
        login: "alice".to_string(),
    };
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn collaborator_clone_is_independent() {
    let original = Collaborator {
        id: 7,
        login: "tester".to_string(),
    };
    let mut cloned = original.clone();
    cloned.login = "modified".to_string();
    assert_eq!(original.login, "tester");
    assert_eq!(cloned.login, "modified");
}
