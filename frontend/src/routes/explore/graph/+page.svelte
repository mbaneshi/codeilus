<script lang="ts">
  import {
    fetchGraph, fetchCommunities, fetchChapters, fetchFiles, fetchProgress,
    fetchNarrativeByTarget, fetchAnnotations, createAnnotation, updateAnnotation,
    toggleAnnotationFlag, deleteAnnotation,
  } from '$lib/api';
  import type { GraphNode, GraphEdge, Community, Chapter, FileRow, Progress, Annotation } from '$lib/types';
  import Markdown from '$lib/Markdown.svelte';
  import {
    ArrowLeft, ArrowRight, BookOpen, Code2, FileCode, Filter, GitBranch, Layers,
    Maximize2, Minimize2, Search, X, ChevronDown, ChevronRight,
    Circle, Square, Diamond, Triangle, Hexagon, Info,
    Flag, MessageSquare, Pencil, Trash2, Plus, Play, Pause,
    GraduationCap, Sparkles, Eye,
  } from 'lucide-svelte';

  const COMMUNITY_COLORS = [
    '#6366f1', '#ec4899', '#14b8a6', '#f59e0b', '#ef4444',
    '#8b5cf6', '#06b6d4', '#84cc16', '#f97316', '#a855f7',
    '#22d3ee', '#4ade80', '#fb923c', '#e879f9', '#2dd4bf',
  ];

  const EDGE_COLORS: Record<string, { color: string; label: string; dash?: string }> = {
    'CALLS':      { color: '#6366f1', label: 'Calls' },
    'IMPORTS':    { color: '#14b8a6', label: 'Imports' },
    'EXTENDS':    { color: '#f59e0b', label: 'Extends' },
    'IMPLEMENTS': { color: '#ec4899', label: 'Implements' },
    'CONTAINS':   { color: '#4b5563', label: 'Contains', dash: '4,2' },
  };

  const KIND_ICONS: Record<string, { label: string; shape: string }> = {
    'function': { label: 'Function', shape: 'circle' },
    'method':   { label: 'Method', shape: 'circle' },
    'class':    { label: 'Class', shape: 'diamond' },
    'struct':   { label: 'Struct', shape: 'square' },
    'enum':     { label: 'Enum', shape: 'hexagon' },
    'trait':    { label: 'Trait', shape: 'triangle' },
    'interface':{ label: 'Interface', shape: 'triangle' },
    'impl':     { label: 'Impl', shape: 'square' },
    'module':   { label: 'Module', shape: 'hexagon' },
    'constant': { label: 'Constant', shape: 'square' },
  };

  const DIFFICULTY_COLORS: Record<string, { bg: string; text: string; border: string }> = {
    beginner:     { bg: 'rgba(52,211,153,0.15)', text: '#34d399', border: 'rgba(52,211,153,0.3)' },
    intermediate: { bg: 'rgba(251,191,36,0.15)', text: '#fbbf24', border: 'rgba(251,191,36,0.3)' },
    advanced:     { bg: 'rgba(248,113,113,0.15)', text: '#f87171', border: 'rgba(248,113,113,0.3)' },
  };

  // ── Core state ──
  let loading = $state(true);
  let error = $state<string | null>(null);
  let allNodes = $state<GraphNode[]>([]);
  let allEdges = $state<GraphEdge[]>([]);
  let communities = $state<Community[]>([]);
  let chapters = $state<Chapter[]>([]);
  let files = $state<FileRow[]>([]);
  let progress = $state<Progress[]>([]);
  let annotations = $state<Annotation[]>([]);
  let selectedCommunity = $state<number | null>(null);
  let selectedKinds = $state<Set<string>>(new Set());
  let selectedNode = $state<GraphNode | null>(null);
  let containerEl: HTMLDivElement | undefined = $state();
  let graph3d: any = $state(null);
  let initialized = false;
  let isFullscreen = $state(false);
  let showSidebar = $state(true);
  let searchQuery = $state('');
  let highlightedNodes = $state<Set<number>>(new Set());
  let sidebarTab = $state<'communities' | 'legend' | 'filter' | 'notes'>('communities');

  // ── Feature 1: Narrative overlay ──
  let nodeNarrative = $state<string | null>(null);
  let narrativeLoading = $state(false);
  let narrativeCache = new Map<string, string | null>();

  // ── Feature 2: Progress indicators ──
  let communityProgress = $derived.by(() => {
    const map = new Map<number, 'completed' | 'in-progress' | 'unvisited'>();
    for (const ch of chapters) {
      if (ch.community_id === null) continue;
      const sectionCount = ch.sections.length;
      if (sectionCount === 0) { map.set(ch.community_id, 'unvisited'); continue; }
      const completed = progress.filter(p => p.chapter_id === ch.id && p.completed).length;
      if (completed >= sectionCount) map.set(ch.community_id, 'completed');
      else if (completed > 0) map.set(ch.community_id, 'in-progress');
      else map.set(ch.community_id, 'unvisited');
    }
    return map;
  });

  // ── Feature 3: Guided tour ──
  let tourMode = $state(false);
  let tourStep = $state(0);
  let tourNarrative = $state<string | null>(null);
  let tourNarrativeLoading = $state(false);

  let tourStops = $derived.by(() => {
    return chapters
      .filter(ch => ch.community_id !== null)
      .sort((a, b) => a.order_index - b.order_index);
  });

  // ── Feature 4: Difficulty badges ──
  let nodeDifficulty = $derived.by(() => {
    const map = new Map<number, string>();
    for (const ch of chapters) {
      if (ch.community_id === null) continue;
      const comm = communities.find(c => c.id === ch.community_id);
      if (!comm) continue;
      for (const memberId of comm.members) {
        map.set(memberId, ch.difficulty);
      }
    }
    return map;
  });

  // ── Feature 5: Learn modal ──
  let showLearnModal = $state(false);
  let learnModalChapter = $state<Chapter | null>(null);

  // ── Feature 7: Community card hover ──
  let hoveredCommunity = $state<number | null>(null);
  let communityNarrativeCache = new Map<number, string | null>();
  let hoveredCommunityNarrative = $state<string | null>(null);
  let communityCardTimeout: ReturnType<typeof setTimeout> | null = null;

  // ── Feature 8: Annotations ──
  let showAnnotationInput = $state(false);
  let annotationText = $state('');
  let editingAnnotationId = $state<number | null>(null);
  let editAnnotationText = $state('');

  // Annotation map for quick lookup
  let annotationMap = $derived.by(() => {
    const map = new Map<string, Annotation[]>();
    for (const a of annotations) {
      const key = `${a.target_type}:${a.target_id}`;
      if (!map.has(key)) map.set(key, []);
      map.get(key)!.push(a);
    }
    return map;
  });

  let annotatedNodeIds = $derived(new Set(
    annotations.filter(a => a.target_type === 'node').map(a => a.target_id)
  ));

  // ── Derived data ──
  let allKinds = $derived([...new Set(allNodes.map(n => n.kind))].sort());

  let filteredData = $derived.by(() => {
    let nodes = allNodes;
    let edges = allEdges;

    if (selectedCommunity !== null) {
      nodes = nodes.filter(n => n.community_id === selectedCommunity);
    }
    if (selectedKinds.size > 0) {
      nodes = nodes.filter(n => selectedKinds.has(n.kind));
    }
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      nodes = nodes.filter(n => n.name.toLowerCase().includes(q));
    }

    const nodeIds = new Set(nodes.map(n => n.id));
    edges = allEdges.filter(e => nodeIds.has(e.source_id) && nodeIds.has(e.target_id));
    return { nodes, edges };
  });

  let multiMemberCommunities = $derived(
    communities.filter(c => c.member_count > 1).sort((a, b) => b.member_count - a.member_count)
  );

  let edgeCounts = $derived.by(() => {
    const counts = new Map<number, number>();
    for (const e of allEdges) {
      counts.set(e.source_id, (counts.get(e.source_id) ?? 0) + 1);
      counts.set(e.target_id, (counts.get(e.target_id) ?? 0) + 1);
    }
    return counts;
  });

  let edgeTypeCounts = $derived.by(() => {
    const counts = new Map<string, number>();
    for (const e of filteredData.edges) {
      counts.set(e.kind, (counts.get(e.kind) ?? 0) + 1);
    }
    return counts;
  });

  let stats = $derived({
    nodes: filteredData.nodes.length,
    edges: filteredData.edges.length,
    communities: new Set(filteredData.nodes.map(n => n.community_id).filter(c => c !== null)).size,
  });

  // Selected node connections
  let selectedConnections = $derived.by(() => {
    if (!selectedNode) return { callers: [], callees: [], imports: [], importedBy: [], extends_: [], implementedBy: [], other: [] };
    const nodeMap = new Map(allNodes.map(n => [n.id, n]));
    const callers: GraphNode[] = [];
    const callees: GraphNode[] = [];
    const imports: GraphNode[] = [];
    const importedBy: GraphNode[] = [];
    const extends_: GraphNode[] = [];
    const implementedBy: GraphNode[] = [];
    const other: GraphNode[] = [];

    for (const e of allEdges) {
      if (e.source_id === selectedNode.id) {
        const target = nodeMap.get(e.target_id);
        if (!target) continue;
        if (e.kind === 'CALLS') callees.push(target);
        else if (e.kind === 'IMPORTS') imports.push(target);
        else if (e.kind === 'EXTENDS') extends_.push(target);
        else other.push(target);
      } else if (e.target_id === selectedNode.id) {
        const source = nodeMap.get(e.source_id);
        if (!source) continue;
        if (e.kind === 'CALLS') callers.push(source);
        else if (e.kind === 'IMPORTS') importedBy.push(source);
        else if (e.kind === 'IMPLEMENTS') implementedBy.push(source);
        else other.push(source);
      }
    }
    return { callers, callees, imports, importedBy, extends_, implementedBy, other };
  });

  // Find linked chapter for selected node
  let linkedChapter = $derived.by(() => {
    if (!selectedNode || selectedNode.community_id === null) return null;
    return chapters.find(ch => ch.community_id === selectedNode!.community_id) ?? null;
  });

  // Find file for selected node
  let linkedFile = $derived.by(() => {
    if (!selectedNode) return null;
    return files.find(f => f.id === selectedNode!.file_id) ?? null;
  });

  // Node annotations
  let selectedNodeAnnotations = $derived.by(() => {
    if (!selectedNode) return [];
    return annotationMap.get(`node:${selectedNode.id}`) ?? [];
  });

  function formatLabel(label: string): string {
    return label.replace(/^cluster_/, '').replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase());
  }

  function getColor(communityId: number | null): string {
    return COMMUNITY_COLORS[(communityId ?? 0) % COMMUNITY_COLORS.length];
  }

  function getProgressColor(communityId: number | null): string {
    if (communityId === null) return getColor(communityId);
    const status = communityProgress.get(communityId);
    if (status === 'completed') return '#34d399';
    if (status === 'unvisited') return getColor(communityId) + '66'; // dimmed
    return getColor(communityId);
  }

  function getNodeSize(nodeId: number): number {
    const ec = edgeCounts.get(nodeId) ?? 0;
    return Math.min(14, Math.max(3, 3 + ec * 0.7));
  }

  function toggleFullscreen() {
    if (!containerEl) return;
    if (!document.fullscreenElement) {
      containerEl.requestFullscreen();
      isFullscreen = true;
    } else {
      document.exitFullscreen();
      isFullscreen = false;
    }
  }

  function toggleKind(kind: string) {
    const next = new Set(selectedKinds);
    if (next.has(kind)) next.delete(kind);
    else next.add(kind);
    selectedKinds = next;
  }

  function focusNode(node: GraphNode) {
    selectedNode = node;
    loadNodeNarrative(node.id);
    if (graph3d) {
      const gNodes = graph3d.graphData().nodes;
      const gNode = gNodes.find((n: any) => n.id === node.id);
      if (gNode) {
        const distance = 100;
        const distRatio = 1 + distance / Math.hypot(gNode.x || 1, gNode.y || 1, gNode.z || 1);
        graph3d.cameraPosition(
          { x: (gNode.x || 0) * distRatio, y: (gNode.y || 0) * distRatio, z: (gNode.z || 0) * distRatio },
          { x: gNode.x || 0, y: gNode.y || 0, z: gNode.z || 0 },
          1200
        );
      }
    }
  }

  function highlightConnected(nodeId: number) {
    const connected = new Set<number>([nodeId]);
    for (const e of allEdges) {
      if (e.source_id === nodeId) connected.add(e.target_id);
      if (e.target_id === nodeId) connected.add(e.source_id);
    }
    highlightedNodes = connected;
  }

  function clearHighlight() {
    highlightedNodes = new Set();
  }

  // ── Feature 1: Load narrative for node ──
  async function loadNodeNarrative(symbolId: number) {
    const cacheKey = `symbol:${symbolId}`;
    if (narrativeCache.has(cacheKey)) {
      nodeNarrative = narrativeCache.get(cacheKey)!;
      return;
    }
    nodeNarrative = null;
    narrativeLoading = true;
    try {
      const n = await fetchNarrativeByTarget('symbol_explanation', symbolId);
      const content = n?.content ?? null;
      narrativeCache.set(cacheKey, content);
      nodeNarrative = content;
    } catch {
      nodeNarrative = null;
    }
    narrativeLoading = false;
  }

  // ── Feature 3: Tour functions ──
  async function startTour() {
    if (tourStops.length === 0) return;
    tourMode = true;
    tourStep = 0;
    await navigateToTourStop(0);
  }

  function exitTour() {
    tourMode = false;
    tourNarrative = null;
    selectedCommunity = null;
    selectedNode = null;
    clearHighlight();
  }

  async function navigateToTourStop(step: number) {
    const stop = tourStops[step];
    if (!stop || stop.community_id === null) return;
    selectedCommunity = stop.community_id;
    selectedNode = null;
    clearHighlight();

    // Load community narrative
    tourNarrativeLoading = true;
    const cacheKey = stop.community_id;
    if (communityNarrativeCache.has(cacheKey)) {
      tourNarrative = communityNarrativeCache.get(cacheKey)!;
      tourNarrativeLoading = false;
    } else {
      try {
        const n = await fetchNarrativeByTarget('module_summary', stop.community_id);
        const content = n?.content ?? null;
        communityNarrativeCache.set(cacheKey, content);
        tourNarrative = content;
      } catch {
        tourNarrative = null;
      }
      tourNarrativeLoading = false;
    }

    // Focus camera on community centroid
    if (graph3d) {
      const gNodes = graph3d.graphData().nodes;
      const commNodes = gNodes.filter((n: any) => n.community_id === stop.community_id);
      if (commNodes.length > 0) {
        const cx = commNodes.reduce((s: number, n: any) => s + (n.x || 0), 0) / commNodes.length;
        const cy = commNodes.reduce((s: number, n: any) => s + (n.y || 0), 0) / commNodes.length;
        const cz = commNodes.reduce((s: number, n: any) => s + (n.z || 0), 0) / commNodes.length;
        const distance = 200;
        const distRatio = 1 + distance / Math.hypot(cx || 1, cy || 1, cz || 1);
        graph3d.cameraPosition(
          { x: cx * distRatio, y: cy * distRatio, z: cz * distRatio },
          { x: cx, y: cy, z: cz },
          1500
        );
      }
    }
  }

  async function tourNext() {
    if (tourStep < tourStops.length - 1) {
      tourStep++;
      await navigateToTourStop(tourStep);
    }
  }

  async function tourBack() {
    if (tourStep > 0) {
      tourStep--;
      await navigateToTourStop(tourStep);
    }
  }

  // ── Feature 5: Learn modal ──
  function openLearnModal() {
    if (!linkedChapter) return;
    learnModalChapter = linkedChapter;
    showLearnModal = true;
  }

  // ── Feature 7: Community card hover ──
  async function handleCommunityHover(commId: number) {
    if (communityCardTimeout) clearTimeout(communityCardTimeout);
    hoveredCommunity = commId;

    if (communityNarrativeCache.has(commId)) {
      hoveredCommunityNarrative = communityNarrativeCache.get(commId)!;
    } else {
      hoveredCommunityNarrative = null;
      try {
        const n = await fetchNarrativeByTarget('module_summary', commId);
        const content = n?.content ?? null;
        communityNarrativeCache.set(commId, content);
        if (hoveredCommunity === commId) {
          hoveredCommunityNarrative = content;
        }
      } catch { /* ignore */ }
    }
  }

  function handleCommunityLeave() {
    communityCardTimeout = setTimeout(() => {
      hoveredCommunity = null;
      hoveredCommunityNarrative = null;
    }, 300);
  }

  function keepCommunityCard() {
    if (communityCardTimeout) clearTimeout(communityCardTimeout);
  }

  // ── Feature 8: Annotation CRUD ──
  async function handleCreateAnnotation() {
    if (!selectedNode || !annotationText.trim()) return;
    const result = await createAnnotation('node', selectedNode.id, annotationText.trim());
    if (result) {
      annotations = [result, ...annotations];
      annotationText = '';
      showAnnotationInput = false;
    }
  }

  async function handleDeleteAnnotation(id: number) {
    const ok = await deleteAnnotation(id);
    if (ok) {
      annotations = annotations.filter(a => a.id !== id);
    }
  }

  async function handleToggleFlag(id: number) {
    const result = await toggleAnnotationFlag(id);
    if (result) {
      annotations = annotations.map(a => a.id === id ? { ...a, flagged: result.flagged } : a);
    }
  }

  async function handleUpdateAnnotation(id: number) {
    if (!editAnnotationText.trim()) return;
    const ok = await updateAnnotation(id, editAnnotationText.trim());
    if (ok) {
      annotations = annotations.map(a => a.id === id ? { ...a, content: editAnnotationText.trim() } : a);
      editingAnnotationId = null;
      editAnnotationText = '';
    }
  }

  // ── Load all data ──
  if (typeof window !== 'undefined') {
    Promise.all([fetchGraph(), fetchCommunities(), fetchChapters(), fetchFiles(), fetchProgress(), fetchAnnotations()])
      .then(([graph, comms, chs, fls, prog, annots]) => {
        allNodes = graph.nodes;
        allEdges = graph.edges;
        communities = comms.sort((a, b) => b.member_count - a.member_count);
        chapters = chs;
        files = fls;
        progress = prog;
        annotations = annots;
        loading = false;
      })
      .catch(e => {
        error = `Failed to load graph data: ${e}`;
        loading = false;
      });
  }

  // ── Initialize 3D graph ──
  $effect(() => {
    if (loading || !containerEl || initialized || allNodes.length === 0) return;
    initialized = true;

    import('3d-force-graph').then(({ default: ForceGraph3D }) => {
      const nodeMap = new Map(allNodes.map(n => [n.id, n]));

      const gData = {
        nodes: allNodes.map(n => ({
          id: n.id,
          name: n.name,
          kind: n.kind,
          community_id: n.community_id,
          file_id: n.file_id,
          val: getNodeSize(n.id),
        })),
        links: allEdges.map(e => ({
          source: e.source_id,
          target: e.target_id,
          kind: e.kind,
          confidence: e.confidence,
        })),
      };

      const fg = ForceGraph3D()(containerEl!)
        .backgroundColor('#0a0a1a')
        .graphData(gData)
        .nodeLabel((node: any) => {
          const ec = edgeCounts.get(node.id) ?? 0;
          const comm = communities.find(c => c.id === node.community_id);
          const commLabel = comm ? formatLabel(comm.label) : 'Uncategorized';
          const diff = nodeDifficulty.get(node.id);
          const diffBadge = diff ? `<span style="font-size:9px;padding:1px 5px;border-radius:3px;background:${DIFFICULTY_COLORS[diff]?.bg ?? 'transparent'};color:${DIFFICULTY_COLORS[diff]?.text ?? '#999'};border:1px solid ${DIFFICULTY_COLORS[diff]?.border ?? 'transparent'}">${diff}</span>` : '';
          const progressStatus = communityProgress.get(node.community_id);
          const progressBadge = progressStatus === 'completed' ? '<span style="font-size:9px;color:#34d399">&#10003; learned</span>' : progressStatus === 'in-progress' ? '<span style="font-size:9px;color:#fbbf24">&#9679; in progress</span>' : '';
          const hasNotes = annotatedNodeIds.has(node.id);
          const notesBadge = hasNotes ? '<span style="font-size:9px;color:#a78bfa">&#9998; has notes</span>' : '';
          return `<div style="font-family:ui-monospace,monospace;font-size:12px;padding:8px 12px;background:rgba(10,10,26,0.95);border:1px solid rgba(255,255,255,0.15);border-radius:8px;max-width:300px;box-shadow:0 8px 32px rgba(0,0,0,0.5)">
            <div style="font-weight:700;font-size:13px;margin-bottom:4px;color:#f1f5f9">${node.name}</div>
            <div style="display:flex;gap:6px;align-items:center;margin-bottom:4px;flex-wrap:wrap">
              <span style="font-size:10px;padding:2px 6px;border-radius:4px;background:${getColor(node.community_id)}22;color:${getColor(node.community_id)};border:1px solid ${getColor(node.community_id)}44">${node.kind}</span>
              ${diffBadge}
              <span style="font-size:10px;color:#94a3b8">${ec} connections</span>
            </div>
            <div style="font-size:10px;color:#64748b;display:flex;gap:8px;align-items:center;flex-wrap:wrap">
              <span><span style="display:inline-block;width:6px;height:6px;border-radius:50%;background:${getColor(node.community_id)};margin-right:4px;vertical-align:middle"></span>${commLabel}</span>
              ${progressBadge}
              ${notesBadge}
            </div>
            <div style="font-size:10px;color:#475569;margin-top:4px;border-top:1px solid rgba(255,255,255,0.06);padding-top:4px">Click to explore details</div>
          </div>`;
        })
        .nodeColor((node: any) => {
          if (highlightedNodes.size > 0 && !highlightedNodes.has(node.id)) {
            return 'rgba(100,100,100,0.2)';
          }
          return getProgressColor(node.community_id);
        })
        .nodeVal((node: any) => node.val)
        .nodeOpacity(0.9)
        // Feature 6: Semantic edge labels with confidence tooltips
        .linkLabel((link: any) => {
          const sNode = typeof link.source === 'object' ? link.source : nodeMap.get(link.source);
          const tNode = typeof link.target === 'object' ? link.target : nodeMap.get(link.target);
          const sourceName = sNode?.name ?? link.source;
          const targetName = tNode?.name ?? link.target;
          const edgeInfo = EDGE_COLORS[link.kind] ?? { label: link.kind, color: '#4b5563' };
          const confidence = Math.round((link.confidence ?? 1) * 100);
          const confColor = confidence >= 80 ? '#34d399' : confidence >= 50 ? '#fbbf24' : '#f87171';
          return `<div style="font-family:ui-monospace,monospace;font-size:11px;padding:6px 10px;background:rgba(10,10,26,0.95);border:1px solid ${edgeInfo.color}44;border-radius:6px;max-width:320px;box-shadow:0 4px 16px rgba(0,0,0,0.4)">
            <div style="margin-bottom:3px"><span style="color:${edgeInfo.color};font-weight:600">${edgeInfo.label}</span></div>
            <div style="font-size:10px;color:#94a3b8;margin-bottom:2px">${sourceName} <span style="color:${edgeInfo.color}">&rarr;</span> ${targetName}</div>
            <div style="font-size:9px"><span style="color:${confColor}">${confidence}%</span> <span style="color:#475569">confidence</span></div>
          </div>`;
        })
        .linkColor((link: any) => {
          const edgeInfo = EDGE_COLORS[link.kind];
          if (highlightedNodes.size > 0) {
            if (highlightedNodes.has(link.source?.id ?? link.source) && highlightedNodes.has(link.target?.id ?? link.target)) {
              return edgeInfo?.color ?? '#4b5563';
            }
            return 'rgba(50,50,50,0.1)';
          }
          return edgeInfo?.color ?? '#4b5563';
        })
        .linkOpacity(0.5)
        .linkWidth((link: any) => {
          if (highlightedNodes.size > 0) {
            const sId = link.source?.id ?? link.source;
            const tId = link.target?.id ?? link.target;
            if (highlightedNodes.has(sId) && highlightedNodes.has(tId)) return 1.5;
            return 0.2;
          }
          // Feature 6: Confidence-based width
          const conf = link.confidence ?? 1;
          const base = link.kind === 'CALLS' ? 0.8 : 0.5;
          return base * (0.5 + conf * 0.5);
        })
        .linkDirectionalParticles((link: any) => link.kind === 'CALLS' ? 2 : link.kind === 'IMPORTS' ? 1 : 0)
        .linkDirectionalParticleWidth(1.5)
        .linkDirectionalParticleSpeed(0.004)
        .linkDirectionalParticleColor((link: any) => EDGE_COLORS[link.kind]?.color ?? '#6366f1')
        .linkDirectionalArrowLength(3)
        .linkDirectionalArrowRelPos(1)
        .linkDirectionalArrowColor((link: any) => EDGE_COLORS[link.kind]?.color ?? '#4b5563');

      fg.d3Force('charge')?.strength(-150);
      fg.d3Force('link')?.distance(60);

      // Add text labels with difficulty badges as sprites
      try {
        fg.nodeThreeObjectExtend(true)
          .nodeThreeObject((node: any) => {
            try {
              const THREE = (window as any).THREE;
              if (!THREE) return null;
              const ec = edgeCounts.get(node.id) ?? 0;
              if (ec < 1 && allNodes.length > 50) return null;
              const canvas = document.createElement('canvas');
              const ctx = canvas.getContext('2d');
              if (!ctx) return null;
              canvas.width = 512;
              canvas.height = 80;

              // Label text
              ctx.font = 'bold 28px ui-monospace, monospace';
              ctx.textAlign = 'center';
              ctx.textBaseline = 'middle';
              const label = node.name.length > 24 ? node.name.slice(0, 22) + '..' : node.name;

              // Draw text with outline
              ctx.strokeStyle = 'rgba(10,10,26,0.8)';
              ctx.lineWidth = 4;

              // Feature 2: Progress-aware text color
              const pStatus = communityProgress.get(node.community_id);
              if (pStatus === 'completed') {
                ctx.fillStyle = '#34d399';
              } else if (pStatus === 'unvisited') {
                ctx.fillStyle = '#64748b';
              } else {
                ctx.fillStyle = '#e2e8f0';
              }

              ctx.strokeText(label, 256, 28);
              ctx.fillText(label, 256, 28);

              // Feature 4: Difficulty badge below label
              const diff = nodeDifficulty.get(node.id);
              if (diff) {
                const colors = DIFFICULTY_COLORS[diff];
                if (colors) {
                  ctx.font = 'bold 16px ui-monospace, monospace';
                  const badgeText = diff.charAt(0).toUpperCase();
                  ctx.fillStyle = colors.text;
                  ctx.fillText(badgeText, 256, 60);
                }
              }

              // Feature 8: Annotation indicator
              if (annotatedNodeIds.has(node.id)) {
                ctx.font = 'bold 18px ui-monospace, monospace';
                ctx.fillStyle = '#a78bfa';
                ctx.fillText('\u270E', 420, 28);
              }

              const texture = new THREE.CanvasTexture(canvas);
              texture.needsUpdate = true;
              const material = new THREE.SpriteMaterial({ map: texture, transparent: true, depthWrite: false });
              const sprite = new THREE.Sprite(material);
              const scale = Math.max(20, 16 + ec * 2);
              sprite.scale.set(scale, scale * 80 / 512, 1);
              sprite.position.set(0, node.val + 5, 0);
              return sprite;
            } catch {
              return null;
            }
          });
      } catch {
        // Labels unavailable
      }

      fg.onNodeClick((node: any) => {
        const graphNode = nodeMap.get(node.id) ?? null;
        selectedNode = graphNode;
        if (graphNode) {
          highlightConnected(graphNode.id);
          loadNodeNarrative(graphNode.id);
        }
        showAnnotationInput = false;
        annotationText = '';

        const distance = 100;
        const distRatio = 1 + distance / Math.hypot(node.x, node.y, node.z);
        fg.cameraPosition(
          { x: node.x * distRatio, y: node.y * distRatio, z: node.z * distRatio },
          { x: node.x, y: node.y, z: node.z },
          1200
        );
      });

      fg.onBackgroundClick(() => {
        selectedNode = null;
        clearHighlight();
        showAnnotationInput = false;
      });

      graph3d = fg;

      const observer = new ResizeObserver(() => {
        if (containerEl) {
          fg.width(containerEl.clientWidth);
          fg.height(containerEl.clientHeight);
        }
      });
      observer.observe(containerEl!);
      return () => observer.disconnect();
    });
  });

  // Update graph data when filters change
  $effect(() => {
    if (!graph3d || loading) return;
    const data = filteredData;
    graph3d.graphData({
      nodes: data.nodes.map((n: GraphNode) => ({
        id: n.id,
        name: n.name,
        kind: n.kind,
        community_id: n.community_id,
        file_id: n.file_id,
        val: getNodeSize(n.id),
      })),
      links: data.edges.map((e: GraphEdge) => ({
        source: e.source_id,
        target: e.target_id,
        kind: e.kind,
        confidence: e.confidence,
      })),
    });
  });
