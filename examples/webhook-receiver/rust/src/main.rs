//! RepoRoller Webhook Receiver — Rust Example
//! ============================================
//! Demonstrates how to receive and verify RepoRoller outbound webhook notifications.
//!
//! # Requirements
//!
//! Dependencies (Cargo.toml):
//! ```toml
//! [dependencies]
//! axum          = "0.7"
//! tokio         = { version = "1", features = ["full"] }
//! hmac          = "0.12"
//! sha2          = "0.10"
//! hex           = "0.4"
//! serde         = { version = "1", features = ["derive"] }
//! serde_json    = "1"
//! tracing       = "0.1"
//! tracing-subscriber = { version = "0.3", features = ["env-filter"] }
//! ```
//!
//! # Usage
//!
//! ```sh
//! WEBHOOK_SECRET="your-shared-secret-value" cargo run
//! ```
//!
//! The server listens on port 8080 and accepts POST /webhook requests.
//!
//! See docs/notifications.md for full webhook documentation.

use std::{collections::HashMap, net::SocketAddr};

use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
    Router,
};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use tracing::{error, info, warn};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct AppState {
    secret: Vec<u8>,
}

/// All fields sent in a `repository.created` event.
/// Fields marked with `Option` are optional in the payload.
#[derive(Debug, Deserialize)]
struct RepositoryCreatedPayload {
    event_type: String,
    event_id: String,
    timestamp: String,
    organization: String,
    repository_name: String,
    repository_url: String,
    repository_id: String,
    created_by: String,
    repository_type: Option<String>,
    template_name: Option<String>,
    content_strategy: String,
    visibility: String,
    team: Option<String>,
    description: Option<String>,
    custom_properties: Option<HashMap<String, String>>,
    // applied_settings omitted for brevity; add if needed
}

/// Minimal struct used only to read `event_type` before full deserialisation.
#[derive(Deserialize)]
struct EventEnvelope {
    event_type: String,
}

// ---------------------------------------------------------------------------
// Signature verification
// ---------------------------------------------------------------------------

/// Returns `true` when `signature_header` matches the HMAC-SHA256 of `body`
/// using `secret`.
///
/// Uses a constant-time comparison to prevent timing attacks.
fn verify_signature(body: &[u8], signature_header: &str, secret: &[u8]) -> bool {
    let Some(hex_part) = signature_header.strip_prefix("sha256=") else {
        return false;
    };

    let Ok(received_bytes) = hex::decode(hex_part) else {
        return false;
    };

    let mut mac = Hmac::<Sha256>::new_from_slice(secret)
        .expect("HMAC accepts any key length");
    mac.update(body);

    // `verify_slice` uses constant-time comparison internally.
    mac.verify_slice(&received_bytes).is_ok()
}

// ---------------------------------------------------------------------------
// Event handlers
// ---------------------------------------------------------------------------

fn handle_repository_created(payload: RepositoryCreatedPayload) {
    info!(
        event_id = %payload.event_id,
        org = %payload.organization,
        name = %payload.repository_name,
        url = %payload.repository_url,
        created_by = %payload.created_by,
        visibility = %payload.visibility,
        template = %payload.template_name.as_deref().unwrap_or("(none)"),
        team = %payload.team.as_deref().unwrap_or("(none)"),
        strategy = %payload.content_strategy,
        "Repository created"
    );

    // Add your integration logic here:
    //   - Post a Slack / Teams notification
    //   - Register the repo in a service catalog
    //   - Trigger a CI provisioning pipeline
    //   - Update an asset inventory database
    let _ = payload; // suppress unused warning in example
}

// ---------------------------------------------------------------------------
// HTTP handler
// ---------------------------------------------------------------------------

async fn webhook_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    // 1. Verify signature.
    let sig_header = headers
        .get("x-reporoller-signature-256")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !verify_signature(&body, sig_header, &state.secret) {
        warn!("Rejected request with invalid signature");
        return StatusCode::UNAUTHORIZED.into_response();
    }

    // 2. Determine event type.
    let envelope: EventEnvelope = match serde_json::from_slice(&body) {
        Ok(e) => e,
        Err(err) => {
            error!(?err, "Failed to parse JSON body");
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    // 3. Dispatch on event type.
    match envelope.event_type.as_str() {
        "repository.created" => {
            match serde_json::from_slice::<RepositoryCreatedPayload>(&body) {
                Ok(payload) => handle_repository_created(payload),
                Err(err) => {
                    error!(?err, "Failed to deserialise repository.created payload");
                    return StatusCode::BAD_REQUEST.into_response();
                }
            }
        }
        other => {
            info!(event_type = %other, "Ignoring unknown event type");
        }
    }

    // Always acknowledge promptly — processing is fire-and-forget from sender's perspective.
    StatusCode::NO_CONTENT.into_response()
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let secret = std::env::var("WEBHOOK_SECRET").unwrap_or_else(|_| {
        error!("WEBHOOK_SECRET environment variable is not set");
        std::process::exit(1);
    });

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let state = AppState {
        secret: secret.into_bytes(),
    };

    let app = Router::new()
        .route("/webhook", post(webhook_handler))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!(%addr, "RepoRoller webhook receiver listening");
    // In production, terminate TLS at a reverse proxy / load balancer.
    // Your notifications.toml endpoint URL must use https://.
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
