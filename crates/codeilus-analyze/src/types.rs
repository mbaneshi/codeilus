use codeilus_core::ids::{FileId, SymbolId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
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

impl PatternKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GodClass => "god_class",
            Self::LongMethod => "long_method",
            Self::CircularDependency => "circular_dependency",
            Self::SecurityHotspot => "security_hotspot",
            Self::TestGap => "test_gap",
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