</script>

<div class="graph-page" bind:this={containerEl}>
  {#if loading}
    <div class="absolute inset-0 flex items-center justify-center z-30 bg-[#0a0a1a]">
      <div class="text-center">
        <div class="w-12 h-12 rounded-full border-2 border-indigo-500 border-t-transparent animate-spin mx-auto mb-4"></div>
        <p class="text-gray-400 text-lg font-medium">Building knowledge graph...</p>
        <p class="text-gray-600 text-sm mt-1">Mapping symbols, edges & communities</p>
      </div>
    </div>
  {:else if error}
    <div class="absolute inset-0 flex items-center justify-center z-30 bg-[#0a0a1a]">
      <div class="text-center max-w-sm">
        <p class="text-red-400 text-lg mb-2">Failed to load graph</p>
        <p class="text-gray-500 text-sm">{error}</p>
      </div>
    </div>
  {:else if allNodes.length === 0}
    <div class="absolute inset-0 flex items-center justify-center z-30 bg-[#0a0a1a]">
      <div class="text-center">
        <p class="text-gray-400 text-lg mb-2">No graph data</p>
        <p class="text-gray-500">Run <code class="text-indigo-400 font-mono">codeilus analyze ./repo</code> first</p>
      </div>
    </div>
  {/if}

  <!-- LEFT SIDEBAR -->
  {#if !loading && allNodes.length > 0}
    <div class="sidebar {showSidebar ? 'open' : 'closed'}">
      <button
        class="sidebar-toggle"
        onclick={() => showSidebar = !showSidebar}
        title={showSidebar ? 'Collapse sidebar' : 'Expand sidebar'}
      >
        {#if showSidebar}
          <ChevronRight size={14} />
        {:else}
          <ChevronDown size={14} />
        {/if}
      </button>

      {#if showSidebar}
        <div class="sidebar-header">
          <a href="/explore" class="back-btn" title="Back to Explore">
            <ArrowLeft size={14} />
          </a>
          <h1 class="text-sm font-semibold text-white flex-1">Knowledge Graph</h1>
          <!-- Feature 3: Tour button -->
          {#if tourStops.length > 0}
            <button
              class="tour-btn"
              onclick={() => tourMode ? exitTour() : startTour()}
              title={tourMode ? 'Exit tour' : 'Start guided tour'}
            >
              {#if tourMode}
                <Pause size={12} />
              {:else}
                <Play size={12} />
              {/if}
              <span class="text-[10px]">{tourMode ? 'Exit' : 'Tour'}</span>
            </button>
          {/if}
        </div>

        <div class="stats-bar">
          <div class="stat">
            <span class="stat-value">{stats.nodes}</span>
            <span class="stat-label">nodes</span>
          </div>
          <div class="stat">
            <span class="stat-value">{stats.edges}</span>
            <span class="stat-label">edges</span>
          </div>
          <div class="stat">
            <span class="stat-value">{stats.communities}</span>
            <span class="stat-label">modules</span>
          </div>
        </div>

        <div class="search-box">
          <Search size={13} class="text-gray-500" />
          <input
            type="text"
            placeholder="Search symbols..."
            bind:value={searchQuery}
            class="search-input"
          />
          {#if searchQuery}
            <button class="text-gray-500 hover:text-gray-300" onclick={() => searchQuery = ''}>
              <X size={12} />
            </button>
          {/if}
        </div>

        <!-- Tab buttons -->
        <div class="tab-bar">
          <button class="tab-btn {sidebarTab === 'communities' ? 'active' : ''}" onclick={() => sidebarTab = 'communities'}>
            <Layers size={12} />
            Modules
          </button>
          <button class="tab-btn {sidebarTab === 'legend' ? 'active' : ''}" onclick={() => sidebarTab = 'legend'}>
            <Info size={12} />
            Legend
          </button>
          <button class="tab-btn {sidebarTab === 'filter' ? 'active' : ''}" onclick={() => sidebarTab = 'filter'}>
            <Filter size={12} />
            Filter
          </button>
          <button class="tab-btn {sidebarTab === 'notes' ? 'active' : ''}" onclick={() => sidebarTab = 'notes'}>
            <MessageSquare size={12} />
            Notes
          </button>
        </div>

        <div class="tab-content">
          {#if sidebarTab === 'communities'}
            <!-- Community list -->
            <button
              class="comm-item {selectedCommunity === null ? 'active' : ''}"
              onclick={() => { selectedCommunity = null; selectedNode = null; clearHighlight(); }}
            >
              <span class="comm-dot" style="background: linear-gradient(135deg, #6366f1, #ec4899, #14b8a6)"></span>
              <span class="comm-name">All modules</span>
              <span class="comm-count">{allNodes.length}</span>
            </button>

            {#each multiMemberCommunities as comm}
              {@const chapter = chapters.find(ch => ch.community_id === comm.id)}
              {@const pStatus = communityProgress.get(comm.id)}
              <div
                class="comm-item-wrapper"
                onmouseenter={() => handleCommunityHover(comm.id)}
                onmouseleave={handleCommunityLeave}
              >
                <button
                  class="comm-item {selectedCommunity === comm.id ? 'active' : ''}"
                  onclick={() => { selectedCommunity = selectedCommunity === comm.id ? null : comm.id; selectedNode = null; clearHighlight(); }}
                >
                  <span class="comm-dot" style="background: {COMMUNITY_COLORS[comm.id % COMMUNITY_COLORS.length]}">
                    {#if pStatus === 'completed'}
                      <span class="comm-check">&#10003;</span>
                    {/if}
                  </span>
                  <div class="comm-info">
                    <span class="comm-name">{formatLabel(comm.label)}</span>
                    <div class="comm-meta">
                      {#if chapter}
                        <a
                          href="/learn/{chapter.id}"
                          class="comm-chapter-link"
                          onclick={(e: MouseEvent) => e.stopPropagation()}
                        >
                          <BookOpen size={10} />
                          Ch.{chapter.order_index + 1}
                        </a>
                      {/if}
                      {#if chapter}
                        <span class="comm-difficulty" style="color: {DIFFICULTY_COLORS[chapter.difficulty]?.text ?? '#999'}">{chapter.difficulty}</span>
                      {/if}
                      {#if pStatus === 'completed'}
                        <span class="comm-progress-badge completed">done</span>
                      {:else if pStatus === 'in-progress'}
                        <span class="comm-progress-badge in-progress">learning</span>
                      {/if}
                    </div>
                  </div>
                  <span class="comm-count">{comm.member_count}</span>
                </button>
              </div>
            {/each}

          {:else if sidebarTab === 'legend'}
            <div class="legend-section">
              <h3 class="legend-title">Edge Types</h3>
              <p class="legend-desc">Lines between nodes show how symbols relate</p>
              {#each Object.entries(EDGE_COLORS) as [kind, info]}
                {@const count = edgeTypeCounts.get(kind) ?? 0}
                {#if count > 0}
                  <div class="legend-item">
                    <span class="legend-line" style="background: {info.color}"></span>
                    <span class="legend-label">{info.label}</span>
                    <span class="legend-count">{count}</span>
                  </div>
                {/if}
              {/each}
            </div>

            <div class="legend-section">
              <h3 class="legend-title">Symbol Types</h3>
              <p class="legend-desc">Each node represents a code symbol</p>
              {#each allKinds as kind}
                {@const info = KIND_ICONS[kind.toLowerCase()] ?? { label: kind, shape: 'circle' }}
                {@const count = filteredData.nodes.filter(n => n.kind === kind).length}
                <div class="legend-item">
                  <span class="legend-kind-dot"></span>
                  <span class="legend-label">{info.label}</span>
                  <span class="legend-count">{count}</span>
                </div>
              {/each}
            </div>

            <!-- Feature 2 & 4: Progress & Difficulty legend -->
            <div class="legend-section">
              <h3 class="legend-title">Learning Progress</h3>
              <div class="legend-item">
                <span class="legend-kind-dot" style="background: #34d399"></span>
                <span class="legend-label">Completed (green text)</span>
              </div>
              <div class="legend-item">
                <span class="legend-kind-dot" style="background: #e2e8f0"></span>
                <span class="legend-label">In progress / not started</span>
              </div>
              <div class="legend-item">
                <span class="legend-kind-dot" style="background: #64748b"></span>
                <span class="legend-label">Unvisited (dimmed)</span>
              </div>
            </div>

            <div class="legend-section">
              <h3 class="legend-title">Difficulty Badges</h3>
              {#each Object.entries(DIFFICULTY_COLORS) as [level, colors]}
                <div class="legend-item">
                  <span class="legend-kind-dot" style="background: {colors.text}"></span>
                  <span class="legend-label" style="color: {colors.text}">{level.charAt(0).toUpperCase() + level.slice(1)}</span>
                </div>
              {/each}
            </div>

            <div class="legend-section">
              <h3 class="legend-title">Edge Confidence</h3>
              <p class="legend-desc">Brighter, thicker edges = higher confidence match. Hover edges for details.</p>
            </div>

            <div class="legend-section">
              <h3 class="legend-title">Colors = Modules</h3>
              <p class="legend-desc">Nodes with the same color belong to the same functional module (detected via community analysis)</p>
            </div>

            <div class="legend-section">
              <h3 class="legend-title">How to Navigate</h3>
              <div class="guide-list">
                <div class="guide-item"><kbd>Click</kbd> node to see details, narrative & annotations</div>
                <div class="guide-item"><kbd>Drag</kbd> to rotate the view</div>
                <div class="guide-item"><kbd>Scroll</kbd> to zoom in/out</div>
                <div class="guide-item"><kbd>Right-drag</kbd> to pan</div>
                <div class="guide-item">Use <strong>Tour</strong> button for guided learning</div>
                <div class="guide-item">Hover <strong>modules</strong> for summary cards</div>
                <div class="guide-item">Click <strong>Learn This</strong> in detail panel for modal</div>
              </div>
            </div>

          {:else if sidebarTab === 'filter'}
            <div class="legend-section">
              <h3 class="legend-title">Filter by Symbol Type</h3>
              <p class="legend-desc">Show only specific types of symbols</p>
              {#each allKinds as kind}
                {@const count = allNodes.filter(n => n.kind === kind).length}
                <button
                  class="filter-chip {selectedKinds.has(kind) ? 'active' : ''}"
                  onclick={() => toggleKind(kind)}
                >
                  <span class="filter-chip-label">{kind}</span>
                  <span class="filter-chip-count">{count}</span>
                </button>
              {/each}
              {#if selectedKinds.size > 0}
                <button class="clear-filter-btn" onclick={() => selectedKinds = new Set()}>
                  Clear filters
                </button>
              {/if}
            </div>

            {#if selectedCommunity !== null || selectedKinds.size > 0 || searchQuery}
              <div class="legend-section">
                <h3 class="legend-title">Active Filters</h3>
                <div class="active-filters">
                  {#if selectedCommunity !== null}
                    {@const comm = communities.find(c => c.id === selectedCommunity)}
                    <span class="active-filter-tag">
                      Module: {comm ? formatLabel(comm.label) : selectedCommunity}
                      <button onclick={() => { selectedCommunity = null; }}>
                        <X size={10} />
                      </button>
                    </span>
                  {/if}
                  {#each [...selectedKinds] as kind}
                    <span class="active-filter-tag">
                      {kind}
                      <button onclick={() => toggleKind(kind)}>
                        <X size={10} />
                      </button>
                    </span>
                  {/each}
                  {#if searchQuery}
                    <span class="active-filter-tag">
                      Search: {searchQuery}
                      <button onclick={() => searchQuery = ''}>
                        <X size={10} />
                      </button>
                    </span>
                  {/if}
                </div>
              </div>
            {/if}

          {:else if sidebarTab === 'notes'}
            <!-- Feature 8: Notes tab -->
            <div class="legend-section">
              <h3 class="legend-title">Your Annotations</h3>
              <p class="legend-desc">Notes pinned to graph nodes</p>
            </div>

            {#if annotations.length === 0}
              <div class="notes-empty">
                <MessageSquare size={24} class="text-gray-600 mx-auto mb-2" />
                <p class="text-xs text-gray-500">No annotations yet. Click a node and add a note.</p>
              </div>
            {:else}
              <!-- Flagged first -->
              {#each annotations.filter(a => a.flagged) as ann}
                {@const node = allNodes.find(n => n.id === ann.target_id)}
                <div class="note-item flagged">
                  <div class="note-header">
                    <Flag size={10} class="text-amber-400" />
                    <button class="note-node-name" onclick={() => node && focusNode(node)}>
                      {node?.name ?? `#${ann.target_id}`}
                    </button>
                  </div>
                  <p class="note-content">{ann.content}</p>
                </div>
              {/each}
              {#each annotations.filter(a => !a.flagged) as ann}
                {@const node = allNodes.find(n => n.id === ann.target_id)}
                <div class="note-item">
                  <div class="note-header">
                    <MessageSquare size={10} class="text-gray-500" />
                    <button class="note-node-name" onclick={() => node && focusNode(node)}>
                      {node?.name ?? `#${ann.target_id}`}
                    </button>
                  </div>
                  <p class="note-content">{ann.content}</p>
                </div>
              {/each}
            {/if}
          {/if}
        </div>
      {/if}
    </div>

    <!-- Feature 7: Community hover card -->
    {#if hoveredCommunity !== null && showSidebar}
      {@const comm = communities.find(c => c.id === hoveredCommunity)}
      {@const chapter = chapters.find(ch => ch.community_id === hoveredCommunity)}
      {@const pStatus = communityProgress.get(hoveredCommunity)}
      {#if comm}
        <div
          class="community-card"
          onmouseenter={keepCommunityCard}
          onmouseleave={handleCommunityLeave}
        >
          <div class="community-card-header" style="border-color: {getColor(hoveredCommunity)}">
            <span class="comm-dot" style="background: {getColor(hoveredCommunity)}"></span>
            <span class="community-card-title">{formatLabel(comm.label)}</span>
          </div>

          <div class="community-card-stats">
            <span>{comm.member_count} symbols</span>
            <span>cohesion: {(comm.cohesion * 100).toFixed(0)}%</span>
          </div>

          {#if chapter}
            <div class="community-card-difficulty">
              <span style="color: {DIFFICULTY_COLORS[chapter.difficulty]?.text ?? '#999'}">{chapter.difficulty}</span>
              {#if pStatus === 'completed'}
                <span class="text-emerald-400">completed</span>
              {:else if pStatus === 'in-progress'}
                <span class="text-amber-400">in progress</span>
              {:else}
                <span class="text-gray-500">not started</span>
              {/if}
            </div>
          {/if}

          {#if hoveredCommunityNarrative}
            <p class="community-card-narrative">{hoveredCommunityNarrative.slice(0, 200)}{hoveredCommunityNarrative.length > 200 ? '...' : ''}</p>
          {/if}

          <div class="community-card-actions">
            <button class="cc-action" onclick={() => { selectedCommunity = hoveredCommunity; hoveredCommunity = null; }}>
              <Eye size={12} /> Focus
            </button>
            {#if chapter}
              <a href="/learn/{chapter.id}" class="cc-action">
                <BookOpen size={12} /> Learn
              </a>
            {/if}
          </div>
        </div>
      {/if}
    {/if}

    <!-- Fullscreen toggle -->
    <button
      class="fullscreen-btn"
      onclick={toggleFullscreen}
      title={isFullscreen ? 'Exit fullscreen' : 'Fullscreen'}
    >
      {#if isFullscreen}
        <Minimize2 size={16} />
      {:else}
        <Maximize2 size={16} />
      {/if}
    </button>

    <!-- RIGHT DETAIL PANEL -->
    {#if selectedNode && !tourMode}
      <div class="detail-panel">
        <div class="detail-header">
          <div class="detail-title-row">
            <span class="detail-kind-badge" style="background: {getColor(selectedNode.community_id)}22; color: {getColor(selectedNode.community_id)}; border-color: {getColor(selectedNode.community_id)}44">
              {selectedNode.kind}
            </span>
            <!-- Feature 4: Difficulty badge -->
            {#if nodeDifficulty.get(selectedNode.id)}
              {@const diff = nodeDifficulty.get(selectedNode.id)!}
              <span class="detail-diff-badge" style="background: {DIFFICULTY_COLORS[diff]?.bg}; color: {DIFFICULTY_COLORS[diff]?.text}; border-color: {DIFFICULTY_COLORS[diff]?.border}">
                {diff}
              </span>
            {/if}
            <button
              class="detail-close"
              onclick={() => { selectedNode = null; clearHighlight(); showAnnotationInput = false; }}
            >
              <X size={16} />
            </button>
          </div>
          <h2 class="detail-name">{selectedNode.name}</h2>
          <!-- Feature 2: Progress status -->
          {#if communityProgress.get(selectedNode.community_id ?? -1)}
            {@const pStatus = communityProgress.get(selectedNode.community_id!)}
            <div class="detail-progress-badge {pStatus}">
              {#if pStatus === 'completed'}
                <GraduationCap size={12} /> Learned
              {:else if pStatus === 'in-progress'}
                <Sparkles size={12} /> Learning
              {:else}
                New
              {/if}
            </div>
          {/if}
        </div>

        <!-- Quick info cards -->
        <div class="detail-cards">
          {#if linkedFile}
            <a href="/explore/tree" class="detail-card clickable" title="View in file tree">
              <FileCode size={14} class="text-emerald-400" />
              <div class="detail-card-content">
                <span class="detail-card-label">File</span>
                <span class="detail-card-value">{linkedFile.path.split('/').pop()}</span>
                <span class="detail-card-sub">{linkedFile.path}</span>
              </div>
            </a>
          {/if}

          {#if selectedNode.community_id !== null}
            {@const comm = communities.find(c => c.id === selectedNode?.community_id)}
            <div class="detail-card">
              <span class="detail-card-dot" style="background: {getColor(selectedNode.community_id)}"></span>
              <div class="detail-card-content">
                <span class="detail-card-label">Module</span>
                <span class="detail-card-value">{comm ? formatLabel(comm.label) : `Community ${selectedNode.community_id}`}</span>
              </div>
            </div>
          {/if}

          <div class="detail-card">
            <GitBranch size={14} class="text-indigo-400" />
            <div class="detail-card-content">
              <span class="detail-card-label">Connections</span>
              <span class="detail-card-value">{edgeCounts.get(selectedNode.id) ?? 0} total</span>
            </div>
          </div>
        </div>

        <!-- Feature 5: Learn link / Learn This modal button -->
        {#if linkedChapter}
          <div class="learn-actions">
            <a href="/learn/{linkedChapter.id}" class="learn-link">
              <BookOpen size={16} />
              <div>
                <div class="learn-link-title">Chapter {linkedChapter.order_index + 1}: {formatLabel(linkedChapter.title)}</div>
                <div class="learn-link-sub">Open full learning material</div>
              </div>
              <ChevronRight size={14} class="ml-auto text-gray-500" />
            </a>
            <button class="learn-modal-btn" onclick={openLearnModal}>
              <GraduationCap size={14} />
              Learn This
            </button>
          </div>
        {/if}

        <!-- Feature 1: Narrative overlay -->
        <div class="detail-narrative">
          {#if narrativeLoading}
            <div class="narrative-loading">
              <div class="w-4 h-4 rounded-full border border-indigo-500 border-t-transparent animate-spin"></div>
              <span class="text-[10px] text-gray-500">Loading explanation...</span>
            </div>
          {:else if nodeNarrative}
            <div class="narrative-card">
              <h3 class="narrative-title">
                <Sparkles size={12} class="text-indigo-400" />
                AI Explanation
              </h3>
              <div class="narrative-body">
                <Markdown content={nodeNarrative} />
              </div>
            </div>
          {/if}
        </div>

        <!-- Feature 8: Annotations section -->
        <div class="detail-annotations">
          <div class="annotations-header">
            <h3 class="conn-label">
              <MessageSquare size={12} class="text-purple-400" />
              Notes
              {#if selectedNodeAnnotations.length > 0}
                <span class="conn-count">{selectedNodeAnnotations.length}</span>
              {/if}
            </h3>
            <button class="add-note-btn" onclick={() => { showAnnotationInput = !showAnnotationInput; }}>
              <Plus size={12} /> Add
            </button>
          </div>

          {#if showAnnotationInput}
            <div class="annotation-input-area">
              <textarea
                bind:value={annotationText}
                placeholder="Add a note about this symbol..."
                class="annotation-textarea"
                rows="3"
              ></textarea>
              <div class="annotation-input-actions">
                <button class="ann-cancel" onclick={() => { showAnnotationInput = false; annotationText = ''; }}>Cancel</button>
                <button class="ann-save" onclick={handleCreateAnnotation} disabled={!annotationText.trim()}>Save</button>
              </div>
            </div>
          {/if}

          {#each selectedNodeAnnotations as ann}
            <div class="annotation-item {ann.flagged ? 'flagged' : ''}">
              {#if editingAnnotationId === ann.id}
                <textarea
                  bind:value={editAnnotationText}
                  class="annotation-textarea"
                  rows="2"
                ></textarea>
                <div class="annotation-input-actions">
                  <button class="ann-cancel" onclick={() => { editingAnnotationId = null; }}>Cancel</button>
                  <button class="ann-save" onclick={() => handleUpdateAnnotation(ann.id)}>Update</button>
                </div>
              {:else}
                <p class="annotation-text">{ann.content}</p>
                <div class="annotation-actions">
                  <button onclick={() => handleToggleFlag(ann.id)} title={ann.flagged ? 'Unflag' : 'Flag for review'}>
                    <Flag size={11} class={ann.flagged ? 'text-amber-400' : 'text-gray-600'} />
                  </button>
                  <button onclick={() => { editingAnnotationId = ann.id; editAnnotationText = ann.content; }} title="Edit">
                    <Pencil size={11} class="text-gray-600 hover:text-gray-300" />
                  </button>
                  <button onclick={() => handleDeleteAnnotation(ann.id)} title="Delete">
                    <Trash2 size={11} class="text-gray-600 hover:text-red-400" />
                  </button>
                </div>
              {/if}
            </div>
          {/each}
        </div>

        <!-- Connections -->
        <div class="detail-connections">
          {#if selectedConnections.callees.length > 0}
            <div class="conn-group">
              <h3 class="conn-label">
                <span class="conn-edge-dot" style="background: {EDGE_COLORS.CALLS.color}"></span>
                Calls <span class="conn-count">{selectedConnections.callees.length}</span>
              </h3>
              {#each selectedConnections.callees as n}
                <button class="conn-item" onclick={() => focusNode(n)}>
                  <span class="conn-node-dot" style="background: {getColor(n.community_id)}"></span>
                  <span class="conn-node-name">{n.name}</span>
                  <span class="conn-node-kind">{n.kind}</span>
                </button>
              {/each}
            </div>
          {/if}

          {#if selectedConnections.callers.length > 0}
            <div class="conn-group">
              <h3 class="conn-label">
                <span class="conn-edge-dot" style="background: {EDGE_COLORS.CALLS.color}"></span>
                Called by <span class="conn-count">{selectedConnections.callers.length}</span>
              </h3>
              {#each selectedConnections.callers as n}
                <button class="conn-item" onclick={() => focusNode(n)}>
                  <span class="conn-node-dot" style="background: {getColor(n.community_id)}"></span>
                  <span class="conn-node-name">{n.name}</span>
                  <span class="conn-node-kind">{n.kind}</span>
                </button>
              {/each}
            </div>
          {/if}

          {#if selectedConnections.imports.length > 0}
            <div class="conn-group">
              <h3 class="conn-label">
                <span class="conn-edge-dot" style="background: {EDGE_COLORS.IMPORTS.color}"></span>
                Imports <span class="conn-count">{selectedConnections.imports.length}</span>
              </h3>
              {#each selectedConnections.imports as n}
                <button class="conn-item" onclick={() => focusNode(n)}>
                  <span class="conn-node-dot" style="background: {getColor(n.community_id)}"></span>
                  <span class="conn-node-name">{n.name}</span>
                  <span class="conn-node-kind">{n.kind}</span>
                </button>
              {/each}
            </div>
          {/if}

          {#if selectedConnections.importedBy.length > 0}
            <div class="conn-group">
              <h3 class="conn-label">
                <span class="conn-edge-dot" style="background: {EDGE_COLORS.IMPORTS.color}"></span>
                Imported by <span class="conn-count">{selectedConnections.importedBy.length}</span>
              </h3>
              {#each selectedConnections.importedBy as n}
                <button class="conn-item" onclick={() => focusNode(n)}>
                  <span class="conn-node-dot" style="background: {getColor(n.community_id)}"></span>
                  <span class="conn-node-name">{n.name}</span>
                  <span class="conn-node-kind">{n.kind}</span>
                </button>
              {/each}
            </div>
          {/if}

          {#if selectedConnections.extends_.length > 0}
            <div class="conn-group">
              <h3 class="conn-label">
                <span class="conn-edge-dot" style="background: {EDGE_COLORS.EXTENDS.color}"></span>
                Extends <span class="conn-count">{selectedConnections.extends_.length}</span>
              </h3>
              {#each selectedConnections.extends_ as n}
                <button class="conn-item" onclick={() => focusNode(n)}>
                  <span class="conn-node-dot" style="background: {getColor(n.community_id)}"></span>
                  <span class="conn-node-name">{n.name}</span>
                  <span class="conn-node-kind">{n.kind}</span>
                </button>
              {/each}
            </div>
          {/if}

          {#if selectedConnections.implementedBy.length > 0}
            <div class="conn-group">
              <h3 class="conn-label">
                <span class="conn-edge-dot" style="background: {EDGE_COLORS.IMPLEMENTS.color}"></span>
                Implemented by <span class="conn-count">{selectedConnections.implementedBy.length}</span>
              </h3>
              {#each selectedConnections.implementedBy as n}
                <button class="conn-item" onclick={() => focusNode(n)}>
                  <span class="conn-node-dot" style="background: {getColor(n.community_id)}"></span>
                  <span class="conn-node-name">{n.name}</span>
                  <span class="conn-node-kind">{n.kind}</span>
                </button>
              {/each}
            </div>
          {/if}

          {#if selectedConnections.other.length > 0}
            <div class="conn-group">
              <h3 class="conn-label">
                Other <span class="conn-count">{selectedConnections.other.length}</span>
              </h3>
              {#each selectedConnections.other as n}
                <button class="conn-item" onclick={() => focusNode(n)}>
                  <span class="conn-node-dot" style="background: {getColor(n.community_id)}"></span>
                  <span class="conn-node-name">{n.name}</span>
                  <span class="conn-node-kind">{n.kind}</span>
                </button>
              {/each}
            </div>
          {/if}

          {#if (edgeCounts.get(selectedNode.id) ?? 0) === 0}
            <p class="text-xs text-gray-500 italic">This symbol has no connections in the graph.</p>
          {/if}
        </div>
      </div>
    {/if}

    <!-- Feature 3: Tour overlay panel -->
    {#if tourMode && tourStops.length > 0}
      {@const stop = tourStops[tourStep]}
      <div class="tour-overlay">
        <div class="tour-header">
          <span class="tour-step-badge">
            {tourStep + 1} / {tourStops.length}
          </span>
          <button class="tour-exit-btn" onclick={exitTour}>
            <X size={14} /> Exit Tour
          </button>
        </div>

        <h3 class="tour-chapter-title">
          <span class="comm-dot" style="background: {getColor(stop.community_id)}"></span>
          Chapter {stop.order_index + 1}: {formatLabel(stop.title)}
        </h3>

        {#if stop.difficulty}
          <span class="tour-difficulty" style="color: {DIFFICULTY_COLORS[stop.difficulty]?.text}; background: {DIFFICULTY_COLORS[stop.difficulty]?.bg}">
            {stop.difficulty}
          </span>
        {/if}

        {#if stop.description}
          <p class="tour-desc">{stop.description}</p>
        {/if}

        {#if tourNarrativeLoading}
          <div class="tour-narrative-loading">
            <div class="w-3 h-3 rounded-full border border-indigo-500 border-t-transparent animate-spin"></div>
            <span>Loading...</span>
          </div>
        {:else if tourNarrative}
          <div class="tour-narrative">
            <Markdown content={tourNarrative} />
          </div>
        {/if}

        <div class="tour-nav">
          <button
            class="tour-nav-btn"
            onclick={tourBack}
            disabled={tourStep === 0}
          >
            <ArrowLeft size={14} /> Back
          </button>
          {#if stop.community_id !== null}
            <a href="/learn/{stop.id}" class="tour-learn-btn">
              <BookOpen size={14} /> Open Chapter
            </a>
          {/if}
          <button
            class="tour-nav-btn"
            onclick={tourNext}
            disabled={tourStep === tourStops.length - 1}
          >
            Next <ArrowRight size={14} />
          </button>
        </div>
      </div>
    {/if}

    <!-- BOTTOM GUIDANCE BAR -->
    {#if !tourMode}
      <div class="guidance-bar">
        <div class="guidance-content">
          <span class="guidance-icon"><Info size={14} /></span>
          <span>
            <strong>Nodes</strong> = code symbols &nbsp;|&nbsp;
            <strong>Colors</strong> = functional modules &nbsp;|&nbsp;
            <strong>Lines</strong> = relationships
            (<span style="color: {EDGE_COLORS.CALLS.color}">calls</span>,
             <span style="color: {EDGE_COLORS.IMPORTS.color}">imports</span>,
             <span style="color: {EDGE_COLORS.EXTENDS.color}">extends</span>)
            &nbsp;|&nbsp; Click any node for details
          </span>
        </div>
      </div>
    {/if}
  {/if}
</div>

<!-- Feature 5: Learn This modal -->
{#if showLearnModal && learnModalChapter}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4">
    <button class="absolute inset-0 bg-black/60 backdrop-blur-sm" onclick={() => showLearnModal = false} aria-label="Close modal"></button>
    <div class="relative bg-[var(--surface-1)] border border-[var(--c-border)] rounded-2xl p-6 w-full max-w-lg shadow-2xl max-h-[80vh] overflow-auto">
      <button
        class="absolute top-4 right-4 text-[var(--c-text-muted)] hover:text-[var(--c-text-primary)] transition-colors"
        onclick={() => showLearnModal = false}
      >
        <X size={18} />
      </button>

      <div class="flex items-center gap-3 mb-4">
        <div class="w-10 h-10 rounded-xl bg-[var(--c-accent)]/10 flex items-center justify-center text-[var(--c-accent)] font-bold text-sm shrink-0">
          {learnModalChapter.order_index + 1}
        </div>
        <div>
          <h2 class="text-lg font-bold">{formatLabel(learnModalChapter.title)}</h2>
          <div class="flex items-center gap-2 mt-0.5">
            <span class="text-xs font-medium px-2 py-0.5 rounded" style="color: {DIFFICULTY_COLORS[learnModalChapter.difficulty]?.text}; background: {DIFFICULTY_COLORS[learnModalChapter.difficulty]?.bg}">
              {learnModalChapter.difficulty}
            </span>
            <span class="text-xs text-[var(--c-text-muted)]">{learnModalChapter.sections.length} sections</span>
          </div>
        </div>
      </div>

      {#if learnModalChapter.description}
        <p class="text-sm text-[var(--c-text-secondary)] mb-4">{learnModalChapter.description}</p>
      {/if}

      {#if learnModalChapter.narrative}
        <div class="mb-4 bg-[var(--surface-2)] rounded-lg p-4">
          <h3 class="text-xs font-semibold text-[var(--c-text-muted)] uppercase tracking-wider mb-2">Module Summary</h3>
          <Markdown content={learnModalChapter.narrative} />
        </div>
      {/if}

      {#if learnModalChapter.sections.find(s => s.kind === 'key_concepts')?.content}
        <div class="mb-4 bg-[var(--surface-2)] rounded-lg p-4">
          <h3 class="text-xs font-semibold text-[var(--c-text-muted)] uppercase tracking-wider mb-2">Key Concepts</h3>
          <Markdown content={learnModalChapter.sections.find(s => s.kind === 'key_concepts')?.content ?? ''} />
        </div>
      {/if}

      <a
        href="/learn/{learnModalChapter.id}"
        class="block text-center py-2.5 bg-[var(--c-accent)] text-white rounded-lg text-sm font-medium hover:bg-[var(--c-accent)]/80 transition-colors"
      >
        Open Full Chapter
      </a>
    </div>
  </div>
{/if}

<style>
  @reference "tailwindcss";

  .graph-page {
    @apply relative w-full h-full overflow-hidden;
    background: #0a0a1a;
  }

  /* -- Sidebar -- */
  .sidebar {
    @apply absolute top-0 left-0 h-full z-20 flex flex-col;
    background: rgba(10, 10, 26, 0.92);
    backdrop-filter: blur(16px);
    border-right: 1px solid rgba(255, 255, 255, 0.08);
    transition: width 0.2s ease;
  }
  .sidebar.open { width: 280px; }
  .sidebar.closed { width: 36px; }

  .sidebar-toggle {
    @apply absolute top-3 right-[-14px] w-7 h-7 rounded-full flex items-center justify-center z-30;
    background: rgba(30, 30, 50, 0.95);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: #94a3b8;
    cursor: pointer;
    transition: all 0.15s;
  }
  .sidebar-toggle:hover { color: white; border-color: rgba(255, 255, 255, 0.2); }

  .sidebar-header { @apply flex items-center gap-2 px-4 pt-4 pb-2; }

  .back-btn {
    @apply w-7 h-7 rounded-lg flex items-center justify-center text-gray-400 hover:text-white transition-colors;
    background: rgba(255, 255, 255, 0.06);
  }
  .back-btn:hover { background: rgba(255, 255, 255, 0.12); }

  /* Tour button */
  .tour-btn {
    @apply flex items-center gap-1 px-2.5 py-1 rounded-lg text-[10px] font-semibold transition-colors;
    background: rgba(99, 102, 241, 0.15);
    color: #818cf8;
    border: 1px solid rgba(99, 102, 241, 0.25);
  }
  .tour-btn:hover {
    background: rgba(99, 102, 241, 0.25);
  }

  /* Stats */
  .stats-bar { @apply flex items-center gap-1 px-4 py-2; }
  .stat {
    @apply flex-1 text-center py-1.5 rounded-lg;
    background: rgba(255, 255, 255, 0.04);
  }
  .stat-value { @apply block text-sm font-bold text-white; }
  .stat-label { @apply block text-[10px] text-gray-500; }

  /* Search */
  .search-box {
    @apply flex items-center gap-2 mx-4 my-2 px-3 py-1.5 rounded-lg;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.08);
  }
  .search-input { @apply flex-1 bg-transparent text-sm text-gray-200 outline-none placeholder:text-gray-600; }

  /* Tabs */
  .tab-bar {
    @apply flex items-center gap-0.5 mx-4 my-1 p-0.5 rounded-lg;
    background: rgba(255, 255, 255, 0.04);
  }
  .tab-btn {
    @apply flex-1 flex items-center justify-center gap-1 py-1.5 rounded-md text-[11px] font-medium transition-colors;
    color: #64748b;
  }
  .tab-btn:hover { color: #94a3b8; }
  .tab-btn.active {
    background: rgba(99, 102, 241, 0.2);
    color: #818cf8;
  }
  .tab-content { @apply flex-1 overflow-auto px-3 py-2; }

  /* Community items */
  .comm-item-wrapper { @apply relative; }
  .comm-item {
    @apply flex items-center gap-2.5 w-full px-2.5 py-2 rounded-lg text-left transition-colors;
    cursor: pointer;
  }
  .comm-item:hover { background: rgba(255, 255, 255, 0.06); }
  .comm-item.active {
    background: rgba(99, 102, 241, 0.12);
    border: 1px solid rgba(99, 102, 241, 0.2);
  }
  .comm-dot { @apply w-3 h-3 rounded-full shrink-0 relative; }
  .comm-check {
    @apply absolute inset-0 flex items-center justify-center text-[8px] font-bold text-white;
  }
  .comm-info { @apply flex-1 min-w-0 flex flex-col; }
  .comm-name { @apply text-xs font-medium text-gray-300 truncate flex-1; }
  .comm-meta { @apply flex items-center gap-2 mt-0.5; }
  .comm-chapter-link {
    @apply inline-flex items-center gap-1 text-[10px] text-indigo-400 hover:text-indigo-300 transition-colors;
  }
  .comm-difficulty { @apply text-[9px] font-medium; }
  .comm-progress-badge {
    @apply text-[9px] font-medium px-1.5 py-0 rounded;
  }
  .comm-progress-badge.completed { background: rgba(52,211,153,0.15); color: #34d399; }
  .comm-progress-badge.in-progress { background: rgba(251,191,36,0.15); color: #fbbf24; }
  .comm-count { @apply text-[10px] text-gray-600 shrink-0 tabular-nums; }

  /* Legend */
  .legend-section { @apply mb-4; }
  .legend-title { @apply text-[11px] font-semibold text-gray-400 uppercase tracking-wider mb-1.5; }
  .legend-desc { @apply text-[10px] text-gray-600 mb-2 leading-relaxed; }
  .legend-item { @apply flex items-center gap-2 py-1; }
  .legend-line { @apply w-5 h-0.5 rounded-full shrink-0; }
  .legend-kind-dot { @apply w-2.5 h-2.5 rounded-full bg-gray-500 shrink-0; }
  .legend-label { @apply text-xs text-gray-300 flex-1; }
  .legend-count { @apply text-[10px] text-gray-600 tabular-nums; }

  .guide-list { @apply space-y-1.5; }
  .guide-item { @apply text-[11px] text-gray-400 leading-relaxed; }
  .guide-item kbd {
    @apply inline-block px-1.5 py-0.5 rounded text-[10px] font-mono;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: #94a3b8;
  }
  .guide-item strong { @apply font-semibold text-gray-300; }

  /* Filter */
  .filter-chip {
    @apply inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md text-xs mr-1 mb-1 transition-colors;
    background: rgba(255, 255, 255, 0.06);
    color: #94a3b8;
    border: 1px solid transparent;
  }
  .filter-chip:hover { background: rgba(255, 255, 255, 0.1); color: #e2e8f0; }
  .filter-chip.active {
    background: rgba(99, 102, 241, 0.15);
    color: #818cf8;
    border-color: rgba(99, 102, 241, 0.3);
  }
  .filter-chip-label { @apply font-medium; }
  .filter-chip-count { @apply text-[10px] text-gray-600; }
  .clear-filter-btn { @apply block mt-2 text-[11px] text-indigo-400 hover:text-indigo-300 transition-colors; }

  .active-filters { @apply flex flex-wrap gap-1; }
  .active-filter-tag {
    @apply inline-flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-medium;
    background: rgba(99, 102, 241, 0.12);
    color: #818cf8;
  }
  .active-filter-tag button { @apply hover:text-white transition-colors; cursor: pointer; }

  /* Notes tab */
  .notes-empty { @apply text-center py-6; }
  .note-item {
    @apply p-2.5 rounded-lg mb-2;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.06);
  }
  .note-item.flagged {
    background: rgba(251, 191, 36, 0.06);
    border-color: rgba(251, 191, 36, 0.15);
  }
  .note-header { @apply flex items-center gap-1.5 mb-1; }
  .note-node-name {
    @apply text-[11px] font-mono font-medium text-gray-300 hover:text-indigo-300 transition-colors truncate;
    cursor: pointer;
  }
  .note-content { @apply text-[10px] text-gray-400 leading-relaxed; }

  /* Community hover card */
  .community-card {
    @apply absolute z-30 w-[250px] rounded-xl p-4;
    left: 290px;
    top: 200px;
    background: rgba(10, 10, 26, 0.96);
    backdrop-filter: blur(16px);
    border: 1px solid rgba(255, 255, 255, 0.1);
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.6);
    animation: fadeIn 0.15s ease-out;
  }
  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(4px); }
    to { opacity: 1; transform: translateY(0); }
  }
  .community-card-header {
    @apply flex items-center gap-2 pb-2 mb-2;
    border-bottom: 2px solid;
  }
  .community-card-title { @apply text-sm font-semibold text-white; }
  .community-card-stats {
    @apply flex items-center gap-3 text-[10px] text-gray-400 mb-2;
  }
  .community-card-difficulty {
    @apply flex items-center gap-2 text-[10px] mb-2;
  }
  .community-card-narrative {
    @apply text-[11px] text-gray-400 leading-relaxed mb-3;
  }
  .community-card-actions { @apply flex items-center gap-2; }
  .cc-action {
    @apply flex items-center gap-1 px-2.5 py-1 rounded-md text-[10px] font-medium transition-colors;
    background: rgba(255, 255, 255, 0.06);
    color: #94a3b8;
  }
  .cc-action:hover { background: rgba(255, 255, 255, 0.12); color: white; }

  /* Fullscreen button */
  .fullscreen-btn {
    @apply absolute top-4 right-4 z-20 w-9 h-9 rounded-lg flex items-center justify-center text-gray-400 hover:text-white transition-all;
    background: rgba(10, 10, 26, 0.8);
    backdrop-filter: blur(8px);
    border: 1px solid rgba(255, 255, 255, 0.08);
  }
  .fullscreen-btn:hover { background: rgba(10, 10, 26, 0.95); border-color: rgba(255, 255, 255, 0.15); }

  /* Detail panel */
  .detail-panel {
    @apply absolute top-0 right-0 h-full z-20 overflow-auto;
    width: 360px;
    background: rgba(10, 10, 26, 0.95);
    backdrop-filter: blur(16px);
    border-left: 1px solid rgba(255, 255, 255, 0.08);
    animation: slideIn 0.2s ease-out;
  }
  @keyframes slideIn {
    from { transform: translateX(100%); opacity: 0; }
    to { transform: translateX(0); opacity: 1; }
  }

  .detail-header {
    @apply px-5 pt-5 pb-4;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }
  .detail-title-row { @apply flex items-center gap-2 mb-2; }
  .detail-kind-badge {
    @apply text-[11px] font-semibold px-2.5 py-1 rounded-md;
    border: 1px solid;
  }
  .detail-diff-badge {
    @apply text-[10px] font-semibold px-2 py-0.5 rounded-md;
    border: 1px solid;
  }
  .detail-close {
    @apply w-7 h-7 rounded-lg flex items-center justify-center text-gray-500 hover:text-white transition-colors ml-auto;
    background: rgba(255, 255, 255, 0.06);
  }
  .detail-name { @apply text-lg font-bold text-white font-mono break-all leading-tight; }

  /* Progress badge */
  .detail-progress-badge {
    @apply inline-flex items-center gap-1 text-[10px] font-medium px-2 py-0.5 rounded mt-2;
  }
  .detail-progress-badge.completed { background: rgba(52,211,153,0.15); color: #34d399; }
  .detail-progress-badge.in-progress { background: rgba(251,191,36,0.15); color: #fbbf24; }
  .detail-progress-badge.unvisited { background: rgba(100,100,120,0.15); color: #64748b; }

  /* Detail cards */
  .detail-cards {
    @apply px-5 py-3 space-y-2;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }
  .detail-card {
    @apply flex items-start gap-3 p-3 rounded-lg;
    background: rgba(255, 255, 255, 0.04);
  }
  .detail-card.clickable { @apply cursor-pointer transition-colors; }
  .detail-card.clickable:hover { background: rgba(255, 255, 255, 0.08); }
  .detail-card-dot { @apply w-3 h-3 rounded-full shrink-0 mt-0.5; }
  .detail-card-content { @apply flex flex-col min-w-0; }
  .detail-card-label { @apply text-[10px] text-gray-500 uppercase tracking-wider; }
  .detail-card-value { @apply text-xs font-medium text-gray-200; }
  .detail-card-sub { @apply text-[10px] text-gray-500 truncate; }

  /* Learn actions */
  .learn-actions {
    @apply px-5 py-3 space-y-2;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }
  .learn-link {
    @apply flex items-center gap-3 p-3 rounded-lg transition-colors;
    background: rgba(99, 102, 241, 0.08);
    border: 1px solid rgba(99, 102, 241, 0.15);
    color: #818cf8;
  }
  .learn-link:hover {
    background: rgba(99, 102, 241, 0.15);
    border-color: rgba(99, 102, 241, 0.3);
  }
  .learn-link-title { @apply text-xs font-semibold text-indigo-300; }
  .learn-link-sub { @apply text-[10px] text-indigo-400/60; }

  .learn-modal-btn {
    @apply w-full flex items-center justify-center gap-2 py-2 rounded-lg text-xs font-semibold transition-colors;
    background: rgba(99, 102, 241, 0.15);
    color: #818cf8;
    border: 1px solid rgba(99, 102, 241, 0.2);
  }
  .learn-modal-btn:hover {
    background: rgba(99, 102, 241, 0.25);
  }

  /* Narrative overlay */
  .detail-narrative {
    @apply px-5 py-2;
  }
  .narrative-loading {
    @apply flex items-center gap-2 py-3;
  }
  .narrative-card {
    @apply rounded-lg p-3;
    background: rgba(99, 102, 241, 0.06);
    border: 1px solid rgba(99, 102, 241, 0.1);
  }
  .narrative-title {
    @apply flex items-center gap-1.5 text-[11px] font-semibold text-indigo-300 mb-2;
  }
  .narrative-body {
    @apply text-[11px] leading-relaxed;
  }

  /* Annotations section */
  .detail-annotations {
    @apply px-5 py-3;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }
  .annotations-header {
    @apply flex items-center justify-between mb-2;
  }
  .add-note-btn {
    @apply flex items-center gap-1 text-[10px] font-medium text-indigo-400 hover:text-indigo-300 transition-colors;
  }
  .annotation-input-area { @apply mb-3; }
  .annotation-textarea {
    @apply w-full bg-transparent text-xs text-gray-300 p-2 rounded-lg outline-none resize-none;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.1);
  }
  .annotation-textarea:focus { border-color: rgba(99, 102, 241, 0.4); }
  .annotation-input-actions { @apply flex justify-end gap-2 mt-1.5; }
  .ann-cancel { @apply text-[10px] text-gray-500 hover:text-gray-300 transition-colors; }
  .ann-save {
    @apply text-[10px] font-medium px-2.5 py-1 rounded-md transition-colors;
    background: rgba(99, 102, 241, 0.2);
    color: #818cf8;
  }
  .ann-save:hover { background: rgba(99, 102, 241, 0.3); }
  .ann-save:disabled { opacity: 0.4; }
  .annotation-item {
    @apply p-2 rounded-lg mb-1.5;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.05);
  }
  .annotation-item.flagged {
    background: rgba(251, 191, 36, 0.05);
    border-color: rgba(251, 191, 36, 0.12);
  }
  .annotation-text { @apply text-[11px] text-gray-400 leading-relaxed mb-1; }
  .annotation-actions { @apply flex items-center gap-2; }
  .annotation-actions button {
    @apply p-1 rounded transition-colors;
    cursor: pointer;
  }
  .annotation-actions button:hover { background: rgba(255, 255, 255, 0.08); }

  /* Connections */
  .detail-connections { @apply px-5 py-3; }
  .conn-group { @apply mb-4; }
  .conn-label { @apply flex items-center gap-1.5 text-[11px] font-semibold text-gray-400 mb-1.5; }
  .conn-edge-dot { @apply w-2 h-2 rounded-full shrink-0; }
  .conn-count { @apply text-gray-600 font-normal; }
  .conn-item {
    @apply flex items-center gap-2 w-full py-1.5 px-2 rounded transition-colors text-left;
    cursor: pointer;
  }
  .conn-item:hover { background: rgba(255, 255, 255, 0.06); }
  .conn-node-dot { @apply w-2 h-2 rounded-full shrink-0; }
  .conn-node-name { @apply text-xs font-mono text-gray-300 truncate flex-1; }
  .conn-node-kind { @apply text-[10px] text-gray-600 shrink-0; }

  /* Tour overlay */
  .tour-overlay {
    @apply absolute bottom-6 left-1/2 z-30 w-[460px] rounded-2xl p-5;
    transform: translateX(-50%);
    background: rgba(10, 10, 26, 0.96);
    backdrop-filter: blur(20px);
    border: 1px solid rgba(99, 102, 241, 0.2);
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.6), 0 0 20px rgba(99, 102, 241, 0.1);
    animation: tourSlideUp 0.3s ease-out;
  }
  @keyframes tourSlideUp {
    from { transform: translateX(-50%) translateY(20px); opacity: 0; }
    to { transform: translateX(-50%) translateY(0); opacity: 1; }
  }
  .tour-header { @apply flex items-center justify-between mb-3; }
  .tour-step-badge {
    @apply text-xs font-bold px-2.5 py-1 rounded-md;
    background: rgba(99, 102, 241, 0.2);
    color: #818cf8;
  }
  .tour-exit-btn {
    @apply flex items-center gap-1 text-[11px] text-gray-500 hover:text-gray-300 transition-colors;
  }
  .tour-chapter-title {
    @apply flex items-center gap-2 text-base font-bold text-white mb-2;
  }
  .tour-difficulty {
    @apply inline-block text-[10px] font-semibold px-2 py-0.5 rounded mb-2;
  }
  .tour-desc {
    @apply text-xs text-gray-400 mb-3 leading-relaxed;
  }
  .tour-narrative-loading {
    @apply flex items-center gap-2 text-[10px] text-gray-500 py-2;
  }
  .tour-narrative {
    @apply max-h-[180px] overflow-auto text-[11px] mb-3 pr-1;
    scrollbar-width: thin;
    scrollbar-color: rgba(255,255,255,0.1) transparent;
  }
  .tour-nav { @apply flex items-center justify-between gap-2 pt-3 border-t border-white/6; }
  .tour-nav-btn {
    @apply flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-colors;
    background: rgba(255, 255, 255, 0.06);
    color: #94a3b8;
  }
  .tour-nav-btn:hover { background: rgba(255, 255, 255, 0.12); color: white; }
  .tour-nav-btn:disabled { opacity: 0.3; pointer-events: none; }
  .tour-learn-btn {
    @apply flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-colors;
    background: rgba(99, 102, 241, 0.15);
    color: #818cf8;
  }
  .tour-learn-btn:hover { background: rgba(99, 102, 241, 0.25); }

  /* Guidance bar */
  .guidance-bar {
    @apply absolute bottom-4 left-1/2 z-10 px-5 py-2.5 rounded-xl;
    transform: translateX(-50%);
    background: rgba(10, 10, 26, 0.85);
    backdrop-filter: blur(12px);
    border: 1px solid rgba(255, 255, 255, 0.08);
    max-width: 700px;
  }
  .guidance-content { @apply flex items-center gap-2 text-[11px] text-gray-400; }
  .guidance-content strong { @apply text-gray-300 font-semibold; }
  .guidance-icon { @apply text-indigo-400 shrink-0; }
</style>
