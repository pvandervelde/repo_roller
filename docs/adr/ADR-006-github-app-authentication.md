# ADR-006: GitHub App Authentication

Status: Accepted
Date: 2026-01-31
Owners: RepoRoller team

## Context

RepoRoller requires authentication with GitHub to create repositories and access organization settings. Authentication affects security, user experience, auditability, and operational complexity. The system must support both interactive use (web UI, CLI) and automated use (CI/CD, Azure Functions).

Requirements:

- Secure credential management (no plaintext tokens)
- Organization-scoped permissions (not tied to individual users)
- Fine-grained permissions (minimal access required)
- Automatic token rotation (no manual renewal)
- Clear audit trail (attribute actions to app, not individuals)
- Support for both interactive and automated workflows

Security constraints:

- Tokens must not be logged or displayed
- Tokens should expire automatically
- Permissions should follow principle of least privilege
- Organizations must control app installation and permissions

## Decision

Use **GitHub App with Installation Tokens** as primary authentication method:

**Authentication Flow:**

1. GitHub App created with required permissions (repo creation, settings)
2. App installed to organization (organization controls permissions)
3. Application uses App ID + Private Key to generate JWT
4. JWT exchanged for Installation Token scoped to organization
5. Installation Token used for GitHub API operations
6. Tokens automatically expire (typically 1 hour) and refreshed as needed

**Token Types:**

- **App Private Key**: Long-lived credential (stored in Azure Key Vault for cloud, system keyring for CLI)
- **JWT Token**: Short-lived (10 minutes), used only to request installation token
- **Installation Token**: Short-lived (1 hour), used for actual GitHub API calls

**Token Storage:**

