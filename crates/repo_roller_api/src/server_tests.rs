//! Tests for server module

use super::*;

#[test]
fn test_default_config() {
    let config = ApiConfig::default();
    assert_eq!(config.port, DEFAULT_PORT);
    assert_eq!(config.host, "0.0.0.0");
}

#[test]
fn test_server_creation() {
    let state = AppState::default();
    let config = ApiConfig::default();
    let server = ApiServer::new(config, state);
    let _router = server.router();
    // Server and router creation should succeed
}

// TODO: Add integration tests for server startup
