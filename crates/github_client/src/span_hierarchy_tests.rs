//! Span-hierarchy tests for `github_client`'s outbound GitHub API calls.
//!
//! Task #2 (distributed tracing) acceptance criterion: "GitHub API calls made
//! during repository creation appear as child spans of the repository-
//! creation trace, not as untraced/orphaned calls." This file verifies that
//! contract directly against `github_client`'s own instrumentation, using a
//! minimal in-process `tracing_subscriber::Layer` (`SpanRecorder`) that
//! records every span's name and immediate parent -- no OpenTelemetry
//! exporter or OTLP collector required, matching the acceptance criteria's
//! instruction not to require a live collector for the test suite to pass.
//!
//! Two categories of test live here:
//! - **Regression guards** (already-instrumented methods): these currently
//!   pass, locking in today's correct behaviour so a future refactor cannot
//!   silently orphan these spans from their parent.
//! - **RED tests** (methods used by `repo_roller_core::create_repository`'s
//!   "GitHub repository creation" phase that are NOT YET instrumented, e.g.
//!   `create_org_repository` and `get_organization_default_branch`): these
//!   fail today and must start passing once Task #2 adds `#[instrument]` to
//!   these methods.

use super::*;
use std::sync::{Arc, Mutex};
use tracing::span::{Attributes, Id};
use tracing::Instrument;
use tracing_subscriber::{layer::Context, layer::SubscriberExt, registry::LookupSpan, Layer};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[ctor::ctor(unsafe)]
fn init_default_crypto_provider() {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
}

/// One recorded span: its `tracing`-level name and the name of its
/// immediate parent span, if any, at the moment it was created.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RecordedSpan {
    name: &'static str,
    parent_name: Option<String>,
    field_names: Vec<String>,
}

/// A `tracing_subscriber::Layer` that records every span's name, immediate
/// parent, and recorded field names into a shared, thread-safe buffer.
///
/// This is real, working test infrastructure (not a stub under test) -- it
/// exists purely to make span creation and parent/child nesting observable
/// from test code without depending on a real OpenTelemetry exporter.
#[derive(Clone, Default)]
struct SpanRecorder {
    spans: Arc<Mutex<Vec<RecordedSpan>>>,
}

impl SpanRecorder {
    fn recorded(&self) -> Vec<RecordedSpan> {
        self.spans
            .lock()
            .expect("span recorder mutex poisoned")
            .clone()
    }
}

/// Visitor that collects only the field *names* seen on a span (never the
/// values -- this harness must never itself become a place where a secret
/// value is retained, even in test code).
struct FieldNameCollector(Vec<String>);

impl tracing::field::Visit for FieldNameCollector {
    fn record_debug(&mut self, field: &tracing::field::Field, _value: &dyn std::fmt::Debug) {
        self.0.push(field.name().to_string());
    }
}

impl<S> Layer<S> for SpanRecorder
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let span_ref = ctx
            .span(id)
            .expect("span must exist in registry for on_new_span");
        let parent_name = span_ref.parent().map(|p| p.name().to_string());

        let mut collector = FieldNameCollector(Vec::new());
        attrs.record(&mut collector);

        self.spans
            .lock()
            .expect("span recorder mutex poisoned")
            .push(RecordedSpan {
                name: span_ref.name(),
                parent_name,
                field_names: collector.0,
            });
    }
}

/// Builds a `GitHubClient` pointed at `mock_server`, bypassing real GitHub
/// App JWT/installation-token flows entirely (mirrors the construction
/// pattern already used by `lib_tests.rs`'s `test_create_org_repository_success`).
fn mock_client(mock_server: &MockServer) -> GitHubClient {
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .build()
        .unwrap();
    GitHubClient { client: octocrab }
}

// ---------------------------------------------------------------------
// Regression guards: already-instrumented methods must remain correctly
// parented under an ambient span.
// ---------------------------------------------------------------------

#[tokio::test(flavor = "current_thread")]
async fn test_get_repository_span_is_child_of_ambient_repository_creation_span() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/repos/test-org/test-repo"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "name": "test-repo",
            "full_name": "test-org/test-repo",
            "node_id": "MDEwOlJlcG9zaXRvcnkx",
            "private": false,
        })))
        .mount(&mock_server)
        .await;

    let recorder = SpanRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    async {
        let _ = client.get_repository("test-org", "test-repo").await;
    }
    .instrument(tracing::info_span!("repository_creation"))
    .await;

    let spans = recorder.recorded();
    let get_repo_span = spans
        .iter()
        .find(|s| s.name == "get_repository")
        .expect("get_repository must create a span (already #[instrument]-ed today)");
    assert_eq!(
        get_repo_span.parent_name.as_deref(),
        Some("repository_creation"),
        "get_repository's span must nest under the ambient repository_creation span, \
         recorded spans: {spans:?}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_get_installation_token_for_org_span_does_not_include_a_token_field() {
    // This is a regression guard against a future engineer accidentally
    // widening `#[instrument(fields(...))]` to include the raw token: the
    // method's *return value* is a secret, but its span attributes today
    // only ever include `org_name` -- verify that stays true.
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/app/installations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&mock_server)
        .await;

    let recorder = SpanRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let _ = client.get_installation_token_for_org("test-org").await;

    let spans = recorder.recorded();
    let token_span = spans
        .iter()
        .find(|s| s.name == "get_installation_token_for_org")
        .expect(
            "get_installation_token_for_org must create a span (already #[instrument]-ed today)",
        );
    assert!(
        !token_span
            .field_names
            .iter()
            .any(|f| f.to_lowercase().contains("token")),
        "installation-token span must never carry a field whose name contains \
         'token', recorded fields: {:?}",
        token_span.field_names
    );
}

// ---------------------------------------------------------------------
// RED tests: methods on the "GitHub repository creation" phase that are
// not yet instrumented at all.
// ---------------------------------------------------------------------

#[tokio::test(flavor = "current_thread")]
async fn test_create_org_repository_span_is_child_of_ambient_repository_creation_span() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/orgs/test-org/repos"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 123456,
            "name": "test-repo",
            "node_id": "MDEwOlJlcG9zaXRvcnkx",
            "url": "https://api.github.com/repos/test-org/test-repo",
        })))
        .mount(&mock_server)
        .await;

    let recorder = SpanRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let payload = RepositoryCreatePayload {
        name: "test-repo".to_string(),
        ..Default::default()
    };
    async {
        let _: Result<_, _> = client.create_org_repository("test-org", &payload).await;
    }
    .instrument(tracing::info_span!("repository_creation"))
    .await;

    let spans = recorder.recorded();
    let create_repo_span = spans
        .iter()
        .find(|s| s.name == "create_org_repository")
        .expect(
            "create_org_repository must create its own span once Task #2 instruments the \
         GitHub-repository-creation phase (currently un-instrumented -- this is the RED \
         signal this test is designed to produce)",
        );
    assert_eq!(
        create_repo_span.parent_name.as_deref(),
        Some("repository_creation"),
        "create_org_repository's span must nest under the ambient repository_creation span, \
         recorded spans: {spans:?}"
    );
}

// ---------------------------------------------------------------------
// RED tests: HIGH #1 security-remediation -- `path` span attribute
// sanitization on `list_directory_contents` and `get_file_content`.
//
// `path` is a file/directory path inside a *template* repository -- an
// explicitly attacker-influenceable component per
// `docs/spec/security/threat-model.md` (an attacker who controls a template
// repository can name a file/directory anything, including secret-shaped
// content). Both methods currently attach `path = %path` directly via
// `#[instrument(fields(...))]` with no call through
// `span_attributes::sanitize_span_attribute_value`, so a secret-shaped or
// pathologically long path reaches the exported OTLP span unmodified.
//
// Verifying redaction/truncation requires observing the recorded *value*,
// not just the field's name, so this section uses a separate
// `FieldValueCollector`/`SpanValueRecorder` (distinct from the name-only
// `FieldNameCollector`/`SpanRecorder` above). Capturing values here is safe:
// every "secret" used below is a test-crafted fixture, never a real
// credential -- the entire point of these tests is to prove such values
// never reach a span attribute unredacted.
// ---------------------------------------------------------------------

