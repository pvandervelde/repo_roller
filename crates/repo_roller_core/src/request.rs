//! Repository creation request types
//!
//! Types for requesting repository creation operations.
//! See specs/interfaces/repository-domain.md for complete specifications.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{OrganizationName, RepositoryName, RepositoryVisibility, TemplateName, Timestamp};

#[cfg(test)]
#[path = "request_tests.rs"]
mod tests;

/// Content generation strategy for repository creation.
///
/// Specifies how repository content should be generated during creation.
/// This enum determines which ContentProvider implementation is used.
///
/// # Variants
///
/// - `Template`: Use template repository (current/default behavior)
/// - `Empty`: Create no files (empty repository)
/// - `CustomInit`: Create selected initialization files only
///
/// # Examples
///
/// ```
/// use repo_roller_core::ContentStrategy;
///
/// // Template-based (default)
/// let strategy = ContentStrategy::Template;
///
/// // Empty repository
/// let strategy = ContentStrategy::Empty;
///
/// // Custom initialization
/// let strategy = ContentStrategy::CustomInit {
///     include_readme: true,
///     include_gitignore: true,
/// };
/// ```
///
/// See specs/interfaces/repository-creation-modes.md#contentstrategy-enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentStrategy {
    /// Fetch and process files from template repository
    Template,

    /// Create no files (empty repository)
    #[serde(rename = "empty")]
    Empty,

    /// Create custom initialization files
    #[serde(rename = "custom_init")]
    CustomInit {
        /// Create README.md file
        include_readme: bool,

        /// Create .gitignore file
        include_gitignore: bool,
    },
}

impl Default for ContentStrategy {
    fn default() -> Self {
        Self::Template
    }
}

/// Request for creating a new repository with validated types.
///
/// This is the new typed API for repository creation that uses
/// branded types for type safety and validation.
///
/// # Examples
///
/// ```rust
/// use repo_roller_core::{
///     RepositoryCreationRequest, RepositoryName, OrganizationName,
///     TemplateName, RepositoryVisibility, ContentStrategy
/// };
/// use std::collections::HashMap;
///
/// // Template-based repository (current behavior)
/// let request = RepositoryCreationRequest {
///     name: RepositoryName::new("my-new-repo").unwrap(),
///     owner: OrganizationName::new("my-org").unwrap(),
///     template: Some(TemplateName::new("rust-library").unwrap()),
///     variables: HashMap::new(),
///     visibility: Some(RepositoryVisibility::Private),
///     content_strategy: ContentStrategy::Template,
/// };
///
/// // Empty repository with template settings
/// let request = RepositoryCreationRequest {
///     name: RepositoryName::new("my-empty-repo").unwrap(),
///     owner: OrganizationName::new("my-org").unwrap(),
///     template: Some(TemplateName::new("github-actions").unwrap()),
///     variables: HashMap::new(),
///     visibility: None,
///     content_strategy: ContentStrategy::Empty,
/// };
/// ```
///
/// See specs/interfaces/repository-domain.md#repositorycreationrequest
#[derive(Debug, Clone)]
pub struct RepositoryCreationRequest {
    /// The name of the repository to create
    pub name: RepositoryName,

    /// The organization or user that will own the repository
    pub owner: OrganizationName,

    /// Optional template name for content and settings.
    ///
    /// When `Some`, template is loaded for:
    /// - File content (if ContentStrategy::Template)
    /// - Repository settings
    /// - Default values for variables
    ///
    /// When `None`:
    /// - Organization defaults used for settings
    /// - Only Empty or CustomInit strategies valid
    pub template: Option<TemplateName>,

    /// Template variables for variable substitution during processing
    pub variables: HashMap<String, String>,

    /// Explicit visibility preference (optional, resolved via hierarchy if None)
    pub visibility: Option<RepositoryVisibility>,

    /// Content generation strategy.
    ///
    /// Determines how repository content is generated:
    /// - Template: Fetch and process template files (default)
    /// - Empty: Create no files
    /// - CustomInit: Create selected initialization files
    ///
    /// See [`ContentStrategy`] for details.
    pub content_strategy: ContentStrategy,
}

/// Result of a successful repository creation operation.
///
/// Contains metadata about the newly created repository.
///
/// # Examples
///
/// ```rust
/// use repo_roller_core::{RepositoryCreationResult, Timestamp};
///
/// let result = RepositoryCreationResult {
///     repository_url: "https://github.com/my-org/my-repo".to_string(),
///     repository_id: "R_kgDOABCDEF".to_string(),
///     created_at: Timestamp::now(),
///     default_branch: "main".to_string(),
/// };
/// ```
///
/// See specs/interfaces/repository-domain.md#repositorycreationresult
#[derive(Debug, Clone)]
pub struct RepositoryCreationResult {
    /// The URL of the created repository
    pub repository_url: String,

    /// The GitHub repository ID
    pub repository_id: String,

    /// When the repository was created
    pub created_at: Timestamp,

    /// The default branch name
    pub default_branch: String,
}

