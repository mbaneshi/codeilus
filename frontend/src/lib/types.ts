export interface CodeilusEvent {
  type: string;
  data: Record<string, unknown>;
}

export interface FileRow {
  id: number;
  path: string;
  language: string | null;
  sloc: number;
  last_modified: string | null;
}

export interface SymbolRow {
  id: number;
  file_id: number;
  name: string;
  kind: string;
  start_line: number;
  end_line: number;
  signature: string | null;
}

export interface GraphNode {
  id: number;
  name: string;
  kind: string;
  file_id: number;
  community_id: number | null;
}

export interface GraphEdge {
  source_id: number;
  target_id: number;
  kind: string;
  confidence: number;
}

export interface GraphResponse {
  nodes: GraphNode[];
  edges: GraphEdge[];
}

export interface Community {
  id: number;
  label: string;
  cohesion: number;
  member_count: number;
  members: number[];
}

export interface ProcessStep {
  order: number;
  symbol_id: number;
  symbol_name: string;
  description: string;
}

export interface ProcessFlow {
  id: number;
  name: string;
  entry_symbol_id: number;
  steps: ProcessStep[];
}
