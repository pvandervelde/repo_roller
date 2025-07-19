# Component Details

## Core Components Overview

RepoRoller is structured as a Rust workspace with multiple focused crates, each handling specific aspects of the repository creation and management process. This modular design promotes code reuse, testability, and maintainability.

## Backend Crate Structure

### `repo_roller_core` - Business Logic Orchestration

**Primary Responsibilities:**

- Orchestrates the complete repository creation workflow
- Defines core data structures and interfaces
- Coordinates interactions between all other components
- Implements business rules and validation logic
- Handles error aggregation and result reporting

**Key Interfaces:**

```rust
// Primary entry point for repository creation
pub async fn create_repository(request: CreateRepositoryRequest) -> Result<RepositoryCreationResult>

// Core data structures
pub struct CreateRepositoryRequest {
    pub name: String,
    pub template_type: String,
    pub owner: String,
    pub variables: HashMap<String, String>,
    pub auth_context: AuthContext,
}

pub struct RepositoryCreationResult {
    pub repository_url: String,
    pub created_at: DateTime<Utc>,
    pub applied_settings: Vec<AppliedSetting>,
    pub warnings: Vec<String>,
}
```

**Dependencies:**

- `github_client` for GitHub API operations
- `template_engine` for content processing
- `config_manager` for configuration loading
- `auth_handler` for authentication validation

### `github_client` - GitHub API Integration

**Primary Responsibilities:**

- Abstracts all GitHub REST API interactions
- Manages GitHub App and OAuth token lifecycle
- Provides typed interfaces for GitHub operations
- Handles rate limiting and retry logic
- Implements error recovery and fallback strategies

**Key Interfaces:**

```rust
// Repository management
pub async fn create_repository(&self, org: &str, repo: CreateRepoRequest) -> Result<Repository>
pub async fn update_repository_settings(&self, owner: &str, repo: &str, settings: RepoSettings) -> Result<()>
pub async fn push_files(&self, owner: &str, repo: &str, files: Vec<FileContent>) -> Result<()>

// Configuration management
pub async fn create_labels(&self, owner: &str, repo: &str, labels: Vec<Label>) -> Result<()>
pub async fn set_branch_protection(&self, owner: &str, repo: &str, rules: BranchProtection) -> Result<()>
pub async fn set_repository_permissions(&self, owner: &str, repo: &str, permissions: TeamPermissions) -> Result<()>
```

**External Dependencies:**

- `reqwest` for HTTP client functionality
- `serde` for JSON serialization/deserialization
- `tokio` for async runtime support

### `template_engine` - Template Processing

**Primary Responsibilities:**

- Retrieves template content from source repositories
- Performs advanced variable substitution using Handlebars templating engine
- Supports file and directory name templating
- Handles file system operations and content transformation
- Manages template compilation caching for performance
- Provides custom helpers for repository-specific transformations

**Key Interfaces:**

```rust
// Template processing workflow
pub async fn process_template(&self, template_config: TemplateConfig, variables: HashMap<String, String>) -> Result<ProcessedTemplate>

// Handlebars-based template engine
pub struct HandlebarsTemplateEngine {
    handlebars: handlebars::Handlebars<'static>,
}

pub struct TemplateContext {
    variables: HashMap<String, serde_json::Value>,
    helpers: Vec<Box<dyn handlebars::HelperDef + Send + Sync>>,
}

// Template content management
pub struct ProcessedTemplate {
    pub files: Vec<ProcessedFile>,
    pub directories: Vec<String>,
    pub metadata: TemplateMetadata,
}

pub struct ProcessedFile {
    pub path: String,
    pub content: String,
    pub is_binary: bool,
    pub permissions: Option<u32>,
}
```

**Processing Pipeline:**

1. **Template Retrieval**: Clone or download template repository content
2. **Context Preparation**: Build Handlebars context from user variables and register custom helpers
3. **Content Discovery**: Scan template for files requiring processing, identify binary files
4. **Path Templating**: Process directory and file names through Handlebars engine
5. **Content Templating**: Apply variable substitution, conditionals, loops, and custom helpers
6. **Security Validation**: Ensure file paths are safe and within target bounds
7. **Result Packaging**: Prepare processed content for repository creation

**Templating Features:**

- **Variable Substitution**: `{{variable_name}}`, `{{object.property}}`
- **Control Structures**: `{{#if condition}}`, `{{#each items}}`, `{{#with object}}`
- **Custom Helpers**: `{{snake_case text}}`, `{{kebab_case text}}`, `{{timestamp format}}`
- **File Path Templating**: Template-driven directory and file name generation
- **Security**: Path traversal prevention, resource limits, sandboxed helpers

### `config_manager` - Configuration Management

**Primary Responsibilities:**

- Loads and validates application configuration from files
- Manages template definitions and repository type configurations
- Provides environment-specific configuration support
- Handles configuration schema validation and default values
- Supports dynamic configuration reloading

**Key Interfaces:**

