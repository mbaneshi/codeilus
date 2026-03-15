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

export interface NarrativeResponse {
  id: number;
  kind: string;
  target_id: number | null;
  content: string;
  generated_at: string;
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

export interface Annotation {
  id: number;
  target_type: 'node' | 'edge';
  target_id: number;
  content: string;
  flagged: boolean;
  created_at: string;
  updated_at: string;
}
