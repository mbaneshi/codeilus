use codeilus_core::ids::{FileId, SymbolId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "info"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternKind {
    GodClass,
    LongMethod,
    CircularDependency,
    SecurityHotspot,
    TestGap,
}

impl std::fmt::Display for PatternKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GodClass => write!(f, "god_class"),
            Self::LongMethod => write!(f, "long_method"),
            Self::CircularDependency => write!(f, "circular_dependency"),
            Self::SecurityHotspot => write!(f, "security_hotspot"),
            Self::TestGap => write!(f, "test_gap"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternFinding {
    pub kind: PatternKind,
    pub severity: Severity,
    pub file_id: Option<FileId>,
    pub symbol_id: Option<SymbolId>,
    pub file_path: String,
    pub line: Option<usize>,
    pub message: String,
    pub suggestion: String,
}
