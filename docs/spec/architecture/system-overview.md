# System Overview

## High-Level Architecture

RepoRoller follows a modular, multi-tier architecture designed for flexibility, maintainability, and multiple deployment scenarios. The system separates concerns into distinct layers while providing multiple user interfaces for different interaction patterns.

```mermaid
graph TD
    subgraph "User Interfaces"
        CLI[CLI Tool<br/>repo_roller_cli]
        Web[Web UI<br/>SvelteKit]
        API[REST API<br/>repo_roller_api]
        MCP[MCP Server<br/>repo_roller_mcp]
    end

    subgraph "Backend Core (Rust Crates)"
        Core[repo_roller_core<br/>Business Logic]
        GitHub[github_client<br/>GitHub API Integration]
        Template[template_engine<br/>Template Processing]
        Config[config_manager<br/>Configuration Management]
        Auth[auth_handler<br/>Authentication & Authorization]
    end

    subgraph "External Services"
        GitHubAPI[GitHub API<br/>Repository Management]
        CloudServices[Cloud Services<br/>Secrets, Monitoring]
        TemplateRepos[Template Repositories<br/>Source Templates]
    end

    CLI --> Core
    Web --> API
    API --> Core
    MCP --> Core

    Core --> GitHub
    Core --> Template
    Core --> Config
    Core --> Auth

    GitHub --> GitHubAPI
    Template --> TemplateRepos
    Auth --> GitHubAPI
    API --> CloudServices

    style Core fill:#f9f,stroke:#333,stroke-width:2px
```

## System Organization

### User Interfaces

Multiple interfaces support different user needs and integration scenarios:

**CLI Tool (`repo_roller_cli`)**

- Command-line interface for developers and automation
- Direct interaction with core business logic
- Ideal for scripting and CI/CD integration

**Web UI (SvelteKit)**

- Interactive web interface for guided repository creation
- Connects to REST API for backend operations
- Provides user-friendly forms and real-time feedback

**REST API (`repo_roller_api`)**

- HTTP-based programmatic interface
- Enables integration with external tools and services
- Supports both direct binary execution and containerized deployment

**MCP Server (`repo_roller_mcp`)**

- Model Context Protocol interface for AI/LLM workflows
- Exposes repository creation as structured tools
- Enables AI agents to create repositories autonomously

### Business Logic Components

The core components contain all business logic and orchestration:

**Core Business Logic (`repo_roller_core`)**

- Central orchestration of repository creation workflow
- Coordinates interactions between all other components
- Defines primary data structures and interfaces
- Implements error handling and result reporting

### Application Services

Specialized services handle specific technical concerns:

**GitHub Client (`github_client`)**

- Abstracts all GitHub API interactions
- Handles authentication token management
- Provides typed interfaces for GitHub operations
- Manages rate limiting and error recovery

**Template Engine (`template_engine`)**

- Processes template repositories and variable substitution
- Handles file system operations and content transformation
- Supports advanced templating features (future: Handlebars)
- Manages template caching and optimization

**Configuration Manager (`config_manager`)**

- Loads and validates application configuration
- Manages template definitions and repository settings
- Handles environment-specific configuration
- Provides schema validation and defaults

**Authentication Handler (`auth_handler`)**

- Manages user authentication flows
- Implements role-based authorization
- Handles GitHub OAuth and token validation
- Provides session management for web interfaces

### External Service Integrations

The system integrates with several external services:

**GitHub API**

- Primary integration point for repository management
- Handles repository creation, configuration, and content
- Provides authentication and permission services

**Cloud Platform Services (Optional)**

- **Secret Managers**: Secure credential storage (Azure Key Vault, AWS Secrets Manager, etc.)
- **Monitoring Backends**: Logging, metrics, and alerting integrations
- **Container Runtimes**: Managed hosting for API containers

**Template Repositories**

- Source repositories containing template content
- Accessed via GitHub API for content retrieval
- Support for public and private template repositories

## Data Flow Architecture

### Request Processing Flow

1. **Request Initiation**
   - User submits repository creation request via any interface
   - Request includes repository name, template type, and variables

2. **Authentication & Authorization**
   - User credentials validated through appropriate mechanism
   - Permissions checked for target organization and repository

3. **Configuration Resolution**
   - Template configuration loaded from config management
   - Repository settings and policies retrieved
   - Variable defaults and validation rules applied

4. **Template Processing**
   - Template repository content retrieved and cached
   - Variable substitution performed on files and metadata
   - Content prepared for repository creation

5. **Repository Creation**
   - New repository created via GitHub API
   - Processed content pushed as initial commit
   - Repository settings and policies applied

6. **Result Reporting**
   - Success/failure status returned to user
   - Detailed logs and metrics recorded
   - Audit trail updated for compliance

### Error Handling Flow

The system implements comprehensive error handling at multiple levels:

**Request Level**

- Input validation and sanitization
- Authentication and authorization failures
- Rate limiting and quota management

**Processing Level**

- Template processing errors
- GitHub API failures and retries
- Configuration validation errors

**System Level**

- Service unavailability handling
- Resource exhaustion recovery
- Monitoring and alerting integration

## Deployment Architectures

### Development Deployment

```mermaid
graph LR
    Dev[Developer] --> CLI[Local CLI]
    Dev --> LocalAPI[Local API Server]
    LocalAPI --> GitHub[GitHub API]
    CLI --> GitHub
```

### Production Deployment

```mermaid
graph TD
    Users[End Users] --> LB[Load Balancer]
    LB --> WebApp[Static Web App]
    LB --> APIM[API Management]
    APIM --> APIContainers[RepoRoller API Containers]
    APIContainers --> GitHub[GitHub API]
    APIContainers --> SecretStore[Secret Manager]
    APIContainers --> Monitor[Monitoring Backend]
```

## Scalability Considerations

### Horizontal Scaling

- **Stateless Design**: All components designed for stateless operation
- **Container Scaling**: Container platforms scale API replicas based on demand
- **API Gateway**: API Management provides load balancing and rate limiting

### Performance Optimization

- **Caching**: Template and configuration caching to reduce processing time
- **Async Processing**: Non-blocking operations for improved throughput
- **Resource Pooling**: Efficient resource utilization in containerized environments

### Rate Limit Management

- **GitHub API Limits**: Intelligent request batching and retry logic
- **User Rate Limits**: Per-user and per-organization request limiting
- **Monitoring**: Real-time monitoring of API usage and limits

## Security Architecture

### Authentication Flow

```mermaid
sequenceDiagram
    participant User
    participant WebUI
    participant API
    participant GitHub
    participant SecretStore

    User->>WebUI: Initiate login
    WebUI->>GitHub: OAuth authorization
    GitHub->>WebUI: Authorization code
    WebUI->>API: Exchange code for token
    API->>GitHub: Validate token
    GitHub->>API: User info + permissions
    API->>SecretStore: Store encrypted token
    API->>WebUI: Session token
```

### Security Boundaries

- **Input Validation**: All user inputs validated and sanitized
- **Token Management**: Secure storage and rotation of GitHub tokens
- **Network Security**: HTTPS enforcement and network isolation
- **Audit Logging**: Comprehensive audit trail for all operations

This architecture provides a solid foundation for a scalable, maintainable, and secure repository automation system while supporting multiple deployment scenarios and user interaction patterns.
