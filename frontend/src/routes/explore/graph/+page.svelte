<script lang="ts">
  import { onMount } from 'svelte';
  import { fetchGraph } from '$lib/api';
  import type { GraphNode, GraphEdge } from '$lib/types';

  const COLORS = ['#6366f1', '#ec4899', '#14b8a6', '#f59e0b', '#ef4444', '#8b5cf6', '#06b6d4', '#84cc16'];

  interface SimNode {
    node: GraphNode;
    x: number;
    y: number;
    vx: number;
    vy: number;
    radius: number;
    color: string;
    edgeCount: number;
  }

  let loading = $state(true);
  let nodes = $state<GraphNode[]>([]);
  let edges = $state<GraphEdge[]>([]);
  let simNodes = $state<SimNode[]>([]);
  let hoveredNode = $state<SimNode | null>(null);
  let selectedNode = $state<SimNode | null>(null);
  let mouseX = $state(0);
  let mouseY = $state(0);
  let svgEl: SVGSVGElement | undefined = $state();
  let animFrame = 0;
  let width = $state(800);
  let height = $state(600);
  let communityCount = $derived(new Set(nodes.map((n) => n.community_id).filter((c) => c !== null)).size);

  function connectedNodeIds(nodeId: number): Set<number> {
    const ids = new Set<number>();
    for (const e of edges) {
      if (e.source_id === nodeId) ids.add(e.target_id);
      if (e.target_id === nodeId) ids.add(e.source_id);
    }
    return ids;
  }

  function initSimulation() {
    const edgeCounts = new Map<number, number>();
    for (const e of edges) {
      edgeCounts.set(e.source_id, (edgeCounts.get(e.source_id) ?? 0) + 1);
      edgeCounts.set(e.target_id, (edgeCounts.get(e.target_id) ?? 0) + 1);
    }

    simNodes = nodes.map((n) => {
      const ec = edgeCounts.get(n.id) ?? 0;
      return {
        node: n,
        x: width / 2 + (Math.random() - 0.5) * width * 0.6,
        y: height / 2 + (Math.random() - 0.5) * height * 0.6,
        vx: 0,
        vy: 0,
        radius: Math.min(16, Math.max(4, 4 + ec * 1.5)),
        color: COLORS[(n.community_id ?? 0) % COLORS.length],
        edgeCount: ec,
      };
    });
  }

  function tick() {
    const nodeMap = new Map<number, SimNode>();
    for (const sn of simNodes) nodeMap.set(sn.node.id, sn);

    const cx = width / 2;
    const cy = height / 2;
    const dt = 0.3;

    for (let i = 0; i < simNodes.length; i++) {
      const a = simNodes[i];
      // Center gravity
      a.vx += (cx - a.x) * 0.0005;
      a.vy += (cy - a.y) * 0.0005;

      // Repulsion
      for (let j = i + 1; j < simNodes.length; j++) {
        const b = simNodes[j];
        let dx = a.x - b.x;
        let dy = a.y - b.y;
        const distSq = dx * dx + dy * dy + 1;
        const force = 800 / distSq;
        const fx = dx * force;
        const fy = dy * force;
        a.vx += fx;
        a.vy += fy;
        b.vx -= fx;
        b.vy -= fy;
      }
    }

    // Edge springs
    for (const e of edges) {
      const a = nodeMap.get(e.source_id);
      const b = nodeMap.get(e.target_id);
      if (!a || !b) continue;
      const dx = b.x - a.x;
      const dy = b.y - a.y;
      const dist = Math.sqrt(dx * dx + dy * dy) + 0.1;
      const target = 60;
      const force = (dist - target) * 0.003;
      const fx = (dx / dist) * force;
      const fy = (dy / dist) * force;
      a.vx += fx;
      a.vy += fy;
      b.vx -= fx;
      b.vy -= fy;
    }

    // Apply velocity with damping
    for (const sn of simNodes) {
      sn.vx *= 0.85;
      sn.vy *= 0.85;
      sn.x += sn.vx * dt;
      sn.y += sn.vy * dt;
      // Clamp to bounds
      sn.x = Math.max(sn.radius, Math.min(width - sn.radius, sn.x));
      sn.y = Math.max(sn.radius, Math.min(height - sn.radius, sn.y));
    }

    simNodes = [...simNodes]; // trigger reactivity
    animFrame = requestAnimationFrame(tick);
  }

  function handleMouseMove(e: MouseEvent) {
    if (!svgEl) return;
    const rect = svgEl.getBoundingClientRect();
    mouseX = e.clientX - rect.left;
    mouseY = e.clientY - rect.top;
  }

  function findNodeAt(x: number, y: number): SimNode | null {
    for (const sn of simNodes) {
      const dx = sn.x - x;
      const dy = sn.y - y;
      if (dx * dx + dy * dy < (sn.radius + 4) * (sn.radius + 4)) return sn;
    }
    return null;
  }

  function handleSvgMouseMove(e: MouseEvent) {
    handleMouseMove(e);
    hoveredNode = findNodeAt(mouseX, mouseY);
  }

  function handleSvgClick() {
    const clicked = findNodeAt(mouseX, mouseY);
    selectedNode = clicked === selectedNode ? null : clicked;
  }

  let selectedConnected = $derived(selectedNode ? connectedNodeIds(selectedNode.node.id) : new Set<number>());

  function nodeOpacity(sn: SimNode): number {
    if (!selectedNode) return 1;
    if (sn.node.id === selectedNode.node.id) return 1;
    if (selectedConnected.has(sn.node.id)) return 1;
    return 0.15;
  }

  function edgeOpacity(e: GraphEdge): number {
    if (!selectedNode) return 0.3;
    if (e.source_id === selectedNode.node.id || e.target_id === selectedNode.node.id) return 0.8;
    return 0.05;
  }

  onMount(() => {
    fetchGraph().then((graph) => {
      nodes = graph.nodes;
      edges = graph.edges;
      loading = false;

      if (nodes.length > 0) {
        if (svgEl) {
          const rect = svgEl.getBoundingClientRect();
          width = rect.width || 800;
          height = rect.height || 600;
        }
        initSimulation();
        animFrame = requestAnimationFrame(tick);
      }
    });

    return () => {
      if (animFrame) cancelAnimationFrame(animFrame);
    };
  });
