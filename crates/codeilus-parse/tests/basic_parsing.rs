use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use codeilus_core::{EventBus, FileId, Language, SymbolKind};
use codeilus_parse::{
    detect_language, parse_repository, parse_repository_incremental, ExistingFile, ParseConfig,
};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[test]
fn detect_language_by_extension() {
    assert_eq!(detect_language(&PathBuf::from("foo.py")), Some(Language::Python));
    assert_eq!(detect_language(&PathBuf::from("foo.ts")), Some(Language::TypeScript));
    assert_eq!(detect_language(&PathBuf::from("foo.js")), Some(Language::JavaScript));
    assert_eq!(detect_language(&PathBuf::from("foo.rs")), Some(Language::Rust));
    assert_eq!(detect_language(&PathBuf::from("foo.go")), Some(Language::Go));
    assert_eq!(detect_language(&PathBuf::from("foo.java")), Some(Language::Java));
}

#[test]
fn parse_repository_finds_all_fixture_files() {
    let config = ParseConfig::new(fixtures_dir());
    let bus = EventBus::new(64);
    let parsed = parse_repository(&config, Some(&bus)).expect("parse_repository");

    // Should find all 3 fixture files (py, ts, rs)
    assert_eq!(parsed.len(), 3, "expected 3 files, got {}", parsed.len());

    let languages: Vec<Language> = parsed.iter().map(|f| f.language).collect();
    assert!(languages.contains(&Language::Python));
    assert!(languages.contains(&Language::TypeScript));
    assert!(languages.contains(&Language::Rust));
}

#[test]
fn python_extracts_symbols() {
    let config = ParseConfig::new(fixtures_dir());
    let parsed = parse_repository(&config, None).unwrap();
    let py = parsed.iter().find(|f| f.language == Language::Python).unwrap();

    let names: Vec<&str> = py.symbols.iter().map(|s| s.name.as_str()).collect();

    // class FileReader
    assert!(names.contains(&"FileReader"), "missing FileReader, got: {names:?}");
    let fr = py.symbols.iter().find(|s| s.name == "FileReader").unwrap();
    assert_eq!(fr.kind, SymbolKind::Class);

    // def __init__
    assert!(names.contains(&"__init__"), "missing __init__, got: {names:?}");
    let init = py.symbols.iter().find(|s| s.name == "__init__").unwrap();
    assert_eq!(init.kind, SymbolKind::Function);

    // def read
    assert!(names.contains(&"read"), "missing read, got: {names:?}");

    // def process
    assert!(names.contains(&"process"), "missing process, got: {names:?}");
    let process = py.symbols.iter().find(|s| s.name == "process").unwrap();
    assert_eq!(process.kind, SymbolKind::Function);
}

#[test]
fn python_extracts_imports() {
    let config = ParseConfig::new(fixtures_dir());
    let parsed = parse_repository(&config, None).unwrap();
    let py = parsed.iter().find(|f| f.language == Language::Python).unwrap();

    assert!(!py.imports.is_empty(), "expected imports, got none");

    let import_modules: Vec<&str> = py.imports.iter().map(|i| i.from.as_str()).collect();
    assert!(
        import_modules.contains(&"os"),
        "missing 'os' import, got: {import_modules:?}"
    );
    assert!(
        import_modules.contains(&"pathlib"),
        "missing 'pathlib' import, got: {import_modules:?}"
    );
}

#[test]
fn typescript_extracts_symbols() {
    let config = ParseConfig::new(fixtures_dir());
    let parsed = parse_repository(&config, None).unwrap();
    let ts = parsed.iter().find(|f| f.language == Language::TypeScript).unwrap();

    let names: Vec<&str> = ts.symbols.iter().map(|s| s.name.as_str()).collect();

    // interface Reader
    assert!(names.contains(&"Reader"), "missing Reader, got: {names:?}");
    let reader = ts.symbols.iter().find(|s| s.name == "Reader").unwrap();
    assert_eq!(reader.kind, SymbolKind::Interface);

    // class FileReader
    assert!(names.contains(&"FileReader"), "missing FileReader, got: {names:?}");
    let fr = ts.symbols.iter().find(|s| s.name == "FileReader").unwrap();
    assert_eq!(fr.kind, SymbolKind::Class);

    // function process
    assert!(names.contains(&"process"), "missing process, got: {names:?}");
    let process = ts.symbols.iter().find(|s| s.name == "process").unwrap();
    assert_eq!(process.kind, SymbolKind::Function);
}

#[test]
fn typescript_extracts_heritage() {
    let config = ParseConfig::new(fixtures_dir());
    let parsed = parse_repository(&config, None).unwrap();
    let ts = parsed.iter().find(|f| f.language == Language::TypeScript).unwrap();

    // FileReader implements Reader
    assert!(
        !ts.heritage.is_empty(),
        "expected heritage (implements Reader), got none"
    );
    let parents: Vec<&str> = ts.heritage.iter().map(|h| h.parent.as_str()).collect();
    assert!(
        parents.contains(&"Reader"),
        "missing 'Reader' in heritage parents, got: {parents:?}"
    );
}

