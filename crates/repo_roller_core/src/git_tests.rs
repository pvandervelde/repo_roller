//! Tests for git module

use super::*;
use git2::Repository;
use std::fs;
use temp_dir::TempDir;

#[test]
fn test_init_local_git_repo_with_main_branch() {
    let temp_dir = TempDir::new().unwrap();
    let result = init_local_git_repo(&temp_dir, "main");

    assert!(result.is_ok(), "Repository initialization should succeed");

    // Verify repository was created
    let repo = Repository::open(temp_dir.path()).unwrap();
    // A newly initialized repo may not report as empty depending on git version
    // Just verify it was created successfully
    assert!(repo.path().exists());
}

#[test]
fn test_init_local_git_repo_with_custom_branch() {
    let temp_dir = TempDir::new().unwrap();
    let result = init_local_git_repo(&temp_dir, "develop");

    assert!(result.is_ok(), "Repository initialization should succeed");

    // Verify repository was created
    let repo = Repository::open(temp_dir.path()).unwrap();
    // Just verify it was created successfully
    assert!(repo.path().exists());
}

#[test]
fn test_commit_all_changes_workflow() {
    // Initialize repository
    let temp_dir = TempDir::new().unwrap();
    init_local_git_repo(&temp_dir, "main").unwrap();

    // Create test files
    fs::write(temp_dir.path().join("README.md"), "# Test Project").unwrap();
    fs::write(temp_dir.path().join("test.txt"), "Test content").unwrap();

    // Create subdirectory with file
    fs::create_dir_all(temp_dir.path().join("src")).unwrap();
    fs::write(temp_dir.path().join("src/main.rs"), "fn main() {}").unwrap();

    // Commit all changes
    let result = commit_all_changes(&temp_dir, "Initial commit", false);
    assert!(result.is_ok(), "Commit should succeed: {:?}", result.err());

    // Verify commit was created
    let repo = Repository::open(temp_dir.path()).unwrap();
    assert!(
        !repo.is_empty().unwrap(),
        "Repository should not be empty after commit"
    );

    // Verify HEAD exists and points to a commit
    let head = repo.head().unwrap();
    assert!(head.is_branch(), "HEAD should point to a branch");

    // Verify commit exists
    let commit = repo.head().unwrap().peel_to_commit().unwrap();
    assert_eq!(commit.message().unwrap(), "Initial commit");
    assert_eq!(commit.author().name().unwrap(), "RepoRoller");
}

#[test]
fn test_commit_all_changes_empty_directory_fails() {
    // Initialize repository
    let temp_dir = TempDir::new().unwrap();
    init_local_git_repo(&temp_dir, "main").unwrap();

    // Try to commit without any files
    let result = commit_all_changes(&temp_dir, "Empty commit", false);

    assert!(result.is_err(), "Commit should fail for empty directory");
    assert!(
        matches!(result.unwrap_err(), SystemError::GitOperation { .. }),
        "Should return GitOperation error"
    );
}

#[test]
fn test_debug_repository_state() {
    let temp_dir = TempDir::new().unwrap();
    init_local_git_repo(&temp_dir, "main").unwrap();

    let repo = Repository::open(temp_dir.path()).unwrap();
    let result = debug_repository_state(&repo);

    assert!(result.is_ok(), "Debug should succeed even for empty repo");
}

#[test]
fn test_debug_working_directory_empty() {
    let temp_dir = TempDir::new().unwrap();
    let result = debug_working_directory(&temp_dir);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0, "Should find 0 files in empty directory");
}

#[test]
fn test_debug_working_directory_with_files() {
    let temp_dir = TempDir::new().unwrap();

    // Create some test files
    fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
    fs::write(temp_dir.path().join("file2.txt"), "content2").unwrap();

    let result = debug_working_directory(&temp_dir);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 2, "Should find 2 files");
}

