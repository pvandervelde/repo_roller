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

/// Verify the server builds a router using the configured state.
#[test]
fn test_server_router_uses_state() {
    let state = AppState::new("custom-metadata-repo");
    let config = ApiConfig {
        host: "127.0.0.1".to_string(),
        port: 9090,
    };
    let server = ApiServer::new(config, state);
    // router() should succeed without panicking
    let _router = server.router();
}
