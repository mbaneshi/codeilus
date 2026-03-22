<script lang="ts">
  import { fetchFiles, fetchFileSymbols, fetchFileSource } from '$lib/api';
  import type { FileRow, SymbolRow, SourceResponse } from '$lib/types';
  import { FolderTree, ArrowLeft, Search } from 'lucide-svelte';
  import { createHighlighter, type Highlighter } from 'shiki';

  interface TreeNode {
    name: string;
    path: string;
    isDir: boolean;
    children: TreeNode[];
    file?: FileRow;
    expanded: boolean;
  }

  // Shiki language map from our language strings
  const LANG_MAP: Record<string, string> = {
    rust: 'rust',
    python: 'python',
    typescript: 'typescript',
    javascript: 'javascript',
    go: 'go',
    java: 'java',
    tsx: 'tsx',
    jsx: 'jsx',
    css: 'css',
    html: 'html',
    json: 'json',
    yaml: 'yaml',
    toml: 'toml',
    markdown: 'markdown',
    sql: 'sql',
    shell: 'shellscript',
    bash: 'shellscript',
    svelte: 'svelte',
    c: 'c',
    cpp: 'cpp',
  };

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
  let highlightedHtml = $state('');
  let highlighter = $state<Highlighter | null>(null);
  let symbolFilter = $state('');
  let searchQuery = $state('');

  // Initialize shiki highlighter — starts empty, languages loaded on demand per file
  async function getHighlighter(): Promise<Highlighter> {
    if (highlighter) return highlighter;
    highlighter = await createHighlighter({
      themes: ['github-dark', 'github-light'],
      langs: [],
    });
    return highlighter;
  }

  // Highlight source code
  async function highlightSource(lines: { number: number; content: string }[], language: string) {
    const code = lines.map((l) => l.content).join('\n');
    const lang = LANG_MAP[language?.toLowerCase()] || 'text';

    try {
      const hl = await getHighlighter();
      // Check if language is loaded
      const loadedLangs = hl.getLoadedLanguages();
      if (!loadedLangs.includes(lang as any)) {
        try {
          await hl.loadLanguage(lang as any);
        } catch {
          // Fall back to plaintext
          highlightedHtml = '';
          return;
        }
      }
      highlightedHtml = hl.codeToHtml(code, {
        lang,
        theme: document.documentElement.classList.contains('light') ? 'github-light' : 'github-dark',
      });
    } catch {
      highlightedHtml = '';
    }
  }

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
    tree = [...tree];
  }

  async function selectFile(file: FileRow) {
    selectedFile = file;
    selectedSymbol = null;
    sourceData = null;
    highlightedHtml = '';
    loadingSymbols = true;
    loadingSource = true;
    symbolFilter = '';

    // Load symbols and full file source in parallel
    const [syms, source] = await Promise.all([
      fetchFileSymbols(file.id),
      fetchFileSource(file.id),
    ]);
    symbols = syms;
    loadingSymbols = false;
    sourceData = source;
    loadingSource = false;

    if (source && source.lines.length > 0 && file.language) {
      await highlightSource(source.lines, file.language);
    }
  }

  async function selectSymbol(sym: SymbolRow) {
    if (!selectedFile) return;
    selectedSymbol = sym;

    // Scroll to the symbol line in the source viewer
    requestAnimationFrame(() => {
      const lineEl = document.querySelector(`[data-line="${sym.start_line}"]`);
      if (lineEl) {
        lineEl.scrollIntoView({ behavior: 'smooth', block: 'center' });
      }
    });
  }

  let filteredSymbols = $derived(
    symbolFilter
      ? symbols.filter((s) => s.name.toLowerCase().includes(symbolFilter.toLowerCase()))
      : symbols
  );

  function kindColor(kind: string): string {
    switch (kind.toLowerCase()) {
      case 'function': return 'bg-indigo-600';
      case 'class': return 'bg-pink-600';
      case 'method': return 'bg-teal-600';
      case 'struct': return 'bg-amber-600';
      case 'enum': return 'bg-purple-600';
      case 'interface': return 'bg-cyan-600';
      case 'trait': return 'bg-orange-600';
      case 'constant': return 'bg-emerald-600';
      case 'variable': return 'bg-slate-600';
      default: return 'bg-gray-600';
    }
  }

  function isHighlightedLine(lineNum: number): boolean {
    if (!selectedSymbol) return false;
    return lineNum >= selectedSymbol.start_line && lineNum <= selectedSymbol.end_line;
  }

  // Filter tree nodes by search
  function matchesSearch(node: TreeNode, query: string): boolean {
    if (!query) return true;
    const q = query.toLowerCase();
    if (node.name.toLowerCase().includes(q)) return true;
    if (node.isDir) return node.children.some((c) => matchesSearch(c, q));
    return false;
  }

  if (typeof window !== 'undefined') {
    // Pre-init highlighter
    getHighlighter();

    // Re-highlight when theme changes
    window.addEventListener('theme-change', () => {
      if (sourceData && sourceData.lines.length > 0 && selectedFile?.language) {
        highlightSource(sourceData.lines, selectedFile.language);
      }
    });

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
  <!-- File tree panel -->
  <div class="w-[320px] shrink-0 flex flex-col border-r border-[var(--c-border)]">
    <div class="p-4 border-b border-[var(--c-border)] shrink-0">
      <div class="flex items-center gap-3 mb-3">
        <a href="/explore" class="w-7 h-7 rounded-lg bg-[var(--surface-2)] flex items-center justify-center text-[var(--c-text-muted)] hover:text-[var(--c-text-primary)] hover:bg-[var(--surface-3)] transition-all">
          <ArrowLeft size={14} />
        </a>
        <div class="w-8 h-8 rounded-xl bg-emerald-500/10 flex items-center justify-center">
          <FolderTree size={16} class="text-emerald-400" />
        </div>
        <h1 class="text-lg font-bold tracking-tight">Files</h1>
        <span class="text-xs text-[var(--c-text-muted)] ml-auto">{files.length}</span>
      </div>

      <!-- Search -->
      <div class="relative">
        <Search size={13} class="absolute left-2.5 top-1/2 -translate-y-1/2 text-[var(--c-text-muted)]" />
        <input
          type="text"
          placeholder="Filter files..."
          bind:value={searchQuery}
          class="w-full text-xs py-1.5 pl-8 pr-3 rounded-lg bg-[var(--surface-1)] border border-[var(--c-border)] text-[var(--c-text-primary)] placeholder:text-[var(--c-text-muted)] focus:outline-none focus:ring-1 focus:ring-[var(--c-accent)]/50"
        />
      </div>
    </div>

    <div class="flex-1 overflow-auto p-2">
      {#if loading}
        <div class="space-y-2 py-4 px-2">
          {#each [1, 2, 3, 4, 5] as _}
            <div class="h-6 bg-[var(--surface-1)] rounded animate-pulse" style="width: {50 + Math.random() * 40}%"></div>
          {/each}
        </div>
      {:else if error}
        <div class="text-center py-16 px-4">
          <p class="text-red-400 text-sm mb-2">Error loading files</p>
          <p class="text-[var(--c-text-muted)] text-xs">{error}</p>
        </div>
      {:else if files.length === 0}
        <div class="text-center py-16 px-4">
          <p class="text-[var(--c-text-secondary)] text-sm mb-2">No files found</p>
          <p class="text-[var(--c-text-muted)] text-xs">Run <code class="text-[var(--c-accent)] font-mono">codeilus analyze ./repo</code></p>
        </div>
      {:else}
        <div class="font-mono text-xs">
          {#each tree as node}
            {#if matchesSearch(node, searchQuery)}
              {@render treeItem(node, 0)}
            {/if}
          {/each}
        </div>
      {/if}
    </div>
  </div>

  <!-- Right panel: source + symbols -->
  {#if selectedFile}
    <div class="flex-1 flex flex-col overflow-hidden">
      <!-- Header bar -->
      <div class="flex items-center gap-3 px-4 py-2.5 border-b border-[var(--c-border)] bg-[var(--surface-1)] shrink-0">
        <span class="font-semibold text-sm text-[var(--c-text-primary)] truncate">{selectedFile.path.split('/').pop()}</span>
        <span class="text-xs text-[var(--c-text-muted)] font-mono truncate hidden sm:inline">{selectedFile.path}</span>
        <div class="flex gap-1.5 ml-auto shrink-0">
          {#if selectedFile.language}
            <span class="text-[10px] px-1.5 py-0.5 bg-[var(--c-accent)]/15 text-[var(--c-accent)] rounded font-medium">{selectedFile.language}</span>
          {/if}
          <span class="text-[10px] px-1.5 py-0.5 bg-[var(--surface-2)] rounded text-[var(--c-text-muted)]">{selectedFile.sloc} SLOC</span>
        </div>
      </div>

      <div class="flex flex-1 overflow-hidden">
        <!-- Source code viewer -->
        <div class="flex-1 overflow-auto bg-[#0d1117]">
          {#if loadingSource}
            <div class="p-6">
              <div class="space-y-2">
                {#each [1, 2, 3, 4, 5, 6, 7, 8] as _}
                  <div class="h-4 bg-white/5 rounded animate-pulse" style="width: {30 + Math.random() * 60}%"></div>
                {/each}
              </div>
            </div>
          {:else if sourceData && sourceData.lines.length > 0}
            <div class="source-viewer">
              {#if highlightedHtml}
                <!-- Shiki highlighted source with line numbers overlay -->
                <div class="shiki-wrapper">
                  <!-- Line numbers gutter -->
                  <div class="line-gutter" aria-hidden="true">
                    {#each sourceData.lines as line}
                      <div
                        class="gutter-line {isHighlightedLine(line.number) ? 'highlighted' : ''}"
                        data-line={line.number}
                      >{line.number}</div>
                    {/each}
                  </div>
                  <!-- Highlighted code -->
                  <div class="code-panel">
                    {#each sourceData.lines as line, i}
                      <div
                        class="code-line {isHighlightedLine(line.number) ? 'highlighted' : ''}"
                        data-line={line.number}
                      ></div>
                    {/each}
                    <div class="shiki-html">{@html highlightedHtml}</div>
                  </div>
                </div>
              {:else}
                <!-- Fallback: plain text with line numbers -->
                <pre class="text-xs leading-5 overflow-x-auto"><code>{#each sourceData.lines as line}<div
                  class="source-line {isHighlightedLine(line.number) ? 'highlighted' : ''}"
                  data-line={line.number}
                ><span class="line-num">{String(line.number).padStart(4, ' ')}</span><span class="line-content">{line.content}</span></div>{/each}</code></pre>
              {/if}
            </div>
          {:else}
            <div class="flex items-center justify-center h-full text-[var(--c-text-muted)] text-sm">
              <p>Could not load source. Ensure the repo path is set.</p>
            </div>
          {/if}
        </div>

        <!-- Symbols sidebar -->
        <div class="w-[240px] shrink-0 border-l border-[var(--c-border)] bg-[var(--surface-1)] flex flex-col overflow-hidden">
          <div class="p-3 border-b border-[var(--c-border)] shrink-0">
            <div class="flex items-center justify-between mb-2">
              <h3 class="text-xs font-semibold text-[var(--c-text-secondary)] uppercase tracking-wider">Symbols</h3>
              <span class="text-[10px] text-[var(--c-text-muted)]">{symbols.length}</span>
            </div>
            {#if symbols.length > 5}
              <input
                type="text"
                placeholder="Filter..."
                bind:value={symbolFilter}
                class="w-full text-xs py-1 px-2 rounded bg-[var(--surface-0)] border border-[var(--c-border)] text-[var(--c-text-primary)] placeholder:text-[var(--c-text-muted)] focus:outline-none focus:ring-1 focus:ring-[var(--c-accent)]/50"
              />
            {/if}
          </div>

          <div class="flex-1 overflow-auto">
            {#if loadingSymbols}
              <div class="p-3 space-y-2">
                {#each [1, 2, 3] as _}
                  <div class="h-7 bg-[var(--surface-2)] rounded animate-pulse"></div>
                {/each}
              </div>
            {:else if filteredSymbols.length === 0}
              <p class="text-[var(--c-text-muted)] text-xs p-3">No symbols</p>
            {:else}
              <div class="p-1.5">
                {#each filteredSymbols as sym}
                  <button
                    class="sym-row w-full text-left px-2 py-1.5 rounded text-xs transition-colors {selectedSymbol?.id === sym.id ? 'bg-[var(--c-accent)]/15 ring-1 ring-[var(--c-accent)]/40' : 'hover:bg-[var(--surface-2)]'}"
                    onclick={() => selectSymbol(sym)}
                  >
                    <div class="flex items-center gap-1.5">
                      <span class="text-[9px] px-1 py-px rounded {kindColor(sym.kind)} text-white shrink-0 font-medium uppercase">{sym.kind.slice(0, 3)}</span>
                      <span class="text-[var(--c-text-primary)] font-mono truncate">{sym.name}</span>
                      <span class="text-[10px] text-[var(--c-text-muted)] ml-auto shrink-0">{sym.start_line}</span>
                    </div>
                    {#if sym.signature}
                      <p class="text-[10px] text-[var(--c-text-muted)] font-mono truncate mt-0.5 pl-6">{sym.signature}</p>
                    {/if}
                  </button>
                {/each}
              </div>
            {/if}
          </div>
        </div>
      </div>
    </div>
  {:else}
    <!-- Empty state -->
    <div class="flex-1 flex items-center justify-center bg-[var(--surface-0)]">
      <div class="text-center">
        <div class="w-16 h-16 rounded-2xl bg-[var(--surface-2)] flex items-center justify-center mx-auto mb-4">
          <FolderTree size={28} class="text-[var(--c-text-muted)]" />
        </div>
        <p class="text-[var(--c-text-secondary)] text-lg mb-1">Select a file</p>
        <p class="text-[var(--c-text-muted)] text-sm">Choose a file from the tree to view its source code</p>
      </div>
    </div>
  {/if}
</div>

{#snippet treeItem(node: TreeNode, depth: number)}
  {#if node.isDir}
    <button
      class="tree-row w-full text-left"
      style="padding-left: {depth * 14 + 4}px"
      onclick={() => toggleNode(node)}
    >
      <span class="text-[var(--c-text-muted)] inline-block w-3.5 text-[10px]">{node.expanded ? '\u25BE' : '\u25B8'}</span>
      <span class="text-[var(--c-text-secondary)]">{node.name}/</span>
    </button>
    {#if node.expanded}
      {#each node.children as child}
        {#if matchesSearch(child, searchQuery)}
          {@render treeItem(child, depth + 1)}
        {/if}
      {/each}
    {/if}
  {:else if node.file}
    <button
      class="tree-row w-full text-left"
      class:active={selectedFile?.id === node.file.id}
      style="padding-left: {depth * 14 + 18}px"
      onclick={() => node.file && selectFile(node.file)}
    >
      <span class="text-[var(--c-text-primary)] truncate">{node.name}</span>
      {#if node.file.language}
        <span class="text-[10px] text-[var(--c-text-muted)] ml-1.5 shrink-0 opacity-60">{node.file.language}</span>
      {/if}
      <span class="text-[10px] text-[var(--c-text-muted)] ml-auto shrink-0">{node.file.sloc}</span>
    </button>
  {/if}
{/snippet}

<style>
  @reference "tailwindcss";

  .tree-row {
    @apply flex items-center py-0.5 px-1 rounded hover:bg-[var(--surface-2)] cursor-pointer transition-colors;
  }
  .tree-row.active {
    @apply bg-[var(--c-accent)]/10 text-[var(--c-accent)];
  }

  .source-viewer {
    font-family: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', ui-monospace, monospace;
    font-size: 13px;
    line-height: 1.6;
  }

  /* Shiki wrapper — overlays line numbers + highlight bars on top of shiki output */
  .shiki-wrapper {
    display: flex;
    position: relative;
  }

  .line-gutter {
    position: sticky;
    left: 0;
    z-index: 2;
    display: flex;
    flex-direction: column;
    padding: 1rem 0;
    user-select: none;
    background: #0d1117;
    border-right: 1px solid rgba(255,255,255,0.06);
  }

  .gutter-line {
    padding: 0 12px 0 16px;
    text-align: right;
    color: rgba(255,255,255,0.2);
    font-size: 12px;
    line-height: 1.6;
    min-height: 1.6em;
  }

  .gutter-line.highlighted {
    color: rgba(99, 102, 241, 0.8);
    background: rgba(99, 102, 241, 0.08);
  }

  .code-panel {
    flex: 1;
    position: relative;
    overflow-x: auto;
  }

  .code-line {
    position: absolute;
    left: 0;
    right: 0;
    height: 1.6em;
    pointer-events: none;
  }

  .code-line.highlighted {
    background: rgba(99, 102, 241, 0.08);
    border-left: 2px solid rgba(99, 102, 241, 0.6);
  }

  /* Override shiki's default styles */
  .shiki-html :global(pre) {
    margin: 0 !important;
    padding: 1rem 1rem 1rem 0.75rem !important;
    background: transparent !important;
    overflow: visible !important;
  }

  .shiki-html :global(code) {
    font-family: inherit !important;
    font-size: inherit !important;
    line-height: 1.6 !important;
  }

  .shiki-html :global(.line) {
    min-height: 1.6em;
    display: inline-block;
    width: 100%;
  }

  /* Fallback plain text */
  .source-line {
    @apply whitespace-pre;
    min-height: 1.6em;
    padding: 0 12px;
  }
  .source-line.highlighted {
    background: rgba(99, 102, 241, 0.08);
    border-left: 2px solid rgba(99, 102, 241, 0.6);
  }
  .line-num {
    color: rgba(255,255,255,0.2);
    user-select: none;
    margin-right: 16px;
    display: inline-block;
  }
  .line-content {
    color: #c9d1d9;
  }

  .sym-row {
    border: 1px solid transparent;
  }
</style>
