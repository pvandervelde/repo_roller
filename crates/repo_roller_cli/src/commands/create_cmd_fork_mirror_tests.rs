//! RED-phase test suite for the CLI fork/mirror request surface
//! (`--fork-from`, `--mirror-from`, `--mirror-history`).
//!
//! See `docs/spec/interfaces/fork-mirror-operations.md` §8a, "Testing
//! Requirements (addendum to §7)".
//!
//! ## Interface contract assumed by these tests (not yet implemented)
//!
//! `CreateArgs` gains three new fields exactly as specified in §8a:
//!
//! ```ignore
//! #[arg(long, value_name = "OWNER/REPO", conflicts_with_all = ["mirror_from", "empty", "init_readme", "init_gitignore"])]
//! pub fork_from: Option<String>,
//!
//! #[arg(long, value_name = "OWNER/REPO", conflicts_with_all = ["fork_from", "empty", "init_readme", "init_gitignore"])]
//! pub mirror_from: Option<String>,
//!
//! #[arg(long, requires = "mirror_from")]
//! pub mirror_history: bool,
//! ```
//!
//! `CreateCommandOptions` must carry the parsed values through to
//! `handle_create_command` (which does not see `CreateArgs` directly). This
//! suite assumes `CreateCommandOptions::new` is extended with three trailing
//! parameters, appended after the existing `collaborators` parameter, in
//! this exact order:
//!
//! ```ignore
//! pub fn new(
//!     config: &'a Option<String>,
//!     name: &'a Option<String>,
//!     owner: &'a Option<String>,
//!     template: &'a Option<String>,
//!     empty: bool,
//!     init_readme: bool,
//!     init_gitignore: bool,
//!     teams: &'a [String],
//!     collaborators: &'a [String],
//!     fork_from: &'a Option<String>,
//!     mirror_from: &'a Option<String>,
//!     mirror_history: bool,
//! ) -> Self
//! ```
//!
//! This mirrors the existing convention of appending new CLI capabilities as
//! trailing positional parameters (as was done for `teams`/`collaborators`).
//! All ~23 pre-existing call sites in `create_cmd_tests.rs` and the one in
//! `main.rs` must be updated by the implementer to pass the three new
//! trailing arguments (e.g. `&None, &None, false`) — this is expected
//! RED-phase fallout from a non-Default struct gaining fields, not a defect
//! in this test suite.

use super::*;
use clap::Parser;
use repo_roller_core::{
    permissions::AccessLevel, CreationMode, OrganizationName, RepoRollerResult,
    RepositoryCreationRequest, RepositoryCreationResult, RepositoryName, SourceRepositoryReference,
    Timestamp,
};
use std::sync::{Arc, Mutex};

// ============================================================================
// Test helpers (mirrors create_cmd_tests.rs conventions)
// ============================================================================

fn make_ask_user_for_value(input_text: &str) -> Result<String, Error> {
    Ok(input_text.to_string())
}

#[derive(Debug, Clone)]
struct CallLog {
    create_repository_args: Vec<RepositoryCreationRequest>,
}

impl CallLog {
    fn new() -> Self {
        Self {
            create_repository_args: Vec::new(),
        }
    }
}

fn make_logged_create_repo_success(
    log: Arc<Mutex<CallLog>>,
) -> impl Fn(
    RepositoryCreationRequest,
) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = RepoRollerResult<RepositoryCreationResult>> + Send>,
> + Send
       + Sync {
    move |req: RepositoryCreationRequest| {
        let log = log.clone();
        Box::pin(async move {
            log.lock().unwrap().create_repository_args.push(req.clone());
            Ok(RepositoryCreationResult {
                repository_url: "https://github.com/test/test-repo".to_string(),
                repository_id: "test-id-123".to_string(),
                created_at: Timestamp::now(),
                default_branch: "main".to_string(),
            })
        })
    }
}

