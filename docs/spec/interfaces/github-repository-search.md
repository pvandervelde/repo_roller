# GitHub Repository Search Interface

**Architectural Layer**: External System Integration
**Module Path**: `crates/github_client/src/lib.rs`
**Responsibilities** (from RDD):

- **Knows**: GitHub search API syntax, repository metadata structure, search result pagination
- **Does**: Searches repositories by topic within organization, constructs search queries, parses search results

## Dependencies

- **Types**: `Repository` ([github-client/src/models.rs](../../crates/github_client/src/models.rs))
- **Errors**: `Error` ([github-client/src/errors.rs](../../crates/github_client/src/errors.rs))
- **External**: Octocrab GitHub API client

## Type Definitions

### RepositorySearchResult

Represents a repository found via GitHub search API.

```rust
/// Repository information returned from GitHub search operations.
///
/// This type extends the basic `Repository` model with search-specific metadata
/// that may be included in search results.
///
/// See docs/spec/interfaces/github-repository-search.md
pub type RepositorySearchResult = models::Repository;
```

**Note**: We reuse the existing `models::Repository` type which already contains all necessary fields:

- `name`: Repository name
- `full_name`: Full repository name with owner (e.g., "org/repo")
- `owner`: Repository owner information
- `html_url`: Repository URL
- `description`: Repository description
- `topics`: Repository topics (used for topic-based discovery)
- `created_at`: Creation timestamp
- `updated_at`: Last update timestamp

**Rationale**: The existing `Repository` model already captures all information needed for search results. Creating a separate type would introduce unnecessary duplication.

## Interface Methods

### search_repositories_by_topic

#### Signature

