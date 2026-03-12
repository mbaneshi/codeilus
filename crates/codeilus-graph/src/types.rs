use codeilus_core::ids::{CommunityId, FileId, SymbolId};
use codeilus_core::types::{Confidence, EdgeKind};
use petgraph::graph::{DiGraph, NodeIndex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The in-memory knowledge graph.
pub struct KnowledgeGraph {
    pub graph: DiGraph<GraphNode, GraphEdge>,
    pub node_index: HashMap<SymbolId, NodeIndex>,
    pub communities: Vec<Community>,
    pub processes: Vec<Process>,
    pub entry_points: Vec<EntryPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub symbol_id: SymbolId,
    pub file_id: FileId,
    pub name: String,
    pub kind: String,
    pub community_id: Option<CommunityId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub kind: EdgeKind,
    pub confidence: Confidence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Community {
    pub id: CommunityId,
    pub label: String,
    pub members: Vec<SymbolId>,
    pub cohesion: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Process {
    pub name: String,
    pub entry_symbol_id: SymbolId,
    pub steps: Vec<ProcessStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessStep {
    pub order: usize,
    pub symbol_id: SymbolId,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryPoint {
    pub symbol_id: SymbolId,
    pub score: f64,
    pub reason: String,
}