/// A `create_repository_fn` stub that must never be invoked. Used to prove
/// that malformed `--fork-from`/`--mirror-from` values are rejected before
/// any repository-creation call is attempted.
fn make_panicking_create_repo() -> impl Fn(
    RepositoryCreationRequest,
) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = RepoRollerResult<RepositoryCreationResult>> + Send>,
> + Send
       + Sync {
    move |_req: RepositoryCreationRequest| {
        Box::pin(async move {
            panic!(
                "create_repository_fn must not be called for a malformed --fork-from/--mirror-from value"
            );
        })
    }
}

fn expected_source(owner: &str, name: &str) -> SourceRepositoryReference {
    SourceRepositoryReference {
        owner: OrganizationName::new(owner).unwrap(),
        name: RepositoryName::new(name).unwrap(),
    }
}

// ============================================================================
// Tier 1: Clap-level parsing — `CreateArgs` flag shape and constraints
// ============================================================================

/// Minimal wrapper so `CreateArgs` (which derives `clap::Args`, not
/// `clap::Parser`) can be exercised with `try_parse_from` in isolation.
#[derive(Parser, Debug)]
struct TestCli {
    #[command(flatten)]
    args: CreateArgs,
}

fn try_parse(argv: &[&str]) -> Result<CreateArgs, clap::Error> {
    let mut full = vec!["test-bin"];
    full.extend_from_slice(argv);
    TestCli::try_parse_from(full).map(|cli| cli.args)
}

#[test]
fn test_clap_fork_from_alone_parses_to_expected_value() {
    let args = try_parse(&[
        "--name",
        "n",
        "--owner",
        "o",
        "--fork-from",
        "upstream/repo",
    ])
    .expect("valid --fork-from alone must parse");
    assert_eq!(args.fork_from, Some("upstream/repo".to_string()));
}

#[test]
fn test_clap_fork_from_alone_leaves_mirror_from_none() {
    let args = try_parse(&[
        "--name",
        "n",
        "--owner",
        "o",
        "--fork-from",
        "upstream/repo",
    ])
    .expect("valid --fork-from alone must parse");
    assert_eq!(args.mirror_from, None);
}

#[test]
fn test_clap_fork_from_alone_leaves_mirror_history_false() {
    let args = try_parse(&[
        "--name",
        "n",
        "--owner",
        "o",
        "--fork-from",
        "upstream/repo",
    ])
    .expect("valid --fork-from alone must parse");
    assert!(!args.mirror_history);
}

#[test]
fn test_clap_mirror_from_alone_parses_to_expected_value() {
    let args = try_parse(&[
        "--name",
        "n",
        "--owner",
        "o",
        "--mirror-from",
        "upstream/repo",
    ])
    .expect("valid --mirror-from alone must parse");
    assert_eq!(args.mirror_from, Some("upstream/repo".to_string()));
}

#[test]
fn test_clap_mirror_from_alone_defaults_mirror_history_to_false() {
    let args = try_parse(&[
        "--name",
        "n",
        "--owner",
        "o",
        "--mirror-from",
        "upstream/repo",
    ])
    .expect("valid --mirror-from alone must parse");
    assert!(!args.mirror_history);
}

#[test]
fn test_clap_mirror_from_with_mirror_history_sets_true() {
    let args = try_parse(&[
        "--name",
        "n",
        "--owner",
        "o",
        "--mirror-from",
        "upstream/repo",
        "--mirror-history",
    ])
    .expect("--mirror-from with --mirror-history must parse");
    assert!(args.mirror_history);
}

/// Baseline/regression: with none of the new flags present, both new
/// `Option<String>` fields default to `None` and the new bool flag defaults
/// to `false` — a stub that hardcodes `Some(...)` would fail this.
#[test]
fn test_clap_no_fork_or_mirror_flags_present_yields_none_and_false_defaults() {
    let args =
        try_parse(&["--name", "n", "--owner", "o", "--template", "t"]).expect("baseline parses");
    assert_eq!(args.fork_from, None);
    assert_eq!(args.mirror_from, None);
    assert!(!args.mirror_history);
}

/// `--fork-from` takes a value; it must not behave like a boolean flag.
#[test]
fn test_clap_fork_from_without_a_value_is_rejected() {
    let result = try_parse(&["--name", "n", "--owner", "o", "--fork-from"]);
    assert!(result.is_err(), "--fork-from requires a value");
}