/// Visitor that collects every span field's name AND formatted value.
/// Deliberately separate from `FieldNameCollector` above -- see the section
/// doc comment for why capturing values is safe and necessary here.
struct FieldValueCollector(std::collections::HashMap<String, String>);

impl tracing::field::Visit for FieldValueCollector {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.0
            .insert(field.name().to_string(), format!("{value:?}"));
    }
}

/// One recorded span's name plus its field name/value map.
#[derive(Debug, Clone)]
struct RecordedSpanWithValues {
    name: &'static str,
    fields: std::collections::HashMap<String, String>,
}

/// A `tracing_subscriber::Layer` that records every span's name and field
/// *values* into a shared, thread-safe buffer.
#[derive(Clone, Default)]
struct SpanValueRecorder {
    spans: Arc<Mutex<Vec<RecordedSpanWithValues>>>,
}

impl SpanValueRecorder {
    fn recorded(&self) -> Vec<RecordedSpanWithValues> {
        self.spans
            .lock()
            .expect("span value recorder mutex poisoned")
            .clone()
    }
}

impl<S> Layer<S> for SpanValueRecorder
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let span_ref = ctx
            .span(id)
            .expect("span must exist in registry for on_new_span");
        let mut collector = FieldValueCollector(std::collections::HashMap::new());
        attrs.record(&mut collector);
        self.spans
            .lock()
            .expect("span value recorder mutex poisoned")
            .push(RecordedSpanWithValues {
                name: span_ref.name(),
                fields: collector.0,
            });
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_list_directory_contents_span_path_field_is_redacted_when_secret_shaped() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let secret_shaped_path = format!("ghp_{}.txt", "a".repeat(24));
    let _ = client
        .list_directory_contents("test-org", "test-repo", &secret_shaped_path, "main")
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "list_directory_contents")
        .expect("list_directory_contents must create a span");
    let recorded_path = span
        .fields
        .get("path")
        .expect("list_directory_contents span must record a `path` field");
    assert_eq!(
        recorded_path,
        crate::span_attributes::SPAN_ATTRIBUTE_REDACTED_MARKER,
        "a GitHub-token-shaped path must be fully redacted before being attached as a span \
         attribute (per span_attributes::sanitize_span_attribute_value's contract), \
         recorded value: {recorded_path:?}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_list_directory_contents_span_path_field_is_truncated_when_over_max_len() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let long_path = "p".repeat(200);
    let _ = client
        .list_directory_contents("test-org", "test-repo", &long_path, "main")
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "list_directory_contents")
        .expect("list_directory_contents must create a span");
    let recorded_path = span
        .fields
        .get("path")
        .expect("list_directory_contents span must record a `path` field");
    assert!(
        recorded_path.chars().count() <= crate::span_attributes::MAX_SPAN_ATTRIBUTE_LEN,
        "path span attribute must be bounded to at most {} chars per \
         span_attributes::sanitize_span_attribute_value's contract, but recorded value has \
         {} chars: {recorded_path:?}",
        crate::span_attributes::MAX_SPAN_ATTRIBUTE_LEN,
        recorded_path.chars().count()
    );
    assert_ne!(
        recorded_path.chars().count(),
        long_path.chars().count(),
        "a 200-char path must actually be truncated (not merely happen to already fit) -- \
         recorded value: {recorded_path:?}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_list_directory_contents_span_path_field_short_benign_path_passes_through_unchanged() {
    // Stub-killing: guards against a lazy "always redact" fix that would
    // replace every path value with the marker regardless of content.
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let benign_path = "src/main.rs";
    let _ = client
        .list_directory_contents("test-org", "test-repo", benign_path, "main")
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "list_directory_contents")
        .expect("list_directory_contents must create a span");
    let recorded_path = span
        .fields
        .get("path")
        .expect("list_directory_contents span must record a `path` field");
    assert_eq!(
        recorded_path, benign_path,
        "an ordinary, short, non-secret-shaped path must pass through sanitization unchanged, \
         recorded value: {recorded_path:?}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_get_file_content_span_path_field_is_redacted_when_secret_shaped() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "name": "file",
            "path": "file",
            "type": "file",
            "sha": "abc123",
            "size": 5,
            "download_url": "https://raw.githubusercontent.com/test-org/test-repo/main/file",
            "content": "aGVsbG8="
        })))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let secret_shaped_path = format!("ghp_{}.txt", "a".repeat(24));
    let _ = client
        .get_file_content("test-org", "test-repo", &secret_shaped_path)
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "get_file_content")
        .expect("get_file_content must create a span");
    let recorded_path = span
        .fields
        .get("path")
        .expect("get_file_content span must record a `path` field");
    assert_eq!(
        recorded_path,
        crate::span_attributes::SPAN_ATTRIBUTE_REDACTED_MARKER,
        "a GitHub-token-shaped path must be fully redacted before being attached as a span \
         attribute (per span_attributes::sanitize_span_attribute_value's contract), \
         recorded value: {recorded_path:?}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_get_file_content_span_path_field_is_truncated_when_over_max_len() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "name": "file",
            "path": "file",
            "type": "file",
            "sha": "abc123",
            "size": 5,
            "download_url": "https://raw.githubusercontent.com/test-org/test-repo/main/file",
            "content": "aGVsbG8="
        })))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let long_path = "p".repeat(200);
    let _ = client
        .get_file_content("test-org", "test-repo", &long_path)
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "get_file_content")
        .expect("get_file_content must create a span");
    let recorded_path = span
        .fields
        .get("path")
        .expect("get_file_content span must record a `path` field");
    assert!(
        recorded_path.chars().count() <= crate::span_attributes::MAX_SPAN_ATTRIBUTE_LEN,
        "path span attribute must be bounded to at most {} chars per \
         span_attributes::sanitize_span_attribute_value's contract, but recorded value has \
         {} chars: {recorded_path:?}",
        crate::span_attributes::MAX_SPAN_ATTRIBUTE_LEN,
        recorded_path.chars().count()
    );
    assert_ne!(
        recorded_path.chars().count(),
        long_path.chars().count(),
        "a 200-char path must actually be truncated (not merely happen to already fit) -- \
         recorded value: {recorded_path:?}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_get_file_content_span_path_field_short_benign_path_passes_through_unchanged() {
    // Stub-killing: guards against a lazy "always redact" fix.
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "name": "file",
            "path": "file",
            "type": "file",
            "sha": "abc123",
            "size": 5,
            "download_url": "https://raw.githubusercontent.com/test-org/test-repo/main/file",
            "content": "aGVsbG8="
        })))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let benign_path = "src/main.rs";
    let _ = client
        .get_file_content("test-org", "test-repo", benign_path)
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "get_file_content")
        .expect("get_file_content must create a span");
    let recorded_path = span
        .fields
        .get("path")
        .expect("get_file_content span must record a `path` field");
    assert_eq!(
        recorded_path, benign_path,
        "an ordinary, short, non-secret-shaped path must pass through sanitization unchanged, \
         recorded value: {recorded_path:?}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_get_organization_default_branch_span_is_child_of_ambient_repository_creation_span() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/orgs/test-org"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "login": "test-org",
        })))
        .mount(&mock_server)
        .await;

    let recorder = SpanRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    async {
        let _ = client.get_organization_default_branch("test-org").await;
    }
    .instrument(tracing::info_span!("repository_creation"))
    .await;

    let spans = recorder.recorded();
    let branch_span = spans
        .iter()
        .find(|s| s.name == "get_organization_default_branch")
        .expect(
            "get_organization_default_branch must create its own span once Task #2 \
             instruments it (currently un-instrumented -- this is the RED signal this \
             test is designed to produce)",
        );
    assert_eq!(
        branch_span.parent_name.as_deref(),
        Some("repository_creation"),
        "get_organization_default_branch's span must nest under the ambient \
         repository_creation span, recorded spans: {spans:?}"
    );
}

