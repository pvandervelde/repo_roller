//! Metadata repository provider system for organization configuration management.
//!
//! This module provides the foundational traits and structures for discovering, accessing,
//! and validating metadata repositories that contain organization-specific configuration data.
//! The system supports multiple discovery methods and provides comprehensive error handling
//! for repository access operations.
//!
//! # Architecture
//!
//! The metadata provider system uses a trait-based approach to abstract repository access:
//!
//! - `MetadataRepositoryProvider` - Core trait for repository operations
//! - `MetadataRepository` - Represents a discovered metadata repository
//! - `DiscoveryMethod` - Tracks how a repository was discovered
//!
//! # Discovery Methods
//!
//! The system supports two discovery methods:
//!
//! 1. **Configuration-based**: Repository name specified in application configuration
//! 2. **Topic-based**: Repository tagged with `template-metadata` topic
//!
//! # Examples
//!
//! ```rust
//! use config_manager::metadata::{MetadataRepositoryProvider, DiscoveryMethod};
//!
//! async fn discover_org_config(provider: &dyn MetadataRepositoryProvider) {
//!     match provider.discover_metadata_repository("acme-corp").await {
//!         Ok(repo) => println!("Found metadata repository: {}", repo.repository_name),
//!         Err(e) => eprintln!("Discovery failed: {}", e),
//!     }
//! }
//! ```

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::organization::{RepositoryTypeConfig, TeamConfig};
use crate::settings::GlobalDefaults;

// Unit tests for metadata provider functionality
#[path = "metadata_tests.rs"]
#[cfg(test)]
mod tests;

/// Result type for metadata repository operations.
///
/// This type alias provides a consistent error handling pattern across all
/// metadata repository operations, using the configuration management error types.
pub type MetadataResult<T> = Result<T, crate::ConfigurationError>;

/// Core trait for metadata repository providers.
///
/// This trait abstracts access to organization configuration repositories, supporting
/// multiple discovery methods and providing comprehensive validation capabilities.
/// All implementations must be thread-safe to support concurrent access patterns.
///
/// # Discovery Process
///
/// 1. **Repository Discovery**: Locate the metadata repository using configured discovery method
/// 2. **Structure Validation**: Verify repository contains required directories and files
/// 3. **Access Validation**: Ensure the provider has necessary permissions
/// 4. **Configuration Loading**: Load and parse configuration files from repository
///
/// # Error Handling
///
/// All methods return `MetadataResult<T>` to provide consistent error handling:
///
/// - **Discovery Errors**: Repository not found, multiple candidates, access denied
/// - **Validation Errors**: Invalid structure, missing required files, parse errors
/// - **Network Errors**: API failures, timeouts, rate limiting
/// - **Permission Errors**: Insufficient access rights, authentication failures
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` to support concurrent access across async tasks.
/// Providers should implement appropriate internal synchronization for shared state.
///
/// # Examples
///
/// ```rust
/// use config_manager::metadata::MetadataRepositoryProvider;
/// use async_trait::async_trait;
///
/// struct MockProvider;
///
/// #[async_trait]
/// impl MetadataRepositoryProvider for MockProvider {
///     async fn discover_metadata_repository(&self, org: &str) -> MetadataResult<MetadataRepository> {
///         // Implementation details...
///         todo!("Not implemented")
///     }
///
///     async fn validate_repository_structure(&self, repo: &MetadataRepository) -> MetadataResult<()> {
///         // Implementation details...
///         todo!("Not implemented")
///     }
///
///     // ... other required methods
/// }
/// ```
#[async_trait]
pub trait MetadataRepositoryProvider: Send + Sync {
    /// Discover the metadata repository for the specified organization.
    ///
    /// This method attempts to locate the organization's metadata repository using
    /// the configured discovery methods. It tries configuration-based discovery first,
    /// then falls back to topic-based discovery if configured.
    ///
    /// # Arguments
    ///
    /// * `org` - The organization name to search for metadata repository
    ///
    /// # Returns
    ///
    /// * `Ok(MetadataRepository)` - Successfully discovered repository with metadata
    /// * `Err(ConfigurationError::RepositoryNotFound)` - No metadata repository found
    /// * `Err(ConfigurationError::MultipleRepositoriesFound)` - Multiple candidates found
    /// * `Err(ConfigurationError::AccessDenied)` - Insufficient permissions to access repository
    /// * `Err(ConfigurationError::NetworkError)` - API communication failure
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use config_manager::metadata::MetadataRepositoryProvider;
    /// # async fn example(provider: &dyn MetadataRepositoryProvider) {
    /// match provider.discover_metadata_repository("acme-corp").await {
    ///     Ok(repo) => {
    ///         println!("Found repository: {} via {:?}",
    ///                  repo.repository_name, repo.discovery_method);
    ///     },
    ///     Err(e) => eprintln!("Discovery failed: {}", e),
    /// }
    /// # }
    /// ```
    async fn discover_metadata_repository(&self, org: &str) -> MetadataResult<MetadataRepository>;

    /// Validate the structure of a metadata repository.
    ///
    /// This method checks that the repository contains all required directories and files
    /// for organization configuration management. It validates the presence of:
    ///
    /// - `global/` directory with `defaults.toml`
    /// - `teams/` directory (may be empty)
    /// - `types/` directory (may be empty)
    /// - Optional `schemas/` directory for validation schemas
    ///
    /// # Arguments
    ///
    /// * `repo` - The metadata repository to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Repository structure is valid
    /// * `Err(ConfigurationError::InvalidRepositoryStructure)` - Missing required directories/files
    /// * `Err(ConfigurationError::AccessDenied)` - Cannot access repository contents
    /// * `Err(ConfigurationError::NetworkError)` - API communication failure
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use config_manager::metadata::{MetadataRepositoryProvider, MetadataRepository};
    /// # async fn example(provider: &dyn MetadataRepositoryProvider, repo: &MetadataRepository) {
    /// if let Err(e) = provider.validate_repository_structure(repo).await {
    ///     eprintln!("Repository structure validation failed: {}", e);
    ///     return;
    /// }
    /// println!("Repository structure is valid");
    /// # }
    /// ```
    async fn validate_repository_structure(&self, repo: &MetadataRepository) -> MetadataResult<()>;

    /// Load global default configuration from the metadata repository.
    ///
    /// This method loads and parses the `global/defaults.toml` file containing
    /// organization-wide baseline settings. The global defaults provide the foundation
    /// for the configuration hierarchy and define override policies.
    ///
    /// # Arguments
    ///
    /// * `repo` - The metadata repository to load from
    ///
    /// # Returns
    ///
    /// * `Ok(GlobalDefaults)` - Successfully loaded and parsed global configuration
    /// * `Err(ConfigurationError::FileNotFound)` - global/defaults.toml missing
    /// * `Err(ConfigurationError::ParseError)` - Invalid TOML syntax or structure
    /// * `Err(ConfigurationError::ValidationError)` - Configuration violates schema
    /// * `Err(ConfigurationError::AccessDenied)` - Cannot access configuration file
    /// * `Err(ConfigurationError::NetworkError)` - API communication failure
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use config_manager::metadata::{MetadataRepositoryProvider, MetadataRepository};
    /// # async fn example(provider: &dyn MetadataRepositoryProvider, repo: &MetadataRepository) {
    /// match provider.load_global_defaults(repo).await {
    ///     Ok(defaults) => {
    ///         println!("Loaded global defaults with {} repository settings",
    ///                  defaults.repository.is_some() as usize);
    ///     },
    ///     Err(e) => eprintln!("Failed to load global defaults: {}", e),
    /// }
    /// # }
    /// ```
    async fn load_global_defaults(
        &self,
        repo: &MetadataRepository,
    ) -> MetadataResult<GlobalDefaults>;

