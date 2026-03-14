//! Tests for the `make-template` command.

use super::*;
use std::fs;
use tempfile::TempDir;

// ============================================================================
// Test helpers
// ============================================================================

/// Creates a minimal `.git` directory so the path is treated as a normal repo.
fn init_git_dir(dir: &std::path::Path) {
    fs::create_dir_all(dir.join(".git")).unwrap();
    fs::write(dir.join(".git").join("HEAD"), "ref: refs/heads/main\n").unwrap();
}

/// Returns a minimal `MakeTemplateArgs` with `--yes` and the given path.
fn make_args(path: &str) -> MakeTemplateArgs {
    MakeTemplateArgs {
        path: path.to_string(),
        name: None,
        description: None,
        author: None,
        force: false,
        renovate: false,
        yes: true,
    }
}

fn ask_yes(_: &str) -> Result<String, Error> {
    Ok("y".to_string())
}

fn ask_no(_: &str) -> Result<String, Error> {
    Ok("n".to_string())
}

// ============================================================================
// Path-validation tests
// ============================================================================

#[tokio::test]
async fn test_make_template_nonexistent_path_returns_invalid_arguments() {
    let args = make_args("/this/path/does/not/exist/9f3a2b1c");
    let result = execute(&args, ask_yes).await;
    assert!(
        matches!(result, Err(Error::InvalidArguments(_))),
        "expected InvalidArguments, got: {result:?}"
    );
}

#[tokio::test]
async fn test_make_template_path_is_a_file_returns_invalid_arguments() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("not_a_dir.txt");
    fs::write(&file, "hello").unwrap();

    let args = make_args(file.to_str().unwrap());
    let result = execute(&args, ask_yes).await;
    assert!(
        matches!(result, Err(Error::InvalidArguments(_))),
        "a file path should return InvalidArguments"
    );
}

#[tokio::test]
async fn test_make_template_directory_without_git_returns_invalid_arguments() {
    let dir = TempDir::new().unwrap();
    // No .git entry created.
    let args = make_args(dir.path().to_str().unwrap());
    let result = execute(&args, ask_yes).await;
    assert!(
        matches!(result, Err(Error::InvalidArguments(_))),
        "directory without .git should return InvalidArguments"
    );
}

#[tokio::test]
async fn test_make_template_git_file_worktree_is_accepted() {
    // Git worktrees have .git as a *file*, not a directory.
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join(".git"),
        "gitdir: ../.git/worktrees/branch\n",
    )
    .unwrap();

    let mut args = make_args(dir.path().to_str().unwrap());
    args.name = Some("worktree-template".to_string());

    let result = execute(&args, ask_yes).await;
    // Should NOT fail with "not a Git repository".
    assert!(
        !matches!(&result, Err(Error::InvalidArguments(msg)) if msg.contains("not a Git")),
        "a .git worktree file should be treated as a valid git repo; got: {result:?}"
    );
}

// ============================================================================
// User-cancellation test
// ============================================================================

#[tokio::test]
async fn test_make_template_user_cancels_returns_cancelled() {
    let dir = TempDir::new().unwrap();
    init_git_dir(dir.path());

    let mut args = make_args(dir.path().to_str().unwrap());
    args.yes = false; // will prompt

    let result = execute(&args, ask_no).await;
    assert!(
        matches!(result, Err(Error::Cancelled(_))),
        "declining confirmation should return Cancelled, not InvalidArguments"
    );
    // No files should have been written.
    assert!(!dir.path().join(".reporoller/template.toml").exists());
}

// ============================================================================
// Template-name resolution (unit tests of the pure helper)
// ============================================================================

#[test]
fn test_resolve_template_name_uses_explicit_name() {
    let dir = std::path::Path::new("/some/path/my-repo");
    let args = MakeTemplateArgs {
        path: "/some/path/my-repo".to_string(),
        name: Some("custom-name".to_string()),
        description: None,
        author: None,
        force: false,
        renovate: false,
        yes: true,
    };
    assert_eq!(resolve_template_name(&args, dir), "custom-name");
}

#[test]
fn test_resolve_template_name_falls_back_to_directory_name() {
    let dir = std::path::Path::new("/some/path/my-repo");
    let args = MakeTemplateArgs {
        path: "/some/path/my-repo".to_string(),
        name: None,
        description: None,
        author: None,
        force: false,
        renovate: false,
        yes: true,
    };
    assert_eq!(resolve_template_name(&args, dir), "my-repo");
}

// ============================================================================
// Happy-path: file creation
// ============================================================================

