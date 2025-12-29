use super::*;
use serde_json::{from_str, to_string};

#[test]
fn test_entry_type_serialization() {
    assert_eq!(to_string(&EntryType::File).unwrap(), r#""file""#);
    assert_eq!(to_string(&EntryType::Dir).unwrap(), r#""dir""#);
    assert_eq!(to_string(&EntryType::Symlink).unwrap(), r#""symlink""#);
    assert_eq!(to_string(&EntryType::Submodule).unwrap(), r#""submodule""#);
}

#[test]
fn test_entry_type_deserialization() {
    assert_eq!(from_str::<EntryType>(r#""file""#).unwrap(), EntryType::File);
    assert_eq!(from_str::<EntryType>(r#""dir""#).unwrap(), EntryType::Dir);
    assert_eq!(
        from_str::<EntryType>(r#""symlink""#).unwrap(),
        EntryType::Symlink
    );
    assert_eq!(
        from_str::<EntryType>(r#""submodule""#).unwrap(),
        EntryType::Submodule
    );
}

#[test]
fn test_tree_entry_serialization() {
    let entry = TreeEntry {
        name: "config.toml".to_string(),
        path: "types/library/config.toml".to_string(),
        entry_type: EntryType::File,
        sha: "abc123def456".to_string(),
        size: 1024,
        download_url: Some(
            "https://raw.githubusercontent.com/org/repo/main/config.toml".to_string(),
        ),
    };

    let json_str = to_string(&entry).expect("Failed to serialize TreeEntry");
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Failed to parse JSON");

    assert_eq!(parsed["name"], "config.toml");
    assert_eq!(parsed["path"], "types/library/config.toml");
    assert_eq!(parsed["type"], "file");
    assert_eq!(parsed["sha"], "abc123def456");
    assert_eq!(parsed["size"], 1024);
    assert_eq!(
        parsed["download_url"],
        "https://raw.githubusercontent.com/org/repo/main/config.toml"
    );
}

#[test]
fn test_tree_entry_deserialization() {
    let json_str = r#"{
        "name": "library",
        "path": "types/library",
        "type": "dir",
        "sha": "xyz789",
        "size": 0,
        "download_url": null
    }"#;

    let entry: TreeEntry = from_str(json_str).expect("Failed to deserialize TreeEntry");

    assert_eq!(entry.name, "library");
    assert_eq!(entry.path, "types/library");
    assert_eq!(entry.entry_type, EntryType::Dir);
    assert_eq!(entry.sha, "xyz789");
    assert_eq!(entry.size, 0);
    assert_eq!(entry.download_url, None);
}

#[test]
fn test_tree_entry_directory_no_download_url() {
    let entry = TreeEntry {
        name: "src".to_string(),
        path: "src".to_string(),
        entry_type: EntryType::Dir,
        sha: "dir123".to_string(),
        size: 0,
        download_url: None,
    };

    assert_eq!(entry.entry_type, EntryType::Dir);
    assert_eq!(entry.size, 0);
    assert_eq!(entry.download_url, None);
}