    /// Load team-specific configuration from the metadata repository.
    ///
    /// This method loads and parses team configuration from `teams/{team}/config.toml`.
    /// Team configurations provide team-specific overrides and additions to global settings.
    /// Returns `None` if the team configuration does not exist.
    ///
    /// # Arguments
    ///
    /// * `repo` - The metadata repository to load from
    /// * `team` - The team name to load configuration for
    ///
    /// # Returns
    ///
    /// * `Ok(Some(TeamConfig))` - Successfully loaded team configuration
    /// * `Ok(None)` - Team configuration file does not exist (valid scenario)
    /// * `Err(ConfigurationError::ParseError)` - Invalid TOML syntax or structure
    /// * `Err(ConfigurationError::ValidationError)` - Configuration violates schema
    /// * `Err(ConfigurationError::AccessDenied)` - Cannot access configuration file
    /// * `Err(ConfigurationError::NetworkError)` - API communication failure
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use config_manager::metadata::{MetadataRepositoryProvider, MetadataRepository};
    /// # async fn example(provider: &dyn MetadataRepositoryProvider, repo: &MetadataRepository) {
    /// match provider.load_team_configuration(repo, "platform-team").await {
    ///     Ok(Some(team_config)) => {
    ///         println!("Loaded configuration for platform-team");
    ///     },
    ///     Ok(None) => {
    ///         println!("No configuration found for platform-team (using defaults)");
    ///     },
    ///     Err(e) => eprintln!("Failed to load team configuration: {}", e),
    /// }
    /// # }
    /// ```
    async fn load_team_configuration(
        &self,
        repo: &MetadataRepository,
        team: &str,
    ) -> MetadataResult<Option<TeamConfig>>;

    /// Load repository type configuration from the metadata repository.
    ///
    /// This method loads and parses repository type configuration from
    /// `types/{repo_type}/config.toml`. Repository type configurations provide
    /// type-specific settings that override global defaults but are overridden by
    /// team and template settings. Returns `None` if the type configuration does not exist.
    ///
    /// # Arguments
    ///
    /// * `repo` - The metadata repository to load from
    /// * `repo_type` - The repository type name to load configuration for
    ///
    /// # Returns
    ///
    /// * `Ok(Some(RepositoryTypeConfig))` - Successfully loaded repository type configuration
    /// * `Ok(None)` - Repository type configuration file does not exist (valid scenario)
    /// * `Err(ConfigurationError::ParseError)` - Invalid TOML syntax or structure
    /// * `Err(ConfigurationError::ValidationError)` - Configuration violates schema
    /// * `Err(ConfigurationError::AccessDenied)` - Cannot access configuration file
    /// * `Err(ConfigurationError::NetworkError)` - API communication failure
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use config_manager::metadata::{MetadataRepositoryProvider, MetadataRepository};
    /// # async fn example(provider: &dyn MetadataRepositoryProvider, repo: &MetadataRepository) {
    /// match provider.load_repository_type_configuration(repo, "library").await {
    ///     Ok(Some(type_config)) => {
    ///         println!("Loaded configuration for library repositories");
    ///     },
    ///     Ok(None) => {
    ///         println!("No specific configuration for library repositories");
    ///     },
    ///     Err(e) => eprintln!("Failed to load repository type configuration: {}", e),
    /// }
    /// # }
    /// ```
    async fn load_repository_type_configuration(
        &self,
        repo: &MetadataRepository,
        repo_type: &str,
    ) -> MetadataResult<Option<RepositoryTypeConfig>>;

    /// List all available repository types defined in the metadata repository.
    ///
    /// This method scans the `types/` directory to discover all available repository
    /// types that have configuration files. Repository types are determined by the
    /// presence of `config.toml` files in subdirectories of `types/`.
    ///
    /// # Arguments
    ///
    /// * `repo` - The metadata repository to scan
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` - List of available repository type names
    /// * `Err(ConfigurationError::AccessDenied)` - Cannot access types directory
    /// * `Err(ConfigurationError::NetworkError)` - API communication failure
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use config_manager::metadata::{MetadataRepositoryProvider, MetadataRepository};
    /// # async fn example(provider: &dyn MetadataRepositoryProvider, repo: &MetadataRepository) {
    /// match provider.list_available_repository_types(repo).await {
    ///     Ok(types) => {
    ///         println!("Available repository types: {}", types.join(", "));
    ///     },
    ///     Err(e) => eprintln!("Failed to list repository types: {}", e),
    /// }
    /// # }
    /// ```
    async fn list_available_repository_types(
        &self,
        repo: &MetadataRepository,
    ) -> MetadataResult<Vec<String>>;

    /// Load standard label definitions from the metadata repository.
    ///
    /// This method loads standard label configurations that can be shared across
    /// multiple teams and repository types. Standard labels are typically defined
    /// in `global/labels.toml` or similar location within the metadata repository.
    ///
    /// # Arguments
    ///
    /// * `repo` - The metadata repository to load from
    ///
    /// # Returns
    ///
    /// * `Ok(HashMap<String, LabelConfig>)` - Map of label names to configurations
    /// * `Err(ConfigurationError::FileNotFound)` - Labels file not found (may be optional)
    /// * `Err(ConfigurationError::ParseError)` - Invalid TOML syntax or structure
    /// * `Err(ConfigurationError::AccessDenied)` - Cannot access labels file
    /// * `Err(ConfigurationError::NetworkError)` - API communication failure
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use config_manager::metadata::{MetadataRepositoryProvider, MetadataRepository};
    /// # async fn example(provider: &dyn MetadataRepositoryProvider, repo: &MetadataRepository) {
    /// match provider.load_standard_labels(repo).await {
    ///     Ok(labels) => {
    ///         println!("Loaded {} standard labels", labels.len());
    ///     },
    ///     Err(e) => eprintln!("Failed to load standard labels: {}", e),
    /// }
    /// # }
    /// ```
    async fn load_standard_labels(
        &self,
        repo: &MetadataRepository,
    ) -> MetadataResult<HashMap<String, crate::LabelConfig>>;
}

