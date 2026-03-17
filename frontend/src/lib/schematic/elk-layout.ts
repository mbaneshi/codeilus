import ELK from 'elkjs/lib/elk.bundled.js';
import type { SchematicNode, SchematicEdge, LayoutResult } from './types';

const elk = new ELK();

interface LayoutOptions {
  algorithm?: 'layered' | 'mrtree' | 'force';
  direction?: 'RIGHT' | 'DOWN' | 'LEFT' | 'UP';
  nodeSpacing?: number;
  layerSpacing?: number;
}

interface ElkNode {
  id: string;
  width: number;
  height: number;
  children?: ElkNode[];
  x?: number;
  y?: number;
}

interface ElkEdge {
  id: string;
  sources: string[];
  targets: string[];
}

function toElkNode(node: SchematicNode): ElkNode {
  const elkNode: ElkNode = {
    id: node.id,
    width: node.width,
    height: node.height,
  };
  if (node.children && node.children.length > 0) {
    elkNode.children = node.children.map(toElkNode);
  }
  return elkNode;
}

function flattenPositions(
  elkNode: { id?: string; x?: number; y?: number; width?: number; height?: number; children?: unknown[] },
  offsetX: number,
  offsetY: number,
  result: Map<string, { x: number; y: number; width: number; height: number }>,
) {
  const children = elkNode.children as typeof elkNode[] | undefined;
  if (children) {
    for (const child of children) {
      const cx = offsetX + (child.x ?? 0);
      const cy = offsetY + (child.y ?? 0);
      result.set(child.id!, { x: cx, y: cy, width: child.width ?? 0, height: child.height ?? 0 });
      flattenPositions(child, cx, cy, result);
    }
  }
}

export async function computeLayout(
  nodes: SchematicNode[],
  edges: SchematicEdge[],
  options: LayoutOptions = {},
): Promise<LayoutResult> {
  const {
    algorithm = 'layered',
    direction = 'RIGHT',
    nodeSpacing = 25,
    layerSpacing = 50,
  } = options;

  const elkGraph = {
    id: 'root',
    layoutOptions: {
      'elk.algorithm': algorithm === 'mrtree' ? 'org.eclipse.elk.mrtree' :
                        algorithm === 'force' ? 'org.eclipse.elk.force' :
                        'org.eclipse.elk.layered',
      'elk.direction': direction,
      'elk.spacing.nodeNode': String(nodeSpacing),
      'elk.layered.spacing.edgeNodeBetweenLayers': String(layerSpacing),
      'elk.hierarchyHandling': 'INCLUDE_CHILDREN',
      'elk.layered.nodePlacement.strategy': 'NETWORK_SIMPLEX',
      'elk.padding': '[top=20,left=20,bottom=20,right=20]',
    },
    children: nodes.map(toElkNode),
    edges: edges.map((e): ElkEdge => ({
      id: e.id,
      sources: [e.source],
      targets: [e.target],
    })),
  };

  const laid = await elk.layout(elkGraph);

  const nodePositions = new Map<string, { x: number; y: number; width: number; height: number }>();
  flattenPositions(laid, 0, 0, nodePositions);
  // Also add top-level children
  if (laid.children) {
    for (const child of laid.children) {
      if (!nodePositions.has(child.id)) {
        nodePositions.set(child.id, {
          x: child.x ?? 0, y: child.y ?? 0,
          width: child.width ?? 0, height: child.height ?? 0,
        });
      }
    }
  }

  const edgePositions = new Map<string, { points: { x: number; y: number }[] }>();
  if (laid.edges) {
    for (const edge of laid.edges) {
      const sections = (edge as unknown as { sections?: { startPoint: { x: number; y: number }; endPoint: { x: number; y: number }; bendPoints?: { x: number; y: number }[] }[] }).sections;
      if (sections && sections.length > 0) {
        const pts: { x: number; y: number }[] = [];
        for (const sec of sections) {
          pts.push(sec.startPoint);
          if (sec.bendPoints) pts.push(...sec.bendPoints);
          pts.push(sec.endPoint);
        }
        edgePositions.set(edge.id, { points: pts });
      } else {
        // Fallback: connect source center to target center
        const src = nodePositions.get((edge as unknown as { sources: string[] }).sources[0]);
        const tgt = nodePositions.get((edge as unknown as { targets: string[] }).targets[0]);
        if (src && tgt) {
          edgePositions.set(edge.id, {
            points: [
              { x: src.x + src.width, y: src.y + src.height / 2 },
              { x: tgt.x, y: tgt.y + tgt.height / 2 },
            ],
          });
        }
      }
    }
  }

  return {
    nodes: nodePositions,
    edges: edgePositions,
    width: laid.width ?? 800,
    height: laid.height ?? 600,
  };
}
