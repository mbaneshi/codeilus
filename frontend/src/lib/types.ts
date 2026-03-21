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

export interface CommunityGraphNode {
  id: number;
  label: string;
  member_count: number;
  cohesion: number;
}

export interface CommunityGraphEdge {
  source_id: number;
  target_id: number;
  weight: number;
}

export interface CommunityGraphResponse {
  nodes: CommunityGraphNode[];
  edges: CommunityGraphEdge[];
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

export interface NarrativeResponse {
  id: number;
  kind: string;
  target_id: number | null;
  content: string;
  generated_at: string;
  is_placeholder?: boolean;
}

export interface ChapterSection {
  id: number;
  title: string;
  kind: string;
  content: string;
}

export interface SourceLine {
  number: number;
  content: string;
}

export interface SourceResponse {
  path: string;
  language: string | null;
  lines: SourceLine[];
  total_lines: number;
}

export interface Chapter {
  id: number;
  order_index: number;
  title: string;
  description: string;
  community_id: number | null;
  difficulty: string;
  sections: ChapterSection[];
  narrative: string | null;
}

export interface Progress {
  chapter_id: number;
  section_id: number;
  completed: boolean;
  completed_at: string | null;
}

export interface QuizQuestion {
  id: number;
  chapter_id: number;
  question: string;
  options: string[];
  kind: string;
}

export interface QuizAnswerResult {
  correct: boolean;
  xp_earned: number;
  explanation?: string;
}

export interface Badge {
  id: number;
  name: string;
  description: string;
  icon: string;
  earned_at: string;
}

export interface LearnerStats {
  total_xp: number;
  streak_days: number;
  last_active: string;
  chapters_completed: number;
  badges: Badge[];
}

// ── Schematic types ──

export interface SchematicNode {
  id: string;
  type: 'directory' | 'file' | 'symbol' | 'community';
  label: string;
  parent_id: string | null;
  file_id?: number;
  symbol_id?: number;
  language?: string;
  sloc?: number;
  kind?: string;
  signature?: string;
  community_id?: number;
  community_label?: string;
  community_color?: string;
  chapter_id?: number;
  chapter_title?: string;
  difficulty?: string;
  progress?: { completed: number; total: number };
  has_children: boolean;
  child_count?: number;
  symbol_count?: number;
}

export interface SchematicEdge {
  id: string;
  source: string;
  target: string;
  type: string;
  confidence?: number;
}

export interface SchematicCommunity {
  id: number;
  label: string;
  color: string;
  cohesion: number;
  member_count: number;
  chapter_id?: number;
  chapter_title?: string;
  difficulty?: string;
  progress?: { completed: number; total: number };
}

export interface SchematicResponse {
  nodes: SchematicNode[];
  edges: SchematicEdge[];
  communities: SchematicCommunity[];
  meta: { total_files: number; total_symbols: number; total_communities: number; depth_returned: number };
}

export interface SchematicDetail {
  node_id: string;
  narrative?: string;
  narrative_kind?: string;
  source?: { path: string; language?: string; lines: { number: number; content: string }[]; total_lines: number };
  callers: { id: string; name: string; kind: string; file_path: string }[];
  callees: { id: string; name: string; kind: string; file_path: string }[];
  chapter?: { id: number; title: string; difficulty: string; progress: { completed: number; total: number } };
}

export interface Annotation {
  id: number;
  target_type: 'node' | 'edge';
  target_id: number;
  content: string;
  flagged: boolean;
  created_at: string;
  updated_at: string;
}
