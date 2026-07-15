//! Tests for routes module

use super::*;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use tower::ServiceExt; // for .oneshot()

#[test]
fn test_router_creation() {
    let state = AppState::default();
    let _router = create_router(state);
    // Router creation should succeed
}

/// Verify the health check endpoint returns 200 without authentication
/// (health check is publicly accessible by design).
#[tokio::test]
async fn test_health_check_endpoint_returns_200() {
    let state = AppState::default();
    // Use the no-auth router so we isolate just the health handler
    let router = create_router_without_auth(state);

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/health")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

/// Verify that a request without a valid Authorization header is rejected (401).
#[tokio::test]
async fn test_protected_endpoint_requires_auth() {
    let state = AppState::default();
    // Use the full router which includes the auth middleware
    let router = create_router(state);

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/repositories")
        .header("Content-Type", "application/json")
        .body(Body::from("{}"))
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ============================================================================
// /metrics endpoint tests (Observability Phase 1)
//
// Assumption / spec gap: the injected task description names the route
// literally "GET /metrics" (see docs/spec/interfaces/event-metrics.md
// "Integration Pattern", which mounts it unprefixed, sibling to `/api/v1`)
// while also saying "like /health" (which lives at `/api/v1/health`). These
// tests assert the literal, doc-matching path `/metrics` at the router root.
// If the Coder instead wires `/api/v1/metrics`, these tests will fail loudly
// rather than silently passing against the wrong path — flag this ambiguity
// back to the architect if that happens.
// ============================================================================

/// The `/metrics` endpoint must be reachable on the *production* router
/// (the one with real auth middleware applied to protected routes) without
/// any Authorization header — it must never be gated behind auth, exactly
/// like `/health`.
#[tokio::test]
async fn test_metrics_endpoint_reachable_without_auth_on_production_router() {
    let state = AppState::default();
    let router = create_router(state);

    let request = Request::builder()
        .method(Method::GET)
        .uri("/metrics")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "/metrics must be reachable without Authorization, like /health"
    );
}

/// The response `Content-Type` must be the Prometheus text exposition format,
/// not `application/json` (the default for most handlers in this API) — a
/// scraper that receives the wrong content type will refuse to parse the body.
#[tokio::test]
async fn test_metrics_endpoint_returns_prometheus_text_content_type() {
    let state = AppState::default();
    let router = create_router(state);

    let request = Request::builder()
        .method(Method::GET)
        .uri("/metrics")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    let content_type = response
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();

    assert!(
        content_type.starts_with("text/plain"),
        "expected a text/plain (Prometheus exposition format) content type, got '{content_type}'"
    );
}

/// The scraped body must include every metric family named in the
/// acceptance criteria: notification_* (pre-existing), repository_creation_*,
/// http_request(s)_*, and github_api_* — proving all four instrumentation
/// points are wired into the one shared registry exposed by this endpoint.
#[tokio::test]
async fn test_metrics_endpoint_body_includes_all_required_metric_family_prefixes() {
    let state = AppState::default();
    let router = create_router(state);

    let request = Request::builder()
        .method(Method::GET)
        .uri("/metrics")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_text = String::from_utf8(body.to_vec()).expect("Prometheus body must be valid UTF-8");

    for expected_prefix in ["notification_", "repository_creation_", "http_request", "github_api_"] {
        assert!(
            body_text.contains(expected_prefix),
            "expected /metrics body to include a metric family starting with '{expected_prefix}', got:\n{body_text}"
        );
    }
}

/// Scraping `/metrics` twice on the same running application instance must
/// not panic and must expose the same set of metric family names both times
/// — this is the closest black-box proxy for "the registry is retained
/// across requests, not recreated per scrape" without depending on any
/// private `AppState` field name.
#[tokio::test]
async fn test_metrics_endpoint_survives_repeated_scrapes_with_stable_family_names() {
    let state = AppState::default();
    let router = create_router(state);

    let first_request = Request::builder().method(Method::GET).uri("/metrics").body(Body::empty()).unwrap();
    let first_response = router.clone().oneshot(first_request).await.unwrap();
    assert_eq!(first_response.status(), StatusCode::OK);
    let first_body = axum::body::to_bytes(first_response.into_body(), usize::MAX).await.unwrap();
    let first_text = String::from_utf8(first_body.to_vec()).unwrap();

    let second_request = Request::builder().method(Method::GET).uri("/metrics").body(Body::empty()).unwrap();
    let second_response = router.oneshot(second_request).await.unwrap();
    assert_eq!(second_response.status(), StatusCode::OK, "second scrape must not panic or fail");
    let second_body = axum::body::to_bytes(second_response.into_body(), usize::MAX).await.unwrap();
    let second_text = String::from_utf8(second_body.to_vec()).unwrap();

    let family_names_from = |text: &str| -> std::collections::BTreeSet<String> {
        text.lines()
            .filter(|l| l.starts_with("# TYPE "))
            .filter_map(|l| l.split_whitespace().nth(2))
            .map(|s| s.to_string())
            .collect()
    };
    assert_eq!(
        family_names_from(&first_text),
        family_names_from(&second_text),
        "the set of exposed metric families must be identical across repeated scrapes"
    );
}

/// `/metrics` must also be reachable on the no-auth test router, so handler
/// wiring can be exercised without any auth-related test scaffolding.
#[tokio::test]
async fn test_metrics_endpoint_reachable_on_test_router() {
    let state = AppState::default();
    let router = create_router_without_auth(state);

    let request = Request::builder().method(Method::GET).uri("/metrics").body(Body::empty()).unwrap();
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
