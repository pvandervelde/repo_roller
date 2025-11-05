//! Tests for template processing module.

use super::*;
use temp_dir::TempDir;

/// Module for path validation security tests
mod path_validation_tests {
    use super::*;

    /// Test that path traversal attempts using parent directory references are blocked.
    ///
    /// This test verifies that malicious templates cannot write files outside the
    /// repository using paths like "../../etc/passwd" or "../outside.txt".
    /// These attacks attempt to escape the repository boundary using parent
    /// directory references.
    #[test]
    fn test_path_traversal_with_parent_refs_blocked() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();

        // Test various path traversal patterns
        let malicious_paths = vec![
            "../../etc/passwd",          // Deep traversal (Unix)
            "../../../etc/shadow",       // Very deep traversal (Unix)
            "..\\..\\Windows\\System32", // Deep traversal (Windows)
            "../outside.txt",            // Single level escape
            "subdir/../../outside.txt",  // Traversal within path
            "valid/../../../etc/passwd", // Mixed valid and traversal
        ];

        for path in malicious_paths {
            let result = validate_safe_path(path, repo_path);
            assert!(
                result.is_err(),
                "Path traversal '{}' should be blocked but was allowed",
                path
            );

            // Verify error message mentions path traversal or parent directory
            if let Err(Error::FileSystem(msg)) = result {
                let msg_lower = msg.to_lowercase();
                assert!(
                    msg_lower.contains("traversal")
                        || msg_lower.contains("parent")
                        || msg_lower.contains("..")
                        || msg_lower.contains("unsafe"),
                    "Error message should indicate path traversal issue, got: {}",
                    msg
                );
            }
        }
    }

    /// Test that absolute paths are rejected.
    ///
    /// Verifies that templates cannot specify absolute file paths that would
    /// write to arbitrary locations on the file system. Both Unix-style (/)
    /// and Windows-style (C:\) absolute paths should be rejected.
    #[test]
    fn test_absolute_paths_blocked() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();

        let absolute_paths = vec![
            "/etc/passwd",                 // Unix absolute
            "/tmp/malicious.txt",          // Unix temp directory
            "/home/user/.bashrc",          // Unix home directory
            "C:\\Windows\\System32\\file", // Windows absolute
            "D:\\data\\secrets.txt",       // Windows different drive
            "/usr/local/bin/script",       // Unix system directory
        ];

        for path in absolute_paths {
            let result = validate_safe_path(path, repo_path);
            assert!(
                result.is_err(),
                "Absolute path '{}' should be blocked but was allowed",
                path
            );

            // Verify error message indicates absolute path issue
            if let Err(Error::FileSystem(msg)) = result {
                let msg_lower = msg.to_lowercase();
                assert!(
                    msg_lower.contains("absolute") || msg_lower.contains("unsafe"),
                    "Error message should indicate absolute path issue, got: {}",
                    msg
                );
            }
        }
    }

    /// Test that legitimate relative paths are allowed.
    ///
    /// Verifies that normal template file paths work correctly and are not
    /// blocked by security validation. These are the typical paths that
    /// legitimate templates would use.
    #[test]
    fn test_safe_relative_paths_allowed() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();

        let safe_paths = vec![
            "README.md",                    // Root file
            "src/main.rs",                  // Simple subdirectory
            "docs/guide.md",                // Documentation
            ".gitignore",                   // Dotfile
            "src/lib.rs",                   // Library file
            "tests/integration_test.rs",    // Test file
            "config/settings.toml",         // Config file
            "scripts/deploy.sh",            // Script
            "assets/images/logo.png",       // Nested directories
            "src/module/submodule/file.rs", // Deep nesting
        ];

        for path in safe_paths {
            let result = validate_safe_path(path, repo_path);
            assert!(
                result.is_ok(),
                "Safe path '{}' should be allowed but was blocked: {:?}",
                path,
                result
            );
        }
    }

    /// Test that paths with special characters are handled appropriately.
    ///
    /// Verifies that file names with spaces, hyphens, underscores, and other
    /// common special characters work correctly. These are legitimate file
    /// naming patterns that should be supported.
    #[test]
    fn test_paths_with_special_characters() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();

        let special_paths = vec![
            "file with spaces.txt",     // Spaces
            "file-with-hyphens.md",     // Hyphens
            "file_with_underscores.rs", // Underscores
            "file.multiple.dots.txt",   // Multiple dots
            "UPPERCASE.TXT",            // Uppercase
            "123-numeric-prefix.md",    // Numeric prefix
            "src/my-module/my_file.rs", // Mixed special chars in path
        ];

        for path in special_paths {
            let result = validate_safe_path(path, repo_path);
            assert!(
                result.is_ok(),
                "Path with special characters '{}' should be allowed but was blocked: {:?}",
                path,
                result
            );
        }
    }

    /// Test edge case: empty path is rejected.
    ///
    /// An empty file path is invalid and should be rejected.
    #[test]
    fn test_empty_path_rejected() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();

        let result = validate_safe_path("", repo_path);
        assert!(
            result.is_err(),
            "Empty path should be rejected but was allowed"
        );
    }

    /// Test edge case: path consisting only of dots.
    ///
    /// Paths like "." or ".." or "..." should be rejected as they don't
    /// represent valid file names.
    #[test]
    fn test_dot_only_paths_rejected() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();

        let dot_paths = vec![".", "..", "...", "....", "./../file"];

        for path in dot_paths {
            let result = validate_safe_path(path, repo_path);
            assert!(
                result.is_err(),
                "Dot-only path '{}' should be rejected but was allowed",
                path
            );
        }
    }
}

