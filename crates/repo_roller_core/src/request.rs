//! Repository creation request types
//!
//! Types for requesting repository creation operations.
//! See specs/interfaces/repository-domain.md for complete specifications.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    permissions::AccessLevel, OrganizationName, RepositoryName, RepositoryVisibility,
    SourceRepositoryReference, TemplateName, Timestamp,
};

#[cfg(test)]
#[path = "request_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "creation_mode_tests.rs"]
mod creation_mode_tests;

#[cfg(test)]
#[path = "source_reference_proptests.rs"]
mod source_reference_proptests;

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
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentStrategy {
    /// Fetch and process files from template repository
    #[default]
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

/// Governs how the GitHub repository itself comes into existence.
///
/// Orthogonal to [`ContentStrategy`], which governs how file content is
/// produced for the *Standard* creation path only. `Fork` and `Mirror`
/// bypass `ContentStrategy` entirely: GitHub performs the fork (content +
/// history) for `Fork`, local `git` operations provide content for
/// `Mirror`.
///
/// See `docs/spec/interfaces/fork-mirror-operations.md` §4.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CreationMode {
    /// Existing behavior: create (org/user) repo, then run `content_strategy`.
    #[default]
    Standard,

    /// Create a GitHub fork of `source`. Bypasses `content_strategy` and
    /// local git entirely — GitHub performs the fork (content + history).
    Fork {
        /// The repository to fork from.
        source: SourceRepositoryReference,
    },

    /// Create an independent repository that is not part of the GitHub
    /// fork network.
    Mirror {
        /// The repository to mirror from.
        source: SourceRepositoryReference,
        /// `true`: full commit history mirror, HEAD = source default branch.
        /// `false`: source-only mirror, delivered via PR, no history.
        preserve_history: bool,
    },
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
///     TemplateName, RepositoryVisibility, ContentStrategy, CreationMode
/// };
/// use repo_roller_core::permissions::AccessLevel;
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
///     creation_mode: CreationMode::Standard,
///     teams: HashMap::new(),
///     collaborators: HashMap::new(),
///     actor_login: "alice".to_string(),
/// };
///
/// // Empty repository with team permissions
/// let mut teams = HashMap::new();
/// teams.insert("platform".to_string(), AccessLevel::Write);
/// let request = RepositoryCreationRequest {
///     name: RepositoryName::new("my-empty-repo").unwrap(),
///     owner: OrganizationName::new("my-org").unwrap(),
///     template: Some(TemplateName::new("github-actions").unwrap()),
///     variables: HashMap::new(),
///     visibility: None,
///     content_strategy: ContentStrategy::Empty,
///     creation_mode: CreationMode::Standard,
///     teams,
///     collaborators: HashMap::new(),
///     actor_login: "bob".to_string(),
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

    /// Governs how the GitHub repository itself comes into existence
    /// (Standard, Fork, or Mirror). See [`CreationMode`] for details.
    pub creation_mode: CreationMode,

    /// GitHub teams to add to the repository at creation time.
    ///
    /// Maps team slug → desired access level.
    /// An empty map means no teams are explicitly assigned (organisation
    /// defaults may still apply via the configuration hierarchy).
    pub teams: HashMap<String, AccessLevel>,

    /// Individual GitHub users to add as collaborators at creation time.
    ///
    /// Maps GitHub username → desired access level.
    /// Use [`AccessLevel::None`] to explicitly remove a user if present.
    pub collaborators: HashMap<String, AccessLevel>,

    /// The identity (login) of the actor initiating this request.
    ///
    /// Used for:
    /// - Audit logging and event notifications (who triggered the creation)
    /// - Built-in template variables (`user_login`, `user_name`)
    ///
    /// For API requests this is the authenticated user's GitHub login if
    /// available, or the app installation slug as a fallback.
    /// For CLI requests this is `"reporoller-cli"`.
    pub actor_login: String,
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
    creation_mode: Option<CreationMode>,
    teams: HashMap<String, AccessLevel>,
    collaborators: HashMap<String, AccessLevel>,
    actor_login: Option<String>,
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
            creation_mode: None,
            teams: HashMap::new(),
            collaborators: HashMap::new(),
            actor_login: None,
        }
    }

    /// Set the identity (GitHub login) of the actor initiating this request.
    ///
    /// Used for audit logging, event notifications, and built-in template
    /// variables. Defaults to `"reporoller"` if not set.
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
    /// .actor("alice".to_string())
    /// .build();
    /// # Ok(())
    /// # }
    /// ```
    pub fn actor(mut self, login: String) -> Self {
        self.actor_login = Some(login);
        self
    }

    /// Set the teams to grant access to the new repository.
    ///
    /// Replaces any previously set teams map.
    ///
    /// # Examples
    ///
    /// ```
    /// # use repo_roller_core::*;
    /// # use repo_roller_core::permissions::AccessLevel;
    /// # use std::collections::HashMap;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut teams = HashMap::new();
    /// teams.insert("platform".to_string(), AccessLevel::Write);
    /// let request = RepositoryCreationRequestBuilder::new(
    ///     RepositoryName::new("my-repo")?,
    ///     OrganizationName::new("my-org")?,
    /// )
    /// .teams(teams)
    /// .build();
    /// # Ok(())
    /// # }
    /// ```
    pub fn teams(mut self, teams: HashMap<String, AccessLevel>) -> Self {
        self.teams = teams;
        self
    }

    /// Set the individual collaborators to grant access to the new repository.
    ///
    /// Replaces any previously set collaborators map.
    ///
    /// # Examples
    ///
    /// ```
    /// # use repo_roller_core::*;
    /// # use repo_roller_core::permissions::AccessLevel;
    /// # use std::collections::HashMap;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut collaborators = HashMap::new();
    /// collaborators.insert("alice".to_string(), AccessLevel::Write);
    /// let request = RepositoryCreationRequestBuilder::new(
    ///     RepositoryName::new("my-repo")?,
    ///     OrganizationName::new("my-org")?,
    /// )
    /// .collaborators(collaborators)
    /// .build();
    /// # Ok(())
    /// # }
    /// ```
    pub fn collaborators(mut self, collaborators: HashMap<String, AccessLevel>) -> Self {
        self.collaborators = collaborators;
        self
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

    /// Set how the GitHub repository itself comes into existence (Standard,
    /// Fork, or Mirror). Defaults to [`CreationMode::Standard`].
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
    /// .creation_mode(CreationMode::Fork {
    ///     source: SourceRepositoryReference {
    ///         owner: OrganizationName::new("upstream-org")?,
    ///         name: RepositoryName::new("upstream-repo")?,
    ///     },
    /// })
    /// .build();
    /// # Ok(())
    /// # }
    /// ```
    pub fn creation_mode(mut self, mode: CreationMode) -> Self {
        self.creation_mode = Some(mode);
        self
    }

    /// Build the final RepositoryCreationRequest.
    ///
    /// # Panics
    ///
    /// - Panics if `ContentStrategy::Template` is used without setting a
    ///   template, for [`CreationMode::Standard`] (the pre-existing rule).
    /// - Panics if an explicit, non-default `content_strategy` is set
    ///   together with [`CreationMode::Fork`] or [`CreationMode::Mirror`]:
    ///   content for those modes comes from GitHub/git (Fork, history
    ///   mirror) or is derived internally (source-only mirror), never from a
    ///   `ContentProvider`.
    pub fn build(self) -> RepositoryCreationRequest {
        let name = self.name.expect("name is required");
        let owner = self.owner.expect("owner is required");
        let creation_mode = self.creation_mode.unwrap_or_default();
        let is_fork_or_mirror = !matches!(creation_mode, CreationMode::Standard);

        if is_fork_or_mirror {
            if let Some(explicit_strategy) = &self.content_strategy {
                if !matches!(explicit_strategy, ContentStrategy::Template) {
                    panic!("content_strategy is not used with CreationMode::Fork/Mirror");
                }
            }
        } else if matches!(
            self.content_strategy.clone().unwrap_or_default(),
            ContentStrategy::Template
        ) && self.template.is_none()
        {
            // Validation: Template strategy requires template (Standard mode only)
            panic!(
                "ContentStrategy::Template requires template to be set. Use .template() method."
            );
        }

        let content_strategy = self.content_strategy.unwrap_or_default();

        RepositoryCreationRequest {
            name,
            owner,
            template: self.template,
            variables: self.variables.unwrap_or_default(),
            visibility: self.visibility,
            content_strategy,
            creation_mode,
            teams: self.teams,
            collaborators: self.collaborators,
            actor_login: self.actor_login.unwrap_or_else(|| "reporoller".to_string()),
        }
    }
}