```rust
// Configuration loading
pub fn load_config() -> Result<AppConfig>
pub fn get_template_config(&self, template_type: &str) -> Result<TemplateConfig>
pub fn get_repository_settings(&self, template_type: &str) -> Result<RepositorySettings>

// Configuration structures
pub struct TemplateConfig {
    pub source_repository: String,
    pub branch: Option<String>,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub variables: HashMap<String, VariableDefinition>,
}

pub struct RepositorySettings {
    pub default_branch: String,
    pub features: RepositoryFeatures,
    pub branch_protection: BranchProtectionConfig,
    pub labels: Vec<LabelDefinition>,
    pub team_permissions: Vec<TeamPermission>,
}
```

**Configuration Sources:**

- **Primary Config**: Main application configuration file (TOML/YAML)
- **Template Definitions**: Individual template configuration files
- **Environment Variables**: Runtime configuration overrides
- **Azure Key Vault**: Sensitive configuration values (production)

### `auth_handler` - Authentication & Authorization

**Primary Responsibilities:**

- Manages GitHub OAuth authentication flows
- Implements role-based access control (RBAC)
- Handles GitHub App token management and refresh
- Validates user permissions for repository operations
- Provides session management for web interfaces

**Key Interfaces:**

```rust
// Authentication flows
pub async fn initiate_github_oauth(&self, redirect_uri: &str) -> Result<OAuthInitiation>
pub async fn complete_github_oauth(&self, code: &str, state: &str) -> Result<AuthenticatedUser>
pub async fn validate_token(&self, token: &str) -> Result<TokenValidation>

// Authorization checks
pub async fn check_repository_permission(&self, user: &AuthenticatedUser, org: &str, action: Action) -> Result<bool>
pub async fn get_user_permissions(&self, user: &AuthenticatedUser) -> Result<UserPermissions>

// GitHub App integration
pub async fn get_installation_token(&self, installation_id: u64) -> Result<InstallationToken>
```

**Security Features:**

- **Token Encryption**: All stored tokens encrypted at rest
- **Session Management**: Secure session handling with automatic expiration
- **Permission Caching**: Efficient permission lookup with TTL-based caching
- **Audit Logging**: Comprehensive audit trail for all authentication events

## Frontend Components

### User Interface Modules (CLI)

**Command Structure:**

```bash
repo-roller create --name my-repo --template library --owner my-org
repo-roller list-templates
repo-roller validate-config
repo-roller auth login
```

**Core CLI Components:**

- **Command Parser**: Argument parsing and validation using `clap`
- **Configuration Handler**: CLI-specific configuration management
- **Output Formatter**: Human-readable output formatting
- **Error Reporter**: User-friendly error message display

### Web Interface Modules (SvelteKit)

**Page Structure:**

- **Authentication Pages**: Login, OAuth callback handling
- **Repository Creation**: Template selection, variable input, progress tracking
- **Dashboard**: Recent repositories, template usage statistics
- **Administration**: Template management, user permissions

**Component Architecture:**

- **Shared Components**: Reusable UI components (forms, buttons, modals)
- **Page Components**: Route-specific page layouts and logic
- **API Client**: Type-safe API communication layer
- **State Management**: Application state and user session management

### API Components

**Endpoint Structure:**

```
POST /api/v1/repositories          # Create repository
GET  /api/v1/templates             # List available templates
GET  /api/v1/templates/{type}      # Get template details
POST /api/v1/auth/github           # GitHub OAuth initiation
GET  /api/v1/auth/callback         # OAuth callback handling
GET  /api/v1/user/permissions      # User permission information
```

**Middleware Stack:**

- **Authentication Middleware**: Token validation and user context
- **Rate Limiting**: Request rate limiting per user/organization
- **Request Logging**: Comprehensive request/response logging
- **Error Handling**: Standardized error response formatting

## Deployment Components

### Azure Function Adapter

**Function Bindings:**

- **HTTP Trigger**: Handles incoming HTTP requests
- **Key Vault Integration**: Retrieves secrets at runtime
- **Application Insights**: Telemetry and logging integration

**Adapter Responsibilities:**

- **Request Translation**: Convert Azure Function context to standard HTTP requests
- **Response Formatting**: Format responses for Azure Function runtime
- **Cold Start Optimization**: Minimize initialization overhead
- **Configuration Management**: Azure-specific configuration handling

### Infrastructure Components

**Terraform Modules:**

- **Core Infrastructure**: Resource groups, storage accounts, networking
- **Compute Resources**: Azure Functions, App Service Plans
- **Security Resources**: Key Vault, managed identities, access policies
- **Monitoring Resources**: Application Insights, Log Analytics, alerts

**Deployment Pipeline:**

- **Build Stage**: Compile Rust binaries, build frontend assets
- **Test Stage**: Run unit tests, integration tests, security scans
- **Infrastructure Stage**: Apply Terraform configurations
- **Deployment Stage**: Deploy applications to Azure resources

This component structure provides clear separation of concerns while enabling flexible deployment and testing scenarios. Each component has well-defined interfaces and responsibilities, making the system maintainable and extensible.