#[test]
fn test_clap_mirror_from_without_a_value_is_rejected() {
    let result = try_parse(&["--name", "n", "--owner", "o", "--mirror-from"]);
    assert!(result.is_err(), "--mirror-from requires a value");
}

// ----------------------------------------------------------------------------
// conflicts_with_all enforcement (clap-level, per spec: "verify via its own
// generated error, not custom code")
// ----------------------------------------------------------------------------

#[test]
fn test_clap_fork_from_conflicts_with_mirror_from() {
    let result = try_parse(&[
        "--name",
        "n",
        "--owner",
        "o",
        "--fork-from",
        "a/b",
        "--mirror-from",
        "c/d",
    ]);
    assert!(
        result.is_err(),
        "--fork-from and --mirror-from must conflict"
    );
}

#[test]
fn test_clap_fork_from_conflicts_with_empty() {
    let result = try_parse(&[
        "--name",
        "n",
        "--owner",
        "o",
        "--fork-from",
        "a/b",
        "--empty",
    ]);
    assert!(result.is_err(), "--fork-from and --empty must conflict");
}

#[test]
fn test_clap_fork_from_conflicts_with_init_readme() {
    let result = try_parse(&[
        "--name",
        "n",
        "--owner",
        "o",
        "--fork-from",
        "a/b",
        "--init-readme",
    ]);
    assert!(
        result.is_err(),
        "--fork-from and --init-readme must conflict"
    );
}

#[test]
fn test_clap_fork_from_conflicts_with_init_gitignore() {
    let result = try_parse(&[
        "--name",
        "n",
        "--owner",
        "o",
        "--fork-from",
        "a/b",
        "--init-gitignore",
    ]);
    assert!(
        result.is_err(),
        "--fork-from and --init-gitignore must conflict"
    );
}

#[test]
fn test_clap_mirror_from_conflicts_with_empty() {
    let result = try_parse(&[
        "--name",
        "n",
        "--owner",
        "o",
        "--mirror-from",
        "a/b",
        "--empty",
    ]);
    assert!(result.is_err(), "--mirror-from and --empty must conflict");
}

#[test]
fn test_clap_mirror_from_conflicts_with_init_readme() {
    let result = try_parse(&[
        "--name",
        "n",
        "--owner",
        "o",
        "--mirror-from",
        "a/b",
        "--init-readme",
    ]);
    assert!(
        result.is_err(),
        "--mirror-from and --init-readme must conflict"
    );
}

#[test]
fn test_clap_mirror_from_conflicts_with_init_gitignore() {
    let result = try_parse(&[
        "--name",
        "n",
        "--owner",
        "o",
        "--mirror-from",
        "a/b",
        "--init-gitignore",
    ]);
    assert!(
        result.is_err(),
        "--mirror-from and --init-gitignore must conflict"
    );
}

/// `--mirror-history` without `--mirror-from` must be rejected by clap's
/// `requires` enforcement. A stub that declares `mirror_history` as a plain
/// bool flag with no `requires` attribute would let this through.
#[test]
fn test_clap_mirror_history_without_mirror_from_is_rejected() {
    let result = try_parse(&["--name", "n", "--owner", "o", "--mirror-history"]);
    assert!(
        result.is_err(),
        "--mirror-history must require --mirror-from"
    );
}

/// Existing flag combinations unaffected by this change must keep working
/// (regression guard against a careless `conflicts_with_all` edit elsewhere).
#[test]
fn test_clap_empty_and_init_readme_still_conflict_after_this_change() {
    let result = try_parse(&["--name", "n", "--owner", "o", "--empty", "--init-readme"]);
    assert!(
        result.is_err(),
        "--empty and --init-readme must still conflict"
    );
}

// ============================================================================
// Tier 1: `handle_create_command` business logic — valid fork/mirror flows
// ============================================================================

