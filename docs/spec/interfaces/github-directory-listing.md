# GitHub Directory Listing Interface

**Architectural Layer**: Infrastructure (GitHub API)
**Crate**: `github_client`
**Module**: `lib.rs`
**Responsibilities**:

- **Knows**: GitHub Contents API structure, directory vs file distinction
- **Does**: Lists repository directory contents, filters by entry type, handles pagination

**Dependencies**: Octocrab GitHub API client

---

## Overview

This interface provides directory listing capabilities using the GitHub Contents API. It enables discovery of directory structures in GitHub repositories, essential for automatically discovering repository types from metadata repository structure.

**Primary Use Case**: Discover available repository types by listing the `types/` directory in metadata repositories.

---

## Interface: list_directory_contents

### Method Signature

```rust
/// List contents of a directory in a GitHub repository.
///
/// Uses the GitHub Contents API to retrieve directory listings. Automatically
/// handles pagination for directories with more than 1000 entries.
///
/// # Arguments
///
/// * `owner` - Repository owner (organization or user name)
/// * `repo` - Repository name
/// * `path` - Directory path to list (relative to repository root)
/// * `branch` - Branch/ref to query (e.g., "main", "master", "develop")
///
/// # Returns
///
/// `Vec<TreeEntry>` - Vector of directory entries with type information
///
/// # Errors
///
/// * `Error::NotFound` - Path doesn't exist in repository
/// * `Error::InvalidResponse` - Path is a file, not a directory
/// * `Error::AuthError` - Authentication failure or insufficient permissions
/// * `Error::RateLimitExceeded` - GitHub API rate limit exceeded
/// * `Error::ApiError` - Other GitHub API errors
///
/// # Examples
///
/// ```rust
/// # use github_client::{GitHubClient, create_app_client};
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// #     let app_id = 123456;
/// #     let private_key = "-----BEGIN RSA PRIVATE KEY-----\n...\n-----END RSA PRIVATE KEY-----";
/// #     let octocrab_client = create_app_client(app_id, private_key).await?;
/// #     let client = GitHubClient::new(octocrab_client);
///
/// // List repository types from metadata repository
/// let entries = client
///     .list_directory_contents("my-org", ".reporoller-test", "types", "main")
///     .await?;
///
/// // Filter to only directories
/// let types: Vec<String> = entries
///     .iter()
///     .filter(|e| matches!(e.entry_type, EntryType::Dir))
///     .map(|e| e.name.clone())
///     .collect();
///
/// println!("Found repository types: {:?}", types);
/// #     Ok(())
/// # }
/// ```
///
/// # GitHub API Details
///
/// - Endpoint: `GET /repos/{owner}/{repo}/contents/{path}?ref={branch}`
/// - Response: Array of content objects for directories, single object for files
/// - Pagination: Uses Link header for large directories (>1000 entries)
/// - Rate Limiting: Counts against authenticated rate limit (5000/hour)
#[async_trait]
pub trait GitHubClient: Send + Sync {
    async fn list_directory_contents(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
        branch: &str,
    ) -> Result<Vec<TreeEntry>, Error>;
}
```

---

## Types

### TreeEntry

Represents a single entry in a directory listing.

```rust
/// A single entry in a GitHub repository directory listing.
///
/// Represents files, directories, symlinks, and submodules returned by
/// the GitHub Contents API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeEntry {
    /// Entry name (e.g., "library", "config.toml")
    pub name: String,

    /// Full path within repository (e.g., "types/library", "types/service/config.toml")
    pub path: String,

    /// Entry type (file, directory, symlink, submodule)
    pub entry_type: EntryType,

    /// Git SHA of the entry
    pub sha: String,

    /// Size in bytes (0 for directories)
    pub size: u64,

    /// Download URL for files (None for directories)
    pub download_url: Option<String>,
}
```

**Field Descriptions**:

- `name`: Basename of the entry (no path separators)
- `path`: Full repository-relative path
- `entry_type`: Discriminates between files, directories, etc.
- `sha`: Git object SHA for content integrity
- `size`: File size in bytes (always 0 for directories)
- `download_url`: Direct download URL for files (useful for fetching content)

### EntryType

Discriminates between different types of repository entries.

```rust
/// Type of entry in a repository directory.
///
/// Maps to GitHub's content type field in the Contents API response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryType {
    /// Regular file
    File,

    /// Directory (can contain other entries)
    Dir,

    /// Symbolic link
    Symlink,

    /// Git submodule reference
    Submodule,
}
```

**Type Mapping**:

- GitHub API returns string: `"file"`, `"dir"`, `"symlink"`, `"submodule"`
- Serde automatically deserializes to enum variants
- Unknown types should return `Error::InvalidResponse`

---

## Error Conditions

### Path Not Found

**Condition**: `path` doesn't exist in repository at specified branch

**Error**: `Error::NotFound`

**HTTP Status**: 404 Not Found

**Example**:

```rust
let result = client
    .list_directory_contents("org", "repo", "nonexistent", "main")
    .await;

