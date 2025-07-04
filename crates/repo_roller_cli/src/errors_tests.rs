use super::*;
use std::io;

#[test]
fn test_auth_error_display() {
    let error = Error::Auth("Invalid token".to_string());
    assert_eq!(error.to_string(), "Authentication error: Invalid token");
}

#[test]
fn test_config_error_display() {
    let error = Error::Config("Missing field 'name'".to_string());
    assert_eq!(error.to_string(), "Configuration error: Missing field 'name'");
}

#[test]
fn test_error_debug_format() {
    let error = Error::Auth("test".to_string());
    let debug_output = format!("{:?}", error);
    assert!(debug_output.contains("Auth"));
    assert!(debug_output.contains("test"));
}

#[test]
fn test_error_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Error>();
}

#[test]
fn test_invalid_arguments_error_display() {
    let error = Error::InvalidArguments("--name is required".to_string());
    assert_eq!(error.to_string(), "Invalid arguments: --name is required");
}

#[test]
fn test_load_file_error_display() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let error = Error::LoadFile(io_error);
    assert_eq!(error.to_string(), "Failed to load file.");
}

#[test]
fn test_parse_toml_file_error_display() {
    // Create a simple TOML parsing error
    let toml_content = "invalid = toml = syntax";
    let parse_error = toml::from_str::<toml::Value>(toml_content).unwrap_err();
    let error = Error::ParseTomlFile(parse_error);
    assert_eq!(error.to_string(), "Failed to parse TOML configuration file.");
}

#[test]
fn test_stdout_flush_failed_error_display() {
    let error = Error::StdOutFlushFailed;
    assert_eq!(error.to_string(), "Failed to flush the std out buffer.");
}
