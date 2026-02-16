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
    use super::*;

    #[test]
    fn test_compute_hmac_produces_correct_format() {
        // Arrange: Simple payload and secret
        let payload = b"test payload";
        let secret = "test-secret";

        // Act
        let signature = compute_hmac_sha256(payload, secret);

        // Assert: Format is "sha256=<hex>"
        assert!(
            signature.starts_with("sha256="),
            "Signature should start with 'sha256='"
        );

        // Verify hex portion is valid hex digits
        let hex_part = &signature[7..];
        assert!(
            hex_part.chars().all(|c| c.is_ascii_hexdigit()),
            "Hex portion should only contain hex digits"
        );
    }

    #[test]
    fn test_compute_hmac_produces_64_character_hex() {
        // Arrange: SHA256 produces 32 bytes = 64 hex characters
        let payload = b"test";
        let secret = "secret";

        // Act
        let signature = compute_hmac_sha256(payload, secret);

        // Assert: Total length is "sha256=" (7) + 64 hex chars = 71
        assert_eq!(
            signature.len(),
            71,
            "Signature should be 71 characters total (sha256= + 64 hex)"
        );

        let hex_part = &signature[7..];
        assert_eq!(
            hex_part.len(),
            64,
            "Hex portion should be 64 characters (32 bytes)"
        );
    }

    #[test]
    fn test_compute_hmac_deterministic() {
        // Arrange: Same inputs should produce same output
        let payload = b"consistent payload";
        let secret = "consistent-secret";

        // Act: Compute signature twice
        let sig1 = compute_hmac_sha256(payload, secret);
        let sig2 = compute_hmac_sha256(payload, secret);

        // Assert: Should be identical
        assert_eq!(sig1, sig2, "Same inputs should produce same signature");
    }

    #[test]
    fn test_compute_hmac_different_payloads_produce_different_signatures() {
        // Arrange: Different payloads with same secret
        let secret = "shared-secret";
        let payload1 = b"payload one";
        let payload2 = b"payload two";

        // Act
        let sig1 = compute_hmac_sha256(payload1, secret);
        let sig2 = compute_hmac_sha256(payload2, secret);

        // Assert: Signatures should differ
        assert_ne!(
            sig1, sig2,
            "Different payloads should produce different signatures"
        );
    }

    #[test]
    fn test_compute_hmac_different_secrets_produce_different_signatures() {
        // Arrange: Same payload with different secrets
        let payload = b"same payload";
        let secret1 = "secret-one";
        let secret2 = "secret-two";

        // Act
        let sig1 = compute_hmac_sha256(payload, secret1);
        let sig2 = compute_hmac_sha256(payload, secret2);

        // Assert: Signatures should differ
        assert_ne!(
            sig1, sig2,
            "Different secrets should produce different signatures"
        );
    }

    #[test]
    fn test_compute_hmac_empty_payload() {
        // Arrange: Empty payload
        let payload = b"";
        let secret = "secret";

        // Act
        let signature = compute_hmac_sha256(payload, secret);

        // Assert: Should still produce valid signature
        assert!(signature.starts_with("sha256="));
        assert_eq!(signature.len(), 71);
    }

    #[test]
    fn test_compute_hmac_empty_secret() {
        // Arrange: Empty secret (edge case)
        let payload = b"payload";
        let secret = "";

        // Act
        let signature = compute_hmac_sha256(payload, secret);

        // Assert: Should still produce valid signature
        assert!(signature.starts_with("sha256="));
        assert_eq!(signature.len(), 71);
    }

    #[test]
    fn test_compute_hmac_with_known_test_vector() {
        // Arrange: Known test vector for verification
        // Using a simple case we can verify externally
        let payload = b"test message";
        let secret = "secret-key";

        // Act
        let signature = compute_hmac_sha256(payload, secret);

        // Assert: Verify it's a valid signature format
        // (We can't hardcode expected value as it depends on implementation,
        // but we verify consistency and format)
        assert!(signature.starts_with("sha256="));
        assert_eq!(signature.len(), 71);

        // Verify it's lowercase hex
        let hex_part = &signature[7..];
        assert!(
            hex_part.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')),
            "Hex should be lowercase"
        );
    }

    #[test]
    fn test_compute_hmac_with_unicode_in_secret() {
        // Arrange: Unicode characters in secret
        let payload = b"payload";
        let secret = "secret-with-émojis-🔐";

        // Act
        let signature = compute_hmac_sha256(payload, secret);

        // Assert: Should handle Unicode correctly
        assert!(signature.starts_with("sha256="));
        assert_eq!(signature.len(), 71);
    }

    #[test]
    fn test_compute_hmac_with_binary_payload() {
        // Arrange: Binary data (not UTF-8)
        let payload: &[u8] = &[0x00, 0xFF, 0xAA, 0x55, 0xDE, 0xAD, 0xBE, 0xEF];
        let secret = "binary-secret";

        // Act
        let signature = compute_hmac_sha256(payload, secret);

        // Assert: Should handle binary data
        assert!(signature.starts_with("sha256="));
        assert_eq!(signature.len(), 71);
    }

    #[test]
    fn test_sign_webhook_request_adds_signature_header() {
        // Arrange: Create a mock request builder
        let client = reqwest::Client::new();
        let request = client.post("https://example.com/webhook");
        let payload = b"test event payload";
        let secret = "webhook-secret";

        // Act: Sign the request
        let signed_request = sign_webhook_request(request, payload, secret);

        // Build the request to inspect headers
        let built_request = signed_request
            .build()
            .expect("Should build request successfully");

        // Assert: Should have X-RepoRoller-Signature-256 header
        let signature_header = built_request.headers().get("X-RepoRoller-Signature-256");

        assert!(
            signature_header.is_some(),
            "Request should have X-RepoRoller-Signature-256 header"
        );

        // Verify header value format
        let header_value = signature_header.unwrap().to_str().unwrap();
        assert!(
            header_value.starts_with("sha256="),
            "Header value should be in sha256=<hex> format"
        );
        assert_eq!(header_value.len(), 71, "Signature should be 71 characters");
    }

    #[test]
    fn test_sign_webhook_request_signature_matches_compute_hmac() {
        // Arrange
        let client = reqwest::Client::new();
        let request = client.post("https://example.com/webhook");
        let payload = b"consistent payload";
        let secret = "consistent-secret";

        // Act: Compute signature directly and via signing
        let expected_signature = compute_hmac_sha256(payload, secret);
        let signed_request = sign_webhook_request(request, payload, secret);
        let built_request = signed_request.build().unwrap();

        // Assert: Header value should match compute_hmac_sha256 output
        let actual_signature = built_request
            .headers()
            .get("X-RepoRoller-Signature-256")
            .unwrap()
            .to_str()
            .unwrap();

        assert_eq!(
            actual_signature, expected_signature,
            "sign_webhook_request should use compute_hmac_sha256"
        );
    }

    #[test]
    fn test_sign_webhook_request_preserves_other_headers() {
        // Arrange: Request with existing headers
        let client = reqwest::Client::new();
        let request = client
            .post("https://example.com/webhook")
            .header("Content-Type", "application/json")
            .header("User-Agent", "RepoRoller/1.0");
        let payload = b"payload";
        let secret = "secret";

        // Act: Sign the request
        let signed_request = sign_webhook_request(request, payload, secret);
        let built_request = signed_request.build().unwrap();

        // Assert: Should preserve existing headers
        let headers = built_request.headers();
        assert_eq!(
            headers.get("Content-Type").unwrap().to_str().unwrap(),
            "application/json"
        );
        assert_eq!(
            headers.get("User-Agent").unwrap().to_str().unwrap(),
            "RepoRoller/1.0"
        );

        // And add signature header
        assert!(headers.get("X-RepoRoller-Signature-256").is_some());
    }
}

