//! Tests for repository type validator.

use super::*;

/// Verify validator can be created.
#[test]
fn test_validator_creation() {
    let validator = RepositoryTypeValidator::new();
    assert!(std::mem::size_of_val(&validator) == 0); // Zero-sized type
}

/// Verify validator default works.
#[test]
fn test_validator_default() {
    let validator = RepositoryTypeValidator::default();
    assert!(std::mem::size_of_val(&validator) == 0);
}

// ============================================================================
// Type Existence Validation Tests
// ============================================================================

/// Verify valid type in available list passes validation.
#[test]
fn test_validate_type_exists_valid() {
    let validator = RepositoryTypeValidator::new();
    let available = vec!["library".to_string(), "service".to_string()];
    let type_name = RepositoryTypeName::try_new("library").unwrap();

    let result = validator.validate_type_exists(&type_name, &available);
    assert!(result.is_ok());
}

/// Verify case-insensitive matching works.
#[test]
fn test_validate_type_exists_case_insensitive() {
    let validator = RepositoryTypeValidator::new();
    let available = vec!["Library".to_string(), "SERVICE".to_string()];
    let type_name = RepositoryTypeName::try_new("library").unwrap();

    // Should match "Library" case-insensitively
    let result = validator.validate_type_exists(&type_name, &available);
    assert!(result.is_ok());
}

/// Verify type not in list fails validation.
#[test]
fn test_validate_type_exists_not_found() {
    let validator = RepositoryTypeValidator::new();
    let available = vec!["library".to_string(), "service".to_string()];
    let type_name = RepositoryTypeName::try_new("unknown").unwrap();

    let result = validator.validate_type_exists(&type_name, &available);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(matches!(
        err,
        ConfigurationError::InvalidConfiguration { .. }
    ));
}

/// Verify empty available list fails validation.
#[test]
fn test_validate_type_exists_empty_list() {
    let validator = RepositoryTypeValidator::new();
    let available: Vec<String> = vec![];
    let type_name = RepositoryTypeName::try_new("library").unwrap();

    let result = validator.validate_type_exists(&type_name, &available);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(matches!(
        err,
        ConfigurationError::InvalidConfiguration { .. }
    ));
}

// ============================================================================
// Policy Enforcement Tests
// ============================================================================

/// Verify Fixed policy with no override returns template type.
#[test]
fn test_validate_policy_fixed_no_override() {
    let validator = RepositoryTypeValidator::new();
    let spec = RepositoryTypeSpec {
        repository_type: "service".to_string(),
        policy: RepositoryTypePolicy::Fixed,
    };

    let result = validator.validate_type_policy(&spec, None).unwrap();
    assert_eq!(result.as_str(), "service");
}

/// Verify Fixed policy with user override fails.
#[test]
fn test_validate_policy_fixed_with_override_fails() {
    let validator = RepositoryTypeValidator::new();
    let spec = RepositoryTypeSpec {
        repository_type: "service".to_string(),
        policy: RepositoryTypePolicy::Fixed,
    };
    let user_override = RepositoryTypeName::try_new("library").unwrap();

    let result = validator.validate_type_policy(&spec, Some(&user_override));
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(matches!(
        err,
        ConfigurationError::InvalidConfiguration { .. }
    ));
}

/// Verify Preferable policy with no override returns template type.
#[test]
fn test_validate_policy_preferable_no_override() {
    let validator = RepositoryTypeValidator::new();
    let spec = RepositoryTypeSpec {
        repository_type: "library".to_string(),
        policy: RepositoryTypePolicy::Preferable,
    };

    let result = validator.validate_type_policy(&spec, None).unwrap();
    assert_eq!(result.as_str(), "library");
}

/// Verify Preferable policy with valid user override returns user type.
#[test]
fn test_validate_policy_preferable_with_override() {
    let validator = RepositoryTypeValidator::new();
    let spec = RepositoryTypeSpec {
        repository_type: "library".to_string(),
        policy: RepositoryTypePolicy::Preferable,
    };
    let user_override = RepositoryTypeName::try_new("documentation").unwrap();

    let result = validator
        .validate_type_policy(&spec, Some(&user_override))
        .unwrap();
    assert_eq!(result.as_str(), "documentation");
}

/// Verify policy enforcement with different types.
#[test]
fn test_validate_policy_various_types() {
    let validator = RepositoryTypeValidator::new();

    // Test multiple repository types
    let types = vec!["service", "library", "documentation", "github-action"];

    for type_name in types {
        let spec = RepositoryTypeSpec {
            repository_type: type_name.to_string(),
            policy: RepositoryTypePolicy::Preferable,
        };

        let result = validator.validate_type_policy(&spec, None).unwrap();
        assert_eq!(result.as_str(), type_name);
    }
}

// ============================================================================
// Custom Property Creation Tests
// ============================================================================

/// Verify custom property creation has correct structure.
#[test]
fn test_create_custom_property_structure() {
    let validator = RepositoryTypeValidator::new();
    let type_name = RepositoryTypeName::try_new("library").unwrap();

    let property = validator.create_type_custom_property(&type_name);

    assert_eq!(property.property_name, "repository_type");
}

/// Verify custom property uses SingleSelect value type.
#[test]
fn test_create_custom_property_value_type() {
    let validator = RepositoryTypeValidator::new();
    let type_name = RepositoryTypeName::try_new("service").unwrap();

    let property = validator.create_type_custom_property(&type_name);

    match property.value {
        CustomPropertyValue::SingleSelect(ref value) => {
            assert_eq!(value, "service");
        }
        _ => panic!("Expected SingleSelect variant"),
    }
}

/// Verify custom property creation for various types.
#[test]
fn test_create_custom_property_various_types() {
    let validator = RepositoryTypeValidator::new();

    let types = vec!["library", "service", "documentation", "github-action"];

    for type_str in types {
        let type_name = RepositoryTypeName::try_new(type_str).unwrap();
        let property = validator.create_type_custom_property(&type_name);

        assert_eq!(property.property_name, "repository_type");

        match property.value {
            CustomPropertyValue::SingleSelect(ref value) => {
                assert_eq!(value, type_str);
            }
            _ => panic!("Expected SingleSelect variant"),
        }
    }
}

/// Verify custom property can be serialized.
#[test]
fn test_create_custom_property_serializable() {
    let validator = RepositoryTypeValidator::new();
    let type_name = RepositoryTypeName::try_new("library").unwrap();

    let property = validator.create_type_custom_property(&type_name);

    // Verify it can be serialized to TOML
    let serialized = toml::to_string(&property);
    assert!(serialized.is_ok());
}