#[tokio::test]
async fn test_fork_from_builds_fork_creation_mode_with_parsed_source() {
    let ask = make_ask_user_for_value;
    let log = Arc::new(Mutex::new(CallLog::new()));
    let create_repo = make_logged_create_repo_success(log.clone());

    let repo_name = Some("new-repo".to_string());
    let org_name = Some("my-org".to_string());
    let fork_from = Some("upstream-org/upstream-repo".to_string());

    let options = CreateCommandOptions::new(
        &None,
        &repo_name,
        &org_name,
        &None,
        false,
        false,
        false,
        &[],
        &[],
        &fork_from,
        &None,
        false,
    );

    let result = handle_create_command(options, ask, create_repo).await;
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result.err());

    let log = log.lock().unwrap();
    let req = &log.create_repository_args[0];
    assert_eq!(
        req.creation_mode,
        CreationMode::Fork {
            source: expected_source("upstream-org", "upstream-repo")
        }
    );
}

#[tokio::test]
async fn test_mirror_from_without_history_builds_source_only_mirror_mode() {
    let ask = make_ask_user_for_value;
    let log = Arc::new(Mutex::new(CallLog::new()));
    let create_repo = make_logged_create_repo_success(log.clone());

    let repo_name = Some("new-repo".to_string());
    let org_name = Some("my-org".to_string());
    let mirror_from = Some("upstream-org/upstream-repo".to_string());

    let options = CreateCommandOptions::new(
        &None,
        &repo_name,
        &org_name,
        &None,
        false,
        false,
        false,
        &[],
        &[],
        &None,
        &mirror_from,
        false,
    );

    let result = handle_create_command(options, ask, create_repo).await;
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result.err());

    let log = log.lock().unwrap();
    let req = &log.create_repository_args[0];
    assert_eq!(
        req.creation_mode,
        CreationMode::Mirror {
            source: expected_source("upstream-org", "upstream-repo"),
            preserve_history: false,
        }
    );
}

#[tokio::test]
async fn test_mirror_from_with_history_builds_history_mirror_mode() {
    let ask = make_ask_user_for_value;
    let log = Arc::new(Mutex::new(CallLog::new()));
    let create_repo = make_logged_create_repo_success(log.clone());

    let repo_name = Some("new-repo".to_string());
    let org_name = Some("my-org".to_string());
    let mirror_from = Some("upstream-org/upstream-repo".to_string());

    let options = CreateCommandOptions::new(
        &None,
        &repo_name,
        &org_name,
        &None,
        false,
        false,
        false,
        &[],
        &[],
        &None,
        &mirror_from,
        true,
    );

    let result = handle_create_command(options, ask, create_repo).await;
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result.err());

    let log = log.lock().unwrap();
    let req = &log.create_repository_args[0];
    assert_eq!(
        req.creation_mode,
        CreationMode::Mirror {
            source: expected_source("upstream-org", "upstream-repo"),
            preserve_history: true,
        }
    );
}

/// Stub-killing: a buggy implementation that ignores `mirror_history` (e.g.
/// always constructs `preserve_history: false`) would pass the two tests
/// above independently if they only checked `matches!`, but comparing full
/// equality against the opposite mode here forces the distinction.
#[tokio::test]
async fn test_mirror_history_true_and_false_produce_distinct_creation_modes() {
    let mirror_from = Some("upstream-org/upstream-repo".to_string());

    for preserve_history in [true, false] {
        let ask = make_ask_user_for_value;
        let log = Arc::new(Mutex::new(CallLog::new()));
        let create_repo = make_logged_create_repo_success(log.clone());
        let repo_name = Some("new-repo".to_string());
        let org_name = Some("my-org".to_string());

        let options = CreateCommandOptions::new(
            &None,
            &repo_name,
            &org_name,
            &None,
            false,
            false,
            false,
            &[],
            &[],
            &None,
            &mirror_from,
            preserve_history,
        );
        let result = handle_create_command(options, ask, create_repo).await;
        assert!(result.is_ok());
        let log = log.lock().unwrap();
        let req = &log.create_repository_args[0];
        match &req.creation_mode {
            CreationMode::Mirror {
                preserve_history: actual,
                ..
            } => assert_eq!(*actual, preserve_history),
            other => panic!("expected Mirror, got {:?}", other),
        }
    }
}

