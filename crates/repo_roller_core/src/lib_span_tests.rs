//! Span-hierarchy tests for `create_repository`'s top-level orchestration.
//!
//! Verifies the contract documented in `observability.rs`: `create_repository`
//! must open a [`crate::observability::SPAN_REPOSITORY_CREATION`] span, with
//! [`crate::observability::SPAN_AUTHENTICATION`] nested directly beneath it
//! as the first phase-boundary child span.
//!
//! ## Scope note (see `observability.rs`'s "Known spec gaps" section)
//!
//! Only the authentication-phase boundary is exercised here.
//! `configuration::resolve_organization_configuration` (backing
//! `SPAN_CONFIGURATION_RESOLUTION`) builds its own `GitHubClient` from the
//! raw installation token with no base-URL override, so driving
//! `create_repository` past authentication in a unit test would require
//! real network access to `api.github.com`. That is out of scope for a
//! deterministic RED/GREEN test loop; see `observability.rs` for the full
//! testability finding and recommendation.
//!
//! Uses a minimal, self-contained `tracing_subscriber::Layer` (`SpanRecorder`)
//! rather than a real OpenTelemetry exporter -- this file tests the
//! `tracing`-level span hierarchy contract that any OTel bridge (tested
//! separately in `repo_roller_api::tracing_otel`) would faithfully export
//! unchanged.

use super::*;
use async_trait::async_trait;
use config_manager::{ConfigurationResult, MetadataRepository, MetadataRepositoryProvider, TemplateConfig};
use std::sync::{Arc, Mutex};
use tracing::span::{Attributes, Id};
use tracing_subscriber::{layer::Context, layer::SubscriberExt, registry::LookupSpan, Layer};
use visibility::{GitHubEnvironmentDetector, PlanLimitations, VisibilityError, VisibilityPolicy, VisibilityPolicyProvider};

#[ctor::ctor(unsafe)]
fn init_default_crypto_provider() {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
}

// ---------------------------------------------------------------------
// Local test infrastructure (span recorder + minimal mocks).
// ---------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
struct RecordedSpan {
    name: &'static str,
    parent_name: Option<String>,
    field_names: Vec<String>,
}

#[derive(Clone, Default)]
struct SpanRecorder {
    spans: Arc<Mutex<Vec<RecordedSpan>>>,
}

impl SpanRecorder {
    fn recorded(&self) -> Vec<RecordedSpan> {
        self.spans.lock().expect("span recorder mutex poisoned").clone()
    }
}

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

/// Mock authentication service that always fails immediately, matching the
/// existing `MockAuthService` pattern in `lib_tests.rs` (kept as a separate,
/// crate-private copy here since sibling `#[cfg(test)]` modules cannot see
/// each other's private items).
struct FailingAuthService;

#[async_trait]
impl auth_handler::UserAuthenticationService for FailingAuthService {
    async fn get_installation_token_for_org(
        &self,
        _org_name: &str,
    ) -> auth_handler::AuthResult<String> {
        Err(auth_handler::AuthError::InvalidCredentials)
    }
}

struct EmptyMetadataProvider;

#[async_trait]
impl MetadataRepositoryProvider for EmptyMetadataProvider {
    async fn load_template_configuration(
        &self,
        _org: &str,
        _template_name: &str,
    ) -> ConfigurationResult<TemplateConfig> {
        unimplemented!("not reached: authentication fails before template loading")
    }

    async fn discover_metadata_repository(
        &self,
        _org: &str,
    ) -> ConfigurationResult<MetadataRepository> {
        unimplemented!("not used in these tests")
    }

    async fn load_global_defaults(
        &self,
        _repo: &MetadataRepository,
    ) -> ConfigurationResult<config_manager::GlobalDefaults> {
        unimplemented!("not used in these tests")
    }

    async fn load_team_configuration(
        &self,
        _repo: &MetadataRepository,
        _team: &str,
    ) -> ConfigurationResult<Option<config_manager::TeamConfig>> {
        unimplemented!("not used in these tests")
    }

    async fn load_repository_type_configuration(
        &self,
        _repo: &MetadataRepository,
        _repo_type: &str,
    ) -> ConfigurationResult<Option<config_manager::RepositoryTypeConfig>> {
        unimplemented!("not used in these tests")
    }

    async fn load_standard_labels(
        &self,
        _repo: &MetadataRepository,
    ) -> ConfigurationResult<std::collections::HashMap<String, config_manager::LabelConfig>> {
        unimplemented!("not used in these tests")
    }

