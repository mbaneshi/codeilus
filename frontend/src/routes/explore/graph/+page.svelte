<script lang="ts">
  import { fetchGraph, fetchCommunities } from '$lib/api';
  import type { GraphNode, GraphEdge, Community } from '$lib/types';

  const COLORS = ['#6366f1', '#ec4899', '#14b8a6', '#f59e0b', '#ef4444', '#8b5cf6', '#06b6d4', '#84cc16', '#f97316', '#a855f7', '#22d3ee', '#4ade80'];

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
  let allNodes = $state<GraphNode[]>([]);
  let allEdges = $state<GraphEdge[]>([]);
  let communities = $state<Community[]>([]);
  let selectedCommunity = $state<number | null>(null);
  let maxNodes = $state(100);
  let nodes = $state<GraphNode[]>([]);
  let edges = $state<GraphEdge[]>([]);
  let simNodes = $state<SimNode[]>([]);
  let hoveredNode = $state<SimNode | null>(null);
  let selectedNode = $state<SimNode | null>(null);
  let mouseX = $state(0);
  let mouseY = $state(0);
  let svgEl: SVGSVGElement | undefined = $state();
  let animFrame = 0;
  let width = $state(900);
  let height = $state(600);

  let communityCount = $derived(new Set(nodes.map((n) => n.community_id).filter((c) => c !== null)).size);

  function formatLabel(label: string): string {
    return label.replace(/^cluster_/, '').replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());
  }

  function applyFilter() {
    // Stop any running simulation
    if (animFrame) cancelAnimationFrame(animFrame);

    // Filter by community
    let filtered = allNodes;
    if (selectedCommunity !== null) {
      filtered = allNodes.filter((n) => n.community_id === selectedCommunity);
    }

    // Sort by edge count (most connected first) and limit
    const edgeCounts = new Map<number, number>();
    for (const e of allEdges) {
      edgeCounts.set(e.source_id, (edgeCounts.get(e.source_id) ?? 0) + 1);
      edgeCounts.set(e.target_id, (edgeCounts.get(e.target_id) ?? 0) + 1);
    }
    filtered = [...filtered].sort((a, b) => (edgeCounts.get(b.id) ?? 0) - (edgeCounts.get(a.id) ?? 0));
    nodes = filtered.slice(0, maxNodes);

    // Only include edges between visible nodes
    const nodeIds = new Set(nodes.map((n) => n.id));
    edges = allEdges.filter((e) => nodeIds.has(e.source_id) && nodeIds.has(e.target_id));

    selectedNode = null;
    hoveredNode = null;

    if (nodes.length > 0) {
      initSimulation();
      animFrame = requestAnimationFrame(tick);
    }
  }

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

    // Place nodes in a circle initially, grouped by community
    const communityGroups = new Map<number, number[]>();
    nodes.forEach((n, idx) => {
      const cid = n.community_id ?? -1;
      if (!communityGroups.has(cid)) communityGroups.set(cid, []);
      communityGroups.get(cid)!.push(idx);
    });

    const cx = width / 2;
    const cy = height / 2;
    const baseRadius = Math.min(width, height) * 0.35;
    let groupAngle = 0;
    const groupAngleStep = (2 * Math.PI) / communityGroups.size;

    const positions: { x: number; y: number }[] = new Array(nodes.length);
    for (const [, indices] of communityGroups) {
      const groupCx = cx + baseRadius * 0.5 * Math.cos(groupAngle);
      const groupCy = cy + baseRadius * 0.5 * Math.sin(groupAngle);
      indices.forEach((idx, j) => {
        const memberAngle = groupAngle + ((j / indices.length) * Math.PI * 0.8 - Math.PI * 0.4);
        const r = baseRadius * 0.3 + Math.random() * baseRadius * 0.2;
        positions[idx] = {
          x: groupCx + r * Math.cos(memberAngle),
          y: groupCy + r * Math.sin(memberAngle),
        };
      });
      groupAngle += groupAngleStep;
    }

    simNodes = nodes.map((n, idx) => {
      const ec = edgeCounts.get(n.id) ?? 0;
      return {
        node: n,
        x: positions[idx]?.x ?? cx,
        y: positions[idx]?.y ?? cy,
        vx: 0,
        vy: 0,
        radius: Math.min(18, Math.max(5, 5 + ec * 1.2)),
        color: COLORS[(n.community_id ?? 0) % COLORS.length],
        edgeCount: ec,
      };
    });
  }

  let tickCount = 0;
  function tick() {
    tickCount++;
    const nodeMap = new Map<number, SimNode>();
    for (const sn of simNodes) nodeMap.set(sn.node.id, sn);

    const cx = width / 2;
    const cy = height / 2;

    // Reduce simulation speed over time for convergence
    const cooling = Math.max(0.1, 1 - tickCount / 300);

    for (let i = 0; i < simNodes.length; i++) {
      const a = simNodes[i];
      // Center gravity
      a.vx += (cx - a.x) * 0.001 * cooling;
      a.vy += (cy - a.y) * 0.001 * cooling;

      // Repulsion (only check nearby nodes for performance)
      for (let j = i + 1; j < simNodes.length; j++) {
        const b = simNodes[j];
        const dx = a.x - b.x;
        const dy = a.y - b.y;
        const distSq = dx * dx + dy * dy + 1;
        if (distSq > 40000) continue; // Skip far-away nodes
        const force = (1200 / distSq) * cooling;
        const fx = dx * force;
        const fy = dy * force;
        a.vx += fx;
        a.vy += fy;
        b.vx -= fx;
        b.vy -= fy;
      }
    }

    // Edge springs — pull connected nodes closer
    for (const e of edges) {
      const a = nodeMap.get(e.source_id);
      const b = nodeMap.get(e.target_id);
      if (!a || !b) continue;
      const dx = b.x - a.x;
      const dy = b.y - a.y;
      const dist = Math.sqrt(dx * dx + dy * dy) + 0.1;
      const target = 80;
      const force = (dist - target) * 0.005 * cooling;
      const fx = (dx / dist) * force;
      const fy = (dy / dist) * force;
      a.vx += fx;
      a.vy += fy;
      b.vx -= fx;
      b.vy -= fy;
    }

    // Community attraction — nodes in same community attract gently
    for (let i = 0; i < simNodes.length; i++) {
      for (let j = i + 1; j < simNodes.length; j++) {
        if (simNodes[i].node.community_id !== null &&
            simNodes[i].node.community_id === simNodes[j].node.community_id) {
          const dx = simNodes[j].x - simNodes[i].x;
          const dy = simNodes[j].y - simNodes[i].y;
          const dist = Math.sqrt(dx * dx + dy * dy) + 0.1;
          if (dist > 150) {
            const force = 0.002 * cooling;
            simNodes[i].vx += dx * force;
            simNodes[i].vy += dy * force;
            simNodes[j].vx -= dx * force;
            simNodes[j].vy -= dy * force;
          }
        }
      }
    }

    // Apply velocity with damping
    for (const sn of simNodes) {
      sn.vx *= 0.8;
      sn.vy *= 0.8;
      sn.x += sn.vx * 0.4;
      sn.y += sn.vy * 0.4;
      // Clamp to bounds with padding
      const pad = 20;
      sn.x = Math.max(sn.radius + pad, Math.min(width - sn.radius - pad, sn.x));
      sn.y = Math.max(sn.radius + pad, Math.min(height - sn.radius - pad, sn.y));
    }

    simNodes = [...simNodes]; // trigger reactivity

    // Stop after convergence
    if (tickCount < 500) {
      animFrame = requestAnimationFrame(tick);
    }
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
    if (!svgEl) return;
    const rect = svgEl.getBoundingClientRect();
    mouseX = e.clientX - rect.left;
    mouseY = e.clientY - rect.top;
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
    return 0.1;
  }

  function edgeOpacity(e: GraphEdge): number {
    if (!selectedNode) return 0.15;
    if (e.source_id === selectedNode.node.id || e.target_id === selectedNode.node.id) return 0.7;
    return 0.02;
  }

  if (typeof window !== 'undefined') {
    Promise.all([fetchGraph(), fetchCommunities()]).then(([graph, comms]) => {
      allNodes = graph.nodes;
      allEdges = graph.edges;
      communities = comms.sort((a, b) => b.member_count - a.member_count);
      loading = false;

      if (allNodes.length > 0) {
        if (svgEl) {
          const rect = svgEl.getBoundingClientRect();
          width = rect.width || 900;
          height = rect.height || 600;
        }
        applyFilter();
      }
    });
  }