assert!(matches!(result, Err(Error::NotFound)));
```

### Path Is File, Not Directory

**Condition**: `path` points to a file, not a directory

**Error**: `Error::InvalidResponse`

**HTTP Status**: 200 OK (but response is object, not array)

**Detection**: GitHub returns single object for files, array for directories

**Example**:

```rust
// README.md is a file, not a directory
let result = client
    .list_directory_contents("org", "repo", "README.md", "main")
    .await;

assert!(matches!(result, Err(Error::InvalidResponse)));
```

### Empty Directory

**Condition**: Directory exists but contains no files

**Error**: None (success)

**Result**: `Ok(vec![])`

**Example**:

```rust
let entries = client
    .list_directory_contents("org", "repo", "empty-dir", "main")
    .await?;

assert_eq!(entries.len(), 0);
```

### Branch Doesn't Exist

**Condition**: `branch` parameter specifies non-existent ref

**Error**: `Error::NotFound`

**HTTP Status**: 404 Not Found

### Insufficient Permissions

**Condition**: App lacks read access to repository

**Error**: `Error::AuthError`

**HTTP Status**: 403 Forbidden

### Rate Limit Exceeded

**Condition**: Too many API calls in time window

**Error**: `Error::RateLimitExceeded`

**HTTP Status**: 403 Forbidden (with rate limit headers)

**Recovery**: Wait until `X-RateLimit-Reset` timestamp, then retry

---

## Implementation Requirements

### Octocrab Integration

Use Octocrab's repos endpoint:

```rust
// Pseudo-code for implementation
let response = self.client
    .repos(owner, repo)
    .get_content()
    .path(path)
    .r#ref(branch)
    .send()
    .await?;

// Parse response
// - Array response → directory (multiple entries)
// - Object response → file (single entry, not a directory)
```

### Pagination Handling

GitHub API limits responses to 1000 entries per page for large directories.

**Implementation**:

1. Check response headers for `Link` header
2. If present, parse pagination URLs
3. Fetch additional pages until exhausted
4. Concatenate all results into single vector

**Note**: Most metadata repositories have <100 entries per directory, so pagination is rare but must be supported.

### Error Mapping

Map Octocrab errors to `github_client::Error`:

| Octocrab Error                | GitHub HTTP | Mapped Error               |
| ----------------------------- | ----------- | -------------------------- |
| `StatusCode(404)`             | 404         | `Error::NotFound`          |
| `StatusCode(403)` + rate info | 403         | `Error::RateLimitExceeded` |
| `StatusCode(403)` normal      | 403         | `Error::AuthError`         |
| `StatusCode(401)`             | 401         | `Error::AuthError`         |
| Network/timeout errors        | N/A         | `Error::ApiError`          |
| Deserialization errors        | N/A         | `Error::Deserialization`   |

### Response Validation

1. Check if response is array or object
2. If object (file response), return `Error::InvalidResponse`
3. If array, parse each entry into `TreeEntry`
4. Validate all entries have required fields
5. Filter out malformed entries with warning logs

### Logging

Include comprehensive logging:

```rust
info!(
    owner = %owner,
    repo = %repo,
    path = %path,
    branch = %branch,
    "Listing directory contents"
);

debug!(
    entry_count = entries.len(),
    "Retrieved directory entries"
);

