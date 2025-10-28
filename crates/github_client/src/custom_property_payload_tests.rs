//! Tests for GitHub custom property payload types.

use super::*;
use serde_json::json;

/// Verify that CustomPropertiesPayload can be created with new().
#[test]
fn test_custom_properties_payload_new() {
    let properties = vec![json!({
        "property_name": "type",
        "value": "library"
    })];

    let payload = CustomPropertiesPayload::new(properties.clone());

    assert_eq!(payload.properties.len(), 1);
    assert_eq!(payload.properties, properties);
}

/// Verify that CustomPropertiesPayload serializes to correct JSON format.
#[test]
fn test_custom_properties_payload_serialization() {
    let properties = vec![
        json!({
            "property_name": "repository_type",
            "value": "library"
        }),
        json!({
            "property_name": "team",
            "value": "backend"
        }),
    ];

    let payload = CustomPropertiesPayload::new(properties);
    let json_str = serde_json::to_string(&payload).expect("Failed to serialize");

    assert!(json_str.contains("\"properties\""));
    assert!(json_str.contains("\"repository_type\""));
    assert!(json_str.contains("\"library\""));
    assert!(json_str.contains("\"team\""));
    assert!(json_str.contains("\"backend\""));
}

/// Verify that CustomPropertiesPayload deserializes from JSON correctly.
#[test]
fn test_custom_properties_payload_deserialization() {
    let json_str = r#"{
        "properties": [
            {
                "property_name": "environment",
                "value": "production"
            }
        ]
    }"#;

    let payload: CustomPropertiesPayload =
        serde_json::from_str(json_str).expect("Failed to deserialize");

    assert_eq!(payload.properties.len(), 1);
    assert_eq!(payload.properties[0]["property_name"], "environment");
    assert_eq!(payload.properties[0]["value"], "production");
}

/// Verify that empty properties list is handled correctly.
#[test]
fn test_custom_properties_payload_empty() {
    let payload = CustomPropertiesPayload::new(vec![]);

    assert_eq!(payload.properties.len(), 0);

    let json_str = serde_json::to_string(&payload).expect("Failed to serialize");
    assert!(json_str.contains("\"properties\":[]"));
}

/// Verify that complex property values (arrays, booleans) serialize correctly.
#[test]
fn test_custom_properties_payload_complex_values() {
    let properties = vec![
        json!({
            "property_name": "tags",
            "value": ["rust", "web"]
        }),
        json!({
            "property_name": "archived",
            "value": false
        }),
    ];

    let payload = CustomPropertiesPayload::new(properties.clone());
    let serialized = serde_json::to_value(&payload).expect("Failed to serialize");

    assert_eq!(serialized["properties"], json!(properties));
}
