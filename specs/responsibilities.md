# Component Responsibilities

This document defines the responsibilities of each component using Responsibility-Driven Design (RDD) analysis. Each component is analyzed for what it **knows** (data/information), what it **does** (operations/behavior), and how it **collaborates** (relationships with other components).

## Business Logic Components

### RepositoryCreationOrchestrator

**Responsibilities:**

- **Knows**: Repository creation workflow stages, business rules for repository creation, validation requirements
- **Does**: Orchestrates complete repository creation process, coordinates between all other components, manages error handling and rollback procedures, maintains audit trail of operations

**Collaborators:**

- **AuthenticationService** (validates user permissions and context)
- **ConfigurationManager** (resolves hierarchical configuration)
- **TemplateProcessor** (processes template content)
- **GitHubRepository** (creates and configures repositories)
- **AuditLogger** (records all significant operations)

**Roles:**

- **Orchestrator**: Coordinates the entire repository creation workflow
- **Validator**: Ensures all business rules and constraints are met
- **Error Handler**: Manages failure scenarios and recovery procedures

### ConfigurationManager

**Responsibilities:**

- **Knows**: Configuration hierarchy rules, override policies, cache expiration strategies, organization metadata locations
- **Does**: Resolves configuration from multiple sources, validates override permissions, manages configuration cache, applies merge logic with precedence rules

**Collaborators:**

- **MetadataRepositoryProvider** (accesses organization configuration repositories)
- **ConfigurationCache** (stores and retrieves cached configurations)
- **OverridePolicyValidator** (validates permission to override settings)
- **ConfigurationMerger** (merges configurations according to hierarchy rules)

**Roles:**

- **Resolver**: Determines final configuration from hierarchical sources
- **Enforcer**: Enforces override policies and security constraints
- **Cache Manager**: Optimizes performance through intelligent caching

### TemplateProcessor

**Responsibilities:**

- **Knows**: Template syntax rules, variable substitution patterns, security constraints for file paths, processing timeouts
- **Does**: Processes templates using Handlebars engine, validates template syntax, performs variable substitution, ensures secure file path generation

**Collaborators:**

- **HandlebarsEngine** (performs template rendering and variable substitution)
- **SecurityValidator** (validates file paths and content for security)
- **VariableValidator** (validates template variables against constraints)
- **FileSystemHandler** (manages template file operations)

**Roles:**

- **Transformer**: Converts template content into repository-ready content
- **Security Guardian**: Prevents security violations during processing
- **Validator**: Ensures template and variable correctness

## Application Services

### AuthenticationService

**Responsibilities:**

- **Knows**: User authentication status, GitHub token information, organization memberships, permission hierarchies
- **Does**: Validates GitHub tokens, manages OAuth flows, checks user permissions, maintains authentication context

**Collaborators:**

- **GitHubClient** (validates tokens and retrieves user information)
- **PermissionResolver** (determines user permissions based on organization/team membership)
- **TokenManager** (manages token lifecycle and refresh)
- **SessionManager** (maintains user sessions for web interfaces)

**Roles:**

- **Authenticator**: Verifies user identity through GitHub
- **Authorizer**: Determines what operations users can perform
- **Session Manager**: Maintains authentication state across requests

### GitHubRepository

**Responsibilities:**

- **Knows**: GitHub API endpoints, rate limits, repository creation parameters, GitHub-specific constraints
- **Does**: Creates repositories via GitHub API, applies repository settings, manages repository permissions, handles API rate limiting

**Collaborators:**

- **GitHubClient** (low-level GitHub REST communication)
- **RateLimitManager** (manages API rate limits and retries)
- **RepositorySettingsApplier** (applies configuration to created repositories)
- **PermissionManager** (sets repository and team permissions)

**Roles:**

- **Repository Factory**: Creates new GitHub repositories
- **Configuration Applier**: Applies settings to repositories
- **API Gateway**: Interfaces with GitHub's REST API

### MetadataRepositoryProvider

**Responsibilities:**

- **Knows**: Metadata repository locations, repository structure conventions, configuration file formats
- **Does**: Discovers organization metadata repositories, loads configuration files, validates repository structure, caches repository access

**Collaborators:**

