use super::*;
use serde_json::{from_str, to_string};

#[test]
fn test_branch_protection_serialization() {
    let protection = BranchProtection {
        required_approving_review_count: Some(2),
        require_code_owner_reviews: Some(true),
        dismiss_stale_reviews: Some(false),
    };

    let json_str = to_string(&protection).expect("Failed to serialize BranchProtection");
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Failed to parse JSON");

    assert_eq!(parsed["required_approving_review_count"], 2);
    assert_eq!(parsed["require_code_owner_reviews"], true);
    assert_eq!(parsed["dismiss_stale_reviews"], false);
}

#[test]
fn test_branch_protection_deserialization() {
    let json_str = r#"{
        "required_approving_review_count": 1,
        "require_code_owner_reviews": false,
        "dismiss_stale_reviews": true
    }"#;

    let protection: BranchProtection =
        from_str(json_str).expect("Failed to deserialize BranchProtection");

    assert_eq!(protection.required_approving_review_count, Some(1));
    assert_eq!(protection.require_code_owner_reviews, Some(false));
    assert_eq!(protection.dismiss_stale_reviews, Some(true));
}

#[test]
fn test_branch_protection_with_null_values() {
    let json_str = r#"{
        "required_approving_review_count": null,
        "require_code_owner_reviews": null,
        "dismiss_stale_reviews": null
    }"#;

    let protection: BranchProtection =
        from_str(json_str).expect("Failed to deserialize BranchProtection");

    assert_eq!(protection.required_approving_review_count, None);
    assert_eq!(protection.require_code_owner_reviews, None);
    assert_eq!(protection.dismiss_stale_reviews, None);
}
