# Catalog (what exists / reuse map)

Purpose: prevent reinventing utilities, modules, patterns, and "hidden" features.

Add to this whenever a reusable component becomes "the standard way".

## Crate Structure

### Business Logic (Core)

- **`crates/repo_roller_core/`** — Core business logic and domain types
  Use when: Implementing repository creation orchestration, domain rules
  Key exports: `CreateRepositoryRequest`, `Repository`, `VisibilityResolver`, error types
  Notes: Never imports external SDKs (octocrab, handlebars). Only port trait definitions.

### Adapters (Infrastructure)

- **`crates/github_client/`** — GitHub API integration via octocrab
  Use when: Need to interact with GitHub API (create repos, fetch content, manage labels)
  Key exports: `GitHubClient`, `create_app_client()`, `GitHubError`
  Notes: Implements GitHub-specific operations, handles authentication

- **`crates/config_manager/`** — Configuration loading and management
  Use when: Loading organization/team/template configuration from .reporoller/ repos
  Key exports: `ConfigurationManager`, `TemplateLoader`, `VisibilityPolicyProvider`
  Notes: Implements 5-level configuration hierarchy with caching

- **`crates/template_engine/`** — Handlebars template processing
  Use when: Processing template files with variable substitution
  Key exports: `HandlebarsTemplateEngine`, `TemplateProcessor`
  Notes: Security-validated (path traversal prevention, resource limits)

- **`crates/auth_handler/`** — Authentication and authorization
  Use when: GitHub App authentication, token management
  Key exports: `GitHubAuthService`, `UserAuthenticationService`
  Notes: Handles JWT generation, installation token retrieval

### User Interfaces

- **`crates/repo_roller_cli/`** — Command-line interface
  Use when: Building CLI commands or argument parsing
  Key exports: CLI argument structures, command handlers
  Notes: Uses clap for argument parsing, system keyring for credentials

- **`crates/repo_roller_api/`** — REST API server (Axum)
  Use when: HTTP API endpoints for repository creation
  Key exports: API handlers, request/response types, middleware
  Notes: Translation layer between HTTP JSON and domain types

### Testing Infrastructure

- **`crates/test_utils/`** — Shared test utilities and mocks
  Use when: Writing tests that need mock implementations or test fixtures
  Key exports: Mock implementations of port traits, test builders
  Notes: Reusable across all test types

- **`crates/integration_tests/`** — Integration tests with real GitHub API
  Use when: Testing full workflows with real GitHub interactions
  Key exports: Integration test helpers, test repository management
  Notes: Uses glitchgrove test organization

- **`crates/e2e_tests/`** — End-to-end tests
  Use when: Testing complete user journeys (CLI → repo creation)
  Key exports: E2E test scenarios
  Notes: Expensive, uses real GitHub API quota, requires cleanup

- **`crates/test_cleanup/`** — Test repository cleanup utilities
  Use when: Cleaning up test repositories after test runs
  Key exports: Cleanup functions for test repositories
  Notes: See also `tests/cleanup-repos.ps1`

## Common Building Blocks

### Domain Types (Branded/Newtype Pattern)

- **`repo_roller_core::RepositoryName`** — Type-safe repository name
  Use when: Passing repository names between functions
  Pattern: `RepositoryName::new("my-repo")?` validates and wraps String

- **`repo_roller_core::OrganizationName`** — Type-safe organization name
  Use when: Passing organization names between functions
  Pattern: Similar to RepositoryName, prevents mixing with repo names

- **`repo_roller_core::GitHubToken`** — Secure token wrapper with secrecy
  Use when: Storing or passing GitHub tokens
  Pattern: `GitHubToken::new(token)`, no Debug impl, uses secrecy::Secret
  Notes: Never logs token value, even in debug output

- **`repo_roller_core::TemplateName`** — Type-safe template identifier
  Use when: Referencing template repositories
  Pattern: `TemplateName::new("rust-library")?`

### Error Handling

- **Error types** — Hierarchical error types with thiserror
  Location: `repo_roller_core/src/errors.rs`
  Pattern: `RepoRollerError` → domain-specific errors (GitHubError, ConfigurationError, etc.)
  Use when: All fallible operations (use `Result<T, ErrorType>`)
  Notes: See `docs/spec/interfaces/error-types.md` for complete hierarchy

### Configuration

- **Configuration hierarchy** — 5-level configuration resolution
  Location: `config_manager/src/lib.rs`
  Pattern: System → Org → Team → Type → Template (most specific wins)
  Use when: Loading any configuration (defaults, policies, template settings)
  Notes: Cached with 5-minute TTL, see ADR-002

### Fork & Mirror Creation (#255)

- **`config_manager::TemplateDesignation`** — special role a template
  self-declares beyond ordinary template-based creation (`Fork`,
  `SourceHistoryMirror`, `SourceOnlyMirror`)
  Location: `config_manager/src/template_config.rs`
  Use when: A template's settings (not files) should apply automatically to
  forks or mirrors; see `TemplateConfig::designation` (`Option<TemplateDesignation>`)
  Notes: At most one template per org may declare a given designation —
  enforced by `find_designated_template` below, not at template-load time. A
  template MUST NOT combine `designation` with a `Fixed` `repository_type`
  policy (validated at template-load time). See
  `docs/spec/interfaces/fork-mirror-operations.md` §1.

- **`OrganizationSettingsManager::find_designated_template(org, designation)`**
  — finds the single template (if any) in `org` declaring `designation`
  Location: `config_manager/src/organization_settings_manager.rs`
  Returns: `Ok(None)` for zero matches, `Ok(Some(name))` for one,
  `Err(ConfigurationError::AmbiguousTemplateDesignation)` for two or more
  Use when: Resolving the fork/mirror settings-only template for a
  `CreationMode::Fork`/`Mirror` request before calling `TemplateLoader`
  Notes: Result shares the 5-minute `template_configurations` cache TTL.

- **`config_manager::settings::CreationCapabilitiesSettings`** — independent
  enable/disable toggles for fork creation and the two mirror modes
  (`fork_enabled`, `source_history_mirror_enabled`, `source_only_mirror_enabled`),
  each an `Option<OverridableValue<bool>>`
  Location: `config_manager/src/settings/creation_capabilities.rs`
  Carried on: `GlobalDefaults`, `RepositoryTypeConfig`, `TeamConfig`,
  `TemplateConfig` (same four structs that carry `repository`/
  `pull_requests`/`branch_protection`), merged into
  `MergedConfiguration::creation_capabilities`
  Use when: Gating any new fork/mirror-like capability behind an
  administrator-controlled, overridable toggle with Global → RepositoryType →
  Team → Template precedence
  Notes: **Fail-closed default** — a fully-`None` value at every level means
  **disabled**, not enabled (unlike most other `Option<OverridableValue<_>>`
  settings in this codebase, which tend to default permissively). Resolve
  with `flag.as_ref().map(|v| *v.get()).unwrap_or(false)` — never
  `unwrap_or(true)`. See `docs/spec/interfaces/fork-mirror-operations.md` §2.

