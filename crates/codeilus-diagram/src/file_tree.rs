//! ASCII file tree generation in 4 styles.

use crate::types::TreeStyle;
use std::collections::BTreeMap;

/// A node in the file tree.
enum TreeNode {
    Dir(BTreeMap<String, TreeNode>),
    File,
}

/// Generate an ASCII file tree from sorted file paths.
pub fn generate(files: &[String], style: TreeStyle) -> String {
    let root = build_tree(files);
    let mut output = String::new();
    render_node(&root, &mut output, "", "", style, true);
    output
}

/// Build a tree structure from flat file paths.
fn build_tree(files: &[String]) -> BTreeMap<String, TreeNode> {
    let mut root: BTreeMap<String, TreeNode> = BTreeMap::new();

    for path in files {
        let parts: Vec<&str> = path.split('/').collect();
        insert_path(&mut root, &parts);
    }

    root
}

fn insert_path(node: &mut BTreeMap<String, TreeNode>, parts: &[&str]) {
    if parts.is_empty() {
        return;
    }

    if parts.len() == 1 {
        node.entry(parts[0].to_string())
            .or_insert(TreeNode::File);
    } else {
        let dir = node
            .entry(parts[0].to_string())
            .or_insert_with(|| TreeNode::Dir(BTreeMap::new()));
        if let TreeNode::Dir(ref mut children) = dir {
            insert_path(children, &parts[1..]);
        }
    }
}

fn render_node(
    children: &BTreeMap<String, TreeNode>,
    output: &mut String,
    prefix: &str,
    _connector: &str,
    style: TreeStyle,
    is_root: bool,
) {
    // Sort: directories first, then files, alphabetical within each
    let mut dirs: Vec<(&String, &TreeNode)> = Vec::new();
    let mut files: Vec<(&String, &TreeNode)> = Vec::new();

    for (name, node) in children {
        match node {
            TreeNode::Dir(_) => dirs.push((name, node)),
            TreeNode::File => files.push((name, node)),
        }
    }

    let mut all: Vec<(&String, &TreeNode)> = Vec::new();
    all.extend(dirs.iter());
    all.extend(files.iter());

    let total = all.len();
    for (i, (name, node)) in all.iter().enumerate() {
        let is_last = i == total - 1;

        match style {
            TreeStyle::Default | TreeStyle::Extended => {
                let conn = if is_root {
                    ""
                } else if is_last {
                    "└── "
                } else {
                    "├── "
                };
                let child_prefix = if is_root {
                    prefix.to_string()
                } else if is_last {
                    format!("{}    ", prefix)
                } else {
                    format!("{}│   ", prefix)
                };

                match node {
                    TreeNode::Dir(children) => {
                        output.push_str(&format!("{}{}{}/\n", prefix, conn, name));
                        render_node(children, output, &child_prefix, conn, style, false);
                    }
                    TreeNode::File => {
                        output.push_str(&format!("{}{}{}\n", prefix, conn, name));
                    }
                }
            }
            TreeStyle::Compact => {
                let conn = if is_root { "" } else { "| " };
                let child_prefix = if is_root {
                    prefix.to_string()
                } else {
                    format!("{}  ", prefix)
                };

                match node {
                    TreeNode::Dir(children) => {
                        output.push_str(&format!("{}{}{}/\n", prefix, conn, name));
                        render_node(children, output, &child_prefix, conn, style, false);
                    }
                    TreeNode::File => {
                        output.push_str(&format!("{}{}{}\n", prefix, conn, name));
                    }
                }
            }
            TreeStyle::Minimal => {
                let child_prefix = if is_root {
                    prefix.to_string()
                } else {
                    format!("{}  ", prefix)
                };

                match node {
                    TreeNode::Dir(children) => {
                        output.push_str(&format!("{}{}/\n", prefix, name));
                        render_node(children, output, &child_prefix, "", style, false);
                    }
                    TreeNode::File => {
                        output.push_str(&format!("{}{}\n", prefix, name));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_tree_default_style() {
        let files = vec![
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
            "src/parser/mod.rs".to_string(),
            "src/parser/python.rs".to_string(),
            "src/utils.rs".to_string(),
        ];
        let result = generate(&files, TreeStyle::Default);
        assert!(result.contains("src/"));
        assert!(result.contains("├──") || result.contains("└──"));
        assert!(result.contains("main.rs"));
        assert!(result.contains("parser/"));
    }

    #[test]
    fn file_tree_compact_style() {
        let files = vec![
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
        ];
        let result = generate(&files, TreeStyle::Compact);
        assert!(result.contains("src/"));
        assert!(result.contains("| "));
        assert!(result.contains("main.rs"));
    }

    #[test]
    fn file_tree_dirs_first() {
        let files = vec![
            "src/main.rs".to_string(),
            "src/parser/mod.rs".to_string(),
            "README.md".to_string(),
        ];
        let result = generate(&files, TreeStyle::Default);
        let lines: Vec<&str> = result.lines().collect();

        // "src/" directory should appear before "README.md" file
        let src_pos = lines.iter().position(|l| l.contains("src/")).unwrap();
        let readme_pos = lines.iter().position(|l| l.contains("README.md")).unwrap();
        assert!(
            src_pos < readme_pos,
            "Directories should be sorted before files"
        );
    }

    #[test]
    fn file_tree_nested_dirs() {
        let files = vec![
            "a/b/c/file.rs".to_string(),
            "a/b/other.rs".to_string(),
            "a/top.rs".to_string(),
        ];
        let result = generate(&files, TreeStyle::Default);
        assert!(result.contains("a/"));
        assert!(result.contains("b/"));
        assert!(result.contains("c/"));
        assert!(result.contains("file.rs"));

        // Check proper nesting — deeper items appear on later lines
        // and "c/" appears after "b/" which appears after "a/"
        let lines: Vec<&str> = result.lines().collect();
        let a_pos = lines.iter().position(|l| l.ends_with("a/")).unwrap();
        let b_pos = lines.iter().position(|l| l.ends_with("b/")).unwrap();
        let c_pos = lines.iter().position(|l| l.ends_with("c/")).unwrap();
        assert!(b_pos > a_pos, "b/ should appear after a/");
        assert!(c_pos > b_pos, "c/ should appear after b/");

        // Verify the c/ line has more leading chars than a/ (indicating nesting)
        let a_line = lines[a_pos];
        let c_line = lines[c_pos];
        assert!(
            c_line.chars().count() > a_line.chars().count(),
            "c/ line should be longer than a/ line due to indentation"
        );
    }

    #[test]
    fn file_tree_minimal_style() {
        let files = vec![
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
        ];
        let result = generate(&files, TreeStyle::Minimal);
        assert!(result.contains("src/"));
        assert!(result.contains("main.rs"));
        // Minimal should not have box-drawing characters
        assert!(!result.contains("├"));
        assert!(!result.contains("└"));
        assert!(!result.contains("│"));
    }
}