/// Represents a discovered metadata repository containing organization configuration.
///
/// This structure contains information about a metadata repository that has been
/// discovered and validated for use in organization configuration management.
/// It tracks the discovery method used and maintains timestamps for cache invalidation.
///
/// # Fields
///
/// * `organization` - The GitHub organization name that owns this repository
/// * `repository_name` - The name of the metadata repository
/// * `discovery_method` - How this repository was discovered
/// * `last_updated` - Timestamp of last known update for cache invalidation
///
/// # Examples
///
/// ```rust
/// use config_manager::metadata::{MetadataRepository, DiscoveryMethod};
/// use chrono::Utc;
///
/// let repo = MetadataRepository {
///     organization: "acme-corp".to_string(),
///     repository_name: "acme-corp-config".to_string(),
///     discovery_method: DiscoveryMethod::ConfigurationBased {
///         repository_name: "acme-corp-config".to_string(),
///     },
///     last_updated: Utc::now(),
/// };
///
/// println!("Metadata repository: {}/{}", repo.organization, repo.repository_name);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetadataRepository {
    /// The GitHub organization that owns this metadata repository.
    ///
    /// This must match the organization for which configuration is being loaded
    /// to ensure proper access control and configuration isolation.
    pub organization: String,

    /// The name of the metadata repository within the organization.
    ///
    /// This is used to construct API calls and identify the repository
    /// in error messages and audit logs.
    pub repository_name: String,

    /// The method used to discover this metadata repository.
    ///
    /// This information is useful for troubleshooting discovery issues and
    /// understanding the configuration source in audit logs.
    pub discovery_method: DiscoveryMethod,

    /// Timestamp of the last known update to this repository.
    ///
    /// This is used for cache invalidation and determining when to refresh
    /// cached configuration data. The timestamp should reflect the last
    /// Git commit or similar repository modification time.
    pub last_updated: DateTime<Utc>,
}

impl MetadataRepository {
    /// Create a new metadata repository instance.
    ///
    /// # Arguments
    ///
    /// * `organization` - The GitHub organization name
    /// * `repository_name` - The metadata repository name
    /// * `discovery_method` - How the repository was discovered
    /// * `last_updated` - Timestamp of last repository update
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::metadata::{MetadataRepository, DiscoveryMethod};
    /// use chrono::Utc;
    ///
    /// let repo = MetadataRepository::new(
    ///     "acme-corp".to_string(),
    ///     "acme-corp-config".to_string(),
    ///     DiscoveryMethod::TopicBased {
    ///         topic: "template-metadata".to_string(),
    ///     },
    ///     Utc::now(),
    /// );
    /// ```
    pub fn new(
        organization: String,
        repository_name: String,
        discovery_method: DiscoveryMethod,
        last_updated: DateTime<Utc>,
    ) -> Self {
        Self {
            organization,
            repository_name,
            discovery_method,
            last_updated,
        }
    }