- **`ConfigurationMerger::merge_creation_capabilities_settings` /
  `merge_capability_field`** — field-by-field scalar merge with override-policy
  validation and source tracking for `CreationCapabilitiesSettings`
  Location: `config_manager/src/merger.rs`
  Pattern: Mirrors `merge_repository_settings`'s per-field
  validate-then-assign-then-track shape (CRA-001/CRA-002), but factors the
  per-field body into a private `merge_capability_field` helper (REFACTOR
  pass) since `CreationCapabilitiesSettings` has 3 fields of the same
  `Option<OverridableValue<bool>>` shape — call this helper rather than
  re-inlining the validate/assign/track block if a fourth capability flag is
  ever added. Note: `merge_repository_settings` and its ~20 pre-existing
  sibling functions (`merge_pull_request_settings`, etc.) still inline this
  shape per-field; they were not touched by this extraction (out of scope —
  see `.llm/findings/fork-mirror-operations.md` for the cross-cutting
  observation).

- **`repo_roller_core::CreationMode`** — orthogonal-to-`ContentStrategy` enum
  governing how the GitHub repository itself comes into existence
  (`Standard`, `Fork { source }`, `Mirror { source, preserve_history }`)
  Location: `repo_roller_core/src/request.rs`
  Use when: A `RepositoryCreationRequest` needs anything other than the
  create-then-run-content-strategy path. Set via
  `RepositoryCreationRequestBuilder::creation_mode(mode)`.
  Notes: `content_strategy` must stay at its default when `creation_mode` is
  `Fork`/`Mirror` — builder-time panic otherwise. See
  `docs/spec/interfaces/fork-mirror-operations.md` §4.

- **`repo_roller_core::SourceRepositoryReference`** — identifies a repository
  to fork/mirror from, which may be outside the managed organization
  Location: `repo_roller_core/src/repository.rs`
  Pattern: `{ owner: OrganizationName, name: RepositoryName }` — no new
  validation rules, delegates entirely to those two branded types
  Use when: Referencing an upstream/source repository in `CreationMode::Fork`/
  `Mirror`

- **`repo_roller_core::create_repository_inner` dispatch on `CreationMode`**
  — four extracted orchestration functions, one per `CreationMode` arm:
  `create_standard_repository`, `create_fork_repository`,
  `create_history_mirror_repository`, `create_source_only_mirror_repository`
  Location: `repo_roller_core/src/lib.rs`
  Pattern: `validate_creation_mode_capability` (capability + designation gate,
  RCF-005/RCF-006) runs once, before the dispatch, for every non-`Standard`
  mode — before any GitHub-mutating call. Each arm ends by building its
  `RepositoryCreationResult` via the shared `build_creation_result(&repo,
  default_branch)` helper (REFACTOR pass — the four arms previously repeated
  the same four-field struct literal verbatim).
  Use when: Adding a fifth `CreationMode` variant — add a new dispatch arm
  calling a new `create_*_repository` function, and reuse
  `build_creation_result`/`build_github_https_url` rather than re-inlining
  their logic.
  Notes: `initialize_git_repository` and `create_github_repository` take
  `&dyn RepositoryClient` (widened from `&GitHubClient` for unit-test
  mockability); `apply_post_creation_settings` /
  `apply_repository_configuration` were deliberately **not** widened — see
  `.llm/findings/fork-mirror-operations.md` item 1 for why and the follow-up
  options.

- **`github_client::RepositoryClient::create_fork` / `create_pull_request` /
  `mark_pull_request_ready_for_review`** — fork creation, PR creation, and
  draft→ready-for-review transition
  Location: `github_client/src/lib.rs` (trait + `GitHubClient` impl),
  payload/response types in `github_client/src/pull_request.rs`
  Key exports: `PullRequestCreatePayload` (`title`, `head`, `base`, `body`,
  `draft` — `draft: false` by default), `PullRequest` (`number`, `html_url`,
  `draft`, `state`)
  Notes: Deserializes directly into RepoRoller's own `Repository`/
  `PullRequest` types rather than `octocrab::models::*`, since the fork/PR
  REST responses' nested `owner`/`head`/`base` author objects don't
  reliably satisfy octocrab's richer `Author` model. Reuses the pre-existing
  `log_octocrab_error` helper and its `match response { Ok(r) => ...,
  Err(e) => {...} }` mapping idiom — already used 50+ times in this file —
  do not introduce a new error-mapping pattern for future `RepositoryClient`
  methods.

- **`repo_roller_core::git::mirror_clone` / `mirror_push` /
  `shallow_clone_working_tree` / `create_snapshot_branch`** — local-git
  primitives for the two mirror modes (no GitHub API calls)
  Location: `repo_roller_core/src/git.rs`
  Key exports: `mirror_clone(source_url, token) -> (TempDir, default_branch)`
  (bare `--mirror`-equivalent clone + all-refs fetch), `mirror_push(dir,
  repo_url, token)` (`--mirror`-equivalent push), `shallow_clone_working_tree`
  (depth-1 clone for current-contents-only mirrors), `create_snapshot_branch`
  (branches off a base branch, replaces the working tree with another
  repository's current contents, commits as a single child — no
  merge/cherry-pick)
  Notes: All four share `installation_token_credential_callbacks(token)` —
  the same `x-access-token` installation-token credential strategy
  `push_to_origin` uses. `push_to_origin` itself was refactored (REFACTOR
  pass) to call this shared helper instead of its own independent inline
  credential closure, consolidating what had become two parallel
  implementations of the same authentication logic into one. `prepare_index_and_tree`,
  `create_initial_commit`, and the new `create_snapshot_branch` share tree/commit
  plumbing; the author/committer identity itself is centralized in
  `repo_roller_signature()` (REFACTOR pass — previously duplicated verbatim
  between `create_initial_commit` and `create_snapshot_branch`). Use
  `repo_roller_signature()` for any future RepoRoller-authored commit rather
  than re-inlining `Signature::now("RepoRoller", "repo-roller@example.com")`.

