//! Tests for WebhookConfig
use super::*;
#[test]
fn test_webhook_creation() {
    let webhook = WebhookConfig {
        url: "https://example.com".to_string(),
        content_type: "json".to_string(),
        secret: None,
        active: true,
        events: vec!["push".to_string()],
    };
    assert_eq!(webhook.url, "https://example.com");
}
