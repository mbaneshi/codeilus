export interface SchematicNode {
  id: string;
  label: string;
  width: number;
  height: number;
  x?: number;
  y?: number;
  children?: SchematicNode[];
  metadata: Record<string, unknown>;
}

export interface SchematicEdge {
  id: string;
  source: string;
  target: string;
  label?: string;
  kind?: string;
  points?: { x: number; y: number }[];
}

export interface LayoutResult {
  nodes: Map<string, { x: number; y: number; width: number; height: number }>;
  edges: Map<string, { points: { x: number; y: number }[] }>;
  width: number;
  height: number;
}
