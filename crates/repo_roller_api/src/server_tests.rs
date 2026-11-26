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
    let state = AppState::new(
        "custom-metadata-repo",
        0,
        "",
        "test-jwt-secret-key-minimum-32b!",
    );
    let config = ApiConfig {
        host: "127.0.0.1".to_string(),
        port: 9090,
    };
    let server = ApiServer::new(config, state);
    // router() should succeed without panicking
    let _router = server.router();
}

/// Sends one raw HTTP/1.1 POST to the token endpoint and returns the status code.
async fn raw_token_post(local: Option<std::net::SocketAddr>, to: std::net::SocketAddr) -> Option<u16> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let sock = tokio::net::TcpSocket::new_v4().ok()?;
    if let Some(l) = local {
        sock.bind(l).ok()?;
    }
    let mut s = sock.connect(to).await.ok()?;
    s.write_all(
        b"POST /api/v1/auth/token HTTP/1.1
Host: x
Content-Length: 0
Connection: close

",
    )
    .await
    .ok()?;
    let mut buf = Vec::new();
    s.read_to_end(&mut buf).await.ok()?;
    let text = String::from_utf8_lossy(&buf).into_owned();
    text.split_whitespace().nth(1)?.parse().ok()
}

/// The production serve path must expose the peer address as `ConnectInfo`
/// so the token-endpoint limiter keys by client IP rather than falling back to
/// one shared bucket.
#[tokio::test]
async fn serve_with_listener_keys_token_limiter_by_peer_ip() {
    use crate::rate_limit::{RateLimitConfig, RateLimiter};
    let limiter = std::sync::Arc::new(RateLimiter::new(RateLimitConfig::new(true, 1, 60)));
    let state = AppState::default().with_auth_rate_limiter(limiter.clone());
    let server = ApiServer::new(ApiConfig::default(), state);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let handle = tokio::spawn(async move {
        server
            .serve_with_listener(listener, async {
                let _ = rx.await;
            })
            .await
    });

    let t = std::time::Duration::from_secs(10);
    let first = tokio::time::timeout(t, raw_token_post(None, addr)).await.unwrap();
    let second = tokio::time::timeout(t, raw_token_post(None, addr)).await.unwrap();
    assert_eq!(first, Some(401), "first request reaches the handler");
    assert_eq!(second, Some(429), "second request from same peer is limited");

    // A different peer IP must have its own budget (skipped where the OS
    // cannot bind a second loopback address).
    let other: std::net::SocketAddr = "127.0.0.2:0".parse().unwrap();
    if tokio::net::TcpSocket::new_v4().unwrap().bind(other).is_ok() {
        let third = tokio::time::timeout(t, raw_token_post(Some(other), addr))
            .await
            .unwrap();
        assert_eq!(third, Some(401), "different peer ip gets a fresh budget");
        assert_eq!(limiter.tracked_keys(), 2);
    }

    let _ = tx.send(());
    let res = tokio::time::timeout(t, handle).await.unwrap().unwrap();
    assert!(res.is_ok(), "serve_with_listener must shut down cleanly: {res:?}");
}