// Log entry types breakdown
let file_count = entries.iter().filter(|e| matches!(e.entry_type, EntryType::File)).count();
let dir_count = entries.iter().filter(|e| matches!(e.entry_type, EntryType::Dir)).count();
debug!(
    files = file_count,
    directories = dir_count,
    "Entry type breakdown"
);
```

---

## Integration with GitHubMetadataProvider

This interface enables automatic repository type discovery:

```rust
// In config_manager crate
impl GitHubMetadataProvider {
    async fn list_available_repository_types(
        &self,
        repo: &MetadataRepository,
    ) -> ConfigurationResult<Vec<String>> {
        // Use new directory listing capability
        let entries = self.client
            .list_directory_contents(&repo.owner, &repo.name, "types", &repo.branch)
            .await
            .map_err(|e| ConfigurationError::MetadataFetchFailed {
                message: format!("Failed to list types directory: {}", e)
            })?;

        // Filter to only directories (ignore files in types/)
        let types: Vec<String> = entries
            .into_iter()
            .filter(|e| matches!(e.entry_type, EntryType::Dir))
            .map(|e| e.name)
            .collect();

        if types.is_empty() {
            warn!(
                repo = %repo.name,
                "No repository types found in types/ directory"
            );
        }

        Ok(types)
    }
}
```

---

## Testing Requirements

### Unit Tests (with wiremock)

Create mock GitHub API responses:

1. **Directory with multiple entries**
   - Mock: `GET /repos/org/repo/contents/types?ref=main` → 200 + JSON array
   - Verify: Returns correct TreeEntry objects
   - Verify: Preserves entry types correctly

2. **Empty directory**
   - Mock: `GET /repos/org/repo/contents/empty?ref=main` → 200 + empty array `[]`
   - Verify: Returns `Ok(vec![])`

3. **Non-existent directory**
   - Mock: `GET /repos/org/repo/contents/missing?ref=main` → 404
   - Verify: Returns `Error::NotFound`

4. **Path is a file**
   - Mock: `GET /repos/org/repo/contents/README.md?ref=main` → 200 + JSON object (not array)
   - Verify: Returns `Error::InvalidResponse`

5. **Mix of files and directories**
   - Mock: Response with both file and dir entries
   - Verify: Both types correctly parsed
   - Verify: Filtering works in consumer code

6. **Pagination (large directory)**
   - Mock: Response with `Link` header for next page
   - Mock: Second page response
   - Verify: All entries from both pages returned
   - Verify: Pagination handled transparently

7. **Authentication error**
   - Mock: `GET /repos/org/repo/contents/path?ref=main` → 401 Unauthorized
   - Verify: Returns `Error::AuthError`

8. **Rate limit exceeded**
   - Mock: `GET /repos/org/repo/contents/path?ref=main` → 403 + rate limit headers
   - Verify: Returns `Error::RateLimitExceeded`

### Integration Tests (real GitHub)

Test against `glitchgrove/.reporoller-test` repository:

1. **List types directory**
   - Path: `types`
   - Branch: `main`
   - Expected: Returns `["library", "service", ...]` (actual types in repo)
   - Verify: Only directories returned, no files

2. **List empty directory** (create test directory if needed)
   - Path: `empty-test-dir`
   - Expected: Returns empty vector

3. **List non-existent directory**
   - Path: `does-not-exist-xyz`
   - Expected: Returns `Error::NotFound`

4. **Different branches**
   - List same directory on different branches
   - Verify: Returns correct content for each branch

---

## Performance Considerations

### Caching Strategy

Directory contents are relatively static:

- **Cache entries**: Yes (with TTL of 5-10 minutes)
- **Cache key**: `(owner, repo, path, branch, timestamp)`
- **Invalidation**: TTL expiration or manual cache clear

### API Rate Limiting

- Each directory listing costs 1 API call
- Pagination adds 1 call per additional page
- Consider caching to reduce API calls
- Use conditional requests (If-None-Match) when possible

### Typical Performance

- Small directories (<100 entries): 200-500ms
- Large directories (>1000 entries): 500ms-2s (pagination)
- Network latency dominates (API round-trip time)

---

## Security Considerations

### Path Traversal Prevention

**Threat**: Malicious input attempts path traversal (e.g., `../../secrets`)

**Mitigation**: GitHub API validates paths server-side

**Defense in Depth**: Validate paths don't contain suspicious patterns:

```rust
// In calling code (config_manager)
if repo_type.contains("..") || repo_type.contains('/') || repo_type.contains('\\') {
    return Err(ConfigurationError::InvalidConfiguration {
        field: "repo_type",
        reason: "Contains invalid characters",
    });
}
```

### Private Repository Access

- Requires installation token with repository read permissions
- Error: `Error::AuthError` if permissions insufficient
- Ensure GitHub App has `contents: read` permission

### Sensitive Path Disclosure

- Be cautious logging full directory contents
- May reveal repository structure to logs
- Redact sensitive directory names if needed

---

## Open Questions

None - specification is complete.

---

## Success Criteria

✅ **Interface Complete** when:

1. `list_directory_contents()` method added to `GitHubClient`
2. `TreeEntry` and `EntryType` types defined with proper serialization
3. All error conditions documented and mapped correctly
4. Stub implementation compiles successfully
5. Interface specification reviewed and approved

✅ **Implementation Complete** when:

1. Octocrab integration working correctly
2. Pagination handled for large directories
3. All error conditions handled with proper error types
4. Comprehensive unit tests pass (wiremock)
5. Integration tests pass (real GitHub API)
6. Logging provides sufficient debugging context
7. `GitHubMetadataProvider` successfully uses new capability

---

## References

- **GitHub API Docs**: [Get repository content](https://docs.github.com/en/rest/repos/contents#get-repository-content)
- **Architecture**: [specs/architecture/system-overview.md](../architecture/system-overview.md)
- **Error Types**: [specs/interfaces/error-types.md](error-types.md)
- **Configuration Provider**: [specs/interfaces/configuration-interfaces.md](configuration-interfaces.md)
- **Design Doc**: [specs/design/organization-repository-settings.md](../design/organization-repository-settings.md)