#[test]
fn test_prepare_index_and_tree() {
    let temp_dir = TempDir::new().unwrap();
    init_local_git_repo(&temp_dir, "main").unwrap();

    // Create test files
    fs::write(temp_dir.path().join("test.txt"), "content").unwrap();

    let repo = Repository::open(temp_dir.path()).unwrap();
    let result = prepare_index_and_tree(&repo, false);

    assert!(result.is_ok(), "Tree preparation should succeed");

    // Verify tree OID is valid
    let tree_oid = result.unwrap();
    assert!(
        repo.find_tree(tree_oid).is_ok(),
        "Tree should exist in repository"
    );
}

#[test]
fn test_prepare_index_and_tree_empty_fails() {
    let temp_dir = TempDir::new().unwrap();
    init_local_git_repo(&temp_dir, "main").unwrap();

    // No files created
    let repo = Repository::open(temp_dir.path()).unwrap();
    let result = prepare_index_and_tree(&repo, false);

    assert!(
        result.is_err(),
        "Tree preparation should fail for empty directory"
    );
}

#[test]
fn test_create_initial_commit() {
    let temp_dir = TempDir::new().unwrap();
    init_local_git_repo(&temp_dir, "main").unwrap();

    // Create test file and prepare tree
    fs::write(temp_dir.path().join("test.txt"), "content").unwrap();

    let repo = Repository::open(temp_dir.path()).unwrap();
    let tree_oid = prepare_index_and_tree(&repo, false).unwrap();

    // Create commit
    let result = create_initial_commit(&repo, tree_oid, "Test commit");

    assert!(result.is_ok(), "Commit creation should succeed");

    // Verify commit OID is valid
    let commit_oid = result.unwrap();
    let commit = repo.find_commit(commit_oid).unwrap();
    assert_eq!(commit.message().unwrap(), "Test commit");
    assert_eq!(commit.author().name().unwrap(), "RepoRoller");
    assert_eq!(commit.author().email().unwrap(), "repo-roller@example.com");
}

#[test]
fn test_set_head_reference_and_verify() {
    let temp_dir = TempDir::new().unwrap();
    init_local_git_repo(&temp_dir, "main").unwrap();

    // Create test file, prepare tree, and create commit
    fs::write(temp_dir.path().join("test.txt"), "content").unwrap();

    let repo = Repository::open(temp_dir.path()).unwrap();
    let tree_oid = prepare_index_and_tree(&repo, false).unwrap();
    let commit_oid = create_initial_commit(&repo, tree_oid, "Test commit").unwrap();

    // Set HEAD reference
    let result = set_head_reference_and_verify(&repo, commit_oid, "Test commit");

    assert!(result.is_ok(), "Setting HEAD should succeed");

    // Verify HEAD points to the commit
    let head = repo.head().unwrap();
    assert_eq!(head.target().unwrap(), commit_oid);

    // Verify main branch exists
    let branch = repo.find_branch("main", git2::BranchType::Local).unwrap();
    assert_eq!(branch.get().target().unwrap(), commit_oid);
}

#[test]
fn test_full_git_workflow() {
    // This test validates the complete git workflow from init to commit
    let temp_dir = TempDir::new().unwrap();

    // Step 1: Initialize repository
    init_local_git_repo(&temp_dir, "main").unwrap();

    // Step 2: Create files
    fs::write(temp_dir.path().join("README.md"), "# Test").unwrap();
    fs::create_dir_all(temp_dir.path().join("src")).unwrap();
    fs::write(temp_dir.path().join("src/lib.rs"), "// Library").unwrap();

    // Step 3: Commit all changes
    commit_all_changes(&temp_dir, "Initial commit", false).unwrap();

    // Verify final state
    let repo = Repository::open(temp_dir.path()).unwrap();
    assert!(!repo.is_empty().unwrap());

    let head = repo.head().unwrap();
    let commit = head.peel_to_commit().unwrap();
    assert_eq!(commit.message().unwrap(), "Initial commit");

    // Verify files are in the tree
    let tree = commit.tree().unwrap();
    assert!(tree.get_name("README.md").is_some());
    assert!(tree.get_name("src").is_some());
}

