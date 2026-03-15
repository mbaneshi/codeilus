<script lang="ts">
  import { fetchFiles, fetchFileSymbols, fetchFileSource } from '$lib/api';
  import type { FileRow, SymbolRow, SourceResponse } from '$lib/types';
  import { FolderTree, ArrowLeft } from 'lucide-svelte';

  interface TreeNode {
    name: string;
    path: string;
    isDir: boolean;
    children: TreeNode[];
    file?: FileRow;
    expanded: boolean;
  }

  let loading = $state(true);
  let error = $state<string | null>(null);
  let files = $state<FileRow[]>([]);
  let tree = $state<TreeNode[]>([]);
  let selectedFile = $state<FileRow | null>(null);
  let symbols = $state<SymbolRow[]>([]);
  let loadingSymbols = $state(false);
  let selectedSymbol = $state<SymbolRow | null>(null);
  let sourceData = $state<SourceResponse | null>(null);
  let loadingSource = $state(false);

  function buildTree(fileList: FileRow[]): TreeNode[] {
    const root: TreeNode = { name: '', path: '', isDir: true, children: [], expanded: true };

    for (const file of fileList) {
      const cleanPath = file.path.replace(/^\.\//, '');
      const parts = cleanPath.split('/');
      let current = root;

      for (let i = 0; i < parts.length; i++) {
        const part = parts[i];
        const isLast = i === parts.length - 1;
        let child = current.children.find((c) => c.name === part);

        if (!child) {
          child = {
            name: part,
            path: parts.slice(0, i + 1).join('/'),
            isDir: !isLast,
            children: [],
            file: isLast ? file : undefined,
            expanded: false,
          };
          current.children.push(child);
        }

        current = child;
      }
    }

    sortTree(root.children);
    return root.children;
  }

  function sortTree(nodes: TreeNode[]) {
    nodes.sort((a, b) => {
      if (a.isDir !== b.isDir) return a.isDir ? -1 : 1;
      return a.name.localeCompare(b.name);
    });
    for (const node of nodes) {
      if (node.isDir) sortTree(node.children);
    }
  }

  function toggleNode(node: TreeNode) {
    node.expanded = !node.expanded;
    tree = [...tree]; // trigger reactivity
  }

  async function selectFile(file: FileRow) {
    selectedFile = file;
    selectedSymbol = null;
    sourceData = null;
    loadingSymbols = true;
    symbols = await fetchFileSymbols(file.id);
    loadingSymbols = false;
  }

  async function selectSymbol(sym: SymbolRow) {
    if (!selectedFile) return;
    selectedSymbol = sym;
    loadingSource = true;
    const start = Math.max(1, sym.start_line - 3);
    const end = sym.end_line + 3;
    sourceData = await fetchFileSource(selectedFile.id, start, end);
    loadingSource = false;
  }

  function kindColor(kind: string): string {
    switch (kind.toLowerCase()) {
      case 'function': return 'bg-indigo-600';
      case 'class': return 'bg-pink-600';
      case 'method': return 'bg-teal-600';
      case 'struct': return 'bg-amber-600';
      case 'enum': return 'bg-purple-600';
      case 'interface': return 'bg-cyan-600';
      default: return 'bg-gray-600';
    }
  }

  function isHighlightedLine(lineNum: number): boolean {
    if (!selectedSymbol) return false;
    return lineNum >= selectedSymbol.start_line && lineNum <= selectedSymbol.end_line;
  }

  if (typeof window !== 'undefined') {
    fetchFiles().then((data) => {
      files = data;
      tree = buildTree(data);
      loading = false;
    }).catch((e) => {
      error = `Failed to load files: ${e}`;
      loading = false;
    });
  }
</script>

<div class="flex h-full">
  <div class="w-[360px] shrink-0 p-6 overflow-auto border-r border-[var(--c-border)]">
    <div class="flex items-center gap-3 mb-5">
      <a href="/explore" class="w-8 h-8 rounded-lg bg-[var(--surface-2)] flex items-center justify-center text-[var(--c-text-muted)] hover:text-[var(--c-text-primary)] hover:bg-[var(--surface-3)] transition-all">
        <ArrowLeft size={16} />
      </a>
      <div class="w-10 h-10 rounded-xl bg-emerald-500/10 flex items-center justify-center">
        <FolderTree size={20} class="text-emerald-400" />
      </div>
      <h1 class="text-2xl font-bold tracking-tight">File Tree</h1>
    </div>

    {#if loading}
      <div class="space-y-2 py-4">
        {#each [1, 2, 3, 4, 5] as _}
          <div class="h-7 bg-[var(--surface-1)] rounded animate-pulse" style="width: {50 + Math.random() * 40}%"></div>
        {/each}
      </div>
    {:else if error}
      <div class="text-center py-16">
        <p class="text-red-400 text-lg mb-2">Error loading files</p>
        <p class="text-[var(--c-text-muted)] text-sm">{error}</p>
      </div>
    {:else if files.length === 0}
      <div class="text-center py-16">
        <p class="text-[var(--c-text-secondary)] text-lg mb-2">No files found</p>
        <p class="text-[var(--c-text-muted)]">Run <code class="text-[var(--c-accent)] font-mono">codeilus analyze ./repo</code> first</p>
      </div>
    {:else}
      <div class="font-mono text-sm">
        {#each tree as node}
          {@render treeItem(node, 0)}
        {/each}
      </div>
    {/if}
  </div>

  {#if selectedFile}
    <div class="flex-1 bg-[var(--surface-1)] flex flex-col overflow-hidden">
      <div class="p-4 border-b border-[var(--c-border)] shrink-0">
        <h2 class="text-lg font-semibold mb-1 truncate text-[var(--c-text-primary)]" title={selectedFile.path}>{selectedFile.path.split('/').pop()}</h2>
        <p class="text-xs text-[var(--c-text-muted)] font-mono mb-3 truncate" title={selectedFile.path}>{selectedFile.path}</p>
        <div class="flex gap-2 mb-4">
          {#if selectedFile.language}
            <span class="text-xs px-2 py-0.5 bg-[var(--surface-2)] rounded text-[var(--c-text-secondary)]">{selectedFile.language}</span>
          {/if}
          <span class="text-xs px-2 py-0.5 bg-[var(--surface-2)] rounded text-[var(--c-text-secondary)]">{selectedFile.sloc} SLOC</span>
        </div>

        <h3 class="text-sm font-semibold text-[var(--c-text-secondary)] mb-2">Symbols</h3>
        {#if loadingSymbols}
          <p class="text-[var(--c-text-muted)] text-sm animate-pulse">Loading...</p>
        {:else if symbols.length === 0}
          <p class="text-[var(--c-text-muted)] text-sm">No symbols found</p>
        {:else}
          <div class="space-y-1 max-h-48 overflow-auto">
            {#each symbols as sym}
              <button
                class="w-full text-left p-2 rounded text-sm transition-colors {selectedSymbol?.id === sym.id ? 'bg-[var(--c-accent)]/10 ring-1 ring-[var(--c-accent)]/50' : 'bg-[var(--surface-2)] hover:bg-[var(--surface-3)]'}"
                onclick={() => selectSymbol(sym)}
              >
                <div class="flex items-center gap-2">
                  <span class="text-xs px-1.5 py-0.5 rounded {kindColor(sym.kind)} text-white shrink-0">{sym.kind}</span>
                  <span class="text-[var(--c-text-primary)] font-mono truncate">{sym.name}</span>
                  <span class="text-xs text-[var(--c-text-muted)] ml-auto shrink-0">L{sym.start_line}</span>
                </div>
              </button>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Source code viewer -->
      <div class="flex-1 overflow-auto">
        {#if loadingSource}
          <div class="p-4">
            <p class="text-[var(--c-text-muted)] text-sm animate-pulse">Loading source...</p>
          </div>
        {:else if sourceData && sourceData.lines.length > 0}
          <div class="source-viewer">
            <div class="px-3 py-2 border-b border-[var(--c-border)] bg-[var(--surface-1)] sticky top-0 z-10">
              <span class="text-xs text-[var(--c-text-muted)]">
                {selectedSymbol?.name ?? 'Source'} &mdash; lines {sourceData.lines[0].number}&ndash;{sourceData.lines[sourceData.lines.length - 1].number} of {sourceData.total_lines}
              </span>
            </div>
            <pre class="text-xs leading-5 overflow-x-auto"><code>{#each sourceData.lines as line}<div
              class="source-line {isHighlightedLine(line.number) ? 'highlighted' : ''}"
            ><span class="line-num">{String(line.number).padStart(4, ' ')}</span><span class="line-content">{line.content}</span></div>{/each}</code></pre>
          </div>
        {:else if selectedSymbol}
          <div class="p-4">
            <p class="text-[var(--c-text-muted)] text-sm">Could not load source. Ensure <code class="text-[var(--c-accent)] font-mono text-xs">repo_root</code> is set.</p>
          </div>
        {:else}
          <div class="p-4">
            <p class="text-[var(--c-text-muted)] text-sm">Click a symbol above to view its source code</p>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

{#snippet treeItem(node: TreeNode, depth: number)}
  {#if node.isDir}
    <button
      class="tree-row w-full text-left"
      style="padding-left: {depth * 16 + 4}px"
      onclick={() => toggleNode(node)}
    >
      <span class="text-[var(--c-text-muted)] inline-block w-4">{node.expanded ? '\u25BE' : '\u25B8'}</span>
      <span class="text-[var(--c-text-secondary)]">{node.name}/</span>
    </button>
    {#if node.expanded}
      {#each node.children as child}
        {@render treeItem(child, depth + 1)}
      {/each}
    {/if}
  {:else if node.file}
    <button
      class="tree-row w-full text-left"
      class:active={selectedFile?.id === node.file.id}
      style="padding-left: {depth * 16 + 20}px"
      onclick={() => node.file && selectFile(node.file)}
    >
      <span class="text-[var(--c-text-primary)]">{node.name}</span>
      {#if node.file.language}
        <span class="text-xs text-[var(--c-text-muted)] ml-2">{node.file.language}</span>
      {/if}
      <span class="text-xs text-[var(--c-text-muted)] ml-auto">{node.file.sloc}</span>
    </button>
  {/if}
{/snippet}

<style>
  @reference "tailwindcss";
  .tree-row {
    @apply flex items-center py-1 px-1 rounded text-sm hover:bg-[var(--surface-2)] cursor-pointer transition-colors;
  }
  .tree-row.active {
    @apply bg-[var(--surface-2)] border-l-2 border-[var(--c-accent)];
  }
  .source-viewer {
    @apply font-mono bg-[var(--surface-0)];
  }
  .source-line {
    @apply px-3 whitespace-pre;
    min-height: 1.25rem;
  }
  .source-line.highlighted {
    @apply bg-[var(--c-accent)]/10 border-l-2 border-[var(--c-accent)];
  }
  .line-num {
    @apply text-[var(--c-text-muted)] select-none mr-4 inline-block;
  }
  .line-content {
    @apply text-[var(--c-text-secondary)];
  }
</style>