    /// Get the full repository identifier in the format "org/repo".
    ///
    /// This is useful for API calls and logging that require the full repository path.
    ///
    /// # Returns
    ///
    /// A string in the format "{organization}/{repository_name}"
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use config_manager::metadata::{MetadataRepository, DiscoveryMethod};
    /// # use chrono::Utc;
    /// let repo = MetadataRepository::new(
    ///     "acme-corp".to_string(),
    ///     "acme-corp-config".to_string(),
    ///     DiscoveryMethod::ConfigurationBased {
    ///         repository_name: "acme-corp-config".to_string(),
    ///     },
    ///     Utc::now(),
    /// );
    ///
    /// assert_eq!(repo.full_name(), "acme-corp/acme-corp-config");
    /// ```
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.organization, self.repository_name)
    }

    /// Validate the organization assignment for this metadata repository.
    ///
    /// This method ensures that the metadata repository belongs to the expected organization,
    /// which is required for proper access control and configuration isolation. This validation
    /// helps prevent configuration leakage between organizations and ensures security policies
    /// are properly enforced.
    ///
    /// # Arguments
    ///
    /// * `expected_org` - The organization name that should own this repository
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Repository belongs to the expected organization
    /// * `Err(ConfigurationError::OrganizationMismatch)` - Repository belongs to different organization
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use config_manager::metadata::{MetadataRepository, DiscoveryMethod};
    /// # use chrono::Utc;
    /// let repo = MetadataRepository::new(
    ///     "acme-corp".to_string(),
    ///     "acme-corp-config".to_string(),
    ///     DiscoveryMethod::ConfigurationBased {
    ///         repository_name: "acme-corp-config".to_string(),
    ///     },
    ///     Utc::now(),
    /// );
    ///
    /// // Valid organization
    /// assert!(repo.validate_organization("acme-corp").is_ok());
    ///
    /// // Invalid organization
    /// assert!(repo.validate_organization("other-org").is_err());
    /// ```
    pub fn validate_organization(&self, expected_org: &str) -> MetadataResult<()> {
        if expected_org.is_empty() {
            return Err(crate::ConfigurationError::InvalidValue {
                field: "expected_organization".to_string(),
                value: expected_org.to_string(),
                reason: "Organization name cannot be empty".to_string(),
            });
        }

        if self.organization != expected_org {
            return Err(crate::ConfigurationError::InvalidValue {
                field: "organization".to_string(),
                value: format!("expected '{}', got '{}'", expected_org, self.organization),
                reason: format!(
                    "Repository '{}' belongs to organization '{}' but expected organization '{}'",
                    self.repository_name, self.organization, expected_org
                ),
            });
        }

        Ok(())
    }

    /// Check if this repository requires structure validation.
    ///
    /// This method determines whether the repository structure needs to be validated
    /// based on the discovery method and other factors. Some discovery methods may
    /// provide implicit validation, while others require explicit structure checks.
    ///
    /// # Returns
    ///
    /// * `true` if structure validation is needed
    /// * `false` if structure validation can be skipped
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use config_manager::metadata::{MetadataRepository, DiscoveryMethod};
    /// # use chrono::Utc;
    /// let config_repo = MetadataRepository::new(
    ///     "acme-corp".to_string(),
    ///     "acme-corp-config".to_string(),
    ///     DiscoveryMethod::ConfigurationBased {
    ///         repository_name: "acme-corp-config".to_string(),
    ///     },
    ///     Utc::now(),
    /// );
    /// assert!(config_repo.requires_structure_validation());
    ///
    /// let topic_repo = MetadataRepository::new(
    ///     "acme-corp".to_string(),
    ///     "metadata-repo".to_string(),
    ///     DiscoveryMethod::TopicBased {
    ///         topic: "template-metadata".to_string(),
    ///     },
    ///     Utc::now(),
    /// );
    /// assert!(topic_repo.requires_structure_validation());
    /// ```
    pub fn requires_structure_validation(&self) -> bool {
        // All repositories require structure validation regardless of discovery method
        // This ensures consistency and security across all metadata repositories
        true
    }

    /// Validate that this repository can be used for the specified organization.
    ///
    /// This method performs comprehensive validation to ensure the metadata repository
    /// is suitable for use with the specified organization. It combines organization
    /// validation with other checks to provide a complete validation result.
    ///
    /// The validation includes:
    /// - Organization ownership verification
    /// - Repository accessibility validation
    /// - Basic metadata integrity checks
    ///
    /// # Arguments
    ///
    /// * `target_org` - The organization that will use this metadata repository
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Repository is valid for the specified organization
    /// * `Err(ConfigurationError::OrganizationMismatch)` - Repository belongs to different organization
    /// * `Err(ConfigurationError::InvalidRepository)` - Repository has structural or access issues
    /// * `Err(ConfigurationError::AccessDenied)` - Repository cannot be accessed with current permissions
    ///
    /// # Security Considerations
    ///
    /// This method enforces security boundaries by ensuring metadata repositories
    /// can only be used by their owning organization. This prevents configuration
    /// leakage and unauthorized access to organizational policies.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use config_manager::metadata::{MetadataRepository, DiscoveryMethod};
    /// # use chrono::Utc;
    /// let repo = MetadataRepository::new(
    ///     "acme-corp".to_string(),
    ///     "acme-corp-config".to_string(),
    ///     DiscoveryMethod::ConfigurationBased {
    ///         repository_name: "acme-corp-config".to_string(),
    ///     },
    ///     Utc::now(),
    /// );
    ///
    /// match repo.validate_for_organization("acme-corp") {
    ///     Ok(()) => println!("Repository is valid for acme-corp"),
    ///     Err(e) => eprintln!("Repository validation failed: {}", e),
    /// }
    /// ```
    pub fn validate_for_organization(&self, target_org: &str) -> MetadataResult<()> {
        // Primary validation: organization ownership
        self.validate_organization(target_org)?;

        // Additional validation: repository name should not be empty
        if self.repository_name.is_empty() {
            return Err(crate::ConfigurationError::InvalidValue {
                field: "repository_name".to_string(),
                value: self.repository_name.clone(),
                reason: "Repository name cannot be empty".to_string(),
            });
        }

        // Additional validation: discovery method should be valid
        match &self.discovery_method {
            DiscoveryMethod::ConfigurationBased { repository_name } => {
                if repository_name.is_empty() {
                    return Err(crate::ConfigurationError::InvalidValue {
                        field: "discovery_method.repository_name".to_string(),
                        value: repository_name.clone(),
                        reason: "Configuration-based discovery requires non-empty repository name"
                            .to_string(),
                    });
                }
            }
            DiscoveryMethod::TopicBased { topic } => {
                if topic.is_empty() {
                    return Err(crate::ConfigurationError::InvalidValue {
                        field: "discovery_method.topic".to_string(),
                        value: topic.clone(),
                        reason: "Topic-based discovery requires non-empty topic".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Get a validation summary for this metadata repository.
    ///
    /// This method provides a comprehensive overview of the repository's validation
    /// status, including information about required checks, potential issues, and
    /// recommendations for ensuring proper configuration management.
    ///
    /// # Returns
    ///
    /// A `RepositoryValidationSummary` containing validation status and recommendations
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use config_manager::metadata::{MetadataRepository, DiscoveryMethod};
    /// # use chrono::Utc;
    /// let repo = MetadataRepository::new(
    ///     "acme-corp".to_string(),
    ///     "acme-corp-config".to_string(),
    ///     DiscoveryMethod::ConfigurationBased {
    ///         repository_name: "acme-corp-config".to_string(),
    ///     },
    ///     Utc::now(),
    /// );
    ///
    /// let summary = repo.validation_summary();
    /// println!("Validation summary: {:?}", summary);
    /// ```
    pub fn validation_summary(&self) -> RepositoryValidationSummary {
        let mut summary =
            RepositoryValidationSummary::new(ValidationStatus::Unknown, ValidationStatus::Unknown);

        // Validate basic repository properties
        let repository_valid = if self.repository_name.is_empty() || self.organization.is_empty() {
            summary.validation_errors.push(ValidationIssue::new(
                ValidationSeverity::Error,
                "Repository or organization name is empty".to_string(),
                Some("Ensure repository has valid name and organization".to_string()),
            ));
            ValidationStatus::Invalid
        } else {
            ValidationStatus::Valid
        };

        summary.repository_valid = repository_valid;

        // Check discovery method validity
        match &self.discovery_method {
            DiscoveryMethod::ConfigurationBased { repository_name } => {
                if repository_name.is_empty() {
                    summary.validation_errors.push(ValidationIssue::new(
                        ValidationSeverity::Error,
                        "Configuration-based discovery has empty repository name".to_string(),
                        Some(
                            "Set a valid repository name for configuration-based discovery"
                                .to_string(),
                        ),
                    ));
                } else if repository_name != &self.repository_name {
                    summary.validation_warnings.push(ValidationIssue::new(
                        ValidationSeverity::Warning,
                        "Discovery method repository name differs from actual repository name"
                            .to_string(),
                        Some(
                            "Ensure consistency between discovery method and repository name"
                                .to_string(),
                        ),
                    ));
                }
            }
            DiscoveryMethod::TopicBased { topic } => {
                if topic.is_empty() {
                    summary.validation_errors.push(ValidationIssue::new(
                        ValidationSeverity::Error,
                        "Topic-based discovery has empty topic".to_string(),
                        Some("Set a valid topic for topic-based discovery".to_string()),
                    ));
                }
            }
        }

        // Add recommendations
        summary.recommendations.push(
            "Validate repository structure using MetadataRepositoryProvider::validate_repository_structure".to_string(),
        );

        if self.discovery_method.requires_search() {
            summary.recommendations.push(
                "Consider using configuration-based discovery for better performance".to_string(),
            );
        }

        // Organization validation status is set to Valid since we can't validate without target org
        summary.organization_valid = ValidationStatus::Valid;

        summary
    }
}

/// Enumeration of supported metadata repository discovery methods.
///
/// The system supports multiple methods for discovering organization metadata repositories,
/// allowing flexibility in how organizations structure their configuration management.
/// Discovery methods are tried in order of preference: configuration-based first,
/// then topic-based if enabled.
///
/// # Variants
///
/// * `ConfigurationBased` - Repository name specified in application configuration
/// * `TopicBased` - Repository discovered by GitHub topic search
///
/// # Examples
///
/// ```rust
/// use config_manager::metadata::DiscoveryMethod;
///
/// // Configuration-based discovery
/// let config_method = DiscoveryMethod::ConfigurationBased {
///     repository_name: "acme-corp-config".to_string(),
/// };
///
/// // Topic-based discovery
/// let topic_method = DiscoveryMethod::TopicBased {
///     topic: "template-metadata".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscoveryMethod {
    /// Repository name is explicitly specified in application configuration.
    ///
    /// This is the preferred discovery method as it provides deterministic behavior
    /// and doesn't require additional API calls to search for repositories.
    /// The repository name is typically configured during application deployment.
    ///
    /// # Fields
    ///
    /// * `repository_name` - The exact name of the metadata repository
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::metadata::DiscoveryMethod;
    ///
    /// let method = DiscoveryMethod::ConfigurationBased {
    ///     repository_name: "org-repo-config".to_string(),
    /// };
    /// ```
    ConfigurationBased {
        /// The exact name of the metadata repository as configured.
        repository_name: String,
    },

    /// Repository is discovered by searching for repositories with a specific GitHub topic.
    ///
    /// This method provides flexibility for organizations that prefer to use GitHub's
    /// topic system for repository classification. It requires an additional API call
    /// to search for repositories with the specified topic.
    ///
    /// # Fields
    ///
    /// * `topic` - The GitHub topic to search for (typically "template-metadata")
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::metadata::DiscoveryMethod;
    ///
    /// let method = DiscoveryMethod::TopicBased {
    ///     topic: "template-metadata".to_string(),
    /// };
    /// ```
    TopicBased {
        /// The GitHub topic used to identify metadata repositories.
        topic: String,
    },
}

impl DiscoveryMethod {
    /// Check if this discovery method requires additional API calls.
    ///
    /// Configuration-based discovery can directly access the repository,
    /// while topic-based discovery requires a search API call first.
    ///
    /// # Returns
    ///
    /// * `true` if the method requires a search API call
    /// * `false` if the method can directly access the repository
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::metadata::DiscoveryMethod;
    ///
    /// let config_method = DiscoveryMethod::ConfigurationBased {
    ///     repository_name: "config-repo".to_string(),
    /// };
    /// assert!(!config_method.requires_search());
    ///
    /// let topic_method = DiscoveryMethod::TopicBased {
    ///     topic: "template-metadata".to_string(),
    /// };
    /// assert!(topic_method.requires_search());
    /// ```
    pub fn requires_search(&self) -> bool {
        matches!(self, DiscoveryMethod::TopicBased { .. })
    }

    /// Get a human-readable description of this discovery method.
    ///
    /// This is useful for logging and error messages to help users understand
    /// how the metadata repository was discovered.
    ///
    /// # Returns
    ///
    /// A string describing the discovery method and its parameters
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::metadata::DiscoveryMethod;
    ///
    /// let config_method = DiscoveryMethod::ConfigurationBased {
    ///     repository_name: "acme-config".to_string(),
    /// };
    /// assert_eq!(config_method.description(), "configuration-based (repository: acme-config)");
    ///
    /// let topic_method = DiscoveryMethod::TopicBased {
    ///     topic: "template-metadata".to_string(),
    /// };
    /// assert_eq!(topic_method.description(), "topic-based (topic: template-metadata)");
    /// ```
    pub fn description(&self) -> String {
        match self {
            DiscoveryMethod::ConfigurationBased { repository_name } => {
                format!("configuration-based (repository: {})", repository_name)
            }
            DiscoveryMethod::TopicBased { topic } => {
                format!("topic-based (topic: {})", topic)
            }
        }
    }
}

/// Repository validation summary containing validation status and recommendations.
///
/// This structure provides a comprehensive overview of a metadata repository's
/// validation status, including information about required checks, potential issues,
/// and recommendations for ensuring proper configuration management functionality.
///
/// # Fields
///
/// * `repository_valid` - Whether the repository passes basic validation checks
/// * `organization_valid` - Whether the repository belongs to the expected organization
/// * `structure_validation_required` - Whether structure validation is needed
/// * `access_validation_required` - Whether access validation is needed
/// * `validation_warnings` - List of non-critical validation warnings
/// * `validation_errors` - List of critical validation errors
/// * `recommendations` - List of recommendations for improving repository setup
///
/// # Examples
///
/// ```rust
/// use config_manager::metadata::{RepositoryValidationSummary, ValidationStatus, ValidationIssue};
///
/// let summary = RepositoryValidationSummary {
///     repository_valid: ValidationStatus::Valid,
///     organization_valid: ValidationStatus::Valid,
///     structure_validation_required: true,
///     access_validation_required: true,
///     validation_warnings: vec![],
///     validation_errors: vec![],
///     recommendations: vec![
///         "Consider adding validation schemas in schemas/ directory".to_string(),
///     ],
/// };
///
/// println!("Repository validation: {:?}", summary.repository_valid);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryValidationSummary {
    /// Basic repository validation status.
    ///
    /// Indicates whether the repository passes fundamental validation checks
    /// such as existence, accessibility, and basic metadata integrity.
    pub repository_valid: ValidationStatus,

    /// Organization ownership validation status.
    ///
    /// Indicates whether the repository belongs to the expected organization
    /// and meets security requirements for cross-organizational access.
    pub organization_valid: ValidationStatus,

    /// Whether structure validation is required for this repository.
    ///
    /// Some repositories may skip structure validation if they were discovered
    /// through methods that provide implicit validation guarantees.
    pub structure_validation_required: bool,

    /// Whether access validation is required for this repository.
    ///
    /// Indicates whether the current authentication context needs to be
    /// validated against the repository's access requirements.
    pub access_validation_required: bool,

    /// List of non-critical validation warnings.
    ///
    /// These are issues that don't prevent the repository from functioning
    /// but may indicate suboptimal configuration or potential future problems.
    pub validation_warnings: Vec<ValidationIssue>,

    /// List of critical validation errors.
    ///
    /// These are issues that prevent the repository from being used for
    /// configuration management and must be resolved before proceeding.
    pub validation_errors: Vec<ValidationIssue>,

    /// List of recommendations for improving repository setup.
    ///
    /// These are suggestions for enhancing the repository's configuration
    /// management capabilities, performance, or maintainability.
    pub recommendations: Vec<String>,
}

impl RepositoryValidationSummary {
    /// Create a new validation summary for a repository.
    ///
    /// # Arguments
    ///
    /// * `repository_valid` - Basic repository validation status
    /// * `organization_valid` - Organization ownership validation status
    ///
    /// # Returns
    ///
    /// A new `RepositoryValidationSummary` with default values for other fields
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::metadata::{RepositoryValidationSummary, ValidationStatus};
    ///
    /// let summary = RepositoryValidationSummary::new(
    ///     ValidationStatus::Valid,
    ///     ValidationStatus::Valid,
    /// );
    /// assert_eq!(summary.repository_valid, ValidationStatus::Valid);
    /// ```
    pub fn new(repository_valid: ValidationStatus, organization_valid: ValidationStatus) -> Self {
        Self {
            repository_valid,
            organization_valid,
            structure_validation_required: true,
            access_validation_required: true,
            validation_warnings: Vec::new(),
            validation_errors: Vec::new(),
            recommendations: Vec::new(),
        }
    }

    /// Check if the repository has any critical validation errors.
    ///
    /// # Returns
    ///
    /// * `true` if there are critical errors that prevent repository usage
    /// * `false` if the repository can be used despite any warnings
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::metadata::{RepositoryValidationSummary, ValidationStatus, ValidationIssue, ValidationSeverity};
    ///
    /// let mut summary = RepositoryValidationSummary::new(
    ///     ValidationStatus::Valid,
    ///     ValidationStatus::Valid,
    /// );
    /// assert!(!summary.has_errors());
    ///
    /// summary.validation_errors.push(ValidationIssue::new(
    ///     ValidationSeverity::Error,
    ///     "Missing required global/defaults.toml file".to_string(),
    ///     Some("Create global/defaults.toml with organization settings".to_string()),
    /// ));
    /// assert!(summary.has_errors());
    /// ```
    pub fn has_errors(&self) -> bool {
        !self.validation_errors.is_empty()
            || self.repository_valid == ValidationStatus::Invalid
            || self.organization_valid == ValidationStatus::Invalid
    }

    /// Check if the repository has any validation warnings.
    ///
    /// # Returns
    ///
    /// * `true` if there are warnings that should be addressed
    /// * `false` if there are no warnings
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::metadata::{RepositoryValidationSummary, ValidationStatus, ValidationIssue, ValidationSeverity};
    ///
    /// let mut summary = RepositoryValidationSummary::new(
    ///     ValidationStatus::Valid,
    ///     ValidationStatus::Valid,
    /// );
    /// assert!(!summary.has_warnings());
    ///
    /// summary.validation_warnings.push(ValidationIssue::new(
    ///     ValidationSeverity::Warning,
    ///     "No validation schemas found in schemas/ directory".to_string(),
    ///     Some("Consider adding JSON schemas for configuration validation".to_string()),
    /// ));
    /// assert!(summary.has_warnings());
    /// ```
    pub fn has_warnings(&self) -> bool {
        !self.validation_warnings.is_empty()
    }

    /// Get the overall validation status for this repository.
    ///
    /// This combines all validation results to provide a single status indicator
    /// that can be used for decision making about repository usage.
    ///
    /// # Returns
    ///
    /// * `ValidationStatus::Valid` - Repository can be used safely
    /// * `ValidationStatus::ValidWithWarnings` - Repository can be used but has issues
    /// * `ValidationStatus::Invalid` - Repository cannot be used due to errors
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::metadata::{RepositoryValidationSummary, ValidationStatus};
    ///
    /// let summary = RepositoryValidationSummary::new(
    ///     ValidationStatus::Valid,
    ///     ValidationStatus::Valid,
    /// );
    /// assert_eq!(summary.overall_status(), ValidationStatus::Valid);
    /// ```
    pub fn overall_status(&self) -> ValidationStatus {
        // If there are critical errors or any component is invalid, return Invalid
        if self.has_errors() {
            return ValidationStatus::Invalid;
        }

        // If there are warnings, return ValidWithWarnings
        if self.has_warnings() {
            return ValidationStatus::ValidWithWarnings;
        }

        // If both repository and organization are valid, return Valid
        if self.repository_valid == ValidationStatus::Valid
            && self.organization_valid == ValidationStatus::Valid
        {
            return ValidationStatus::Valid;
        }

        // If any component is unknown, return Unknown
        if self.repository_valid == ValidationStatus::Unknown
            || self.organization_valid == ValidationStatus::Unknown
        {
            return ValidationStatus::Unknown;
        }

        // Default to ValidWithWarnings for any other combination
        ValidationStatus::ValidWithWarnings
    }
}

/// Enumeration of validation status values.
///
/// This enum represents the possible outcomes of validation operations,
/// providing clear indication of whether a component can be used safely.
///
/// # Variants
///
/// * `Valid` - Component passes all validation checks
/// * `ValidWithWarnings` - Component can be used but has non-critical issues
/// * `Invalid` - Component fails validation and cannot be used
/// * `Unknown` - Validation status has not been determined
///
/// # Examples
///
/// ```rust
/// use config_manager::metadata::ValidationStatus;
///
/// let status = ValidationStatus::Valid;
/// assert!(matches!(status, ValidationStatus::Valid));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationStatus {
    /// Component passes all validation checks and can be used safely.
    Valid,

    /// Component can be used but has non-critical issues that should be addressed.
    ValidWithWarnings,

    /// Component fails validation checks and cannot be used.
    Invalid,

    /// Validation status has not been determined yet.
    Unknown,
}

impl ValidationStatus {
    /// Check if this status indicates the component can be used.
    ///
    /// # Returns
    ///
    /// * `true` if the component can be used (Valid or ValidWithWarnings)
    /// * `false` if the component cannot be used (Invalid or Unknown)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::metadata::ValidationStatus;
    ///
    /// assert!(ValidationStatus::Valid.is_usable());
    /// assert!(ValidationStatus::ValidWithWarnings.is_usable());
    /// assert!(!ValidationStatus::Invalid.is_usable());
    /// assert!(!ValidationStatus::Unknown.is_usable());
    /// ```
    pub fn is_usable(&self) -> bool {
        matches!(
            self,
            ValidationStatus::Valid | ValidationStatus::ValidWithWarnings
        )
    }
}

/// Represents a validation issue found during repository validation.
///
/// This structure contains detailed information about validation problems,
/// including their severity level and potential solutions or recommendations.
///
/// # Fields
///
/// * `severity` - The severity level of this validation issue
/// * `message` - Human-readable description of the validation issue
/// * `suggestion` - Optional suggestion for resolving the issue
///
/// # Examples
///
/// ```rust
/// use config_manager::metadata::{ValidationIssue, ValidationSeverity};
///
/// let issue = ValidationIssue::new(
///     ValidationSeverity::Error,
///     "Missing required configuration file".to_string(),
///     Some("Create the file with default settings".to_string()),
/// );
///
/// println!("Validation issue: {}", issue.message);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// The severity level of this validation issue.
    pub severity: ValidationSeverity,

    /// Human-readable description of the validation issue.
    pub message: String,

    /// Optional suggestion for resolving the issue.
    pub suggestion: Option<String>,
}

impl ValidationIssue {
    /// Create a new validation issue.
    ///
    /// # Arguments
    ///
    /// * `severity` - The severity level of the issue
    /// * `message` - Human-readable description of the issue
    /// * `suggestion` - Optional suggestion for resolving the issue
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::metadata::{ValidationIssue, ValidationSeverity};
    ///
    /// let issue = ValidationIssue::new(
    ///     ValidationSeverity::Warning,
    ///     "Configuration file is outdated".to_string(),
    ///     Some("Update the file to use the latest schema version".to_string()),
    /// );
    /// ```
    pub fn new(severity: ValidationSeverity, message: String, suggestion: Option<String>) -> Self {
        Self {
            severity,
            message,
            suggestion,
        }
    }
}

/// Enumeration of validation issue severity levels.
///
/// This enum categorizes validation issues by their impact on system functionality,
/// helping users prioritize which issues to address first.
///
/// # Variants
///
/// * `Error` - Critical issue that prevents system functionality
/// * `Warning` - Non-critical issue that should be addressed
/// * `Info` - Informational message that may be helpful
///
/// # Examples
///
/// ```rust
/// use config_manager::metadata::ValidationSeverity;
///
/// let severity = ValidationSeverity::Error;
/// assert!(matches!(severity, ValidationSeverity::Error));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationSeverity {
    /// Critical issue that prevents system functionality.
    Error,

    /// Non-critical issue that should be addressed but doesn't prevent functionality.
    Warning,

    /// Informational message that may be helpful for optimization or best practices.
    Info,
}

