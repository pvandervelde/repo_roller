//! RED-phase adversarial test suite for `GET /api/v1/orgs/:org/creation-capabilities`.
//!
//! Written ahead of the implementation per the project's TDD pipeline (Tester
//! role runs before Coder). These tests reference `handlers::get_creation_capabilities`,
//! `handlers::CreationCapabilitiesQuery`, and
//! `models::response::CreationCapabilitiesResponse` — none of which exist yet —
//! so this file is EXPECTED to fail `cargo check --tests` until the Coder adds
//! them. This mirrors the existing RED-phase precedent in this repository
//! (see git history: "test(cli,api): add RED-phase tests for fork/mirror
//! request surface").
//!
//! ## Specification sources
//!
//! - `docs/spec/interfaces/fork-mirror-operations.md` §1 (template
//!   designation), §2 (capability toggles + `allowed_source_owners`
//!   fail-closed default), and §8b (this endpoint's route, handler pseudocode,
//!   response shape, and error table).
//! - Assertion IAA-003 (Creation Capability Discovery Endpoint) and
//!   IAA-004 (Ambiguity and Validation Errors).
//!
//! ## Test harness
//!
//! Follows the exact pattern already established by the `preview_configuration`
//! tests in `handlers_tests.rs`: a `wiremock::MockServer` stands in for the
//! GitHub API, `AppState::default().with_github_api_base_url(...).with_mock_installation_token(...)`
//! points the handler's GitHub client at it, and `create_router_without_auth`
//! (plus a synthetic `AuthContext` injection layer) builds a router that
//! bypasses JWT auth while still exercising real handler/routing logic.
//!
//! Helper functions (`repo_json`, `file_content_response`,
//! `search_repositories_response`) are intentionally duplicated from
//! `handlers_tests.rs` rather than imported, because that module's helpers are
//! private to its own `mod tests` scope and this is a separate `#[path]`
//! module — consistent with how `translation_fork_mirror_tests.rs` and
//! `create_cmd_fork_mirror_tests.rs` duplicate their own fixtures rather than
//! reaching into sibling test modules.
//!
//! ## Noted spec gap (see Tester report)
//!
//! §8b's prose says this endpoint "must perform zero GitHub API calls and
//! zero mutations," but the handler pseudocode in the same section calls
//! `resolve_configuration` and `find_designated_template`, both of which read
//! configuration files from the metadata/template repositories via the GitHub
//! API. Read literally, "zero GitHub API calls" contradicts "reuse existing
//! merge/designation machinery." These tests interpret the intent as "zero
//! *mutating* calls" (no POST/PUT/PATCH/DELETE to GitHub) and assert that
//! specifically — see `test_creation_capabilities_zero_mutating_http_calls_made_to_github`.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware,
};
use serde_json::{json, Value};
use tower::ServiceExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::handlers::CreationCapabilitiesQuery;
use crate::models::response::CreationCapabilitiesResponse;
use crate::routes::{create_router, create_router_without_auth};
use crate::AppState;

// ============================================================================
// Shared fixtures (duplicated from handlers_tests.rs — see module doc above)
// ============================================================================

/// Build a base64-encoded file-content API response for a given TOML string,
/// matching the shape GitHub's Contents API returns for `GET
/// /repos/{owner}/{repo}/contents/{path}` on a file.
fn file_content_response(
    org: &str,
    repo: &str,
    file_path: &str,
    toml_content: &str,
) -> serde_json::Value {
    use base64::{engine::general_purpose, Engine as _};
    let encoded = general_purpose::STANDARD.encode(toml_content.as_bytes());
    let api_url = format!("https://api.github.com/repos/{org}/{repo}/contents/{file_path}");
    let html_url = format!("https://github.com/{org}/{repo}/blob/main/{file_path}");
    json!({
        "name": file_path.split('/').next_back().unwrap_or(file_path),
        "path": file_path,
        "sha": "abc123def456",
        "size": toml_content.len(),
        "type": "file",
        "content": encoded,
        "encoding": "base64",
        "url": api_url,
        "html_url": html_url,
        "git_url": null,
        "download_url": null,
        "_links": {
            "self": api_url,
            "git": null,
            "html": html_url
        }
    })
}

