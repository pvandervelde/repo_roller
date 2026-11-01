//! Pull request domain types.
//!
//! This module contains types representing GitHub pull requests, used by
//! the source-only mirror creation flow
//! (`docs/spec/interfaces/fork-mirror-operations.md` §5-6) to open a review
//! PR carrying the mirrored source contents.

use serde::Deserialize;

/// Payload for creating a new pull request via the GitHub REST API.
///
/// # Examples
///
/// ```rust
/// use github_client::PullRequestCreatePayload;
///
/// let payload = PullRequestCreatePayload {
///     title: "Mirror source contents".to_string(),
///     head: "mirror-branch".to_string(),
///     base: "main".to_string(),
///     body: Some("Mirrors the current contents of the source repository.".to_string()),
///     draft: false,
/// };
/// ```
#[derive(Debug, Clone, Default)]
pub struct PullRequestCreatePayload {
    /// The title of the pull request.
    pub title: String,
    /// The branch containing the changes.
    pub head: String,
    /// The branch to merge into (typically the default branch).
    pub base: String,
    /// The pull request description (optional).
    pub body: Option<String>,
    /// Whether to open the pull request as a draft.
    ///
    /// Defaults to `false`: `PullRequestCreatePayload::default()` must not
    /// silently create a draft PR.
    pub draft: bool,
}

/// A GitHub pull request, as returned by the create/update pull-request
/// operations on [`crate::RepositoryClient`].
///
/// Deserialized directly from the GitHub API response, extracting only the
/// fields RepoRoller needs. Deliberately does not route through
/// `octocrab::models::pulls::PullRequest`: that type requires full `Author`
/// objects (with every GitHub profile URL field) for the nested `head`/`base`
/// users, which is more than RepoRoller needs and more than GitHub's API
/// response is guaranteed to populate in every context.
#[derive(Debug, Clone, Deserialize)]
pub struct PullRequest {
    /// The pull request number.
    pub number: u64,
    /// The web URL of the pull request.
    pub html_url: String,
    /// Whether the pull request is currently a draft.
    pub draft: bool,
    /// The pull request's state (e.g. `"open"`, `"closed"`).
    pub state: String,
}