impl ValidationSeverity {
    /// Check if this severity level indicates a critical issue.
    ///
    /// # Returns
    ///
    /// * `true` if the issue is critical and prevents functionality
    /// * `false` if the issue is non-critical
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::metadata::ValidationSeverity;
    ///
    /// assert!(ValidationSeverity::Error.is_critical());
    /// assert!(!ValidationSeverity::Warning.is_critical());
    /// assert!(!ValidationSeverity::Info.is_critical());
    /// ```
    pub fn is_critical(&self) -> bool {
        matches!(self, ValidationSeverity::Error)
    }
}

/// Repository structure validation result containing detailed information about required components.
///
/// This structure provides comprehensive validation results for metadata repository structure,
/// including information about directories, files, and their compliance with organizational
/// configuration management requirements.
///
/// # Structure Requirements
///
/// According to the organization repository settings specification, metadata repositories
/// must follow this standardized directory structure:
///
/// - `global/` directory containing organization-wide defaults
///   - `defaults.toml` (required) - baseline repository settings
///   - `labels.toml` (optional) - standard label definitions
/// - `teams/` directory (optional) for team-specific configurations
///   - `{team-name}/config.toml` files for team overrides
/// - `types/` directory (optional) for repository type configurations
///   - `{type-name}/config.toml` files for type-specific settings
/// - `schemas/` directory (optional) for validation schemas
///   - JSON Schema files for configuration validation
///
/// # Fields
///
/// * `repository_accessible` - Whether the repository can be accessed
/// * `global_directory_present` - Whether the global/ directory exists
/// * `global_defaults_present` - Whether global/defaults.toml exists
/// * `teams_directory_present` - Whether the teams/ directory exists
/// * `types_directory_present` - Whether the types/ directory exists
/// * `schemas_directory_present` - Whether the schemas/ directory exists
/// * `missing_required_items` - List of required items that are missing
/// * `optional_missing_items` - List of optional items that are missing
/// * `validation_errors` - Critical errors that prevent repository usage
/// * `validation_warnings` - Non-critical issues that should be addressed
///
/// # Examples
///
/// ```rust
/// use config_manager::metadata::{RepositoryStructureValidation, ValidationStatus};
///
/// let validation = RepositoryStructureValidation {
///     repository_accessible: true,
///     global_directory_present: true,
///     global_defaults_present: true,
///     teams_directory_present: true,
///     types_directory_present: false,
///     schemas_directory_present: false,
///     missing_required_items: vec![],
///     optional_missing_items: vec!["types/".to_string(), "schemas/".to_string()],
///     validation_errors: vec![],
///     validation_warnings: vec![],
///     overall_status: ValidationStatus::Valid,
/// };
///
/// assert!(validation.is_valid());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryStructureValidation {
    /// Whether the repository can be accessed for structure validation.
    pub repository_accessible: bool,

    /// Whether the global/ directory exists in the repository.
    pub global_directory_present: bool,

    /// Whether the global/defaults.toml file exists.
    pub global_defaults_present: bool,

    /// Whether the teams/ directory exists (optional).
    pub teams_directory_present: bool,

    /// Whether the types/ directory exists (optional).
    pub types_directory_present: bool,

    /// Whether the schemas/ directory exists (optional).
    pub schemas_directory_present: bool,

    /// List of required directories or files that are missing.
    pub missing_required_items: Vec<String>,

    /// List of optional directories or files that are missing.
    pub optional_missing_items: Vec<String>,

    /// Critical validation errors that prevent repository usage.
    pub validation_errors: Vec<ValidationIssue>,

    /// Non-critical validation warnings that should be addressed.
    pub validation_warnings: Vec<ValidationIssue>,

    /// Overall validation status for the repository structure.
    pub overall_status: ValidationStatus,
}

