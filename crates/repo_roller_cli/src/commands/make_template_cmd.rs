//! `make-template` command: converts a local Git repository into a RepoRoller template.
//!
//! This module implements the `repo-roller make-template <path>` command which
//! scaffolds all necessary template files into an existing Git repository:
//!
//! | File | Purpose | In output repos? |
//! |---|---|---|
//! | `.reporoller/template.toml` | Template configuration (all options documented) | Never |
//! | `README.md` | Template developer docs shown on GitHub | No (excluded via `exclude_patterns`) |
//! | `README.md.template` | README scaffold for repos created from this template | Yes → renamed to `README.md` |
//! | `.gitignore` | Template developer gitignore | No (excluded via `exclude_patterns`) |
//! | `.gitignore.template` | Starter gitignore for created repos | Yes → renamed to `.gitignore` |
//! | `.github/workflows/test-template.yml` | CI that validates the template structure | No (excluded) |
//! | `.github/workflows/ci.yml.template` | CI scaffold for repos created from this template | Yes → renamed to `ci.yml` |
//! | `renovate.json` | Dependency-update config (opt-in via `--renovate`) | Yes |
//!
//! The `.template` suffix is stripped automatically by the RepoRoller template engine
//! when a repository is created from this template.
//!
//! # Usage
//!
//! ```bash
//! repo-roller make-template ./my-repo --name my-lib --description "Library template" --author "Platform Team"
//! repo-roller make-template ./my-repo --renovate --yes
//! ```

use clap::Args;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::debug;

use crate::errors::Error;

#[cfg(test)]
#[path = "make_template_cmd_tests.rs"]
mod tests;

// ============================================================================
// Public types
// ============================================================================

/// Planned action for a single file during template initialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileAction {
    /// File will be created (does not exist yet).
    Create,
    /// File already exists and will be overwritten because `--force` was passed.
    Overwrite,
    /// File already exists and will be left unchanged (no `--force`).
    Skip,
}

/// A planned file operation shown to the user in the preview before any writes occur.
#[derive(Debug, Clone)]
pub struct PlannedFile {
    /// Path relative to the repository root (forward-slash separators).
    pub relative_path: String,
    /// What will happen to this file.
    pub action: FileAction,
    /// Optional note shown in the preview (e.g. "excluded from output repos").
    pub note: Option<String>,
}

/// Result returned by a successful [`execute`] call.
// Fields are used in tests and by callers that display the summary.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MakeTemplateResult {
    /// The template name written into `template.toml`.
    pub template_name: String,
    /// Paths of files that were created or overwritten.
    pub written_files: Vec<String>,
    /// Paths of files that were skipped because they already existed.
    pub skipped_files: Vec<String>,
}

// ============================================================================
// Command-line arguments
// ============================================================================

/// Arguments for `repo-roller make-template`.
#[derive(Args, Debug, Clone)]
pub struct MakeTemplateArgs {
    /// Path to the Git repository to convert into a RepoRoller template.
    pub path: String,

    /// Template name written into `template.toml` (defaults to the directory name).
    #[arg(long)]
    pub name: Option<String>,

    /// Human-readable description of this template.
    #[arg(long)]
    pub description: Option<String>,

    /// Author name or team owning this template.
    #[arg(long)]
    pub author: Option<String>,

    /// Overwrite existing files rather than skipping them.
    #[arg(long)]
    pub force: bool,

    /// Include a `renovate.json` dependency-update configuration.
    #[arg(long)]
    pub renovate: bool,

    /// Skip the confirmation prompt (suitable for CI / non-interactive use).
    #[arg(long, short = 'y')]
    pub yes: bool,
}

// ============================================================================
// Public entry point
// ============================================================================

