//! Tests for repository type name validation.

use super::*;

/// Verify valid repository type names are accepted.
#[test]
fn test_valid_repository_type_names() {
    // Simple lowercase names
    assert!(RepositoryTypeName::try_new("library").is_ok());
    assert!(RepositoryTypeName::try_new("service").is_ok());
    assert!(RepositoryTypeName::try_new("documentation").is_ok());

    // Names with hyphens (not at start/end)
    assert!(RepositoryTypeName::try_new("rust-library").is_ok());
    assert!(RepositoryTypeName::try_new("micro-service").is_ok());
    assert!(RepositoryTypeName::try_new("team-docs").is_ok());

    // Names with underscores
    assert!(RepositoryTypeName::try_new("github_action").is_ok());
    assert!(RepositoryTypeName::try_new("personal_project").is_ok());

    // Names with numbers
    assert!(RepositoryTypeName::try_new("library2").is_ok());
    assert!(RepositoryTypeName::try_new("v2service").is_ok());

    // Single character (minimum length)
    assert!(RepositoryTypeName::try_new("l").is_ok());

    // Maximum length (50 characters)
    let max_length = "a".repeat(50);
    assert!(RepositoryTypeName::try_new(&max_length).is_ok());
}

/// Verify invalid repository type names are rejected.
#[test]
fn test_invalid_empty_name() {
    let result = RepositoryTypeName::try_new("");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(
        err,
        ConfigurationError::InvalidConfiguration { .. }
    ));
}

/// Verify names that are too long are rejected.
#[test]
fn test_invalid_too_long() {
    let too_long = "a".repeat(51);
    let result = RepositoryTypeName::try_new(&too_long);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(
        err,
        ConfigurationError::InvalidConfiguration { .. }
    ));
}

/// Verify uppercase letters are rejected.
#[test]
fn test_invalid_uppercase() {
    assert!(RepositoryTypeName::try_new("Library").is_err());
    assert!(RepositoryTypeName::try_new("LIBRARY").is_err());
    assert!(RepositoryTypeName::try_new("myLibrary").is_err());
}

/// Verify names starting with hyphen are rejected.
#[test]
fn test_invalid_starts_with_hyphen() {
    let result = RepositoryTypeName::try_new("-library");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(
        err,
        ConfigurationError::InvalidConfiguration { .. }
    ));
}

/// Verify names ending with hyphen are rejected.
#[test]
fn test_invalid_ends_with_hyphen() {
    let result = RepositoryTypeName::try_new("library-");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(
        err,
        ConfigurationError::InvalidConfiguration { .. }
    ));
}

/// Verify names with invalid characters are rejected.
#[test]
fn test_invalid_special_characters() {
    assert!(RepositoryTypeName::try_new("library.rs").is_err());
    assert!(RepositoryTypeName::try_new("library@v1").is_err());
    assert!(RepositoryTypeName::try_new("library/service").is_err());
    assert!(RepositoryTypeName::try_new("library service").is_err());
    assert!(RepositoryTypeName::try_new("library#1").is_err());
}

/// Verify as_str() returns the correct value.
#[test]
fn test_as_str() {
    let name = RepositoryTypeName::try_new("library").unwrap();
    assert_eq!(name.as_str(), "library");
}

/// Verify into_string() returns the correct value.
#[test]
fn test_into_string() {
    let name = RepositoryTypeName::try_new("service").unwrap();
    assert_eq!(name.into_string(), "service");
}

/// Verify Display trait formatting.
#[test]
fn test_display() {
    let name = RepositoryTypeName::try_new("documentation").unwrap();
    assert_eq!(format!("{}", name), "documentation");
}

/// Verify AsRef<str> trait implementation.
#[test]
fn test_as_ref() {
    let name = RepositoryTypeName::try_new("library").unwrap();
    let s: &str = name.as_ref();
    assert_eq!(s, "library");
}

/// Verify Deref trait allows dereferencing to str.
#[test]
fn test_deref() {
    let name = RepositoryTypeName::try_new("library").unwrap();
    // Deref allows using methods on &str directly
    assert_eq!(name.len(), 7);
    assert!(name.starts_with("lib"));
    assert!(name.contains("bra"));
}

/// Verify Borrow<str> trait implementation.
#[test]
fn test_borrow() {
    use std::borrow::Borrow;

    let name = RepositoryTypeName::try_new("library").unwrap();
    let borrowed: &str = name.borrow();
    assert_eq!(borrowed, "library");
}

/// Verify From trait converts to String.
#[test]
fn test_from_into_string() {
    let name = RepositoryTypeName::try_new("service").unwrap();
    let s: String = name.into();
    assert_eq!(s, "service");
}

/// Verify Clone trait works correctly.
#[test]
fn test_clone() {
    let name1 = RepositoryTypeName::try_new("library").unwrap();
    let name2 = name1.clone();
    assert_eq!(name1, name2);
}

/// Verify PartialEq trait works correctly.
#[test]
fn test_equality() {
    let name1 = RepositoryTypeName::try_new("library").unwrap();
    let name2 = RepositoryTypeName::try_new("library").unwrap();
    let name3 = RepositoryTypeName::try_new("service").unwrap();

    assert_eq!(name1, name2);
    assert_ne!(name1, name3);
}

/// Verify serialization to TOML.
#[test]
fn test_serialize() {
    use serde::Serialize;

    #[derive(Serialize)]
    struct Wrapper {
        name: RepositoryTypeName,
    }

    let name = RepositoryTypeName::try_new("library").unwrap();
    let wrapper = Wrapper { name };
    let serialized = toml::to_string(&wrapper).expect("Failed to serialize");
    assert!(serialized.contains("library"));
}

/// Verify deserialization from TOML.
#[test]
fn test_deserialize() {
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Wrapper {
        name: RepositoryTypeName,
    }

    let toml = r#"name = "library""#;
    let wrapper: Wrapper = toml::from_str(toml).expect("Failed to deserialize");
    assert_eq!(wrapper.name.as_str(), "library");
}

/// Verify round-trip serialization/deserialization.
#[test]
fn test_serialize_deserialize_round_trip() {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Wrapper {
        name: RepositoryTypeName,
    }

    let original = Wrapper {
        name: RepositoryTypeName::try_new("microservice").unwrap(),
    };
    let serialized = toml::to_string(&original).expect("Failed to serialize");
    let deserialized: Wrapper = toml::from_str(&serialized).expect("Failed to deserialize");
    assert_eq!(original, deserialized);
}
