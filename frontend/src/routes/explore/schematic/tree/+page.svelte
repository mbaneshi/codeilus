<script lang="ts">
  import { fetchFiles, fetchFileSymbols, fetchNarrativeByTarget, fetchFileSource } from '$lib/api';
  import type { FileRow, SymbolRow, NarrativeResponse, SourceResponse } from '$lib/types';
  import { layoutTree, type LayoutNode, type LayoutEdge } from '$lib/schematic/layout';
  import { LoadingSpinner } from '$lib/components';

  const LANG_COLORS: Record<string, string> = {
    rust: '#dea584', python: '#3572a5', typescript: '#3178c6', javascript: '#f1e05a',
    go: '#00add8', java: '#b07219', svelte: '#ff3e00', css: '#563d7c',
    html: '#e34c26', json: '#292929', yaml: '#cb171e', toml: '#9c4221',
  };

  let loading = $state(true);
  let nodes = $state<LayoutNode[]>([]);
  let edges = $state<LayoutEdge[]>([]);
  let canvasW = $state(4000);
  let canvasH = $state(3000);
  let searchQuery = $state('');
  let highlighted = $state<Set<string>>(new Set());

  // Pan/zoom
  let tx = $state(20);
  let ty = $state(20);
  let scale = $state(0.7);
  let dragging = $state(false);
  let dragStart = { x: 0, y: 0, tx: 0, ty: 0 };

  // Modal
  let modalOpen = $state(false);
  let modalTitle = $state('');
  let modalFile = $state<FileRow | null>(null);
  let modalSymbols = $state<SymbolRow[]>([]);
  let modalNarrative = $state<NarrativeResponse | null>(null);
  let modalSource = $state<SourceResponse | null>(null);
  let modalLoading = $state(false);

  interface DirTree {
    id: string;
    label: string;
    data: Record<string, unknown>;
    children: DirTree[];
  }

  let idCounter = 0;
  function buildDirTree(fileList: FileRow[]): DirTree {
    const root: DirTree = { id: `d-${idCounter++}`, label: 'repo', data: { type: 'dir' }, children: [] };
    const dirMap = new Map<string, DirTree>();
    dirMap.set('', root);

    for (const file of fileList) {
      const clean = file.path.replace(/^\.\//, '');
      const parts = clean.split('/');
      let parentPath = '';
      let parent = root;

      for (let i = 0; i < parts.length - 1; i++) {
        const dirPath = parentPath ? `${parentPath}/${parts[i]}` : parts[i];
        if (!dirMap.has(dirPath)) {
          const dir: DirTree = { id: `d-${idCounter++}`, label: parts[i], data: { type: 'dir' }, children: [] };
          parent.children.push(dir);
          dirMap.set(dirPath, dir);
        }
        parent = dirMap.get(dirPath)!;
        parentPath = dirPath;
      }

      const fileName = parts[parts.length - 1];
      parent.children.push({
        id: `f-${file.id}`,
        label: fileName,
        data: { type: 'file', file, language: file.language, sloc: file.sloc },
        children: [],
      });
    }
    return root;
  }

  function flattenNodes(node: LayoutNode, result: LayoutNode[]): void {
    result.push(node);
    if (node.children) node.children.forEach(c => flattenNodes(c, result));
  }

  async function openFileModal(file: FileRow) {
    modalOpen = true;
    modalTitle = file.path.split('/').pop() || file.path;
    modalFile = file;
    modalSymbols = [];
    modalNarrative = null;
    modalSource = null;
    modalLoading = true;
    const [syms, narr, src] = await Promise.all([
      fetchFileSymbols(file.id),
      fetchNarrativeByTarget('file_overview', file.id),
      fetchFileSource(file.id, 1, 40),
    ]);
    modalSymbols = syms;
    modalNarrative = narr;
    modalSource = src;
    modalLoading = false;
  }

  function handleNodeClick(node: LayoutNode) {
    if (node.data.type === 'file' && node.data.file) {
      openFileModal(node.data.file as FileRow);
    }
  }

  function doSearch() {
    if (searchQuery.trim().length < 2) { highlighted = new Set(); return; }
    const q = searchQuery.toLowerCase();
    highlighted = new Set(nodes.filter(n => n.label.toLowerCase().includes(q)).map(n => n.id));
  }

  function onWheel(e: WheelEvent) {
    e.preventDefault();
    const factor = e.deltaY > 0 ? 0.92 : 1.08;
    scale = Math.max(0.05, Math.min(4, scale * factor));
  }

  function onPointerDown(e: PointerEvent) {
    if (e.button !== 0) return;
    dragging = true;
    dragStart = { x: e.clientX, y: e.clientY, tx, ty };
  }

  function onPointerMove(e: PointerEvent) {
    if (!dragging) return;
    tx = dragStart.tx + (e.clientX - dragStart.x);
    ty = dragStart.ty + (e.clientY - dragStart.y);
  }

  function onPointerUp() { dragging = false; }

  function edgePath(from: LayoutNode, to: LayoutNode): string {
    const x1 = from.x + from.width;
    const y1 = from.y + from.height / 2;
    const x2 = to.x;
    const y2 = to.y + to.height / 2;
    const mx = (x1 + x2) / 2;
    return `M ${x1} ${y1} C ${mx} ${y1}, ${mx} ${y2}, ${x2} ${y2}`;
  }

  fetchFiles().then(files => {
    if (files.length === 0) { loading = false; return; }
    idCounter = 0;
    const tree = buildDirTree(files);
    const result = layoutTree(tree);
    nodes = result.nodes;
    edges = result.edges;
    canvasW = result.width;
    canvasH = result.height;
    loading = false;
  });

  let nodeMap = $derived(new Map(nodes.map(n => [n.id, n])));
</script>

<div class="h-full flex flex-col">
  <!-- Header -->
  <div class="flex items-center gap-3 px-5 py-3 border-b border-[var(--c-border)] bg-[var(--surface-1)] shrink-0">
    <a href="/explore" class="p-1.5 rounded-lg hover:bg-[var(--surface-2)] text-[var(--c-text-muted)] hover:text-[var(--c-text-primary)] transition-colors text-sm">&larr;</a>
    <h1 class="text-base font-semibold">Codebase Tree</h1>
    <span class="text-xs text-[var(--c-text-muted)]">{nodes.length} nodes</span>
    <div class="ml-auto">
      <input
        type="text"
        placeholder="Search files..."
        class="bg-[var(--surface-2)] border border-[var(--c-border)] rounded-lg px-3 py-1 text-sm text-[var(--c-text-primary)] placeholder:text-[var(--c-text-muted)] focus:border-[var(--c-accent)] outline-none w-48"
        bind:value={searchQuery}
        oninput={doSearch}
      />
    </div>
  </div>

  <!-- Canvas -->
  <div class="flex-1 min-h-0 relative">
    {#if loading}
      <div class="flex items-center justify-center h-full"><LoadingSpinner /></div>
    {:else if nodes.length === 0}
      <div class="flex items-center justify-center h-full text-[var(--c-text-muted)]">No files found.</div>
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
          <!-- Edges -->
          {#each edges as edge}
            {@const from = nodeMap.get(edge.from)}
            {@const to = nodeMap.get(edge.to)}
            {#if from && to}
              <path d={edgePath(from, to)} fill="none" stroke="var(--c-border-hover)" stroke-width="1.2" marker-end="url(#ah)" opacity="0.5" />
            {/if}
          {/each}

          <!-- Nodes -->
          {#each nodes as node}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <g
              transform="translate({node.x},{node.y})"
              onclick={() => handleNodeClick(node)}
              style="cursor: {node.data.type === 'file' ? 'pointer' : 'default'}"
            >
              <rect
                width={node.width} height={node.height} rx="6"
                fill="var(--surface-1)"
                stroke={highlighted.has(node.id) ? 'var(--c-accent)' : 'var(--c-border)'}
                stroke-width={highlighted.has(node.id) ? 2 : 1}
              />
              {#if node.data.type === 'dir'}
                <text x="10" y="23" font-size="11" fill="var(--c-text-secondary)" font-family="var(--font-sans)">
                  📁 {node.label}
                </text>
              {:else}
                {@const lang = (node.data.language as string) || ''}
                {@const color = LANG_COLORS[lang.toLowerCase()] || 'var(--c-text-muted)'}
                <circle cx="10" cy="18" r="4" fill={color} />
                <text x="20" y="16" font-size="11" fill="var(--c-text-primary)" font-family="var(--font-mono)">{node.label}</text>
                <text x="20" y="30" font-size="9" fill="var(--c-text-muted)" font-family="var(--font-sans)">{lang} · {node.data.sloc} loc</text>
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
        {:else if modalFile}
          <div class="text-xs text-[var(--c-text-muted)] font-mono break-all">{modalFile.path}</div>
          <div class="flex gap-2 text-sm">
            {#if modalFile.language}
              <span class="px-2 py-0.5 rounded bg-[var(--c-accent)]/10 text-[var(--c-accent)] text-xs">{modalFile.language}</span>
            {/if}
            <span class="text-[var(--c-text-secondary)]">{modalFile.sloc} lines</span>
          </div>

          {#if modalNarrative?.content}
            <div>
              <h3 class="text-sm font-medium text-[var(--c-text-primary)] mb-1">Overview</h3>
              <p class="text-sm text-[var(--c-text-secondary)] leading-relaxed">{modalNarrative.content}</p>
            </div>
          {/if}

          {#if modalSymbols.length > 0}
            <div>
              <h3 class="text-sm font-medium text-[var(--c-text-primary)] mb-1">Symbols ({modalSymbols.length})</h3>
              <div class="space-y-1 max-h-48 overflow-auto">
                {#each modalSymbols as sym}
                  <div class="flex items-center gap-2 px-2 py-1.5 rounded bg-[var(--surface-2)] text-xs">
                    <span class="px-1 py-0.5 rounded bg-[var(--surface-3)] text-[var(--c-text-muted)] uppercase text-[10px]">{sym.kind}</span>
                    <span class="font-mono text-[var(--c-text-primary)] truncate">{sym.name}</span>
                    <span class="text-[var(--c-text-muted)] ml-auto">L{sym.start_line}</span>
                  </div>
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