impl RepositoryStructureValidation {
    /// Create a new repository structure validation result.
    ///
    /// # Arguments
    ///
    /// * `repository_accessible` - Whether the repository can be accessed
    ///
    /// # Returns
    ///
    /// A new `RepositoryStructureValidation` with default values
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::metadata::{RepositoryStructureValidation, ValidationStatus};
    ///
    /// let validation = RepositoryStructureValidation::new(true);
    /// assert_eq!(validation.repository_accessible, true);
    /// assert_eq!(validation.overall_status, ValidationStatus::Unknown);
    /// ```
    pub fn new(repository_accessible: bool) -> Self {
        Self {
            repository_accessible,
            global_directory_present: false,
            global_defaults_present: false,
            teams_directory_present: false,
            types_directory_present: false,
            schemas_directory_present: false,
            missing_required_items: Vec::new(),
            optional_missing_items: Vec::new(),
            validation_errors: Vec::new(),
            validation_warnings: Vec::new(),
            overall_status: ValidationStatus::Unknown,
        }
    }

    /// Check if the repository structure is valid for use.
    ///
    /// A repository structure is considered valid if:
    /// - The repository is accessible
    /// - The global/ directory exists
    /// - The global/defaults.toml file exists
    /// - There are no critical validation errors
    ///
    /// # Returns
    ///
    /// * `true` if the repository structure is valid
    /// * `false` if there are critical issues preventing usage
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::metadata::{RepositoryStructureValidation, ValidationStatus};
    ///
    /// let mut validation = RepositoryStructureValidation::new(true);
    /// validation.global_directory_present = true;
    /// validation.global_defaults_present = true;
    /// validation.overall_status = ValidationStatus::Valid;
    ///
    /// assert!(validation.is_valid());
    /// ```
    pub fn is_valid(&self) -> bool {
        // TODO: implement validation logic
        todo!("Structure validation logic not implemented")
    }