    async fn list_available_repository_types(
        &self,
        _repo: &MetadataRepository,
    ) -> ConfigurationResult<Vec<String>> {
        unimplemented!("not used in these tests")
    }

    async fn validate_repository_structure(&self, _repo: &MetadataRepository) -> ConfigurationResult<()> {
        unimplemented!("not used in these tests")
    }

    async fn list_templates(&self, _org: &str) -> ConfigurationResult<Vec<String>> {
        unimplemented!("not used in these tests")
    }

    async fn load_global_webhooks(
        &self,
        _repo: &MetadataRepository,
    ) -> ConfigurationResult<Vec<config_manager::settings::WebhookConfig>> {
        unimplemented!("not used in these tests")
    }
}

struct UnrestrictedVisibilityPolicyProvider;

#[async_trait]
impl VisibilityPolicyProvider for UnrestrictedVisibilityPolicyProvider {
    async fn get_policy(&self, _organization: &str) -> Result<VisibilityPolicy, VisibilityError> {
        Ok(VisibilityPolicy::Unrestricted)
    }

    async fn invalidate_cache(&self, _organization: &str) {}
}

struct PaidPlanEnvironmentDetector;

#[async_trait]
impl GitHubEnvironmentDetector for PaidPlanEnvironmentDetector {
    async fn get_plan_limitations(
        &self,
        _organization: &str,
    ) -> Result<PlanLimitations, github_client::Error> {
        Ok(PlanLimitations {
            supports_private_repos: true,
            supports_internal_repos: false,
            private_repo_limit: None,
            is_enterprise: false,
        })
    }

    async fn is_enterprise(&self, _organization: &str) -> Result<bool, github_client::Error> {
        Ok(false)
    }
}

/// Runs `create_repository` with a `FailingAuthService` (so the call returns
/// before ever making a real network request) under a fresh `SpanRecorder`,
/// returning the recorded spans.
async fn run_create_repository_with_failing_auth() -> Vec<RecordedSpan> {
    run_create_repository_with_failing_auth_and_template(Some(TemplateName::new("basic").unwrap())).await
}

/// Same as [`run_create_repository_with_failing_auth`], but lets the caller
/// control whether the request specifies a template -- needed to assert on
/// [`crate::observability::ATTR_TEMPLATE`]'s conditional field presence at
/// the span level (see the `find_attribute`/`repository_creation_span`
/// mutation-kill tests below).
async fn run_create_repository_with_failing_auth_and_template(template: Option<TemplateName>) -> Vec<RecordedSpan> {
    let recorder = SpanRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let mut builder = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("test-repo").unwrap(),
        OrganizationName::new("test-org").unwrap(),
    );
    builder = match template {
        Some(template) => builder.template(template),
        // `ContentStrategy` defaults to `Template`, which panics on `build()`
        // unless a template is also set -- use `Empty` for the no-template case.
        None => builder.content_strategy(ContentStrategy::Empty),
    };
    let request = builder.build();

    let metadata_provider = EmptyMetadataProvider;
    let auth_service = FailingAuthService;
    let visibility_policy_provider = Arc::new(UnrestrictedVisibilityPolicyProvider);
    let environment_detector = Arc::new(PaidPlanEnvironmentDetector);
    let secret_resolver = Arc::new(crate::event_secrets::EnvironmentSecretResolver::new());
    let test_registry = prometheus::Registry::new();
    let metrics = Arc::new(crate::event_metrics::PrometheusEventMetrics::new(&test_registry));

    let result = create_repository(
        request,
        &metadata_provider,
        &auth_service,
        ".reporoller",
        visibility_policy_provider,
        environment_detector,
        EventNotificationContext::new("span-test", secret_resolver, metrics),
    )
    .await;

    assert!(
        result.is_err(),
        "FailingAuthService must cause create_repository to fail before any network call"
    );

    recorder.recorded()
}

// ---------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------

