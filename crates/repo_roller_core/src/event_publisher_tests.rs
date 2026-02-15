//! Tests for event_publisher module.
//! See docs/spec/interfaces/event-publisher.md for specifications.

use super::*;
use crate::{
    ContentStrategy, OrganizationName, RepositoryCreationRequest, RepositoryCreationResult,
    RepositoryName, RepositoryVisibility, TemplateName, Timestamp,
};
use chrono::Utc;
use std::collections::HashMap;

mod event_serialization_tests {
    use super::*;

    #[test]
    fn test_event_serialization_with_complete_payload() {
        // Arrange: Create event with all fields populated
        let event = RepositoryCreatedEvent {
            event_type: "repository.created".to_string(),
            event_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            timestamp: Utc::now(),
            organization: "test-org".to_string(),
            repository_name: "test-repo".to_string(),
            repository_url: "https://github.com/test-org/test-repo".to_string(),
            repository_id: "R_kgDOH1234567".to_string(),
            created_by: "jane.doe".to_string(),
            repository_type: Some("service".to_string()),
            template_name: Some("rust-service".to_string()),
            content_strategy: "template".to_string(),
            visibility: "private".to_string(),
            team: Some("backend-team".to_string()),
            description: Some("Test repository".to_string()),
            custom_properties: Some({
                let mut props = HashMap::new();
                props.insert("cost_center".to_string(), "engineering".to_string());
                props
            }),
            applied_settings: Some(AppliedSettings {
                has_issues: Some(true),
                has_wiki: Some(false),
                has_projects: Some(true),
                has_discussions: Some(false),
            }),
        };

        // Act: Serialize to JSON
        let json = serde_json::to_string(&event).expect("Serialization should succeed");

        // Assert: Parse back and verify all fields present
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Should be valid JSON");

        // Required fields (12)
        assert_eq!(parsed["event_type"], "repository.created");
        assert_eq!(parsed["event_id"], "550e8400-e29b-41d4-a716-446655440000");
        assert!(
            parsed["timestamp"].is_string(),
            "Timestamp should be string"
        );
        assert_eq!(parsed["organization"], "test-org");
        assert_eq!(parsed["repository_name"], "test-repo");
        assert_eq!(
            parsed["repository_url"],
            "https://github.com/test-org/test-repo"
        );
        assert_eq!(parsed["repository_id"], "R_kgDOH1234567");
        assert_eq!(parsed["created_by"], "jane.doe");
        assert_eq!(parsed["content_strategy"], "template");
        assert_eq!(parsed["visibility"], "private");

        // Optional fields (6) - all present in this test
        assert_eq!(parsed["repository_type"], "service");
        assert_eq!(parsed["template_name"], "rust-service");
        assert_eq!(parsed["team"], "backend-team");
        assert_eq!(parsed["description"], "Test repository");
        assert!(parsed["custom_properties"].is_object());
        assert_eq!(parsed["custom_properties"]["cost_center"], "engineering");
        assert!(parsed["applied_settings"].is_object());
        assert_eq!(parsed["applied_settings"]["has_issues"], true);
        assert_eq!(parsed["applied_settings"]["has_wiki"], false);
    }

    #[test]
    fn test_event_serialization_with_minimal_payload() {
        // Arrange: Create event with only required fields
        let event = RepositoryCreatedEvent {
            event_type: "repository.created".to_string(),
            event_id: "550e8400-e29b-41d4-a716-446655440001".to_string(),
            timestamp: Utc::now(),
            organization: "minimal-org".to_string(),
            repository_name: "minimal-repo".to_string(),
            repository_url: "https://github.com/minimal-org/minimal-repo".to_string(),
            repository_id: "R_kgDOH1234568".to_string(),
            created_by: "john.smith".to_string(),
            repository_type: None,
            template_name: None,
            content_strategy: "empty".to_string(),
            visibility: "public".to_string(),
            team: None,
            description: None,
            custom_properties: None,
            applied_settings: None,
        };

        // Act: Serialize to JSON
        let json = serde_json::to_string(&event).expect("Serialization should succeed");

        // Assert: Optional fields should be omitted from JSON
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Should be valid JSON");

        // Required fields present
        assert_eq!(parsed["event_type"], "repository.created");
        assert_eq!(parsed["organization"], "minimal-org");

        // Optional fields absent
        assert!(
            parsed.get("repository_type").is_none(),
            "repository_type should be omitted when None"
        );
        assert!(
            parsed.get("template_name").is_none(),
            "template_name should be omitted when None"
        );
        assert!(
            parsed.get("team").is_none(),
            "team should be omitted when None"
        );
        assert!(
            parsed.get("description").is_none(),
            "description should be omitted when None"
        );
        assert!(
            parsed.get("custom_properties").is_none(),
            "custom_properties should be omitted when None"
        );
        assert!(
            parsed.get("applied_settings").is_none(),
            "applied_settings should be omitted when None"
        );
    }

