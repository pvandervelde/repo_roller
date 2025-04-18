# RepoRoller General Specification

## 1. Problem Description

Developers and teams often need to create new GitHub repositories based on standardized templates.
Manually copying templates, replacing placeholders, applying settings, configuring labels, and setting
permissions is time-consuming, error-prone, and inconsistent. RepoRoller aims to automate this
process, ensuring new repositories adhere to organizational standards from the start.

## 2. Surrounding Context

This tool will operate within a GitHub environment, interacting with the GitHub API to manage
repositories. It needs to handle various repository types (e.g., libraries, services, actions)
with potentially different templates and settings. Users will interact via a CLI (for testing/dev),
a web interface, a REST API, or an MCP server. The backend logic will be implemented in Rust and
deployable as an Azure Function or a CLI tool.

## 3. Proposed Solution

RepoRoller will be a GitHub App or use a PAT/GitHub App token to interact with the GitHub API. It
will provide interfaces (CLI, Web, REST, MCP) for users to request new repository creation based on
predefined templates.

The core logic will:

1. Receive a request specifying the desired repository name, template type, owner, and any
    template-specific variables.
2. Authenticate and authorize the request.
3. Select the appropriate template repository.
4. Clone the template content.
5. Perform variable substitution (e.g., replacing `{{repo_name}}` with the actual name).
6. Create the new repository on GitHub.
7. Push the processed template files as the initial commit to the `main` or `master` branch.
8. Apply standard repository settings (branch protection, etc.) based on the template type.
9. Create standard issue labels.
10. Configure team/user permissions.
11. Report success or failure back to the user.

### 3.1. Design Goals

- **Automation:** Fully automate the creation and initial setup of standard repositories.
- **Consistency:** Ensure all new repositories adhere to predefined standards.
- **Flexibility:** Support multiple repository templates and types.
- **Extensibility:** Allow easy addition of new templates and configuration options.
- **Usability:** Provide multiple, convenient interfaces for users.
- **Security:** Securely handle GitHub credentials and API interactions.

### 3.2. Design Constraints

- Relies on the GitHub API and its rate limits.
- Requires appropriate permissions (via GitHub App or PAT) to manage repositories.
- Initial setup requires defining templates and configurations.

### 3.3. Design Decisions

- **Backend Language:** Rust (for performance, safety, and suitability for CLI/WASM/Cloud Functions).
- **Deployment:** Azure Functions (production), CLI (testing/dev).
- **Frontend:** SvelteKit (ease of use, maintainability).
- **Authentication:** GitHub OAuth primary, with potential for others. RBAC for authorization.
- **Template Engine:** A simple substitution mechanism initially (e.g., Handlebars or similar).

### 3.4. Alternatives Considered

- **Other Backend Languages:** Go (similar benefits to Rust), Python/Node.js (simpler but potentially
    less performant/safe for complex logic).
- **Other Frontend Frameworks:** React/Vue (larger ecosystems but potentially steeper learning curve).
- **Configuration Management Tools:** Terraform/Pulumi (powerful but might be overkill for just
    repository setup, could be integrated later for infrastructure).

## 4. Design

### 4.1. High-Level Architecture

```mermaid
graph TD
    subgraph User Interfaces
        CLI[CLI (Rust)]
        Web[Web UI (SvelteKit)]
        API[REST API (Rust)]
        AzureFnAdapter[Azure Function Adapter (Rust)]
        MCP[MCP Server (Rust)]
    end

    subgraph Backend Core (Rust Crates)
        Core[repo_roller_core]
        GitHub[github_client]
        Templating[template_engine]
        Config[config_manager]
        Auth[auth_handler]
    end

    subgraph External Services
        GitHubAPI[GitHub API]
        AzureFunc[Azure Functions Host]
        TemplateRepos[Template Repositories]
    end

    CLI --> Core
    Web --> API
    API --> Core
    AzureFnAdapter --> API # Azure Function hosts the API
    MCP --> Core

    Core --> GitHub
    Core --> Templating
    Core --> Config
    Core --> Auth

    GitHub --> GitHubAPI
    Templating --> TemplateRepos
    AzureFnAdapter -- Deployed To --> AzureFunc
    Auth --> GitHubAPI

    style Core fill:#f9f,stroke:#333,stroke-width:2px
    style AzureFnAdapter fill:#ccf,stroke:#333,stroke-width:2px
```

### 4.2. Proposed Module Breakdown (Rust Crates)

The backend logic will be structured as a Rust workspace with several crates:

1. **`repo_roller_core`**:
    - **Responsibility:** Orchestrates the entire repository creation process. Contains the main
        business logic, coordinating interactions between other modules. Defines core data structures
        (Request, Config, Template, etc.).
    - **Interfaces:** Provides the primary functions called by the CLI, API, and MCP server.