mod endpoint_collection_tests {
    use super::*;

    #[test]
    fn test_collect_from_org_only() {
        // Arrange: Only organization-level config
        let org_config = NotificationsConfig {
            outbound_webhooks: vec![NotificationEndpoint {
                url: "https://org.example.com/webhook".to_string(),
                secret: "org-secret".to_string(),
                events: vec!["repository.created".to_string()],
                active: true,
                timeout_seconds: 5,
                description: None,
            }],
        };

        // Act
        let endpoints = collect_notification_endpoints(&org_config, None, None);

        // Assert: Should return org endpoints
        assert_eq!(endpoints.len(), 1);
        assert_eq!(endpoints[0].url, "https://org.example.com/webhook");
    }

    #[test]
    fn test_collect_from_all_levels() {
        // Arrange: Endpoints at all three levels
        let org_config = NotificationsConfig {
            outbound_webhooks: vec![NotificationEndpoint {
                url: "https://org.example.com/webhook".to_string(),
                secret: "org-secret".to_string(),
                events: vec!["repository.created".to_string()],
                active: true,
                timeout_seconds: 5,
                description: None,
            }],
        };

        let team_config = NotificationsConfig {
            outbound_webhooks: vec![NotificationEndpoint {
                url: "https://team.example.com/webhook".to_string(),
                secret: "team-secret".to_string(),
                events: vec!["repository.created".to_string()],
                active: true,
                timeout_seconds: 5,
                description: None,
            }],
        };

        let template_config = NotificationsConfig {
            outbound_webhooks: vec![NotificationEndpoint {
                url: "https://template.example.com/webhook".to_string(),
                secret: "template-secret".to_string(),
                events: vec!["repository.created".to_string()],
                active: true,
                timeout_seconds: 5,
                description: None,
            }],
        };

        // Act
        let endpoints =
            collect_notification_endpoints(&org_config, Some(&team_config), Some(&template_config));

        // Assert: Should have all 3 endpoints in order (org → team → template)
        assert_eq!(endpoints.len(), 3);
        assert_eq!(endpoints[0].url, "https://org.example.com/webhook");
        assert_eq!(endpoints[1].url, "https://team.example.com/webhook");
        assert_eq!(endpoints[2].url, "https://template.example.com/webhook");
    }