// ---------------------------------------------------------------------
// RED tests: remaining unsanitized span-attribute sites (Security Reviewer
// follow-up pass on the ~21 `#[instrument(fields(...))]` sites left after
// the original 4 HIGH-finding fixes above).
//
// The Security Reviewer spot-checked these sites and rated them lower-risk
// than the original `path` findings (GitHub-validated identifiers like org
// names, repo names, team slugs -- not arbitrary free text), but
// recommended fixing them anyway since the `field = %value` ->
// `field = %span_attributes::sanitize_span_attribute_value(value)` pattern
// is proven and mechanically cheap. The user approved fixing all of them.
//
// This section gives REAL, value-observing RED-test coverage to a
// representative subset chosen for genuinely elevated (or at-least-equal)
// risk versus the fields the Security Reviewer characterised as lower-risk:
//   - `get_installation_token_for_org`'s `org_name` -- identical
//     field/risk category to the already-fixed `create_org_repository`/
//     `get_organization_default_branch` `org_name` sites above; this one
//     was simply missed in the earlier remediation pass.
//   - `search_repositories_by_topic`'s `topic` -- a free-text search
//     term, not a GitHub-round-tripped identifier (unlike org/repo names).
//   - `get_collaborator_permission`/`add_repository_collaborator`'s
//     `username` -- typically sourced from user-maintained repository
//     configuration (collaborator lists), not validated by GitHub before
//     this call is made.
//
// The remaining ~11 github_client sites (`owner`/`repo`/`org`/
// `team_slug`/`permission`/`repo_owner` fields on `get_repository`,
// `list_organization_teams`, `get_team_members`,
// `add_team_to_repository`, `set_team_repository_permission`,
// `get_team_repository_permission`, `get_repository_team_permission`,
// `list_repository_collaborators`, `set_collaborator_permission`,
// `remove_repository_collaborator`, and `update_repository_settings`) are
// fixed by the exact same mechanical edit, verified by code review plus
// `span_attributes.rs`'s own exhaustive unit/property-test coverage of the
// sanitizer entry point itself (secret-shape rejection, truncation,
// idempotency, total-function/no-panic guarantees) -- not by a dedicated
// call-site test here. Driving all ~11 remaining methods through wiremock
// individually would mean ~11 near-duplicate fixture/mock setups
// re-testing the exact same `sanitize_span_attribute_value` entry point
// already proven correct above, in `span_attributes_tests.rs`, and in
// `span_attributes_proptests.rs`.
// ---------------------------------------------------------------------

#[tokio::test(flavor = "current_thread")]
async fn test_get_installation_token_for_org_span_org_name_field_is_redacted_when_secret_shaped() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/app/installations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let secret_shaped_org = format!("ghp_{}", "a".repeat(24));
    let _ = client
        .get_installation_token_for_org(&secret_shaped_org)
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "get_installation_token_for_org")
        .expect("get_installation_token_for_org must create a span");
    let recorded_org_name = span
        .fields
        .get("org_name")
        .expect("get_installation_token_for_org span must record an `org_name` field");
    assert_eq!(
        recorded_org_name,
        crate::span_attributes::SPAN_ATTRIBUTE_REDACTED_MARKER,
        "a GitHub-token-shaped org_name must be fully redacted before being attached as a \
         span attribute, recorded value: {recorded_org_name:?}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_get_installation_token_for_org_span_org_name_field_short_benign_value_passes_through_unchanged(
) {
    // Stub-killing: guards against a lazy "always redact" fix.
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/app/installations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let benign_org = "my-real-org";
    let _ = client.get_installation_token_for_org(benign_org).await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "get_installation_token_for_org")
        .expect("get_installation_token_for_org must create a span");
    let recorded_org_name = span
        .fields
        .get("org_name")
        .expect("get_installation_token_for_org span must record an `org_name` field");
    assert_eq!(
        recorded_org_name, benign_org,
        "an ordinary, short, non-secret-shaped org_name must pass through sanitization \
         unchanged, recorded value: {recorded_org_name:?}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_search_repositories_by_topic_span_topic_field_is_redacted_when_secret_shaped() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/search/repositories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "total_count": 0,
            "incomplete_results": false,
            "items": []
        })))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let secret_shaped_topic = format!("ghp_{}", "b".repeat(24));
    let _ = client
        .search_repositories_by_topic("test-org", &secret_shaped_topic)
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "search_repositories_by_topic")
        .expect("search_repositories_by_topic must create a span");
    let recorded_topic = span
        .fields
        .get("topic")
        .expect("search_repositories_by_topic span must record a `topic` field");
    assert_eq!(
        recorded_topic,
        crate::span_attributes::SPAN_ATTRIBUTE_REDACTED_MARKER,
        "a GitHub-token-shaped topic must be fully redacted before being attached as a span \
         attribute, recorded value: {recorded_topic:?}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_search_repositories_by_topic_span_topic_field_short_benign_value_passes_through_unchanged(
) {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/search/repositories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "total_count": 0,
            "incomplete_results": false,
            "items": []
        })))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let benign_topic = "reporoller-metadata";
    let _ = client
        .search_repositories_by_topic("test-org", benign_topic)
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "search_repositories_by_topic")
        .expect("search_repositories_by_topic must create a span");
    let recorded_topic = span
        .fields
        .get("topic")
        .expect("search_repositories_by_topic span must record a `topic` field");
    assert_eq!(
        recorded_topic, benign_topic,
        "an ordinary, short, non-secret-shaped topic must pass through sanitization \
         unchanged, recorded value: {recorded_topic:?}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_get_collaborator_permission_span_username_field_is_redacted_when_secret_shaped() {
    let mock_server = MockServer::start().await;
    let secret_shaped_username = format!("ghp_{}", "c".repeat(24));
    Mock::given(method("GET"))
        .and(path(format!(
            "/repos/test-org/test-repo/collaborators/{secret_shaped_username}/permission"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "role_name": "write"
        })))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let _ = client
        .get_collaborator_permission("test-org", "test-repo", &secret_shaped_username)
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "get_collaborator_permission")
        .expect("get_collaborator_permission must create a span");
    let recorded_username = span
        .fields
        .get("username")
        .expect("get_collaborator_permission span must record a `username` field");
    assert_eq!(
        recorded_username,
        crate::span_attributes::SPAN_ATTRIBUTE_REDACTED_MARKER,
        "a GitHub-token-shaped username must be fully redacted before being attached as a \
         span attribute, recorded value: {recorded_username:?}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_get_collaborator_permission_span_username_field_short_benign_value_passes_through_unchanged(
) {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/repos/test-org/test-repo/collaborators/alice/permission",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "role_name": "write"
        })))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let _ = client
        .get_collaborator_permission("test-org", "test-repo", "alice")
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "get_collaborator_permission")
        .expect("get_collaborator_permission must create a span");
    let recorded_username = span
        .fields
        .get("username")
        .expect("get_collaborator_permission span must record a `username` field");
    assert_eq!(
        recorded_username, "alice",
        "an ordinary, short, non-secret-shaped username must pass through sanitization \
         unchanged, recorded value: {recorded_username:?}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_add_repository_collaborator_span_username_field_is_redacted_when_secret_shaped() {
    let mock_server = MockServer::start().await;
    let secret_shaped_username = format!("ghp_{}", "d".repeat(24));
    Mock::given(method("PUT"))
        .and(path(format!(
            "/repos/test-org/test-repo/collaborators/{secret_shaped_username}"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let _ = client
        .add_repository_collaborator("test-org", "test-repo", &secret_shaped_username, "push")
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "add_repository_collaborator")
        .expect("add_repository_collaborator must create a span");
    let recorded_username = span
        .fields
        .get("username")
        .expect("add_repository_collaborator span must record a `username` field");
    assert_eq!(
        recorded_username,
        crate::span_attributes::SPAN_ATTRIBUTE_REDACTED_MARKER,
        "a GitHub-token-shaped username must be fully redacted before being attached as a \
         span attribute, recorded value: {recorded_username:?}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_add_repository_collaborator_span_username_field_short_benign_value_passes_through_unchanged(
) {
    let mock_server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/repos/test-org/test-repo/collaborators/bob"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let _ = client
        .add_repository_collaborator("test-org", "test-repo", "bob", "push")
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "add_repository_collaborator")
        .expect("add_repository_collaborator must create a span");
    let recorded_username = span
        .fields
        .get("username")
        .expect("add_repository_collaborator span must record a `username` field");
    assert_eq!(
        recorded_username, "bob",
        "an ordinary, short, non-secret-shaped username must pass through sanitization \
         unchanged, recorded value: {recorded_username:?}"
    );
}

// ---------------------------------------------------------------------
// RED tests: Tester's independent, exhaustive verification pass.
//
// The Security Reviewer's spot-check estimated "~21 more `#[instrument]`
// sites". Grepping every `#[instrument]` attribute in `lib.rs` directly
// (rather than trusting that estimate) found exactly 20 annotation sites
// (51 individual field occurrences) that still attach a business-context
// value without routing it through
// `span_attributes::sanitize_span_attribute_value`. Four of those are
// already covered by real, value-observing RED tests above
// (`get_installation_token_for_org`'s `org_name`,
// `search_repositories_by_topic`'s `topic`, `get_collaborator_permission`'s
// `username`, `add_repository_collaborator`'s `username`).
//
// This section covers the remaining 16, PLUS 5 field-level gaps this
// Tester found that were not on the reviewer's or the earlier test-writing
// pass's radar at all, because they hide on annotations that were
// *partially* fixed already:
//   - `list_directory_contents` (line 408) sanitizes `path` but not its
//     `owner`/`repo`/`branch` fields.
//   - `get_file_content` (line 674) sanitizes `path` but not its
//     `owner`/`repo` fields.
//   - `authenticate_with_access_token` (line 3581), `create_octocrab_client`
//     (line 3754), and `create_github_client` (line 3777) were not
//     mentioned by the reviewer at all: the first attaches
//     `repository_owner`/`source_repository` via implicit `#[instrument]`
//     argument capture (no `fields(...)` override), the latter two attach
//     `base_url` the same way. All three are free functions, not methods,
//     which is likely why the reviewer's spot-check (which cited only
//     method line numbers) missed them.
//
// Every method/function below gets exactly two tests: one proving a
// secret-shaped value is not currently redacted (the real RED signal),
// and one stub-killing test proving an ordinary benign value is not
// currently mangled (guards against a future lazy "always redact"
// non-fix). Every field the annotation attaches gets at least one
// assertion in each test — none are skipped for brevity, per this pass's
// completeness mandate. Truncation-at-length-bound is deliberately NOT
// re-tested at every call site: it is already covered exhaustively for
// the sanitizer entry point itself in `span_attributes_tests.rs` and
// `span_attributes_proptests.rs`, and per-call-site truncation behaviour
// for the `path` field is already proven above.
// ---------------------------------------------------------------------

// --- get_repository (line 232): owner, repo ---

#[tokio::test(flavor = "current_thread")]
async fn test_get_repository_span_fields_redacted_when_secret_shaped() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "name": "test-repo",
            "full_name": "test-org/test-repo",
            "node_id": "MDEwOlJlcG9zaXRvcnkx",
            "private": false,
        })))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let secret_shaped_owner = format!("ghp_{}", "f".repeat(24));
    let secret_shaped_repo = format!("ghp_{}", "g".repeat(24));
    let _ = client
        .get_repository(&secret_shaped_owner, &secret_shaped_repo)
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "get_repository")
        .expect("get_repository must create a span");
    for field in ["owner", "repo"] {
        assert_eq!(
            span.fields
                .get(field)
                .unwrap_or_else(|| panic!("get_repository span must record a `{field}` field")),
            crate::span_attributes::SPAN_ATTRIBUTE_REDACTED_MARKER,
            "a GitHub-token-shaped `{field}` must be fully redacted, recorded fields: {:?}",
            span.fields
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_get_repository_span_fields_short_benign_values_pass_through_unchanged() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "name": "test-repo",
            "full_name": "test-org/test-repo",
            "node_id": "MDEwOlJlcG9zaXRvcnkx",
            "private": false,
        })))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let _ = client.get_repository("benign-owner", "benign-repo").await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "get_repository")
        .expect("get_repository must create a span");
    assert_eq!(
        span.fields.get("owner").unwrap(),
        "benign-owner",
        "recorded fields: {:?}",
        span.fields
    );
    assert_eq!(
        span.fields.get("repo").unwrap(),
        "benign-repo",
        "recorded fields: {:?}",
        span.fields
    );
}