#[tokio::test]
async fn test_fork_from_with_explicit_template_passes_template_through() {
    let ask = make_ask_user_for_value;
    let log = Arc::new(Mutex::new(CallLog::new()));
    let create_repo = make_logged_create_repo_success(log.clone());

    let repo_name = Some("new-repo".to_string());
    let org_name = Some("my-org".to_string());
    let template = Some("fork-settings".to_string());
    let fork_from = Some("upstream-org/upstream-repo".to_string());

    let options = CreateCommandOptions::new(
        &None,
        &repo_name,
        &org_name,
        &template,
        false,
        false,
        false,
        &[],
        &[],
        &fork_from,
        &None,
        false,
    );

    let result = handle_create_command(options, ask, create_repo).await;
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result.err());

    let log = log.lock().unwrap();
    let req = &log.create_repository_args[0];
    assert_eq!(req.template.as_ref().unwrap().as_str(), "fork-settings");
}

/// Template is optional for Fork/Mirror (spec §4: "template == None is valid
/// for all three non-Standard modes") — the CLI must not prompt for it.
#[tokio::test]
async fn test_fork_from_without_template_does_not_prompt_for_template() {
    let ask_count = Arc::new(Mutex::new(0));
    let ask_count_clone = ask_count.clone();
    let ask = move |prompt: &str| {
        *ask_count_clone.lock().unwrap() += 1;
        Ok(format!("unexpected-prompt: {}", prompt))
    };

    let log = Arc::new(Mutex::new(CallLog::new()));
    let create_repo = make_logged_create_repo_success(log.clone());

    let repo_name = Some("new-repo".to_string());
    let org_name = Some("my-org".to_string());
    let fork_from = Some("upstream-org/upstream-repo".to_string());

    let options = CreateCommandOptions::new(
        &None,
        &repo_name,
        &org_name,
        &None,
        false,
        false,
        false,
        &[],
        &[],
        &fork_from,
        &None,
        false,
    );

    let result = handle_create_command(options, ask, create_repo).await;
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result.err());
    assert_eq!(
        *ask_count.lock().unwrap(),
        0,
        "Should not prompt for template when --fork-from is used without --template"
    );
}

#[tokio::test]
async fn test_mirror_from_source_only_without_template_does_not_prompt_for_template() {
    let ask_count = Arc::new(Mutex::new(0));
    let ask_count_clone = ask_count.clone();
    let ask = move |prompt: &str| {
        *ask_count_clone.lock().unwrap() += 1;
        Ok(format!("unexpected-prompt: {}", prompt))
    };

    let log = Arc::new(Mutex::new(CallLog::new()));
    let create_repo = make_logged_create_repo_success(log.clone());

    let repo_name = Some("new-repo".to_string());
    let org_name = Some("my-org".to_string());
    let mirror_from = Some("upstream-org/upstream-repo".to_string());

    let options = CreateCommandOptions::new(
        &None,
        &repo_name,
        &org_name,
        &None,
        false,
        false,
        false,
        &[],
        &[],
        &None,
        &mirror_from,
        false,
    );

    let result = handle_create_command(options, ask, create_repo).await;
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result.err());
    assert_eq!(
        *ask_count.lock().unwrap(),
        0,
        "Should not prompt for template when --mirror-from is used without --template"
    );
}

#[tokio::test]
async fn test_fork_from_with_teams_and_collaborators_still_wires_them_through() {
    let ask = make_ask_user_for_value;
    let log = Arc::new(Mutex::new(CallLog::new()));
    let create_repo = make_logged_create_repo_success(log.clone());

    let repo_name = Some("new-repo".to_string());
    let org_name = Some("my-org".to_string());
    let fork_from = Some("upstream-org/upstream-repo".to_string());
    let teams = vec!["platform:write".to_string()];
    let collaborators = vec!["alice:read".to_string()];

    let options = CreateCommandOptions::new(
        &None,
        &repo_name,
        &org_name,
        &None,
        false,
        false,
        false,
        &teams,
        &collaborators,
        &fork_from,
        &None,
        false,
    );

    let result = handle_create_command(options, ask, create_repo).await;
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result.err());

    let log = log.lock().unwrap();
    let req = &log.create_repository_args[0];
    assert_eq!(req.teams.get("platform"), Some(&AccessLevel::Write));
    assert_eq!(req.collaborators.get("alice"), Some(&AccessLevel::Read));
}

