<script lang="ts">
  import { fetchGraph, fetchCommunityGraph, fetchCommunities, fetchFiles, fetchNarrativeByTarget, fetchFileSource } from '$lib/api';
  import type { GraphNode, GraphEdge, Community, CommunityGraphNode, CommunityGraphEdge, FileRow, NarrativeResponse, SourceResponse } from '$lib/types';
  import { layoutLayered, type LayoutNode, type LayoutEdge } from '$lib/schematic/layout';
  import { LoadingSpinner } from '$lib/components';

  const COMMUNITY_COLORS = ['#6366f1','#ec4899','#14b8a6','#f59e0b','#8b5cf6','#06b6d4','#f97316','#84cc16','#ef4444','#a855f7'];
  const EDGE_COLORS: Record<string, string> = { CALLS:'#6366f1', IMPORTS:'#14b8a6', EXTENDS:'#f59e0b', IMPLEMENTS:'#ec4899', CONTAINS:'#4b5563' };
  const KIND_LABELS: Record<string, string> = { function:'FUN', method:'MTD', class:'CLS', struct:'STR', enum:'ENM', trait:'TRT', interface:'IFC', impl:'IMP', module:'MOD' };

  let loading = $state(true);
  let nodes = $state<LayoutNode[]>([]);
  let edges = $state<LayoutEdge[]>([]);
  let canvasW = $state(4000);
  let canvasH = $state(3000);
  let searchQuery = $state('');
  let highlighted = $state<Set<string>>(new Set());
  let breadcrumb = $state<{ label: string; action: () => void }[]>([]);

  // Raw data
  let allNodes: GraphNode[] = [];
  let allEdges: GraphEdge[] = [];
  let communities: Community[] = [];
  let communityNodes: CommunityGraphNode[] = [];
  let communityEdges: CommunityGraphEdge[] = [];
  let files: FileRow[] = [];

  // Pan/zoom
  let tx = $state(20);
  let ty = $state(20);
  let scale = $state(0.8);
  let dragging = $state(false);
  let dragStart = { x: 0, y: 0, tx: 0, ty: 0 };

  // Modal
  let modalOpen = $state(false);
  let modalTitle = $state('');
  let modalSymbol = $state<GraphNode | null>(null);
  let modalNarrative = $state<NarrativeResponse | null>(null);
  let modalSource = $state<SourceResponse | null>(null);
  let modalLoading = $state(false);
  let modalCallers = $state<GraphNode[]>([]);
  let modalCallees = $state<GraphNode[]>([]);

  function commColor(id: number) { return COMMUNITY_COLORS[id % COMMUNITY_COLORS.length]; }
  function fileName(fid: number) { const f = files.find(f => f.id === fid); return f ? (f.path.split('/').pop() || f.path) : ''; }

  function showCommunities() {
    breadcrumb = [];
    const result = layoutLayered({
      nodes: communityNodes.map(c => ({
        id: `c-${c.id}`,
        label: c.label || `Community ${c.id}`,
        data: { type: 'community', communityId: c.id, memberCount: c.member_count, cohesion: c.cohesion },
      })),
      edges: communityEdges.map((e, i) => ({ from: `c-${e.source_id}`, to: `c-${e.target_id}` })),
    });
    nodes = result.nodes;
    edges = result.edges;
    canvasW = result.width;
    canvasH = result.height;
  }

  function drillCommunity(communityId: number) {
    const comm = communities.find(c => c.id === communityId);
    breadcrumb = [{ label: 'Communities', action: showCommunities }];
    const members = new Set(comm?.members || []);
    const memberNodes = allNodes.filter(n => members.has(n.id));
    const memberEdges = allEdges.filter(e => members.has(e.source_id) && members.has(e.target_id));

    const result = layoutLayered({
      nodes: memberNodes.map(n => ({
        id: `s-${n.id}`,
        label: n.name,
        data: { type: 'symbol', node: n, kind: n.kind, fileId: n.file_id, communityId: n.community_id },
      })),
      edges: memberEdges.map((e, i) => ({ from: `s-${e.source_id}`, to: `s-${e.target_id}`, kind: e.kind, label: e.kind.toLowerCase() })),
    });
    nodes = result.nodes;
    edges = result.edges;
    canvasW = result.width;
    canvasH = result.height;
  }

  function handleNodeClick(node: LayoutNode) {
    if (node.data.type === 'community') {
      drillCommunity(node.data.communityId as number);
    } else if (node.data.type === 'symbol') {
      openSymbolModal(node.data.node as GraphNode);
    }
  }

  async function openSymbolModal(gn: GraphNode) {
    modalOpen = true;
    modalTitle = gn.name;
    modalSymbol = gn;
    modalNarrative = null;
    modalSource = null;
    modalLoading = true;
    modalCallers = allEdges.filter(e => e.target_id === gn.id).map(e => allNodes.find(n => n.id === e.source_id)).filter(Boolean) as GraphNode[];
    modalCallees = allEdges.filter(e => e.source_id === gn.id).map(e => allNodes.find(n => n.id === e.target_id)).filter(Boolean) as GraphNode[];
    const [narr, src] = await Promise.all([
      fetchNarrativeByTarget('symbol_explanation', gn.id),
      fetchFileSource(gn.file_id),
    ]);
    modalNarrative = narr;
    modalSource = src;
    modalLoading = false;
  }

  function doSearch() {
    if (searchQuery.trim().length < 2) { highlighted = new Set(); return; }
    const q = searchQuery.toLowerCase();
    highlighted = new Set(nodes.filter(n => n.label.toLowerCase().includes(q)).map(n => n.id));
  }

  function onWheel(e: WheelEvent) { e.preventDefault(); scale = Math.max(0.05, Math.min(4, scale * (e.deltaY > 0 ? 0.92 : 1.08))); }
  function onPointerDown(e: PointerEvent) { if (e.button !== 0) return; dragging = true; dragStart = { x: e.clientX, y: e.clientY, tx, ty }; }
  function onPointerMove(e: PointerEvent) { if (!dragging) return; tx = dragStart.tx + (e.clientX - dragStart.x); ty = dragStart.ty + (e.clientY - dragStart.y); }
  function onPointerUp() { dragging = false; }

  let nodeMap = $derived(new Map(nodes.map(n => [n.id, n])));

  function edgeLine(from: LayoutNode, to: LayoutNode): string {
    const x1 = from.x + from.width / 2, y1 = from.y + from.height;
    const x2 = to.x + to.width / 2, y2 = to.y;
    return `M ${x1} ${y1} L ${x2} ${y2}`;
  }

  Promise.all([fetchGraph(), fetchCommunityGraph(), fetchCommunities(), fetchFiles()]).then(([gd, cg, co, fl]) => {
    allNodes = gd.nodes;
    allEdges = gd.edges;
    communities = co;
    communityNodes = cg.nodes;
    communityEdges = cg.edges;
    files = fl;
    showCommunities();
    loading = false;
  });
