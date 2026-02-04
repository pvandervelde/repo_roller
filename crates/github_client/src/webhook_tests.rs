//! Tests for webhook types.

use super::*;

#[test]
fn test_webhook_event_serialization() {
    // Test individual event serialization
    assert_eq!(
        serde_json::to_string(&WebhookEvent::Push).unwrap(),
        r#""push""#
    );
    assert_eq!(
        serde_json::to_string(&WebhookEvent::PullRequest).unwrap(),
        r#""pull_request""#
    );
    assert_eq!(serde_json::to_string(&WebhookEvent::All).unwrap(), r#""*""#);
}

#[test]
fn test_webhook_event_deserialization() {
    // Test individual event deserialization
    assert_eq!(
        serde_json::from_str::<WebhookEvent>(r#""push""#).unwrap(),
        WebhookEvent::Push
    );
    assert_eq!(
        serde_json::from_str::<WebhookEvent>(r#""pull_request""#).unwrap(),
        WebhookEvent::PullRequest
    );
    assert_eq!(
        serde_json::from_str::<WebhookEvent>(r#""*""#).unwrap(),
        WebhookEvent::All
    );
}

#[test]
fn test_webhook_event_from_str() {
    use std::str::FromStr;

    assert_eq!(WebhookEvent::from_str("push"), Ok(WebhookEvent::Push));
    assert_eq!(
        WebhookEvent::from_str("pull_request"),
        Ok(WebhookEvent::PullRequest)
    );
    assert_eq!(WebhookEvent::from_str("*"), Ok(WebhookEvent::All));
    assert!(WebhookEvent::from_str("invalid").is_err());
}

#[test]
fn test_webhook_event_as_str() {
    assert_eq!(WebhookEvent::Push.as_str(), "push");
    assert_eq!(WebhookEvent::PullRequest.as_str(), "pull_request");
    assert_eq!(WebhookEvent::All.as_str(), "*");
}

#[test]
fn test_webhook_serialization() {
    let webhook = Webhook {
        id: 12345,
        url: "https://example.com/webhook".to_string(),
        active: true,
        events: vec![WebhookEvent::Push, WebhookEvent::PullRequest],
        config: WebhookDetails {
            url: "https://example.com/webhook".to_string(),
            content_type: "json".to_string(),
            insecure_ssl: false,
        },
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
    };

    let json = serde_json::to_string(&webhook).unwrap();
    let deserialized: Webhook = serde_json::from_str(&json).unwrap();
    assert_eq!(webhook, deserialized);
}

#[test]
fn test_webhook_deserialization_from_github_api() {
    // Test that we can deserialize from actual GitHub API format
    let github_json = r#"{
        "id": 12345,
        "url": "https://example.com/webhook",
        "active": true,
        "events": ["push", "pull_request"],
        "config": {
            "url": "https://example.com/webhook",
            "content_type": "json",
            "insecure_ssl": "0"
        },
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z"
    }"#;

    let webhook: Webhook = serde_json::from_str(github_json).unwrap();
    assert_eq!(webhook.id, 12345);
    assert_eq!(webhook.url, "https://example.com/webhook");
    assert_eq!(webhook.events.len(), 2);
    assert_eq!(webhook.events[0], WebhookEvent::Push);
    assert_eq!(webhook.events[1], WebhookEvent::PullRequest);
    assert!(!webhook.config.insecure_ssl); // "0" -> false
}

#[test]
fn test_webhook_details_insecure_ssl_serialization() {
    // Test secure (verify SSL)
    let secure_details = WebhookDetails {
        url: "https://example.com".to_string(),
        content_type: "json".to_string(),
        insecure_ssl: false,
    };
    let json = serde_json::to_value(&secure_details).unwrap();
    assert_eq!(json["insecure_ssl"], "0");

    // Test insecure (skip SSL verification)
    let insecure_details = WebhookDetails {
        url: "https://example.com".to_string(),
        content_type: "json".to_string(),
        insecure_ssl: true,
    };
    let json = serde_json::to_value(&insecure_details).unwrap();
    assert_eq!(json["insecure_ssl"], "1");
}

#[test]
fn test_webhook_details_insecure_ssl_deserialization() {
    // Test "0" -> false
    let json = r#"{"url": "https://example.com", "content_type": "json", "insecure_ssl": "0"}"#;
    let details: WebhookDetails = serde_json::from_str(json).unwrap();
    assert!(!details.insecure_ssl);

    // Test "1" -> true
    let json = r#"{"url": "https://example.com", "content_type": "json", "insecure_ssl": "1"}"#;
    let details: WebhookDetails = serde_json::from_str(json).unwrap();
    assert!(details.insecure_ssl);
}

#[test]
fn test_webhook_details_default_insecure_ssl() {
    // Test that insecure_ssl defaults to false (secure) when not provided
    let json = r#"{"url": "https://example.com", "content_type": "json"}"#;
    let details: WebhookDetails = serde_json::from_str(json).unwrap();
    assert!(!details.insecure_ssl); // Should default to false (secure)
}
