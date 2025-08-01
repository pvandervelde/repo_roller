//! Hierarchical configuration value types.
//!
//! This module provides the `OverridableValue<T>` type and related utilities for
//! implementing hierarchical configuration with override controls.

use serde::{Deserialize, Serialize};

#[cfg(test)]
#[path = "hierarchy_tests.rs"]
mod hierarchy_tests;

/// A generic wrapper for configuration values that supports override control.
///
/// This type allows configuration values to specify whether they can be overridden
/// by higher levels in the configuration hierarchy. This is essential for security
/// and policy enforcement where certain template requirements must be preserved.
///
/// # Type Parameters
///
/// * `T` - The type of the configuration value. Must implement `Clone` for merging operations.
///
/// # Examples
///
/// ```rust
/// use config_manager::hierarchy::OverridableValue;
///
/// // A value that can be overridden by higher levels
/// let flexible_setting = OverridableValue::new("default_value".to_string(), true);
///
/// // A security-critical value that cannot be overridden
/// let fixed_setting = OverridableValue::new("required_value".to_string(), false);
///
/// // Check if override is allowed
/// assert!(flexible_setting.can_override());
/// assert!(!fixed_setting.can_override());
///
/// // Get the actual value
/// assert_eq!(flexible_setting.value(), "default_value");
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OverridableValue<T: Clone> {
    /// The actual configuration value.
    value: T,
    /// Whether this value can be overridden by higher levels in the hierarchy.
    /// When `false`, this value is considered fixed and must not be changed.
    can_override: bool,
}

impl<T: Clone> OverridableValue<T> {
    /// Creates a new `OverridableValue` with the specified value and override permission.
    ///
    /// # Arguments
    ///
    /// * `value` - The configuration value to wrap
    /// * `can_override` - Whether this value can be overridden by higher hierarchy levels
    ///
    /// # Returns
    ///
    /// A new `OverridableValue` instance containing the provided value and override setting.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::hierarchy::OverridableValue;
    ///
    /// let setting = OverridableValue::new(42, true);
    /// assert_eq!(setting.value(), 42);
    /// assert!(setting.can_override());
    /// ```
    pub fn new(value: T, can_override: bool) -> Self {
        Self {
            value,
            can_override,
        }
    }

    /// Creates a new `OverridableValue` that allows override (convenience method).
    ///
    /// This is equivalent to calling `new(value, true)` but provides a more readable
    /// way to create overridable values.
    ///
    /// # Arguments
    ///
    /// * `value` - The configuration value to wrap
    ///
    /// # Returns
    ///
    /// A new `OverridableValue` that can be overridden by higher hierarchy levels.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::hierarchy::OverridableValue;
    ///
    /// let setting = OverridableValue::overridable("flexible_value".to_string());
    /// assert!(setting.can_override());
    /// ```
    pub fn overridable(value: T) -> Self {
        Self::new(value, true)
    }

    /// Creates a new `OverridableValue` that cannot be overridden (convenience method).
    ///
    /// This is equivalent to calling `new(value, false)` but provides a more readable
    /// way to create fixed values that enforce security or policy requirements.
    ///
    /// # Arguments
    ///
    /// * `value` - The configuration value to wrap
    ///
    /// # Returns
    ///
    /// A new `OverridableValue` that cannot be overridden by higher hierarchy levels.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::hierarchy::OverridableValue;
    ///
    /// let setting = OverridableValue::fixed("security_requirement".to_string());
    /// assert!(!setting.can_override());
    /// ```
    pub fn fixed(value: T) -> Self {
        Self::new(value, false)
    }

    /// Returns a reference to the wrapped configuration value.
    ///
    /// # Returns
    ///
    /// A reference to the configuration value of type `T`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::hierarchy::OverridableValue;
    ///
    /// let setting = OverridableValue::new(42, true);
    /// assert_eq!(setting.value(), 42);
    /// ```
    pub fn value(&self) -> T {
        self.value.clone()
    }

    /// Returns whether this value can be overridden by higher hierarchy levels.
    ///
    /// # Returns
    ///
    /// `true` if the value can be overridden, `false` if it is fixed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::hierarchy::OverridableValue;
    ///
    /// let flexible = OverridableValue::overridable("test".to_string());
    /// let fixed = OverridableValue::fixed("test".to_string());
    ///
    /// assert!(flexible.can_override());
    /// assert!(!fixed.can_override());
    /// ```
    pub fn can_override(&self) -> bool {
        self.can_override
    }

    /// Attempts to override this value with a new value from a higher hierarchy level.
    ///
    /// This operation will only succeed if `can_override` is `true`. If the value
    /// cannot be overridden, the original value is returned unchanged.
    ///
    /// # Arguments
    ///
    /// * `new_value` - The new value to apply if override is allowed
    ///
    /// # Returns
    ///
    /// A new `OverridableValue` with the new value if override is allowed,
    /// or the original value if override is not permitted.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::hierarchy::OverridableValue;
    ///
    /// let flexible = OverridableValue::overridable(42);
    /// let overridden = flexible.try_override(100);
    /// assert_eq!(overridden.value(), 100);
    ///
    /// let fixed = OverridableValue::fixed(42);
    /// let unchanged = fixed.try_override(100);
    /// assert_eq!(unchanged.value(), 42);
    /// ```
    pub fn try_override(&self, new_value: T) -> Self {
        if self.can_override {
            Self::new(new_value, self.can_override)
        } else {
            self.clone()
        }
    }
}
