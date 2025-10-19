//! Generic overridable value type for hierarchical configuration.
//!
//! This module provides the `OverridableValue<T>` type which wraps a value along with
//! a flag indicating whether that value can be overridden by higher-precedence configuration
//! levels (team or template configurations).
//!
//! See: specs/design/organization-repository-settings.md

use serde::{Deserialize, Serialize};

/// A value that can optionally be overridden by higher-precedence configuration levels.
///
/// This type is used in global defaults to control which settings can be customized
/// by teams or templates. When `override_allowed` is `false`, the value represents
/// an organization-wide policy that cannot be changed.
///
/// # Examples
///
/// ```rust
/// use config_manager::OverridableValue;
///
/// // A security policy that cannot be overridden
/// let security_setting = OverridableValue {
///     value: true,
///     override_allowed: false,
/// };
/// assert!(!security_setting.can_override());
///
/// // A default that teams can customize
/// let customizable = OverridableValue {
///     value: false,
///     override_allowed: true,
/// };
/// assert!(customizable.can_override());
/// ```
///
/// # TOML Format
///
/// When serialized to TOML, the format is:
///
/// ```toml
/// setting_name = { value = true, override_allowed = false }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OverridableValue<T> {
    /// The actual value of the setting.
    pub value: T,

    /// Whether this value can be overridden by team or template configuration.
    ///
    /// When `false`, this represents an organization-wide policy that cannot be changed.
    pub override_allowed: bool,
}

impl<T> OverridableValue<T> {
    /// Create a new overridable value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::OverridableValue;
    ///
    /// let value = OverridableValue::new(42, true);
    /// assert_eq!(value.value, 42);
    /// assert!(value.can_override());
    /// ```
    pub fn new(value: T, override_allowed: bool) -> Self {
        Self {
            value,
            override_allowed,
        }
    }

    /// Create a new overridable value that allows overrides.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::OverridableValue;
    ///
    /// let value = OverridableValue::allowed(42);
    /// assert!(value.can_override());
    /// ```
    pub fn allowed(value: T) -> Self {
        Self::new(value, true)
    }

    /// Create a new overridable value that prohibits overrides (policy).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::OverridableValue;
    ///
    /// let policy = OverridableValue::fixed(true);
    /// assert!(!policy.can_override());
    /// ```
    pub fn fixed(value: T) -> Self {
        Self::new(value, false)
    }

    /// Check if this value can be overridden.
    ///
    /// Returns `true` if team or template configuration can override this value,
    /// `false` if this represents a fixed organizational policy.
    pub fn can_override(&self) -> bool {
        self.override_allowed
    }

    /// Get a reference to the underlying value.
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Get a mutable reference to the underlying value.
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }

    /// Consume this overridable value and return the inner value.
    pub fn into_inner(self) -> T {
        self.value
    }

    /// Map the value using a function while preserving the override policy.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::OverridableValue;
    ///
    /// let value = OverridableValue::new(42, true);
    /// let doubled = value.map(|x| x * 2);
    /// assert_eq!(doubled.value, 84);
    /// assert!(doubled.can_override());
    /// ```
    pub fn map<U, F>(self, f: F) -> OverridableValue<U>
    where
        F: FnOnce(T) -> U,
    {
        OverridableValue {
            value: f(self.value),
            override_allowed: self.override_allowed,
        }
    }
}

// Implement Default only when T implements Default
impl<T: Default> Default for OverridableValue<T> {
    fn default() -> Self {
        Self {
            value: T::default(),
            override_allowed: true, // Default to allowing overrides
        }
    }
}

#[cfg(test)]
#[path = "overridable_tests.rs"]
mod tests;