- Cloud deployment: Azure Key Vault
- CLI: System keyring (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- Never in configuration files or environment variables

**Secondary method (CLI only):**
Personal Access Token (PAT) supported for users who cannot install GitHub App.

## Consequences

**Enables:**

- Organization controls app permissions through installation settings
- Automatic token rotation (no manual renewal)
- Fine-grained permissions (only what's needed for repository creation)
- Clear audit trail (actions attributed to app installation)
- Works for both interactive and automated scenarios
- Supports multiple organizations with separate installations
- Secure token storage (Key Vault, keyring)

**Forbids:**

- Storing tokens in configuration files
- Logging token values (even debug logs)
- Long-lived tokens without rotation
- User-scoped permissions only (PAT-only approach)

**Trade-offs:**

- More complex setup (GitHub App creation and installation required)
- Need to handle both app authentication and token management
- JWT generation adds complexity
- Fallback to PAT adds code paths

## Alternatives considered

### Option A: Personal Access Token (PAT) only

**Why not**:

- Token management burden on users (manual creation, renewal, revocation)
- Tied to individual user accounts (permissions follow user)
- Security risk (long-lived tokens, no automatic rotation)
- Poor audit trail (actions attributed to user, not app)
- No organization-level control

### Option B: OAuth App with web flow

**Why not**:

- Requires web redirect flow (incompatible with CLI automation)
- Limited to user permissions (cannot exceed user's access)
- Tokens tied to user (actions attributed to user)
- No organization-level permission management
- Requires interactive authentication

### Option C: Personal Access Token with automatic rotation

**Why not**:

- GitHub doesn't support PAT auto-rotation
- Would require custom token management service
- Still tied to individual users
- No organization control

### Option D: Multiple authentication methods without GitHub App

**Why not**:

- More code paths to maintain
- Inconsistent behavior between methods
- No single "best practice" to recommend
- Harder to test and secure

## Implementation notes

**GitHub App Setup:**

```
Required Permissions:
- Repository: Read & Write (create repos, configure settings)
- Organization: Read (list repos, get org settings)
- Metadata: Read (basic repo info)

Installation:
- Installed per organization
- Organization admin controls which repos accessible
- Can restrict to specific repositories or allow all
```

**Authentication Flow (Cloud):**

```rust
// 1. Load app credentials from Azure Key Vault
let app_id = key_vault.get_secret("github-app-id").await?;
let private_key = key_vault.get_secret("github-app-private-key").await?;

// 2. Create app client (generates JWT)
let app_client = create_app_client(app_id, &private_key).await?;

// 3. Get installation token for organization
let installation_token = app_client
    .get_installation_token_for_org("acme-corp")
    .await?;

// 4. Create API client with installation token
let github_client = GitHubClient::new(installation_token);
```

**Authentication Flow (CLI):**

```rust
// 1. Load app credentials from system keyring
let keyring = Entry::new("repo_roller_cli", "github_app")?;
let credentials: AppCredentials = keyring.get_password()?;

// 2-4. Same as cloud
```

**Token Security:**

```rust
// GitHubToken wraps secrecy::Secret to prevent logging
#[derive(Clone)]
pub struct GitHubToken(secrecy::Secret<String>);

// Never derive Debug (would expose token)
impl std::fmt::Debug for GitHubToken {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "GitHubToken(***)")
    }
}

// Explicit access required
impl GitHubToken {
    pub fn expose_secret(&self) -> &str {
        use secrecy::ExposeSecret;
        self.0.expose_secret()
    }
}
```

**Token Caching:**
Installation tokens cached with expiration tracking:

```rust
struct TokenCache {
    token: GitHubToken,
    expires_at: Instant,
}

impl TokenCache {
    async fn get_valid_token(&mut self) -> Result<&GitHubToken> {
        if Instant::now() > self.expires_at {
            // Token expired, refresh
            self.token = self.fetch_new_token().await?;
            self.expires_at = Instant::now() + Duration::from_secs(3600);
        }
        Ok(&self.token)
    }
}
```

## Examples

**GitHub App authentication (primary method):**

```rust
use github_client::{create_app_client, GitHubClient};

// App credentials from secure storage
let app_id = 12345;
let private_key = load_private_key_from_keyring()?;

// Create authenticated client
let app_client = create_app_client(app_id, &private_key).await?;
let token = app_client
    .get_installation_token_for_org("my-org")
    .await?;

let client = GitHubClient::new_with_token(token);

// Use client for operations
let repo = client
    .create_repository("my-repo", "my-org")
    .await?;
```

**PAT authentication (fallback for CLI):**

```rust
use github_client::GitHubClient;

// PAT from user input (CLI only)
let token = prompt_for_token()?;  // User enters PAT
let client = GitHubClient::new_with_token(token);

// Same API as GitHub App authentication
let repo = client
    .create_repository("my-repo", "my-org")
    .await?;
```

**Token storage (CLI):**

```bash
# User configures GitHub App credentials
$ repo-roller auth github app
App ID: 12345
Private Key path: ~/.ssh/github-app.pem
✓ Credentials stored securely in system keyring

# Credentials retrieved automatically on subsequent use
$ repo-roller create repo my-repo --org my-org --template rust-lib
✓ Using GitHub App authentication
✓ Repository created: https://github.com/my-org/my-repo
```

**Token security (no logging):**

```rust
let token = GitHubToken::new("ghs_secrettoken123");

// Debug output safe
println!("{:?}", token);  // Prints: GitHubToken(***)

// Logging safe
tracing::info!(?token, "Authenticated");  // Logs: GitHubToken(***)

// Explicit access when needed
let token_str = token.expose_secret();  // Use with caution
```

## References

- [Authentication Requirements](../spec/tradeoffs.md#authentication-and-authorization)
- [Security Design Goals](../spec/overview/design-goals.md#security-decisions)
- [GitHub Token Type](../spec/interfaces/shared-types.md#githubtoken)
- GitHub App Documentation: <https://docs.github.com/en/apps/creating-github-apps/about-creating-github-apps/about-creating-github-apps>
- Installation Tokens: <https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/generating-an-installation-access-token-for-a-github-app>
