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
        self.spans.lock().expect("span recorder mutex poisoned").clone()
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
        let span_ref = ctx.span(id).expect("span must exist in registry for on_new_span");
        let parent_name = span_ref.parent().map(|p| p.name().to_string());

        let mut collector = FieldNameCollector(Vec::new());
        attrs.record(&mut collector);

        self.spans.lock().expect("span recorder mutex poisoned").push(RecordedSpan {
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
        .expect("get_installation_token_for_org must create a span (already #[instrument]-ed today)");
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
    let create_repo_span = spans.iter().find(|s| s.name == "create_org_repository").expect(
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
