//! Unit tests for the config_manager crate.

use super::*; // Import items from the parent module (lib.rs)
use std::io::Write;
use tempfile::NamedTempFile;

/// Test loading a valid configuration file.
#[test]
fn test_load_valid_config() {
    let content = r#"
[[templates]]
name = "rust-library"
source_repo = "template-org/rust-library-template"

[[templates]]
name = "python-service"
source_repo = "template-org/python-service-template"
"#;

    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(content.as_bytes())
        .expect("Failed to write to temp file");

    let config_result = load_config(file.path());

    assert!(config_result.is_ok());
    let config = config_result.unwrap();

    assert_eq!(config.templates.len(), 2);
    assert_eq!(config.templates[0].name, "rust-library");
    assert_eq!(
        config.templates[0].source_repo,
        "template-org/rust-library-template"
    );
    assert_eq!(config.templates[1].name, "python-service");
    assert_eq!(
        config.templates[1].source_repo,
        "template-org/python-service-template"
    );
}

/// Test loading a configuration file with invalid TOML syntax.
#[test]
fn test_load_invalid_toml() {
    let content = r#"
[[templates] # Missing closing bracket
name = "rust-library"
source_repo = "template-org/rust-library-template"
"#;

    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(content.as_bytes())
        .expect("Failed to write to temp file");

    let config_result = load_config(file.path());

    assert!(config_result.is_err());
    match config_result.err().unwrap() {
        ConfigError::Toml(_) => {} // Expected error type
        _ => panic!("Expected Toml error, got different error"),
    }
}

/// Test loading a non-existent configuration file.
#[test]
fn test_load_non_existent_file() {
    let path = Path::new("non_existent_config_file.toml");
    let config_result = load_config(path);

    assert!(config_result.is_err());
    match config_result.err().unwrap() {
        ConfigError::Io(_) => {} // Expected error type
        _ => panic!("Expected Io error, got different error"),
    }
}
