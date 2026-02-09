# Operational Constraints (things we keep forgetting)

This file contains **practical reminders** about things we frequently forget.
For architectural rules, see the ADRs. For technology choices, see `.tech-decisions.yml`.

Keep this file SHORT and focused on operational gotchas.

## Testing Infrastructure (DON'T FORGET THIS EXISTS!)

- **Integration and E2E tests run against the `glitchgrove` GitHub organization**
  - NOT your personal repos
  - NOT the production organization
  - Tests use real GitHub API with real repositories
  - Location: `crates/integration_tests/` and `crates/e2e_tests/`
  - See: `tests/TESTING_STRATEGY.md` for details

- **Test repository sources are in `tests/` directory**
  - `tests/templates/` — Source definitions for template repositories
  - `tests/metadata/` — Source definitions for metadata repositories (.reporoller, .reporoller-required, .reporoller-restricted)
  - **CRITICAL**: Never edit test repos directly in GitHub!
  - **WORKFLOW**: Edit files in `tests/` → Run appropriate script → Changes pushed to GitHub

- **Scripts to update test repositories** (in `tests/` directory):
  - `create-test-repositories.ps1` — Initial creation of template repos
  - `setup-test-metadata-repos.ps1` — Initial creation of metadata repos
  - `update-templates-visibility.ps1` — Update visibility settings on template repos
  - `update-metadata-repos-visibility.ps1` — Update visibility settings on metadata repos
  - See: `tests/README.md` and `tests/README_METADATA.md` for usage

- **Test cleanup is required**
  - E2E tests create real repositories in glitchgrove org
  - Use `test_cleanup` crate to delete test repos after tests
  - See: `crates/test_cleanup/README.md`
  - Source: `.tech-decisions.yml` (e2e_tests section)

## Configuration Management (5 LEVELS!)

When adding or modifying configuration, remember there are **5 hierarchical levels**:

1. **System-wide defaults** — Built into the application
2. **Organization-wide** — `.reporoller/global/` in org metadata repo
3. **Team-level** — `.reporoller/teams/{team-name}/` in org metadata repo
4. **Repository type** — `.reporoller/types/{type-name}/` in org metadata repo
5. **Template-specific** — `.reporoller/` in template repository

**Resolution order**: Template → Type → Team → Org → System (most specific wins)

**Organization policies** (in `.reporoller-required/` and `.reporoller-restricted/`) cannot be overridden by lower levels.

Source: `docs/adr/ADR-002-hierarchical-configuration.md`

## Critical Technical Constraints

- **Never log tokens or secrets**: Use `secrecy::Secret` wrapper, no Debug impl
  - Pattern: `GitHubToken(secrecy::Secret<String>)`
  - Violating this leaks credentials in logs!

- **Use branded types for domain identifiers**: Never raw String/u64
  - Pattern: `RepositoryName(String)`, `OrganizationName(String)`, `UserId(u64)`
  - Prevents mixing up identifiers (passing repo name where org name expected)

- **Core crate stays pure**: `repo_roller_core` never imports infrastructure
  - No: `octocrab`, `handlebars`, `axum`, `tokio::fs`
  - Yes: Port traits defined in core, implemented in adapters
  - Source: `docs/adr/ADR-001-hexagonal-architecture.md`

## GitHub API Quirks (learn from our pain!)

- **Repository Rulesets API has different response shapes**:
  - **LIST** `/repos/{owner}/{repo}/rules`: Returns metadata ONLY (no `rules` field)
  - **GET** `/repos/{owner}/{repo}/rules/{ruleset_id}`: Returns full details (with `rules` field)
  - **CREATE** `/repos/{owner}/{repo}/rules`: Requires full payload (with `rules` field)
  - **Fix**: Use `#[serde(default)]` on optional fields to handle both shapes
  - Why this matters: Deserialization fails with "missing field `rules`" if struct expects it always

- **Ruleset ref patterns must match target type**:
  - Branch rulesets → Use `refs/heads/*` or `refs/heads/main` patterns
  - Tag rulesets → Use `refs/tags/*` or `refs/tags/v*` patterns
  - Mismatch causes GitHub API "Validation Failed" error (cryptic!)
  - Helper functions must be context-aware: select pattern based on target type

- **Private repositories + rulesets = GitHub Pro required**:
  - Free tier GitHub orgs cannot apply rulesets to private repos
  - Error message: "Upgrade to GitHub Pro or make repository public"
  - **Test strategy**: Use public repos for integration/e2e tests on free orgs
  - Don't waste cycles debugging—just make test repos public!

## Before You Build

- **New component?** Check `docs/catalog.md` — avoid reinventing the wheel
- **New error type?** Check `docs/spec/interfaces/error-types.md` — reuse existing hierarchy
- **Changing config?** Remember the 5 levels — update the right one!
- **Touching auth?** Read `docs/adr/ADR-006-github-app-authentication.md` — don't break security
- **Running tests?** Remember integration/e2e use glitchgrove org — don't spam production!

## Quick Reference

- Test infrastructure: `tests/TESTING_STRATEGY.md`
- Test repo workflow: `tests/README.md` + `tests/README_METADATA.md`
- Configuration hierarchy: `docs/adr/ADR-002-hierarchical-configuration.md`
- Architecture overview: `docs/adr/ADR-001-hexagonal-architecture.md`
- All ADRs: `docs/adr/README.md`
- Technology choices: `.tech-decisions.yml`
