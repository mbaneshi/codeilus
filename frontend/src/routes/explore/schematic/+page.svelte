<script lang="ts">
  import { goto } from '$app/navigation';
  import { fetchSchematic, fetchSchematicExpand, fetchSchematicDetail, fetchAnnotationsByTarget, createAnnotation, deleteAnnotation, toggleAnnotationFlag } from '$lib/api';
  import type { SchematicNode, SchematicEdge, SchematicCommunity, SchematicDetail, Annotation } from '$lib/types';
  import { layoutTree, layoutLayered, computeFitToView, type LayoutNode, type LayoutEdge } from '$lib/schematic/layout';
  import { LoadingSpinner } from '$lib/components';
  import SchematicTooltip from '$lib/schematic/SchematicTooltip.svelte';
  import SchematicDetailTabs from '$lib/schematic/SchematicDetailTabs.svelte';
  import SchematicSourcePopup from '$lib/schematic/SchematicSourcePopup.svelte';
  import SchematicContextMenu from '$lib/schematic/SchematicContextMenu.svelte';
  import type { ContextAction } from '$lib/schematic/SchematicContextMenu.svelte';
  import SchematicKeyboardOverlay from '$lib/schematic/SchematicKeyboardOverlay.svelte';
  import SchematicMinimap from '$lib/schematic/SchematicMinimap.svelte';

  type Mode = 'tree' | 'graph';

  // ── Core state ──
  let loading = $state(true);
  let mode = $state<Mode>('tree');
  let layoutNodes = $state<LayoutNode[]>([]);
  let layoutEdges = $state<LayoutEdge[]>([]);
  let canvasW = $state(4000);
  let canvasH = $state(3000);
  let communities = $state<SchematicCommunity[]>([]);
  let meta = $state({ total_files: 0, total_symbols: 0, total_communities: 0, depth_returned: 0 });
  let allNodes = $state<Map<string, SchematicNode>>(new Map());
  let allEdges = $state<SchematicEdge[]>([]);
  let expandedSet = $state<Set<string>>(new Set());
  let expandingSet = $state<Set<string>>(new Set());
  let hiddenNodes = $state<Set<string>>(new Set());

  // ── Legend ──
  let showLegend = $state(false);

  // ── Search ──
  let searchQuery = $state('');
  let highlighted = $state<Set<string>>(new Set());

  // ── Pan/zoom ──
  let tx = $state(20);
  let ty = $state(20);
  let scale = $state(0.7);
  let dragging = $state(false);
  let dragStart = { x: 0, y: 0, tx: 0, ty: 0 };
  let containerW = $state(800);
  let containerH = $state(600);

  // ── Hover ──
  let hoveredNodeId = $state<string | null>(null);
  let tooltipX = $state(0);
  let tooltipY = $state(0);
  let hoveredEdgeIds = $derived(new Set(
    hoveredNodeId ? layoutEdges.filter(e => e.from === hoveredNodeId || e.to === hoveredNodeId).map(e => e.id) : []
  ));
  let ghostNodeIds = $derived(new Set(
    hoveredNodeId ? layoutEdges
      .filter(e => e.from === hoveredNodeId || e.to === hoveredNodeId)
      .flatMap(e => [e.from, e.to])
      .filter(id => id !== hoveredNodeId) : []
  ));

  // ── Detail panel ──
  let detailOpen = $state(false);
  let detailLoading = $state(false);
  let detailNode = $state<SchematicNode | null>(null);
  let detailData = $state<SchematicDetail | null>(null);
  let annotations = $state<Annotation[]>([]);
  let annotationsLoading = $state(false);

  // ── Source popup (double-click symbol) ──
  let sourcePopupNode = $state<SchematicNode | null>(null);
  let sourcePopupX = $state(0);
  let sourcePopupY = $state(0);

  // ── Context menu ──
  let contextMenuNode = $state<SchematicNode | null>(null);
  let contextMenuX = $state(0);
  let contextMenuY = $state(0);

  // ── Keyboard overlay ──
  let showKeyboardHelp = $state(false);

  // ── Community sidebar ──
  let selectedCommunity = $state<number | null>(null);

  // ── Click timer (single vs double) ──
  let clickTimer: ReturnType<typeof setTimeout> | null = null;

  const KIND_LABELS: Record<string, string> = { function:'FN', method:'MT', class:'CL', struct:'ST', enum:'EN', trait:'TR', interface:'IF', impl:'IM', module:'MD' };
  const EDGE_COLORS: Record<string, string> = { CALLS:'#6366f1', IMPORTS:'#14b8a6', EXTENDS:'#f59e0b', IMPLEMENTS:'#ec4899', contains:'var(--c-border)' };

  function commColor(id: number | undefined): string {
    if (id === undefined) return 'var(--c-border)';
    return communities.find(c => c.id === id)?.color || 'var(--c-border)';
  }

  // ── Layout ──
  function rebuildLayout() {
    const nodes = [...allNodes.values()].filter(n => !hiddenNodes.has(n.id));
    if (mode === 'tree') rebuildTreeLayout(nodes);
    else rebuildGraphLayout(nodes);
  }

  interface TreeBuildNode { id: string; label: string; data: Record<string, unknown>; children: TreeBuildNode[]; }

  function rebuildTreeLayout(nodes: SchematicNode[]) {
    const childMap = new Map<string, SchematicNode[]>();
    for (const n of nodes) {
      if (n.parent_id && allNodes.has(n.parent_id)) {
        const list = childMap.get(n.parent_id) || [];
        list.push(n);
        childMap.set(n.parent_id, list);
      }
    }

    function toTreeNode(sn: SchematicNode): TreeBuildNode {
      const children = expandedSet.has(sn.id) || sn.parent_id === null
        ? (childMap.get(sn.id) || []).map(toTreeNode) : [];
      return { id: sn.id, label: sn.label, data: { ...sn } as Record<string, unknown>, children };
    }

    const root = nodes.find(n => n.id === 'dir:.') || nodes[0];
    if (!root) { layoutNodes = []; layoutEdges = []; return; }
    const tree = toTreeNode(root);
    const result = layoutTree(tree);
    layoutNodes = result.nodes;
    layoutEdges = result.edges;
    canvasW = result.width;
    canvasH = result.height;
  }

  function rebuildGraphLayout(nodes: SchematicNode[]) {
    if (selectedCommunity) {
      const symNodes = nodes.filter(n => n.type === 'symbol' && n.community_id === selectedCommunity)
        .map(n => ({ id: n.id, label: n.label, data: { ...n } as Record<string, unknown> }));
      const symEdges = allEdges
        .filter(e => symNodes.some(n => n.id === e.source) && symNodes.some(n => n.id === e.target))
        .map(e => ({ from: e.source, to: e.target, kind: e.type, label: e.type.toLowerCase() }));
      const r = layoutLayered({ nodes: symNodes, edges: symEdges });
      layoutNodes = r.nodes; layoutEdges = r.edges; canvasW = r.width; canvasH = r.height;
      return;
    }
    const commNodes = communities.map(c => ({
      id: `comm:${c.id}`, label: c.label,
      data: { type: 'community', communityId: c.id, memberCount: c.member_count, color: c.color, chapter_title: c.chapter_title, progress: c.progress } as Record<string, unknown>,
    }));
    const r = layoutLayered({ nodes: commNodes, edges: [] });
    layoutNodes = r.nodes; layoutEdges = r.edges; canvasW = r.width; canvasH = r.height;
  }

  // ── Actions ──
  async function expandNode(nodeId: string) {
    if (expandedSet.has(nodeId) || expandingSet.has(nodeId)) return;
    expandingSet = new Set([...expandingSet, nodeId]);
    const isFile = allNodes.get(nodeId)?.type === 'file';
    const resp = await fetchSchematicExpand(nodeId, isFile, isFile);
    const newMap = new Map(allNodes);
    for (const n of resp.nodes) newMap.set(n.id, n);
    allNodes = newMap;
    allEdges = [...allEdges, ...resp.edges];
    expandedSet = new Set([...expandedSet, nodeId]);
    expandingSet = new Set([...expandingSet].filter(id => id !== nodeId));
    rebuildLayout();
  }

  function collapseNode(nodeId: string) {
    const toRemove = new Set<string>();
    function collect(pid: string) { for (const [id, n] of allNodes) { if (n.parent_id === pid) { toRemove.add(id); collect(id); } } }
    collect(nodeId);
    const newMap = new Map(allNodes);
    for (const id of toRemove) newMap.delete(id);
    allNodes = newMap;
    expandedSet = new Set([...expandedSet].filter(id => id !== nodeId));
    rebuildLayout();
  }

  async function selectNode(node: SchematicNode) {
    detailOpen = true;
    detailLoading = true;
    detailNode = node;
    detailData = null;
    annotations = [];
    annotationsLoading = true;
    const [detail, anns] = await Promise.all([
      fetchSchematicDetail(node.id),
      node.symbol_id ? fetchAnnotationsByTarget('node', node.symbol_id) :
      node.file_id ? fetchAnnotationsByTarget('node', node.file_id) : Promise.resolve([]),
    ]);
    detailData = detail;
    annotations = anns;
    detailLoading = false;
    annotationsLoading = false;
  }

  // ── Click handling (single vs double) ──
  function handleSingleClick(node: LayoutNode) {
    const sn = allNodes.get(node.id);
    if (!sn) return;
    if (sn.has_children && (sn.type === 'directory' || sn.type === 'file')) {
      expandedSet.has(sn.id) ? collapseNode(sn.id) : expandNode(sn.id);
    }
    if (sn.type === 'file' || sn.type === 'symbol') selectNode(sn);
  }

  function handleDoubleClick(node: LayoutNode, e: MouseEvent) {
    const sn = allNodes.get(node.id);
    if (!sn) return;
    if (sn.type === 'file' && sn.file_id) goto(`/explore/tree?fileId=${sn.file_id}`);
    else if (sn.type === 'symbol') { sourcePopupNode = sn; sourcePopupX = e.clientX; sourcePopupY = e.clientY; }
    else if (sn.type === 'community' && sn.chapter_id) goto(`/learn/${sn.chapter_id}`);
    else if (sn.type === 'directory') { /* recursive expand */ expandNode(sn.id); }
  }

  function handleNodePointerDown(node: LayoutNode, e: MouseEvent) {
    if (e.detail === 2) {
      // Double click
      if (clickTimer) { clearTimeout(clickTimer); clickTimer = null; }
      handleDoubleClick(node, e);
    } else {
      if (clickTimer) clearTimeout(clickTimer);
      clickTimer = setTimeout(() => { handleSingleClick(node); clickTimer = null; }, 220);
    }
  }

  function handleContextMenu(node: LayoutNode, e: MouseEvent) {
    e.preventDefault();
    const sn = allNodes.get(node.id);
    if (!sn) return;
    contextMenuNode = sn;
    contextMenuX = e.clientX;
    contextMenuY = e.clientY;
  }

  function handleContextAction(action: ContextAction) {
    if (!contextMenuNode) return;
    const sn = contextMenuNode;
    switch (action) {
      case 'copy-name': {
        try { navigator.clipboard?.writeText(sn.label); } catch { /* fallback */ }
        break;
      }
      case 'focus-here': {
        const pos = layoutNodes.find(n => n.id === sn.id);
        if (pos) { const fit = computeFitToView([pos], containerW, containerH, 100); tx = fit.tx; ty = fit.ty; scale = fit.scale; }
        break;
      }
      case 'hide': hiddenNodes = new Set([...hiddenNodes, sn.id]); rebuildLayout(); break;
      case 'view-source': if (sn.file_id) goto(`/explore/tree?fileId=${sn.file_id}`); break;
      case 'ask-ai': goto(`/ask`); break;
      case 'add-note': selectNode(sn); break;
      case 'show-call-chain': selectNode(sn); break;
      case 'start-quiz': if (sn.chapter_id) goto(`/learn/${sn.chapter_id}`); break;
      case 'start-learning': if (sn.chapter_id) goto(`/learn/${sn.chapter_id}`); break;
      case 'show-members':
      case 'filter-community':
        if (sn.community_id) {
          selectedCommunity = sn.community_id;
          fetchSchematic(10, sn.community_id, true, true).then(r => {
            const m = new Map(allNodes); for (const n of r.nodes) m.set(n.id, n); allNodes = m;
            allEdges = [...allEdges, ...r.edges]; rebuildLayout();
          });
        }
        break;
    }
  }

  function handleNavigate(nodeId: string) {
    // Find node on canvas or expand parent
    const pos = layoutNodes.find(n => n.id === nodeId);
    if (pos) {
      const fit = computeFitToView([pos], containerW, containerH, 100);
      tx = fit.tx; ty = fit.ty; scale = fit.scale;
      const sn = allNodes.get(nodeId);
      if (sn) selectNode(sn);
    }
  }

  // ── Search ──
  function doSearch() {
    if (searchQuery.trim().length < 2) { highlighted = new Set(); return; }
    const q = searchQuery.toLowerCase();
    highlighted = new Set([...allNodes.values()].filter(n => n.label.toLowerCase().includes(q)).map(n => n.id));
  }

  // ── Mode switch ──
  function switchMode(m: Mode) { mode = m; selectedCommunity = null; rebuildLayout(); }

  // ── Fit to view ──
  function fitToView() {
    const fit = computeFitToView(layoutNodes, containerW, containerH);
    tx = fit.tx; ty = fit.ty; scale = fit.scale;
  }

  // ── Pan/zoom ──
  function onWheel(e: WheelEvent) { e.preventDefault(); scale = Math.max(0.05, Math.min(4, scale * (e.deltaY > 0 ? 0.92 : 1.08))); }
  function onPointerDown(e: PointerEvent) { if (e.button !== 0) return; dragging = true; dragStart = { x: e.clientX, y: e.clientY, tx, ty }; }
  function onPointerMove(e: PointerEvent) { if (!dragging) return; tx = dragStart.tx + (e.clientX - dragStart.x); ty = dragStart.ty + (e.clientY - dragStart.y); }
  function onPointerUp() { dragging = false; }

  function edgePath(from: LayoutNode, to: LayoutNode): string {
    const x1 = from.x + from.width, y1 = from.y + from.height / 2;
    const x2 = to.x, y2 = to.y + to.height / 2;
    const mx = (x1 + x2) / 2;
    return `M ${x1} ${y1} C ${mx} ${y1}, ${mx} ${y2}, ${x2} ${y2}`;
  }

  let nodeMap = $derived(new Map(layoutNodes.map(n => [n.id, n])));

  // ── Keyboard shortcuts ──
  function handleKeydown(e: KeyboardEvent) {
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
    if ((e.metaKey || e.ctrlKey) && e.key === '/') { e.preventDefault(); goto('/ask'); }
    if (e.key === '1') switchMode('tree');
    if (e.key === '2') switchMode('graph');
    if (e.key === 'f' || e.key === 'F') fitToView();
    if (e.key === '?') showKeyboardHelp = !showKeyboardHelp;
    if (e.key === 'Escape') {
      if (showKeyboardHelp) showKeyboardHelp = false;
      else if (contextMenuNode) contextMenuNode = null;
      else if (sourcePopupNode) sourcePopupNode = null;
      else if (detailOpen) detailOpen = false;
    }
  }

  // ── Annotation handlers ──
  async function handleAnnotationCreate(content: string) {
    if (!detailNode) return;
    const targetType = detailNode.type === 'symbol' ? 'node' : 'node';
    const targetId = detailNode.symbol_id || detailNode.file_id;
    if (!targetId) return;
    const ann = await createAnnotation(targetType, targetId, content);
    if (ann) annotations = [...annotations, ann];
  }
  async function handleAnnotationDelete(id: number) {
    await deleteAnnotation(id);
    annotations = annotations.filter(a => a.id !== id);
  }
  async function handleAnnotationFlag(id: number) {
    const result = await toggleAnnotationFlag(id);
    if (result) annotations = annotations.map(a => a.id === id ? { ...a, flagged: result.flagged } : a);
  }

  // ── Init ──
  fetchSchematic(2).then(resp => {
    const map = new Map<string, SchematicNode>();
    for (const n of resp.nodes) map.set(n.id, n);
    allNodes = map;
    allEdges = resp.edges;
    communities = resp.communities;
    meta = resp.meta;
    expandedSet = new Set(['dir:.']);
    rebuildLayout();
    loading = false;
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="h-full flex flex-col">
  <!-- Toolbar -->
  <div class="flex items-center gap-2 px-4 py-2 border-b border-[var(--c-border)] bg-[var(--surface-1)] shrink-0 text-xs">
    <a href="/explore" class="text-[var(--c-text-muted)] hover:text-[var(--c-text-primary)]">&larr;</a>
    <span class="font-semibold text-sm">Schematic</span>

    <div class="flex bg-[var(--surface-2)] rounded-lg p-0.5">
      <button class="px-2.5 py-0.5 rounded-md {mode === 'tree' ? 'bg-[var(--c-accent)] text-white' : 'text-[var(--c-text-secondary)]'}" onclick={() => switchMode('tree')}>Tree</button>
      <button class="px-2.5 py-0.5 rounded-md {mode === 'graph' ? 'bg-[var(--c-accent)] text-white' : 'text-[var(--c-text-secondary)]'}" onclick={() => switchMode('graph')}>Graph</button>
    </div>

    {#if selectedCommunity}
      <span class="text-[var(--c-text-muted)]">&rsaquo;</span>
      <button onclick={() => { selectedCommunity = null; rebuildLayout(); }} class="text-[var(--c-accent)] hover:underline">Communities</button>
      <span class="text-[var(--c-text-muted)]">&rsaquo;</span>
      <span class="text-[var(--c-text-secondary)]">{communities.find(c => c.id === selectedCommunity)?.label}</span>
    {/if}

    <div class="ml-auto flex items-center gap-2">
      <button onclick={fitToView} class="px-2 py-0.5 rounded bg-[var(--surface-2)] hover:bg-[var(--surface-3)] text-[var(--c-text-muted)]" title="Fit to view (F)">Fit</button>
      <button onclick={() => showLegend = !showLegend} class="px-2 py-0.5 rounded {showLegend ? 'bg-[var(--c-accent)] text-white' : 'bg-[var(--surface-2)] text-[var(--c-text-muted)]'} hover:bg-[var(--surface-3)]" title="Toggle legend">Legend</button>
      <button onclick={() => showKeyboardHelp = true} class="px-2 py-0.5 rounded bg-[var(--surface-2)] hover:bg-[var(--surface-3)] text-[var(--c-text-muted)]" title="Shortcuts (?)">?</button>
      <span class="text-[10px] text-[var(--c-text-muted)]">{meta.total_files}f &middot; {meta.total_symbols}s</span>
      <input
        type="text" placeholder="Search..."
        class="bg-[var(--surface-2)] border border-[var(--c-border)] rounded-lg px-2 py-0.5 text-xs text-[var(--c-text-primary)] placeholder:text-[var(--c-text-muted)] focus:border-[var(--c-accent)] outline-none w-36"
        bind:value={searchQuery} oninput={doSearch}
      />
    </div>
  </div>

  <div class="flex-1 min-h-0 flex">
    <!-- Community sidebar (graph mode) -->
    {#if mode === 'graph' && !selectedCommunity}
      <div class="w-52 shrink-0 border-r border-[var(--c-border)] bg-[var(--surface-1)] overflow-auto p-2 space-y-1">
        <h2 class="text-[10px] font-semibold text-[var(--c-text-muted)] uppercase tracking-wider mb-1 px-1">Communities</h2>
        {#each communities as c}
          <button
            class="w-full text-left px-2 py-1.5 rounded-lg hover:bg-[var(--surface-2)] transition-colors"
            onclick={() => { selectedCommunity = c.id; fetchSchematic(10, c.id, true, true).then(r => { const m = new Map(allNodes); for (const n of r.nodes) m.set(n.id, n); allNodes = m; allEdges = [...allEdges, ...r.edges]; rebuildLayout(); }); }}
          >
            <div class="flex items-center gap-1.5">
              <span class="w-2 h-2 rounded-full" style="background: {c.color}"></span>
              <span class="text-xs text-[var(--c-text-primary)] truncate">{c.label}</span>
              <span class="text-[10px] text-[var(--c-text-muted)] ml-auto">{c.member_count}</span>
            </div>
            {#if c.progress}
              <div class="mt-0.5 h-1 bg-[var(--surface-3)] rounded-full overflow-hidden">
                <div class="h-full bg-[var(--c-success)] rounded-full" style="width: {(c.progress.completed / Math.max(c.progress.total, 1)) * 100}%"></div>
              </div>
            {/if}
          </button>
        {/each}
      </div>
    {/if}

    <!-- Canvas -->
    <div class="flex-1 min-h-0 relative" bind:clientWidth={containerW} bind:clientHeight={containerH}>
      {#if loading}
        <div class="flex items-center justify-center h-full"><LoadingSpinner /></div>
      {:else}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <svg
          class="w-full h-full select-none"
          style="cursor: {dragging ? 'grabbing' : 'grab'}; background: var(--surface-0);"
          onwheel={onWheel} onpointerdown={onPointerDown} onpointermove={onPointerMove} onpointerup={onPointerUp}
        >
          <defs>
            <filter id="shadow" x="-10%" y="-10%" width="130%" height="130%">
              <feDropShadow dx="0" dy="2" stdDeviation="3" flood-opacity="0.15" />
            </filter>
            <marker id="ah" markerWidth="6" markerHeight="4" refX="6" refY="2" orient="auto">
              <polygon points="0 0, 6 2, 0 4" fill="var(--c-text-muted)" opacity="0.4" />
            </marker>
          </defs>
          <g transform="translate({tx},{ty}) scale({scale})">
            <!-- Edges -->
            {#each layoutEdges as edge}
              {@const from = nodeMap.get(edge.from)}
              {@const to = nodeMap.get(edge.to)}
              {#if from && to}
                <path
                  d={edgePath(from, to)} fill="none"
                  stroke={EDGE_COLORS[edge.kind || ''] || 'var(--c-border-hover)'}
                  stroke-width={hoveredEdgeIds.has(edge.id) ? 2.5 : 1.2}
                  marker-end="url(#ah)"
                  opacity={hoveredNodeId ? (hoveredEdgeIds.has(edge.id) ? 0.9 : 0.1) : 0.5}
                />
              {/if}
            {/each}

            <!-- Nodes -->
            {#each layoutNodes as node}
              {@const sn = allNodes.get(node.id)}
              {@const isHovered = node.id === hoveredNodeId}
              {@const isGhost = ghostNodeIds.has(node.id)}
              {@const dimmed = hoveredNodeId && !isHovered && !isGhost}
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <g
                transform="translate({node.x},{node.y})"
                style="cursor: pointer; opacity: {dimmed ? 0.25 : 1}; transition: opacity 0.15s;"
                onclick={(e) => handleNodePointerDown(node, e)}
                oncontextmenu={(e) => handleContextMenu(node, e)}
                onpointerenter={(e) => { hoveredNodeId = node.id; tooltipX = e.clientX; tooltipY = e.clientY; }}
                onpointerleave={() => hoveredNodeId = null}
              >
                <rect
                  width={node.width} height={node.height} rx="8"
                  fill="var(--surface-1)"
                  stroke={highlighted.has(node.id) ? 'var(--c-accent)' : isHovered ? 'var(--c-accent-hover)' : commColor(sn?.community_id)}
                  stroke-width={highlighted.has(node.id) || isHovered ? 2.5 : sn?.community_id ? 1.5 : 1}
                  filter={isHovered ? 'url(#shadow)' : undefined}
                />
                {#if sn?.community_color}
                  <rect x="0" y="0" width="3" height={node.height} rx="1.5" fill={sn.community_color} />
                {/if}

                {#if sn?.type === 'directory'}
                  {@const isExp = expandedSet.has(node.id)}
                  <text x="8" y="23" font-size="11" fill="var(--c-text-secondary)" font-family="var(--font-sans)">
                    {isExp ? '▼' : '▶'} {node.label === '.' ? 'codeilus' : node.label}
                  </text>
                  {#if expandingSet.has(node.id)}
                    <text x={node.width - 20} y="23" font-size="9" fill="var(--c-accent)">...</text>
                  {/if}
                {:else if sn?.type === 'file'}
                  <circle cx="12" cy="18" r="4" fill={sn.community_color || 'var(--c-text-muted)'} />
                  <text x="22" y="15" font-size="11" fill="var(--c-text-primary)" font-family="var(--font-mono)">{node.label}</text>
                  <text x="22" y="28" font-size="9" fill="var(--c-text-muted)">{sn.language || ''} · {sn.sloc} loc</text>
                {:else if sn?.type === 'symbol'}
                  <text x="8" y="14" font-size="9" fill="var(--c-text-muted)">{KIND_LABELS[sn.kind || ''] || sn.kind}</text>
                  <text x="8" y="28" font-size="11" fill="var(--c-text-primary)" font-family="var(--font-mono)">{node.label}</text>
                {:else}
                  {@const data = node.data}
                  {#if data.color}
                    <circle cx="14" cy={node.height / 2} r="5" fill={data.color as string} />
                  {/if}
                  <text x="26" y="14" font-size="11" fill="var(--c-text-primary)" font-weight="600">{node.label}</text>
                  <text x="26" y="27" font-size="9" fill="var(--c-text-muted)">{data.memberCount} symbols</text>
                {/if}
              </g>
            {/each}
          </g>
        </svg>

        <!-- Minimap -->
        <SchematicMinimap
          nodes={layoutNodes} {canvasW} {canvasH}
          viewX={tx} viewY={ty} viewScale={scale} viewW={containerW} viewH={containerH}
          onpan={(newTx, newTy) => { tx = newTx; ty = newTy; }}
        />

        <!-- Legend -->
        {#if showLegend}
          <div class="absolute bottom-3 left-3 z-20 bg-[var(--surface-1)] border border-[var(--c-border)] rounded-lg shadow-lg p-3 w-[200px] text-[10px]">
            <div class="font-semibold text-[var(--c-text-primary)] text-xs mb-2">Legend</div>

            <div class="mb-2">
              <div class="text-[var(--c-text-muted)] uppercase tracking-wider mb-1">Nodes</div>
              <div class="space-y-1">
                <div class="flex items-center gap-2"><span class="text-[var(--c-text-secondary)]">▶</span> <span class="text-[var(--c-text-secondary)]">Directory (click to expand)</span></div>
                <div class="flex items-center gap-2"><span class="w-2.5 h-2.5 rounded-full bg-[var(--c-text-muted)]"></span> <span class="text-[var(--c-text-secondary)]">File</span></div>
                <div class="flex items-center gap-2"><span class="text-[var(--c-text-muted)]">FN</span> <span class="text-[var(--c-text-secondary)]">Symbol (function, class...)</span></div>
                <div class="flex items-center gap-2"><span class="w-2.5 h-2.5 rounded-full bg-[var(--c-accent)]"></span> <span class="text-[var(--c-text-secondary)]">Community</span></div>
              </div>
            </div>

            <div class="mb-2">
              <div class="text-[var(--c-text-muted)] uppercase tracking-wider mb-1">Edges</div>
              <div class="space-y-1">
                <div class="flex items-center gap-2"><span class="w-4 h-0.5 rounded" style="background: #6366f1"></span> <span class="text-[var(--c-text-secondary)]">Calls</span></div>
                <div class="flex items-center gap-2"><span class="w-4 h-0.5 rounded" style="background: #14b8a6"></span> <span class="text-[var(--c-text-secondary)]">Imports</span></div>
                <div class="flex items-center gap-2"><span class="w-4 h-0.5 rounded" style="background: #f59e0b"></span> <span class="text-[var(--c-text-secondary)]">Extends</span></div>
                <div class="flex items-center gap-2"><span class="w-4 h-0.5 rounded" style="background: #ec4899"></span> <span class="text-[var(--c-text-secondary)]">Implements</span></div>
              </div>
            </div>

            <div>
              <div class="text-[var(--c-text-muted)] uppercase tracking-wider mb-1">Colors</div>
              <div class="text-[var(--c-text-secondary)]">Left stripe = community color</div>
              <div class="flex flex-wrap gap-1 mt-1">
                {#each communities.slice(0, 8) as c}
                  <span class="w-3 h-3 rounded-sm" style="background: {c.color}" title={c.label}></span>
                {/each}
                {#if communities.length > 8}
                  <span class="text-[var(--c-text-muted)]">+{communities.length - 8}</span>
                {/if}
              </div>
            </div>

            <div class="mt-2 pt-2 border-t border-[var(--c-border)]">
              <div class="text-[var(--c-text-muted)] uppercase tracking-wider mb-1">Interactions</div>
              <div class="space-y-0.5 text-[var(--c-text-secondary)]">
                <div>Click &middot; expand / select</div>
                <div>Double-click &middot; deep dive</div>
                <div>Right-click &middot; actions menu</div>
                <div>Hover &middot; highlight connections</div>
              </div>
            </div>
          </div>
        {/if}
      {/if}
    </div>

    <!-- Detail panel -->
    {#if detailOpen && detailNode}
      <SchematicDetailTabs
        node={detailNode}
        detail={detailData}
        loading={detailLoading}
        {annotations}
        annotationsLoading={annotationsLoading}
        onclose={() => detailOpen = false}
        onnavigate={handleNavigate}
        onannotationcreate={handleAnnotationCreate}
        onannotationdelete={handleAnnotationDelete}
        onannotationflag={handleAnnotationFlag}
      />
    {/if}
  </div>
</div>

<!-- Tooltip -->
<SchematicTooltip
  node={hoveredNodeId ? allNodes.get(hoveredNodeId) ?? null : null}
  x={tooltipX} y={tooltipY}
  {communities}
/>

<!-- Source popup -->
{#if sourcePopupNode}
  <SchematicSourcePopup
    node={sourcePopupNode}
    x={sourcePopupX} y={sourcePopupY}
    onclose={() => sourcePopupNode = null}
    onviewfull={() => { if (sourcePopupNode?.file_id) goto(`/explore/tree?fileId=${sourcePopupNode.file_id}`); sourcePopupNode = null; }}
  />
{/if}

<!-- Context menu -->
{#if contextMenuNode}
  <SchematicContextMenu
    node={contextMenuNode}
    x={contextMenuX} y={contextMenuY}
    onaction={handleContextAction}
    onclose={() => contextMenuNode = null}
  />
{/if}

<!-- Keyboard shortcuts help -->
<SchematicKeyboardOverlay visible={showKeyboardHelp} onclose={() => showKeyboardHelp = false} />