/// Build a minimal but deserializable GitHub repository JSON object
/// (satisfies octocrab's `Repository` required fields).
fn repo_json(org: &str, name: &str) -> serde_json::Value {
    let base = format!("https://api.github.com/repos/{org}/{name}");
    let html = format!("https://github.com/{org}/{name}");
    let user_base = format!("https://api.github.com/users/{org}");
    let user_html = format!("https://github.com/{org}");

    let owner = json!({
        "login": org, "id": 1, "node_id": "MDQ6VXNlcjE=",
        "avatar_url": format!("{user_base}/avatars"),
        "gravatar_id": "",
        "url": user_base,
        "html_url": user_html,
        "followers_url": format!("{user_base}/followers"),
        "following_url": format!("{user_base}/following{{/other_user}}"),
        "gists_url": format!("{user_base}/gists{{/gist_id}}"),
        "starred_url": format!("{user_base}/starred{{/owner}}{{/repo}}"),
        "subscriptions_url": format!("{user_base}/subscriptions"),
        "organizations_url": format!("{user_base}/orgs"),
        "repos_url": format!("{user_base}/repos"),
        "events_url": format!("{user_base}/events{{/privacy}}"),
        "received_events_url": format!("{user_base}/received_events"),
        "type": "Organization", "site_admin": false,
        "patch_url": null, "email": null
    });

    let mut obj = serde_json::Map::new();
    obj.insert("id".into(), json!(1));
    obj.insert("node_id".into(), json!("MDEwOlJlcG9zaXRvcnkx"));
    obj.insert("name".into(), json!(name));
    obj.insert("full_name".into(), json!(format!("{org}/{name}")));
    obj.insert("private".into(), json!(false));
    obj.insert("html_url".into(), json!(html));
    obj.insert("url".into(), json!(base.clone()));
    obj.insert("forks_url".into(), json!(format!("{base}/forks")));
    obj.insert("keys_url".into(), json!(format!("{base}/keys{{/key_id}}")));
    obj.insert(
        "collaborators_url".into(),
        json!(format!("{base}/collaborators{{/collaborator}}")),
    );
    obj.insert("teams_url".into(), json!(format!("{base}/teams")));
    obj.insert("hooks_url".into(), json!(format!("{base}/hooks")));
    obj.insert(
        "issue_events_url".into(),
        json!(format!("{base}/issues/events{{/number}}")),
    );
    obj.insert("events_url".into(), json!(format!("{base}/events")));
    obj.insert(
        "assignees_url".into(),
        json!(format!("{base}/assignees{{/user}}")),
    );
    obj.insert(
        "branches_url".into(),
        json!(format!("{base}/branches{{/branch}}")),
    );
    obj.insert("tags_url".into(), json!(format!("{base}/tags")));
    obj.insert(
        "blobs_url".into(),
        json!(format!("{base}/git/blobs{{/sha}}")),
    );
    obj.insert(
        "git_tags_url".into(),
        json!(format!("{base}/git/tags{{/sha}}")),
    );
    obj.insert(
        "git_refs_url".into(),
        json!(format!("{base}/git/refs{{/sha}}")),
    );
    obj.insert(
        "trees_url".into(),
        json!(format!("{base}/git/trees{{/sha}}")),
    );
    obj.insert(
        "statuses_url".into(),
        json!(format!("{base}/statuses/{{sha}}")),
    );
    obj.insert("languages_url".into(), json!(format!("{base}/languages")));
    obj.insert("stargazers_url".into(), json!(format!("{base}/stargazers")));
    obj.insert(
        "contributors_url".into(),
        json!(format!("{base}/contributors")),
    );
    obj.insert(
        "subscribers_url".into(),
        json!(format!("{base}/subscribers")),
    );
    obj.insert(
        "subscription_url".into(),
        json!(format!("{base}/subscription")),
    );
    obj.insert(
        "commits_url".into(),
        json!(format!("{base}/commits{{/sha}}")),
    );
    obj.insert(
        "git_commits_url".into(),
        json!(format!("{base}/git/commits{{/sha}}")),
    );
    obj.insert(
        "comments_url".into(),
        json!(format!("{base}/comments{{/number}}")),
    );
    obj.insert(
        "issue_comment_url".into(),
        json!(format!("{base}/issues/comments{{/number}}")),
    );
    obj.insert(
        "contents_url".into(),
        json!(format!("{base}/contents/{{+path}}")),
    );
    obj.insert(
        "compare_url".into(),
        json!(format!("{base}/compare/{{base}}...{{head}}")),
    );
    obj.insert("merges_url".into(), json!(format!("{base}/merges")));
    obj.insert(
        "archive_url".into(),
        json!(format!("{base}/{{archive_format}}{{/ref}}")),
    );
    obj.insert("downloads_url".into(), json!(format!("{base}/downloads")));
    obj.insert(
        "issues_url".into(),
        json!(format!("{base}/issues{{/number}}")),
    );
    obj.insert(
        "pulls_url".into(),
        json!(format!("{base}/pulls{{/number}}")),
    );
    obj.insert(
        "milestones_url".into(),
        json!(format!("{base}/milestones{{/number}}")),
    );
    obj.insert(
        "notifications_url".into(),
        json!(format!("{base}/notifications{{?since,all,participating}}")),
    );
    obj.insert(
        "labels_url".into(),
        json!(format!("{base}/labels{{/name}}")),
    );
    obj.insert(
        "releases_url".into(),
        json!(format!("{base}/releases{{/id}}")),
    );
    obj.insert(
        "deployments_url".into(),
        json!(format!("{base}/deployments")),
    );
    obj.insert(
        "git_url".into(),
        json!(format!("git://github.com/{org}/{name}.git")),
    );
    obj.insert(
        "ssh_url".into(),
        json!(format!("git@github.com:{org}/{name}.git")),
    );
    obj.insert(
        "clone_url".into(),
        json!(format!("https://github.com/{org}/{name}.git")),
    );
    obj.insert(
        "svn_url".into(),
        json!(format!("https://github.com/{org}/{name}")),
    );
    obj.insert("homepage".into(), serde_json::Value::Null);
    obj.insert("size".into(), json!(0));
    obj.insert("stargazers_count".into(), json!(0));
    obj.insert("watchers_count".into(), json!(0));
    obj.insert("language".into(), serde_json::Value::Null);
    obj.insert("has_issues".into(), json!(true));
    obj.insert("has_projects".into(), json!(true));
    obj.insert("has_downloads".into(), json!(true));
    obj.insert("has_wiki".into(), json!(false));
    obj.insert("has_pages".into(), json!(false));
    obj.insert("has_discussions".into(), json!(false));
    obj.insert("forks_count".into(), json!(0));
    obj.insert("archived".into(), json!(false));
    obj.insert("disabled".into(), json!(false));
    obj.insert("open_issues_count".into(), json!(0));
    obj.insert("default_branch".into(), json!("main"));
    obj.insert("created_at".into(), json!("2024-01-01T00:00:00Z"));
    obj.insert("updated_at".into(), json!("2024-01-01T00:00:00Z"));
    obj.insert("pushed_at".into(), json!("2024-01-01T00:00:00Z"));
    obj.insert("owner".into(), owner);
    serde_json::Value::Object(obj)
}

/// Build a GitHub search-repositories response envelope wrapping full
/// repository objects for the given template slugs.
fn search_repositories_response(org: &str, template_names: &[&str]) -> serde_json::Value {
    let items: Vec<serde_json::Value> = template_names
        .iter()
        .map(|name| repo_json(org, name))
        .collect();
    json!({
        "total_count": items.len(),
        "incomplete_results": false,
        "items": items,
    })
}

// ============================================================================
// Mock-mounting helpers specific to this endpoint
// ============================================================================

/// Mounts metadata-repository discovery (`GET /repos/{org}/.reporoller`) and
/// `global/defaults.toml` content.
async fn mount_metadata_and_defaults(server: &MockServer, org: &str, defaults_toml: &str) {
    Mock::given(method("GET"))
        .and(path(format!("/repos/{org}/.reporoller")))
        .respond_with(ResponseTemplate::new(200).set_body_json(repo_json(org, ".reporoller")))
        .mount(server)
        .await;

    Mock::given(method("GET"))
        .and(path(format!(
            "/repos/{org}/.reporoller/contents/global/defaults.toml"
        )))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(file_content_response(
                org,
                ".reporoller",
                "global/defaults.toml",
                defaults_toml,
            )),
        )
        .mount(server)
        .await;
}

/// Mounts a repository-type configuration file at
/// `types/{type_name}/config.toml`.
async fn mount_repository_type(server: &MockServer, org: &str, type_name: &str, toml: &str) {
    let file_path = format!("types/{type_name}/config.toml");
    Mock::given(method("GET"))
        .and(path(format!(
            "/repos/{org}/.reporoller/contents/{file_path}"
        )))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(file_content_response(
                org,
                ".reporoller",
                &file_path,
                toml,
            )),
        )
        .mount(server)
        .await;
}

/// Mounts a team configuration file at `teams/{team_name}/config.toml`.
async fn mount_team(server: &MockServer, org: &str, team_name: &str, toml: &str) {
    let file_path = format!("teams/{team_name}/config.toml");
    Mock::given(method("GET"))
        .and(path(format!(
            "/repos/{org}/.reporoller/contents/{file_path}"
        )))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(file_content_response(
                org,
                ".reporoller",
                &file_path,
                toml,
            )),
        )
        .mount(server)
        .await;
}