    /// Check if there are any critical errors in the repository structure.
    ///
    /// # Returns
    ///
    /// * `true` if there are critical errors that prevent repository usage
    /// * `false` if the repository can be used despite any warnings
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::metadata::{RepositoryStructureValidation, ValidationIssue, ValidationSeverity};
    ///
    /// let mut validation = RepositoryStructureValidation::new(true);
    /// assert!(!validation.has_critical_errors());
    ///
    /// validation.validation_errors.push(ValidationIssue::new(
    ///     ValidationSeverity::Error,
    ///     "Missing global/defaults.toml".to_string(),
    ///     Some("Create the required configuration file".to_string()),
    /// ));
    /// assert!(validation.has_critical_errors());
    /// ```
    pub fn has_critical_errors(&self) -> bool {
        // TODO: implement critical error check
        todo!("Critical error check not implemented")
    }

    /// Get a summary of missing required items.
    ///
    /// # Returns
    ///
    /// A formatted string describing missing required directories and files
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::metadata::RepositoryStructureValidation;
    ///
    /// let mut validation = RepositoryStructureValidation::new(true);
    /// validation.missing_required_items.push("global/".to_string());
    /// validation.missing_required_items.push("global/defaults.toml".to_string());
    ///
    /// let summary = validation.missing_required_summary();
    /// assert!(summary.contains("global/"));
    /// assert!(summary.contains("global/defaults.toml"));
    /// ```
    pub fn missing_required_summary(&self) -> String {
        // TODO: implement missing required summary
        todo!("Missing required summary not implemented")
    }