#[tokio::test]
async fn test_make_template_creates_all_required_files() {
    let dir = TempDir::new().unwrap();
    init_git_dir(dir.path());

    let mut args = make_args(dir.path().to_str().unwrap());
    args.name = Some("test-template".to_string());

    let result = execute(&args, ask_yes).await;
    assert!(result.is_ok(), "should succeed; got: {result:?}");

    assert!(
        dir.path().join(".reporoller/template.toml").exists(),
        ".reporoller/template.toml must be created"
    );
    assert!(
        dir.path().join("README.md").exists(),
        "README.md (template-developer docs) must be created"
    );
    assert!(
        dir.path().join("README.md.template").exists(),
        "README.md.template (new-repo scaffold) must be created"
    );
    assert!(
        dir.path().join(".gitignore").exists(),
        ".gitignore (template-developer gitignore) must be created"
    );
    assert!(
        dir.path().join(".gitignore.template").exists(),
        ".gitignore.template (new-repo starter gitignore) must be created"
    );
    assert!(
        dir.path()
            .join(".github/workflows/test-template.yml")
            .exists(),
        ".github/workflows/test-template.yml must be created"
    );
    assert!(
        dir.path()
            .join(".github/workflows/ci.yml.template")
            .exists(),
        ".github/workflows/ci.yml.template must be created"
    );
}

#[tokio::test]
async fn test_make_template_no_renovate_json_by_default() {
    let dir = TempDir::new().unwrap();
    init_git_dir(dir.path());

    let mut args = make_args(dir.path().to_str().unwrap());
    args.name = Some("no-renovate".to_string());
    args.renovate = false;

    execute(&args, ask_yes).await.unwrap();
    assert!(
        !dir.path().join("renovate.json").exists(),
        "renovate.json must NOT be created without --renovate"
    );
}

#[tokio::test]
async fn test_make_template_creates_renovate_json_with_flag() {
    let dir = TempDir::new().unwrap();
    init_git_dir(dir.path());

    let mut args = make_args(dir.path().to_str().unwrap());
    args.name = Some("with-renovate".to_string());
    args.renovate = true;

    execute(&args, ask_yes).await.unwrap();
    assert!(
        dir.path().join("renovate.json").exists(),
        "renovate.json must be created with --renovate"
    );
}

// ============================================================================
// Force / skip behaviour
// ============================================================================

#[tokio::test]
async fn test_make_template_skips_existing_file_without_force() {
    let dir = TempDir::new().unwrap();
    init_git_dir(dir.path());
    fs::create_dir_all(dir.path().join(".reporoller")).unwrap();
    fs::write(dir.path().join(".reporoller/template.toml"), "# existing").unwrap();

    let mut args = make_args(dir.path().to_str().unwrap());
    args.force = false;

    let result = execute(&args, ask_yes).await;
    assert!(
        result.is_ok(),
        "should succeed even with existing file: {result:?}"
    );

    let res = result.unwrap();
    assert!(
        res.skipped_files
            .iter()
            .any(|f| f.contains("template.toml")),
        "template.toml should appear in skipped_files; got: {:?}",
        res.skipped_files
    );
    // Original content must be preserved.
    let content = fs::read_to_string(dir.path().join(".reporoller/template.toml")).unwrap();
    assert_eq!(content, "# existing");
}

#[tokio::test]
async fn test_make_template_overwrites_existing_file_with_force() {
    let dir = TempDir::new().unwrap();
    init_git_dir(dir.path());
    fs::create_dir_all(dir.path().join(".reporoller")).unwrap();
    fs::write(dir.path().join(".reporoller/template.toml"), "# existing").unwrap();

    let mut args = make_args(dir.path().to_str().unwrap());
    args.force = true;
    args.name = Some("forced-template".to_string());

    execute(&args, ask_yes).await.unwrap();

    let content = fs::read_to_string(dir.path().join(".reporoller/template.toml")).unwrap();
    assert_ne!(content, "# existing", "file should have been overwritten");
    assert!(
        content.contains("forced-template"),
        "overwritten file should contain new template name"
    );
}

// ============================================================================
// Result fields
// ============================================================================

#[tokio::test]
async fn test_make_template_result_contains_template_name() {
    let dir = TempDir::new().unwrap();
    init_git_dir(dir.path());

    let mut args = make_args(dir.path().to_str().unwrap());
    args.name = Some("my-lib-template".to_string());

    let result = execute(&args, ask_yes).await.unwrap();
    assert_eq!(result.template_name, "my-lib-template");
}

#[tokio::test]
async fn test_make_template_result_uses_directory_name_when_no_name_given() {
    let dir = TempDir::new().unwrap();
    init_git_dir(dir.path());

    let args = make_args(dir.path().to_str().unwrap());
    let result = execute(&args, ask_yes).await.unwrap();

    let expected_name = dir
        .path()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert_eq!(result.template_name, expected_name);
}