/// Mounts the template-discovery search (`GET /search/repositories`) used by
/// `find_designated_template`'s `list_templates` call, plus, for each
/// `(slug, template_toml)` pair, the per-template existence check
/// (`GET /repos/{org}/{slug}`) and `.reporoller/template.toml` content fetch.
///
/// Every test that exercises the real handler must call this (even with an
/// empty `templates` slice) because the handler always performs all three
/// designation lookups (Fork/SourceHistoryMirror/SourceOnlyMirror) regardless
/// of query params — an unmounted search endpoint would otherwise 404 and
/// propagate as an unrelated 500.
async fn mount_template_discovery(server: &MockServer, org: &str, templates: &[(&str, &str)]) {
    let slugs: Vec<&str> = templates.iter().map(|(slug, _)| *slug).collect();
    Mock::given(method("GET"))
        .and(path("/search/repositories"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(search_repositories_response(org, &slugs)),
        )
        .mount(server)
        .await;

    for (slug, toml) in templates {
        Mock::given(method("GET"))
            .and(path(format!("/repos/{org}/{slug}")))
            .respond_with(ResponseTemplate::new(200).set_body_json(repo_json(org, slug)))
            .mount(server)
            .await;

        Mock::given(method("GET"))
            .and(path(format!(
                "/repos/{org}/{slug}/contents/.reporoller/template.toml"
            )))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(file_content_response(
                    org,
                    slug,
                    ".reporoller/template.toml",
                    toml,
                )),
            )
            .mount(server)
            .await;
    }
}

/// Convenience wrapper: mounts metadata/global-defaults plus template
/// discovery in one call, covering the common case where only global
/// defaults and template designations are under test (no repository-type or
/// team overrides).
async fn mount_full_resolution(
    server: &MockServer,
    org: &str,
    defaults_toml: &str,
    templates: &[(&str, &str)],
) {
    mount_metadata_and_defaults(server, org, defaults_toml).await;
    mount_template_discovery(server, org, templates).await;
}

/// Builds the unauthenticated test router, pointed at `mock_server`, with a
/// synthetic `AuthContext` injected ahead of the handler (mirroring the
/// pattern every other handler test in this crate uses).
fn test_app(mock_server: &MockServer) -> axum::Router {
    let state = AppState::default()
        .with_github_api_base_url(mock_server.uri())
        .with_mock_installation_token("x");
    create_router_without_auth(state).layer(middleware::from_fn(
        |mut req: axum::extract::Request, next: axum::middleware::Next| async move {
            req.extensions_mut()
                .insert(crate::middleware::AuthContext::new());
            next.run(req).await
        },
    ))
}

/// Builds a global `defaults.toml` body with the three capability flags and
/// an allow-list, using the explicit `{ value = ..., override_allowed = ... }`
/// `OverridableValue` TOML format (see
/// `config_manager::global_defaults_creation_capabilities_tests`).
fn capabilities_toml(fork: bool, history_mirror: bool, only_mirror: bool, owners: &[&str]) -> String {
    let owners_toml = owners
        .iter()
        .map(|o| format!("\"{o}\""))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "[creation_capabilities]\n\
         fork_enabled = {{ value = {fork}, override_allowed = true }}\n\
         source_history_mirror_enabled = {{ value = {history_mirror}, override_allowed = true }}\n\
         source_only_mirror_enabled = {{ value = {only_mirror}, override_allowed = true }}\n\
         allowed_source_owners = {{ value = [{owners_toml}], override_allowed = true }}\n"
    )
}

/// TOML with no `[creation_capabilities]` section at all — exercises the
/// "unset at every level" fail-closed default path.
fn no_capabilities_toml() -> String {
    "[repository]\nissues = true\n".to_string()
}

/// Builds a `.reporoller/template.toml` body with an optional designation.
fn template_toml_with_designation(name: &str, designation: Option<&str>) -> String {
    let designation_line = designation
        .map(|d| format!("designation = \"{d}\"\n"))
        .unwrap_or_default();
    format!(
        "{designation_line}[template]\nname = \"{name}\"\ndescription = \"test\"\nauthor = \"test\"\ntags = []\n"
    )
}

// ============================================================================
// Tier 1: Specification tests (Assertion IAA-003)
// ============================================================================

/// Zero query params: must resolve purely from Global defaults and return 200.
#[tokio::test]
async fn test_creation_capabilities_zero_query_params_resolves_global_only() {
    let mock_server = MockServer::start().await;
    let defaults = capabilities_toml(true, false, false, &["upstream-org"]);
    mount_full_resolution(&mock_server, "testorg", &defaults, &[]).await;

    let app = test_app(&mock_server);
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/creation-capabilities")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(
        status,
        StatusCode::OK,
        "expected 200 for zero-query-param (global-only) resolution; body: {}",
        String::from_utf8_lossy(&body)
    );

    let resp: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(resp["forkEnabled"], true);
    assert_eq!(resp["sourceHistoryMirrorEnabled"], false);
    assert_eq!(resp["sourceOnlyMirrorEnabled"], false);
    assert_eq!(resp["allowedSourceOwners"], json!(["upstream-org"]));
    assert_eq!(resp["forkTemplate"], Value::Null);
    assert_eq!(resp["sourceHistoryMirrorTemplate"], Value::Null);
    assert_eq!(resp["sourceOnlyMirrorTemplate"], Value::Null);
}

/// A disabled capability must be reported as `false` with HTTP 200 — never an
/// error. This is the precise wording of IAA-003's last clause.
#[tokio::test]
async fn test_creation_capabilities_disabled_capability_returns_false_with_200_not_error() {
    let mock_server = MockServer::start().await;
    let defaults = capabilities_toml(false, false, false, &[]);
    mount_full_resolution(&mock_server, "testorg", &defaults, &[]).await;

    let app = test_app(&mock_server);
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/creation-capabilities")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "a disabled capability must be reported via 200, never an error status"
    );
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let resp: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(resp["forkEnabled"], false);
    assert_eq!(resp["sourceHistoryMirrorEnabled"], false);
    assert_eq!(resp["sourceOnlyMirrorEnabled"], false);
}

/// `allowedSourceOwners` must be `[]`, not omitted/null, when unset at every
/// configuration level (fail-closed security default, IAA-003).
#[tokio::test]
async fn test_creation_capabilities_allowed_source_owners_empty_array_when_unset_everywhere() {
    let mock_server = MockServer::start().await;
    mount_full_resolution(&mock_server, "testorg", &no_capabilities_toml(), &[]).await;

    let app = test_app(&mock_server);
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/creation-capabilities")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let resp: Value = serde_json::from_slice(&body).unwrap();

    assert!(
        resp.get("allowedSourceOwners").is_some(),
        "allowedSourceOwners key must be present, never omitted; got: {resp}"
    );
    let owners = resp["allowedSourceOwners"]
        .as_array()
        .expect("allowedSourceOwners must be a JSON array, never null");
    assert!(
        owners.is_empty(),
        "allowedSourceOwners must be [] when unset at every level; got: {resp}"
    );
    // Also verify the fail-closed flags in the same response.
    assert_eq!(resp["forkEnabled"], false);
    assert_eq!(resp["sourceHistoryMirrorEnabled"], false);
    assert_eq!(resp["sourceOnlyMirrorEnabled"], false);
}