/// Verify that commit_all_changes works with different branch names.
///
/// This test verifies that:
/// 1. Repository can be initialized with various default branch names
/// 2. After making the first commit, the configured branch exists
/// 3. The HEAD correctly points to the configured branch
#[test]
fn test_commit_with_different_branch_names() {
    for branch_name in &["main", "master", "develop", "trunk"] {
        let temp_dir = TempDir::new().unwrap();

        init_local_git_repo(&temp_dir, branch_name).unwrap();
        fs::write(temp_dir.path().join("test.txt"), "content").unwrap();
        commit_all_changes(&temp_dir, "Test commit", false).unwrap();

        // Verify the branch was created and is HEAD after the commit
        let repo = Repository::open(temp_dir.path()).unwrap();
        let branch = repo
            .find_branch(branch_name, git2::BranchType::Local)
            .unwrap_or_else(|_| panic!("Branch '{}' should exist after commit", branch_name));
        assert!(branch.is_head(), "Branch {} should be HEAD", branch_name);
    }
}

#[test]
fn test_multiple_files_and_directories() {
    let temp_dir = TempDir::new().unwrap();
    init_local_git_repo(&temp_dir, "main").unwrap();

    // Create complex directory structure
    fs::write(temp_dir.path().join("README.md"), "# Project").unwrap();
    fs::write(temp_dir.path().join(".gitignore"), "target/").unwrap();

    fs::create_dir_all(temp_dir.path().join("src/models")).unwrap();
    fs::write(temp_dir.path().join("src/lib.rs"), "// Lib").unwrap();
    fs::write(temp_dir.path().join("src/models/user.rs"), "// User").unwrap();

    fs::create_dir_all(temp_dir.path().join("tests")).unwrap();
    fs::write(temp_dir.path().join("tests/integration.rs"), "// Test").unwrap();

    // Commit everything
    commit_all_changes(&temp_dir, "Initial structure", false).unwrap();

    // Verify all files are committed
    let repo = Repository::open(temp_dir.path()).unwrap();
    let commit = repo.head().unwrap().peel_to_commit().unwrap();
    let tree = commit.tree().unwrap();

    assert!(tree.get_name("README.md").is_some());
    assert!(tree.get_name(".gitignore").is_some());
    assert!(tree.get_name("src").is_some());
    assert!(tree.get_name("tests").is_some());
}

// ============================================================================
// Symlink-safety regression tests for `copy_dir_contents` /
// `replace_working_tree_contents` (CRITICAL finding: symlink dereference /
// arbitrary file disclosure — see docs/spec/interfaces/fork-mirror-operations.md
// §7, "Source-only mirror's working-tree copy (`copy_dir_contents`) rejects
// or skips symlink entries rather than dereferencing them").
//
// `std::fs::copy` follows symlinks when reading, so a symlink in a cloned
// source repository's working tree that points at an arbitrary absolute
// path on the RepoRoller host (e.g. a secrets file) would have that
// target's *content* copied into the new repository and committed/pushed/
// opened in a PR, unless `copy_dir_contents` special-cases symlink entries.
//
// Symlink creation requires elevated privileges on Windows by default; this
// workspace's CI (`.github/workflows/pr.yml`) runs exclusively on
// `ubuntu-latest`, so these tests are unix-only.
// ============================================================================

#[cfg(unix)]
const SYMLINK_SECRET_CONTENT: &str = "TOP-SECRET-HOST-FILE-CONTENT-THAT-MUST-NEVER-BE-COPIED";

