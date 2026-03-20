/** Pure-TS layout algorithms. No browser APIs, no external deps. */

export interface LayoutNode {
  id: string;
  label: string;
  width: number;
  height: number;
  x: number;
  y: number;
  data: Record<string, unknown>;
  children?: LayoutNode[];
}

export interface LayoutEdge {
  id: string;
  from: string;
  to: string;
  label?: string;
  kind?: string;
}

// ── Tree Layout (right-flowing) ──

interface TreeInput {
  id: string;
  label: string;
  data: Record<string, unknown>;
  children: TreeInput[];
}

const NODE_H = 36;
const NODE_PAD_Y = 6;
const LAYER_GAP = 180;

function measureLabel(label: string, minW = 100): number {
  return Math.max(minW, label.length * 7.5 + 32);
}

function layoutTreeRecursive(
  node: TreeInput,
  depth: number,
  yOffset: number,
  nodes: LayoutNode[],
  edges: LayoutEdge[],
): number {
  const w = measureLabel(node.label, depth === 0 ? 120 : 140);
  const x = depth * LAYER_GAP;

  if (node.children.length === 0) {
    const ln: LayoutNode = { id: node.id, label: node.label, width: w, height: NODE_H, x, y: yOffset, data: node.data };
    nodes.push(ln);
    return yOffset + NODE_H + NODE_PAD_Y;
  }

  const childStartY = yOffset;
  let currentY = yOffset;
  for (const child of node.children) {
    edges.push({ id: `e-${node.id}-${child.id}`, from: node.id, to: child.id });
    currentY = layoutTreeRecursive(child, depth + 1, currentY, nodes, edges);
  }

  // Center parent vertically among its children
  const childEndY = currentY - NODE_PAD_Y;
  const centerY = (childStartY + childEndY - NODE_H) / 2;

  const ln: LayoutNode = { id: node.id, label: node.label, width: w, height: NODE_H, x, y: centerY, data: node.data };
  nodes.push(ln);

  return currentY;
}

export function layoutTree(root: TreeInput): { nodes: LayoutNode[]; edges: LayoutEdge[]; width: number; height: number } {
  const nodes: LayoutNode[] = [];
  const edges: LayoutEdge[] = [];
  const totalH = layoutTreeRecursive(root, 0, 20, nodes, edges);
  const maxX = Math.max(...nodes.map(n => n.x + n.width));
  return { nodes, edges, width: maxX + 40, height: totalH + 20 };
}

// ── Layered Layout (top-down, for symbol graphs) ──

interface LayeredInput {
  nodes: { id: string; label: string; data: Record<string, unknown> }[];
  edges: { from: string; to: string; label?: string; kind?: string }[];
}

const LAYER_V_GAP = 70;
const NODE_H_GAP = 24;

export function layoutLayered(input: LayeredInput): { nodes: LayoutNode[]; edges: LayoutEdge[]; width: number; height: number } {
  const { nodes: inputNodes, edges: inputEdges } = input;

  // If no edges, use grid layout
  if (inputEdges.length === 0) {
    return layoutGrid(inputNodes);
  }

  // Build adjacency
  const incoming = new Map<string, Set<string>>();
  const outgoing = new Map<string, Set<string>>();
  for (const n of inputNodes) {
    incoming.set(n.id, new Set());
    outgoing.set(n.id, new Set());
  }
  for (const e of inputEdges) {
    outgoing.get(e.from)?.add(e.to);
    incoming.get(e.to)?.add(e.from);
  }

  // Topological layering (Kahn's algorithm)
  const layers: string[][] = [];
  const assigned = new Set<string>();
  const remaining = new Set(inputNodes.map(n => n.id));

  while (remaining.size > 0) {
    const layer: string[] = [];
    for (const id of remaining) {
      const inc = incoming.get(id)!;
      const unassignedIncoming = [...inc].filter(i => !assigned.has(i));
      if (unassignedIncoming.length === 0) {
        layer.push(id);
      }
    }
    if (layer.length === 0) {
      layer.push(...remaining);
    }
    for (const id of layer) {
      assigned.add(id);
      remaining.delete(id);
    }
    layers.push(layer);
  }

  // Position nodes
  const nodeMap = new Map(inputNodes.map(n => [n.id, n]));
  const layoutNodes: LayoutNode[] = [];
  let maxWidth = 0;

  for (let li = 0; li < layers.length; li++) {
    const layer = layers[li];
    const y = 20 + li * LAYER_V_GAP;
    let x = 20;
    for (const id of layer) {
      const n = nodeMap.get(id)!;
      const w = measureLabel(n.label, 130);
      layoutNodes.push({ id, label: n.label, width: w, height: NODE_H, x, y, data: n.data });
      x += w + NODE_H_GAP;
    }
    maxWidth = Math.max(maxWidth, x);
  }

  const maxY = Math.max(...layoutNodes.map(n => n.y + n.height));
  const layoutEdges: LayoutEdge[] = inputEdges.map((e, i) => ({
    id: `e-${i}`,
    from: e.from,
    to: e.to,
    label: e.label,
    kind: e.kind,
  }));

  return { nodes: layoutNodes, edges: layoutEdges, width: maxWidth + 20, height: maxY + 40 };
}

// ── Grid Layout (for nodes without edges, e.g. community overview) ──

function layoutGrid(inputNodes: { id: string; label: string; data: Record<string, unknown> }[]): { nodes: LayoutNode[]; edges: LayoutEdge[]; width: number; height: number } {
  const CARD_W = 200;
  const CARD_H = 60;
  const GAP_X = 24;
  const GAP_Y = 20;
  const COLS = Math.max(1, Math.min(5, Math.ceil(Math.sqrt(inputNodes.length))));

  const layoutNodes: LayoutNode[] = [];
  for (let i = 0; i < inputNodes.length; i++) {
    const col = i % COLS;
    const row = Math.floor(i / COLS);
    const n = inputNodes[i];
    layoutNodes.push({
      id: n.id,
      label: n.label,
      width: CARD_W,
      height: CARD_H,
      x: 20 + col * (CARD_W + GAP_X),
      y: 20 + row * (CARD_H + GAP_Y),
      data: n.data,
    });
  }

  const maxX = Math.max(...layoutNodes.map(n => n.x + n.width), 0);
  const maxY = Math.max(...layoutNodes.map(n => n.y + n.height), 0);
  return { nodes: layoutNodes, edges: [], width: maxX + 20, height: maxY + 20 };
}

// ── Fit-to-view ──

export function computeFitToView(
  nodes: LayoutNode[],
  viewportW: number,
  viewportH: number,
  padding = 40,
): { tx: number; ty: number; scale: number } {
  if (nodes.length === 0) return { tx: padding, ty: padding, scale: 1 };
  let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
  for (const n of nodes) {
    minX = Math.min(minX, n.x);
    minY = Math.min(minY, n.y);
    maxX = Math.max(maxX, n.x + n.width);
    maxY = Math.max(maxY, n.y + n.height);
  }
  const bboxW = maxX - minX;
  const bboxH = maxY - minY;
  const scale = Math.min((viewportW - padding * 2) / bboxW, (viewportH - padding * 2) / bboxH, 1.5);
  const tx = (viewportW - bboxW * scale) / 2 - minX * scale;
  const ty = (viewportH - bboxH * scale) / 2 - minY * scale;
  return { tx, ty, scale };
}
