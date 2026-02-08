# ADR-007: Repository Rulesets for Branch Protection

Status: Accepted
Date: 2026-02-09
Owners: RepoRoller team

## Context

RepoRoller manages repository configuration through hierarchical TOML files (ADR-002). Repository protection has historically been implemented via GitHub's branch protection API, but GitHub has introduced Repository Rulesets as a more flexible and powerful alternative.

Challenges with branch protection rules:

- Limited to branch-level protection (cannot protect tags)
- Complex API with many parameters
- Difficult to compose protections from multiple levels
- No evaluation mode for testing rules
- Limited bypass control

Repository Rulesets offer:

- Unified protection for branches, tags, and pushes
- Hierarchical composition support
- Evaluation mode for testing without enforcement
- Fine-grained bypass controls (per user, team, or app)
- Better support for organization-wide policies

When implementing repository ruleset support, several design decisions needed to be made:

1. **Additive vs name-based merging**: Should rulesets with the same name across configuration levels be merged or kept separate?
2. **Conflict detection**: Should the system detect and prevent conflicting rulesets?
3. **Rule type coverage**: Should all documented GitHub ruleset rule types be implemented initially?
4. **Integration point**: Where in the configuration pipeline should rulesets be applied?

## Decision

Implement **repository rulesets with additive composition** as the primary repository protection mechanism:

**Composition Strategy:**

- All rulesets from all configuration levels (Global → Type → Team → Template) are applied to repositories
- Rulesets are **additive** - each ruleset creates a separate GitHub ruleset entity
- Duplicate names are **not merged** - they create independent rulesets with the same name
- A warning is logged when duplicate names are detected to alert operators

**Conflict Detection:**

- **Not implemented initially** - rulesets are applied independently without validation
- GitHub's API will reject truly invalid configurations
- Operators are responsible for ensuring compatible rulesets
- Future enhancement: conflict detection can be added if operational experience shows need

**Rule Type Coverage:**

- Implement **8 core rule types** that cover 95% of use cases:
  - `deletion`, `non_fast_forward`, `required_linear_history`, `update`
  - `pull_request`, `required_status_checks`, `required_signatures`, `creation`
- **Defer 4 specialized rule types** until user need is demonstrated:
  - `required_deployments`, `commit_message_pattern`
  - `commit_author_email_pattern`, `committer_email_pattern`
- Additional rule types can be added incrementally based on demand

**Integration Point:**

- Apply rulesets **after webhook configuration** in the repository configuration pipeline
- Rulesets are part of repository settings (like labels, webhooks)
- Follows the same idempotent update pattern as other settings

## Consequences

**Enables:**

- Protection for branches, tags, and push operations through single API
- Hierarchical composition of repository protection across organization
- Testing protection rules in evaluation mode before enforcement
- Fine-grained bypass controls for automation and emergency access
- Organization-wide policy enforcement through global rulesets
- Template-specific protection customization

**Forbids:**

- Automatic merging of rulesets with the same name (creates duplicates instead)
- Automatic conflict detection between rulesets (manual coordination required)
- Using unimplemented rule types (`required_deployments`, commit patterns)

**Trade-offs:**

**Additive composition** (vs name-based merging):

- ✅ Simpler implementation - no merge logic required
- ✅ Clearer behavior - each level's rulesets are visible
- ✅ More flexible - allows intentional duplication
- ❌ Duplicate names create confusion - requires careful naming
- ❌ No automatic consolidation - more ruleset entities

**No conflict detection** (vs validation):

- ✅ Faster implementation - no complex validation logic
- ✅ Simpler code - fewer edge cases to handle
- ✅ GitHub API validates actual conflicts
- ❌ Errors discovered at apply time, not configuration time
- ❌ Requires operator understanding of ruleset interactions

**Limited rule types** (vs complete coverage):

- ✅ Covers common use cases immediately
- ✅ Reduces testing surface area
- ✅ Can add more based on actual demand
- ❌ Documentation must clearly indicate supported types
- ❌ Users may expect undocumented types to work

## Alternatives considered

### Option A: Name-based ruleset merging

Rulesets with the same name across levels would be merged into a single ruleset, with higher levels taking precedence for conflicting rules.

**Why not**:

- Complex merge semantics for rules (which fields merge? which replace?)
- Unclear precedence for nested arrays (required_checks, bypass_actors)
- Conditions require merge logic (include/exclude patterns)
- Harder to debug - resulting ruleset is non-obvious
- GitHub API doesn't require merging - multiple rulesets work fine

