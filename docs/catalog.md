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

- **`frontend/src/lib/observability/spans.ts`** — span-wrapping helper for instrumenting
  discrete user operations (form submissions, data loads)
  Use when: Instrumenting a user-facing async operation (API call triggered by a component
  action) with a named span
  Key exports: `withSpan(name, fn, attributes?)` (starts a span, runs `fn` with it active,
  ends it exactly once on every path, rethrows `fn`'s error unchanged), `SPAN_NAMES`
  (`CREATE_REPOSITORY_SUBMIT`, `LOAD_TEMPLATES` — add new operation names here, never inline
  a span-name string literal at a call site), `SpanHandle` type
  Notes: Already wired into `routes/create/+page.svelte`'s `loadTemplates()` (wraps
  `listTemplates()`) and `submitCreate()` (wraps `createRepository()`).

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
  malformed endpoint) falls back to `ConsoleSpanExporter` rather than throwing.

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
