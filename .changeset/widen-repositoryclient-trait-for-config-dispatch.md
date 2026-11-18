---
"github_client": minor
"repo_roller_core": minor
---

Make the repository-creation dispatch helpers uniformly trait-object based so
every helper is unit-testable with a mock `RepositoryClient` (follow-up to the
fork/mirror work in #346, Deferred Issue #1).

Previously `create_github_repository` and `initialize_git_repository` took
`&dyn RepositoryClient`, but `apply_post_creation_settings` and
`apply_repository_configuration` still required the concrete `GitHubClient`,
because the `LabelManager`, `WebhookManager`, `RulesetManager`, and
`PermissionManager` they build held an owned `GitHubClient`. That split meant
some helpers in `create_repository_inner`'s dispatch path were mockable and
others were not.

Changes:

- **`github_client`**: the `RepositoryClient` trait gained four methods that
  were previously inherent-only on `GitHubClient` —
  `add_team_to_repository`, `list_repository_collaborators`,
  `add_repository_collaborator`, and `remove_repository_collaborator`. These are
  the collaborator/team operations `PermissionManager` performs, now callable
  through the trait object. (Breaking for any external `RepositoryClient`
  implementor: the four methods must be implemented.)
- **`repo_roller_core`**: `LabelManager`, `WebhookManager`, `RulesetManager`,
  and `PermissionManager` now hold `Arc<dyn RepositoryClient>` instead of an
  owned `GitHubClient`, and their `new(...)` constructors take
  `Arc<dyn RepositoryClient>`. `apply_post_creation_settings` and
  `apply_repository_configuration` are widened to match.

This is a behaviour-preserving refactor — no change to what gets created or how
repositories are configured. Migration for `*Manager::new` callers: wrap the
client, e.g. `LabelManager::new(Arc::new(github_client))`.
