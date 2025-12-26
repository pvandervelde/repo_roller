//! Repository contents domain types.
//!
//! This module contains types for working with GitHub repository contents,
//! including directory listings and file information.

use serde::{Deserialize, Serialize};

#[cfg(test)]
#[path = "contents_tests.rs"]
mod tests;

/// A single entry in a GitHub repository directory listing.
///
/// Represents files, directories, symlinks, and submodules returned by
/// the GitHub Contents API. Use this type when listing directory contents
/// to understand the structure of a repository.
///
/// See specs/interfaces/github-directory-listing.md for full specification.
///
/// # Examples
///
/// ```rust
/// use github_client::{TreeEntry, EntryType};
///
/// let entry = TreeEntry {
///     name: "library".to_string(),
///     path: "types/library".to_string(),
///     entry_type: EntryType::Dir,
///     sha: "abc123".to_string(),
///     size: 0,
///     download_url: None,
/// };
///
/// // Filter for directories only
/// if matches!(entry.entry_type, EntryType::Dir) {
///     println!("Found directory: {}", entry.name);
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeEntry {
    /// Entry name (e.g., "library", "config.toml")
    pub name: String,

    /// Full path within repository (e.g., "types/library", "types/service/config.toml")
    pub path: String,

    /// Entry type (file, directory, symlink, submodule)
    #[serde(rename = "type")]
    pub entry_type: EntryType,

    /// Git SHA of the entry
    pub sha: String,

    /// Size in bytes (0 for directories)
    pub size: u64,

    /// Download URL for files (None for directories)
    pub download_url: Option<String>,
}

/// Type of entry in a repository directory.
///
/// Maps to GitHub's content type field in the Contents API response.
/// Use pattern matching to distinguish between different entry types.
///
/// See specs/interfaces/github-directory-listing.md for full specification.
///
/// # Examples
///
/// ```rust
/// use github_client::EntryType;
///
/// let entry_type = EntryType::Dir;
///
/// match entry_type {
///     EntryType::Dir => println!("This is a directory"),
///     EntryType::File => println!("This is a file"),
///     EntryType::Symlink => println!("This is a symbolic link"),
///     EntryType::Submodule => println!("This is a git submodule"),
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryType {
    /// Regular file
    File,

    /// Directory (can contain other entries)
    Dir,

    /// Symbolic link
    Symlink,

    /// Git submodule reference
    Submodule,
}