    #[test]
    fn test_timestamp_formatted_as_iso8601_utc() {
        // Arrange
        let now = Utc::now();
        let event = RepositoryCreatedEvent {
            event_type: "repository.created".to_string(),
            event_id: "test-id".to_string(),
            timestamp: now,
            organization: "org".to_string(),
            repository_name: "repo".to_string(),
            repository_url: "https://github.com/org/repo".to_string(),
            repository_id: "R_123".to_string(),
            created_by: "user".to_string(),
            repository_type: None,
            template_name: None,
            content_strategy: "empty".to_string(),
            visibility: "private".to_string(),
            team: None,
            description: None,
            custom_properties: None,
            applied_settings: None,
        };

        // Act
        let json = serde_json::to_string(&event).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Assert: Timestamp should be ISO 8601 format with UTC timezone
        let timestamp_str = parsed["timestamp"]
            .as_str()
            .expect("Timestamp should be string");
        assert!(
            timestamp_str.ends_with('Z'),
            "Timestamp should end with 'Z' (UTC)"
        );
        assert!(
            timestamp_str.contains('T'),
            "Timestamp should contain 'T' separator"
        );

        // Verify it can be parsed back
        let _parsed_time: chrono::DateTime<Utc> = timestamp_str
            .parse()
            .expect("Should parse as valid ISO 8601 UTC");
    }

    #[test]
    fn test_event_round_trip_deserialization() {
        // Arrange: Create event and serialize
        let original = RepositoryCreatedEvent {
            event_type: "repository.created".to_string(),
            event_id: "round-trip-test".to_string(),
            timestamp: Utc::now(),
            organization: "test-org".to_string(),
            repository_name: "test-repo".to_string(),
            repository_url: "https://github.com/test-org/test-repo".to_string(),
            repository_id: "R_test".to_string(),
            created_by: "tester".to_string(),
            repository_type: Some("library".to_string()),
            template_name: Some("rust-lib".to_string()),
            content_strategy: "template".to_string(),
            visibility: "internal".to_string(),
            team: Some("platform".to_string()),
            description: Some("Test library".to_string()),
            custom_properties: None,
            applied_settings: Some(AppliedSettings {
                has_issues: Some(true),
                has_wiki: Some(true),
                has_projects: Some(false),
                has_discussions: Some(true),
            }),
        };

        let json = serde_json::to_string(&original).unwrap();

        // Act: Deserialize back
        let deserialized: RepositoryCreatedEvent =
            serde_json::from_str(&json).expect("Should deserialize successfully");

        // Assert: All fields match
        assert_eq!(deserialized.event_type, original.event_type);
        assert_eq!(deserialized.event_id, original.event_id);
        assert_eq!(deserialized.organization, original.organization);
        assert_eq!(deserialized.repository_name, original.repository_name);
        assert_eq!(deserialized.repository_type, original.repository_type);
        assert_eq!(deserialized.template_name, original.template_name);
        assert_eq!(deserialized.content_strategy, original.content_strategy);
        assert_eq!(deserialized.visibility, original.visibility);
        assert_eq!(deserialized.team, original.team);
        assert_eq!(deserialized.description, original.description);
        assert!(deserialized.applied_settings.is_some());
    }
}

mod event_construction_tests {
    use super::*;

