<script lang="ts">
  import { onMount } from 'svelte';
  import { fetchFiles, fetchFileSymbols, fetchNarrativeByTarget, fetchFileSource } from '$lib/api';
  import type { FileRow, SymbolRow, SourceResponse, NarrativeResponse } from '$lib/types';
  import type { SchematicNode, SchematicEdge } from '$lib/schematic/types';
  import { computeLayout, type LayoutResult } from '$lib/schematic/elk-layout';
  import { edgePathD } from '$lib/schematic/edge-path';
  import SchematicCanvas from '$lib/schematic/SchematicCanvas.svelte';
  import SchematicSearch from '$lib/schematic/SchematicSearch.svelte';
  import SchematicModal from '$lib/schematic/SchematicModal.svelte';
  import { LoadingSpinner } from '$lib/components';
  import { FolderTree, FileCode, ArrowLeft } from 'lucide-svelte';

  const LANG_COLORS: Record<string, string> = {
    rust: '#dea584', python: '#3572a5', typescript: '#3178c6', javascript: '#f1e05a',
    go: '#00add8', java: '#b07219', svelte: '#ff3e00', css: '#563d7c',
    html: '#e34c26', json: '#292929', yaml: '#cb171e', toml: '#9c4221',
    c: '#555555', cpp: '#f34b7d', tsx: '#3178c6', jsx: '#f1e05a',
  };

  let loading = $state(true);
  let layout = $state<LayoutResult | null>(null);
  let flatNodes = $state<SchematicNode[]>([]);
  let edges = $state<SchematicEdge[]>([]);
  let highlighted = $state<Set<string>>(new Set());
  let canvasRef: SchematicCanvas | undefined = $state();

  // Modal state
  let modalOpen = $state(false);
  let modalTitle = $state('');
  let modalFile = $state<FileRow | null>(null);
  let modalSymbols = $state<SymbolRow[]>([]);
  let modalNarrative = $state<NarrativeResponse | null>(null);
  let modalSource = $state<SourceResponse | null>(null);
  let modalLoading = $state(false);

  interface TreeDir {
    name: string;
    path: string;
    children: TreeDir[];
    files: FileRow[];
  }

  function buildDirTree(fileList: FileRow[]): TreeDir {
    const root: TreeDir = { name: 'repo', path: '', children: [], files: [] };
    for (const file of fileList) {
      const clean = file.path.replace(/^\.\//, '');
      const parts = clean.split('/');
      let current = root;
      for (let i = 0; i < parts.length - 1; i++) {
        let child = current.children.find(c => c.name === parts[i]);
        if (!child) {
          child = { name: parts[i], path: parts.slice(0, i + 1).join('/'), children: [], files: [] };
          current.children.push(child);
        }
        current = child;
      }
      current.files.push(file);
    }
    return root;
  }

  let nodeIdCounter = 0;
  function dirToSchematic(dir: TreeDir, parentId: string | null, nodes: SchematicNode[], edgeList: SchematicEdge[]): string {
    const id = `dir-${nodeIdCounter++}`;
    const fileCount = dir.files.length + dir.children.reduce((a, c) => a + countFiles(c), 0);
    nodes.push({
      id,
      label: dir.name || 'root',
      width: Math.max(120, (dir.name || 'root').length * 8 + 40),
      height: 40,
      metadata: { type: 'dir', fileCount, path: dir.path },
    });
    if (parentId) {
      edgeList.push({ id: `e-${parentId}-${id}`, source: parentId, target: id });
    }
    for (const child of dir.children) {
      dirToSchematic(child, id, nodes, edgeList);
    }
    for (const file of dir.files) {
      const fid = `file-${file.id}`;
      const fname = file.path.split('/').pop() || file.path;
      nodes.push({
        id: fid,
        label: fname,
        width: Math.max(140, fname.length * 7 + 60),
        height: 50,
        metadata: { type: 'file', file, language: file.language, sloc: file.sloc },
      });
      edgeList.push({ id: `e-${id}-${fid}`, source: id, target: fid });
    }
    return id;
  }

  function countFiles(dir: TreeDir): number {
    return dir.files.length + dir.children.reduce((a, c) => a + countFiles(c), 0);
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

  function handleNodeClick(nodeId: string) {
    const node = flatNodes.find(n => n.id === nodeId);
    if (!node) return;
    if (node.metadata.type === 'file' && node.metadata.file) {
      openFileModal(node.metadata.file as FileRow);
    }
  }

  function handleFocus(nodeId: string) {
    const pos = layout?.nodes.get(nodeId);
    if (pos && canvasRef) {
      canvasRef.zoomToNode(pos.x, pos.y, pos.width, pos.height);
    }
  }

  onMount(async () => {
    const files = await fetchFiles();
    if (files.length === 0) { loading = false; return; }

    const dirTree = buildDirTree(files);
    const nodeList: SchematicNode[] = [];
    const edgeList: SchematicEdge[] = [];
    nodeIdCounter = 0;
    dirToSchematic(dirTree, null, nodeList, edgeList);
    flatNodes = nodeList;
    edges = edgeList;

    layout = await computeLayout(nodeList, edgeList, {
      algorithm: 'mrtree',
      direction: 'RIGHT',
      nodeSpacing: 15,
      layerSpacing: 60,
    });

    loading = false;
    requestAnimationFrame(() => {
      if (canvasRef && layout) {
        canvasRef.fitToView(layout.width, layout.height);
      }
    });
  });
</script>

<div class="h-full flex flex-col">
  <div class="flex items-center gap-3 px-5 py-3 border-b border-[var(--c-border)] bg-[var(--surface-1)] shrink-0">
    <a href="/explore" class="p-1.5 rounded-lg hover:bg-[var(--surface-2)] text-[var(--c-text-muted)] hover:text-[var(--c-text-primary)] transition-colors">
      <ArrowLeft size={16} />
    </a>
    <FolderTree size={18} class="text-emerald-400" />
    <h1 class="text-base font-semibold">Codebase Tree</h1>
    <span class="text-xs text-[var(--c-text-muted)]">{flatNodes.length} nodes</span>
  </div>

  <div class="flex-1 min-h-0 relative">
    {#if loading}
      <div class="flex items-center justify-center h-full">
        <LoadingSpinner />
      </div>
    {:else if flatNodes.length === 0}
      <div class="flex items-center justify-center h-full text-[var(--c-text-muted)]">
        No files found. Analyze a codebase first.
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
          {#each edges as edge}
            {@const pts = layout?.edges.get(edge.id)?.points}
            {#if pts}
              <path
                d={edgePathD(pts)}
                fill="none"
                stroke="var(--c-border-hover)"
                stroke-width="1.5"
                marker-end="url(#arrowhead)"
                opacity="0.6"
              />
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
                style="cursor: {node.metadata.type === 'file' ? 'pointer' : 'default'}"
              >
                <rect
                  width={pos.width}
                  height={pos.height}
                  rx="8"
                  fill="var(--surface-1)"
                  stroke={highlighted.has(node.id) ? 'var(--c-accent)' : 'var(--c-border)'}
                  stroke-width={highlighted.has(node.id) ? 2 : 1}
                />
                {#if node.metadata.type === 'dir'}
                  <foreignObject width={pos.width} height={pos.height}>
                    <div class="flex items-center gap-2 px-3 h-full" xmlns="http://www.w3.org/1999/xhtml">
                      <FolderTree size={14} class="text-[var(--c-text-muted)] shrink-0" />
                      <span class="text-xs font-medium text-[var(--c-text-secondary)] truncate">{node.label}</span>
                      <span class="text-[10px] text-[var(--c-text-muted)] ml-auto">{node.metadata.fileCount}</span>
                    </div>
                  </foreignObject>
                {:else}
                  {@const lang = (node.metadata.language as string) || ''}
                  {@const color = LANG_COLORS[lang.toLowerCase()] || 'var(--c-text-muted)'}
                  <foreignObject width={pos.width} height={pos.height}>
                    <div class="flex flex-col justify-center px-3 h-full gap-0.5" xmlns="http://www.w3.org/1999/xhtml">
                      <div class="flex items-center gap-2">
                        <span class="w-2 h-2 rounded-full shrink-0" style="background: {color}"></span>
                        <span class="text-xs font-medium text-[var(--c-text-primary)] truncate">{node.label}</span>
                      </div>
                      <div class="flex items-center gap-2 pl-4">
                        {#if lang}
                          <span class="text-[10px] text-[var(--c-text-muted)]">{lang}</span>
                        {/if}
                        <span class="text-[10px] text-[var(--c-text-muted)]">{node.metadata.sloc} loc</span>
                      </div>
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

<!-- Detail Modal -->
<SchematicModal open={modalOpen} title={modalTitle} onclose={() => modalOpen = false}>
  {#snippet children()}
    {#if modalLoading}
      <div class="flex justify-center py-8"><LoadingSpinner /></div>
    {:else if modalFile}
      <div class="space-y-4">
        <div class="text-xs text-[var(--c-text-muted)] font-mono break-all">{modalFile.path}</div>
        <div class="flex gap-3 text-sm">
          {#if modalFile.language}
            <span class="px-2 py-0.5 rounded bg-[var(--c-accent)]/10 text-[var(--c-accent)] text-xs">{modalFile.language}</span>
          {/if}
          <span class="text-[var(--c-text-secondary)]">{modalFile.sloc} lines</span>
        </div>

        {#if modalNarrative?.content}
          <div>
            <h3 class="text-sm font-medium text-[var(--c-text-primary)] mb-2">Overview</h3>
            <p class="text-sm text-[var(--c-text-secondary)] leading-relaxed">{modalNarrative.content}</p>
          </div>
        {/if}

        {#if modalSymbols.length > 0}
          <div>
            <h3 class="text-sm font-medium text-[var(--c-text-primary)] mb-2">Symbols ({modalSymbols.length})</h3>
            <div class="space-y-1 max-h-48 overflow-auto">
              {#each modalSymbols as sym}
                <div class="flex items-center gap-2 px-2 py-1.5 rounded bg-[var(--surface-2)] text-xs">
                  <span class="px-1.5 py-0.5 rounded bg-[var(--surface-3)] text-[var(--c-text-muted)] uppercase text-[10px] font-medium">{sym.kind}</span>
                  <span class="font-mono text-[var(--c-text-primary)] truncate">{sym.name}</span>
                  <span class="text-[var(--c-text-muted)] ml-auto">L{sym.start_line}</span>
                </div>
              {/each}
            </div>
          </div>
        {/if}

        {#if modalSource}
          <div>
            <h3 class="text-sm font-medium text-[var(--c-text-primary)] mb-2">Source Preview</h3>
            <pre class="text-[11px] font-mono bg-[var(--surface-2)] rounded-lg p-3 overflow-auto max-h-64 text-[var(--c-text-secondary)]">{modalSource.lines.map(l => `${String(l.number).padStart(3)} ${l.content}`).join('\n')}</pre>
          </div>
        {/if}
      </div>
    {/if}
  {/snippet}
</SchematicModal>