#[tokio::test(flavor = "current_thread")]
async fn test_create_repository_opens_repository_creation_span() {
    let spans = run_create_repository_with_failing_auth().await;
    assert!(
        spans.iter().any(|s| s.name == crate::observability::SPAN_REPOSITORY_CREATION),
        "create_repository must open a span named {:?}, recorded spans: {spans:?}",
        crate::observability::SPAN_REPOSITORY_CREATION
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_create_repository_authentication_span_is_child_of_repository_creation_span() {
    let spans = run_create_repository_with_failing_auth().await;
    let auth_span = spans
        .iter()
        .find(|s| s.name == crate::observability::SPAN_AUTHENTICATION)
        .unwrap_or_else(|| {
            panic!(
                "create_repository must open a span named {:?} for the authentication phase, \
                 recorded spans: {spans:?}",
                crate::observability::SPAN_AUTHENTICATION
            )
        });
    assert_eq!(
        auth_span.parent_name.as_deref(),
        Some(crate::observability::SPAN_REPOSITORY_CREATION),
        "authentication span must nest directly under repository_creation, recorded spans: {spans:?}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_create_repository_auth_failure_does_not_record_configuration_resolution_span() {
    let spans = run_create_repository_with_failing_auth().await;
    assert!(
        !spans
            .iter()
            .any(|s| s.name == crate::observability::SPAN_CONFIGURATION_RESOLUTION),
        "configuration_resolution must not run when authentication fails first, \
         recorded spans: {spans:?}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_create_repository_auth_failure_does_not_record_template_processing_span() {
    let spans = run_create_repository_with_failing_auth().await;
    assert!(
        !spans
            .iter()
            .any(|s| s.name == crate::observability::SPAN_TEMPLATE_PROCESSING),
        "template_processing must not run when authentication fails first, recorded spans: {spans:?}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_create_repository_auth_failure_does_not_record_github_repository_creation_span() {
    let spans = run_create_repository_with_failing_auth().await;
    assert!(
        !spans
            .iter()
            .any(|s| s.name == crate::observability::SPAN_GITHUB_REPOSITORY_CREATION),
        "github_repository_creation must not run when authentication fails first, \
         recorded spans: {spans:?}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_create_repository_repository_creation_span_carries_organization_attribute() {
    let spans = run_create_repository_with_failing_auth().await;
    let top_level = spans
        .iter()
        .find(|s| s.name == crate::observability::SPAN_REPOSITORY_CREATION)
        .expect("repository_creation span must exist");
    assert!(
        top_level
            .field_names
            .iter()
            .any(|f| f == crate::observability::ATTR_ORGANIZATION),
        "repository_creation span must record an {:?} field (Context Enrichment requirement), \
         recorded fields: {:?}",
        crate::observability::ATTR_ORGANIZATION,
        top_level.field_names
    );
}

// ---------------------------------------------------------------------
// Tier 3: Mutation-kill tests (QA audit pass, #2 tracing) --
// `find_attribute`/`repository_creation_span`'s conditional `template`
// field presence.
// ---------------------------------------------------------------------

#[tokio::test(flavor = "current_thread")]
async fn test_create_repository_repository_creation_span_carries_template_attribute_when_request_has_template() {
    // Kills: `find_attribute -> Option<String> with None` (would omit the
    // `template` field even though this request specifies one).
    let spans =
        run_create_repository_with_failing_auth_and_template(Some(TemplateName::new("basic").unwrap())).await;
    let top_level = spans
        .iter()
        .find(|s| s.name == crate::observability::SPAN_REPOSITORY_CREATION)
        .expect("repository_creation span must exist");
    assert!(
        top_level
            .field_names
            .iter()
            .any(|f| f == crate::observability::ATTR_TEMPLATE),
        "repository_creation span must record a {:?} field when the request specifies a \
         template, recorded fields: {:?}",
        crate::observability::ATTR_TEMPLATE,
        top_level.field_names
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_create_repository_repository_creation_span_omits_template_attribute_when_request_has_no_template() {
    // Kills: `find_attribute -> Option<String> with Some(String::new())`,
    // `find_attribute -> Option<String> with Some("xyzzy".into())`, and
    // `replace == with != in find_attribute` -- all three cause the
    // `template` field to be (incorrectly) present even though this
    // request specifies no template.
    let spans = run_create_repository_with_failing_auth_and_template(None).await;
    let top_level = spans
        .iter()
        .find(|s| s.name == crate::observability::SPAN_REPOSITORY_CREATION)
        .expect("repository_creation span must exist");
    assert!(
        !top_level
            .field_names
            .iter()
            .any(|f| f == crate::observability::ATTR_TEMPLATE),
        "repository_creation span must NOT record a {:?} field when the request has no \
         template, recorded fields: {:?}",
        crate::observability::ATTR_TEMPLATE,
        top_level.field_names
    );
}