/// Regression/contract guard: the CLI must never call `.content_strategy(...)`
/// with an explicit non-default value when a fork/mirror flag is set — doing
/// so would panic inside `RepositoryCreationRequestBuilder::build()` (see
/// `request.rs`). If that guard is violated, this test aborts via panic
/// rather than failing gracefully, which is itself the detection mechanism.
#[tokio::test]
async fn test_fork_from_never_triggers_the_builder_content_strategy_panic_guard() {
    let ask = make_ask_user_for_value;
    let log = Arc::new(Mutex::new(CallLog::new()));
    let create_repo = make_logged_create_repo_success(log.clone());

    let repo_name = Some("new-repo".to_string());
    let org_name = Some("my-org".to_string());
    let fork_from = Some("upstream-org/upstream-repo".to_string());

    let options = CreateCommandOptions::new(
        &None,
        &repo_name,
        &org_name,
        &None,
        false,
        false,
        false,
        &[],
        &[],
        &fork_from,
        &None,
        false,
    );

    // No panic must occur; if the implementation calls
    // `.content_strategy(ContentStrategy::Empty)` (or similar) alongside
    // `.creation_mode(CreationMode::Fork { .. })`, the builder panics and
    // this test aborts rather than returning a normal pass/fail.
    let result = handle_create_command(options, ask, create_repo).await;
    assert!(result.is_ok());
}

// ============================================================================
// Tier 2: Adversarial — malformed "owner/repo" parsing
// ============================================================================

async fn assert_fork_from_rejected(fork_from_value: &str) {
    let ask = make_ask_user_for_value;
    let create_repo = make_panicking_create_repo();

    let repo_name = Some("new-repo".to_string());
    let org_name = Some("my-org".to_string());
    let fork_from = Some(fork_from_value.to_string());

    let options = CreateCommandOptions::new(
        &None,
        &repo_name,
        &org_name,
        &None,
        false,
        false,
        false,
        &[],
        &[],
        &fork_from,
        &None,
        false,
    );

    let result = handle_create_command(options, ask, create_repo).await;
    assert!(
        matches!(result, Err(Error::InvalidArguments(_))),
        "expected Error::InvalidArguments for --fork-from '{}', got: {:?}",
        fork_from_value,
        result
    );
}

async fn assert_mirror_from_rejected(mirror_from_value: &str) {
    let ask = make_ask_user_for_value;
    let create_repo = make_panicking_create_repo();

    let repo_name = Some("new-repo".to_string());
    let org_name = Some("my-org".to_string());
    let mirror_from = Some(mirror_from_value.to_string());

    let options = CreateCommandOptions::new(
        &None,
        &repo_name,
        &org_name,
        &None,
        false,
        false,
        false,
        &[],
        &[],
        &None,
        &mirror_from,
        false,
    );

    let result = handle_create_command(options, ask, create_repo).await;
    assert!(
        matches!(result, Err(Error::InvalidArguments(_))),
        "expected Error::InvalidArguments for --mirror-from '{}', got: {:?}",
        mirror_from_value,
        result
    );
}

#[tokio::test]
async fn test_fork_from_without_slash_is_rejected() {
    assert_fork_from_rejected("no-slash-here").await;
}

#[tokio::test]
async fn test_fork_from_with_two_slashes_is_rejected() {
    assert_fork_from_rejected("owner/repo/extra").await;
}

#[tokio::test]
async fn test_fork_from_with_empty_owner_is_rejected() {
    assert_fork_from_rejected("/repo").await;
}

#[tokio::test]
async fn test_fork_from_with_empty_repo_name_is_rejected() {
    assert_fork_from_rejected("owner/").await;
}

#[tokio::test]
async fn test_fork_from_with_space_in_owner_is_rejected() {
    assert_fork_from_rejected("up stream/repo").await;
}