#[tokio::test]
async fn test_make_template_result_written_files_not_empty() {
    let dir = TempDir::new().unwrap();
    init_git_dir(dir.path());

    let mut args = make_args(dir.path().to_str().unwrap());
    args.name = Some("written-test".to_string());

    let result = execute(&args, ask_yes).await.unwrap();
    assert!(
        !result.written_files.is_empty(),
        "at least one file should be written"
    );
}

// ============================================================================
// Content-validation tests
// ============================================================================

#[tokio::test]
async fn test_make_template_toml_is_valid_toml() {
    let dir = TempDir::new().unwrap();
    init_git_dir(dir.path());

    let mut args = make_args(dir.path().to_str().unwrap());
    args.name = Some("valid-toml-test".to_string());

    execute(&args, ask_yes).await.unwrap();

    let content = fs::read_to_string(dir.path().join(".reporoller/template.toml")).unwrap();
    let parsed: Result<toml::Table, _> = toml::from_str(&content);
    assert!(
        parsed.is_ok(),
        "template.toml should be valid TOML; parse error: {:?}",
        parsed.err()
    );
}

#[tokio::test]
async fn test_make_template_toml_contains_required_template_section() {
    let dir = TempDir::new().unwrap();
    init_git_dir(dir.path());

    let mut args = make_args(dir.path().to_str().unwrap());
    args.name = Some("section-test".to_string());
    args.description = Some("Test description".to_string());
    args.author = Some("Test Author".to_string());

    execute(&args, ask_yes).await.unwrap();

    let content = fs::read_to_string(dir.path().join(".reporoller/template.toml")).unwrap();
    let table: toml::Table = toml::from_str(&content).unwrap();

    let tmpl = table
        .get("template")
        .expect("missing [template] section in template.toml");
    assert_eq!(
        tmpl.get("name").and_then(|v| v.as_str()),
        Some("section-test"),
        "template.name should match --name"
    );
    assert_eq!(
        tmpl.get("description").and_then(|v| v.as_str()),
        Some("Test description"),
        "template.description should match --description"
    );
    assert_eq!(
        tmpl.get("author").and_then(|v| v.as_str()),
        Some("Test Author"),
        "template.author should match --author"
    );
}

#[tokio::test]
async fn test_make_template_toml_has_exclude_patterns_for_template_only_files() {
    let dir = TempDir::new().unwrap();
    init_git_dir(dir.path());

    let mut args = make_args(dir.path().to_str().unwrap());
    args.name = Some("exclude-test".to_string());

    execute(&args, ask_yes).await.unwrap();

    let content = fs::read_to_string(dir.path().join(".reporoller/template.toml")).unwrap();
    let table: toml::Table = toml::from_str(&content).unwrap();

    let templating = table
        .get("templating")
        .expect("missing [templating] section in template.toml");
    let excludes = templating
        .get("exclude_patterns")
        .and_then(|v| v.as_array())
        .expect("exclude_patterns should be an array");

    let strs: Vec<&str> = excludes.iter().filter_map(|v| v.as_str()).collect();
    assert!(
        strs.contains(&"README.md"),
        "exclude_patterns should contain README.md"
    );
    assert!(
        strs.contains(&".gitignore"),
        "exclude_patterns should contain .gitignore"
    );
    assert!(
        strs.contains(&".github/workflows/test-template.yml"),
        "exclude_patterns should contain .github/workflows/test-template.yml"
    );
}

#[tokio::test]
async fn test_make_template_readme_template_has_repo_name_placeholder() {
    let dir = TempDir::new().unwrap();
    init_git_dir(dir.path());

    let mut args = make_args(dir.path().to_str().unwrap());
    args.name = Some("placeholder-test".to_string());

    execute(&args, ask_yes).await.unwrap();

    let content = fs::read_to_string(dir.path().join("README.md.template")).unwrap();
    assert!(
        content.contains("{{repo_name}}"),
        "README.md.template should contain {{{{repo_name}}}} placeholder; got:\n{content}"
    );
}

#[tokio::test]
async fn test_make_template_readme_developer_docs_contains_template_name() {
    let dir = TempDir::new().unwrap();
    init_git_dir(dir.path());

    let mut args = make_args(dir.path().to_str().unwrap());
    args.name = Some("dev-docs-test".to_string());

    execute(&args, ask_yes).await.unwrap();

    let content = fs::read_to_string(dir.path().join("README.md")).unwrap();
    assert!(
        content.contains("dev-docs-test"),
        "README.md should contain the template name"
    );
}