/// Executes the `make-template` command.
///
/// Shows a preview of all files that will be created, asks for confirmation
/// (unless `--yes` is passed), then writes the scaffold files.
///
/// # Arguments
///
/// * `args` – Parsed CLI arguments.
/// * `ask`  – Callback for prompting the user; receives the prompt string and
///   returns the user's response. Injected for testability.
///
/// # Errors
///
/// * `Error::InvalidArguments` – the path does not exist, is not a directory,
///   is not a Git repository, or the user declined the confirmation prompt.
/// * `Error::LoadFile` – a file could not be written to disk.
pub async fn execute(
    args: &MakeTemplateArgs,
    ask: impl Fn(&str) -> Result<String, Error>,
) -> Result<MakeTemplateResult, Error> {
    let dir = Path::new(&args.path);

    // ── Phase 1: Validate ────────────────────────────────────────────────────
    if !dir.exists() {
        return Err(Error::InvalidArguments(format!(
            "Path does not exist: {}",
            args.path
        )));
    }
    if !dir.is_dir() {
        return Err(Error::InvalidArguments(format!(
            "Path is not a directory: {}",
            args.path
        )));
    }
    // Accept both a .git directory (normal repo) and a .git file (worktree / submodule).
    if !dir.join(".git").exists() {
        return Err(Error::InvalidArguments(format!(
            "Path is not a Git repository (no .git entry found): {}",
            args.path
        )));
    }

    // ── Phase 2: Plan ────────────────────────────────────────────────────────
    let template_name = resolve_template_name(args, dir);
    let description = args
        .description
        .as_deref()
        .unwrap_or("A new repository template");
    let author = args.author.as_deref().unwrap_or("Your Name / Team");

    let planned = plan_files(dir, args);

    // ── Phase 3: Preview ─────────────────────────────────────────────────────
    print_preview(dir, &planned);

    // ── Phase 4: Confirm ─────────────────────────────────────────────────────
    if !args.yes {
        let answer = ask("Proceed? [y/N]: ")?;
        if !answer.trim().eq_ignore_ascii_case("y") && !answer.trim().eq_ignore_ascii_case("yes") {
            return Err(Error::InvalidArguments(
                "Template initialization cancelled by user.".to_string(),
            ));
        }
    }

    // ── Phase 5: Build content ───────────────────────────────────────────────
    let content_map = build_content_map(&template_name, description, author, args.renovate);

    // ── Phase 6: Write files ─────────────────────────────────────────────────
    let mut written = Vec::new();
    let mut skipped = Vec::new();

    for planned_file in &planned {
        match planned_file.action {
            FileAction::Skip => {
                debug!("Skipping existing file: {}", planned_file.relative_path);
                skipped.push(planned_file.relative_path.clone());
            }
            FileAction::Create | FileAction::Overwrite => {
                if let Some(content) = content_map.get(&planned_file.relative_path) {
                    let full_path = dir.join(&planned_file.relative_path);
                    write_file(&full_path, content)?;
                    debug!("Wrote: {}", planned_file.relative_path);
                    written.push(planned_file.relative_path.clone());
                }
            }
        }
    }

    // ── Phase 7: Summary ─────────────────────────────────────────────────────
    println!("\nTemplate initialization complete!");
    println!("  Written : {} file(s)", written.len());
    if !skipped.is_empty() {
        println!(
            "  Skipped : {} file(s) (already exist; use --force to overwrite)",
            skipped.len()
        );
    }
    println!("\nNext steps:");
    println!("  1. Edit .reporoller/template.toml to configure your template");
    println!(
        "  2. Customize README.md.template and .gitignore.template for repos created from this template"
    );
    println!("  3. Update .github/workflows/ci.yml.template for your project's CI needs");
    println!("  4. Push this repository to GitHub and register it with RepoRoller");

    Ok(MakeTemplateResult {
        template_name,
        written_files: written,
        skipped_files: skipped,
    })
}

// ============================================================================
// Internal helpers
// ============================================================================

/// Derives the template name from `--name` or from the repository directory name.
pub(crate) fn resolve_template_name(args: &MakeTemplateArgs, dir: &Path) -> String {
    if let Some(ref n) = args.name {
        return n.clone();
    }
    dir.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my-template")
        .to_string()
}

