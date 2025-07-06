use super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_app_config_default() {
    let config = AppConfig::default();

    assert!(config.core.templates.is_empty());
    assert_eq!(config.authentication.auth_method, "token");
}

#[test]
fn test_app_config_load_invalid_toml() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("invalid_config.toml");

    // Write invalid TOML content
    fs::write(&config_path, "invalid = toml = syntax").expect("Failed to write invalid TOML");

    let result = AppConfig::load(&config_path);

    assert!(result.is_err());
    if let Err(Error::Config(msg)) = result {
        assert!(msg.contains("Failed to parse configuration file"));
    } else {
        panic!("Expected Config error");
    }
}

#[test]
fn test_app_config_load_nonexistent_file() {
    let nonexistent_path = PathBuf::from("nonexistent_config.toml");
    let result = AppConfig::load(&nonexistent_path);

    assert!(result.is_err());
    if let Err(Error::Config(msg)) = result {
        assert!(msg.contains("Configuration file not found"));
    } else {
        panic!("Expected Config error");
    }
}

#[test]
fn test_app_config_save_and_load() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("test_config.toml");

    // Create a test configuration
    let original_config = AppConfig {
        core: Config { templates: vec![] },
        authentication: AuthenticationConfig {
            auth_method: "token".to_string(),
        },
    };

    // Save the configuration
    original_config
        .save(&config_path)
        .expect("Failed to save config");

    // Verify file was created
    assert!(config_path.exists());

    // Load the configuration back
    let loaded_config = AppConfig::load(&config_path).expect("Failed to load config");

    // Verify the loaded configuration matches
    assert_eq!(loaded_config.authentication.auth_method, "token");
    assert!(loaded_config.core.templates.is_empty());
}

#[test]
fn test_app_config_save_creates_parent_directories() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let nested_path = temp_dir
        .path()
        .join("nested")
        .join("directory")
        .join("config.toml");

    let config = AppConfig::default();

    // Save should create parent directories
    config
        .save(&nested_path)
        .expect("Failed to save config with nested path");

    // Verify file was created
    assert!(nested_path.exists());

    // Verify we can load it back
    let loaded_config = AppConfig::load(&nested_path).expect("Failed to load config");
    assert_eq!(loaded_config.authentication.auth_method, "token");
}

#[test]
fn test_authentication_config_default() {
    let auth_config = AuthenticationConfig::default();
    assert_eq!(auth_config.auth_method, "token");
}

#[test]
fn test_authentication_config_new() {
    let auth_config = AuthenticationConfig::new();
    assert_eq!(auth_config.auth_method, "token");
}

#[test]
fn test_get_config_path_with_none() {
    let result = get_config_path(None);
    let expected = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(DEFAULT_CONFIG_FILENAME);
    assert_eq!(result, expected);
}

#[test]
fn test_get_config_path_with_provided_path() {
    let custom_path = "/custom/path/config.toml";
    let result = get_config_path(Some(custom_path));
    assert_eq!(result, PathBuf::from(custom_path));
}
