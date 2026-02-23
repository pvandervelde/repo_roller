#[cfg(test)]
mod notification_endpoint_tests {
    use crate::settings::notifications::NotificationEndpoint;

    #[test]
    fn validate_accepts_valid_https_endpoint() {
        let endpoint = NotificationEndpoint {
            url: "https://example.com/webhook".to_string(),
            secret: "my-secret".to_string(),
            events: vec!["repository.created".to_string()],
            active: true,
            timeout_seconds: 10,
            description: Some("Test webhook".to_string()),
        };

        assert!(endpoint.validate().is_ok());
    }

    #[test]
    fn validate_rejects_http_endpoint() {
        let endpoint = NotificationEndpoint {
            url: "http://example.com/webhook".to_string(),
            secret: "my-secret".to_string(),
            events: vec!["repository.created".to_string()],
            active: true,
            timeout_seconds: 10,
            description: None,
        };

        let result = endpoint.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("HTTPS"));
    }

    #[test]
    fn validate_rejects_empty_secret() {
        let endpoint = NotificationEndpoint {
            url: "https://example.com/webhook".to_string(),
            secret: "".to_string(),
            events: vec!["repository.created".to_string()],
            active: true,
            timeout_seconds: 10,
            description: None,
        };

        let result = endpoint.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Secret"));
    }

    #[test]
    fn validate_rejects_empty_events_list() {
        let endpoint = NotificationEndpoint {
            url: "https://example.com/webhook".to_string(),
            secret: "my-secret".to_string(),
            events: vec![],
            active: true,
            timeout_seconds: 10,
            description: None,
        };

        let result = endpoint.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("events"));
    }

    #[test]
    fn validate_rejects_timeout_below_minimum() {
        let endpoint = NotificationEndpoint {
            url: "https://example.com/webhook".to_string(),
            secret: "my-secret".to_string(),
            events: vec!["repository.created".to_string()],
            active: true,
            timeout_seconds: 0,
            description: None,
        };

        let result = endpoint.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("timeout_seconds"));
    }

    #[test]
    fn validate_rejects_timeout_above_maximum() {
        let endpoint = NotificationEndpoint {
            url: "https://example.com/webhook".to_string(),
            secret: "my-secret".to_string(),
            events: vec!["repository.created".to_string()],
            active: true,
            timeout_seconds: 31,
            description: None,
        };

        let result = endpoint.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("timeout_seconds"));
    }

    #[test]
    fn validate_accepts_minimum_timeout() {
        let endpoint = NotificationEndpoint {
            url: "https://example.com/webhook".to_string(),
            secret: "my-secret".to_string(),
            events: vec!["repository.created".to_string()],
            active: true,
            timeout_seconds: 1,
            description: None,
        };

        assert!(endpoint.validate().is_ok());
    }

    #[test]
    fn validate_accepts_maximum_timeout() {
        let endpoint = NotificationEndpoint {
            url: "https://example.com/webhook".to_string(),
            secret: "my-secret".to_string(),
            events: vec!["repository.created".to_string()],
            active: true,
            timeout_seconds: 30,
            description: None,
        };

        assert!(endpoint.validate().is_ok());
    }

    #[test]
    fn validate_accepts_wildcard_event() {
        let endpoint = NotificationEndpoint {
            url: "https://example.com/webhook".to_string(),
            secret: "my-secret".to_string(),
            events: vec!["*".to_string()],
            active: true,
            timeout_seconds: 10,
            description: None,
        };

        assert!(endpoint.validate().is_ok());
    }

    #[test]
    fn accepts_event_returns_false_when_inactive() {
        let endpoint = NotificationEndpoint {
            url: "https://example.com/webhook".to_string(),
            secret: "my-secret".to_string(),
            events: vec!["repository.created".to_string()],
            active: false,
            timeout_seconds: 10,
            description: None,
        };

        assert!(!endpoint.accepts_event("repository.created"));
    }

    #[test]
    fn accepts_event_returns_true_for_matching_event() {
        let endpoint = NotificationEndpoint {
            url: "https://example.com/webhook".to_string(),
            secret: "my-secret".to_string(),
            events: vec!["repository.created".to_string()],
            active: true,
            timeout_seconds: 10,
            description: None,
        };

        assert!(endpoint.accepts_event("repository.created"));
    }

    #[test]
    fn accepts_event_returns_false_for_non_matching_event() {
        let endpoint = NotificationEndpoint {
            url: "https://example.com/webhook".to_string(),
            secret: "my-secret".to_string(),
            events: vec!["repository.created".to_string()],
            active: true,
            timeout_seconds: 10,
            description: None,
        };

        assert!(!endpoint.accepts_event("repository.deleted"));
    }

    #[test]
    fn accepts_event_returns_true_for_wildcard() {
        let endpoint = NotificationEndpoint {
            url: "https://example.com/webhook".to_string(),
            secret: "my-secret".to_string(),
            events: vec!["*".to_string()],
            active: true,
            timeout_seconds: 10,
            description: None,
        };

        assert!(endpoint.accepts_event("repository.created"));
        assert!(endpoint.accepts_event("repository.deleted"));
        assert!(endpoint.accepts_event("any.event"));
    }

    #[test]
    fn accepts_event_matches_multiple_configured_events() {
        let endpoint = NotificationEndpoint {
            url: "https://example.com/webhook".to_string(),
            secret: "my-secret".to_string(),
            events: vec![
                "repository.created".to_string(),
                "repository.deleted".to_string(),
            ],
            active: true,
            timeout_seconds: 10,
            description: None,
        };

        assert!(endpoint.accepts_event("repository.created"));
        assert!(endpoint.accepts_event("repository.deleted"));
        assert!(!endpoint.accepts_event("repository.updated"));
    }
}

