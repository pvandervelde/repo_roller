//! Span-hierarchy contract for `create_repository`'s orchestration phases.
//!
//! This file (together with its test siblings `observability_tests.rs` and
//! `observability_proptests.rs`) defines the informal interface contract for
//! Task #2 ("Distributed tracing: OpenTelemetry + OTLP"), mirroring the
//! precedent set by `repo_roller_api::logging` for Task #1.
//!
//! ## Contract
//!
//! Per `docs/spec/operations/observability.md` § "Distributed Tracing" ->
//! "Span Organization" and "Key Trace Points" -> "Repository Creation Flow",
//! `create_repository` must open one top-level span
//! ([`SPAN_REPOSITORY_CREATION`]) for the whole operation, with the
//! following phase-boundary spans nested directly beneath it (siblings of
//! each other, all children of the top-level span) in call order:
//!
//! 1. [`SPAN_AUTHENTICATION`] -- GitHub App installation authentication and
//!    GitHub client construction (`setup_github_clients`, Steps 1-2).
//! 2. [`SPAN_CONFIGURATION_RESOLUTION`] -- organization configuration
//!    hierarchy resolution and template-config loading (`load_creation_config`,
//!    Steps 3-4).
//! 3. [`SPAN_TEMPLATE_PROCESSING`] -- template content retrieval, variable
//!    substitution, and local repository content generation
//!    (`generate_repository_content`, Step 6).
//! 4. [`SPAN_GITHUB_REPOSITORY_CREATION`] -- the GitHub repository-creation
//!    API call and its immediate configuration (`create_github_repository`,
//!    Step 8).
//!
//! Each constant is the literal `tracing` span *name* the Coder's
//! instrumentation (whether via `#[instrument(name = ...)]` or an explicit
//! `tracing::info_span!(...)`) must produce -- tests reference these
//! constants symbolically rather than hard-coding the literal string, so a
//! future rename only requires updating one definition.
//!
//! [`business_context_attributes`] is the single, bounded, sanitized set of
//! business-context key/value pairs (organization, template, requestor) the
//! Coder must attach to [`SPAN_REPOSITORY_CREATION`] as span fields/
//! attributes. See its own doc comment for the exact contract; it delegates
//! all value sanitization to
//! [`github_client::span_attributes::sanitize_span_attribute_value`] so that
//! this crate never re-implements the secret-shape-rejection / length-
//! bounding logic independently (`repo_roller_core` already depends on
//! `github_client`, so there is no "shared low-level dependency" tradeoff to
//! make here -- see `github_client::span_attributes`'s module docs for why
//! that crate is the canonical home).
//!
//! ## Known spec gaps / testability finding (flagged for the architect / Tech Lead)
//!
//! `configuration::resolve_organization_configuration` (called from
//! `load_creation_config`, which backs [`SPAN_CONFIGURATION_RESOLUTION`])
//! constructs its own `GitHubClient` directly from the raw installation
//! token via `github_client::create_token_client`, with **no way to
//! override the GitHub API base URL**. This means `create_repository`
//! cannot be driven past the authentication phase in a pure unit test
//! without hitting the real `api.github.com` -- there is currently no
//! wiremock-based path to a deterministic, network-free test of the
//! `configuration_resolution` -> `template_processing` ->
//! `github_repository_creation` span chain at this crate's unit-test tier.
//! (The existing `crates/integration_tests` crate exercises the full chain,
//! but against the *real* GitHub API with real App credentials -- true E2E,
//! not suitable for a deterministic RED/GREEN unit-test loop.) This gap
//! predates Task #2 and is not something a Tester should fix by adding
//! production-code plumbing; flagged here so the Coder/architect can decide
//! whether to thread a base-URL override through
//! `resolve_organization_configuration` (mirroring
//! `repo_roller_api::AppState::github_api_base_url`) for future
//! testability. Until then, `lib_span_tests.rs` in this crate covers only
//! the [`SPAN_AUTHENTICATION`] boundary (the one phase transition reachable
//! without real network access); the QA Engineer's audit phase should rely
//! on `crates/integration_tests` (gated, credentialed) for confidence on the
//! deeper phase spans.

/// Canonical span name for the top-level `create_repository` orchestration.
///
/// Opened once per call and closed only when the whole operation --
/// including post-creation settings application -- completes or fails.
pub const SPAN_REPOSITORY_CREATION: &str = "repository_creation";

/// Canonical span name for GitHub App installation authentication and
/// GitHub client construction (`setup_github_clients`).
pub const SPAN_AUTHENTICATION: &str = "authentication";

/// Canonical span name for organization configuration hierarchy resolution
/// and template-config loading (`load_creation_config`).
pub const SPAN_CONFIGURATION_RESOLUTION: &str = "configuration_resolution";

/// Canonical span name for template content retrieval, variable
/// substitution, and local repository content generation
/// (`generate_repository_content`).
pub const SPAN_TEMPLATE_PROCESSING: &str = "template_processing";

/// Canonical span name for the GitHub repository-creation API call
/// (`create_github_repository`).
pub const SPAN_GITHUB_REPOSITORY_CREATION: &str = "github_repository_creation";

/// Key used for the sanitized organization attribute on
/// [`SPAN_REPOSITORY_CREATION`].
pub const ATTR_ORGANIZATION: &str = "organization";

/// Key used for the sanitized template attribute on
/// [`SPAN_REPOSITORY_CREATION`]. Only present when the request specifies a
/// template (see [`business_context_attributes`]'s contract).
pub const ATTR_TEMPLATE: &str = "template";

/// Key used for the sanitized requestor attribute on
/// [`SPAN_REPOSITORY_CREATION`].
pub const ATTR_REQUESTOR: &str = "requestor";

/// Builds the bounded, sanitized set of business-context span attributes for
/// [`SPAN_REPOSITORY_CREATION`] from a repository-creation request's raw,
/// potentially attacker-influenced fields.
///
/// # Contract
/// - Every value is passed through
///   [`github_client::span_attributes::sanitize_span_attribute_value`]
///   before being included -- callers must never attach `organization`,
///   `template`, or `requestor` to a span using the raw, unsanitized string.
/// - Always includes [`ATTR_ORGANIZATION`] and [`ATTR_REQUESTOR`] (both are
///   mandatory fields on every repository-creation request).
/// - Includes [`ATTR_TEMPLATE`] **only** when `template` is `Some` --
///   omitted entirely (not an empty string) when the request has no
///   template (empty-repository / custom-init creations).
/// - Never panics on any input, including empty strings or pathologically
///   long/adversarial values (delegates all safety guarantees to
///   `sanitize_span_attribute_value`).
/// - The returned `Vec`'s key order is stable: `organization`, then
///   `template` (if present), then `requestor` -- tests may rely on this
///   order rather than searching unordered.
pub fn business_context_attributes(
    organization: &str,
    template: Option<&str>,
    requestor: &str,
) -> Vec<(&'static str, String)> {
    use github_client::span_attributes::sanitize_span_attribute_value;

    let mut attributes = Vec::with_capacity(3);
    attributes.push((ATTR_ORGANIZATION, sanitize_span_attribute_value(organization)));
    if let Some(template_name) = template {
        attributes.push((ATTR_TEMPLATE, sanitize_span_attribute_value(template_name)));
    }
    attributes.push((ATTR_REQUESTOR, sanitize_span_attribute_value(requestor)));
    attributes
}

#[cfg(test)]
#[path = "observability_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "observability_proptests.rs"]
mod proptests;