- **`repo_roller_core::git::copy_dir_contents`'s symlink guard** —
  security-remediation fix (CRITICAL finding) for arbitrary file disclosure
  via symlink dereference during the source-only mirror's working-tree-copy
  step
  Location: `repo_roller_core/src/git.rs`, called from
  `replace_working_tree_contents` ← `create_snapshot_branch`
  Pattern: Checks `entry.file_type()?.is_symlink()` for every directory entry
  (recursively) and `continue`s past it — neither dereferencing it (the
  `std::fs::copy` default behavior, which would copy a symlink *target's*
  bytes even when the target lives outside the cloned source tree) nor
  recreating it as a symlink in the destination. Regular sibling files/dirs
  are still copied normally when a symlink is skipped. Use this
  `is_symlink()`-check-then-skip pattern for any future recursive
  tree-copy/tree-walk helper that reads attacker-influenced (cloned source
  repository) content into a location that gets committed/pushed/exposed.
  See `docs/spec/interfaces/fork-mirror-operations.md` §7.

### Caching

- **Policy cache** — In-memory cache with TTL
  Location: Throughout config_manager and github_client
  Pattern: `Arc<RwLock<HashMap<Key, CachedValue>>>`
  TTLs: Org policies (5 min), GitHub environment (1 hour), templates (5 min)
  Notes: Thread-safe, automatic expiration, explicit invalidation

### Metrics (Observability Phase 1)

- **`repo_roller_core::event_metrics::EventMetrics`** — event-delivery metrics trait
  Use when: Recording notification delivery success/failure/latency
  Impls: `PrometheusEventMetrics` (production), `NoOpEventMetrics` (tests/disabled)
  Pattern this crate's other metrics traits mirror: `trait` + `Prometheus*`/`NoOp*` impls,
  `Prometheus*::new(&registry)` panics on duplicate registration.

- **`repo_roller_core::repository_metrics::RepositoryCreationMetrics`** — repository-creation
  request/success/failure/duration/active-tasks metrics trait
  Use when: Instrumenting the repository-creation flow (`create_repository` handler)
  Impls: `PrometheusRepositoryCreationMetrics`, `NoOpRepositoryCreationMetrics`
  Also exports: `error_category(&RepoRollerError) -> &'static str` — maps errors to one of
  8 bounded label categories; never use the error's `Display`/`to_string()` as a label.
  Metric names: `repository_creation_{requests,successes,failures}_total`,
  `repository_creation_duration_seconds`, `repository_creation_active_tasks`
  **Security (post-Security-Review remediation)**: `record_request()`/`record_success(duration)`
  take no parameters and `record_failure(error_category, duration)` takes only the bounded
  `error_category` label. `organization`/`template` were removed as label values — they came
  directly from the untrusted HTTP request body and, combined with the unauthenticated
  `/metrics` endpoint, allowed unbounded Prometheus label cardinality (resource exhaustion) and
  cleartext exposure of organization names. `repository_creation_requests_total`,
  `_successes_total`, and `duration_seconds` are now plain `Counter`/`Histogram`; only
  `repository_creation_failures_total` remains a `CounterVec`, labeled solely by
  `error_category`.

- **`github_client::api_metrics::GitHubApiMetrics`** — GitHub API call-count/error/rate-limit
  metrics trait
  Use when: Instrumenting a GitHub API call site (see `handlers.rs`'s
  `validate_repository_name`/`list_organization_teams` for the call/record_error pattern)
  Impls: `PrometheusGitHubApiMetrics`, `NoOpGitHubApiMetrics`
  Also exports: `status_category(&github_client::Error) -> &'static str` — same bounded-label
  rule as `error_category` above
  Metric names: `github_api_calls_total`, `github_api_errors_total`,
  `github_api_rate_limit_remaining`

- **`repo_roller_api::http_metrics::HttpMetrics`** — per-endpoint HTTP request metrics trait
  Use when: Already wired globally via `http_metrics_middleware`; new handlers get this for
  free, no per-handler code needed
  Impls: `PrometheusHttpMetrics`, `NoOpHttpMetrics`
  Middleware: `http_metrics_middleware` (axum `from_fn_with_state`), labels by axum's
  `MatchedPath` route *template* (never the concrete request path) — falls back to the
  `"unmatched"` sentinel on 404s so label cardinality stays bounded
  Metric names: `http_requests_total`, `http_request_duration_seconds`

- **`/metrics` endpoint & shared registry** — `AppState::metrics_registry`
  (`Arc<prometheus::Registry>`) is the single registry every metrics collector above
  registers against; `handlers::metrics_handler` gathers it and encodes with
  `prometheus::TextEncoder`. Unauthenticated (same tier as `/health`), mounted at the router
  root (`/metrics`, not `/api/v1/metrics`) in both `create_router` and
  `create_router_without_auth`.
  Notes: All vector metrics (`CounterVec`/`HistogramVec`) pre-seed a sentinel label
  combination at construction so their metric family survives `Registry::gather()`'s
  pruning of zero-activity families before the first real request. See
  `repo_roller_core::repository_metrics::UNSEEDED_SENTINEL` for the rationale.
  Internal helpers (private, `repo_roller_api` crate only — use these rather than
  re-inlining registry/route construction if you add a new metrics collector or router
  variant): `main::MetricsBundle::new()` builds the registry plus all four collectors in
  one call (used by both `AppState::new` and the `#[cfg(test)] Default` impl);
  `routes::mount_metrics(api_v1, &state)` wires the `/metrics` route + `http_metrics_middleware`
  onto an inner `api_v1` router (used by both `create_router` and `create_router_without_auth`).

### Logging & Correlation (Observability Task 1)