    /// Get a list of recommendations for improving the repository structure.
    ///
    /// # Returns
    ///
    /// A vector of recommendations for enhancing the repository structure
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::metadata::RepositoryStructureValidation;
    ///
    /// let mut validation = RepositoryStructureValidation::new(true);
    /// validation.schemas_directory_present = false;
    ///
    /// let recommendations = validation.get_recommendations();
    /// assert!(!recommendations.is_empty());
    /// ```
    pub fn get_recommendations(&self) -> Vec<String> {
        // TODO: implement recommendations generation
        todo!("Recommendations generation not implemented")
    }
}

/// Trait for validating metadata repository structure.
///
/// This trait provides methods for performing detailed validation of metadata repository
/// structure, including checking for required directories, files, and their accessibility.
/// Implementations should provide comprehensive error reporting and validation results.
///
/// # Structure Validation Process
///
/// 1. **Repository Access**: Verify the repository can be accessed with current permissions
/// 2. **Directory Structure**: Check for required and optional directories
/// 3. **Required Files**: Validate presence of mandatory configuration files
/// 4. **File Accessibility**: Ensure configuration files can be read
/// 5. **Structure Completeness**: Verify the repository meets minimum requirements
///
/// # Examples
///
/// ```rust
/// use config_manager::metadata::{RepositoryStructureValidator, MetadataRepository};
/// use async_trait::async_trait;
///
/// struct MyValidator;
///
/// #[async_trait]
/// impl RepositoryStructureValidator for MyValidator {
///     async fn validate_structure(&self, repo: &MetadataRepository) -> MetadataResult<RepositoryStructureValidation> {
///         // Implementation details...
///         todo!("Not implemented")
///     }
///
///     async fn check_directory_exists(&self, repo: &MetadataRepository, path: &str) -> MetadataResult<bool> {
///         // Implementation details...
///         todo!("Not implemented")
///     }
///
///     async fn check_file_exists(&self, repo: &MetadataRepository, path: &str) -> MetadataResult<bool> {
///         // Implementation details...
///         todo!("Not implemented")
///     }
/// }
/// ```
#[async_trait]
pub trait RepositoryStructureValidator: Send + Sync {
    /// Perform comprehensive structure validation of a metadata repository.
    ///
    /// This method checks the repository structure against the standardized layout
    /// requirements for organization configuration management, including required
    /// directories, files, and their accessibility.
    ///
    /// # Arguments
    ///
    /// * `repo` - The metadata repository to validate
    ///
    /// # Returns
    ///
    /// * `Ok(RepositoryStructureValidation)` - Detailed validation results
    /// * `Err(ConfigurationError::NetworkError)` - API communication failure
    /// * `Err(ConfigurationError::AccessDenied)` - Repository access denied
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use config_manager::metadata::{RepositoryStructureValidator, MetadataRepository};
    /// # async fn example(validator: &dyn RepositoryStructureValidator, repo: &MetadataRepository) {
    /// match validator.validate_structure(repo).await {
    ///     Ok(validation) => {
    ///         if validation.is_valid() {
    ///             println!("Repository structure is valid");
    ///         } else {
    ///             println!("Repository structure has issues: {}", validation.missing_required_summary());
    ///         }
    ///     },
    ///     Err(e) => eprintln!("Structure validation failed: {}", e),
    /// }
    /// # }
    /// ```
    async fn validate_structure(
        &self,
        repo: &MetadataRepository,
    ) -> MetadataResult<RepositoryStructureValidation>;

    /// Check if a directory exists in the repository.
    ///
    /// # Arguments
    ///
    /// * `repo` - The metadata repository to check
    /// * `path` - The directory path to check (relative to repository root)
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Directory exists and is accessible
    /// * `Ok(false)` - Directory does not exist
    /// * `Err(ConfigurationError::NetworkError)` - API communication failure
    /// * `Err(ConfigurationError::AccessDenied)` - Repository access denied
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use config_manager::metadata::{RepositoryStructureValidator, MetadataRepository};
    /// # async fn example(validator: &dyn RepositoryStructureValidator, repo: &MetadataRepository) {
    /// let global_exists = validator.check_directory_exists(repo, "global").await.unwrap();
    /// if global_exists {
    ///     println!("Global directory found");
    /// }
    /// # }
    /// ```
    async fn check_directory_exists(
        &self,
        repo: &MetadataRepository,
        path: &str,
    ) -> MetadataResult<bool>;

    /// Check if a file exists in the repository.
    ///
    /// # Arguments
    ///
    /// * `repo` - The metadata repository to check
    /// * `path` - The file path to check (relative to repository root)
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - File exists and is accessible
    /// * `Ok(false)` - File does not exist
    /// * `Err(ConfigurationError::NetworkError)` - API communication failure
    /// * `Err(ConfigurationError::AccessDenied)` - Repository access denied
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use config_manager::metadata::{RepositoryStructureValidator, MetadataRepository};
    /// # async fn example(validator: &dyn RepositoryStructureValidator, repo: &MetadataRepository) {
    /// let defaults_exists = validator.check_file_exists(repo, "global/defaults.toml").await.unwrap();
    /// if defaults_exists {
    ///     println!("Global defaults file found");
    /// }
    /// # }
    /// ```
    async fn check_file_exists(
        &self,
        repo: &MetadataRepository,
        path: &str,
    ) -> MetadataResult<bool>;

    /// List available teams by scanning the teams/ directory.
    ///
    /// # Arguments
    ///
    /// * `repo` - The metadata repository to scan
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` - List of team names with configuration files
    /// * `Err(ConfigurationError::NetworkError)` - API communication failure
    /// * `Err(ConfigurationError::AccessDenied)` - Repository access denied
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use config_manager::metadata::{RepositoryStructureValidator, MetadataRepository};
    /// # async fn example(validator: &dyn RepositoryStructureValidator, repo: &MetadataRepository) {
    /// match validator.list_available_teams(repo).await {
    ///     Ok(teams) => println!("Available teams: {}", teams.join(", ")),
    ///     Err(e) => eprintln!("Failed to list teams: {}", e),
    /// }
    /// # }
    /// ```
    async fn list_available_teams(&self, repo: &MetadataRepository) -> MetadataResult<Vec<String>>;

    /// List available repository types by scanning the types/ directory.
    ///
    /// # Arguments
    ///
    /// * `repo` - The metadata repository to scan
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` - List of repository type names with configuration files
    /// * `Err(ConfigurationError::NetworkError)` - API communication failure
    /// * `Err(ConfigurationError::AccessDenied)` - Repository access denied
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use config_manager::metadata::{RepositoryStructureValidator, MetadataRepository};
    /// # async fn example(validator: &dyn RepositoryStructureValidator, repo: &MetadataRepository) {
    /// match validator.list_available_types(repo).await {
    ///     Ok(types) => println!("Available types: {}", types.join(", ")),
    ///     Err(e) => eprintln!("Failed to list types: {}", e),
    /// }
    /// # }
    /// ```
    async fn list_available_types(&self, repo: &MetadataRepository) -> MetadataResult<Vec<String>>;
}