/// Exactly one template designated for Fork must be reported by name.
#[tokio::test]
async fn test_creation_capabilities_fork_template_returns_designated_template_name() {
    let mock_server = MockServer::start().await;
    let defaults = capabilities_toml(true, false, false, &[]);
    let templates = [
        (
            "fork-settings",
            template_toml_with_designation("Fork Settings", Some("fork")),
        ),
        (
            "plain-template",
            template_toml_with_designation("Plain", None),
        ),
    ];
    let templates_ref: Vec<(&str, &str)> = templates
        .iter()
        .map(|(slug, toml)| (*slug, toml.as_str()))
        .collect();
    mount_full_resolution(&mock_server, "testorg", &defaults, &templates_ref).await;

    let app = test_app(&mock_server);
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/creation-capabilities")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let resp: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(resp["forkTemplate"], "fork-settings");
    assert_eq!(resp["sourceHistoryMirrorTemplate"], Value::Null);
    assert_eq!(resp["sourceOnlyMirrorTemplate"], Value::Null);
}

/// No template declares a designation at all: `forkTemplate` must be `null`.
#[tokio::test]
async fn test_creation_capabilities_fork_template_null_when_no_template_designated() {
    let mock_server = MockServer::start().await;
    let defaults = capabilities_toml(true, false, false, &[]);
    let templates = [(
        "plain-template",
        template_toml_with_designation("Plain", None),
    )];
    let templates_ref: Vec<(&str, &str)> = templates
        .iter()
        .map(|(slug, toml)| (*slug, toml.as_str()))
        .collect();
    mount_full_resolution(&mock_server, "testorg", &defaults, &templates_ref).await;

    let app = test_app(&mock_server);
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/creation-capabilities")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let resp: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        resp["forkTemplate"],
        Value::Null,
        "no template declares designation=fork; forkTemplate must be null; got: {resp}"
    );
}

/// Source+history mirror designation is reported on the correct field, not
/// cross-wired to `forkTemplate` or `sourceOnlyMirrorTemplate`.
#[tokio::test]
async fn test_creation_capabilities_source_history_mirror_template_returns_designated_name() {
    let mock_server = MockServer::start().await;
    let defaults = capabilities_toml(false, true, false, &[]);
    let templates = [(
        "history-mirror-settings",
        template_toml_with_designation("History Mirror Settings", Some("source_history_mirror")),
    )];
    let templates_ref: Vec<(&str, &str)> = templates
        .iter()
        .map(|(slug, toml)| (*slug, toml.as_str()))
        .collect();
    mount_full_resolution(&mock_server, "testorg", &defaults, &templates_ref).await;

    let app = test_app(&mock_server);
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/creation-capabilities")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let resp: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(resp["sourceHistoryMirrorTemplate"], "history-mirror-settings");
    assert_eq!(
        resp["forkTemplate"],
        Value::Null,
        "the history-mirror designation must not leak onto forkTemplate"
    );
    assert_eq!(resp["sourceOnlyMirrorTemplate"], Value::Null);
}

/// Source-only mirror designation is reported on the correct field.
#[tokio::test]
async fn test_creation_capabilities_source_only_mirror_template_returns_designated_name() {
    let mock_server = MockServer::start().await;
    let defaults = capabilities_toml(false, false, true, &[]);
    let templates = [(
        "only-mirror-settings",
        template_toml_with_designation("Only Mirror Settings", Some("source_only_mirror")),
    )];
    let templates_ref: Vec<(&str, &str)> = templates
        .iter()
        .map(|(slug, toml)| (*slug, toml.as_str()))
        .collect();
    mount_full_resolution(&mock_server, "testorg", &defaults, &templates_ref).await;

    let app = test_app(&mock_server);
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/creation-capabilities")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let resp: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(resp["sourceOnlyMirrorTemplate"], "only-mirror-settings");
    assert_eq!(resp["forkTemplate"], Value::Null);
    assert_eq!(resp["sourceHistoryMirrorTemplate"], Value::Null);
}

/// An organization with zero templates at all must still succeed with all
/// three template fields `null` (not a 404/500 from the designation lookup).
#[tokio::test]
async fn test_creation_capabilities_works_with_zero_templates_in_org() {
    let mock_server = MockServer::start().await;
    let defaults = capabilities_toml(true, true, true, &["some-org"]);
    mount_full_resolution(&mock_server, "testorg", &defaults, &[]).await;

    let app = test_app(&mock_server);
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/creation-capabilities")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let resp: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(resp["forkTemplate"], Value::Null);
    assert_eq!(resp["sourceHistoryMirrorTemplate"], Value::Null);
    assert_eq!(resp["sourceOnlyMirrorTemplate"], Value::Null);
}

// ----------------------------------------------------------------------------
// Precedence tests: Global -> RepositoryType -> Team
// ----------------------------------------------------------------------------

/// RepositoryType-level setting must override Global for the same field.
#[tokio::test]
async fn test_creation_capabilities_repository_type_overrides_global_for_fork_enabled() {
    let mock_server = MockServer::start().await;
    let global = capabilities_toml(false, false, false, &[]);
    mount_full_resolution(&mock_server, "testorg", &global, &[]).await;
    let repo_type_toml = "[creation_capabilities]\nfork_enabled = { value = true, override_allowed = true }\n";
    mount_repository_type(&mock_server, "testorg", "service", repo_type_toml).await;

    let app = test_app(&mock_server);
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/creation-capabilities?repositoryType=service")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(
        status,
        StatusCode::OK,
        "body: {}",
        String::from_utf8_lossy(&body)
    );
    let resp: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        resp["forkEnabled"], true,
        "RepositoryType must override Global's false; got: {resp}"
    );
}

/// Team-level setting must override RepositoryType for the same field.
#[tokio::test]
async fn test_creation_capabilities_team_overrides_repository_type_for_source_history_mirror() {
    let mock_server = MockServer::start().await;
    let global = capabilities_toml(false, false, false, &[]);
    mount_full_resolution(&mock_server, "testorg", &global, &[]).await;
    let repo_type_toml = "[creation_capabilities]\nsource_history_mirror_enabled = { value = false, override_allowed = true }\n";
    mount_repository_type(&mock_server, "testorg", "service", repo_type_toml).await;
    let team_toml = "[creation_capabilities]\nsource_history_mirror_enabled = { value = true, override_allowed = true }\n";
    mount_team(&mock_server, "testorg", "platform", team_toml).await;

    let app = test_app(&mock_server);
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/creation-capabilities?team=platform&repositoryType=service")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(
        status,
        StatusCode::OK,
        "body: {}",
        String::from_utf8_lossy(&body)
    );
    let resp: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        resp["sourceHistoryMirrorEnabled"], true,
        "Team must override RepositoryType's false; got: {resp}"
    );
}

