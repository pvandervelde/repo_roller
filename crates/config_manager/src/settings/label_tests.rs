//! Tests for label configuration.

use super::*;

#[test]
fn test_label_creation() {
    let label = LabelConfig {
        name: "bug".to_string(),
        color: "d73a4a".to_string(),
        description: "Something isn't working".to_string(),
    };

    assert_eq!(label.name, "bug");
    assert_eq!(label.color, "d73a4a");
    assert_eq!(label.description, "Something isn't working");
}

#[test]
fn test_label_serialization() {
    let label = LabelConfig {
        name: "enhancement".to_string(),
        color: "a2eeef".to_string(),
        description: "New feature or request".to_string(),
    };

    let toml = toml::to_string(&label).expect("Failed to serialize");
    assert!(toml.contains("name = \"enhancement\""));
    assert!(toml.contains("color = \"a2eeef\""));
    assert!(toml.contains("description = \"New feature or request\""));
}

#[test]
fn test_label_deserialization() {
    let toml = r#"
        name = "documentation"
        color = "0052cc"
        description = "Improvements or additions to documentation"
    "#;

    let label: LabelConfig = toml::from_str(toml).expect("Failed to parse");
    assert_eq!(label.name, "documentation");
    assert_eq!(label.color, "0052cc");
    assert_eq!(
        label.description,
        "Improvements or additions to documentation"
    );
}
