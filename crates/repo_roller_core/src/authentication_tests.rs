//! Tests for UserId and SessionId

use super::*;
use uuid::Uuid;

#[test]
fn test_user_id_creation() {
    let id1 = UserId::new();
    let id2 = UserId::new();
    assert_ne!(id1, id2); // Should be unique
}

#[test]
fn test_user_id_from_uuid() {
    let uuid = Uuid::new_v4();
    let user_id = UserId::from_uuid(uuid);
    assert_eq!(user_id.as_uuid(), &uuid);
}

#[test]
fn test_session_id_creation() {
    let id1 = SessionId::new();
    let id2 = SessionId::new();
    assert_ne!(id1, id2); // Should be unique
}

#[test]
fn test_session_id_from_uuid() {
    let uuid = Uuid::new_v4();
    let session_id = SessionId::from_uuid(uuid);
    assert_eq!(session_id.as_uuid(), &uuid);
}
