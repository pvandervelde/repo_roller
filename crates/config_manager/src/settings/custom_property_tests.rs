//! Tests for CustomProperty
use super::*;
#[test]
fn test_custom_property_creation() {
    let prop = CustomProperty {
        property_name: "test".to_string(),
        value: CustomPropertyValue::String("value".to_string()),
    };
    assert_eq!(prop.property_name, "test");
}
