//! Tests for GitHub team domain types.

use super::*;

// ============================================================================
// Team Serialization / Deserialization Tests
// ============================================================================

#[test]
fn test_team_deserialization_from_github_api() {
    let json = r#"{
        "id": 12345,
        "slug": "backend-engineers",
        "name": "Backend Engineers",
        "description": "The backend engineering team"
    }"#;

    let team: Team = serde_json::from_str(json).expect("Failed to deserialize Team");

    assert_eq!(team.id, 12345);
    assert_eq!(team.slug, "backend-engineers");
    assert_eq!(team.name, "Backend Engineers");
    assert_eq!(
        team.description,
        Some("The backend engineering team".to_string())
    );
}

#[test]
fn test_team_deserialization_without_description() {
    let json = r#"{
        "id": 67890,
        "slug": "frontend",
        "name": "Frontend",
        "description": null
    }"#;

    let team: Team =
        serde_json::from_str(json).expect("Failed to deserialize Team without description");

    assert_eq!(team.id, 67890);
    assert_eq!(team.slug, "frontend");
    assert_eq!(team.name, "Frontend");
    assert!(team.description.is_none());
}

#[test]
fn test_team_deserialization_missing_description_field() {
    // GitHub API sometimes omits optional fields entirely
    let json = r#"{
        "id": 11111,
        "slug": "devops",
        "name": "DevOps"
    }"#;

    let team: Team =
        serde_json::from_str(json).expect("Failed to deserialize Team with missing description");

    assert_eq!(team.id, 11111);
    assert_eq!(team.slug, "devops");
    assert_eq!(team.name, "DevOps");
    assert!(team.description.is_none());
}

#[test]
fn test_team_serialization_round_trip() {
    let original = Team {
        id: 55555,
        slug: "security-team".to_string(),
        name: "Security Team".to_string(),
        description: Some("Handles security reviews".to_string()),
    };

    let json = serde_json::to_string(&original).expect("Failed to serialize Team");
    let deserialized: Team =
        serde_json::from_str(&json).expect("Failed to deserialize Team round-trip");

    assert_eq!(original, deserialized);
}

#[test]
fn test_team_clone_and_debug() {
    let team = Team {
        id: 9999,
        slug: "test-team".to_string(),
        name: "Test Team".to_string(),
        description: None,
    };

    let cloned = team.clone();
    assert_eq!(team, cloned);

    // Ensure Debug formatting doesn't panic
    let debug_str = format!("{:?}", team);
    assert!(debug_str.contains("Test Team"));
}

// ============================================================================
// TeamMember Serialization / Deserialization Tests
// ============================================================================

#[test]
fn test_team_member_deserialization_from_github_api() {
    let json = r#"{
        "id": 98765,
        "login": "jsmith"
    }"#;

    let member: TeamMember = serde_json::from_str(json).expect("Failed to deserialize TeamMember");

    assert_eq!(member.id, 98765);
    assert_eq!(member.login, "jsmith");
}

#[test]
fn test_team_member_serialization_round_trip() {
    let original = TeamMember {
        id: 11223,
        login: "alice-dev".to_string(),
    };

    let json = serde_json::to_string(&original).expect("Failed to serialize TeamMember");
    let deserialized: TeamMember =
        serde_json::from_str(&json).expect("Failed to deserialize TeamMember round-trip");

    assert_eq!(original, deserialized);
}

#[test]
fn test_team_member_clone_and_debug() {
    let member = TeamMember {
        id: 42,
        login: "bob-reviewer".to_string(),
    };

    let cloned = member.clone();
    assert_eq!(member, cloned);

    let debug_str = format!("{:?}", member);
    assert!(debug_str.contains("bob-reviewer"));
}

#[test]
fn test_team_member_list_deserialization() {
    // Simulate a GitHub API paginated response array
    let json = r#"[
        {"id": 1, "login": "alice"},
        {"id": 2, "login": "bob"},
        {"id": 3, "login": "carol"}
    ]"#;

    let members: Vec<TeamMember> =
        serde_json::from_str(json).expect("Failed to deserialize TeamMember list");

    assert_eq!(members.len(), 3);
    assert_eq!(members[0].login, "alice");
    assert_eq!(members[1].login, "bob");
    assert_eq!(members[2].login, "carol");
}