/// Determines whether a file should be created, overwritten, or skipped.
fn determine_action(dir: &Path, rel_path: &str, force: bool) -> FileAction {
    if dir.join(rel_path).exists() {
        if force {
            FileAction::Overwrite
        } else {
            FileAction::Skip
        }
    } else {
        FileAction::Create
    }
}

/// Builds the list of files to create, checking which already exist.
fn plan_files(dir: &Path, args: &MakeTemplateArgs) -> Vec<PlannedFile> {
    let mut plans: Vec<(&str, Option<&str>)> = vec![
        (".reporoller/template.toml", None),
        (
            "README.md",
            Some("template developer docs — excluded from output repos"),
        ),
        (
            "README.md.template",
            Some("scaffold for repos created from this template → renamed to README.md"),
        ),
        (
            ".gitignore",
            Some("template developer gitignore — excluded from output repos"),
        ),
        (
            ".gitignore.template",
            Some("starter gitignore for new repos → renamed to .gitignore"),
        ),
        (
            ".github/workflows/test-template.yml",
            Some("validates template structure in CI — excluded from output repos"),
        ),
        (
            ".github/workflows/ci.yml.template",
            Some("CI scaffold for new repos → renamed to ci.yml"),
        ),
    ];

    if args.renovate {
        plans.push(("renovate.json", Some("dependency update configuration")));
    }

    plans
        .into_iter()
        .map(|(rel, note)| PlannedFile {
            relative_path: rel.to_string(),
            action: determine_action(dir, rel, args.force),
            note: note.map(|s| s.to_string()),
        })
        .collect()
}

/// Writes `content` to `path`, creating parent directories as needed.
fn write_file(path: &Path, content: &str) -> Result<(), Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(Error::LoadFile)?;
    }
    fs::write(path, content).map_err(Error::LoadFile)?;
    Ok(())
}

/// Builds a map of relative path → file content for all scaffold files.
fn build_content_map(
    name: &str,
    description: &str,
    author: &str,
    renovate: bool,
) -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert(
        ".reporoller/template.toml".to_string(),
        generate_template_toml(name, description, author),
    );
    map.insert("README.md".to_string(), generate_readme(name, description));
    map.insert(
        "README.md.template".to_string(),
        generate_readme_template(name, description),
    );
    map.insert(".gitignore".to_string(), generate_gitignore());
    map.insert(
        ".gitignore.template".to_string(),
        generate_gitignore_template(),
    );
    map.insert(
        ".github/workflows/test-template.yml".to_string(),
        generate_test_template_workflow(name),
    );
    map.insert(
        ".github/workflows/ci.yml.template".to_string(),
        generate_ci_template_workflow(),
    );
    if renovate {
        map.insert("renovate.json".to_string(), generate_renovate_config());
    }
    map
}

/// Prints a formatted preview table to stdout.
fn print_preview(dir: &Path, files: &[PlannedFile]) {
    println!("\nInitializing template in: {}\n", dir.display());
    println!("  The following files will be affected:\n");
    for f in files {
        let action_str = match f.action {
            FileAction::Create => "CREATE   ",
            FileAction::Overwrite => "OVERWRITE",
            FileAction::Skip => "SKIP     ",
        };
        if let Some(ref note) = f.note {
            println!("  {}  {}  [{}]", action_str, f.relative_path, note);
        } else {
            println!("  {}  {}", action_str, f.relative_path);
        }
    }
    println!();
}

// ============================================================================
// Content generation
// ============================================================================