```rust
/// Searches for repositories in an organization that have a specific topic.
///
/// This method constructs a GitHub search query to find repositories within
/// the specified organization that are tagged with the given topic. It uses
/// GitHub's repository search API with query syntax: `org:{org} topic:{topic}`
///
/// # Arguments
///
/// * `org` - The organization name to search within
/// * `topic` - The topic tag to search for
///
/// # Returns
///
/// A vector of `Repository` objects matching the search criteria.
/// Returns an empty vector if no repositories match.
///
/// # Errors
///
/// * `Error::ApiError` - GitHub API request failed
/// * `Error::InvalidResponse` - Search response could not be parsed
///
/// # Behavior
///
/// 1. Constructs search query: `org:{org} topic:{topic}`
/// 2. Calls GitHub search API via octocrab
/// 3. Parses search results into `Repository` objects
/// 4. Returns all matching repositories (handles pagination automatically)
///
/// # GitHub API Rate Limits
///
/// This method counts against GitHub's search API rate limits:
/// - Authenticated: 30 requests per minute
/// - Unauthenticated: 10 requests per minute
///
/// # Example Usage
///
/// ```rust,no_run
/// use github_client::GitHubClient;
///
/// # async fn example(client: &GitHubClient) -> Result<(), github_client::Error> {
/// // Find all repositories in "my-org" with topic "reporoller-metadata"
/// let repos = client.search_repositories_by_topic("my-org", "reporoller-metadata").await?;
///
/// if repos.is_empty() {
///     println!("No repositories found with this topic");
/// } else {
///     for repo in repos {
///         println!("Found: {} - {}", repo.name, repo.html_url);
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Integration with MetadataRepositoryProvider
///
/// This method is used by `GitHubMetadataProvider::discover_by_topic()` to
/// implement topic-based metadata repository discovery. The metadata provider
/// validates that exactly one repository is found and returns appropriate
/// errors for 0 or multiple matches.
///
/// See docs/spec/interfaces/github-repository-search.md for full contract
pub async fn search_repositories_by_topic(
    &self,
    org: &str,
    topic: &str,
) -> Result<Vec<models::Repository>, Error> {
    unimplemented!("See docs/spec/interfaces/github-repository-search.md")
}
```

## Error Conditions

### API Errors (`Error::ApiError`)

Returned when:

- GitHub API is unreachable or returns HTTP error
- Authentication token is invalid or expired
- Rate limit exceeded
- Network connectivity issues
- GitHub service is unavailable

### Invalid Response (`Error::InvalidResponse`)

Returned when:

- Search response cannot be parsed
- Response structure doesn't match expected format
- Response contains invalid data

## Search Query Syntax

The method constructs GitHub search queries using the following syntax:

```
org:{organization} topic:{topic}
```

**Examples**:

- `org:glitchgrove topic:reporoller-metadata` - Find metadata repos in glitchgrove org
- `org:my-company topic:template` - Find template repos in my-company org

**GitHub Search API Documentation**: <https://docs.github.com/en/rest/search/search#search-repositories>

## Empty Results Handling

When no repositories match the search criteria:

- Method returns `Ok(Vec::new())` (empty vector)
- This is **not an error condition** at the GitHub client level
- The calling code (e.g., `MetadataRepositoryProvider`) determines if empty results are an error for the business operation

**Rationale**: The GitHub API successfully executed the search; finding zero results is a valid outcome. Business logic determines if zero results constitute a problem.

## Multiple Results Handling

When multiple repositories match:

- Method returns `Ok(Vec<Repository>)` with all matches
- No filtering or deduplication at this level
- Business logic determines if multiple results are acceptable

**For metadata repository discovery**:

- `MetadataRepositoryProvider::discover_by_topic()` validates exactly one match
- Returns error if zero or multiple repositories found
- Error includes list of found repositories for debugging

## Performance Considerations

### Caching Strategy

The `search_repositories_by_topic` method itself does **not** implement caching. Caching is handled at higher levels:

- **MetadataRepositoryProvider** caches discovered metadata repositories
- **ConfigurationManager** caches loaded configurations
- This separation allows flexible caching policies per use case

### Pagination

GitHub's search API returns paginated results:

- Octocrab client handles pagination automatically
- All matching repositories are returned in a single vector
- For large organizations with many matches, consider query refinement

### Rate Limit Management

GitHub enforces strict rate limits on search API:

- Monitor rate limit headers in responses
- Implement exponential backoff on rate limit errors
- Consider caching search results to reduce API calls

**Future Enhancement**: Add rate limit tracking and automatic retry logic.

## Testing Requirements

### Unit Tests (with wiremock)

Required test scenarios:

1. **Successful search with single result**
   - Mock GitHub search endpoint returning one repository
   - Verify query construction: `org:test-org topic:test-topic`
   - Verify result parsing

2. **Successful search with multiple results**
   - Mock endpoint returning 3 repositories
   - Verify all results returned in correct order

3. **Successful search with zero results**
   - Mock endpoint returning empty items array
   - Verify returns empty vector (not an error)

4. **API error handling (403 Forbidden)**
   - Mock 403 response (insufficient permissions)
   - Verify returns `Error::ApiError`

5. **API error handling (404 Not Found)**
   - Mock 404 response (organization doesn't exist)
   - Verify returns `Error::ApiError`

6. **API error handling (422 Unprocessable Entity)**
   - Mock 422 response (invalid query syntax)
   - Verify returns `Error::ApiError`

7. **Invalid response format**
   - Mock response with missing required fields
   - Verify returns `Error::InvalidResponse`

8. **Network connectivity failure**
   - Simulate connection timeout
   - Verify returns `Error::ApiError`

### Integration Tests (with real GitHub)

Required test scenarios using `glitchgrove` organization:

1. **Find metadata repository**
   - Search for topic `reporoller-metadata` in `glitchgrove`
   - Verify finds `.reporoller-test` repository
   - Verify repository has correct topic

2. **Search for non-existent topic**
   - Search for topic `nonexistent-topic-12345` in `glitchgrove`
   - Verify returns empty vector (no error)

3. **Search in non-existent organization**
   - Search for topic in organization that doesn't exist
   - Verify returns appropriate error

4. **Search with multiple results**
   - Search for common topic (e.g., `template`) in `glitchgrove`
   - Verify returns multiple repositories
   - Verify all have the specified topic

### Contract Tests

Verify GitHub API contract assumptions:

1. **Response structure matches `Repository` model**
2. **Topics field is always present (may be empty)**
3. **Required fields are never null**

## Security Considerations

### Input Validation

- Organization names must be valid GitHub organization identifiers
- Topic names must follow GitHub topic naming rules (lowercase, hyphens, alphanumeric)
- No special characters that could break query syntax

**Recommendation**: Add input validation in future enhancement to prevent malformed queries.

### Authentication

- All searches require authenticated GitHub client
- Searches respect organization visibility (private repos only visible if token has access)
- Topics on private repositories only returned if token has read access

### Information Disclosure

- Search results may reveal repository names and topics
- Ensure logging doesn't expose sensitive repository information
- Sanitize error messages to prevent information leakage

## Implementation Notes

### Query Construction

The search query is constructed as:

```rust
let query = format!("org:{} topic:{}", org, topic);
self.search_repositories(&query).await
```

This leverages the existing `search_repositories` method which handles:

- API request execution
- Response parsing
- Error handling
- Result mapping to `models::Repository`

### Relationship to Existing Methods

- **`search_repositories(query: &str)`** - Generic search (already exists)
- **`search_repositories_by_topic(org: &str, topic: &str)`** - Specialized topic search (new)

The new method provides a convenient, type-safe interface for the common use case of finding repositories by topic within an organization.

### Future Enhancements

Potential improvements for future iterations:

1. **Additional search methods**:
   - `search_repositories_by_language(org: &str, language: &str)`
   - `search_repositories_by_name_pattern(org: &str, pattern: &str)`

2. **Search options**:
   - Sort order (stars, updated, created)
   - Result limit
   - Include archived repositories flag

3. **Rate limit handling**:
   - Automatic retry with exponential backoff
   - Rate limit status tracking
   - Cache-aware searching

4. **Performance**:
   - Parallel searches
   - Result streaming for large result sets

## Maintenance Notes

### Breaking Changes

Changes that would break existing code:

- Changing return type from `Vec<Repository>` to custom search result type
- Adding required parameters to method signature
- Changing error variants returned

### Compatible Changes

Changes that maintain backward compatibility:

- Adding optional parameters with defaults
- Adding new fields to `Repository` (existing fields remain)
- Adding new error variants (callers using catch-all patterns unaffected)

### Deprecation Path

If this interface needs replacement:

1. Mark method as `#[deprecated]` with migration guidance
2. Provide new method with improved interface
3. Update all internal callers to new method
4. Remove deprecated method in next major version

**Note**: Project is pre-release, so deprecation may not be necessary. Clean removal is acceptable.