/// Team-level setting must override Global directly, with no
/// `repositoryType` query param present at all.
#[tokio::test]
async fn test_creation_capabilities_team_overrides_global_without_repository_type_param() {
    let mock_server = MockServer::start().await;
    let global = capabilities_toml(false, false, false, &[]);
    mount_full_resolution(&mock_server, "testorg", &global, &[]).await;
    let team_toml = "[creation_capabilities]\nsource_only_mirror_enabled = { value = true, override_allowed = true }\n";
    mount_team(&mock_server, "testorg", "platform", team_toml).await;

    let app = test_app(&mock_server);
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/creation-capabilities?team=platform")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let resp: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(resp["sourceOnlyMirrorEnabled"], true);
}

/// `allowedSourceOwners` must use the highest-precedence level that sets it —
/// Team's list must *replace* Global's, not be unioned with it.
#[tokio::test]
async fn test_creation_capabilities_allowed_source_owners_highest_precedence_wins_not_additive() {
    let mock_server = MockServer::start().await;
    let global = capabilities_toml(true, true, true, &["global-org"]);
    mount_full_resolution(&mock_server, "testorg", &global, &[]).await;
    let team_toml = "[creation_capabilities]\nallowed_source_owners = { value = [\"team-org\"], override_allowed = true }\n";
    mount_team(&mock_server, "testorg", "platform", team_toml).await;

    let app = test_app(&mock_server);
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/creation-capabilities?team=platform")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let resp: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        resp["allowedSourceOwners"],
        json!(["team-org"]),
        "Team's allow-list must replace Global's wholesale, not union with it; got: {resp}"
    );
}

/// A team overriding only ONE capability field must not perturb the other
/// two fields, which must still fall through to Global. Kills a "whole
/// struct replacement" stub that would also blank the untouched fields.
#[tokio::test]
async fn test_creation_capabilities_field_level_override_does_not_perturb_sibling_fields() {
    let mock_server = MockServer::start().await;
    // Global: fork=true, history-mirror=true, only-mirror=false.
    let global = capabilities_toml(true, true, false, &[]);
    mount_full_resolution(&mock_server, "testorg", &global, &[]).await;
    // Team overrides ONLY fork_enabled -> false; leaves the other two fields
    // (and allowed_source_owners) entirely absent from its TOML.
    let team_toml =
        "[creation_capabilities]\nfork_enabled = { value = false, override_allowed = true }\n";
    mount_team(&mock_server, "testorg", "platform", team_toml).await;

    let app = test_app(&mock_server);
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/creation-capabilities?team=platform")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let resp: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        resp["forkEnabled"], false,
        "team's explicit override must win for fork_enabled; got: {resp}"
    );
    assert_eq!(
        resp["sourceHistoryMirrorEnabled"], true,
        "untouched sibling field must fall through to Global's true, not be reset; got: {resp}"
    );
    assert_eq!(
        resp["sourceOnlyMirrorEnabled"], false,
        "untouched sibling field must retain Global's false; got: {resp}"
    );
}

/// Contract test: the response body is camelCase end-to-end, with the exact
/// seven keys the interface contract specifies — no snake_case leakage, no
/// extra/missing keys (this is a purpose-built response type, not a raw
/// `serde_json::to_value` dump of a `config_manager` struct, unlike
/// `/defaults` and `/configuration/preview`).
#[tokio::test]
async fn test_creation_capabilities_response_is_camelcase_with_exact_field_set() {
    let mock_server = MockServer::start().await;
    let defaults = capabilities_toml(true, false, false, &["upstream-org"]);
    mount_full_resolution(&mock_server, "testorg", &defaults, &[]).await;

    let app = test_app(&mock_server);
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/creation-capabilities")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let resp: Value = serde_json::from_slice(&body).unwrap();
    let obj = resp.as_object().expect("response body must be a JSON object");

    let expected: std::collections::BTreeSet<&str> = [
        "forkEnabled",
        "sourceHistoryMirrorEnabled",
        "sourceOnlyMirrorEnabled",
        "allowedSourceOwners",
        "forkTemplate",
        "sourceHistoryMirrorTemplate",
        "sourceOnlyMirrorTemplate",
    ]
    .into_iter()
    .collect();
    let actual: std::collections::BTreeSet<&str> = obj.keys().map(String::as_str).collect();

    assert_eq!(
        actual, expected,
        "response must contain exactly the camelCase field set from the interface contract"
    );

    for snake_case_leak in [
        "fork_enabled",
        "source_history_mirror_enabled",
        "source_only_mirror_enabled",
        "allowed_source_owners",
        "fork_template",
    ] {
        assert!(
            obj.get(snake_case_leak).is_none(),
            "response must not contain snake_case key '{snake_case_leak}'"
        );
    }
}

// ============================================================================
// Tier 2: Adversarial tests (Assertion IAA-004 + boundary/security/routing)
// ============================================================================

/// Two templates declaring the same (Fork) designation must fail the whole
/// request with 409 AmbiguousTemplateDesignation.
#[tokio::test]
async fn test_creation_capabilities_two_templates_same_fork_designation_returns_409() {
    let mock_server = MockServer::start().await;
    let defaults = capabilities_toml(true, false, false, &[]);
    let templates = [
        (
            "fork-a",
            template_toml_with_designation("Fork A", Some("fork")),
        ),
        (
            "fork-b",
            template_toml_with_designation("Fork B", Some("fork")),
        ),
    ];
    let templates_ref: Vec<(&str, &str)> = templates
        .iter()
        .map(|(slug, toml)| (*slug, toml.as_str()))
        .collect();
    mount_full_resolution(&mock_server, "testorg", &defaults, &templates_ref).await;

    let app = test_app(&mock_server);
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/creation-capabilities")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "expected 409 for ambiguous Fork designation; body: {}",
        String::from_utf8_lossy(&body)
    );
    let resp: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(resp["error"]["code"], "AmbiguousTemplateDesignation");
}