/// Generates the content of `.reporoller/template.toml`.
///
/// The `[template]` and `[templating]` sections are active; everything else is
/// commented out with inline documentation so template authors know all options.
pub fn generate_template_toml(name: &str, description: &str, author: &str) -> String {
    format!(
        r#"# .reporoller/template.toml
# Generated by: repo-roller make-template
#
# This file configures how this template repository behaves when somebody
# uses RepoRoller to create a new repository from it.
#
# Reference: https://github.com/pvandervelde/repo_roller

# ─── Required: Template Metadata ─────────────────────────────────────────────────
[template]
name        = "{name}"
description = "{description}"
author      = "{author}"
tags        = []  # e.g. ["rust", "service", "backend"]

# ─── File filtering ────────────────────────────────────────────────────────────
# Files matching exclude_patterns are present in this template repository but
# are NOT copied into repositories created from this template.
#
# Files with a .template suffix have that suffix stripped when copied, so:
#   README.md.template  →  README.md
#   .gitignore.template →  .gitignore
#   ci.yml.template     →  ci.yml
[templating]
exclude_patterns = [
  "README.md",                            # template developer docs — new repos get README.md.template
  ".gitignore",                           # template dev gitignore  — new repos get .gitignore.template
  ".github/workflows/test-template.yml",  # template CI — not relevant to output repos
]
# include_patterns = ["**/*"]   # default: include everything not excluded

# ─── Optional: Default visibility for repos created from this template ────────
# default_visibility = "private"   # Options: "public" | "private" | "internal"

# ─── Optional: Restrict repository type ────────────────────────────────────────
# [repository_type]
# type   = "service"       # Must match a type defined in the org config
# policy = "fixed"         # "fixed" = cannot override; "preferable" = can override

# ─── Optional: Template variables users supply at creation time ───────────────
# Variables are substituted via Handlebars ({{{{variable_name}}}}) in file
# content AND in file/directory names.
#
# [variables.service_name]
# description = "Name of the service"
# required    = true
# example     = "user-service"
# pattern     = "^[a-z][a-z0-9-]*$"   # Regex validation (optional)
# min_length  = 3
# max_length  = 63
# default     = ""

# ─── Optional: Repository feature settings ───────────────────────────────────
# [repository]
# has_wiki              = false
# has_issues            = true
# has_projects          = false
# has_discussions       = false
# delete_branch_on_merge = true
# security_advisories   = false

# ─── Optional: Pull request settings ────────────────────────────────────────
# [pull_requests]
# allow_squash_merge              = true
# allow_merge_commit              = false
# allow_rebase_merge              = false
# required_approving_review_count = 1
# dismiss_stale_reviews           = true
# require_code_owner_reviews      = false

# ─── Optional: Labels for created repos (additive with org labels) ───────────
# [[labels]]
# name        = "bug"
# color       = "d73a4a"
# description = "Something isn't working"

# [[labels]]
# name        = "enhancement"
# color       = "a2eeef"
# description = "New feature or request"

# ─── Optional: Webhooks for created repos ──────────────────────────────────
# [[webhooks]]
# url          = "https://hooks.example.com/repo-events"
# content_type = "json"
# active       = true
# events       = ["push", "pull_request"]
# secret       = "env:WEBHOOK_SECRET"   # Reference env var for secret

# ─── Optional: Branch-protection rulesets ──────────────────────────────────
# [[rulesets]]
# name        = "main-branch-protection"
# target      = "branch"
# enforcement = "active"
#
# [rulesets.conditions.ref_name]
# include = ["refs/heads/main"]
# exclude = []
#
# [[rulesets.rules]]
# type = "pull_request"
# [rulesets.rules.parameters]
# required_approving_review_count = 1
# dismiss_stale_reviews_on_push   = true

# ─── Optional: Default teams for created repos ─────────────────────────────
# [[teams]]
# slug         = "platform-team"
# access_level = "write"    # Options: "read" | "triage" | "write" | "maintain" | "admin"

# ─── Optional: Default collaborators for created repos ──────────────────────
# [[collaborators]]
# username     = "code-owner"
# access_level = "write"

# ─── Optional: Repository naming rules ─────────────────────────────────────
# [[naming_rules]]
# description        = "All repos from this template must end with -svc"
# required_suffix    = "-svc"
# forbidden_patterns = [".*--.*"]
# reserved_words     = ["test", "temp"]
# min_length         = 5
# max_length         = 50

# ─── Optional: Event notification webhooks ───────────────────────────────
# [[notifications.outbound_webhooks]]
# url     = "https://notify.example.com/hooks/repo-created"
# events  = ["repository.created"]
# secret  = "env:NOTIFY_SECRET"
"#,
        name = name,
        description = description,
        author = author,
    )
}

