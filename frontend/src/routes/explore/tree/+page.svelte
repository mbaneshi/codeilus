<script lang="ts">
  import { fetchFiles, fetchFileSymbols } from '$lib/api';
  import type { FileRow, SymbolRow } from '$lib/types';

  interface TreeNode {
    name: string;
    path: string;
    isDir: boolean;
    children: TreeNode[];
    file?: FileRow;
    expanded: boolean;
  }

  let loading = $state(true);
  let files = $state<FileRow[]>([]);
  let tree = $state<TreeNode[]>([]);
  let selectedFile = $state<FileRow | null>(null);
  let symbols = $state<SymbolRow[]>([]);
  let loadingSymbols = $state(false);

  function buildTree(fileList: FileRow[]): TreeNode[] {
    const root: TreeNode = { name: '', path: '', isDir: true, children: [], expanded: true };

    for (const file of fileList) {
      const parts = file.path.split('/');
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
    loadingSymbols = true;
    symbols = await fetchFileSymbols(file.id);
    loadingSymbols = false;
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

  if (typeof window !== 'undefined') {
    fetchFiles().then((data) => {
      files = data;
      tree = buildTree(data);
      loading = false;
    });
  }
</script>

<div class="flex h-full">
  <div class="flex-1 p-6 overflow-auto">
    <div class="flex items-center gap-3 mb-4">
      <a href="/explore" class="text-gray-500 hover:text-gray-300 transition-colors">&larr;</a>
      <h1 class="text-2xl font-bold">File Tree</h1>
    </div>

    {#if loading}
      <p class="text-gray-400 animate-pulse">Loading...</p>
    {:else if files.length === 0}
      <div class="text-center py-16">
        <p class="text-gray-400 text-lg mb-2">No files found</p>
        <p class="text-gray-500">Run <code class="text-indigo-400 font-mono">codeilus analyze ./repo</code> first</p>
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
    <div class="w-80 border-l border-gray-800 bg-gray-900 p-4 overflow-auto">
      <h2 class="text-lg font-semibold mb-1 truncate" title={selectedFile.path}>{selectedFile.path.split('/').pop()}</h2>
      <p class="text-xs text-gray-500 font-mono mb-3 truncate" title={selectedFile.path}>{selectedFile.path}</p>
      <div class="flex gap-2 mb-4">
        {#if selectedFile.language}
          <span class="text-xs px-2 py-0.5 bg-gray-800 rounded text-gray-300">{selectedFile.language}</span>
        {/if}
        <span class="text-xs px-2 py-0.5 bg-gray-800 rounded text-gray-300">{selectedFile.sloc} SLOC</span>
      </div>

      <h3 class="text-sm font-semibold text-gray-300 mb-2">Symbols</h3>
      {#if loadingSymbols}
        <p class="text-gray-400 text-sm animate-pulse">Loading...</p>
      {:else if symbols.length === 0}
        <p class="text-gray-500 text-sm">No symbols found</p>
      {:else}
        <div class="space-y-2">
          {#each symbols as sym}
            <div class="p-2 bg-gray-800 rounded text-sm">
              <div class="flex items-center gap-2 mb-1">
                <span class="text-xs px-1.5 py-0.5 rounded {kindColor(sym.kind)} text-white">{sym.kind}</span>
                <span class="text-gray-100 font-mono truncate">{sym.name}</span>
              </div>
              <div class="text-xs text-gray-500">Lines {sym.start_line}&ndash;{sym.end_line}</div>
              {#if sym.signature}
                <div class="text-xs text-gray-400 font-mono mt-1 truncate" title={sym.signature}>{sym.signature}</div>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
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
      <span class="text-gray-500 inline-block w-4">{node.expanded ? '\u25BE' : '\u25B8'}</span>
      <span class="text-gray-300">{node.name}/</span>
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
      <span class="text-gray-100">{node.name}</span>
      {#if node.file.language}
        <span class="text-xs text-gray-500 ml-2">{node.file.language}</span>
      {/if}
      <span class="text-xs text-gray-600 ml-auto">{node.file.sloc}</span>
    </button>
  {/if}
{/snippet}

<style>
  @reference "tailwindcss";
  .tree-row {
    @apply flex items-center py-1 px-1 rounded text-sm hover:bg-gray-800 cursor-pointer transition-colors;
  }
  .tree-row.active {
    @apply bg-gray-800 border-l-2 border-indigo-500;
  }
</style>