    #[test]
    fn test_from_result_and_request_with_template() {
        // Arrange: Create repository creation result and request
        let result = RepositoryCreationResult {
            repository_url: "https://github.com/my-org/my-repo".to_string(),
            repository_id: "R_kgDOH9876543".to_string(),
            created_at: Timestamp::now(),
            default_branch: "main".to_string(),
        };

        let request = RepositoryCreationRequest {
            name: RepositoryName::new("my-repo").unwrap(),
            owner: OrganizationName::new("my-org").unwrap(),
            template: Some(TemplateName::new("rust-service").unwrap()),
            variables: {
                let mut vars = HashMap::new();
                vars.insert("project_name".to_string(), "MyService".to_string());
                vars
            },
            visibility: Some(RepositoryVisibility::Private),
            content_strategy: ContentStrategy::Template,
        };

        let merged_config = config_manager::MergedConfiguration::new();
        let created_by = "jane.doe";

        // Act: Create event from result and request
        let event = RepositoryCreatedEvent::from_result_and_request(
            &result,
            &request,
            &merged_config,
            created_by,
        );

        // Assert: Required fields populated correctly
        assert_eq!(event.event_type, "repository.created");
        assert!(!event.event_id.is_empty(), "event_id should be generated");
        assert_eq!(event.organization, "my-org");
        assert_eq!(event.repository_name, "my-repo");
        assert_eq!(event.repository_url, "https://github.com/my-org/my-repo");
        assert_eq!(event.repository_id, "R_kgDOH9876543");
        assert_eq!(event.created_by, "jane.doe");
        assert_eq!(event.content_strategy, "template");
        assert_eq!(event.visibility, "private");

        // Assert: Optional fields from template request
        assert_eq!(event.template_name, Some("rust-service".to_string()));

        // Verify event_id is valid UUID v4 format
        assert!(
            uuid::Uuid::parse_str(&event.event_id).is_ok(),
            "event_id should be valid UUID"
        );
    }

    #[test]
    fn test_from_result_and_request_empty_repository() {
        // Arrange: Empty repository without template
        let result = RepositoryCreationResult {
            repository_url: "https://github.com/test-org/empty-repo".to_string(),
            repository_id: "R_kgDOH1111111".to_string(),
            created_at: Timestamp::now(),
            default_branch: "main".to_string(),
        };

        let request = RepositoryCreationRequest {
            name: RepositoryName::new("empty-repo").unwrap(),
            owner: OrganizationName::new("test-org").unwrap(),
            template: None,
            variables: HashMap::new(),
            visibility: Some(RepositoryVisibility::Public),
            content_strategy: ContentStrategy::Empty,
        };

        let merged_config = config_manager::MergedConfiguration::new();
        let created_by = "john.smith";

        // Act
        let event = RepositoryCreatedEvent::from_result_and_request(
            &result,
            &request,
            &merged_config,
            created_by,
        );

        // Assert: Required fields
        assert_eq!(event.event_type, "repository.created");
        assert_eq!(event.organization, "test-org");
        assert_eq!(event.repository_name, "empty-repo");
        assert_eq!(event.created_by, "john.smith");
        assert_eq!(event.content_strategy, "empty");
        assert_eq!(event.visibility, "public");

        // Assert: Optional fields should be None for empty repo
        assert_eq!(
            event.template_name, None,
            "template_name should be None for empty repo"
        );
    }

    #[test]
    fn test_from_result_and_request_custom_init() {
        // Arrange: Custom init strategy
        let result = RepositoryCreationResult {
            repository_url: "https://github.com/org/custom-repo".to_string(),
            repository_id: "R_kgDOH2222222".to_string(),
            created_at: Timestamp::now(),
            default_branch: "main".to_string(),
        };

        let request = RepositoryCreationRequest {
            name: RepositoryName::new("custom-repo").unwrap(),
            owner: OrganizationName::new("org").unwrap(),
            template: None,
            variables: HashMap::new(),
            visibility: Some(RepositoryVisibility::Internal),
            content_strategy: ContentStrategy::CustomInit {
                include_readme: true,
                include_gitignore: true,
            },
        };

        let merged_config = config_manager::MergedConfiguration::new();

        // Act
        let event = RepositoryCreatedEvent::from_result_and_request(
            &result,
            &request,
            &merged_config,
            "admin",
        );

        // Assert
        assert_eq!(event.content_strategy, "custom_init");
        assert_eq!(event.visibility, "internal");
        assert_eq!(event.template_name, None);
    }