/// Builder for constructing RepositoryCreationRequest instances.
///
/// Provides an ergonomic API for building repository creation requests
/// with optional template, variables, visibility, and content strategy.
///
/// # Examples
///
/// ```rust
/// use repo_roller_core::{
///     RepositoryCreationRequestBuilder, RepositoryName, OrganizationName,
///     TemplateName, RepositoryVisibility, ContentStrategy
/// };
///
/// // Template-based repository (current behavior)
/// let request = RepositoryCreationRequestBuilder::new(
///     RepositoryName::new("my-repo").unwrap(),
///     OrganizationName::new("my-org").unwrap(),
/// )
/// .template(TemplateName::new("rust-library").unwrap())
/// .variable("project_name", "MyProject")
/// .variable("author", "John Doe")
/// .with_visibility(RepositoryVisibility::Private)
/// .build();
///
/// // Empty repository with template settings
/// let request = RepositoryCreationRequestBuilder::new(
///     RepositoryName::new("my-empty-repo").unwrap(),
///     OrganizationName::new("my-org").unwrap(),
/// )
/// .template(TemplateName::new("github-actions").unwrap())
/// .content_strategy(ContentStrategy::Empty)
/// .build();
///
/// // Custom initialization without template
/// let request = RepositoryCreationRequestBuilder::new(
///     RepositoryName::new("quick-start").unwrap(),
///     OrganizationName::new("my-org").unwrap(),
/// )
/// .content_strategy(ContentStrategy::CustomInit {
///     include_readme: true,
///     include_gitignore: true,
/// })
/// .build();
/// ```
#[derive(Debug, Clone)]
pub struct RepositoryCreationRequestBuilder {
    name: Option<RepositoryName>,
    owner: Option<OrganizationName>,
    template: Option<TemplateName>,
    variables: Option<HashMap<String, String>>,
    visibility: Option<RepositoryVisibility>,
    content_strategy: Option<ContentStrategy>,
}

impl RepositoryCreationRequestBuilder {
    /// Create a new builder with required fields.
    ///
    /// Template is now optional and set via the `template()` method.
    ///
    /// # Examples
    ///
    /// ```
    /// use repo_roller_core::{RepositoryCreationRequestBuilder, RepositoryName, OrganizationName};
    ///
    /// let builder = RepositoryCreationRequestBuilder::new(
    ///     RepositoryName::new("my-repo").unwrap(),
    ///     OrganizationName::new("my-org").unwrap(),
    /// );
    /// ```
    pub fn new(name: RepositoryName, owner: OrganizationName) -> Self {
        Self {
            name: Some(name),
            owner: Some(owner),
            template: None,
            variables: None,
            visibility: None,
            content_strategy: None,
        }
    }

    /// Set the template for content and settings.
    ///
    /// When provided, the template is loaded for file content (if using Template strategy)
    /// and repository settings.
    ///
    /// # Examples
    ///
    /// ```
    /// # use repo_roller_core::*;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let request = RepositoryCreationRequestBuilder::new(
    ///     RepositoryName::new("my-repo")?,
    ///     OrganizationName::new("my-org")?,
    /// )
    /// .template(TemplateName::new("rust-service")?)
    /// .build();
    /// # Ok(())
    /// # }
    /// ```
    pub fn template(mut self, template: TemplateName) -> Self {
        self.template = Some(template);
        self
    }

    /// Add a single template variable.
    ///
    /// If a variable with the same key already exists, it will be overwritten.
    pub fn variable(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.variables
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value.into());
        self
    }

    /// Add multiple template variables at once.
    ///
    /// Existing variables with the same keys will be overwritten.
    pub fn variables(mut self, vars: HashMap<String, String>) -> Self {
        self.variables.get_or_insert_with(HashMap::new).extend(vars);
        self
    }

    /// Set explicit visibility preference.
    ///
    /// This visibility will be validated against organization policies
    /// during repository creation.
    pub fn with_visibility(mut self, visibility: RepositoryVisibility) -> Self {
        self.visibility = Some(visibility);
        self
    }

    /// Set the content generation strategy.
    ///
    /// Determines how repository content is generated. Defaults to Template strategy.
    ///
    /// # Examples
    ///
    /// ```
    /// # use repo_roller_core::*;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Empty repository
    /// let request = RepositoryCreationRequestBuilder::new(
    ///     RepositoryName::new("my-repo")?,
    ///     OrganizationName::new("my-org")?,
    /// )
    /// .template(TemplateName::new("github-actions")?)
    /// .content_strategy(ContentStrategy::Empty)
    /// .build();
    ///
    /// // Custom initialization
    /// let request = RepositoryCreationRequestBuilder::new(
    ///     RepositoryName::new("my-repo")?,
    ///     OrganizationName::new("my-org")?,
    /// )
    /// .content_strategy(ContentStrategy::CustomInit {
    ///     include_readme: true,
    ///     include_gitignore: true,
    /// })
    /// .build();
    /// # Ok(())
    /// # }
    /// ```
    pub fn content_strategy(mut self, strategy: ContentStrategy) -> Self {
        self.content_strategy = Some(strategy);
        self
    }

    /// Build the final RepositoryCreationRequest.
    ///
    /// # Panics
    ///
    /// Panics if ContentStrategy::Template is used without setting a template.
    pub fn build(self) -> RepositoryCreationRequest {
        let name = self.name.expect("name is required");
        let owner = self.owner.expect("owner is required");
        let content_strategy = self.content_strategy.unwrap_or_default();

        // Validation: Template strategy requires template
        if matches!(content_strategy, ContentStrategy::Template) && self.template.is_none() {
            panic!(
                "ContentStrategy::Template requires template to be set. Use .template() method."
            );
        }

        RepositoryCreationRequest {
            name,
            owner,
            template: self.template,
            variables: self.variables.unwrap_or_default(),
            visibility: self.visibility,
            content_strategy,
        }
    }
}