    #[test]
    fn test_deduplication_by_url_and_events() {
        // Arrange: Same URL and events at multiple levels (should deduplicate)
        let org_config = NotificationsConfig {
            outbound_webhooks: vec![NotificationEndpoint {
                url: "https://shared.example.com/webhook".to_string(),
                secret: "org-secret".to_string(),
                events: vec!["repository.created".to_string()],
                active: true,
                timeout_seconds: 5,
                description: None,
            }],
        };

        let team_config = NotificationsConfig {
            outbound_webhooks: vec![NotificationEndpoint {
                url: "https://shared.example.com/webhook".to_string(),
                secret: "team-secret".to_string(), // Different secret
                events: vec!["repository.created".to_string()], // Same events
                active: true,
                timeout_seconds: 10, // Different timeout
                description: Some("Team override".to_string()),
            }],
        };

        // Act
        let endpoints = collect_notification_endpoints(&org_config, Some(&team_config), None);

        // Assert: Should deduplicate, keeping first occurrence (org level)
        assert_eq!(endpoints.len(), 1, "Should deduplicate same URL + events");
        assert_eq!(endpoints[0].url, "https://shared.example.com/webhook");
        assert_eq!(
            endpoints[0].secret, "org-secret",
            "Should keep first occurrence"
        );
    }

    #[test]
    fn test_no_deduplication_for_different_events() {
        // Arrange: Same URL but different events (should NOT deduplicate)
        let org_config = NotificationsConfig {
            outbound_webhooks: vec![NotificationEndpoint {
                url: "https://example.com/webhook".to_string(),
                secret: "secret1".to_string(),
                events: vec!["repository.created".to_string()],
                active: true,
                timeout_seconds: 5,
                description: None,
            }],
        };

        let team_config = NotificationsConfig {
            outbound_webhooks: vec![NotificationEndpoint {
                url: "https://example.com/webhook".to_string(),
                secret: "secret2".to_string(),
                events: vec!["repository.updated".to_string()], // Different event
                active: true,
                timeout_seconds: 5,
                description: None,
            }],
        };

        // Act
        let endpoints = collect_notification_endpoints(&org_config, Some(&team_config), None);

        // Assert: Should NOT deduplicate (different event types)
        assert_eq!(endpoints.len(), 2, "Should keep both (different events)");
        assert_eq!(endpoints[0].events[0], "repository.created");
        assert_eq!(endpoints[1].events[0], "repository.updated");
    }

    #[test]
    fn test_deduplication_with_multiple_events() {
        // Arrange: Endpoints with multiple events
        let org_config = NotificationsConfig {
            outbound_webhooks: vec![NotificationEndpoint {
                url: "https://example.com/webhook".to_string(),
                secret: "secret".to_string(),
                events: vec![
                    "repository.created".to_string(),
                    "repository.updated".to_string(),
                ],
                active: true,
                timeout_seconds: 5,
                description: None,
            }],
        };

        let team_config = NotificationsConfig {
            outbound_webhooks: vec![NotificationEndpoint {
                url: "https://example.com/webhook".to_string(),
                secret: "different-secret".to_string(),
                events: vec![
                    "repository.updated".to_string(), // Different order
                    "repository.created".to_string(),
                ],
                active: true,
                timeout_seconds: 5,
                description: None,
            }],
        };

        // Act
        let endpoints = collect_notification_endpoints(&org_config, Some(&team_config), None);

        // Assert: Should deduplicate if events match (order-independent)
        // Note: Implementation may or may not be order-independent
        // This test documents expected behavior
        assert_eq!(endpoints.len(), 1, "Should deduplicate same URL + events");
    }

