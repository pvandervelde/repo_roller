use super::*;
use serde_json::{from_str, to_string};

#[test]
fn test_label_deserialization() {
    // Create JSON
    let json_str = r#"{"name": "feature"}"#;

    // Deserialize from JSON
    let label: Label = from_str(json_str).expect("Failed to deserialize Label");

    // Verify fields
    assert_eq!(label.name, "feature");
}

#[test]
fn test_label_serialization() {
    // Create a label
    let label = Label {
        name: "bug".to_string(),
    };

    // Serialize to JSON
    let json_str = to_string(&label).expect("Failed to serialize Label");

    // Verify JSON structure
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Failed to parse JSON");
    assert_eq!(parsed["name"], "bug");
}