    #[test]
    fn test_from_result_and_request_extracts_applied_settings() {
        // Arrange: MergedConfiguration with repository settings
        let mut merged_config = config_manager::MergedConfiguration::new();
        merged_config.repository.issues = Some(config_manager::overridable::OverridableValue::new(
            true, true,
        ));
        merged_config.repository.wiki = Some(config_manager::overridable::OverridableValue::new(
            false, true,
        ));
        merged_config.repository.projects = Some(
            config_manager::overridable::OverridableValue::new(true, true),
        );
        merged_config.repository.discussions = Some(
            config_manager::overridable::OverridableValue::new(false, true),
        );

        let result = RepositoryCreationResult {
            repository_url: "https://github.com/org/repo".to_string(),
            repository_id: "R_test".to_string(),
            created_at: Timestamp::now(),
            default_branch: "main".to_string(),
        };

        let request = RepositoryCreationRequest {
            name: RepositoryName::new("repo").unwrap(),
            owner: OrganizationName::new("org").unwrap(),
            template: None,
            variables: HashMap::new(),
            visibility: None,
            content_strategy: ContentStrategy::Empty,
        };

        // Act
        let event = RepositoryCreatedEvent::from_result_and_request(
            &result,
            &request,
            &merged_config,
            "user",
        );

        // Assert: applied_settings extracted from merged_config
        assert!(
            event.applied_settings.is_some(),
            "applied_settings should be populated"
        );
        let settings = event.applied_settings.unwrap();
        assert_eq!(settings.has_issues, Some(true));
        assert_eq!(settings.has_wiki, Some(false));
        assert_eq!(settings.has_projects, Some(true));
        assert_eq!(settings.has_discussions, Some(false));
    }

    #[test]
    fn test_from_result_and_request_with_custom_properties() {
        // Arrange: MergedConfiguration with custom properties
        let mut merged_config = config_manager::MergedConfiguration::new();
        merged_config.custom_properties = vec![
            config_manager::settings::CustomProperty {
                property_name: "team".to_string(),
                value: config_manager::settings::custom_property::CustomPropertyValue::String(
                    "backend".to_string(),
                ),
            },
            config_manager::settings::CustomProperty {
                property_name: "cost_center".to_string(),
                value: config_manager::settings::custom_property::CustomPropertyValue::String(
                    "engineering".to_string(),
                ),
            },
        ];

        let result = RepositoryCreationResult {
            repository_url: "https://github.com/org/repo".to_string(),
            repository_id: "R_test".to_string(),
            created_at: Timestamp::now(),
            default_branch: "main".to_string(),
        };

        let request = RepositoryCreationRequest {
            name: RepositoryName::new("repo").unwrap(),
            owner: OrganizationName::new("org").unwrap(),
            template: None,
            variables: HashMap::new(),
            visibility: None,
            content_strategy: ContentStrategy::Empty,
        };

        // Act
        let event = RepositoryCreatedEvent::from_result_and_request(
            &result,
            &request,
            &merged_config,
            "user",
        );

        // Assert: custom_properties extracted and converted to HashMap
        assert!(
            event.custom_properties.is_some(),
            "custom_properties should be populated"
        );
        let props = event.custom_properties.unwrap();
        assert_eq!(props.len(), 2);
        assert_eq!(props.get("team"), Some(&"backend".to_string()));
        assert_eq!(props.get("cost_center"), Some(&"engineering".to_string()));
    }

