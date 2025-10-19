//! Generic overridable value type for hierarchical configuration.
//!
//! This module provides the `OverridableValue<T>` type which wraps a value along with
//! a flag indicating whether that value can be overridden by higher-precedence configuration
//! levels (team or template configurations).
//!
//! See: specs/design/organization-repository-settings.md

use serde::{Deserialize, Deserializer, Serialize};

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
/// Supports two deserialization formats:
///
/// **Explicit format** (for GlobalDefaults):
/// ```toml
/// setting_name = { value = true, override_allowed = false }
/// ```
///
/// **Simple format** (for Team/RepositoryType/Template configs):
/// ```toml
/// setting_name = true
/// ```
/// The simple format automatically wraps the value with `override_allowed = true`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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

// Custom deserialization to support both explicit and simple formats
impl<'de, T> Deserialize<'de> for OverridableValue<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;
        use std::marker::PhantomData;

        // Helper struct for explicit format deserialization
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Value,
            OverrideAllowed,
        }

        struct OverridableValueVisitor<T> {
            marker: PhantomData<T>,
        }

        impl<'de, T> Visitor<'de> for OverridableValueVisitor<T>
        where
            T: Deserialize<'de>,
        {
            type Value = OverridableValue<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an OverridableValue in either explicit or simple format")
            }

            // Handle explicit format: { value = X, override_allowed = Y }
            fn visit_map<V>(self, mut map: V) -> Result<OverridableValue<T>, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut value = None;
                let mut override_allowed = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Value => {
                            if value.is_some() {
                                return Err(de::Error::duplicate_field("value"));
                            }
                            value = Some(map.next_value()?);
                        }
                        Field::OverrideAllowed => {
                            if override_allowed.is_some() {
                                return Err(de::Error::duplicate_field("override_allowed"));
                            }
                            override_allowed = Some(map.next_value()?);
                        }
                    }
                }

                let value = value.ok_or_else(|| de::Error::missing_field("value"))?;
                let override_allowed =
                    override_allowed.ok_or_else(|| de::Error::missing_field("override_allowed"))?;

                Ok(OverridableValue {
                    value,
                    override_allowed,
                })
            }

            // Handle simple format: just the value directly
            // This delegates to T's deserializer for any type
            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
                T: Deserialize<'de>,
            {
                // Deserialize bool directly into T (if T is bool)
                let value = T::deserialize(de::value::BoolDeserializer::new(v)).map_err(
                    |_: de::value::Error| {
                        E::invalid_type(
                            de::Unexpected::Bool(v),
                            &"explicit OverridableValue format",
                        )
                    },
                )?;
                Ok(OverridableValue {
                    value,
                    override_allowed: true,
                })
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let value = T::deserialize(de::value::I64Deserializer::new(v)).map_err(
                    |_: de::value::Error| {
                        E::invalid_type(
                            de::Unexpected::Signed(v),
                            &"explicit OverridableValue format",
                        )
                    },
                )?;
                Ok(OverridableValue {
                    value,
                    override_allowed: true,
                })
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let value = T::deserialize(de::value::U64Deserializer::new(v)).map_err(
                    |_: de::value::Error| {
                        E::invalid_type(
                            de::Unexpected::Unsigned(v),
                            &"explicit OverridableValue format",
                        )
                    },
                )?;
                Ok(OverridableValue {
                    value,
                    override_allowed: true,
                })
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let value = T::deserialize(de::value::F64Deserializer::new(v)).map_err(
                    |_: de::value::Error| {
                        E::invalid_type(
                            de::Unexpected::Float(v),
                            &"explicit OverridableValue format",
                        )
                    },
                )?;
                Ok(OverridableValue {
                    value,
                    override_allowed: true,
                })
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let value = T::deserialize(de::value::StrDeserializer::new(v)).map_err(
                    |_: de::value::Error| {
                        E::invalid_type(de::Unexpected::Str(v), &"explicit OverridableValue format")
                    },
                )?;
                Ok(OverridableValue {
                    value,
                    override_allowed: true,
                })
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let value = T::deserialize(de::value::StringDeserializer::new(v.clone())).map_err(
                    |_: de::value::Error| {
                        E::invalid_type(
                            de::Unexpected::Str(&v),
                            &"explicit OverridableValue format",
                        )
                    },
                )?;
                Ok(OverridableValue {
                    value,
                    override_allowed: true,
                })
            }
        }

        deserializer.deserialize_any(OverridableValueVisitor {
            marker: PhantomData,
        })
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
