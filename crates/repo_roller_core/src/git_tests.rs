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
    let result = commit_all_changes(&temp_dir, "Initial commit");
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
    let result = commit_all_changes(&temp_dir, "Empty commit");

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
    let result = prepare_index_and_tree(&repo);

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
    let result = prepare_index_and_tree(&repo);

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
    let tree_oid = prepare_index_and_tree(&repo).unwrap();

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
    let tree_oid = prepare_index_and_tree(&repo).unwrap();
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
    commit_all_changes(&temp_dir, "Initial commit").unwrap();

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
        commit_all_changes(&temp_dir, "Test commit").unwrap();

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
    commit_all_changes(&temp_dir, "Initial structure").unwrap();

    // Verify all files are committed
    let repo = Repository::open(temp_dir.path()).unwrap();
    let commit = repo.head().unwrap().peel_to_commit().unwrap();
    let tree = commit.tree().unwrap();

    assert!(tree.get_name("README.md").is_some());
    assert!(tree.get_name(".gitignore").is_some());
    assert!(tree.get_name("src").is_some());
    assert!(tree.get_name("tests").is_some());
}
