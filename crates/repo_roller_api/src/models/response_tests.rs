//! Tests for response models

use super::*;

#[test]
fn test_create_repository_response_serialization() {
    let response = CreateRepositoryResponse {
        repository: RepositoryInfo {
            name: "my-repo".to_string(),
            full_name: "myorg/my-repo".to_string(),
            url: "https://github.com/myorg/my-repo".to_string(),
            visibility: "private".to_string(),
            description: None,
        },
        applied_configuration: serde_json::json!({}),
        created_at: "2025-11-12T10:30:00Z".to_string(),
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"repository\""));
    assert!(json.contains("\"createdAt\""));
}

#[test]
fn test_validation_response_serialization() {
    let response = ValidateRepositoryNameResponse {
        valid: false,
        available: true,
        messages: Some(vec!["Name must be lowercase".to_string()]),
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"valid\":false"));
    assert!(json.contains("\"available\":true"));
}

// ============================================================================
// TemplateSummary / TemplateDetailsResponse -- displayName + tags (#286)
//
// Issue 2 ("Human-readable display name"): `TemplateSummary` and
// `TemplateDetailsResponse` must carry a `displayName` field (serialized as
// `"displayName"` per the struct's `rename_all = "camelCase"`), independent
// of the existing `name` field (which remains the slug/identifier).
//
// Issue 3 ("Tags not displaying properly"): the old `category: Option<String>`
// field (which only ever carried `tags.first()`) is replaced by a plain
// `tags: Vec<String>` field that round-trips the full tag list, and
// `"category"` must no longer appear in the serialized JSON at all.
// ============================================================================

#[test]
fn test_list_templates_response_serialization() {
    let response = ListTemplatesResponse {
        templates: vec![TemplateSummary {
            name: "rust-library".to_string(),
            display_name: "Rust Library".to_string(),
            description: "Rust library template".to_string(),
            tags: vec!["rust".to_string(), "library".to_string()],
            variables: vec!["project_name".to_string()],
        }],
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"rust-library\""));
    assert!(
        json.contains("\"displayName\":\"Rust Library\""),
        "expected camelCase 'displayName' field with the human-readable name; got: {json}"
    );
    assert!(
        json.contains("\"tags\":[\"rust\",\"library\"]"),
        "expected the full tags array (not just the first tag) to be serialized; got: {json}"
    );
    assert!(
        !json.contains("\"category\""),
        "the deprecated 'category' field must no longer be present in the serialized JSON; got: {json}"
    );
}

/// `display_name` and `name` are independent fields that may legitimately
/// differ (slug vs. human-readable name). A hardcoded implementation that
/// copies `name` into `display_name` (or vice versa) would make this test
/// fail because the two asserted values are intentionally distinct.
#[test]
fn test_template_summary_display_name_and_name_are_independent_fields() {
    let summary = TemplateSummary {
        name: "rust-library".to_string(),
        display_name: "A Completely Different Human Name".to_string(),
        description: "desc".to_string(),
        tags: vec![],
        variables: vec![],
    };

    let json = serde_json::to_string(&summary).unwrap();
    assert!(json.contains("\"name\":\"rust-library\""));
    assert!(json.contains("\"displayName\":\"A Completely Different Human Name\""));
}

/// Tags is always present as a (possibly empty) JSON array -- it must never
/// be omitted the way the old `Option<String>` `category` field was (via
/// `skip_serializing_if`). A stub that keeps `skip_serializing_if` on the new
/// field, or that forgets to serialize an empty vec as `[]`, fails this test.
#[test]
fn test_template_summary_with_empty_tags_serializes_as_empty_array_not_omitted() {
    let summary = TemplateSummary {
        name: "bare-template".to_string(),
        display_name: "bare-template".to_string(),
        description: "no tags".to_string(),
        tags: vec![],
        variables: vec![],
    };

    let json = serde_json::to_string(&summary).unwrap();
    assert!(
        json.contains("\"tags\":[]"),
        "empty tags must serialize as an explicit empty array, not be omitted; got: {json}"
    );
}

/// Round-trip check: deserializing a wire-format JSON payload (camelCase
/// `displayName`, full `tags` array, no `category`) into `TemplateSummary`
/// must populate `display_name` and `tags` correctly. This protects against
/// a `#[serde(rename = ...)]` typo or a field left un-annotated.
#[test]
fn test_template_summary_deserializes_camel_case_display_name_and_tags() {
    let json = r#"{"name":"slug","displayName":"Slug Template","description":"desc","tags":["a","b"],"variables":[]}"#;
    let parsed: TemplateSummary = serde_json::from_str(json).unwrap();

    assert_eq!(parsed.display_name, "Slug Template");
    assert_eq!(parsed.tags, vec!["a".to_string(), "b".to_string()]);
    assert_eq!(parsed.name, "slug");
}

#[test]
fn test_template_details_response_serialization_includes_display_name_and_tags() {
    let response = TemplateDetailsResponse {
        name: "rust-library".to_string(),
        display_name: "Rust Library".to_string(),
        description: "Rust library template".to_string(),
        tags: vec!["rust".to_string(), "library".to_string()],
        variables: HashMap::new(),
        configuration: serde_json::json!({}),
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(
        json.contains("\"displayName\":\"Rust Library\""),
        "expected camelCase 'displayName' field; got: {json}"
    );
    assert!(
        json.contains("\"tags\":[\"rust\",\"library\"]"),
        "expected the full tags array to be serialized; got: {json}"
    );
    assert!(
        !json.contains("\"category\""),
        "the deprecated 'category' field must no longer be present; got: {json}"
    );
}

/// Same empty-tags-not-omitted guarantee as `TemplateSummary`, applied to
/// `TemplateDetailsResponse`.
#[test]
fn test_template_details_response_with_empty_tags_serializes_as_empty_array() {
    let response = TemplateDetailsResponse {
        name: "bare-template".to_string(),
        display_name: "bare-template".to_string(),
        description: "no tags".to_string(),
        tags: vec![],
        variables: HashMap::new(),
        configuration: serde_json::json!({}),
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(
        json.contains("\"tags\":[]"),
        "empty tags must serialize as an explicit empty array, not be omitted; got: {json}"
    );
}