</script>

<div class="flex h-full">
  <div class="flex-1 flex flex-col">
    <div class="p-4 border-b border-gray-800 flex items-center gap-4">
      <h1 class="text-2xl font-bold">Knowledge Graph</h1>
      {#if !loading && nodes.length > 0}
        <span class="text-sm text-gray-400">{nodes.length} nodes</span>
        <span class="text-sm text-gray-400">{edges.length} edges</span>
        <span class="text-sm text-gray-400">{communityCount} communities</span>
      {/if}
    </div>

    {#if loading}
      <div class="flex-1 flex items-center justify-center">
        <p class="text-gray-400 animate-pulse">Loading...</p>
      </div>
    {:else if nodes.length === 0}
      <div class="flex-1 flex items-center justify-center">
        <div class="text-center">
          <p class="text-gray-400 text-lg mb-2">No graph data</p>
          <p class="text-gray-500">Run <code class="text-indigo-400 font-mono">codeilus analyze ./repo</code> first</p>
        </div>
      </div>
    {:else}
      <div class="flex-1 relative">
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <svg
          bind:this={svgEl}
          class="w-full h-full bg-gray-950"
          onmousemove={handleSvgMouseMove}
          onclick={handleSvgClick}
        >
          {#each edges as edge}
            {@const src = simNodes.find((n) => n.node.id === edge.source_id)}
            {@const tgt = simNodes.find((n) => n.node.id === edge.target_id)}
            {#if src && tgt}
              <line
                x1={src.x} y1={src.y}
                x2={tgt.x} y2={tgt.y}
                stroke="#4b5563"
                stroke-width="1"
                opacity={edgeOpacity(edge)}
              />
            {/if}
          {/each}
          {#each simNodes as sn}
            <circle
              cx={sn.x} cy={sn.y} r={sn.radius}
              fill={sn.color}
              opacity={nodeOpacity(sn)}
              class="cursor-pointer"
            />
          {/each}
        </svg>
        {#if hoveredNode}
          <div
            class="absolute pointer-events-none bg-gray-900 border border-gray-700 rounded px-2 py-1 text-sm text-gray-100 shadow-lg"
            style="left: {mouseX + 12}px; top: {mouseY - 8}px"
          >
            {hoveredNode.node.name}
          </div>
        {/if}
      </div>
    {/if}
  </div>

  {#if selectedNode}
    <div class="w-72 border-l border-gray-800 bg-gray-900 p-4 overflow-auto">
      <h2 class="text-lg font-semibold mb-2">{selectedNode.node.name}</h2>
      <div class="space-y-2 text-sm">
        <div><span class="text-gray-500">Kind:</span> <span class="text-gray-300">{selectedNode.node.kind}</span></div>
        <div><span class="text-gray-500">File ID:</span> <span class="text-gray-300 font-mono">{selectedNode.node.file_id}</span></div>
        {#if selectedNode.node.community_id !== null}
          <div><span class="text-gray-500">Community:</span> <span class="text-gray-300">{selectedNode.node.community_id}</span></div>
        {/if}
        <div><span class="text-gray-500">Connections:</span> <span class="text-gray-300">{selectedNode.edgeCount}</span></div>
      </div>

      <h3 class="text-sm font-semibold text-gray-300 mt-4 mb-2">Connected Nodes</h3>
      <div class="space-y-1">
        {#each simNodes.filter((sn) => selectedConnected.has(sn.node.id)) as conn}
          <div class="text-sm text-gray-400 flex items-center gap-2">
            <span class="w-2 h-2 rounded-full inline-block" style="background: {conn.color}"></span>
            {conn.node.name}
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>