/// Generates `README.md` — template developer documentation shown on GitHub.
pub fn generate_readme(name: &str, description: &str) -> String {
    format!(
        r#"# {name} (Template)

> **This repository is a RepoRoller template.**
> It is used to create new repositories, not as a project itself.

## Description

{description}

## Using this template

Create a new repository from this template using RepoRoller:

```bash
repo-roller create --template {name} --name my-new-repo --owner my-org
```

## Template files

| File | Purpose |
|---|
| `README.md.template` | README scaffold copied and renamed to `README.md` in new repos |
| `.gitignore.template` | Gitignore scaffold copied and renamed to `.gitignore` in new repos |
| `.github/workflows/ci.yml.template` | CI workflow scaffold renamed to `ci.yml` in new repos |

## Template variables

Edit `.reporoller/template.toml` → `[variables]` to document the variables
your template uses. Variables are referenced as `{{{{variable_name}}}}` in
file content and file names.

## Configuration

See [`.reporoller/template.toml`](.reporoller/template.toml) for the full
template configuration including settings applied to every repository created
from this template.

## Development

To validate the template structure locally:

```bash
# Run the template CI workflow
act -j validate-template
```
"#,
        name = name,
        description = description,
    )
}

/// Generates `README.md.template` — the README scaffold for repos created from this template.
///
/// This file is copied into new repositories and renamed to `README.md`.
/// It uses `{{repo_name}}` and other Handlebars variables that are resolved
/// at creation time.
pub fn generate_readme_template(name: &str, description: &str) -> String {
    format!(
        r#"# {{{{repo_name}}}}

{description}

## Getting started

<!-- TODO: Add getting started instructions for repos created from the {name} template -->

## Contributing

<!-- TODO: Add contribution guidelines -->

## License

<!-- TODO: Add license information -->
"#,
        name = name,
        description = description,
    )
}

/// Generates `.gitignore` — the template developer's own gitignore.
///
/// This file is excluded from repositories created from this template.
pub fn generate_gitignore() -> String {
    r#"# Template developer gitignore
# This file governs what is ignored in THIS template repository.
# For repos created from this template, see .gitignore.template

# OS generated files
.DS_Store
.DS_Store?
._*
.Spotlight-V100
.Trashes
ehthumbs.db
Thumbs.db

# Editor directories and files
.idea/
.vscode/
*.swp
*.swo
*~

# Build artefacts
target/
dist/
build/
*.log
"#
    .to_string()
}

/// Generates `.gitignore.template` — starter gitignore for repos created from this template.
///
/// This file is copied into new repositories and renamed to `.gitignore`.
pub fn generate_gitignore_template() -> String {
    r#"# .gitignore for {{repo_name}}
# Generated from the {{template_name}} template

# OS generated files
.DS_Store
.DS_Store?
._*
.Spotlight-V100
.Trashes
ehthumbs.db
Thumbs.db

# Editor directories and files
.idea/
.vscode/
*.swp
*.swo
*~

# Build artefacts
target/
dist/
build/
*.log

# Environment / secrets
.env
.env.local
*.pem
*.key
"#
    .to_string()
}