- **`repo_roller_api::logging`** — JSON/pretty log format selection plus a
  sensitive-field sanitization layer for the API binary's `tracing` subscriber
  Use when: Wiring up (or extending) the API binary's global tracing subscriber —
  new call sites should go through `logging::init_tracing()` (already wired in
  `main.rs`) rather than calling `tracing_subscriber::fmt()...init()` directly
  Key exports: `resolve_log_format` (pure `LOG_FORMAT` env-var parsing, JSON is the
  fail-safe default), `SENSITIVE_FIELD_NAMES` / `is_sensitive_field_name`
  (case-insensitive substring deny-list, biased toward over-redaction),
  `redact_sensitive_fields` (recursive JSON redaction of object *keys*, idempotent,
  passes non-JSON input through unchanged), `looks_like_secret_value` /
  `redact_secret_shaped_substrings` (value-level, pattern-based secret detection —
  see below), `init_tracing` (production entry point)
  Pattern: `SanitizingWriter<W: io::Write>` / `SanitizingMakeWriter<W>` (now
  `pub(crate)` in `repo_roller_api::sanitizing_writer` -- extracted there
  during Task #2's REFACTOR pass, previously private to this module; see
  that module's doc comment for why the writer-wrapping *types* are shared
  crate-wide even though `logging::build_sanitizing_json_subscriber` and
  `tracing_otel::init_tracing_with_otel`/`build_composed_subscriber_for_test`
  build genuinely different `Subscriber` shapes around them) wrap any
  `tracing_subscriber::fmt::MakeWriter` so every formatted JSON line is redacted
  before any byte reaches the underlying writer; `build_sanitizing_json_subscriber(writer)`
  assembles the full `Subscriber` from this wrapper (exposed chiefly for tests that
  supply an in-memory writer). Reuse `crate::sanitizing_writer`'s types for any future
  writer-level cross-cutting concern rather than inventing a new one.
  Notes: Redaction matches on object *key* name (never a field's rendered value/prose
  by keyword), **and independently** scans every string value in the JSON tree
  (including `message`) for secret-*shaped* substrings via
  `redact_secret_shaped_substrings` — this is defense-in-depth against a secret
  reaching a log line via `{:?}`/`{}` interpolation into free text rather than as a
  sensitively-named field (the HIGH-severity gap fixed alongside
  `crates/config_manager/src/github_metadata_provider.rs`'s `load_template_configuration`,
  see below). Recognised shapes: GitHub token prefix (`ghp_`/`gho_`/`ghu_`/`ghs_`/
  `ghr_`/`github_pat_` + >=20 char suffix), the literal word `Bearer`/`bearer`
  followed by one space and >=8 non-whitespace chars, three-segment JWT shape,
  and a >=32-char high-entropy run of
  `[A-Za-z0-9+/]` containing mixed case + a digit — see the module doc on
  `crates/repo_roller_api/src/logging.rs` for the full contract, exact thresholds,
  and known spec gaps. Internal pattern: `SecretShapeScanner` (private,
  `logging.rs`) precomputes per-position maximal-run lengths for each shape's
  character class via one backward O(n) dynamic-programming pass per class, so the
  forward substring scan is O(n) overall rather than O(n^2) on long homogeneous
  runs — reuse this precomputed-run-length approach rather than a naive
  per-position rescan if a future shape needs adding. The GitHub-token-prefix and
  Bearer-header shapes both reduce to "one of a fixed set of literal prefixes,
  followed by a minimum-length run of a fixed character class" and share that
  logic via `SecretShapeScanner::literal_prefixed_run_match_len` (private) — reuse
  this helper (rather than re-deriving the find-map-over-literals shape) for any
  future shape with the same "literal prefix + threshold run" structure.

- **Correlation ID / request span** — `repo_roller_api::middleware::tracing_middleware`
  Use when: Any future outermost middleware layer needs every log line for the
  lifetime of one request to share an identifier — extend this span's fields
  rather than inventing a second per-request ID scheme
  Pattern: generate one UUID v4 per request, open a
  `tracing::info_span!("http_request", request_id = %request_id, method = %.., uri = %..)`
  and run the rest of the middleware chain `.instrument`ed inside it (a span field,
  not a one-off event field) so every nested `tracing::info!`/`warn!`/etc. call
  anywhere in the handler call stack automatically inherits `request_id`; the same
  value is echoed back as the `X-Request-Id` response header for client/server log
  correlation.

### Distributed Tracing (Observability Task 2)

