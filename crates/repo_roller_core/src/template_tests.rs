//! Tests for TemplateName

use super::*;

#[test]
fn test_template_name_valid() {
    assert!(TemplateName::new("rust-library").is_ok());
    assert!(TemplateName::new("go-microservice").is_ok());
    assert!(TemplateName::new("node-api-v2").is_ok());
}

#[test]
fn test_template_name_invalid() {
    assert!(TemplateName::new("RustLibrary").is_err()); // No uppercase
    assert!(TemplateName::new("rust_library").is_err()); // No underscores
    assert!(TemplateName::new("-starts-dash").is_err());
    assert!(TemplateName::new("ends-dash-").is_err());
    assert!(TemplateName::new("").is_err());
    assert!(TemplateName::new("a".repeat(51)).is_err());
}