2. **`github_client`**:
    - **Responsibility:** Handles all interactions with the GitHub REST API. Abstracts away `reqwest`
        and JSON serialization/deserialization for GitHub objects. Manages authentication tokens.
    - **Interfaces:** Functions for creating repositories, getting/setting settings, managing labels,
        permissions, pushing files, etc.

3. **`template_engine`**:
    - **Responsibility:** Fetches template content (e.g., cloning a repo), performs variable
        substitution using a chosen templating library (like Handlebars or Tera).
    - **Interfaces:** Function to process a directory of template files given a context map.

4. **`config_manager`**:
    - **Responsibility:** Loads and manages application configuration, including template definitions,
        standard settings per repository type, label definitions, and permission rules. Reads from
        configuration files (e.g., TOML, YAML).
    - **Interfaces:** Functions to retrieve template details, settings, labels, etc.

5. **`auth_handler`**:
    - **Responsibility:** Manages authentication (e.g., GitHub OAuth flow for web/API) and
        authorization (checking if a user/token has permission to perform actions).
    - **Interfaces:** Functions for initiating OAuth flows, validating tokens, checking permissions.

6. **`repo_roller_cli`**:
    - **Responsibility:** Provides the command-line interface. Parses arguments, interacts with
        `repo_roller_core`, and displays output to the user.
    - **Interfaces:** Main executable.

7. **`repo_roller_api`**:
    - **Responsibility:** Implements the REST API endpoints. Likely built using `axum` or `actix-web`.
        Handles HTTP requests/responses, calls `repo_roller_core`. Designed to be deployable as an
        Azure Function.
    - **Interfaces:** HTTP endpoints.

8. **`repo_roller_mcp`**:
    - **Responsibility:** Implements the MCP server, exposing core functionality as MCP tools/resources.
    - **Interfaces:** MCP tool handlers.

9. **`repo_roller_azure_fn`**:
    - **Responsibility:** Acts as the adapter layer specifically for Azure Functions. It translates
        Azure Function triggers (HTTP requests) into calls to the `repo_roller_api` crate (or potentially
        `repo_roller_core` directly if the API crate is thin). Handles Azure-specific context and response
        formatting.
    - **Interfaces:** Azure Function entry points.

### 4.3. Frontend (SvelteKit)

- A separate project/directory `frontend`.
- Will interact with the `repo_roller_api`.
- Components for login (GitHub OAuth), requesting repository creation (form), viewing request status.

### 4.4. Data Flow (Simplified Example: CLI Request)

1. User runs `repo-roller create --name my-new-repo --template library --owner my-org`.
2. `repo_roller_cli` parses args.
3. `repo_roller_cli` calls `repo_roller_core::create_repository(...)`.
4. `repo_roller_core` uses `config_manager` to load config for the `library` template.
5. `repo_roller_core` uses `auth_handler` to verify the user/token has permission (if applicable).
6. `repo_roller_core` uses `template_engine` to fetch and process the template files.
7. `repo_roller_core` uses `github_client` to:
    - Create the repository `my-org/my-new-repo`.
    - Push processed files.
    - Apply settings, labels, permissions from the loaded config.
8. `repo_roller_core` returns result to `repo_roller_cli`.
9. `repo_roller_cli` displays success/error message.

### 4.5. Deployment Infrastructure (Azure)

- **Compute:** Azure Functions (Consumption Plan or Premium Plan depending on performance/scaling needs).
- **API Management:** Optional: Azure API Management to act as a gateway, providing security
    (key/JWT validation), rate limiting, caching, and a unified frontend for APIs.
- **Storage:** Azure Blob Storage might be needed for temporary storage during template processing or
    for storing configuration/state if not managed elsewhere.
- **Secrets Management:** Azure Key Vault for securely storing GitHub tokens, API keys, and other
    secrets.
- **Networking:** Configure network security groups (NSGs) and potentially private endpoints if
    integrating with other Azure services within a VNet.
- **Infrastructure as Code (IaC):** Terraform will be used to define and manage the Azure infrastructure
    (Functions, API Management, Key Vault, Storage, Networking). Terraform configurations will be stored
    within this repository or a dedicated infrastructure repository.
- **CI/CD:** GitHub Actions to build, test, and deploy the Rust code (CLI and Azure Function package)
    and the SvelteKit frontend. The CI/CD pipeline will also include steps to run `terraform plan` and
    `terraform apply` for infrastructure changes. Deployment to Azure will use Azure CLI or specific
    GitHub Actions for Azure, orchestrated via the Terraform apply step.

### 4.6. Observability

- **Logging:** Implement structured logging (e.g., using the `tracing` crate in Rust). Logs from Azure
    Functions will be ingested into Azure Monitor Logs (Log Analytics Workspace). Frontend logs can be
    sent to Azure Application Insights or another logging service.
