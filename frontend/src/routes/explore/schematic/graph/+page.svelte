<script lang="ts">
  import { onMount } from 'svelte';
  import {
    fetchGraph, fetchCommunityGraph, fetchCommunities, fetchFiles,
    fetchNarrativeByTarget, fetchFileSource,
  } from '$lib/api';
  import type {
    GraphNode, GraphEdge, Community, CommunityGraphNode, CommunityGraphEdge,
    FileRow, SymbolRow, NarrativeResponse, SourceResponse,
  } from '$lib/types';
  import type { SchematicNode, SchematicEdge } from '$lib/schematic/types';
  import { computeLayout, type LayoutResult } from '$lib/schematic/elk-layout';
  import { edgePathD, edgeMidpoint } from '$lib/schematic/edge-path';
  import SchematicCanvas from '$lib/schematic/SchematicCanvas.svelte';
  import SchematicSearch from '$lib/schematic/SchematicSearch.svelte';
  import SchematicModal from '$lib/schematic/SchematicModal.svelte';
  import { LoadingSpinner } from '$lib/components';
  import { ArrowLeft, ChevronRight, Network, Layers, FileCode } from 'lucide-svelte';

  const COMMUNITY_COLORS = [
    '#6366f1', '#ec4899', '#14b8a6', '#f59e0b', '#8b5cf6',
    '#06b6d4', '#f97316', '#84cc16', '#ef4444', '#a855f7',
  ];

  const EDGE_COLORS: Record<string, string> = {
    'CALLS': '#6366f1', 'IMPORTS': '#14b8a6', 'EXTENDS': '#f59e0b',
    'IMPLEMENTS': '#ec4899', 'CONTAINS': '#4b5563',
  };

  const KIND_BADGES: Record<string, { label: string; bg: string }> = {
    function: { label: 'FUN', bg: 'rgba(99,102,241,0.15)' },
    method:   { label: 'MTD', bg: 'rgba(99,102,241,0.15)' },
    class:    { label: 'CLS', bg: 'rgba(236,72,153,0.15)' },
    struct:   { label: 'STR', bg: 'rgba(20,184,166,0.15)' },
    enum:     { label: 'ENM', bg: 'rgba(245,158,11,0.15)' },
    trait:    { label: 'TRT', bg: 'rgba(139,92,246,0.15)' },
    interface:{ label: 'IFC', bg: 'rgba(139,92,246,0.15)' },
    impl:     { label: 'IMP', bg: 'rgba(6,182,212,0.15)' },
    module:   { label: 'MOD', bg: 'rgba(132,204,22,0.15)' },
    constant: { label: 'CON', bg: 'rgba(156,163,175,0.15)' },
  };

  type ZoomLevel = 'communities' | 'community' | 'symbols';

  let loading = $state(true);
  let zoomLevel = $state<ZoomLevel>('communities');
  let layout = $state<LayoutResult | null>(null);
  let flatNodes = $state<SchematicNode[]>([]);
  let visibleEdges = $state<SchematicEdge[]>([]);
  let highlighted = $state<Set<string>>(new Set());
  let canvasRef: SchematicCanvas | undefined = $state();
  let breadcrumb = $state<{ label: string; action: () => void }[]>([]);

  // Raw data
  let allNodes = $state<GraphNode[]>([]);
  let allEdges = $state<GraphEdge[]>([]);
  let communities = $state<Community[]>([]);
  let communityNodes = $state<CommunityGraphNode[]>([]);
  let communityEdges = $state<CommunityGraphEdge[]>([]);
  let files = $state<FileRow[]>([]);
  let activeCommunityId = $state<number | null>(null);

  // Modal
  let modalOpen = $state(false);
  let modalTitle = $state('');
  let modalSymbol = $state<GraphNode | null>(null);
  let modalNarrative = $state<NarrativeResponse | null>(null);
  let modalSource = $state<SourceResponse | null>(null);
  let modalLoading = $state(false);
  let modalConnections = $state<{ callers: GraphNode[]; callees: GraphNode[] }>({ callers: [], callees: [] });

  function communityColor(id: number): string {
    return COMMUNITY_COLORS[id % COMMUNITY_COLORS.length];
  }

  function fileNameById(fileId: number): string {
    const f = files.find(f => f.id === fileId);
    return f ? (f.path.split('/').pop() || f.path) : `file:${fileId}`;
  }

  async function showCommunities() {
    zoomLevel = 'communities';
    activeCommunityId = null;
    breadcrumb = [];

    const nodes: SchematicNode[] = communityNodes.map(c => ({
      id: `comm-${c.id}`,
      label: c.label || `Community ${c.id}`,
      width: Math.max(160, (c.label || '').length * 8 + 60),
      height: 60,
      metadata: { type: 'community', communityId: c.id, memberCount: c.member_count, cohesion: c.cohesion },
    }));

    const edges: SchematicEdge[] = communityEdges.map((e, i) => ({
      id: `ce-${i}`,
      source: `comm-${e.source_id}`,
      target: `comm-${e.target_id}`,
      kind: 'CALLS',
    }));

    flatNodes = nodes;
    visibleEdges = edges;
    layout = await computeLayout(nodes, edges, { algorithm: 'layered', direction: 'RIGHT', nodeSpacing: 30, layerSpacing: 80 });
    requestAnimationFrame(() => canvasRef?.fitToView(layout!.width, layout!.height));
  }

  async function drillIntoCommunity(communityId: number) {
    loading = true;
    zoomLevel = 'community';
    activeCommunityId = communityId;
    const comm = communities.find(c => c.id === communityId);
    const commLabel = comm?.label || `Community ${communityId}`;

    breadcrumb = [
      { label: 'All Communities', action: () => { showCommunities(); } },
    ];

    const members = new Set(comm?.members || []);
    const memberNodes = allNodes.filter(n => members.has(n.id));
    const memberEdges = allEdges.filter(e => members.has(e.source_id) && members.has(e.target_id));

    const nodes: SchematicNode[] = memberNodes.map(n => ({
      id: `sym-${n.id}`,
      label: n.name,
      width: Math.max(140, n.name.length * 7 + 50),
      height: 44,
      metadata: { type: 'symbol', node: n, kind: n.kind, fileId: n.file_id, communityId: n.community_id },
    }));

    const edges: SchematicEdge[] = memberEdges.map((e, i) => ({
      id: `se-${i}`,
      source: `sym-${e.source_id}`,
      target: `sym-${e.target_id}`,
      kind: e.kind,
      label: e.kind.toLowerCase(),
    }));

    flatNodes = nodes;
    visibleEdges = edges;
    layout = await computeLayout(nodes, edges, { algorithm: 'layered', direction: 'DOWN', nodeSpacing: 20, layerSpacing: 50 });
    loading = false;
    requestAnimationFrame(() => canvasRef?.fitToView(layout!.width, layout!.height));
  }

  async function openSymbolModal(node: GraphNode) {
    modalOpen = true;
    modalTitle = node.name;
    modalSymbol = node;
    modalNarrative = null;
    modalSource = null;
    modalLoading = true;

    const callers = allEdges.filter(e => e.target_id === node.id).map(e => allNodes.find(n => n.id === e.source_id)).filter(Boolean) as GraphNode[];
    const callees = allEdges.filter(e => e.source_id === node.id).map(e => allNodes.find(n => n.id === e.target_id)).filter(Boolean) as GraphNode[];
    modalConnections = { callers, callees };

    const [narr, src] = await Promise.all([
      fetchNarrativeByTarget('symbol_explanation', node.id),
      fetchFileSource(node.file_id),
    ]);
    modalNarrative = narr;
    modalSource = src;
    modalLoading = false;
  }

  function handleNodeClick(nodeId: string) {
    const node = flatNodes.find(n => n.id === nodeId);
    if (!node) return;
    if (node.metadata.type === 'community') {
      drillIntoCommunity(node.metadata.communityId as number);
    } else if (node.metadata.type === 'symbol') {
      openSymbolModal(node.metadata.node as GraphNode);
    }
  }

  function handleFocus(nodeId: string) {
    const pos = layout?.nodes.get(nodeId);
    if (pos && canvasRef) {
      canvasRef.zoomToNode(pos.x, pos.y, pos.width, pos.height);
    }
  }

  onMount(async () => {
    const [graphData, commGraph, comms, fileList] = await Promise.all([
      fetchGraph(), fetchCommunityGraph(), fetchCommunities(), fetchFiles(),
    ]);
    allNodes = graphData.nodes;
    allEdges = graphData.edges;
    communities = comms;
    communityNodes = commGraph.nodes;
    communityEdges = commGraph.edges;
    files = fileList;

    await showCommunities();
    loading = false;
  });