- **GitHubClient** (accesses metadata repository content)
- **ConfigurationFileParser** (parses TOML/YAML configuration files)
- **RepositoryStructureValidator** (validates metadata repository structure)
- **ContentCache** (caches frequently accessed configuration content)

**Roles:**

- **Discovery Agent**: Locates organization metadata repositories
- **Content Provider**: Loads and parses configuration content
- **Structure Validator**: Ensures metadata repositories follow conventions

## External System Integrations

### GitHubClient

**Responsibilities:**

- **Knows**: GitHub REST API specifications, authentication mechanisms, error codes, request/response formats
- **Does**: Sends HTTP requests to GitHub API, handles authentication headers, manages API responses, implements retry logic for transient failures

**Collaborators:**

- **HttpClient** (performs HTTP operations)
- **TokenProvider** (provides authentication tokens)
- **ErrorHandler** (processes API error responses)
- **RetryPolicy** (implements retry logic for failed requests)

**Roles:**

- **GitHub Client**: Low-level communication with GitHub's REST service
- **Protocol Handler**: Manages HTTP protocol specifics
- **Error Translator**: Converts GitHub service errors to business errors

### ConfigurationCache

**Responsibilities:**

- **Knows**: Cache keys and expiration times, cached configuration data, cache hit/miss statistics
- **Does**: Stores and retrieves cached configurations, manages cache expiration, invalidates stale entries, provides cache statistics

**Collaborators:**

- **CacheStorage** (persistent cache storage mechanism)
- **ExpirationManager** (manages TTL and expiration policies)
- **CacheMetrics** (tracks cache performance metrics)
- **InvalidationNotifier** (handles cache invalidation events)

**Roles:**

- **Performance Optimizer**: Reduces configuration loading time through caching
- **Data Store**: Temporarily stores frequently accessed configuration data
- **Invalidation Coordinator**: Ensures cache consistency with source data

### HandlebarsEngine

**Responsibilities:**

- **Knows**: Handlebars template syntax, registered helpers, compilation rules, rendering context
- **Does**: Compiles templates, renders templates with variables, executes template helpers, manages template compilation cache

**Collaborators:**

- **TemplateCompiler** (compiles Handlebars templates)
- **HelperRegistry** (manages custom template helpers)
- **RenderingContext** (provides variables and functions during rendering)
- **CompilationCache** (caches compiled templates for performance)

**Roles:**

- **Template Engine**: Processes templates and substitutes variables
- **Helper Executor**: Runs custom template helper functions
- **Compiler**: Converts template syntax into executable form

## User Interfaces

### RepositoryCreationWebService

**Responsibilities:**

- **Knows**: HTTP request/response formats, web service endpoint specifications, request validation rules
- **Does**: Handles HTTP requests for repository creation, validates request format, converts between HTTP and business objects, returns appropriate HTTP responses

**Collaborators:**

- **RepositoryCreationOrchestrator** (executes repository creation business logic)
- **RequestValidator** (validates incoming HTTP requests)
- **ResponseFormatter** (formats domain results as HTTP responses)
- **ErrorHandler** (converts domain errors to HTTP error responses)

**Roles:**

- **Web Service**: Entry point for HTTP-based repository creation requests
- **Protocol Handler**: Converts between HTTP and business representations
- **Request Processor**: Processes incoming web requests

### CommandLineInterface

**Responsibilities:**

- **Knows**: Command-line argument specifications, output formatting options, user interaction patterns
- **Does**: Parses command-line arguments, validates CLI input, formats output for terminal display, manages interactive prompts

**Collaborators:**

- **RepositoryCreationOrchestrator** (executes repository creation business logic)
- **ArgumentParser** (parses command-line arguments)
- **OutputFormatter** (formats results for terminal display)
- **UserInteractionHandler** (manages prompts and user input)

**Roles:**

- **Command Processor**: Interprets and executes command-line commands
- **User Interface**: Provides terminal-based interaction for repository creation
- **Input Validator**: Validates command-line arguments and options

### BrowserInterface

**Responsibilities:**

- **Knows**: Web application routing, session management, user interface state, form validation rules
- **Does**: Renders interactive web pages, handles user input through forms, manages user sessions, provides real-time feedback on repository creation progress

