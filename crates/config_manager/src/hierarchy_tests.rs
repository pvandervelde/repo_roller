//! Tests for hierarchical configuration value types.

use crate::hierarchy::*;

#[cfg(test)]
mod overridable_value_tests {
    use super::*;

    #[test]
    fn new_creates_overridable_value_with_correct_properties() {
        let value = OverridableValue::new(42, true);
        assert_eq!(value.value(), 42);
        assert!(value.can_override());

        let fixed_value = OverridableValue::new("test".to_string(), false);
        assert_eq!(fixed_value.value(), "test");
        assert!(!fixed_value.can_override());
    }

    #[test]
    fn overridable_creates_value_that_can_be_overridden() {
        let value = OverridableValue::overridable(100);
        assert_eq!(value.value(), 100);
        assert!(value.can_override());
    }

    #[test]
    fn fixed_creates_value_that_cannot_be_overridden() {
        let value = OverridableValue::fixed("security_setting".to_string());
        assert_eq!(value.value(), "security_setting");
        assert!(!value.can_override());
    }

    #[test]
    fn try_override_succeeds_when_override_allowed() {
        let original = OverridableValue::overridable(10);
        let overridden = original.try_override(20);

        assert_eq!(overridden.value(), 20);
        assert!(overridden.can_override()); // Override permission is preserved
    }

    #[test]
    fn try_override_fails_when_override_not_allowed() {
        let original = OverridableValue::fixed(10);
        let unchanged = original.try_override(20);

        assert_eq!(unchanged.value(), 10); // Value remains unchanged
        assert!(!unchanged.can_override()); // Still cannot be overridden
    }

    #[test]
    fn overridable_value_equality() {
        let value1 = OverridableValue::new("test".to_string(), true);
        let value2 = OverridableValue::new("test".to_string(), true);
        let value3 = OverridableValue::new("test".to_string(), false);
        let value4 = OverridableValue::new("different".to_string(), true);

        assert_eq!(value1, value2);
        assert_ne!(value1, value3); // Different override permission
        assert_ne!(value1, value4); // Different value
    }

    #[test]
    fn overridable_value_clone() {
        let original = OverridableValue::new(vec![1, 2, 3], true);
        let cloned = original.clone();

        assert_eq!(original.value(), cloned.value());
        assert_eq!(original.can_override(), cloned.can_override());
    }

    #[test]
    fn overridable_value_with_complex_types() {
        use std::collections::HashMap;

        let mut map = HashMap::new();
        map.insert("key1".to_string(), "value1".to_string());

        let overridable_map = OverridableValue::overridable(map.clone());
        assert_eq!(overridable_map.value(), map);
        assert!(overridable_map.can_override());

        let fixed_map = OverridableValue::fixed(map.clone());
        assert_eq!(fixed_map.value(), map);
        assert!(!fixed_map.can_override());
    }

    #[test]
    fn try_override_preserves_original_when_not_allowed() {
        let original = OverridableValue::fixed("original".to_string());
        let result = original.try_override("new_value".to_string());

        // Original should be unchanged
        assert_eq!(original.value(), "original");
        assert!(!original.can_override());

        // Result should be a clone of original, not the new value
        assert_eq!(result.value(), "original");
        assert!(!result.can_override());
    }

    #[test]
    fn try_override_creates_new_instance_when_allowed() {
        let original = OverridableValue::overridable("original".to_string());
        let result = original.try_override("new_value".to_string());

        // Original should be unchanged
        assert_eq!(original.value(), "original");
        assert!(original.can_override());

        // Result should have the new value
        assert_eq!(result.value(), "new_value");
        assert!(result.can_override());
    }
}