/// Ambiguity detection must not be Fork-specific: two SourceOnlyMirror
/// templates must also trigger 409.
#[tokio::test]
async fn test_creation_capabilities_two_templates_same_source_only_mirror_designation_returns_409()
{
    let mock_server = MockServer::start().await;
    let defaults = capabilities_toml(false, false, true, &[]);
    let templates = [
        (
            "only-a",
            template_toml_with_designation("Only A", Some("source_only_mirror")),
        ),
        (
            "only-b",
            template_toml_with_designation("Only B", Some("source_only_mirror")),
        ),
    ];
    let templates_ref: Vec<(&str, &str)> = templates
        .iter()
        .map(|(slug, toml)| (*slug, toml.as_str()))
        .collect();
    mount_full_resolution(&mock_server, "testorg", &defaults, &templates_ref).await;

    let app = test_app(&mock_server);
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/creation-capabilities")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

/// If ONE of the three designations is ambiguous, the whole request must
/// fail — even though the other two designations resolve cleanly. Kills a
/// stub that silently drops/nulls the failing designation and returns 200
/// with partial data.
#[tokio::test]
async fn test_creation_capabilities_ambiguity_in_any_one_designation_fails_whole_request() {
    let mock_server = MockServer::start().await;
    let defaults = capabilities_toml(true, false, true, &[]);
    let templates = [
        (
            "fork-a",
            template_toml_with_designation("Fork A", Some("fork")),
        ),
        (
            "only-a",
            template_toml_with_designation("Only A", Some("source_only_mirror")),
        ),
        (
            "only-b",
            template_toml_with_designation("Only B", Some("source_only_mirror")),
        ),
    ];
    let templates_ref: Vec<(&str, &str)> = templates
        .iter()
        .map(|(slug, toml)| (*slug, toml.as_str()))
        .collect();
    mount_full_resolution(&mock_server, "testorg", &defaults, &templates_ref).await;

    let app = test_app(&mock_server);
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/creation-capabilities")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::CONFLICT,
        "a single ambiguous designation must fail the whole response, not be silently dropped"
    );
}

/// An unknown `repositoryType` query value must return 400, reusing the same
/// pattern `preview_configuration` already uses.
#[tokio::test]
async fn test_creation_capabilities_unknown_repository_type_returns_400() {
    let mock_server = MockServer::start().await;
    let defaults = capabilities_toml(true, false, false, &[]);
    mount_full_resolution(&mock_server, "testorg", &defaults, &[]).await;
    // `types/nonexistent/config.toml` intentionally left unmounted (404 by default).

    let app = test_app(&mock_server);
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/creation-capabilities?repositoryType=nonexistent")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "unknown repositoryType query value must return 400"
    );
}

/// A repository type that exists but sets no `creation_capabilities` at all
/// must not error — resolution simply falls through to Global.
#[tokio::test]
async fn test_creation_capabilities_known_repository_type_with_no_overrides_succeeds() {
    let mock_server = MockServer::start().await;
    let defaults = capabilities_toml(true, false, false, &[]);
    mount_full_resolution(&mock_server, "testorg", &defaults, &[]).await;
    // Repository type file exists but only sets unrelated settings.
    mount_repository_type(
        &mock_server,
        "testorg",
        "service",
        "[repository]\nwiki = false\n",
    )
    .await;

    let app = test_app(&mock_server);
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/creation-capabilities?repositoryType=service")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(
        status,
        StatusCode::OK,
        "body: {}",
        String::from_utf8_lossy(&body)
    );
    let resp: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(resp["forkEnabled"], true, "must fall through to Global");
}

/// A `repositoryType` value containing path-traversal characters must be
/// rejected cleanly as 400, never surfaced as an unhandled 500.
#[tokio::test]
async fn test_creation_capabilities_path_traversal_in_repository_type_returns_400_not_500() {
    let mock_server = MockServer::start().await;
    let defaults = capabilities_toml(true, false, false, &[]);
    mount_full_resolution(&mock_server, "testorg", &defaults, &[]).await;

    let app = test_app(&mock_server);
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/creation-capabilities?repositoryType=../../etc")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_ne!(
        response.status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "path-traversal repositoryType must not surface as 500"
    );
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

/// A `team` value containing path-traversal characters must degrade to a
/// clean client error, never an unhandled 500.
#[tokio::test]
async fn test_creation_capabilities_path_traversal_in_team_does_not_return_500() {
    let mock_server = MockServer::start().await;
    let defaults = capabilities_toml(true, false, false, &[]);
    mount_full_resolution(&mock_server, "testorg", &defaults, &[]).await;

    let app = test_app(&mock_server);
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/creation-capabilities?team=../../etc")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_ne!(
        response.status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "path-traversal team value must not surface as an unhandled 500"
    );
}

/// Malformed TOML in `global/defaults.toml` must surface as 400
/// (`ConfigurationParseError`), not an unhandled 500.
#[tokio::test]
async fn test_creation_capabilities_malformed_global_defaults_toml_returns_400_not_500() {
    let mock_server = MockServer::start().await;
    mount_metadata_and_defaults(&mock_server, "testorg", "not [ valid } toml ===").await;
    mount_template_discovery(&mock_server, "testorg", &[]).await;

    let app = test_app(&mock_server);
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/creation-capabilities")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "malformed global defaults TOML must map to 400, not 500"
    );
}

/// Route must be registered in `organization_routes()` (the auth-required
/// variant): without an `Authorization` header the auth middleware must
/// reject with 401 — a 404 here would mean the route is missing from the
/// authenticated router, not merely unauthenticated.
#[tokio::test]
async fn test_creation_capabilities_route_registered_in_auth_required_router() {
    let state = AppState::default();
    let router = create_router(state);

    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/creation-capabilities")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "expected 401 (route exists, auth required) not 404 (route missing)"
    );
}

/// Route must be registered in `organization_routes_without_auth()` (the
/// test-only variant) and respond 200 for a valid request.
#[tokio::test]
async fn test_creation_capabilities_route_registered_in_no_auth_router() {
    let mock_server = MockServer::start().await;
    let defaults = capabilities_toml(true, false, false, &[]);
    mount_full_resolution(&mock_server, "testorg", &defaults, &[]).await;

    let app = test_app(&mock_server);
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/creation-capabilities")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

/// The route must be GET-only: a POST to the same path must not be silently
/// accepted (405 Method Not Allowed, not 200/404). Catches an overly broad
/// route registration (e.g. accidentally wired with `any()` or duplicated
/// under `post()`).
#[tokio::test]
async fn test_creation_capabilities_rejects_post_method() {
    let mock_server = MockServer::start().await;
    let defaults = capabilities_toml(true, false, false, &[]);
    mount_full_resolution(&mock_server, "testorg", &defaults, &[]).await;

    let app = test_app(&mock_server);
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/orgs/testorg/creation-capabilities")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::METHOD_NOT_ALLOWED,
        "the discovery endpoint must be GET-only"
    );
}