**Collaborators:**

- **RepositoryCreationOrchestrator** (executes repository creation business logic)
- **AuthenticationService** (manages user login and session state)
- **TemplateMetadataProvider** (loads available templates for selection)
- **ConfigurationManager** (provides organization and team options)
- **WebSocketHandler** (provides real-time progress updates)

**Roles:**

- **User Interface**: Provides web-based interaction for repository creation
- **Session Manager**: Maintains user authentication state across web requests
- **Progress Reporter**: Displays real-time status during repository creation operations
- **Form Handler**: Validates and processes user input from web forms

### ModelContextProtocolServer

**Responsibilities:**

- **Knows**: Model Context Protocol specifications, tool definitions, message formats
- **Does**: Implements MCP server protocol, exposes repository creation as MCP tools, handles MCP message exchange, provides structured responses

**Collaborators:**

- **RepositoryCreationOrchestrator** (executes repository creation business logic)
- **McpProtocolHandler** (manages MCP protocol compliance)
- **ToolDefinitionProvider** (defines available MCP tools)
- **MessageSerializer** (handles MCP message serialization)

**Roles:**

- **Protocol Server**: Implements Model Context Protocol for AI/LLM integration
- **Tool Provider**: Exposes repository creation capabilities as structured tools
- **Message Handler**: Processes MCP messages and provides appropriate responses

## Collaboration Patterns

### Repository Creation Workflow

1. **Request Reception**: Interface layer (API/CLI/MCP) receives repository creation request
2. **Authentication**: AuthenticationService validates user credentials and permissions
3. **Configuration Resolution**: ConfigurationManager resolves hierarchical configuration
4. **Template Processing**: TemplateProcessor transforms template content with variables
5. **Repository Creation**: GitHubRepository creates repository via GitHub API
6. **Configuration Application**: GitHubRepository applies resolved configuration to repository
7. **Result Reporting**: User interface returns success/failure response to user

### Configuration Resolution Workflow

1. **Context Preparation**: ConfigurationManager receives organization, team, template information
2. **Cache Check**: ConfigurationCache checked for existing resolved configuration
3. **Metadata Discovery**: MetadataRepositoryProvider discovers organization metadata repository
4. **Configuration Loading**: Multiple configuration sources loaded in parallel
5. **Hierarchy Resolution**: ConfigurationMerger applies precedence rules and override policies
6. **Validation**: Final configuration validated for completeness and security
7. **Cache Update**: Resolved configuration cached for future use

### Error Handling Collaboration

1. **Error Detection**: Any component detects error condition during operation
2. **Error Classification**: Component classifies error type (validation, processing, system, configuration)
3. **Context Preservation**: Error wrapped with relevant context information
4. **Error Propagation**: Error passed up through collaboration chain
5. **Error Translation**: Interface layer translates domain errors to appropriate format
6. **Recovery Assessment**: System determines if operation can be retried or requires user intervention

## Boundary Definitions

### Business Logic Boundary

**Inside Business Logic:**

- RepositoryCreationOrchestrator
- ConfigurationManager
- TemplateProcessor
- Core business value objects and entities

**Outside Business Logic:**

- All external system integrations (GitHubClient, ConfigurationCache, etc.)
- All user interface components (Web Service, CLI, MCP Server)
- External services (GitHub REST service, Azure services)

### Application Service Boundary

**Authentication Services:**

- User authentication and token management
- Permission resolution and authorization
- Session management for web interfaces

**Configuration Services:**

- Hierarchical configuration resolution
- Override policy enforcement
- Cache management and invalidation

**Template Services:**

- Template processing and variable substitution
- Security validation for template content
- File path validation and sanitization

### External System Boundary

**GitHub Integration:**

- GitHub REST service communication
- Repository and organization management
- User authentication through GitHub

**Azure Integration:**

- Azure service integration
- Credential storage and management
- Monitoring and logging services

**System Resources:**

- File system operations
- Network communication
- Environment variable access

This responsibility analysis ensures clear separation of concerns, well-defined collaboration patterns, and maintainable component boundaries. Each component has a single, well-defined purpose and collaborates with others through explicit interfaces rather than tight coupling.