</script>

<div class="h-full flex flex-col">
  <div class="flex items-center gap-3 px-5 py-3 border-b border-[var(--c-border)] bg-[var(--surface-1)] shrink-0">
    <a href="/explore" class="p-1.5 rounded-lg hover:bg-[var(--surface-2)] text-[var(--c-text-muted)] hover:text-[var(--c-text-primary)] transition-colors text-sm">&larr;</a>
    <h1 class="text-base font-semibold">Symbol Graph</h1>
    {#each breadcrumb as crumb}
      <span class="text-[var(--c-text-muted)]">&rsaquo;</span>
      <button onclick={crumb.action} class="text-sm text-[var(--c-accent)] hover:underline">{crumb.label}</button>
    {/each}
    <span class="text-xs text-[var(--c-text-muted)] ml-auto">{nodes.length} nodes</span>
    <input
      type="text"
      placeholder="Search..."
      class="bg-[var(--surface-2)] border border-[var(--c-border)] rounded-lg px-3 py-1 text-sm text-[var(--c-text-primary)] placeholder:text-[var(--c-text-muted)] focus:border-[var(--c-accent)] outline-none w-40"
      bind:value={searchQuery}
      oninput={doSearch}
    />
  </div>

  <div class="flex-1 min-h-0 relative">
    {#if loading}
      <div class="flex items-center justify-center h-full"><LoadingSpinner /></div>
    {:else}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <svg
        class="w-full h-full select-none"
        style="cursor: {dragging ? 'grabbing' : 'grab'}; background: var(--surface-0);"
        onwheel={onWheel}
        onpointerdown={onPointerDown}
        onpointermove={onPointerMove}
        onpointerup={onPointerUp}
      >
        <defs>
          <marker id="ah" markerWidth="6" markerHeight="4" refX="6" refY="2" orient="auto">
            <polygon points="0 0, 6 2, 0 4" fill="var(--c-text-muted)" opacity="0.4" />
          </marker>
        </defs>
        <g transform="translate({tx},{ty}) scale({scale})">
          {#each edges as edge}
            {@const from = nodeMap.get(edge.from)}
            {@const to = nodeMap.get(edge.to)}
            {#if from && to}
              <path
                d={edgeLine(from, to)}
                fill="none"
                stroke={EDGE_COLORS[edge.kind || ''] || 'var(--c-border-hover)'}
                stroke-width="1.2"
                marker-end="url(#ah)"
                opacity="0.5"
              />
            {/if}
          {/each}

          {#each nodes as node}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <g transform="translate({node.x},{node.y})" onclick={() => handleNodeClick(node)} style="cursor: pointer">
              {#if node.data.type === 'community'}
                {@const cid = node.data.communityId as number}
                <rect
                  width={node.width} height={node.height} rx="8"
                  fill="var(--surface-1)"
                  stroke={highlighted.has(node.id) ? 'var(--c-accent)' : commColor(cid)}
                  stroke-width={highlighted.has(node.id) ? 2.5 : 2}
                />
                <circle cx="14" cy={node.height / 2} r="5" fill={commColor(cid)} />
                <text x="26" y="15" font-size="11" fill="var(--c-text-primary)" font-weight="600" font-family="var(--font-sans)">{node.label}</text>
                <text x="26" y="28" font-size="9" fill="var(--c-text-muted)" font-family="var(--font-sans)">{node.data.memberCount} symbols</text>
              {:else}
                {@const gn = node.data.node as GraphNode}
                {@const cid = gn.community_id ?? 0}
                <rect
                  width={node.width} height={node.height} rx="6"
                  fill="var(--surface-1)"
                  stroke={highlighted.has(node.id) ? 'var(--c-accent)' : commColor(cid)}
                  stroke-width={highlighted.has(node.id) ? 2 : 1}
                />
                <rect x="0" y="0" width="3" height={node.height} rx="1.5" fill={commColor(cid)} />
                <text x="10" y="14" font-size="9" fill="var(--c-text-muted)" font-family="var(--font-sans)">{KIND_LABELS[gn.kind] || gn.kind}</text>
                <text x="10" y="28" font-size="11" fill="var(--c-text-primary)" font-family="var(--font-mono)">{node.label}</text>
              {/if}
            </g>
          {/each}
        </g>
      </svg>
    {/if}
  </div>
</div>

<!-- Modal -->
{#if modalOpen}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="fixed inset-0 z-50 flex justify-end" onclick={() => modalOpen = false}>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="w-[400px] h-full bg-[var(--surface-1)] border-l border-[var(--c-border)] shadow-2xl overflow-auto" onclick={(e) => e.stopPropagation()}>
      <div class="flex items-center justify-between px-5 py-4 border-b border-[var(--c-border)] sticky top-0 bg-[var(--surface-1)]">
        <h2 class="text-base font-semibold text-[var(--c-text-primary)] truncate">{modalTitle}</h2>
        <button onclick={() => modalOpen = false} class="text-[var(--c-text-muted)] hover:text-[var(--c-text-primary)]">✕</button>
      </div>
      <div class="p-5 space-y-4">
        {#if modalLoading}
          <LoadingSpinner />
        {:else if modalSymbol}
          <div class="flex items-center gap-2 text-sm">
            <span class="px-2 py-0.5 rounded bg-[var(--surface-2)] text-[var(--c-text-muted)] text-xs">{KIND_LABELS[modalSymbol.kind] || modalSymbol.kind}</span>
            <span class="text-[var(--c-text-secondary)]">{fileName(modalSymbol.file_id)}</span>
          </div>

          {#if modalNarrative?.content}
            <div>
              <h3 class="text-sm font-medium text-[var(--c-text-primary)] mb-1">Explanation</h3>
              <p class="text-sm text-[var(--c-text-secondary)] leading-relaxed">{modalNarrative.content}</p>
            </div>
          {/if}

          {#if modalCallers.length > 0}
            <div>
              <h3 class="text-sm font-medium text-[var(--c-text-primary)] mb-1">Called by ({modalCallers.length})</h3>
              <div class="flex flex-wrap gap-1">
                {#each modalCallers.slice(0, 10) as c}
                  <span class="px-2 py-1 bg-[var(--surface-2)] rounded text-xs font-mono text-[var(--c-text-secondary)]">{c.name}</span>
                {/each}
              </div>
            </div>
          {/if}

          {#if modalCallees.length > 0}
            <div>
              <h3 class="text-sm font-medium text-[var(--c-text-primary)] mb-1">Calls ({modalCallees.length})</h3>
              <div class="flex flex-wrap gap-1">
                {#each modalCallees.slice(0, 10) as c}
                  <span class="px-2 py-1 bg-[var(--surface-2)] rounded text-xs font-mono text-[var(--c-text-secondary)]">{c.name}</span>
                {/each}
              </div>
            </div>
          {/if}

          {#if modalSource}
            <div>
              <h3 class="text-sm font-medium text-[var(--c-text-primary)] mb-1">Source</h3>
              <pre class="text-[11px] font-mono bg-[var(--surface-2)] rounded-lg p-3 overflow-auto max-h-64 text-[var(--c-text-secondary)]">{modalSource.lines.map(l => `${String(l.number).padStart(3)} ${l.content}`).join('\n')}</pre>
            </div>
          {/if}
        {/if}
      </div>
    </div>
  </div>
{/if}