/// Tests that verify the documented `notifications.toml` TOML format parses correctly.
///
/// These tests serve as living documentation for the accepted configuration syntax,
/// ensuring that any format shown in `docs/notifications.md` actually works at runtime.
#[cfg(test)]
mod toml_format_tests {
    use crate::settings::notifications::{NotificationEndpoint, NotificationsConfig};

    #[test]
    fn deserialize_full_notifications_config_from_toml() {
        let toml = r#"
[[outbound_webhooks]]
url = "https://monitoring.example.com/hooks/repo-created"
secret = "REPOROLLER_ORG_WEBHOOK_SECRET"
events = ["repository.created"]
active = true
timeout_seconds = 10
description = "Central monitoring system"
"#;

        let config: NotificationsConfig = toml::from_str(toml).expect("valid TOML should parse");
        assert_eq!(config.outbound_webhooks.len(), 1);
        let ep = &config.outbound_webhooks[0];
        assert_eq!(ep.url, "https://monitoring.example.com/hooks/repo-created");
        assert_eq!(ep.secret, "REPOROLLER_ORG_WEBHOOK_SECRET");
        assert_eq!(ep.events, vec!["repository.created"]);
        assert!(ep.active);
        assert_eq!(ep.timeout_seconds, 10);
        assert_eq!(ep.description.as_deref(), Some("Central monitoring system"));
    }

    #[test]
    fn deserialize_minimal_notifications_config_applies_defaults() {
        // Only required fields: url, secret, events
        let toml = r#"
[[outbound_webhooks]]
url = "https://hooks.example.com/repo"
secret = "WEBHOOK_SECRET"
events = ["repository.created"]
"#;

        let config: NotificationsConfig = toml::from_str(toml).expect("minimal TOML should parse");
        assert_eq!(config.outbound_webhooks.len(), 1);
        let ep = &config.outbound_webhooks[0];
        // Verify defaults
        assert!(ep.active, "active defaults to true");
        assert_eq!(ep.timeout_seconds, 5, "timeout_seconds defaults to 5");
        assert!(ep.description.is_none(), "description defaults to None");
    }

    #[test]
    fn deserialize_multiple_endpoints_from_toml() {
        let toml = r#"
[[outbound_webhooks]]
url = "https://ci.example.com/webhook"
secret = "CI_WEBHOOK_SECRET"
events = ["repository.created"]
description = "CI system"

[[outbound_webhooks]]
url = "https://audit.example.com/webhook"
secret = "AUDIT_WEBHOOK_SECRET"
events = ["*"]
active = false
timeout_seconds = 30
description = "Audit log (disabled)"
"#;

        let config: NotificationsConfig =
            toml::from_str(toml).expect("multi-endpoint TOML should parse");
        assert_eq!(config.outbound_webhooks.len(), 2);

        let ci = &config.outbound_webhooks[0];
        assert_eq!(ci.url, "https://ci.example.com/webhook");
        assert!(ci.active);

        let audit = &config.outbound_webhooks[1];
        assert_eq!(audit.events, vec!["*"]);
        assert!(!audit.active);
        assert_eq!(audit.timeout_seconds, 30);
    }

    #[test]
    fn deserialize_empty_notifications_config_from_toml() {
        let toml = "";
        let config: NotificationsConfig = toml::from_str(toml).expect("empty TOML should parse");
        assert!(config.outbound_webhooks.is_empty());
    }

    #[test]
    fn deserialize_endpoint_with_multiple_events() {
        let toml = r#"
[[outbound_webhooks]]
url = "https://example.com/webhook"
secret = "MY_SECRET"
events = ["repository.created", "repository.deleted", "repository.updated"]
"#;

        let config: NotificationsConfig =
            toml::from_str(toml).expect("multi-event TOML should parse");
        let ep = &config.outbound_webhooks[0];
        assert_eq!(ep.events.len(), 3);
        assert!(ep.events.contains(&"repository.created".to_string()));
    }

    #[test]
    fn deserialize_endpoint_with_wildcard_event() {
        let toml = r#"
[[outbound_webhooks]]
url = "https://example.com/webhook"
secret = "MY_SECRET"
events = ["*"]
"#;

        let config: NotificationsConfig = toml::from_str(toml).expect("wildcard TOML should parse");
        let ep = &config.outbound_webhooks[0];
        assert_eq!(ep.events, vec!["*"]);
        assert!(ep.accepts_event("repository.created"));
        assert!(ep.accepts_event("any.event.type"));
    }

    #[test]
    fn serialize_notifications_config_to_toml_round_trips() {
        let config = NotificationsConfig {
            outbound_webhooks: vec![NotificationEndpoint {
                url: "https://example.com/webhook".to_string(),
                secret: "MY_SECRET".to_string(),
                events: vec!["repository.created".to_string()],
                active: true,
                timeout_seconds: 5,
                description: Some("Test endpoint".to_string()),
            }],
        };

        let serialized = toml::to_string(&config).expect("should serialize to TOML");
        let deserialized: NotificationsConfig =
            toml::from_str(&serialized).expect("serialized TOML should round-trip");
        assert_eq!(config, deserialized);
    }
}