/// Generates `.github/workflows/test-template.yml` — CI that validates the template itself.
///
/// This workflow runs in the template repository and checks structural integrity.
/// It is excluded from repositories created from this template.
pub fn generate_test_template_workflow(name: &str) -> String {
    format!(
        r#"# .github/workflows/test-template.yml
#
# Validates the {name} template structure.
# This workflow runs in the template repository only — it is NOT included
# in repositories created from this template (see exclude_patterns in
# .reporoller/template.toml).

name: Validate Template

on:
  push:
    branches: ["**"]
  pull_request:
    branches: [master, main]

permissions:
  contents: read

jobs:
  validate:
    name: Validate template structure
    runs-on: ubuntu-latest

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Check required template files exist
        run: |
          required_files=(
            ".reporoller/template.toml"
            "README.md"
            "README.md.template"
            ".gitignore"
            ".gitignore.template"
            ".github/workflows/ci.yml.template"
          )
          missing=0
          for f in "${{required_files[@]}}"; do
            if [ ! -f "$f" ]; then
              echo "MISSING: $f"
              missing=$((missing + 1))
            else
              echo "OK: $f"
            fi
          done
          if [ $missing -gt 0 ]; then
            echo "::error::$missing required template file(s) are missing"
            exit 1
          fi

      - name: Validate template.toml is valid TOML
        run: |
          python3 -c "
          import sys
          try:
              import tomllib
          except ImportError:
              import tomli as tomllib
          with open('.reporoller/template.toml', 'rb') as f:
              config = tomllib.load(f)
          assert 'template' in config, 'Missing [template] section'
          assert 'name' in config['template'], 'Missing template.name'
          assert 'description' in config['template'], 'Missing template.description'
          assert 'author' in config['template'], 'Missing template.author'
          print('template.toml is valid')
          "

      - name: Check for unresolved template placeholders in scaffold files
        run: |
          # Ensure .template scaffold files contain valid Handlebars syntax
          # (no obviously broken {{variable}} references)
          for f in README.md.template .gitignore.template .github/workflows/ci.yml.template; do
            if [ -f "$f" ]; then
              open=$(grep -o '{{{{' "$f" | wc -l)
              close=$(grep -o '}}}}' "$f" | wc -l)
              if [ "$open" != "$close" ]; then
                echo "WARN: possibly unbalanced braces in $f (open=$open close=$close)"
              else
                echo "OK: $f"
              fi
            fi
          done
"#,
        name = name,
    )
}

/// Generates `.github/workflows/ci.yml.template` — CI scaffold for created repos.
///
/// This file is copied into new repositories and renamed to `ci.yml`.
/// It uses `{{repo_name}}` which is resolved at repository creation time.
pub fn generate_ci_template_workflow() -> String {
    r#"# .github/workflows/ci.yml
# Generated from the {{template_name}} template for {{repo_name}}

name: CI

on:
  push:
    branches: ["**"]
  pull_request:
    branches: [master, main]

permissions:
  contents: read

jobs:
  build:
    name: Build and test
    runs-on: ubuntu-latest

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      # TODO: Add build steps appropriate for this project type
      # Examples:
      #
      # Rust:
      # - uses: dtolnay/rust-toolchain@stable
      # - run: cargo build --release
      # - run: cargo test
      #
      # Node.js:
      # - uses: actions/setup-node@v4
      #   with: { node-version: "20" }
      # - run: npm ci
      # - run: npm test
      #
      # Python:
      # - uses: actions/setup-python@v5
      #   with: { python-version: "3.12" }
      # - run: pip install -e ".[dev]"
      # - run: pytest

      - name: Placeholder build step
        run: echo "Build {{repo_name}} here"
"#
    .to_string()
}

/// Generates `renovate.json` — dependency-update automation configuration.
pub fn generate_renovate_config() -> String {
    r#"{
  "$schema": "https://docs.renovatebot.com/renovate-schema.json",
  "extends": [
    "config:recommended"
  ],
  "schedule": ["before 6am on Monday"],
  "labels": ["dependencies"],
  "prConcurrentLimit": 5,
  "platformCommit": true
}
"#
    .to_string()
}