/// Security rule: this is a read-only discovery endpoint — it must never
/// issue a mutating (POST/PUT/PATCH/DELETE) call to the GitHub API. Verifies
/// by inspecting every request actually received by the mock GitHub server.
#[tokio::test]
async fn test_creation_capabilities_zero_mutating_http_calls_made_to_github() {
    let mock_server = MockServer::start().await;
    let defaults = capabilities_toml(true, true, true, &["upstream-org"]);
    let templates = [(
        "fork-settings",
        template_toml_with_designation("Fork Settings", Some("fork")),
    )];
    let templates_ref: Vec<(&str, &str)> = templates
        .iter()
        .map(|(slug, toml)| (*slug, toml.as_str()))
        .collect();
    mount_full_resolution(&mock_server, "testorg", &defaults, &templates_ref).await;
    mount_repository_type(
        &mock_server,
        "testorg",
        "service",
        "[creation_capabilities]\nfork_enabled = { value = true, override_allowed = true }\n",
    )
    .await;
    mount_team(
        &mock_server,
        "testorg",
        "platform",
        "[creation_capabilities]\nfork_enabled = { value = true, override_allowed = true }\n",
    )
    .await;

    let app = test_app(&mock_server);
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/creation-capabilities?team=platform&repositoryType=service")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let received = mock_server.received_requests().await.expect(
        "wiremock request recording must be enabled by default for MockServer::start()",
    );
    assert!(!received.is_empty(), "expected at least one GitHub API call");
    for req in &received {
        assert_eq!(
            req.method.as_str(),
            "GET",
            "creation-capabilities must only perform read (GET) calls to GitHub; observed a {} to {}",
            req.method,
            req.url
        );
    }
}

/// `allowedSourceOwners` must preserve the configured casing and order
/// exactly — no silent lowercasing, sorting, or deduplication that would
/// misrepresent the security-critical allow-list to the caller.
#[tokio::test]
async fn test_creation_capabilities_allowed_source_owners_preserves_casing_and_order() {
    let mock_server = MockServer::start().await;
    let defaults = capabilities_toml(true, false, false, &["Upstream-Org", "some-VENDOR-org"]);
    mount_full_resolution(&mock_server, "testorg", &defaults, &[]).await;

    let app = test_app(&mock_server);
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/creation-capabilities")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let resp: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        resp["allowedSourceOwners"],
        json!(["Upstream-Org", "some-VENDOR-org"]),
        "owner casing and order must be reported exactly as configured; got: {resp}"
    );
}

/// A capability being enabled must NOT imply a non-empty allow-list — the two
/// settings are independent and the endpoint must report the allow-list
/// faithfully even when some other capability is enabled.
#[tokio::test]
async fn test_creation_capabilities_enabled_capability_does_not_imply_non_empty_owners() {
    let mock_server = MockServer::start().await;
    // fork_enabled=true, but allowed_source_owners key entirely absent.
    let defaults =
        "[creation_capabilities]\nfork_enabled = { value = true, override_allowed = true }\n"
            .to_string();
    mount_full_resolution(&mock_server, "testorg", &defaults, &[]).await;

    let app = test_app(&mock_server);
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/creation-capabilities")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let resp: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(resp["forkEnabled"], true);
    let owners = resp["allowedSourceOwners"]
        .as_array()
        .expect("allowedSourceOwners must be an array");
    assert!(
        owners.is_empty(),
        "an enabled capability must not imply a permissive/non-empty allow-list; got: {resp}"
    );
}

// ============================================================================
// Tier 1 (continued): Direct struct-level contract tests (no router)
// ============================================================================

/// Absent designation fields must serialize as JSON `null`, not be omitted
/// from the object — the response type uses plain `Option<String>` fields
/// with no `skip_serializing_if`.
#[test]
fn test_creation_capabilities_response_struct_serializes_absent_templates_as_null() {
    let response = CreationCapabilitiesResponse {
        fork_enabled: false,
        source_history_mirror_enabled: false,
        source_only_mirror_enabled: false,
        allowed_source_owners: vec![],
        fork_template: None,
        source_history_mirror_template: None,
        source_only_mirror_template: None,
    };

    let value = serde_json::to_value(&response).expect("response must serialize");
    assert_eq!(
        value.get("forkTemplate"),
        Some(&Value::Null),
        "forkTemplate key must be present as null, not omitted; got: {value}"
    );
    assert_eq!(value.get("sourceHistoryMirrorTemplate"), Some(&Value::Null));
    assert_eq!(value.get("sourceOnlyMirrorTemplate"), Some(&Value::Null));
}

/// An empty `allowed_source_owners` list must serialize as `[]`, never
/// omitted — this is the direct struct-level counterpart to the router-level
/// test above, isolating the serialization contract from handler logic.
#[test]
fn test_creation_capabilities_response_struct_serializes_empty_owners_as_empty_array() {
    let response = CreationCapabilitiesResponse {
        fork_enabled: true,
        source_history_mirror_enabled: true,
        source_only_mirror_enabled: true,
        allowed_source_owners: vec![],
        fork_template: Some("x".to_string()),
        source_history_mirror_template: Some("y".to_string()),
        source_only_mirror_template: Some("z".to_string()),
    };

    let value = serde_json::to_value(&response).expect("response must serialize");
    let owners = value
        .get("allowedSourceOwners")
        .expect("allowedSourceOwners key must be present")
        .as_array()
        .expect("allowedSourceOwners must serialize as an array");
    assert!(owners.is_empty());
}

/// Each of the three template fields must serialize to its OWN distinct
/// value, not be cross-wired. Using three distinct string values ("x", "y",
/// "z") makes any field-swap mutation (e.g. `fork_template` accidentally
/// populated from `source_only_mirror_template`) immediately observable.
#[test]
fn test_creation_capabilities_response_struct_does_not_cross_wire_template_fields() {
    let response = CreationCapabilitiesResponse {
        fork_enabled: true,
        source_history_mirror_enabled: true,
        source_only_mirror_enabled: true,
        allowed_source_owners: vec!["owner-a".to_string()],
        fork_template: Some("x".to_string()),
        source_history_mirror_template: Some("y".to_string()),
        source_only_mirror_template: Some("z".to_string()),
    };

    let value = serde_json::to_value(&response).expect("response must serialize");
    assert_eq!(value["forkTemplate"], "x");
    assert_eq!(value["sourceHistoryMirrorTemplate"], "y");
    assert_eq!(value["sourceOnlyMirrorTemplate"], "z");
}

/// `CreationCapabilitiesQuery` deserializes `repositoryType` (camelCase wire
/// field) into the `repository_type` Rust field, leaving `team` as `None`
/// when absent. Kills a field-name/rename_all regression.
#[test]
fn test_creation_capabilities_query_deserializes_repository_type_camelcase() {
    let query: CreationCapabilitiesQuery =
        serde_json::from_value(json!({ "repositoryType": "service" }))
            .expect("query struct must deserialize");
    assert_eq!(query.repository_type, Some("service".to_string()));
    assert_eq!(query.team, None);
}