- **`repo_roller_api::tracing_otel`** — OpenTelemetry/OTLP span-export wiring for
  the API binary, composed with Task 1's sanitizing JSON logging layer on one
  `tracing_subscriber::Registry`
  Use when: Wiring up (or extending) the API binary's global tracing subscriber
  with OTel export — new call sites should go through `tracing_otel::init_tracing_with_otel()`
  (already wired in `main.rs`, replacing Task 1's `logging::init_tracing()` call
  site) rather than calling `tracing_subscriber::fmt()...init()` or
  `logging::init_tracing()` directly
  Key exports: `resolve_otel_settings` (pure env-var parsing of
  `OTEL_TRACES_ENABLED`/`OTEL_EXPORTER_OTLP_ENDPOINT`/`OTEL_SERVICE_NAME`,
  fail-safe-**off** default — unlike `logging::resolve_log_format`'s JSON-safe
  default, an unset/unrecognised value here must never silently enable
  background network export), `build_otel_layer_with_exporter` (exporter-agnostic
  `Layer` builder using a synchronous `SimpleSpanProcessor` — accepts any
  `opentelemetry_sdk::trace::SpanExporter`, so tests inject `InMemorySpanExporter`
  and production injects the real OTLP/gRPC exporter), `build_composed_subscriber_for_test`
  (testing seam combining Task 1's redaction layer with an optional OTel layer),
  `init_tracing_with_otel` (production entry point)
  Notes: builds its JSON layer with `crate::sanitizing_writer::SanitizingMakeWriter`
  (extracted from `logging.rs` during Task #2's REFACTOR pass) rather than a
  second copy of the same writer-wrapping types — see
  "Logging & Correlation (Observability Task 1)" above for that pattern.
  Gotcha: `opentelemetry_sdk::trace::SdkTracerProvider` auto-triggers `shutdown()`
  when its *last* strong reference is dropped, which for exporters with
  reset-on-shutdown semantics (e.g. `InMemorySpanExporter`, used throughout this
  module's own tests) silently clears already-exported spans. `build_otel_layer_with_exporter`
  deliberately leaks one extra provider reference (`std::mem::forget(provider.clone())`)
  so the returned `Layer` being dropped (e.g. a test's scoped `tracing::subscriber::set_default`
  guard going out of scope) never triggers this — reuse this pattern for any
  future function that builds a `Layer` from a fresh `SdkTracerProvider` and hands
  ownership of the tracer (not the provider) to the caller.

- **`github_client::span_attributes`** — business-context span-attribute
  sanitization, a data path entirely separate from `repo_roller_api::logging`'s
  redaction (span attributes bypass the `fmt` layer's JSON output entirely)
  Use when: Attaching any attacker-influenceable business-context value
  (organization login, repository name, template name, requestor/user login,
  team slug, path, etc.) to a `tracing` span via `Span::record`/
  `#[instrument(fields(...))]`/`tracing::field::debug` for a span that may be
  exported to OpenTelemetry — always pass the raw value through
  `sanitize_span_attribute_value` first, never attach it raw
  Key exports: `sanitize_span_attribute_value` (whole-value secret-shape
  rejection — same four shapes as `logging.rs`'s scanner, reimplemented
  independently since `github_client` must not depend on `repo_roller_api` —
  plus length-bounding to `MAX_SPAN_ATTRIBUTE_LEN` (128 `char`s) for cardinality
  safety), `SPAN_ATTRIBUTE_REDACTED_MARKER`
  Notes: as of this task, every `github_client::lib` `#[instrument]` site
  that attaches a business-context field (`org_name`, `owner`, `repo`,
  `team_slug`, `username`, `path`, `topic`, `permission`, `repo_owner`,
  `base_url`, etc.) routes it through `sanitize_span_attribute_value` —
  earlier passes left ~23 sites unsanitized (see
  `.llm/findings/task-2-tracing.md` for history), but follow-up commits
  closed all of them, including `repo_roller_core::permission_manager`'s
  `owner`/`repo` fields and `repo_roller_core::policy_engine`'s `requestor`
  field, and `config_manager::organization_settings_manager`'s `team`/
  `repository_type` fields. A later Verifier pass found that
  `repo_roller_core::policy_engine`'s `evaluate_permission_request` span
  also left its `repository`/`organization` fields unsanitized, on the
  disproven assumption that the `RepositoryName`/`OrganizationName`
  newtypes' constructor-time charset validation makes a secret-shaped value
  impossible to construct (it does not — e.g. a 32-char mixed-case
  alphanumeric `OrganizationName`, or a `ghp_`-prefixed `RepositoryName`,
  both satisfy the newtypes' validation while still being secret-shaped);
  both fields now also route through `sanitize_span_attribute_value` (via
  each newtype's `as_str()` accessor), closing this gap.
  Gotcha (`#[instrument]` `fields()` override vs `skip()`): a `fields(...)`
  entry only suppresses `tracing`'s auto-Debug-capture of a parameter when
  the field name is an *exact* match for that parameter's name. Building a
  sanitized field from a dotted/method-call expression on a struct
  parameter — e.g. `fields(requestor = %sanitize(&request.requestor))` or
  `fields(organization = %sanitize(context.organization()))` — does **not**
  count as naming the `request`/`context` parameter itself, so unless that
  parameter is *also* added to `skip(...)`, the whole struct still gets
  auto-captured raw under a field literally named after the parameter
  (`request`/`context`), re-leaking the already-sanitized values plus any
  other never-individually-sanitized fields on the struct (e.g.
  `justification`) right next to the correctly-sanitized named fields. Two
  sites had exactly this gap — `repo_roller_core::policy_engine`'s
  `evaluate_permission_request` (`skip(self, hierarchy)` missing `request`)
  and `config_manager::organization_settings_manager`'s
  `resolve_configuration` (`skip(self)` missing `context`) — found by a
  Verifier after two prior fix rounds focused only on individual field
  sanitization. Both are now fixed by adding the struct parameter to
  `skip(...)`. Rule for future `#[instrument]` sites with a struct
  parameter: either (a) add the struct parameter to `skip(...)` and list
  each sanitized field individually in `fields(...)` as these two sites now
  do, or (b) give the sanitized field the exact same name as the parameter
  it replaces (only viable when there is a single field derived from that
  parameter).

- **`repo_roller_api::middleware::extract_trace_context`** — inbound W3C
  `traceparent` extraction/continuation (the receiving-side counterpart to
  `tracing_otel`'s outbound OTLP export above — extraction decodes an
  external caller's trace context *into* this process; `tracing_otel`
  decodes nothing, it only ships this process's own spans *out*)
  Use when: Any inbound-HTTP entry point that should continue an external
  caller's trace rather than always starting a fresh root span — currently
  wired into `middleware::tracing_middleware`'s `http_request` span via
  `tracing_opentelemetry::OpenTelemetrySpanExt::set_parent` (call before the
  span is entered, never after — `tracing-opentelemetry` rejects a
  post-entry `set_parent`)
  Key exports: `extract_trace_context(&HeaderMap) -> opentelemetry::Context`
  (reads the `opentelemetry::global::get_text_map_propagator`, i.e. honors
  whatever propagator `tracing_otel::init_tracing_with_otel` installed
  process-wide — currently `opentelemetry_sdk::propagation::TraceContextPropagator`;
  absent/malformed `traceparent` fail-safe to an invalid context, equivalent
  to today's pre-extraction fresh-root behavior, never panics)
  Notes: `HeaderMapExtractor` (private, local to `extract_trace_context`) is
  a minimal `opentelemetry::propagation::Extractor` adapter over
  `axum::http::HeaderMap` — deliberately NOT a dependency on
  `opentelemetry-http`'s `HeaderExtractor`, to avoid pulling in a whole crate
  for one trait impl; reuse this adapter (or lift it out of the function) if
  a second inbound-extraction call site appears rather than re-deriving it.
  Test-only helpers `ensure_trace_context_propagator_installed` /
  `headers_with_traceparent`, shared by `middleware_tests.rs` and
  `middleware_proptests.rs`, live in `middleware_test_support.rs` (extracted
  there during this task's REFACTOR pass to remove a duplicate definition in
  each file) — reuse these rather than re-defining either helper if a third
  test module needs them.

- **`repo_roller_core::observability`** — canonical span names and hierarchy
  contract for `create_repository`'s orchestration phases
  Use when: Instrumenting (or reasoning about) the `create_repository` span tree —
  reference `SPAN_REPOSITORY_CREATION`/`SPAN_AUTHENTICATION`/
  `SPAN_CONFIGURATION_RESOLUTION`/`SPAN_TEMPLATE_PROCESSING`/
  `SPAN_GITHUB_REPOSITORY_CREATION` symbolically rather than hard-coding span
  name string literals
  Key exports: the five `SPAN_*` name constants above, `ATTR_ORGANIZATION`/
  `ATTR_TEMPLATE`/`ATTR_REQUESTOR`, `business_context_attributes` (builds the
  bounded, sanitized attribute set for the top-level span by delegating to
  `github_client::span_attributes::sanitize_span_attribute_value` — never
  re-implements sanitization independently)

### Frontend Observability (Observability Task 3)

- **`frontend/src/lib/logger.server.ts`** — server-only structured logger
  Use when: Logging anything in `*.server.ts`/`+page.server.ts`/`+server.ts` SvelteKit code —
  replaces raw `console.*` calls
  Key exports: `logger` (singleton `Logger`: `debug`/`info`/`warn`/`error`, each taking
  `(message: string, meta?: LogMetadata)`), `LogRecord`/`LogMetadata`/`LogSink` types,
  `_setLogSinkForTesting(sink)`/`_resetLogSinkForTesting()` test hooks (mirrors
  `brand.server.ts`'s `_resetBrandConfigForTesting` convention)
  Notes: Library-agnostic by design — does NOT use `pino` (see Rules & Tips in
  `.llm/tasks.md` for why: `pino`'s `"browser"` package.json field gets picked up under this
  project's Vitest config and silently falls back to calling `console.*` internally). Default
  (no test sink) emission path writes one JSON line per call via `process.stdout.write()`.
  Every field under `meta` is deep-sanitized before emission: `Error` instances become
  `{name, message, stack}` (plain `JSON.stringify` of an `Error` produces `{}`), circular
  references become the string `'[Circular]'`. Never throws regardless of `meta` contents.
  Server-only (not safe to import client-side).

- **`frontend/src/lib/observability/trace-context.ts`** — W3C Trace Context (`traceparent`)
  helpers, pure/framework-agnostic (works in both browser and server code)
  Use when: Attaching a `traceparent` header to any outbound `fetch()` call that should
  participate in the same distributed trace as the backend (Task 2)
  Key exports: `getCurrentTraceparent()` (never throws, always returns a well-formed header —
  the function to call at an actual `fetch()` call site), `getActiveSpanContext()`,
  `buildTraceparentHeader(parent?)`, `TRACEPARENT_FORMAT` regex, `ActiveSpanContext` type
  Notes: Manual propagation by design, NOT OTel auto-instrumentation — auto-instrumentation
  patches global `fetch`, which is invisible to this codebase's `vi.stubGlobal('fetch', ...)`
  test pattern. Already wired into `api/client.ts`'s `apiFetch()` and `proxy.server.ts`'s
  `proxyToBackend()` — any other outbound `fetch()` call that should be traced must call
  `getCurrentTraceparent()` itself the same way, auto-instrumentation will not catch it.

- **`frontend/src/lib/observability/spans.ts`** — span-wrapping helpers for instrumenting
  discrete user operations (form submissions, data loads) AND for continuing an inbound
  distributed trace across the SvelteKit server hop
  Use when: Instrumenting a user-facing async operation (API call triggered by a component
  action) with a named span (`withSpan`), or wrapping the SvelteKit server's own per-request
  handling so it parents to whatever trace the browser started (`withRequestSpan`)
  Key exports: `withSpan(name, fn, attributes?)` (starts a span — fresh root or child of
  whatever's currently active — runs `fn` with it active, ends it exactly once on every path,
  rethrows `fn`'s error unchanged); `withRequestSpan(request, fn)` (extracts a W3C
  `traceparent` from `request.headers` via `propagation.extract()` against the globally
  registered propagator, starts `SPAN_NAMES.SVELTEKIT_REQUEST` as a child of that extracted
  context — or a fresh root span if the header is absent/malformed/all-zero — then runs `fn`
  with the same end-exactly-once/error-passthrough contract as `withSpan`; depends only on
  the standard `Request.headers` interface, not SvelteKit's `RequestEvent`, so it stays
  testable independent of SvelteKit's routing machinery); `SPAN_NAMES` (`CREATE_REPOSITORY_SUBMIT`,
  `LOAD_TEMPLATES`, `SVELTEKIT_REQUEST` — add new operation names here, never inline a
  span-name string literal at a call site); `SpanHandle` type
  Notes: `withSpan` is wired into `routes/create/+page.svelte`'s `loadTemplates()` (wraps
  `listTemplates()`) and `submitCreate()` (wraps `createRepository()`); `withRequestSpan` is
  wired into `hooks.server.ts`'s `handle` hook, wrapping its `resolve(event)` call — this is
  what lets `proxy.server.ts`'s `proxyToBackend()` → `getCurrentTraceparent()` (see
  `trace-context.ts` above) find an active span to continue later in the same request instead
  of minting a disconnected new root trace. Both functions share their end-exactly-once /
  error-passthrough span lifecycle via a private `runSpanLifecycle(span, fn)` helper
  (extracted during this task's REFACTOR pass — `withRequestSpan` initially duplicated
  `withSpan`'s inline try/end/catch-record-status-end-rethrow block verbatim); reuse this
  helper rather than re-inlining that block if a third `withSpan`-family function is ever
  added. Internal-only `headersTextMapGetter` (private, local to this module) is a minimal
  `@opentelemetry/api` `TextMapGetter<Headers>` adapter letting the registered
  `TextMapPropagator` read a standard `Headers` object — the conceptual (not code-shared,
  different language) counterpart to the Rust backend's `HeaderMapExtractor` in
  `repo_roller_api::middleware::extract_trace_context` above; confirmed no second copy exists
  elsewhere in `frontend/`.

- **`frontend/src/lib/observability/observability.server.ts` /
  `observability.client.ts`** — Node/browser OTel SDK bootstrap (`NodeTracerProvider`/
  `WebTracerProvider` + `UndiciInstrumentation`/`FetchInstrumentation`), with real OTLP
  exporters wired in (mirroring the backend's `repo_roller_api::tracing_otel`)
  Use when: Already wired globally — `initServerObservability()` is called once at module
  load in `hooks.server.ts` (reads `OTEL_TRACES_ENABLED`/`OTEL_EXPORTER_OTLP_ENDPOINT`/
  `OTEL_SERVICE_NAME` from `process.env` internally), `initClientObservability({ otlpEndpoint })`
  once at module load in `hooks.client.ts` (endpoint read from `$env/dynamic/public`'s
  `PUBLIC_OTEL_EXPORTER_OTLP_ENDPOINT` by the caller, since the browser cannot read
  `process.env`). No per-route code needed.
  Key exports: `init*Observability()` (idempotent, never throws — falls back silently if SDK
  registration fails), `is*ObservabilityInitialized()`, `_resetObservabilityForTesting()`,
  `buildServerSpanExporter(settings)` (gRPC `OTLPTraceExporter` from
  `@opentelemetry/exporter-trace-otlp-grpc` when `settings.enabled`, else
  `ConsoleSpanExporter`), `buildClientSpanExporter(otlpEndpoint)` (HTTP `OTLPTraceExporter`
  from `@opentelemetry/exporter-trace-otlp-http` when a non-empty endpoint is given, else
  `ConsoleSpanExporter`)
  Notes: `observability.client.ts` is browser-only and throws synchronously (before any OTel
  work) if `window` is undefined — never import it from `*.server.ts` code. Both
  `build*SpanExporter()` functions are fail-safe-off: exporter construction failure (e.g. a
  malformed endpoint) falls back to `ConsoleSpanExporter` rather than throwing — both now
  delegate that shared gate/try/fallback decision to
  `exporter-fallback.ts`'s `buildFailSafeSpanExporter()` (extracted during this task's
  REFACTOR pass) rather than each re-implementing it; see that entry below.

- **`frontend/src/lib/observability/exporter-fallback.ts`** — shared "fail-safe-off"
  span exporter selection logic, factored out (REFACTOR pass) of
  `observability.server.ts`'s `buildServerSpanExporter()` and `observability.client.ts`'s
  `buildClientSpanExporter()`, which both independently implemented the identical
  gate/try/fallback control flow despite constructing different concrete exporters
  (`sdk-trace-node` + gRPC vs. `sdk-trace-web` + HTTP)
  Use when: Building any `SpanExporter`-selection function that must be fail-safe-off
  (return a no-op exporter when export isn't enabled/configured, and ALSO fall back to
  the no-op exporter — never throw — if constructing the real exporter fails)
  Key exports: `buildFailSafeSpanExporter<E>(shouldExportReal, buildReal, buildFallback,
  onFallback?)` — generic over the exporter type so this module never imports either OTel
  SDK package itself; callers supply both constructors as closures and own their own
  imports
  Notes: Both current call sites explicitly instantiate the generic as
  `buildFailSafeSpanExporter<SpanExporter>(...)` — omitting the explicit type argument
  lets TypeScript infer `E` from whichever closure it checks first, which then fails to
  unify against the other closure's unrelated concrete exporter type. Verified before
  extraction that `@opentelemetry/sdk-trace-node` and `@opentelemetry/sdk-trace-web` both
  depend on the same `@opentelemetry/sdk-trace-base` version (pnpm dedupes to one copy),
  so `ConsoleSpanExporter`/`SpanExporter` instances remain `instanceof`-compatible across
  either import path — existing tests assert on this and needed no changes.

- **`frontend/src/lib/observability/otel-settings.ts`** — pure OTLP export configuration
  resolution, mirrors `repo_roller_api::tracing_otel::resolve_otel_settings`'s env-var
  semantics exactly (frontend and backend agree on what "OTel export enabled" means)
  Use when: Resolving `OTEL_TRACES_ENABLED`/`OTEL_EXPORTER_OTLP_ENDPOINT`/`OTEL_SERVICE_NAME`
  into settings for the Node-side exporter. Not used by the browser side directly — see
  `PUBLIC_OTEL_ENDPOINT_ENV_VAR` doc comment for why the browser's endpoint is a separate,
  `$env/dynamic/public`-sourced concern threaded through `initClientObservability()` instead.
  Key exports: `resolveOtelSettings(enabledRaw, endpointRaw, serviceNameRaw)` (pure, never
  throws, fail-safe-off default `enabled: false`), `OtelSettings` type,
  `DEFAULT_OTLP_ENDPOINT`/`DEFAULT_SERVICE_NAME`, `OTEL_ENABLED_ENV_VAR`/
  `OTEL_ENDPOINT_ENV_VAR`/`OTEL_SERVICE_NAME_ENV_VAR`/`PUBLIC_OTEL_ENDPOINT_ENV_VAR`
  Notes: Only the case-insensitive literals `"true"`/`"1"` enable export; every other value
  (including unset) resolves to disabled.

### Template Listing API Contract (Frontend Usability, #286)

- **`resolve_template_display_name`** — fn — `repo_roller_api::handlers` —
  Resolves a template's human-readable `display_name`, falling back to the
  template slug when the configured `template.name` is empty. Shared by
  `list_templates` and `get_template_details` (extracted during this task's
  REFACTOR pass — both handlers previously duplicated the identical
  `if config.template.name.is_empty() { slug } else { config.template.name }`
  block inline). Use when adding a third endpoint that surfaces a template's
  display name — call this rather than re-inlining the empty-check.
  Tags: templates, api, naming

- **`TemplateSummary`/`TemplateDetailsResponse` display_name/tags fields** —
  struct fields — `repo_roller_api::models::response` — Both response types
  now carry `name` (slug), `display_name` (human-readable, falls back to slug
  server-side via `resolve_template_display_name` above), and `tags: Vec<String>`
  (replacing the old single `category: Option<String>` field, which was always
  just `tags.first()`). Frontend's `frontend/src/lib/api/types.ts` `Template`/
  `TemplateDetails` interfaces and `frontend/src/lib/api/client.ts`'s
  `getTemplateDetails` mirror this shape 1:1, including the same slug-fallback
  rule for `displayName` as a defensive client-side re-check (not shared code —
  same concept implemented independently per language, same pattern as
  `HeaderMapExtractor`/`headersTextMapGetter` under Distributed Tracing above).
  Use when consuming or extending the templates-list/template-details API —
  don't resurrect `category`.
  Tags: templates, api, contract

- **Deferred (cross-scope, not touched by this task)**: `repo_roller_cli`'s
  `template_cmd.rs` (~line 291) and `org_settings_cmd.rs` (~lines 298, 350)
  each independently implement the same shape of "fall back to a default when
  a configured string is empty" (`metadata_repository_name` falling back to
  `DEFAULT_METADATA_REPOSITORY_NAME`), 3 occurrences total, structurally
  identical to the `display_name` pattern above but a different concept and a
  different crate. Not extracted here — doing so would require a shared
  helper living in a crate both `repo_roller_cli` and `repo_roller_api` depend
  on (e.g. `repo_roller_core`), which is an architecture decision out of this
  task's scope. Filed as tech debt — labels `tech-debt,refactor`.

### Frontend UI Components (Create-Wizard UX Follow-up)

- **`Spinner`** — Svelte component — `frontend/src/lib/components/Spinner.svelte` —
  Shared loading spinner, parameterized by `size` (`'sm' | 'md' | 'lg'`) and an optional
  `label`. When `label` is given, renders self-announcing (`role="status"
  aria-label={label}`); when omitted, renders `aria-hidden="true"` for nesting inside a
  parent-owned `aria-live` region. Extracted from `CreationOverlay.svelte`'s original inline
  `.creation-overlay__spinner` + `@keyframes spin` (this task's GREEN phase); now also consumed
  by `TemplateDetailsPanel.svelte` (`size="sm"`, no label, inside its own `role="status"
  aria-live="polite"` wrapper).
  Use when: Showing an indeterminate loading state anywhere in the frontend — call this rather
  than re-inlining a spinner `<div>`/`@keyframes spin` pair.
  Notes: A third, still-unconsolidated spinner implementation remains at
  `frontend/src/routes/auth/callback/+page.svelte` (structurally identical to this component's
  `size="md"` variant) — deferred as cross-scope tech debt, see
  `.llm/findings/create-wizard-ux-followup.md`.
  Tags: frontend, svelte, loading, a11y

- **`TagList`** — Svelte component — `frontend/src/lib/components/TagList.svelte` — Shared
  tag-chip list (`<ul aria-label={ariaLabel}><li>` per tag), with an optional `max` cap and a
  `topGap` modifier (adds a top margin for block-layout callers that don't already manage
  spacing via a flex `gap`). Renders nothing when there are no tags (post-cap).
  Use when: Rendering any list of plain string tags/labels as chips — call this rather than
  reimplementing the `.template-card__tags`/`.template-card__tag` markup+CSS.
  Impls/call sites: `TemplateCard.svelte` (`max={4}`, `topGap`, CMP-004's original cap),
  `TemplateDetailsPanel.svelte` (uncapped, no gap — parent manages spacing via flex `gap`),
  `RepositorySummary.svelte` (uncapped, no gap — CMP-009's Review-step tags/variables props,
  this task's GREEN phase).
  Notes: Extracted (REFACTOR pass) from markup+CSS that had been independently duplicated three
  times. Deliberately keeps the `template-card__tags`/`template-card__tag` class names (not a
  neutral `tag-list` name) because `TemplateGrid.test.ts`/`TemplateCard.test.ts` (frozen
  RED-phase contracts, out of this refactor's scope to edit) assert against those literal
  strings via `querySelector` — renaming would make `TemplateGrid.test.ts`'s "renders no tags
  for an empty array" assertion vacuously pass instead of actually verifying absence. Also
  fixes a latent bug: `TemplateDetailsPanel.svelte` had copy-pasted the `<li
  class="template-card__tag">` markup without importing `TemplateCard`, which rendered
  completely unstyled — Svelte scopes a component's `<style>` block to markup that component
  itself renders, so a class literal borrowed from another component's stylesheet never
  actually matches. Routing every caller through this shared component (which owns its own
  `<style>` block) fixes that for all three call sites.
  Tags: frontend, svelte, chips, dry

- **`RepositoryTypeBadge`** — Svelte component —
  `frontend/src/lib/components/RepositoryTypeBadge.svelte` — Shared repository-type badge
  (`data-policy` attribute + visible `typeName` text), with a `topGap` modifier (see `TagList`
  above for the same spacing rationale). `policy`/`typeName` are typed as plain
  `string`/`string | null | undefined` (wider than TemplateCard's own
  `'fixed' | 'preferable' | 'optional'` union) since `TemplateDetailsPanel` feeds it directly
  from `GetTemplateDetailsResponse`'s looser wire-shaped `RepositoryTypePolicy` type; this is a
  DOM `data-*` attribute, not a styling switch, so an unrecognised value is harmless.
  Use when: Rendering the repository-type policy badge — call this rather than reimplementing
  the `.template-card__type-badge` markup+CSS.
  Impls/call sites: `TemplateCard.svelte` (`topGap`, CMP-004's original badge),
  `TemplateDetailsPanel.svelte` (no gap — parent manages spacing via flex `gap`).
  Notes: Extracted (REFACTOR pass) for the same reasons as `TagList` above — same class-name
  preservation rationale (`TemplateCard.test.ts`/`TemplateDetailsPanel.test.ts` assert on the
  literal `.template-card__type-badge` string) and the same previously-unstyled-in-
  `TemplateDetailsPanel` bug fix.
  Tags: frontend, svelte, badge, dry

- **`TemplateDetailsPanel`** — Svelte component —
  `frontend/src/lib/components/TemplateDetailsPanel.svelte` — Step-1 sidebar showing full
  details (description, tags via `TagList`, type badge via `RepositoryTypeBadge`, variables) for
  the currently-selected template, across four states (empty/loading/error/content). New
  component, this task's GREEN phase; its own reusable value was exactly the two patterns above
  (now factored out into `TagList`/`RepositoryTypeBadge`), so no further abstraction extraction
  was needed from the panel itself.
  Notes: Paired with the `create/+page.svelte` wizard's `.wizard--step1` CSS modifier (widens the
  wizard to 80rem and switches to a two-column flex layout ≥60rem, for this sidebar to sit
  beside `TemplateGrid`) and the wizard's new final "Review" step (always appended after
  Settings/Variables; owns the sole "Create Repository" action). Both are single-use,
  route-local conventions (not extracted as separate reusable abstractions) — recorded here only
  for discoverability.
  Tags: frontend, svelte, sidebar, create-wizard

## Cross-Cutting Helpers

- **Logging**: `tracing` crate throughout
  Usage: `use tracing::{info, warn, error, debug};`
  Pattern: Structured logging with spans for request tracking
  Notes: Never log tokens or secrets (use `?` for Debug, not secrets). See
  "Logging & Correlation (Observability Task 1)" above for the API binary's JSON
  format selection, sanitization layer, and correlation-ID span pattern.

- **Error handling**: `thiserror` for error derivation
  Pattern: `#[derive(Error, Debug)] enum MyError { ... }`
  Usage: All public APIs return `Result<T, E>`, no panic in production

- **Configuration loading**: `config_manager::ConfigurationManager`
  Usage: Inject as dependency, loads from GitHub .reporoller/ repos
  Pattern: Hierarchical resolution with caching

- **Testing utilities**: `test_utils` crate
  Usage: Mock implementations of `GitHubClient`, `ConfigurationProvider`, etc.
  Pattern: `MockGitHubClient::new()` for unit tests

- **Authentication**: `auth_handler::GitHubAuthService`
  Usage: GitHub App authentication with installation tokens
  Pattern: Load credentials → Generate JWT → Get installation token → API calls

## Where to Add New Stuff

- **Reusable domain logic**: Add to `repo_roller_core/src/`
  Pattern: Define port trait in core, implement in adapter crate

- **GitHub API operations**: Add to `github_client/src/`
  Pattern: Implement trait defined in repo_roller_core

- **Configuration types**: Add to `config_manager/src/`
  Pattern: TOML-deserializable structs with validation

- **Template helpers**: Add to `template_engine/src/`
  Pattern: Handlebars helper functions registered in engine

- **CLI commands**: Add to `repo_roller_cli/src/commands/`
  Pattern: Clap subcommand with translation to domain types

- **API endpoints**: Add to `repo_roller_api/src/handlers/`
  Pattern: Axum handler with JSON request/response translation

- **Test utilities**: Add to `test_utils/src/`
  Pattern: Mock implementations, test builders, fixtures

- **Experimental**: Create new branch, mark with TODO
  Pattern: Must not be depended on by production code

## Port Traits (Hexagonal Architecture)

Core business logic depends on these trait abstractions (not concrete implementations):

- **`GitHubOperations`** — GitHub API operations
  Implemented by: `github_client::GitHubClient`

- **`ConfigurationProvider`** — Configuration loading
  Implemented by: `config_manager::ConfigurationManager`

- **`TemplateProcessor`** — Template processing
  Implemented by: `template_engine::HandlebarsTemplateEngine`

- **`VisibilityPolicyProvider`** — Visibility policy resolution
  Implemented by: `config_manager` (policy provider)

- **`GitHubEnvironmentDetector`** — GitHub plan detection
  Implemented by: `github_client` (environment detector)

See `docs/adr/ADR-001-hexagonal-architecture.md` for pattern details.

## Search Keywords

`logging`, `config`, `github-api`, `cache`, `retry`, `auth`, `metrics`, `tracing`, `cli`, `validation`, `error-handling`, `templates`, `visibility`, `testing`, `mocks`, `fixtures`