- **Metrics:** Azure Functions provide standard execution metrics. Custom application metrics (e.g.,
    repository creation time, template usage counts) can be emitted using `tracing` or a dedicated
    metrics library and collected in Azure Monitor Metrics or Application Insights.
- **Tracing:** Implement distributed tracing using `tracing` and potentially OpenTelemetry. This helps
    track requests across different components (API -> Core -> GitHub Client). Traces can be sent to
    Azure Application Insights or another compatible backend (Jaeger, Zipkin).
- **Alerting:** Configure alerts in Azure Monitor based on logs (e.g., error rates) or metrics (e.g.,
    high function execution duration, queue lengths).

### 4.7. Security

- **Authentication:**
  - Web UI/API: GitHub OAuth mandatory. Implement robust OAuth flow handling. Consider state
        parameters and PKCE for added security.
  - API (Direct): Support API Keys or JWTs issued after authentication, managed via Azure API
        Management or application logic.
  - GitHub Interaction: Use short-lived GitHub App installation tokens preferably, or securely
        managed PATs stored in Azure Key Vault. Avoid hardcoding credentials.
- **Authorization:** Implement Role-Based Access Control (RBAC). Define roles (e.g., User, Admin,
    TemplateManager) and associate permissions. Verify permissions in `auth_handler` before executing
    sensitive operations.
- **Input Validation:** Rigorously validate all inputs from users (API requests, CLI args) to prevent
    injection attacks or unexpected behavior. Use libraries like `serde` for validation.
- **Secrets Management:** All secrets (API keys, GitHub tokens, database credentials if any) must be
    stored securely in Azure Key Vault and accessed at runtime, not embedded in code or configuration files.
- **Infrastructure Security:**
  - Secure Azure Function endpoints (e.g., using Function Keys, integrating with API Management,
        using network restrictions).
  - Regularly update dependencies (Rust crates, frontend libraries, OS) to patch vulnerabilities
        (use `cargo audit`, `npm audit`).
  - Apply principle of least privilege for Azure resources and GitHub App permissions.
- **Rate Limiting:** Implement rate limiting at the API gateway (APIM) or application level to prevent
    abuse.
- **HTTPS:** Ensure all communication is over HTTPS. Azure Functions and APIM handle this by default.

### 4.8. Managed Repository Settings & Features

RepoRoller, via the `github_client` and driven by `config_manager`, will be responsible for configuring
the following aspects of newly created repositories, based on template type and organizational defaults:

- **Basic Information:**
  - Repository Name (provided in request)
  - Repository Description/About
  - Homepage URL (optional)
- **Topics:** Assign relevant topics, potentially including standard topics based on repository type.
- **Feature Toggles:** Enable or disable:
  - Issues
  - Projects
  - Discussions
  - Wiki
  - *(Note: These settings might be standardized across an organization)*
- **Pull Request Settings:**
  - Allowed Merge Commit Types (Merge, Squash, Rebase)
  - Default Merge Commit Message Format
  - Automatic Deletion of Head Branches after merge
- **Branch/Tag Protection (via Rulesets):**
  - Define and apply GitHub Rulesets to enforce branch protection rules, required status checks,
        signed commits, etc.
  - Limit who can push to matching branches/tags.
- **Action Permissions:** Configure permissions for GitHub Actions (e.g., allowed actions, workflow
    permissions).
- **Custom Action Usage Permissions:** For repositories identified as custom actions, configure which
    other repositories or organizations can use them.
- **Push Limits:** Limit the number of branches and tags that can be updated in a single push.
- **Custom Properties:** Define and set organization-level custom properties on the repository.
- **Environments:** Create predefined deployment environments (e.g., `staging`, `production`). Note:
    RepoRoller will *not* manage secret values for these environments.
- **Labels:** Create a standard set of issue labels with defined names and colors.
- **Discussion Categories:** If discussions are enabled, create standard discussion categories.
- **Repository Templates:** Add standard templates to the repository:
  - Issue Templates (`.github/ISSUE_TEMPLATE/`)
  - Pull Request Templates (`.github/PULL_REQUEST_TEMPLATE.md`)
- **LLM Configuration Files:** Add standard AI/LLM configuration files like `.clinerules` or
    `.github/copilot-instructions.md`.

Configuration for these settings will be defined within the `config_manager`'s scope, likely loaded
from TOML or YAML files, allowing different settings profiles per template type.

## 5. Conclusion

This specification outlines a robust system for automating GitHub repository creation and setup. The
modular Rust backend allows for flexibility in deployment (CLI, Azure Function) and interaction
(Web UI, REST API, MCP). The next steps involve creating detailed specifications for each module and
then implementing them, starting with the core logic and GitHub client.