</script>

<div class="flex h-full">
  <div class="flex-1 flex flex-col">
    <div class="p-4 border-b border-gray-800">
      <div class="flex items-center gap-4 mb-2">
        <a href="/explore" class="text-gray-500 hover:text-gray-300 transition-colors">&larr;</a>
        <h1 class="text-2xl font-bold">Knowledge Graph</h1>
        {#if !loading && nodes.length > 0}
          <span class="text-sm text-gray-400">{nodes.length} nodes</span>
          <span class="text-sm text-gray-400">{edges.length} edges</span>
          <span class="text-sm text-gray-400">{communityCount} communities</span>
        {/if}
      </div>

      {#if !loading && allNodes.length > 0}
        <div class="flex items-center gap-3 flex-wrap">
          <select
            class="bg-gray-800 border border-gray-700 rounded px-2 py-1 text-sm text-gray-200 outline-none focus:border-indigo-500"
            onchange={(e) => { selectedCommunity = (e.target as HTMLSelectElement).value === '' ? null : parseInt((e.target as HTMLSelectElement).value); tickCount = 0; applyFilter(); }}
          >
            <option value="">All communities</option>
            {#each communities as comm}
              <option value={comm.id}>{formatLabel(comm.label)} ({comm.member_count})</option>
            {/each}
          </select>

          <select
            class="bg-gray-800 border border-gray-700 rounded px-2 py-1 text-sm text-gray-200 outline-none focus:border-indigo-500"
            bind:value={maxNodes}
            onchange={() => { tickCount = 0; applyFilter(); }}
          >
            <option value={30}>Top 30 nodes</option>
            <option value={50}>Top 50 nodes</option>
            <option value={100}>Top 100 nodes</option>
            <option value={200}>Top 200 nodes</option>
            <option value={9999}>All nodes</option>
          </select>

          <span class="text-xs text-gray-500">Showing most connected nodes. Click a node to highlight connections.</span>
        </div>
      {/if}
    </div>

    {#if loading}
      <div class="flex-1 flex items-center justify-center">
        <p class="text-gray-400 animate-pulse">Loading...</p>
      </div>
    {:else if allNodes.length === 0}
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
                stroke-width="0.5"
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
              stroke={selectedNode?.node.id === sn.node.id ? '#fff' : 'none'}
              stroke-width="2"
            />
            {#if sn.radius >= 10}
              <text
                x={sn.x} y={sn.y + sn.radius + 12}
                text-anchor="middle"
                fill="#9ca3af"
                font-size="9"
                opacity={nodeOpacity(sn)}
              >{sn.node.name}</text>
            {/if}
          {/each}
        </svg>
        {#if hoveredNode}
          <div
            class="absolute pointer-events-none bg-gray-900 border border-gray-700 rounded px-3 py-2 text-sm text-gray-100 shadow-lg z-10"
            style="left: {mouseX + 12}px; top: {mouseY - 8}px"
          >
            <div class="font-mono font-semibold">{hoveredNode.node.name}</div>
            <div class="text-xs text-gray-400">{hoveredNode.node.kind} &middot; {hoveredNode.edgeCount} connections</div>
          </div>
        {/if}
      </div>
    {/if}
  </div>

  {#if selectedNode}
    <div class="w-72 border-l border-gray-800 bg-gray-900 p-4 overflow-auto">
      <h2 class="text-lg font-semibold mb-2 font-mono">{selectedNode.node.name}</h2>
      <div class="space-y-2 text-sm">
        <div><span class="text-gray-500">Kind:</span> <span class="text-gray-300">{selectedNode.node.kind}</span></div>
        {#if selectedNode.node.community_id !== null}
          {@const comm = communities.find((c) => c.id === selectedNode?.node.community_id)}
          <div><span class="text-gray-500">Module:</span> <span class="text-gray-300">{comm ? formatLabel(comm.label) : `Community ${selectedNode.node.community_id}`}</span></div>
        {/if}
        <div><span class="text-gray-500">Connections:</span> <span class="text-gray-300">{selectedNode.edgeCount}</span></div>
      </div>

      <h3 class="text-sm font-semibold text-gray-300 mt-4 mb-2">Connected ({selectedConnected.size})</h3>
      <div class="space-y-1 max-h-96 overflow-auto">
        {#each simNodes.filter((sn) => selectedConnected.has(sn.node.id)).sort((a, b) => b.edgeCount - a.edgeCount) as conn}
          <div class="text-sm text-gray-400 flex items-center gap-2 py-0.5">
            <span class="w-2 h-2 rounded-full inline-block shrink-0" style="background: {conn.color}"></span>
            <span class="font-mono text-xs truncate">{conn.node.name}</span>
            <span class="text-xs text-gray-600 ml-auto">{conn.node.kind}</span>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>