// --- list_directory_contents (line 408): owner, repo, branch ---
// (`path` is already sanitized and covered above; this covers the three
// sibling fields on the same annotation that are not.)

#[tokio::test(flavor = "current_thread")]
async fn test_list_directory_contents_span_owner_repo_branch_fields_redacted_when_secret_shaped() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let secret_shaped_owner = format!("ghp_{}", "h".repeat(24));
    let secret_shaped_repo = format!("ghp_{}", "i".repeat(24));
    let secret_shaped_branch = format!("ghp_{}", "j".repeat(24));
    let _ = client
        .list_directory_contents(
            &secret_shaped_owner,
            &secret_shaped_repo,
            "benign/path",
            &secret_shaped_branch,
        )
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "list_directory_contents")
        .expect("list_directory_contents must create a span");
    for field in ["owner", "repo", "branch"] {
        assert_eq!(
            span.fields.get(field).unwrap_or_else(|| panic!(
                "list_directory_contents span must record a `{field}` field"
            )),
            crate::span_attributes::SPAN_ATTRIBUTE_REDACTED_MARKER,
            "a GitHub-token-shaped `{field}` must be fully redacted, recorded fields: {:?}",
            span.fields
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_list_directory_contents_span_owner_repo_branch_fields_short_benign_values_pass_through_unchanged(
) {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let _ = client
        .list_directory_contents("benign-owner", "benign-repo", "src", "benign-branch")
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "list_directory_contents")
        .expect("list_directory_contents must create a span");
    assert_eq!(span.fields.get("owner").unwrap(), "benign-owner");
    assert_eq!(span.fields.get("repo").unwrap(), "benign-repo");
    assert_eq!(span.fields.get("branch").unwrap(), "benign-branch");
}

// --- get_file_content (line 674): owner, repo ---
// (`path` is already sanitized and covered above.)

#[tokio::test(flavor = "current_thread")]
async fn test_get_file_content_span_owner_repo_fields_redacted_when_secret_shaped() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "name": "file",
            "path": "file",
            "type": "file",
            "sha": "abc123",
            "size": 5,
            "download_url": "https://raw.githubusercontent.com/test-org/test-repo/main/file",
            "content": "aGVsbG8="
        })))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let secret_shaped_owner = format!("ghp_{}", "k".repeat(24));
    let secret_shaped_repo = format!("ghp_{}", "l".repeat(24));
    let _ = client
        .get_file_content(&secret_shaped_owner, &secret_shaped_repo, "benign/path")
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "get_file_content")
        .expect("get_file_content must create a span");
    for field in ["owner", "repo"] {
        assert_eq!(
            span.fields
                .get(field)
                .unwrap_or_else(|| panic!("get_file_content span must record a `{field}` field")),
            crate::span_attributes::SPAN_ATTRIBUTE_REDACTED_MARKER,
            "a GitHub-token-shaped `{field}` must be fully redacted, recorded fields: {:?}",
            span.fields
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_get_file_content_span_owner_repo_fields_short_benign_values_pass_through_unchanged() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "name": "file",
            "path": "file",
            "type": "file",
            "sha": "abc123",
            "size": 5,
            "download_url": "https://raw.githubusercontent.com/test-org/test-repo/main/file",
            "content": "aGVsbG8="
        })))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let _ = client
        .get_file_content("benign-owner", "benign-repo", "src/main.rs")
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "get_file_content")
        .expect("get_file_content must create a span");
    assert_eq!(span.fields.get("owner").unwrap(), "benign-owner");
    assert_eq!(span.fields.get("repo").unwrap(), "benign-repo");
}