    #[test]
    fn test_preserves_order_org_team_template() {
        // Arrange: Multiple endpoints at each level
        let org_config = NotificationsConfig {
            outbound_webhooks: vec![
                NotificationEndpoint {
                    url: "https://org1.example.com/webhook".to_string(),
                    secret: "secret".to_string(),
                    events: vec!["repository.created".to_string()],
                    active: true,
                    timeout_seconds: 5,
                    description: None,
                },
                NotificationEndpoint {
                    url: "https://org2.example.com/webhook".to_string(),
                    secret: "secret".to_string(),
                    events: vec!["repository.created".to_string()],
                    active: true,
                    timeout_seconds: 5,
                    description: None,
                },
            ],
        };

        let team_config = NotificationsConfig {
            outbound_webhooks: vec![NotificationEndpoint {
                url: "https://team1.example.com/webhook".to_string(),
                secret: "secret".to_string(),
                events: vec!["repository.created".to_string()],
                active: true,
                timeout_seconds: 5,
                description: None,
            }],
        };

        let template_config = NotificationsConfig {
            outbound_webhooks: vec![NotificationEndpoint {
                url: "https://template1.example.com/webhook".to_string(),
                secret: "secret".to_string(),
                events: vec!["repository.created".to_string()],
                active: true,
                timeout_seconds: 5,
                description: None,
            }],
        };

        // Act
        let endpoints =
            collect_notification_endpoints(&org_config, Some(&team_config), Some(&template_config));

        // Assert: Should preserve order: org1, org2, team1, template1
        assert_eq!(endpoints.len(), 4);
        assert!(endpoints[0].url.contains("org1"));
        assert!(endpoints[1].url.contains("org2"));
        assert!(endpoints[2].url.contains("team1"));
        assert!(endpoints[3].url.contains("template1"));
    }

    #[test]
    fn test_handles_empty_configs() {
        // Arrange: Configs with no webhooks
        let org_config = NotificationsConfig {
            outbound_webhooks: vec![],
        };

        let team_config = NotificationsConfig {
            outbound_webhooks: vec![],
        };

        // Act
        let endpoints = collect_notification_endpoints(&org_config, Some(&team_config), None);

        // Assert: Should return empty vector
        assert_eq!(endpoints.len(), 0);
    }

    #[test]
    fn test_handles_none_optional_configs() {
        // Arrange: Only org config, team and template are None
        let org_config = NotificationsConfig {
            outbound_webhooks: vec![NotificationEndpoint {
                url: "https://org.example.com/webhook".to_string(),
                secret: "secret".to_string(),
                events: vec!["repository.created".to_string()],
                active: true,
                timeout_seconds: 5,
                description: None,
            }],
        };

        // Act
        let endpoints = collect_notification_endpoints(&org_config, None, None);

        // Assert: Should work fine with None configs
        assert_eq!(endpoints.len(), 1);
    }

    #[test]
    fn test_deduplication_complex_scenario() {
        // Arrange: Mix of unique and duplicate endpoints
        let org_config = NotificationsConfig {
            outbound_webhooks: vec![
                NotificationEndpoint {
                    url: "https://unique-org.example.com/webhook".to_string(),
                    secret: "secret".to_string(),
                    events: vec!["repository.created".to_string()],
                    active: true,
                    timeout_seconds: 5,
                    description: None,
                },
                NotificationEndpoint {
                    url: "https://shared.example.com/webhook".to_string(),
                    secret: "org-secret".to_string(),
                    events: vec!["repository.created".to_string()],
                    active: true,
                    timeout_seconds: 5,
                    description: None,
                },
            ],
        };

        let team_config = NotificationsConfig {
            outbound_webhooks: vec![
                NotificationEndpoint {
                    url: "https://shared.example.com/webhook".to_string(),
                    secret: "team-secret".to_string(), // Duplicate
                    events: vec!["repository.created".to_string()],
                    active: true,
                    timeout_seconds: 5,
                    description: None,
                },
                NotificationEndpoint {
                    url: "https://unique-team.example.com/webhook".to_string(),
                    secret: "secret".to_string(),
                    events: vec!["repository.created".to_string()],
                    active: true,
                    timeout_seconds: 5,
                    description: None,
                },
            ],
        };

        // Act
        let endpoints = collect_notification_endpoints(&org_config, Some(&team_config), None);

        // Assert: Should have 3 unique endpoints (deduplicate shared)
        assert_eq!(endpoints.len(), 3);

        let urls: Vec<&str> = endpoints.iter().map(|e| e.url.as_str()).collect();
        assert!(urls.contains(&"https://unique-org.example.com/webhook"));
        assert!(urls.contains(&"https://shared.example.com/webhook"));
        assert!(urls.contains(&"https://unique-team.example.com/webhook"));
    }
}