**When to reconsider**: If operators consistently request "update this ruleset at template level" and find duplicate names problematic.

### Option B: Implement comprehensive conflict detection

Detect conflicts before applying: overlapping branch patterns with contradictory rules, merge strategy deadlocks, bypass actors that contradict enforcement.

**Why not**:

- Adds significant complexity for unclear benefit
- GitHub API already validates truly breaking conflicts
- Many "conflicts" are actually valid (e.g., multiple rulesets for same branch)
- False positives would require override mechanism
- Operators understand their protection needs better than automated checks

**When to reconsider**: If operational experience shows common mistakes that conflict detection would prevent.

### Option C: Complete rule type coverage

Implement all 12 GitHub ruleset rule types including deployment requirements and commit patterns.

**Why not**:

- 4 unimplemented types have unclear use cases in RepoRoller context
- Deployment requirements depend on GitHub Environments configuration
- Commit patterns (email, message) are rarely used in practice
- Can add incrementally when users request them
- Reduces testing burden and time to ship

**When to reconsider**: When users request specific rule types with concrete use cases.

### Option D: Branch protection rules instead of rulesets

Continue using GitHub's branch protection API instead of adopting rulesets.

**Why not**:

- Branch protection cannot protect tags or push operations
- Less flexible composition model
- GitHub recommends migrating to rulesets
- No evaluation mode for testing
- More complex API surface

**When to reconsider**: If rulesets prove problematic or are deprecated by GitHub (unlikely).

## Implementation notes

**Configuration Structure:**

Rulesets are configured in TOML at any hierarchy level:

```toml
[[rulesets]]
name = "main-branch-protection"
target = "branch"
enforcement = "active"

[rulesets.conditions.ref_name]
include = ["refs/heads/main"]

[[rulesets.rules]]
type = "deletion"

[[rulesets.rules]]
type = "pull_request"
required_approving_review_count = 2
```

**Duplicate Name Detection:**

The merger logs warnings but allows duplicates:

```rust
if target.iter().any(|r| r.name == ruleset.name) {
    tracing::warn!(
        ruleset_name = %ruleset.name,
        source = ?source,
        "Ruleset with duplicate name detected - will create separate rulesets, not merge"
    );
}
```

**Idempotent Application:**

Rulesets follow the same update-or-create pattern as other settings:

```rust
// List existing rulesets
let existing = github_client.list_repository_rulesets(owner, repo).await?;

// Update existing or create new
for (name, config) in rulesets {
    if let Some(existing) = existing.get(name) {
        github_client.update_repository_ruleset(owner, repo, existing.id, config).await?;
    } else {
        github_client.create_repository_ruleset(owner, repo, config).await?;
    }
}
```

**Architecture Alignment:**

- **ADR-001 (Hexagonal Architecture)**: Domain types in `github_client`, configuration in `config_manager`, orchestration in `repo_roller_core`
- **ADR-002 (Hierarchical Configuration)**: Rulesets participate in 4-level hierarchy with override controls
- **ADR-003 (Stateless Architecture)**: No local state - all ruleset data via GitHub API

**Testing Strategy:**

- 25 tests: GitHub API integration (create, update, list rulesets)
- 24 tests: RulesetManager orchestration and idempotency
- 23 tests: TOML deserialization and validation
- 7 tests: Hierarchical configuration merging
- 5 tests: End-to-end with real GitHub API

## Examples

**Global organization defaults:**

```toml
# global/config.toml
[[rulesets]]
name = "org-wide-main-protection"
target = "branch"
enforcement = "active"

[rulesets.conditions.ref_name]
include = ["refs/heads/main", "refs/heads/master"]

[[rulesets.rules]]
type = "deletion"

[[rulesets.rules]]
type = "pull_request"
required_approving_review_count = 1
```

**Template-specific protection:**

```toml
# .reporoller/template.toml
[[rulesets]]
name = "rust-service-checks"
target = "branch"
enforcement = "active"

[rulesets.conditions.ref_name]
include = ["refs/heads/main"]

[[rulesets.rules]]
type = "required_status_checks"
required_checks = [
  { context = "rust/build" },
  { context = "rust/test" },
  { context = "rust/clippy" }
]
```

**Result**: Repository gets both rulesets applied independently.

## References

- GitHub Documentation: [Repository Rulesets](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets)
- Configuration Guide: `docs/configuration-guide.md` (sections on rulesets)
- Spec: `docs/spec/design/organization-repository-settings.md`
- Implementation: `crates/repo_roller_core/src/ruleset_manager.rs`