// --- list_organization_teams (line 847): org ---

#[tokio::test(flavor = "current_thread")]
async fn test_list_organization_teams_span_org_field_redacted_when_secret_shaped() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let secret_shaped_org = format!("ghp_{}", "m".repeat(24));
    let _ = client.list_organization_teams(&secret_shaped_org).await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "list_organization_teams")
        .expect("list_organization_teams must create a span");
    assert_eq!(
        span.fields.get("org").unwrap(),
        crate::span_attributes::SPAN_ATTRIBUTE_REDACTED_MARKER,
        "recorded fields: {:?}",
        span.fields
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_list_organization_teams_span_org_field_short_benign_value_passes_through_unchanged() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let _ = client.list_organization_teams("benign-org").await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "list_organization_teams")
        .expect("list_organization_teams must create a span");
    assert_eq!(span.fields.get("org").unwrap(), "benign-org");
}

// --- get_team_members (line 957): org, team_slug ---

#[tokio::test(flavor = "current_thread")]
async fn test_get_team_members_span_fields_redacted_when_secret_shaped() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let secret_shaped_org = format!("ghp_{}", "n".repeat(24));
    let secret_shaped_team_slug = format!("ghp_{}", "o".repeat(24));
    let _ = client
        .get_team_members(&secret_shaped_org, &secret_shaped_team_slug)
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "get_team_members")
        .expect("get_team_members must create a span");
    for field in ["org", "team_slug"] {
        assert_eq!(
            span.fields
                .get(field)
                .unwrap_or_else(|| panic!("get_team_members span must record a `{field}` field")),
            crate::span_attributes::SPAN_ATTRIBUTE_REDACTED_MARKER,
            "recorded fields: {:?}",
            span.fields
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_get_team_members_span_fields_short_benign_values_pass_through_unchanged() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let _ = client.get_team_members("benign-org", "benign-team").await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "get_team_members")
        .expect("get_team_members must create a span");
    assert_eq!(span.fields.get("org").unwrap(), "benign-org");
    assert_eq!(span.fields.get("team_slug").unwrap(), "benign-team");
}

// --- add_team_to_repository (line 1092): org, team_slug, repo, permission ---

