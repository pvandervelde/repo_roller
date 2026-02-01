# ADR-003: Stateless Architecture with External Storage

Status: Accepted
Date: 2026-01-31
Owners: RepoRoller team

## Context

RepoRoller needs to store configuration, templates, and operational data. The system must support multiple deployment models including serverless (Azure Functions), containerized API servers, and CLI tools. We need to decide whether to maintain application state in a database or rely entirely on external systems (GitHub) for persistence.

Requirements:

- Support serverless deployment (Azure Functions with cold starts)
- Enable horizontal scaling without state synchronization
- Maintain audit trail of repository creation activities
- Store configuration in version-controlled, auditable format
- Minimize operational complexity and infrastructure dependencies

Constraints:

- Configuration data is naturally stored in GitHub repositories
- Created repositories are tracked in GitHub
- No complex relational queries needed
- Read-heavy workload with infrequent writes

## Decision

Implement **stateless architecture** with all persistent data stored in external systems:

- **Configuration Storage**: GitHub repositories (`.reporoller/` metadata repos)
- **Template Storage**: GitHub repositories (template repos)
- **Created Repository Tracking**: GitHub repository metadata and topics
- **Application State**: None - all requests are independent
- **Caching**: In-memory only, no persistent cache

All application logic is stateless. Each request fetches required data from GitHub, processes it, and creates repositories without maintaining local state.

## Consequences

**Enables:**

- Simple serverless deployment (no database connection pooling or state management)
- Effortless horizontal scaling (no shared state to coordinate)
- No database infrastructure to maintain (no backups, monitoring, scaling)
- Configuration changes immediately visible through GitHub
- Natural audit trail through Git history
- Simplified disaster recovery (no database to restore)

**Forbids:**

- Complex queries across repository metadata
- Local caching persistence between deployments
- Application-managed audit logs (rely on GitHub audit instead)
- Long-running transactions with rollback

**Trade-offs:**

- Dependent on GitHub API availability (cannot operate during GitHub outages)
- Higher latency for operations requiring multiple GitHub API calls
- Cannot perform analytical queries across repositories
- Limited control over data retention and backup policies
- GitHub API rate limits become a concern

## Alternatives considered

### Option A: Embedded database (SQLite)

**Why not**:

- Breaks serverless model (persistent file system required)
- Complicates deployment (where to store SQLite file?)
- State management complexity in scaled environments
- Backup and recovery procedures needed
- Adds no real value given GitHub already stores all needed data

### Option B: External database (PostgreSQL/SQL Server)

**Why not**:

- Significant infrastructure overhead (database server, connection pooling, monitoring)
- Additional cost for database hosting
- Over-engineered for simple configuration storage
- Duplicates data already in GitHub
- Backup and disaster recovery complexity
- No compelling queries that require relational database

### Option C: Redis/Memcached for persistent cache

**Why not**:

- Adds infrastructure complexity
- Cache invalidation becomes distributed problem
- Not needed for serverless cold-start scenario
- In-memory cache with TTL sufficient for hot instances
- Additional cost and monitoring burden

### Option D: Hybrid (stateless with optional database)

**Why not**:

- Increases complexity significantly
- Two code paths to maintain (with/without database)
- Unclear which operations require database
- Violates YAGNI principle

## Implementation notes

**Data storage patterns:**

1. **Configuration data**:
   - Stored in `.reporoller/` metadata repositories
   - Read via GitHub API
   - Cached in-memory with 5-minute TTL
   - Version controlled with Git history

2. **Template definitions**:
   - Stored in template repositories
   - Discovered via GitHub search (topic tags)
   - Content fetched on-demand
   - No local storage

3. **Created repository tracking**:
   - GitHub repository metadata (creation timestamp, creator)
   - Repository topics include `reporoller-created`
   - Can query via GitHub API/GraphQL

4. **Audit trail**:
   - GitHub audit log for API operations
   - Application structured logging for business events
   - No local audit log storage

**Caching strategy:**

- In-memory caches with TTL (5 minutes for config, 1 hour for environment detection)
- Caches per-instance only (no shared cache)
- Cache invalidation via webhook or manual API call
- Cold-start penalty acceptable for serverless

**GitHub API dependency management:**

- Implement retry logic with exponential backoff
- Graceful degradation where possible
- Clear error messages on GitHub outage
- Rate limit handling (5000 requests/hour authenticated)

**Operational considerations:**

- Monitor GitHub API usage against rate limits
- Alert on approaching rate limit thresholds
- No backup procedures needed (GitHub provides backups)
- No database maintenance windows

## Examples

**Configuration loading (no local storage):**

```rust
pub async fn get_organization_config(
    &self,
    org: &OrganizationName
) -> Result<OrganizationConfig, ConfigError> {
    // Check in-memory cache first
    if let Some(cached) = self.cache.get(org) {
        return Ok(cached.clone());
    }

    // Fetch from GitHub (external storage)
    let content = self.github
        .get_file_content(org, ".reporoller", "defaults.toml")
        .await?;

    let config: OrganizationConfig = toml::from_str(&content)?;

    // Cache in-memory only
    self.cache.insert(org.clone(), config.clone());

    Ok(config)
}
```

**Repository creation (stateless operation):**

```rust
pub async fn create_repository(
    &self,
    request: CreateRepositoryRequest,
) -> Result<Repository, RepoRollerError> {
    // No database transaction - just orchestrate GitHub API calls

    // 1. Fetch config from GitHub
    let config = self.config_provider.resolve_configuration(&request).await?;

    // 2. Fetch template from GitHub
    let template = self.template_engine.load_template(&request.template).await?;

    // 3. Process template
    let content = self.template_engine.process(template, &config).await?;

    // 4. Create repository via GitHub API
    let repo = self.github.create_repository(&request.name, content).await?;

    // No local state to save - GitHub now owns this repository
    Ok(repo)
}
```

**Finding created repositories (query GitHub, not local DB):**

```rust
pub async fn list_created_repositories(
    &self,
    org: &OrganizationName,
) -> Result<Vec<Repository>, GitHubError> {
    // Query GitHub directly using topics
    let repos = self.github
        .search_repositories()
        .org(org)
        .topic("reporoller-created")
        .sort("created")
        .desc()
        .execute()
        .await?;

    Ok(repos)
}
```

## References

- [Architectural Tradeoffs](../spec/tradeoffs.md#data-storage-and-persistence)
- [System Overview](../spec/architecture/system-overview.md)
- [Deployment Constraints](../spec/constraints.md#deployment-constraints)