/// `CreationCapabilitiesQuery` deserializes `team` without disturbing
/// `repository_type` — distinct value from the test above, in case a
/// copy/paste mutation swaps the two fields.
#[test]
fn test_creation_capabilities_query_deserializes_team_without_affecting_repository_type() {
    let query: CreationCapabilitiesQuery = serde_json::from_value(json!({ "team": "platform" }))
        .expect("query struct must deserialize");
    assert_eq!(query.team, Some("platform".to_string()));
    assert_eq!(query.repository_type, None);
}

/// Both query fields absent (`{}`) must deserialize successfully with both
/// `None` — the zero-query-param case the router-level test above exercises
/// end-to-end; this isolates just the struct's `Option` defaulting.
#[test]
fn test_creation_capabilities_query_both_fields_absent_deserialize_to_none() {
    let query: CreationCapabilitiesQuery =
        serde_json::from_value(json!({})).expect("query struct must deserialize from {}");
    assert_eq!(query.team, None);
    assert_eq!(query.repository_type, None);
}

// ============================================================================
// Tier 3: Property-based tests
// ============================================================================

mod property_tests {
    use super::*;
    use proptest::prelude::*;

    /// Runs an async future to completion on a fresh current-thread Tokio
    /// runtime. `proptest!`'s `#[test]` functions are synchronous, and
    /// wiremock's `MockServer` requires an async executor, so each property
    /// test case gets its own short-lived runtime (mirroring the common
    /// `tokio::runtime::Builder` pattern used elsewhere for sync/async
    /// boundary crossing, e.g. `tracing_otel.rs` in this same crate).
    fn block_on<F: std::future::Future>(fut: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build a current-thread Tokio runtime for a property test case")
            .block_on(fut)
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20))]

        /// For any combination of the three boolean capability flags set at
        /// Global level (no team/repositoryType/template involved), the
        /// response must report exactly those values — a direct round-trip
        /// invariant that a hardcoded-`true`/hardcoded-`false` stub cannot
        /// satisfy across all eight combinations.
        #[test]
        fn prop_capability_flags_round_trip_through_global_defaults(
            fork in any::<bool>(),
            history_mirror in any::<bool>(),
            only_mirror in any::<bool>(),
        ) {
            block_on(async {
                let mock_server = MockServer::start().await;
                let defaults = capabilities_toml(fork, history_mirror, only_mirror, &[]);
                mount_full_resolution(&mock_server, "testorg", &defaults, &[]).await;

                let app = test_app(&mock_server);
                let request = Request::builder()
                    .method("GET")
                    .uri("/api/v1/orgs/testorg/creation-capabilities")
                    .body(Body::empty())
                    .unwrap();

                let response = app.oneshot(request).await.unwrap();
                prop_assert_eq!(response.status(), StatusCode::OK);
                let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                    .await
                    .unwrap();
                let resp: Value = serde_json::from_slice(&body).unwrap();

                prop_assert_eq!(resp["forkEnabled"].as_bool(), Some(fork));
                prop_assert_eq!(resp["sourceHistoryMirrorEnabled"].as_bool(), Some(history_mirror));
                prop_assert_eq!(resp["sourceOnlyMirrorEnabled"].as_bool(), Some(only_mirror));
                Ok(())
            })?;
        }

        /// For any list of up to 5 simple owner names, the configured list
        /// (order preserved, no dedup/sort/case-fold) must come back exactly
        /// as configured. Also proves the response is always a JSON array
        /// (never `null`) regardless of list length, including zero.
        #[test]
        fn prop_allowed_source_owners_round_trips_exactly(
            owners in proptest::collection::vec(
                proptest::string::string_regex("[a-zA-Z][a-zA-Z0-9-]{0,12}").unwrap(),
                0..5,
            ),
        ) {
            block_on(async {
                let mock_server = MockServer::start().await;
                let owner_refs: Vec<&str> = owners.iter().map(String::as_str).collect();
                let defaults = capabilities_toml(true, true, true, &owner_refs);
                mount_full_resolution(&mock_server, "testorg", &defaults, &[]).await;

                let app = test_app(&mock_server);
                let request = Request::builder()
                    .method("GET")
                    .uri("/api/v1/orgs/testorg/creation-capabilities")
                    .body(Body::empty())
                    .unwrap();

                let response = app.oneshot(request).await.unwrap();
                prop_assert_eq!(response.status(), StatusCode::OK);
                let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                    .await
                    .unwrap();
                let resp: Value = serde_json::from_slice(&body).unwrap();

                let returned = resp["allowedSourceOwners"]
                    .as_array()
                    .expect("allowedSourceOwners must always be a JSON array");
                let returned_strings: Vec<String> = returned
                    .iter()
                    .map(|v| v.as_str().unwrap_or_default().to_string())
                    .collect();
                prop_assert_eq!(returned_strings, owners);
                Ok(())
            })?;
        }

        /// For arbitrary (including malformed/path-traversal-like) `team`
        /// and `repositoryType` query values drawn from the unreserved URL
        /// query character set, the endpoint must never panic and must never
        /// return 500 — only a clean 200/400/404/409. This is the no-panic /
        /// bounded-status invariant required for any caller-controlled query
        /// parameter at an API boundary.
        #[test]
        fn prop_arbitrary_team_and_repository_type_values_never_cause_500(
            team in proptest::string::string_regex("[a-zA-Z0-9_./-]{0,24}").unwrap(),
            repository_type in proptest::string::string_regex("[a-zA-Z0-9_./-]{0,24}").unwrap(),
        ) {
            block_on(async {
                let mock_server = MockServer::start().await;
                let defaults = capabilities_toml(true, false, false, &[]);
                mount_full_resolution(&mock_server, "testorg", &defaults, &[]).await;

                let app = test_app(&mock_server);
                let uri = format!(
                    "/api/v1/orgs/testorg/creation-capabilities?team={team}&repositoryType={repository_type}"
                );
                let request = Request::builder()
                    .method("GET")
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap();

                let response = app.oneshot(request).await.unwrap();
                let status = response.status();
                prop_assert_ne!(
                    status,
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "arbitrary query values must never surface as an unhandled 500"
                );
                let allowed = [
                    StatusCode::OK,
                    StatusCode::BAD_REQUEST,
                    StatusCode::NOT_FOUND,
                    StatusCode::CONFLICT,
                ];
                prop_assert!(
                    allowed.contains(&status),
                    "unexpected status {status} for team={team:?} repositoryType={repository_type:?}"
                );
                // The body must always be valid JSON, even on error paths.
                let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                    .await
                    .unwrap();
                let parsed: Result<Value, _> = serde_json::from_slice(&body);
                prop_assert!(parsed.is_ok(), "response body must always be valid JSON");
                Ok(())
            })?;
        }
    }
}
