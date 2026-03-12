use std::fs;
use std::path::PathBuf;

use codeilus_core::{EventBus, Language};
use codeilus_parse::{detect_language, parse_repository, ParseConfig};

fn write_temp_file(ext: &str, contents: &str) -> PathBuf {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join(format!("test.{}", ext));
    fs::write(&path, contents).expect("write");
    path
}

#[test]
fn detect_language_by_extension() {
    let root = PathBuf::from("foo.py");
    assert_eq!(detect_language(&root), Some(Language::Python));
    let root = PathBuf::from("foo.ts");
    assert_eq!(detect_language(&root), Some(Language::TypeScript));
    let root = PathBuf::from("foo.js");
    assert_eq!(detect_language(&root), Some(Language::JavaScript));
    let root = PathBuf::from("foo.rs");
    assert_eq!(detect_language(&root), Some(Language::Rust));
    let root = PathBuf::from("foo.go");
    assert_eq!(detect_language(&root), Some(Language::Go));
    let root = PathBuf::from("foo.java");
    assert_eq!(detect_language(&root), Some(Language::Java));
}

#[test]
fn parse_repository_basic() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();

    fs::write(
        root.join("main.py"),
        "import os\n\nclass C:\n    pass\n\ndef foo():\n    return 1\n",
    )
    .unwrap();

    let config = ParseConfig::new(root.to_path_buf());
    let bus = EventBus::new();
    let parsed = parse_repository(&config, Some(&bus)).expect("parse_repository");

    assert!(!parsed.is_empty());
    let file = &parsed[0];
    assert_eq!(file.language, Language::Python);
    assert!(!file.symbols.is_empty());
}