    #[test]
    fn test_event_id_is_unique_uuid_v4() {
        // Arrange
        let result = RepositoryCreationResult {
            repository_url: "https://github.com/org/repo".to_string(),
            repository_id: "R_test".to_string(),
            created_at: Timestamp::now(),
            default_branch: "main".to_string(),
        };

        let request = RepositoryCreationRequest {
            name: RepositoryName::new("repo").unwrap(),
            owner: OrganizationName::new("org").unwrap(),
            template: None,
            variables: HashMap::new(),
            visibility: None,
            content_strategy: ContentStrategy::Empty,
        };

        let merged_config = config_manager::MergedConfiguration::new();

        // Act: Create multiple events
        let event1 = RepositoryCreatedEvent::from_result_and_request(
            &result,
            &request,
            &merged_config,
            "user",
        );

        let event2 = RepositoryCreatedEvent::from_result_and_request(
            &result,
            &request,
            &merged_config,
            "user",
        );

        // Assert: Each event gets unique UUID
        assert_ne!(
            event1.event_id, event2.event_id,
            "event_id should be unique for each event"
        );

        // Verify UUID v4 format
        let uuid1 = uuid::Uuid::parse_str(&event1.event_id).expect("Should be valid UUID");
        let uuid2 = uuid::Uuid::parse_str(&event2.event_id).expect("Should be valid UUID");

        assert_eq!(
            uuid1.get_version(),
            Some(uuid::Version::Random),
            "Should be UUID v4"
        );
        assert_eq!(
            uuid2.get_version(),
            Some(uuid::Version::Random),
            "Should be UUID v4"
        );
    }

    #[test]
    fn test_timestamp_is_current_utc() {
        // Arrange
        let before = Utc::now();

        let result = RepositoryCreationResult {
            repository_url: "https://github.com/org/repo".to_string(),
            repository_id: "R_test".to_string(),
            created_at: Timestamp::now(),
            default_branch: "main".to_string(),
        };

        let request = RepositoryCreationRequest {
            name: RepositoryName::new("repo").unwrap(),
            owner: OrganizationName::new("org").unwrap(),
            template: None,
            variables: HashMap::new(),
            visibility: None,
            content_strategy: ContentStrategy::Empty,
        };

        let merged_config = config_manager::MergedConfiguration::new();

        // Act
        let event = RepositoryCreatedEvent::from_result_and_request(
            &result,
            &request,
            &merged_config,
            "user",
        );

        let after = Utc::now();

        // Assert: Timestamp should be between before and after
        assert!(
            event.timestamp >= before,
            "Timestamp should not be before event creation"
        );
        assert!(
            event.timestamp <= after,
            "Timestamp should not be after event creation"
        );
    }
}

mod endpoint_validation_tests {
    use super::*;

    #[test]
    fn test_endpoint_validation_accepts_valid_https_url() {
        // Arrange: Valid HTTPS endpoint
        let endpoint = NotificationEndpoint {
            url: "https://example.com/webhook".to_string(),
            secret: "my-secret".to_string(),
            events: vec!["repository.created".to_string()],
            active: true,
            timeout_seconds: 5,
            description: None,
        };

        // Act
        let result = endpoint.validate();

        // Assert: Should pass validation
        assert!(result.is_ok(), "Valid endpoint should pass validation");
    }

    #[test]
    fn test_endpoint_validation_rejects_http() {
        // Arrange: HTTP URL (not HTTPS)
        let endpoint = NotificationEndpoint {
            url: "http://example.com/webhook".to_string(),
            secret: "my-secret".to_string(),
            events: vec!["repository.created".to_string()],
            active: true,
            timeout_seconds: 5,
            description: None,
        };

        // Act
        let result = endpoint.validate();

        // Assert: Should reject with InvalidFormat error
        assert!(result.is_err(), "HTTP URL should be rejected");
        match result.unwrap_err() {
            ValidationError::InvalidFormat { field, reason } => {
                assert_eq!(field, "url", "Error should be for 'url' field");
                assert!(
                    reason.contains("HTTPS") || reason.contains("https"),
                    "Error message should mention HTTPS requirement"
                );
            }
            other => panic!("Expected InvalidFormat error, got: {:?}", other),
        }
    }