</script>

<div class="h-full flex flex-col">
  <div class="flex items-center gap-3 px-5 py-3 border-b border-[var(--c-border)] bg-[var(--surface-1)] shrink-0">
    <a href="/explore" class="p-1.5 rounded-lg hover:bg-[var(--surface-2)] text-[var(--c-text-muted)] hover:text-[var(--c-text-primary)] transition-colors">
      <ArrowLeft size={16} />
    </a>
    <Network size={18} class="text-violet-400" />
    <h1 class="text-base font-semibold">Symbol Graph</h1>

    {#if breadcrumb.length > 0}
      {#each breadcrumb as crumb}
        <ChevronRight size={14} class="text-[var(--c-text-muted)]" />
        <button onclick={crumb.action} class="text-sm text-[var(--c-accent)] hover:underline">{crumb.label}</button>
      {/each}
      <ChevronRight size={14} class="text-[var(--c-text-muted)]" />
      <span class="text-sm text-[var(--c-text-secondary)]">
        {#if activeCommunityId !== null}
          {communities.find(c => c.id === activeCommunityId)?.label || `Community ${activeCommunityId}`}
        {/if}
      </span>
    {/if}

    <span class="text-xs text-[var(--c-text-muted)] ml-auto">{flatNodes.length} nodes</span>
  </div>

  <div class="flex-1 min-h-0 relative">
    {#if loading}
      <div class="flex items-center justify-center h-full">
        <LoadingSpinner />
      </div>
    {:else}
      <SchematicSearch
        nodes={flatNodes}
        onfocus={handleFocus}
        onhighlight={(ids) => highlighted = ids}
      />
      <SchematicCanvas bind:this={canvasRef} width={layout?.width ?? 4000} height={layout?.height ?? 3000}>
        {#snippet children()}
          <!-- Edges -->
          {#each visibleEdges as edge}
            {@const pts = layout?.edges.get(edge.id)?.points}
            {@const color = EDGE_COLORS[edge.kind || ''] || 'var(--c-border-hover)'}
            {#if pts}
              <path
                d={edgePathD(pts)}
                fill="none"
                stroke={color}
                stroke-width="1.5"
                marker-end="url(#arrowhead)"
                opacity="0.5"
              />
              {#if edge.label}
                {@const mid = edgeMidpoint(pts)}
                <text x={mid.x} y={mid.y - 6} text-anchor="middle" fill="var(--c-text-muted)" font-size="9">{edge.label}</text>
              {/if}
            {/if}
          {/each}

          <!-- Nodes -->
          {#each flatNodes as node}
            {@const pos = layout?.nodes.get(node.id)}
            {#if pos}
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <g
                transform="translate({pos.x},{pos.y})"
                onclick={() => handleNodeClick(node.id)}
                style="cursor: pointer"
              >
                {#if node.metadata.type === 'community'}
                  {@const cid = node.metadata.communityId as number}
                  <rect
                    width={pos.width} height={pos.height} rx="10"
                    fill="var(--surface-1)"
                    stroke={highlighted.has(node.id) ? 'var(--c-accent)' : communityColor(cid)}
                    stroke-width={highlighted.has(node.id) ? 2.5 : 2}
                  />
                  <foreignObject width={pos.width} height={pos.height}>
                    <div class="flex flex-col items-center justify-center h-full gap-1" xmlns="http://www.w3.org/1999/xhtml">
                      <div class="flex items-center gap-2">
                        <span class="w-2.5 h-2.5 rounded-full" style="background: {communityColor(cid)}"></span>
                        <span class="text-xs font-semibold text-[var(--c-text-primary)]">{node.label}</span>
                      </div>
                      <span class="text-[10px] text-[var(--c-text-muted)]">{node.metadata.memberCount} symbols</span>
                    </div>
                  </foreignObject>

                {:else if node.metadata.type === 'symbol'}
                  {@const gn = node.metadata.node as GraphNode}
                  {@const cid = gn.community_id ?? 0}
                  {@const badge = KIND_BADGES[gn.kind] || { label: gn.kind.slice(0,3).toUpperCase(), bg: 'rgba(156,163,175,0.15)' }}
                  <rect
                    width={pos.width} height={pos.height} rx="8"
                    fill="var(--surface-1)"
                    stroke={highlighted.has(node.id) ? 'var(--c-accent)' : communityColor(cid)}
                    stroke-width={highlighted.has(node.id) ? 2 : 1}
                  />
                  <!-- Community color accent bar -->
                  <rect x="0" y="0" width="3" height={pos.height} rx="1.5" fill={communityColor(cid)} />
                  <foreignObject width={pos.width} height={pos.height}>
                    <div class="flex items-center gap-2 px-3 h-full" xmlns="http://www.w3.org/1999/xhtml">
                      <span class="px-1.5 py-0.5 rounded text-[10px] font-medium" style="background: {badge.bg}; color: var(--c-text-secondary)">{badge.label}</span>
                      <span class="text-xs font-mono text-[var(--c-text-primary)] truncate">{node.label}</span>
                      <span class="text-[10px] text-[var(--c-text-muted)] ml-auto shrink-0">{fileNameById(gn.file_id)}</span>
                    </div>
                  </foreignObject>
                {/if}
              </g>
            {/if}
          {/each}
        {/snippet}
      </SchematicCanvas>
    {/if}
  </div>
</div>

<!-- Symbol Detail Modal -->
<SchematicModal open={modalOpen} title={modalTitle} onclose={() => modalOpen = false}>
  {#snippet children()}
    {#if modalLoading}
      <div class="flex justify-center py-8"><LoadingSpinner /></div>
    {:else if modalSymbol}
      {@const badge = KIND_BADGES[modalSymbol.kind] || { label: modalSymbol.kind, bg: 'rgba(156,163,175,0.15)' }}
      <div class="space-y-4">
        <div class="flex items-center gap-2">
          <span class="px-2 py-0.5 rounded text-xs font-medium" style="background: {badge.bg}">{badge.label}</span>
          <span class="text-sm text-[var(--c-text-secondary)]">{fileNameById(modalSymbol.file_id)}</span>
        </div>

        {#if modalNarrative?.content}
          <div>
            <h3 class="text-sm font-medium text-[var(--c-text-primary)] mb-2">Explanation</h3>
            <p class="text-sm text-[var(--c-text-secondary)] leading-relaxed">{modalNarrative.content}</p>
          </div>
        {/if}

        {#if modalConnections.callers.length > 0}
          <div>
            <h3 class="text-sm font-medium text-[var(--c-text-primary)] mb-2">Called by ({modalConnections.callers.length})</h3>
            <div class="flex flex-wrap gap-1">
              {#each modalConnections.callers.slice(0, 10) as caller}
                <span class="px-2 py-1 bg-[var(--surface-2)] rounded text-xs font-mono text-[var(--c-text-secondary)]">{caller.name}</span>
              {/each}
            </div>
          </div>
        {/if}

        {#if modalConnections.callees.length > 0}
          <div>
            <h3 class="text-sm font-medium text-[var(--c-text-primary)] mb-2">Calls ({modalConnections.callees.length})</h3>
            <div class="flex flex-wrap gap-1">
              {#each modalConnections.callees.slice(0, 10) as callee}
                <span class="px-2 py-1 bg-[var(--surface-2)] rounded text-xs font-mono text-[var(--c-text-secondary)]">{callee.name}</span>
              {/each}
            </div>
          </div>
        {/if}

        {#if modalSource}
          <div>
            <h3 class="text-sm font-medium text-[var(--c-text-primary)] mb-2">Source</h3>
            <pre class="text-[11px] font-mono bg-[var(--surface-2)] rounded-lg p-3 overflow-auto max-h-64 text-[var(--c-text-secondary)]">{modalSource.lines.map(l => `${String(l.number).padStart(3)} ${l.content}`).join('\n')}</pre>
          </div>
        {/if}
      </div>
    {/if}
  {/snippet}
</SchematicModal>
