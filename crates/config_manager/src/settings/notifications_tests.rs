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