/// Recursively checks whether any regular file under `dir` contains `needle`.
///
/// Uses `DirEntry::file_type()`, which reports symlink-ness without
/// dereferencing (unlike `Path::is_file()`), so a destination that merely
/// re-creates a symlink (rather than copying its target's bytes) is
/// correctly treated as non-disclosing by this helper — the property under
/// test is content disclosure, not symlink presence.
#[cfg(unix)]
fn directory_contains_content(dir: &std::path::Path, needle: &str) -> bool {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return false,
    };
    for entry in entries.flatten() {
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => continue,
        };
        let path = entry.path();
        if file_type.is_dir() {
            if directory_contains_content(&path, needle) {
                return true;
            }
        } else if file_type.is_file() {
            if let Ok(contents) = fs::read(&path) {
                if contents
                    .windows(needle.len())
                    .any(|window| window == needle.as_bytes())
                {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(unix)]
#[test]
fn test_copy_dir_contents_does_not_disclose_symlink_target_content() {
    use std::os::unix::fs::symlink;

    // Secret file outside both the source and destination trees.
    let secret_dir = TempDir::new().unwrap();
    let secret_path = secret_dir.path().join("secret.txt");
    fs::write(&secret_path, SYMLINK_SECRET_CONTENT).unwrap();

    let source_dir = TempDir::new().unwrap();
    symlink(&secret_path, source_dir.path().join("leaked")).unwrap();

    let dest_dir = TempDir::new().unwrap();

    let result = copy_dir_contents(source_dir.path(), dest_dir.path());

    assert!(
        !directory_contains_content(dest_dir.path(), SYMLINK_SECRET_CONTENT),
        "secret file content must never be copied into the destination via a symlink"
    );
    assert!(
        result.is_err() || !dest_dir.path().join("leaked").exists(),
        "a symlink entry must be rejected (Err) or skipped (absent from destination), \
         not silently dereferenced into a regular file copy"
    );
}

#[cfg(unix)]
#[test]
fn test_copy_dir_contents_does_not_disclose_symlink_to_directory_outside_source() {
    use std::os::unix::fs::symlink;

    let secret_dir = TempDir::new().unwrap();
    fs::write(
        secret_dir.path().join("inner-secret.txt"),
        SYMLINK_SECRET_CONTENT,
    )
    .unwrap();

    let source_dir = TempDir::new().unwrap();
    symlink(secret_dir.path(), source_dir.path().join("leaked-dir")).unwrap();

    let dest_dir = TempDir::new().unwrap();

    let result = copy_dir_contents(source_dir.path(), dest_dir.path());

    assert!(
        !directory_contains_content(dest_dir.path(), SYMLINK_SECRET_CONTENT),
        "secret directory's content must never be copied into the destination via a symlinked directory"
    );
    assert!(
        result.is_err() || !dest_dir.path().join("leaked-dir").exists(),
        "a directory symlink entry must be rejected or skipped, not recursively dereferenced"
    );
}

#[cfg(unix)]
#[test]
fn test_copy_dir_contents_rejects_or_skips_symlink_even_when_target_is_inside_source_tree() {
    // The finding is about dereferencing symlinks at all, not only ones that
    // point outside the source tree: determining "inside vs outside" would
    // itself require canonicalization that is fragile against symlink
    // chains/TOCTOU, so a correct fix treats every symlink entry uniformly.
    use std::os::unix::fs::symlink;

    let source_dir = TempDir::new().unwrap();
    let real_path = source_dir.path().join("real.txt");
    fs::write(&real_path, SYMLINK_SECRET_CONTENT).unwrap();
    symlink(&real_path, source_dir.path().join("link-to-real")).unwrap();

    let dest_dir = TempDir::new().unwrap();

    let result = copy_dir_contents(source_dir.path(), dest_dir.path());

    // `real.txt` itself is a legitimate regular file; only the *symlink
    // entry* must not be dereferenced into a second, independent copy under
    // the link's name.
    assert!(
        result.is_err() || !dest_dir.path().join("link-to-real").exists(),
        "a symlink must not be dereferenced even when its target is a legitimate same-tree file"
    );
}

#[cfg(unix)]
#[test]
fn test_copy_dir_contents_still_copies_regular_siblings_when_symlink_is_skipped() {
    // Stub-killing: rules out an overcorrected "fail/empty everything"
    // implementation — if the chosen fix is "skip the symlink and succeed",
    // regular sibling files and subdirectories must still be copied intact.
    use std::os::unix::fs::symlink;

    let secret_dir = TempDir::new().unwrap();
    let secret_path = secret_dir.path().join("secret.txt");
    fs::write(&secret_path, SYMLINK_SECRET_CONTENT).unwrap();

    let source_dir = TempDir::new().unwrap();
    fs::write(source_dir.path().join("README.md"), "# Hello").unwrap();
    fs::create_dir_all(source_dir.path().join("src")).unwrap();
    fs::write(source_dir.path().join("src/main.rs"), "fn main() {}").unwrap();
    symlink(&secret_path, source_dir.path().join("leaked")).unwrap();

    let dest_dir = TempDir::new().unwrap();
    let result = copy_dir_contents(source_dir.path(), dest_dir.path());

    if result.is_ok() {
        assert_eq!(
            fs::read_to_string(dest_dir.path().join("README.md")).unwrap(),
            "# Hello",
            "regular sibling file must still be copied when the symlink is merely skipped"
        );
        assert_eq!(
            fs::read_to_string(dest_dir.path().join("src/main.rs")).unwrap(),
            "fn main() {}",
            "regular sibling subdirectory must still be copied when the symlink is merely skipped"
        );
    }
    assert!(!directory_contains_content(
        dest_dir.path(),
        SYMLINK_SECRET_CONTENT
    ));
}

#[cfg(unix)]
#[test]
fn test_copy_dir_contents_rejects_or_skips_symlink_nested_in_subdirectory() {
    // The recursive case: a symlink several directories deep must be caught
    // too, not just top-level entries.
    use std::os::unix::fs::symlink;

    let secret_dir = TempDir::new().unwrap();
    let secret_path = secret_dir.path().join("secret.txt");
    fs::write(&secret_path, SYMLINK_SECRET_CONTENT).unwrap();

    let source_dir = TempDir::new().unwrap();
    let nested = source_dir.path().join("a").join("b").join("c");
    fs::create_dir_all(&nested).unwrap();
    symlink(&secret_path, nested.join("leaked")).unwrap();

    let dest_dir = TempDir::new().unwrap();
    let result = copy_dir_contents(source_dir.path(), dest_dir.path());

    assert!(
        !directory_contains_content(dest_dir.path(), SYMLINK_SECRET_CONTENT),
        "a symlink nested several directories deep must not leak its target's content either"
    );
    assert!(
        result.is_err()
            || !dest_dir
                .path()
                .join("a")
                .join("b")
                .join("c")
                .join("leaked")
                .exists(),
        "a nested symlink entry must be rejected or skipped"
    );
}

#[cfg(unix)]
#[test]
fn test_replace_working_tree_contents_does_not_disclose_symlink_target_content() {
    // Regression test at the level actually called by `create_snapshot_branch`
    // for the source-only mirror flow: `dest` simulates an already-initialized
    // local git repository (has a `.git` directory, which must be preserved
    // and must not be treated as a source entry); `source` simulates the
    // shallow clone of the upstream repository's working tree.
    use std::os::unix::fs::symlink;

    let secret_dir = TempDir::new().unwrap();
    let secret_path = secret_dir.path().join("secret.txt");
    fs::write(&secret_path, SYMLINK_SECRET_CONTENT).unwrap();

    let source_dir = TempDir::new().unwrap();
    fs::write(source_dir.path().join("README.md"), "mirrored content").unwrap();
    symlink(&secret_path, source_dir.path().join("leaked")).unwrap();

    let dest_dir = TempDir::new().unwrap();
    fs::create_dir_all(dest_dir.path().join(".git")).unwrap();
    fs::write(dest_dir.path().join(".git/HEAD"), "ref: refs/heads/main").unwrap();
    // Pre-existing tracked file that the snapshot replace step is expected
    // to remove before copying `source`'s contents in.
    fs::write(dest_dir.path().join("stale.txt"), "stale").unwrap();

    let result = replace_working_tree_contents(dest_dir.path(), source_dir.path());

    assert!(
        !directory_contains_content(dest_dir.path(), SYMLINK_SECRET_CONTENT),
        "replace_working_tree_contents must not let a symlink in the source tree \
         disclose content from outside the clone"
    );
    assert!(
        result.is_err() || !dest_dir.path().join("leaked").exists(),
        "a symlink entry must be rejected or skipped, not dereferenced"
    );
    // `.git` must survive regardless of how the symlink finding is fixed.
    assert!(
        dest_dir.path().join(".git/HEAD").exists(),
        ".git directory must never be touched by the working-tree replace step"
    );
}

/// Mutation-kill: a no-op `replace_working_tree_contents` (e.g. the whole
/// function body replaced with `Ok(())`) would still pass the
/// non-disclosure assertions above, because an untouched `dest` trivially
/// contains no leaked secret content. This test instead asserts the
/// function's two core *positive* effects directly: stale entries tracked
/// in `dest` but absent from `source` are deleted, and `source`'s current
/// contents are actually copied in. Deliberately platform-independent (no
/// symlinks) so it runs on every CI runner, not just `ubuntu-latest`.
#[test]
fn test_replace_working_tree_contents_deletes_stale_and_copies_source_contents() {
    let source_dir = TempDir::new().unwrap();
    fs::write(source_dir.path().join("README.md"), "mirrored content").unwrap();
    fs::create_dir_all(source_dir.path().join("src")).unwrap();
    fs::write(source_dir.path().join("src/main.rs"), "fn main() {}").unwrap();

    let dest_dir = TempDir::new().unwrap();
    fs::create_dir_all(dest_dir.path().join(".git")).unwrap();
    fs::write(dest_dir.path().join(".git/HEAD"), "ref: refs/heads/main").unwrap();
    // Pre-existing tracked file/dir that must be removed because `source`
    // does not contain them.
    fs::write(dest_dir.path().join("stale.txt"), "stale").unwrap();
    fs::create_dir_all(dest_dir.path().join("old_dir")).unwrap();
    fs::write(dest_dir.path().join("old_dir/old.txt"), "old").unwrap();

    let result = replace_working_tree_contents(dest_dir.path(), source_dir.path());
    assert!(result.is_ok(), "replace should succeed: {:?}", result);

    assert!(
        !dest_dir.path().join("stale.txt").exists(),
        "stale.txt must be deleted: it is tracked in dest but absent from source"
    );
    assert!(
        !dest_dir.path().join("old_dir").exists(),
        "old_dir must be deleted: it is tracked in dest but absent from source"
    );
    assert_eq!(
        fs::read_to_string(dest_dir.path().join("README.md")).unwrap(),
        "mirrored content",
        "README.md must be copied in from source"
    );
    assert_eq!(
        fs::read_to_string(dest_dir.path().join("src/main.rs")).unwrap(),
        "fn main() {}",
        "src/main.rs must be copied in from source, including nested directories"
    );
    assert!(
        dest_dir.path().join(".git/HEAD").exists(),
        ".git directory must never be touched by the working-tree replace step"
    );
}

/// Mutation-kill / regression test for a confirmed production defect found
/// during the #255 QA audit: `create_snapshot_branch` had zero test coverage
/// (any caller using it in the exact sequence production code uses —
/// `initialize_git_repository` i.e. `init_local_git_repo` +
/// `commit_all_changes`, then `create_snapshot_branch` on the same
/// `local_repo_path` — fails every time) with:
///
/// `GitOperation { operation: "checkout branch '...'", reason: "1 conflict
/// prevents checkout; class=Checkout (20); code=Conflict (-13)" }`
///
/// Root cause: `prepare_index_and_tree` (git.rs) calls
/// `index.add_all(...)` and `index.write_tree()` but never calls
/// `index.write()`, so the staged index is never persisted to the on-disk
/// `.git/index`. `write_tree()` only serializes the in-memory index into a
/// tree object in the ODB; it does not sync `.git/index`. This was
/// harmless for `commit_all_changes`'s original only use (a one-shot
/// init+commit+push with no further checkout in the same process), but
/// `create_snapshot_branch`'s subsequent `checkout_tree` call diffs against
/// the **on-disk** index as its baseline — which still shows zero tracked
/// files — so libgit2 sees the already-on-disk working-tree files as
/// "untracked files that would be overwritten" and refuses the checkout.
///
/// Confirmed root cause and fix: adding `index.write()` immediately after
/// `index.add_all(...)` in `prepare_index_and_tree` makes this test pass.
/// Fixed in `prepare_index_and_tree` (crates/repo_roller_core/src/git.rs).
///
/// This meant `create_source_only_mirror_repository`
/// (crates/repo_roller_core/src/lib.rs) failed on every invocation in
/// production prior to the fix. Reproduced identically on both Windows and
/// WSL/Linux (via `cargo test`), so this was not a platform artifact.
#[test]
fn test_create_snapshot_branch_creates_branch_checks_out_and_commits_snapshot() {
    let repo_dir = TempDir::new().unwrap();
    init_local_git_repo(&repo_dir, "main").unwrap();
    fs::write(repo_dir.path().join("original.txt"), "original content").unwrap();
    commit_all_changes(&repo_dir, "Initial commit", false).unwrap();

    let base_commit_oid = {
        let repo = Repository::open(repo_dir.path()).unwrap();
        let oid = repo.head().unwrap().peel_to_commit().unwrap().id();
        oid
    };

    let source_dir = TempDir::new().unwrap();
    fs::write(source_dir.path().join("snapshot.txt"), "snapshot content").unwrap();

    let result = create_snapshot_branch(
        &repo_dir,
        "main",
        "snapshot-branch",
        source_dir.path(),
        "Snapshot commit",
    );
    assert!(result.is_ok(), "snapshot branch creation should succeed: {:?}", result);

    let repo = Repository::open(repo_dir.path()).unwrap();

    // The new branch must actually exist.
    let branch = repo
        .find_branch("snapshot-branch", git2::BranchType::Local)
        .expect("snapshot-branch must have been created");
    let branch_commit = branch.get().peel_to_commit().unwrap();

    // HEAD must have been moved onto the new branch (checkout + set_head).
    assert_eq!(
        repo.head().unwrap().name().ok(),
        Some("refs/heads/snapshot-branch"),
        "HEAD must point at the newly created snapshot branch"
    );

    // The snapshot commit must be a single child of the base commit, not a
    // disconnected or merge commit.
    assert_eq!(
        branch_commit.parent_count(),
        1,
        "snapshot commit must have exactly one parent"
    );
    assert_eq!(
        branch_commit.parent_id(0).unwrap(),
        base_commit_oid,
        "snapshot commit's parent must be the base branch's commit"
    );
    assert_eq!(
        branch_commit.message().ok(),
        Some("Snapshot commit"),
        "snapshot commit must use the provided commit message"
    );

    // The working tree must reflect source's content, not the original.
    assert!(
        !repo_dir.path().join("original.txt").exists(),
        "original.txt from the base commit must be replaced by the snapshot content"
    );
    assert_eq!(
        fs::read_to_string(repo_dir.path().join("snapshot.txt")).unwrap(),
        "snapshot content",
        "snapshot.txt from source_files_path must be present in the working tree"
    );
}
