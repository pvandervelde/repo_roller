//! Property-based test for `copy_dir_contents`'s symlink-safety invariant.
//!
//! CRITICAL finding: `copy_dir_contents` (used by the source-only mirror's
//! working-tree-snapshot step) must never let a symlink entry's target
//! content reach the destination tree, for *any* mix of regular files and
//! symlinks present in the source directory — not just the hand-picked
//! examples in `git_tests.rs`.
//!
//! See docs/spec/interfaces/fork-mirror-operations.md §7.
//!
//! Unix-only: symlink creation requires elevated privileges on Windows by
//! default, and this workspace's CI (`.github/workflows/pr.yml`) runs
//! exclusively on `ubuntu-latest`.
#![cfg(unix)]

use super::*;
use proptest::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::os::unix::fs::symlink;
use temp_dir::TempDir;

const SECRET_CONTENT: &str = "TOP-SECRET-HOST-FILE-CONTENT-THAT-MUST-NEVER-BE-COPIED";

/// Recursively checks whether any regular file under `dir` contains `needle`.
/// Mirrors the helper in `git_tests.rs` (kept separate since proptests and
/// unit tests are compiled as distinct modules).
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

#[derive(Debug, Clone)]
enum GeneratedEntry {
    Regular(String),
    SymlinkToSecret,
}

fn entry_strategy() -> impl Strategy<Value = (String, GeneratedEntry)> {
    (
        "[a-z][a-z0-9]{1,8}",
        prop_oneof![
            "[a-zA-Z0-9 ]{1,20}".prop_map(GeneratedEntry::Regular),
            Just(GeneratedEntry::SymlinkToSecret),
        ],
    )
}

proptest! {
    // Filesystem I/O per case; keep the case count modest so the suite
    // stays fast while still exercising many entry-count/ordering/content
    // combinations.
    #![proptest_config(ProptestConfig { cases: 32, ..ProptestConfig::default() })]

    /// For any mix of regular files and symlinks-to-a-secret-outside-the-
    /// source-tree placed in a source directory (1 to 5 distinctly-named
    /// entries), `copy_dir_contents` never lets the secret file's content
    /// reach the destination tree, and — whenever the call succeeds — every
    /// *regular* sibling entry is still faithfully copied byte-for-byte
    /// (ruling out an overcorrected fix that empties the destination
    /// entirely instead of just skipping/rejecting symlinks).
    #[test]
    fn prop_copy_dir_contents_never_discloses_symlink_targets_and_preserves_regular_files(
        raw_entries in prop::collection::vec(entry_strategy(), 1..6)
    ) {
        // Distinct names only: duplicate names are last-write-wins
        // filesystem semantics, not a case this property is about.
        let mut seen = HashSet::new();
        let entries: Vec<_> = raw_entries
            .into_iter()
            .filter(|(name, _)| seen.insert(name.clone()))
            .collect();
        prop_assume!(!entries.is_empty());

        let secret_dir = TempDir::new().unwrap();
        let secret_path = secret_dir.path().join("secret.txt");
        fs::write(&secret_path, SECRET_CONTENT).unwrap();

        let source_dir = TempDir::new().unwrap();
        let dest_dir = TempDir::new().unwrap();

        let mut expected_regular_files = Vec::new();
        for (name, entry) in &entries {
            let path = source_dir.path().join(name);
            match entry {
                GeneratedEntry::Regular(content) => {
                    fs::write(&path, content).unwrap();
                    expected_regular_files.push((name.clone(), content.clone()));
                }
                GeneratedEntry::SymlinkToSecret => {
                    symlink(&secret_path, &path).unwrap();
                }
            }
        }

        let result = copy_dir_contents(source_dir.path(), dest_dir.path());

        prop_assert!(
            !directory_contains_content(dest_dir.path(), SECRET_CONTENT),
            "secret content must never leak into the destination via a symlink, \
             for entry set {:?}",
            entries
        );

        if result.is_ok() {
            for (name, content) in &expected_regular_files {
                let copied_path = dest_dir.path().join(name);
                prop_assert!(
                    copied_path.is_file(),
                    "regular sibling entry '{}' must still be copied when the call succeeds",
                    name
                );
                let copied_content = fs::read_to_string(&copied_path).unwrap();
                prop_assert_eq!(&copied_content, content);
            }
        }
    }
}