#[tokio::test]
async fn test_fork_from_with_space_in_repo_name_is_rejected() {
    assert_fork_from_rejected("upstream/re po").await;
}

#[tokio::test]
async fn test_fork_from_with_leading_hyphen_owner_is_rejected() {
    assert_fork_from_rejected("-upstream/repo").await;
}

#[tokio::test]
async fn test_fork_from_with_repo_name_starting_with_dot_is_rejected() {
    assert_fork_from_rejected("upstream/.repo").await;
}

#[tokio::test]
async fn test_fork_from_with_double_hyphen_owner_is_rejected() {
    assert_fork_from_rejected("up--stream/repo").await;
}

#[tokio::test]
async fn test_mirror_from_without_slash_is_rejected() {
    assert_mirror_from_rejected("no-slash-here").await;
}

#[tokio::test]
async fn test_mirror_from_with_two_slashes_is_rejected() {
    assert_mirror_from_rejected("owner/repo/extra").await;
}

#[tokio::test]
async fn test_mirror_from_with_empty_owner_is_rejected() {
    assert_mirror_from_rejected("/repo").await;
}

#[tokio::test]
async fn test_mirror_from_with_empty_repo_name_is_rejected() {
    assert_mirror_from_rejected("owner/").await;
}

#[tokio::test]
async fn test_mirror_from_with_space_in_owner_is_rejected() {
    assert_mirror_from_rejected("up stream/repo").await;
}

/// Side-effect verification: malformed input must be rejected *before* any
/// attempt to call the repository-creation function. `make_panicking_create_repo`
/// turns an accidental call into an immediate test failure (panic), which is
/// stronger than a call-count assertion — it cannot be satisfied by a stub
/// that calls through and happens to also return an unrelated error.
#[tokio::test]
async fn test_fork_from_malformed_value_never_invokes_create_repository_fn() {
    assert_fork_from_rejected("totally-malformed-no-slash").await;
}

#[tokio::test]
async fn test_mirror_from_malformed_value_never_invokes_create_repository_fn() {
    assert_mirror_from_rejected("totally-malformed-no-slash").await;
}

#[tokio::test]
async fn test_fork_from_error_message_is_descriptive() {
    let ask = make_ask_user_for_value;
    let create_repo = make_panicking_create_repo();

    let repo_name = Some("new-repo".to_string());
    let org_name = Some("my-org".to_string());
    let fork_from = Some("malformed-no-slash".to_string());

    let options = CreateCommandOptions::new(
        &None,
        &repo_name,
        &org_name,
        &None,
        false,
        false,
        false,
        &[],
        &[],
        &fork_from,
        &None,
        false,
    );

    let result = handle_create_command(options, ask, create_repo).await;
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("fork") || msg.contains("owner/repo") || msg.contains("malformed-no-slash"),
        "error message should mention the offending value or expected format, got: {msg}"
    );
}

/// Explicit non-RepoRollerError wiring check: a success-returning
/// `create_repository_fn` must simply never be reached, distinguishing a
/// correct "reject before dispatch" implementation from one that
/// accidentally dispatches and gets lucky with a success response.
#[tokio::test]
async fn test_fork_from_malformed_value_rejected_even_when_create_repo_fn_would_succeed() {
    let ask = make_ask_user_for_value;
    let log = Arc::new(Mutex::new(CallLog::new()));
    let create_repo = make_logged_create_repo_success(log.clone());

    let repo_name = Some("new-repo".to_string());
    let org_name = Some("my-org".to_string());
    let fork_from = Some("malformed-no-slash".to_string());

    let options = CreateCommandOptions::new(
        &None,
        &repo_name,
        &org_name,
        &None,
        false,
        false,
        false,
        &[],
        &[],
        &fork_from,
        &None,
        false,
    );

    let result = handle_create_command(options, ask, create_repo).await;
    assert!(matches!(result, Err(Error::InvalidArguments(_))));
    assert!(
        log.lock().unwrap().create_repository_args.is_empty(),
        "create_repository_fn must not have been called"
    );
}