    #[test]
    fn test_endpoint_validation_rejects_malformed_url() {
        // Arrange: Malformed URL
        let endpoint = NotificationEndpoint {
            url: "not-a-valid-url".to_string(),
            secret: "my-secret".to_string(),
            events: vec!["repository.created".to_string()],
            active: true,
            timeout_seconds: 5,
            description: None,
        };

        // Act
        let result = endpoint.validate();

        // Assert: Should reject with InvalidFormat error
        assert!(result.is_err(), "Malformed URL should be rejected");
        let err = result.unwrap_err();
        assert!(
            matches!(err, ValidationError::InvalidFormat { field, .. } if field == "url"),
            "Error should be InvalidFormat for 'url' field"
        );
    }

    #[test]
    fn test_endpoint_validation_rejects_empty_secret() {
        // Arrange: Empty secret
        let endpoint = NotificationEndpoint {
            url: "https://example.com/webhook".to_string(),
            secret: "".to_string(),
            events: vec!["repository.created".to_string()],
            active: true,
            timeout_seconds: 5,
            description: None,
        };

        // Act
        let result = endpoint.validate();

        // Assert: Should reject with EmptyField error
        assert!(result.is_err(), "Empty secret should be rejected");
        let err = result.unwrap_err();
        assert!(
            matches!(err, ValidationError::EmptyField { field } if field == "secret"),
            "Error should be EmptyField for 'secret' field"
        );
    }

    #[test]
    fn test_endpoint_validation_rejects_empty_events() {
        // Arrange: Empty events array
        let endpoint = NotificationEndpoint {
            url: "https://example.com/webhook".to_string(),
            secret: "my-secret".to_string(),
            events: vec![],
            active: true,
            timeout_seconds: 5,
            description: None,
        };

        // Act
        let result = endpoint.validate();

        // Assert: Should reject with InvalidFormat error
        assert!(result.is_err(), "Empty events array should be rejected");
        let err = result.unwrap_err();
        assert!(
            matches!(err, ValidationError::InvalidFormat { field, .. } if field == "events"),
            "Error should be InvalidFormat for 'events' field"
        );
    }

    #[test]
    fn test_endpoint_validation_rejects_timeout_below_minimum() {
        // Arrange: Timeout = 0 (below minimum of 1)
        let endpoint = NotificationEndpoint {
            url: "https://example.com/webhook".to_string(),
            secret: "my-secret".to_string(),
            events: vec!["repository.created".to_string()],
            active: true,
            timeout_seconds: 0,
            description: None,
        };

        // Act
        let result = endpoint.validate();

        // Assert: Should reject with InvalidFormat error
        assert!(result.is_err(), "Timeout < 1 should be rejected");
        let err = result.unwrap_err();
        assert!(
            matches!(err, ValidationError::InvalidFormat { field, .. } if field == "timeout_seconds"),
            "Error should be InvalidFormat for 'timeout_seconds' field"
        );
    }

    #[test]
    fn test_endpoint_validation_rejects_timeout_above_maximum() {
        // Arrange: Timeout = 31 (above maximum of 30)
        let endpoint = NotificationEndpoint {
            url: "https://example.com/webhook".to_string(),
            secret: "my-secret".to_string(),
            events: vec!["repository.created".to_string()],
            active: true,
            timeout_seconds: 31,
            description: None,
        };

        // Act
        let result = endpoint.validate();

        // Assert: Should reject with InvalidFormat error
        assert!(result.is_err(), "Timeout > 30 should be rejected");
        let err = result.unwrap_err();
        assert!(
            matches!(err, ValidationError::InvalidFormat { field, .. } if field == "timeout_seconds"),
            "Error should be InvalidFormat for 'timeout_seconds' field"
        );
    }

    #[test]
    fn test_endpoint_validation_accepts_minimum_timeout() {
        // Arrange: Timeout = 1 (minimum boundary)
        let endpoint = NotificationEndpoint {
            url: "https://example.com/webhook".to_string(),
            secret: "my-secret".to_string(),
            events: vec!["repository.created".to_string()],
            active: true,
            timeout_seconds: 1,
            description: None,
        };

        // Act
        let result = endpoint.validate();

        // Assert: Should pass validation
        assert!(result.is_ok(), "Minimum timeout (1s) should be valid");
    }