#[tokio::test]
async fn test_make_template_ci_template_has_repo_name_placeholder() {
    let dir = TempDir::new().unwrap();
    init_git_dir(dir.path());

    let mut args = make_args(dir.path().to_str().unwrap());
    args.name = Some("ci-placeholder-test".to_string());

    execute(&args, ask_yes).await.unwrap();

    let content = fs::read_to_string(dir.path().join(".github/workflows/ci.yml.template")).unwrap();
    assert!(
        content.contains("{{repo_name}}"),
        ".github/workflows/ci.yml.template should contain {{{{repo_name}}}} placeholder"
    );
}

#[tokio::test]
async fn test_make_template_renovate_json_is_valid_json() {
    let dir = TempDir::new().unwrap();
    init_git_dir(dir.path());

    let mut args = make_args(dir.path().to_str().unwrap());
    args.name = Some("renovate-json-test".to_string());
    args.renovate = true;

    execute(&args, ask_yes).await.unwrap();

    let content = fs::read_to_string(dir.path().join("renovate.json")).unwrap();
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&content);
    assert!(
        parsed.is_ok(),
        "renovate.json should be valid JSON; error: {:?}",
        parsed.err()
    );
}

// ============================================================================
// generate_* unit tests (pure functions, no filesystem)
// ============================================================================

#[test]
fn test_generate_template_toml_is_valid_toml() {
    let content = generate_template_toml("my-lib", "A library", "Platform Team");
    let parsed: Result<toml::Table, _> = toml::from_str(&content);
    assert!(
        parsed.is_ok(),
        "generate_template_toml should produce valid TOML; error: {:?}",
        parsed.err()
    );
}

#[test]
fn test_generate_template_toml_embeds_name_and_description() {
    let content = generate_template_toml("embedded-name", "Embedded desc", "Author");
    assert!(content.contains("embedded-name"));
    assert!(content.contains("Embedded desc"));
    assert!(content.contains("Author"));
}

#[test]
fn test_generate_readme_template_has_replacement_variables() {
    let content = generate_readme_template("any-template", "any desc");
    assert!(
        content.contains("{{repo_name}}"),
        "README.md.template should use {{{{repo_name}}}}"
    );
}

#[test]
fn test_generate_gitignore_template_has_replacement_variable() {
    let content = generate_gitignore_template();
    assert!(
        content.contains("{{repo_name}}") || content.contains("{{template_name}}"),
        ".gitignore.template should reference at least one template variable"
    );
}

#[test]
fn test_generate_ci_template_workflow_has_replacement_variable() {
    let content = generate_ci_template_workflow();
    assert!(
        content.contains("{{repo_name}}"),
        "ci.yml.template should reference {{{{repo_name}}}}"
    );
}

#[test]
fn test_generate_renovate_config_is_valid_json() {
    let content = generate_renovate_config();
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&content);
    assert!(
        parsed.is_ok(),
        "generate_renovate_config should produce valid JSON; error: {:?}",
        parsed.err()
    );
}

// ============================================================================
// TOML injection guard
// ============================================================================

#[test]
fn test_generate_template_toml_special_characters_produce_valid_toml() {
    // Values containing `"` and `\` would corrupt the TOML string without
    // proper escaping via toml_string_value.
    let content = generate_template_toml(
        r#"name"with"quotes"#,
        r#"desc with \ backslash"#,
        r#"author "quoted""#,
    );
    let parsed: Result<toml::Table, _> = toml::from_str(&content);
    assert!(
        parsed.is_ok(),
        "special characters must be escaped to produce valid TOML; error: {:?}",
        parsed.err()
    );
    let table = parsed.unwrap();
    let tmpl = table.get("template").unwrap();
    assert_eq!(
        tmpl.get("name").and_then(|v| v.as_str()),
        Some(r#"name"with"quotes"#),
        "name must round-trip correctly through TOML escaping"
    );
}

// ============================================================================
// Plan / content-map sync
// ============================================================================

#[tokio::test]
async fn test_make_template_all_planned_files_are_written_or_skipped() {
    let dir = TempDir::new().unwrap();
    init_git_dir(dir.path());

    let mut args = make_args(dir.path().to_str().unwrap());
    args.name = Some("sync-check".to_string());
    // No --renovate, no pre-existing files: all 7 files should be written.
    let result = execute(&args, ask_yes).await.unwrap();
    assert_eq!(
        result.written_files.len() + result.skipped_files.len(),
        7,
        "all 7 planned files must be either written or skipped; \
         if this fails, plan_files and build_content_map are out of sync"
    );
}