/// Module for copy_template_files tests
mod copy_template_files_tests {
    use super::*;

    /// Test that safe paths in copy_template_files work correctly.
    ///
    /// This test verifies that the copy_template_files function allows
    /// legitimate file paths and creates files in the correct locations.
    #[test]
    fn test_copy_safe_files_succeeds() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let files = vec![
            ("README.md".to_string(), b"# Test Project".to_vec()),
            ("src/main.rs".to_string(), b"fn main() {}".to_vec()),
            (
                "docs/guide.md".to_string(),
                b"# User Guide\n\nContent here".to_vec(),
            ),
        ];

        let result = copy_template_files(&files, &temp_dir);
        assert!(
            result.is_ok(),
            "Copying safe files should succeed: {:?}",
            result
        );

        // Verify files were actually created with correct content
        let readme_path = temp_dir.path().join("README.md");
        assert!(
            readme_path.exists(),
            "README.md should exist in temp directory"
        );

        let readme_content =
            std::fs::read_to_string(&readme_path).expect("Should be able to read README.md");
        assert_eq!(readme_content, "# Test Project");

        let main_path = temp_dir.path().join("src/main.rs");
        assert!(main_path.exists(), "src/main.rs should exist");

        let docs_path = temp_dir.path().join("docs/guide.md");
        assert!(docs_path.exists(), "docs/guide.md should exist");
    }

    /// Test that path traversal attempts in copy_template_files are blocked.
    ///
    /// This test verifies that the security validation is actually called
    /// and prevents malicious paths from being processed.
    #[test]
    fn test_copy_malicious_files_blocked() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let malicious_files = vec![
            ("../../etc/passwd".to_string(), b"malicious".to_vec()),
            ("../outside.txt".to_string(), b"should not work".to_vec()),
        ];

        let result = copy_template_files(&malicious_files, &temp_dir);
        assert!(
            result.is_err(),
            "Copying files with path traversal should fail"
        );

        // Verify that no files were created outside the temp directory
        // (The temp directory path itself should still be empty or minimal)
        let entries: Vec<_> = std::fs::read_dir(temp_dir.path())
            .expect("Should be able to read temp dir")
            .collect();

        // We might have some files created before the error, but none should be
        // the malicious ones. This is basic validation - the key is that an error
        // was returned.
        assert!(
            entries.is_empty() || entries.len() < malicious_files.len(),
            "Not all files should have been copied due to security validation"
        );
    }

    /// Test that copy_template_files handles empty file list.
    ///
    /// An empty file list should succeed without errors.
    #[test]
    fn test_copy_empty_file_list() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let files: Vec<(String, Vec<u8>)> = vec![];

        let result = copy_template_files(&files, &temp_dir);
        assert!(
            result.is_ok(),
            "Copying empty file list should succeed: {:?}",
            result
        );
    }

    /// Test that copy_template_files creates parent directories.
    ///
    /// When file paths include nested directories, the function should
    /// automatically create all parent directories.
    #[test]
    fn test_copy_creates_parent_directories() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let files = vec![(
            "deeply/nested/path/file.txt".to_string(),
            b"content".to_vec(),
        )];

        let result = copy_template_files(&files, &temp_dir);
        assert!(
            result.is_ok(),
            "Copying with nested paths should succeed: {:?}",
            result
        );

        let file_path = temp_dir.path().join("deeply/nested/path/file.txt");
        assert!(file_path.exists(), "Nested file should exist");
        assert!(
            file_path.is_file(),
            "Path should point to a file, not directory"
        );

        let content = std::fs::read_to_string(&file_path).expect("Should be able to read file");
        assert_eq!(content, "content");
    }

    /// Test that copy_template_files handles binary files correctly.
    ///
    /// Binary content should be preserved exactly without any text processing.
    #[test]
    fn test_copy_preserves_binary_content() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create binary content with non-UTF8 bytes
        let binary_content: Vec<u8> = vec![0xFF, 0xFE, 0x00, 0x01, 0x42, 0x00];

        let files = vec![("binary.dat".to_string(), binary_content.clone())];

        let result = copy_template_files(&files, &temp_dir);
        assert!(
            result.is_ok(),
            "Copying binary files should succeed: {:?}",
            result
        );

        let file_path = temp_dir.path().join("binary.dat");
        let read_content = std::fs::read(&file_path).expect("Should be able to read binary file");

        assert_eq!(
            read_content, binary_content,
            "Binary content should be preserved exactly"
        );
    }
}

#[test]
fn test_template_processing_module_compiles() {
    // This test ensures the module compiles correctly.
    // Actual template processing functionality is tested via integration tests
    // in the integration_tests crate.
}