    #[test]
    fn test_endpoint_validation_accepts_maximum_timeout() {
        // Arrange: Timeout = 30 (maximum boundary)
        let endpoint = NotificationEndpoint {
            url: "https://example.com/webhook".to_string(),
            secret: "my-secret".to_string(),
            events: vec!["repository.created".to_string()],
            active: true,
            timeout_seconds: 30,
            description: None,
        };

        // Act
        let result = endpoint.validate();

        // Assert: Should pass validation
        assert!(result.is_ok(), "Maximum timeout (30s) should be valid");
    }

    #[test]
    fn test_accepts_event_filters_by_event_type() {
        // Arrange: Endpoint configured for specific events
        let endpoint = NotificationEndpoint {
            url: "https://example.com/webhook".to_string(),
            secret: "my-secret".to_string(),
            events: vec![
                "repository.created".to_string(),
                "repository.updated".to_string(),
            ],
            active: true,
            timeout_seconds: 5,
            description: None,
        };

        // Act & Assert: Should accept configured events
        assert!(
            endpoint.accepts_event("repository.created"),
            "Should accept repository.created"
        );
        assert!(
            endpoint.accepts_event("repository.updated"),
            "Should accept repository.updated"
        );

        // Should reject non-configured events
        assert!(
            !endpoint.accepts_event("repository.deleted"),
            "Should reject repository.deleted"
        );
        assert!(
            !endpoint.accepts_event("other.event"),
            "Should reject other.event"
        );
    }

    #[test]
    fn test_accepts_event_respects_active_status() {
        // Arrange: Inactive endpoint
        let inactive_endpoint = NotificationEndpoint {
            url: "https://example.com/webhook".to_string(),
            secret: "my-secret".to_string(),
            events: vec!["repository.created".to_string()],
            active: false,
            timeout_seconds: 5,
            description: None,
        };

        // Act & Assert: Inactive endpoint should not accept any events
        assert!(
            !inactive_endpoint.accepts_event("repository.created"),
            "Inactive endpoint should not accept events"
        );

        // Arrange: Active endpoint
        let active_endpoint = NotificationEndpoint {
            url: "https://example.com/webhook".to_string(),
            secret: "my-secret".to_string(),
            events: vec!["repository.created".to_string()],
            active: true,
            timeout_seconds: 5,
            description: None,
        };

        // Act & Assert: Active endpoint should accept configured events
        assert!(
            active_endpoint.accepts_event("repository.created"),
            "Active endpoint should accept configured events"
        );
    }

    #[test]
    fn test_accepts_event_case_sensitive() {
        // Arrange: Endpoint with specific event type
        let endpoint = NotificationEndpoint {
            url: "https://example.com/webhook".to_string(),
            secret: "my-secret".to_string(),
            events: vec!["repository.created".to_string()],
            active: true,
            timeout_seconds: 5,
            description: None,
        };

        // Act & Assert: Event type matching should be case-sensitive
        assert!(
            endpoint.accepts_event("repository.created"),
            "Exact match should be accepted"
        );
        assert!(
            !endpoint.accepts_event("Repository.Created"),
            "Case mismatch should be rejected"
        );
        assert!(
            !endpoint.accepts_event("REPOSITORY.CREATED"),
            "Uppercase mismatch should be rejected"
        );
    }
}

mod signature_tests {
    #[test]
    fn test_signature_computation() {
        // TODO: Implement per docs/spec/interfaces/event-publisher.md
        // - Produces correct HMAC-SHA256 signature
        // - Format matches "sha256=<hex>" pattern
        // - Signature length is 71 characters
        // - Same input produces same output
        unimplemented!()
    }
}

mod endpoint_collection_tests {
    #[test]
    fn test_endpoint_deduplication() {
        // TODO: Implement per docs/spec/interfaces/event-publisher.md
        // - Accumulates endpoints from all levels
        // - Deduplicates by URL + event type
        // - Preserves order (org, team, template)
        // - Handles missing team/template configs
        unimplemented!()
    }
}