#[test]
fn rust_extracts_symbols() {
    let config = ParseConfig::new(fixtures_dir());
    let parsed = parse_repository(&config, None).unwrap();
    let rs = parsed.iter().find(|f| f.language == Language::Rust).unwrap();

    let names: Vec<&str> = rs.symbols.iter().map(|s| s.name.as_str()).collect();

    // struct Config
    assert!(names.contains(&"Config"), "missing Config, got: {names:?}");
    let config_sym = rs.symbols.iter().find(|s| s.name == "Config").unwrap();
    assert_eq!(config_sym.kind, SymbolKind::Struct);

    // fn load
    assert!(names.contains(&"load"), "missing load, got: {names:?}");

    // fn read
    assert!(names.contains(&"read"), "missing read, got: {names:?}");

    // fn process
    assert!(names.contains(&"process"), "missing process, got: {names:?}");
    let process = rs.symbols.iter().find(|s| s.name == "process").unwrap();
    assert_eq!(process.kind, SymbolKind::Function);
}

#[test]
fn sloc_counts_are_reasonable() {
    let config = ParseConfig::new(fixtures_dir());
    let parsed = parse_repository(&config, None).unwrap();

    for file in &parsed {
        assert!(file.sloc > 0, "{}: sloc should be > 0", file.path.display());
        // SLOC should be <= total line count of file
        let source = fs::read_to_string(&file.path).unwrap();
        let total_lines = source.lines().count();
        assert!(
            file.sloc <= total_lines,
            "{}: sloc ({}) > total lines ({})",
            file.path.display(),
            file.sloc,
            total_lines
        );
    }
}

#[test]
fn parse_repository_with_tempdir() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();

    fs::write(
        root.join("main.py"),
        "import os\n\nclass C:\n    pass\n\ndef foo():\n    return 1\n",
    )
    .unwrap();

    let config = ParseConfig::new(root.to_path_buf());
    let bus = EventBus::new(64);
    let parsed = parse_repository(&config, Some(&bus)).expect("parse_repository");

    assert!(!parsed.is_empty());
    let file = &parsed[0];
    assert_eq!(file.language, Language::Python);
    assert!(!file.symbols.is_empty());
    assert!(file.sloc > 0);
}

#[test]
fn each_parsed_file_has_correct_language_and_nonempty_symbols() {
    let config = ParseConfig::new(fixtures_dir());
    let parsed = parse_repository(&config, None).unwrap();

    for file in &parsed {
        assert!(
            !file.symbols.is_empty(),
            "{}: expected symbols, got none",
            file.path.display()
        );
        assert!(file.sloc > 0, "{}: expected non-zero sloc", file.path.display());

        // Verify language matches extension
        let ext = file.path.extension().unwrap().to_str().unwrap();
        let expected = Language::from_extension(ext).unwrap();
        assert_eq!(file.language, expected);
    }
}

#[test]
fn incremental_parse_skips_unchanged_files() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();

    // Create two files
    fs::write(
        root.join("unchanged.py"),
        "class Stable:\n    pass\n",
    )
    .unwrap();
    fs::write(
        root.join("changed.py"),
        "def old():\n    return 1\n",
    )
    .unwrap();

    // First parse — full
    let config = ParseConfig::new(root.to_path_buf());
    let first = parse_repository(&config, None).expect("first parse");
    assert_eq!(first.len(), 2);

    // Record current mtimes as the "existing" DB state
    let now = SystemTime::now();
    let mut existing = HashMap::new();
    existing.insert(
        "unchanged.py".to_string(),
        ExistingFile {
            id: FileId(1),
            last_modified: Some(now),
        },
    );
    existing.insert(
        "changed.py".to_string(),
        ExistingFile {
            id: FileId(2),
            // Use epoch so any file looks newer
            last_modified: Some(SystemTime::UNIX_EPOCH),
        },
    );

    // Incremental parse: unchanged.py has a future mtime so it's "unchanged",
    // changed.py has epoch mtime so it's "changed"
    let result = parse_repository_incremental(&config, &existing, None)
        .expect("incremental parse");

    // unchanged.py should be in unchanged_ids
    assert!(
        result.unchanged_ids.contains(&FileId(1)),
        "unchanged.py should be skipped, unchanged_ids: {:?}",
        result.unchanged_ids,
    );

    // changed.py should be re-parsed
    assert!(
        !result.changed_files.is_empty(),
        "changed.py should be re-parsed"
    );
    assert!(
        result.changed_files.iter().any(|f| f.path.ends_with("changed.py")),
        "changed_files should contain changed.py"
    );
}

#[test]
fn incremental_parse_handles_new_files() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();

    fs::write(root.join("existing.py"), "def f(): pass\n").unwrap();
    fs::write(root.join("brand_new.py"), "def g(): pass\n").unwrap();

    let config = ParseConfig::new(root.to_path_buf());

    // Only existing.py is known
    let mut existing = HashMap::new();
    existing.insert(
        "existing.py".to_string(),
        ExistingFile {
            id: FileId(1),
            last_modified: Some(SystemTime::now()),
        },
    );

    let result = parse_repository_incremental(&config, &existing, None)
        .expect("incremental parse");

    // brand_new.py should be parsed since it's not in existing
    assert!(
        result.changed_files.iter().any(|f| f.path.ends_with("brand_new.py")),
        "new files should be parsed"
    );

    // existing.py should be unchanged
    assert!(
        result.unchanged_ids.contains(&FileId(1)),
        "existing unchanged file should be in unchanged_ids"
    );
}