#[tokio::test(flavor = "current_thread")]
async fn test_add_team_to_repository_span_fields_redacted_when_secret_shaped() {
    let mock_server = MockServer::start().await;
    Mock::given(method("PUT"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let secret_shaped_org = format!("ghp_{}", "p".repeat(24));
    let secret_shaped_team_slug = format!("ghp_{}", "q".repeat(24));
    let secret_shaped_repo = format!("ghp_{}", "r".repeat(24));
    let secret_shaped_permission = format!("ghp_{}", "s".repeat(24));
    let _ = client
        .add_team_to_repository(
            &secret_shaped_org,
            &secret_shaped_team_slug,
            &secret_shaped_repo,
            &secret_shaped_permission,
        )
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "add_team_to_repository")
        .expect("add_team_to_repository must create a span");
    for field in ["org", "team_slug", "repo", "permission"] {
        assert_eq!(
            span.fields.get(field).unwrap_or_else(|| panic!(
                "add_team_to_repository span must record a `{field}` field"
            )),
            crate::span_attributes::SPAN_ATTRIBUTE_REDACTED_MARKER,
            "recorded fields: {:?}",
            span.fields
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_add_team_to_repository_span_fields_short_benign_values_pass_through_unchanged() {
    let mock_server = MockServer::start().await;
    Mock::given(method("PUT"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let _ = client
        .add_team_to_repository("benign-org", "benign-team", "benign-repo", "push")
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "add_team_to_repository")
        .expect("add_team_to_repository must create a span");
    assert_eq!(span.fields.get("org").unwrap(), "benign-org");
    assert_eq!(span.fields.get("team_slug").unwrap(), "benign-team");
    assert_eq!(span.fields.get("repo").unwrap(), "benign-repo");
    assert_eq!(span.fields.get("permission").unwrap(), "push");
}

// --- set_team_repository_permission (line 1143): org, team_slug, repo_owner, repo, permission ---

#[tokio::test(flavor = "current_thread")]
async fn test_set_team_repository_permission_span_fields_redacted_when_secret_shaped() {
    let mock_server = MockServer::start().await;
    Mock::given(method("PUT"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let secret_shaped_org = format!("ghp_{}", "t".repeat(24));
    let secret_shaped_team_slug = format!("ghp_{}", "u".repeat(24));
    let secret_shaped_repo_owner = format!("ghp_{}", "v".repeat(24));
    let secret_shaped_repo = format!("ghp_{}", "w".repeat(24));
    let secret_shaped_permission = format!("ghp_{}", "x".repeat(24));
    let _ = client
        .set_team_repository_permission(
            &secret_shaped_org,
            &secret_shaped_team_slug,
            &secret_shaped_repo_owner,
            &secret_shaped_repo,
            &secret_shaped_permission,
        )
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "set_team_repository_permission")
        .expect("set_team_repository_permission must create a span");
    for field in ["org", "team_slug", "repo_owner", "repo", "permission"] {
        assert_eq!(
            span.fields.get(field).unwrap_or_else(|| panic!(
                "set_team_repository_permission span must record a `{field}` field"
            )),
            crate::span_attributes::SPAN_ATTRIBUTE_REDACTED_MARKER,
            "recorded fields: {:?}",
            span.fields
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_set_team_repository_permission_span_fields_short_benign_values_pass_through_unchanged(
) {
    let mock_server = MockServer::start().await;
    Mock::given(method("PUT"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let _ = client
        .set_team_repository_permission(
            "benign-org",
            "benign-team",
            "benign-owner",
            "benign-repo",
            "maintain",
        )
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "set_team_repository_permission")
        .expect("set_team_repository_permission must create a span");
    assert_eq!(span.fields.get("org").unwrap(), "benign-org");
    assert_eq!(span.fields.get("team_slug").unwrap(), "benign-team");
    assert_eq!(span.fields.get("repo_owner").unwrap(), "benign-owner");
    assert_eq!(span.fields.get("repo").unwrap(), "benign-repo");
    assert_eq!(span.fields.get("permission").unwrap(), "maintain");
}

// --- get_team_repository_permission (line 1277): org, team_slug, repo_owner, repo ---

#[tokio::test(flavor = "current_thread")]
async fn test_get_team_repository_permission_span_fields_redacted_when_secret_shaped() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let secret_shaped_org = format!("ghp_{}", "y".repeat(24));
    let secret_shaped_team_slug = format!("ghp_{}", "z".repeat(24));
    let secret_shaped_repo_owner = format!("ghp_{}A", "a".repeat(24));
    let secret_shaped_repo = format!("ghp_{}B", "b".repeat(24));
    let _ = client
        .get_team_repository_permission(
            &secret_shaped_org,
            &secret_shaped_team_slug,
            &secret_shaped_repo_owner,
            &secret_shaped_repo,
        )
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "get_team_repository_permission")
        .expect("get_team_repository_permission must create a span");
    for field in ["org", "team_slug", "repo_owner", "repo"] {
        assert_eq!(
            span.fields.get(field).unwrap_or_else(|| panic!(
                "get_team_repository_permission span must record a `{field}` field"
            )),
            crate::span_attributes::SPAN_ATTRIBUTE_REDACTED_MARKER,
            "recorded fields: {:?}",
            span.fields
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_get_team_repository_permission_span_fields_short_benign_values_pass_through_unchanged(
) {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let _ = client
        .get_team_repository_permission("benign-org", "benign-team", "benign-owner", "benign-repo")
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "get_team_repository_permission")
        .expect("get_team_repository_permission must create a span");
    assert_eq!(span.fields.get("org").unwrap(), "benign-org");
    assert_eq!(span.fields.get("team_slug").unwrap(), "benign-team");
    assert_eq!(span.fields.get("repo_owner").unwrap(), "benign-owner");
    assert_eq!(span.fields.get("repo").unwrap(), "benign-repo");
}

// --- get_repository_team_permission (line 1406): owner, repo, team_slug ---

#[tokio::test(flavor = "current_thread")]
async fn test_get_repository_team_permission_span_fields_redacted_when_secret_shaped() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let secret_shaped_owner = format!("ghp_{}C", "c".repeat(24));
    let secret_shaped_repo = format!("ghp_{}D", "d".repeat(24));
    let secret_shaped_team_slug = format!("ghp_{}E", "e".repeat(24));
    let _ = client
        .get_repository_team_permission(
            &secret_shaped_owner,
            &secret_shaped_repo,
            &secret_shaped_team_slug,
        )
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "get_repository_team_permission")
        .expect("get_repository_team_permission must create a span");
    for field in ["owner", "repo", "team_slug"] {
        assert_eq!(
            span.fields.get(field).unwrap_or_else(|| panic!(
                "get_repository_team_permission span must record a `{field}` field"
            )),
            crate::span_attributes::SPAN_ATTRIBUTE_REDACTED_MARKER,
            "recorded fields: {:?}",
            span.fields
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_get_repository_team_permission_span_fields_short_benign_values_pass_through_unchanged(
) {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let _ = client
        .get_repository_team_permission("benign-owner", "benign-repo", "benign-team")
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "get_repository_team_permission")
        .expect("get_repository_team_permission must create a span");
    assert_eq!(span.fields.get("owner").unwrap(), "benign-owner");
    assert_eq!(span.fields.get("repo").unwrap(), "benign-repo");
    assert_eq!(span.fields.get("team_slug").unwrap(), "benign-team");
}

// --- list_repository_collaborators (line 1642): owner, repo ---

#[tokio::test(flavor = "current_thread")]
async fn test_list_repository_collaborators_span_fields_redacted_when_secret_shaped() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let secret_shaped_owner = format!("ghp_{}F", "f".repeat(24));
    let secret_shaped_repo = format!("ghp_{}G", "g".repeat(24));
    let _ = client
        .list_repository_collaborators(&secret_shaped_owner, &secret_shaped_repo)
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "list_repository_collaborators")
        .expect("list_repository_collaborators must create a span");
    for field in ["owner", "repo"] {
        assert_eq!(
            span.fields.get(field).unwrap_or_else(|| panic!(
                "list_repository_collaborators span must record a `{field}` field"
            )),
            crate::span_attributes::SPAN_ATTRIBUTE_REDACTED_MARKER,
            "recorded fields: {:?}",
            span.fields
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_list_repository_collaborators_span_fields_short_benign_values_pass_through_unchanged()
{
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let _ = client
        .list_repository_collaborators("benign-owner", "benign-repo")
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "list_repository_collaborators")
        .expect("list_repository_collaborators must create a span");
    assert_eq!(span.fields.get("owner").unwrap(), "benign-owner");
    assert_eq!(span.fields.get("repo").unwrap(), "benign-repo");
}

// --- set_collaborator_permission (line 1843): owner, repo, username, permission ---

#[tokio::test(flavor = "current_thread")]
async fn test_set_collaborator_permission_span_fields_redacted_when_secret_shaped() {
    let mock_server = MockServer::start().await;
    Mock::given(method("PUT"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let secret_shaped_owner = format!("ghp_{}H", "h".repeat(24));
    let secret_shaped_repo = format!("ghp_{}I", "i".repeat(24));
    let secret_shaped_username = format!("ghp_{}J", "j".repeat(24));
    let secret_shaped_permission = format!("ghp_{}K", "k".repeat(24));
    let _ = client
        .set_collaborator_permission(
            &secret_shaped_owner,
            &secret_shaped_repo,
            &secret_shaped_username,
            &secret_shaped_permission,
        )
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "set_collaborator_permission")
        .expect("set_collaborator_permission must create a span");
    for field in ["owner", "repo", "username", "permission"] {
        assert_eq!(
            span.fields.get(field).unwrap_or_else(|| panic!(
                "set_collaborator_permission span must record a `{field}` field"
            )),
            crate::span_attributes::SPAN_ATTRIBUTE_REDACTED_MARKER,
            "recorded fields: {:?}",
            span.fields
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_set_collaborator_permission_span_fields_short_benign_values_pass_through_unchanged() {
    let mock_server = MockServer::start().await;
    Mock::given(method("PUT"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let _ = client
        .set_collaborator_permission("benign-owner", "benign-repo", "benign-user", "push")
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "set_collaborator_permission")
        .expect("set_collaborator_permission must create a span");
    assert_eq!(span.fields.get("owner").unwrap(), "benign-owner");
    assert_eq!(span.fields.get("repo").unwrap(), "benign-repo");
    assert_eq!(span.fields.get("username").unwrap(), "benign-user");
    assert_eq!(span.fields.get("permission").unwrap(), "push");
}

// --- remove_repository_collaborator (line 1872): owner, repo, username ---

#[tokio::test(flavor = "current_thread")]
async fn test_remove_repository_collaborator_span_fields_redacted_when_secret_shaped() {
    let mock_server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let secret_shaped_owner = format!("ghp_{}L", "l".repeat(24));
    let secret_shaped_repo = format!("ghp_{}M", "m".repeat(24));
    let secret_shaped_username = format!("ghp_{}N", "n".repeat(24));
    let _ = client
        .remove_repository_collaborator(
            &secret_shaped_owner,
            &secret_shaped_repo,
            &secret_shaped_username,
        )
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "remove_repository_collaborator")
        .expect("remove_repository_collaborator must create a span");
    for field in ["owner", "repo", "username"] {
        assert_eq!(
            span.fields.get(field).unwrap_or_else(|| panic!(
                "remove_repository_collaborator span must record a `{field}` field"
            )),
            crate::span_attributes::SPAN_ATTRIBUTE_REDACTED_MARKER,
            "recorded fields: {:?}",
            span.fields
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_remove_repository_collaborator_span_fields_short_benign_values_pass_through_unchanged(
) {
    let mock_server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let _ = client
        .remove_repository_collaborator("benign-owner", "benign-repo", "benign-user")
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "remove_repository_collaborator")
        .expect("remove_repository_collaborator must create a span");
    assert_eq!(span.fields.get("owner").unwrap(), "benign-owner");
    assert_eq!(span.fields.get("repo").unwrap(), "benign-repo");
    assert_eq!(span.fields.get("username").unwrap(), "benign-user");
}

// --- update_repository_settings (line 2007): owner, repo ---
// Reached only via the `RepositoryClient` trait (no inherent method of this
// name exists on `GitHubClient`); `RepositoryClient` is brought into scope
// by `use super::*;` at the top of this file, so ordinary dot-call syntax
// resolves to the trait method.

#[tokio::test(flavor = "current_thread")]
async fn test_update_repository_settings_span_fields_redacted_when_secret_shaped() {
    let mock_server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 123456,
            "name": "test-repo",
            "url": "https://api.github.com/repos/test-owner/test-repo"
        })))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let secret_shaped_owner = format!("ghp_{}O", "o".repeat(24));
    let secret_shaped_repo = format!("ghp_{}P", "p".repeat(24));
    let settings = RepositorySettingsUpdate::default();
    let _ = client
        .update_repository_settings(&secret_shaped_owner, &secret_shaped_repo, &settings)
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "update_repository_settings")
        .expect("update_repository_settings must create a span");
    for field in ["owner", "repo"] {
        assert_eq!(
            span.fields.get(field).unwrap_or_else(|| panic!(
                "update_repository_settings span must record a `{field}` field"
            )),
            crate::span_attributes::SPAN_ATTRIBUTE_REDACTED_MARKER,
            "recorded fields: {:?}",
            span.fields
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_update_repository_settings_span_fields_short_benign_values_pass_through_unchanged() {
    let mock_server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 123456,
            "name": "test-repo",
            "url": "https://api.github.com/repos/test-owner/test-repo"
        })))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let settings = RepositorySettingsUpdate::default();
    let _ = client
        .update_repository_settings("benign-owner", "benign-repo", &settings)
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "update_repository_settings")
        .expect("update_repository_settings must create a span");
    assert_eq!(span.fields.get("owner").unwrap(), "benign-owner");
    assert_eq!(span.fields.get("repo").unwrap(), "benign-repo");
}

// --- authenticate_with_access_token (line 3581): repository_owner, source_repository ---
//
// Free function, not a `GitHubClient` method. `#[instrument]` has no
// `skip(...)`/`fields(...)` at all, so every non-`Self` parameter is
// auto-captured by implicit argument recording -- including
// `repository_owner`/`source_repository` (attacker-influenceable strings,
// exactly analogous to `owner`/`repo` elsewhere) and `installation_id`
// (a `u64`, out of scope for string sanitization). `octocrab: &Octocrab`
// is also auto-captured; this Tester flags that as a SEPARATE, likely
// MORE severe concern for the architect/Security Reviewer (an `Octocrab`
// client's `Debug` output may include internal auth state) that needs a
// `skip(octocrab)` fix, not a `sanitize_span_attribute_value` fix -- no
// test is added for it here since it requires a different remediation
// shape than this pass's mechanical pattern.

#[tokio::test(flavor = "current_thread")]
async fn test_authenticate_with_access_token_span_fields_redacted_when_secret_shaped() {
    let mock_server = MockServer::start().await;
    let installation_id: u64 = 42;
    Mock::given(method("POST"))
        .and(path(format!(
            "/app/installations/{installation_id}/access_tokens"
        )))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "token": "test-token-value",
            "expires_at": "2099-12-31T23:59:59Z"
        })))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(99999u64.into(), test_rsa_private_key_pem().as_bytes())
        .expect("test RSA PEM must parse")
        .build()
        .expect("test octocrab client must build");

    let secret_shaped_repository_owner = format!("ghp_{}Q", "q".repeat(24));
    let secret_shaped_source_repository = format!("ghp_{}R", "r".repeat(24));
    let _ = authenticate_with_access_token(
        &octocrab,
        installation_id,
        &secret_shaped_repository_owner,
        &secret_shaped_source_repository,
    )
    .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "authenticate_with_access_token")
        .expect("authenticate_with_access_token must create a span");
    for field in ["repository_owner", "source_repository"] {
        assert_eq!(
            span.fields.get(field).unwrap_or_else(|| panic!(
                "authenticate_with_access_token span must record a `{field}` field"
            )),
            crate::span_attributes::SPAN_ATTRIBUTE_REDACTED_MARKER,
            "recorded fields: {:?}",
            span.fields
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_authenticate_with_access_token_span_fields_short_benign_values_pass_through_unchanged(
) {
    let mock_server = MockServer::start().await;
    let installation_id: u64 = 43;
    Mock::given(method("POST"))
        .and(path(format!(
            "/app/installations/{installation_id}/access_tokens"
        )))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "token": "test-token-value",
            "expires_at": "2099-12-31T23:59:59Z"
        })))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(99999u64.into(), test_rsa_private_key_pem().as_bytes())
        .expect("test RSA PEM must parse")
        .build()
        .expect("test octocrab client must build");

    let _ =
        authenticate_with_access_token(&octocrab, installation_id, "benign-owner", "benign-repo")
            .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "authenticate_with_access_token")
        .expect("authenticate_with_access_token must create a span");
    assert_eq!(span.fields.get("repository_owner").unwrap(), "benign-owner");
    assert_eq!(span.fields.get("source_repository").unwrap(), "benign-repo");
}

// --- create_octocrab_client (line 3754) / create_github_client (line 3777): base_url ---
//
// Both are synchronous free functions with no network I/O of their own
// (`create_github_client` delegates to `create_octocrab_client`, which
// itself only calls `Octocrab::builder()`), so no `MockServer` is needed:
// span-attribute capture happens on function entry, before the builder
// runs, independent of whether the (deliberately non-URL-shaped)
// secret-shaped input later fails to parse as a URI.
//
// `base_url: Option<&str>` is not covered by any explicit
// `fields(...)` clause, so `#[instrument(skip(token))]` auto-captures it
// via `Debug` -- meaning the recorded value today is the raw
// `Some("<value>")` debug-formatted string, not a bare string. The
// assertions below therefore check for the ABSENCE of the raw secret
// substring (robust to whatever exact wrapping/formatting a fix chooses)
// rather than exact equality to the redaction marker.

#[tokio::test(flavor = "current_thread")]
async fn test_create_octocrab_client_span_base_url_field_redacted_when_secret_shaped() {
    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let secret_shaped_base_url = format!("ghp_{}S", "s".repeat(24));
    let _ = create_octocrab_client("test-token", Some(secret_shaped_base_url.as_str()));

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "create_octocrab_client")
        .expect("create_octocrab_client must create a span");
    let recorded_base_url = span
        .fields
        .get("base_url")
        .expect("create_octocrab_client span must record a `base_url` field");
    assert!(
        !recorded_base_url.contains(&secret_shaped_base_url),
        "a GitHub-token-shaped base_url must not appear verbatim in the recorded span \
         attribute, recorded value: {recorded_base_url:?}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_create_octocrab_client_span_base_url_field_short_benign_value_passes_through_unchanged(
) {
    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let _ = create_octocrab_client("test-token", Some("https://github.example.com"));

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "create_octocrab_client")
        .expect("create_octocrab_client must create a span");
    let recorded_base_url = span
        .fields
        .get("base_url")
        .expect("create_octocrab_client span must record a `base_url` field");
    assert!(
        recorded_base_url.contains("https://github.example.com"),
        "an ordinary, short, non-secret-shaped base_url must remain recognisable, \
         recorded value: {recorded_base_url:?}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_create_github_client_span_base_url_field_redacted_when_secret_shaped() {
    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let secret_shaped_base_url = format!("ghp_{}T", "t".repeat(24));
    let _ = create_github_client("test-token", Some(secret_shaped_base_url.as_str()));

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "create_github_client")
        .expect("create_github_client must create a span");
    let recorded_base_url = span
        .fields
        .get("base_url")
        .expect("create_github_client span must record a `base_url` field");
    assert!(
        !recorded_base_url.contains(&secret_shaped_base_url),
        "a GitHub-token-shaped base_url must not appear verbatim in the recorded span \
         attribute, recorded value: {recorded_base_url:?}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_create_github_client_span_base_url_field_short_benign_value_passes_through_unchanged()
{
    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let _ = create_github_client("test-token", Some("https://github.example.com"));

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "create_github_client")
        .expect("create_github_client must create a span");
    let recorded_base_url = span
        .fields
        .get("base_url")
        .expect("create_github_client span must record a `base_url` field");
    assert!(
        recorded_base_url.contains("https://github.example.com"),
        "an ordinary, short, non-secret-shaped base_url must remain recognisable, \
         recorded value: {recorded_base_url:?}"
    );
}

// ---------------------------------------------------------------------
// RED tests: scoping-gap follow-up. The Coder's fix (commit b539eb3)
// correctly sanitized `username`/`permission`-as-a-whole-annotation on
// every site in this Tester's original table, but two of those table rows
// under-scoped what needed fixing: `get_collaborator_permission` and
// `add_repository_collaborator` each attach `owner`/`repo` (and, on the
// latter, `permission`) on the EXACT SAME `#[instrument]` line as the
// `username` field that did get fixed. Only `username` was sanitized;
// `owner`/`repo`/`permission` on these two lines are still raw today:
//
//   get_collaborator_permission (line ~1549):
//     fields(owner = %owner, repo = %repo,
//            username = %span_attributes::sanitize_span_attribute_value(username))
//   add_repository_collaborator (line ~1755):
//     fields(owner = %owner, repo = %repo,
//            username = %span_attributes::sanitize_span_attribute_value(username),
//            permission = %permission)
//
// This section proves that gap with the same harness used throughout this
// file. `username` is deliberately left benign in both tests below (its
// sanitization is already covered by the tests above) so these tests
// isolate the specific fields under scrutiny.
// ---------------------------------------------------------------------

#[tokio::test(flavor = "current_thread")]
async fn test_get_collaborator_permission_span_owner_repo_fields_redacted_when_secret_shaped() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "role_name": "write"
        })))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let secret_shaped_owner = format!("ghp_{}", "u".repeat(24));
    let secret_shaped_repo = format!("ghp_{}", "v".repeat(24));
    let _ = client
        .get_collaborator_permission(&secret_shaped_owner, &secret_shaped_repo, "benign-user")
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "get_collaborator_permission")
        .expect("get_collaborator_permission must create a span");
    for field in ["owner", "repo"] {
        assert_eq!(
            span.fields.get(field).unwrap_or_else(|| panic!(
                "get_collaborator_permission span must record a `{field}` field"
            )),
            crate::span_attributes::SPAN_ATTRIBUTE_REDACTED_MARKER,
            "a GitHub-token-shaped `{field}` must be fully redacted before being attached as a \
             span attribute, even though it shares its #[instrument] line with the \
             already-sanitized `username` field, recorded fields: {:?}",
            span.fields
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_get_collaborator_permission_span_owner_repo_fields_short_benign_values_pass_through_unchanged(
) {
    // Stub-killing: guards against a lazy "always redact" fix that would
    // replace every owner/repo value with the marker regardless of content.
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "role_name": "write"
        })))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let _ = client
        .get_collaborator_permission("benign-owner", "benign-repo", "benign-user")
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "get_collaborator_permission")
        .expect("get_collaborator_permission must create a span");
    assert_eq!(
        span.fields.get("owner").unwrap(),
        "benign-owner",
        "recorded fields: {:?}",
        span.fields
    );
    assert_eq!(
        span.fields.get("repo").unwrap(),
        "benign-repo",
        "recorded fields: {:?}",
        span.fields
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_add_repository_collaborator_span_owner_repo_permission_fields_redacted_when_secret_shaped(
) {
    let mock_server = MockServer::start().await;
    Mock::given(method("PUT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let secret_shaped_owner = format!("ghp_{}", "w".repeat(24));
    let secret_shaped_repo = format!("ghp_{}", "x".repeat(24));
    let secret_shaped_permission = format!("ghp_{}", "y".repeat(24));
    let _ = client
        .add_repository_collaborator(
            &secret_shaped_owner,
            &secret_shaped_repo,
            "benign-user",
            &secret_shaped_permission,
        )
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "add_repository_collaborator")
        .expect("add_repository_collaborator must create a span");
    for field in ["owner", "repo", "permission"] {
        assert_eq!(
            span.fields.get(field).unwrap_or_else(|| panic!(
                "add_repository_collaborator span must record a `{field}` field"
            )),
            crate::span_attributes::SPAN_ATTRIBUTE_REDACTED_MARKER,
            "a GitHub-token-shaped `{field}` must be fully redacted before being attached as a \
             span attribute, even though it shares its #[instrument] line with the \
             already-sanitized `username` field, recorded fields: {:?}",
            span.fields
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_add_repository_collaborator_span_owner_repo_permission_fields_short_benign_values_pass_through_unchanged(
) {
    // Stub-killing: guards against a lazy "always redact" fix.
    let mock_server = MockServer::start().await;
    Mock::given(method("PUT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&mock_server)
        .await;

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let client = mock_client(&mock_server);
    let _ = client
        .add_repository_collaborator("benign-owner", "benign-repo", "benign-user", "push")
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "add_repository_collaborator")
        .expect("add_repository_collaborator must create a span");
    assert_eq!(
        span.fields.get("owner").unwrap(),
        "benign-owner",
        "recorded fields: {:?}",
        span.fields
    );
    assert_eq!(
        span.fields.get("repo").unwrap(),
        "benign-repo",
        "recorded fields: {:?}",
        span.fields
    );
    assert_eq!(
        span.fields.get("permission").unwrap(),
        "push",
        "recorded fields: {:?}",
        span.fields
    );
}

/// Pre-generated RSA-2048 test key (PKCS8 PEM), local to this file so it
/// does not depend on the sibling `lib_tests` module's private helper.
/// For tests only -- never use in production.
fn test_rsa_private_key_pem() -> &'static str {
    "-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQDLRZFg/A26veyg
1H1vIG8FFeTgiShtI+qBQsocdr4Gno6MO2vpdMS7ZXykOemvxfZFo2EBMHxuJYu2
GYK71KRifRILdeYhFJC3f0Szcx9AmjcrHJOzvWzijf2X74cN7Scg63FTup4X4YMF
CJ3r59eMU28Yx2NhsGxmXcPc6xMOKUq1eklT3Xzl4C1EDzEjxYlQXdgElkm8Hcn/
u/GYvzxCJtWYHT3FkDMd0/sxMEccGWU07GeUV5dpYYfheiswfxnUjb2OsrUejCsj
KLuiU/Xj/eIwpaSE48/8l1g+zhqMZqjN/E52rqw47IqK0WQ80fRcPPcrZ/uV/s6n
0FDr5u9RAgMBAAECggEAfUcv9U0DDS+WkfzEHeQoaEL5H6WIw9MZ6XaHaAmKzjuU
cTsCJAzyXuiKbBDBRLLGOF8cHwaJ+bysIZwRZcAFXCnnICcWoRayplUKu2pdPYyO
KcTgByvc2rhWLfjTDLM3pWNWIhtm6iLzg/uukMD/qJ2XX/cq+f8on+J0H0jGIrgY
md4D3HS7k2dxHjcHmV7oLHFv5uc6oUqDCNr5gVdYe5iqA3KBWiba3VT4CtGgPruK
l53S6HPXNjAb4x+y1d4fYoHSkX750MrVsjIT6iAHI0gQMSVqACVBS4K3ck+Og36z
CT3+ERmr2wL5JFCjxOXOxL870uSO5lYN4U4JKVn4kQKBgQDwxLSkSWrQ3N68P0cO
VcgVHW//yFBJe4LDpf2hFUYSssupTaCWm7qx/fuPXiQtUSJl04GgTt+c0a+Y4y96
ccKXhTpidsEnkCvZZ1IHUZcIam3KMenMHtH1XbmEhJSJBKXUs2APjPRFnAOdQ4ia
Nc7skEIqRA0iShEFEodw3F7vbQKBgQDYIZiwzr1iezX0iQlugpOn4iwTZIGrmR8v
BPwTk91KujNNhud7ZSGmSh1oj8igIbvpieOkQH0SCWxD6lK9b8O7mHCjGrj3BDCn
cWS4YhRKIpdHL9MN5qRHcKFhotgDrztIVNZ4rwbdANfJrqJGh08qSPDJ/MgJH8jf
j6txCn989QKBgEgkWJAlY3kjxSWYvDzCXUpeYLG+nrf15y+ibAW5Lx7pLfcTyHt4
PA+bL6Y8qhBeEDV8k4mqOHxvo/Lml9JTheAKzny0YpntLAZfXAfPXbbq1OzR2eEM
lxKzO6RGaY9IsjuyZ6UhMgwtfDPCCYtTdiYR75FkhJ7ynHjtnWGm1lddAoGAW57y
YZFii0r79mvZSbWk7nhcguQKbDn/FCmiypHWL4V7lhPoMVEP0LR+fwLk460pQbko
fUqxOmIwOSIubQviB7Z5mUsMufpWvcS+E8PQIhVD1/OYHr51SaufXgpwVC+GjAUa
3hIkcK+YynBzbj6R+rTtDlNQYT3OI6d93++5V7kCgYEAoE/4vPrDtNyaNi39h3TF
seOmBxWp9ZSiyaDAaJckPop1Y0kqwZNaAe9LXKpYp15PRyD4ehRLNbJLxx6UFHYf
5IN2MmSKZDV9jdJsjelyucVio1J3lyN4JeCRXly11n/2615GiWaZJ6Q4dflDz0iS
JC29NBAKJUVTF8zCccVfB2E=
-----END PRIVATE KEY-----"
}
