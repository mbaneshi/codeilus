<script lang="ts">
  import { onMount } from 'svelte';
  import { fetchFiles } from '$lib/api';
  import type { FileRow } from '$lib/types';

  const LANG_COLORS: Record<string, string> = {
    rust: '#dea584',
    typescript: '#3178c6',
    javascript: '#f7df1e',
    python: '#3572a5',
    go: '#00add8',
    java: '#b07219',
    c: '#555555',
    cpp: '#f34b7d',
    ruby: '#701516',
    swift: '#f05138',
  };

  let loading = $state(true);
  let files = $state<FileRow[]>([]);
  let sortKey = $state<'sloc' | 'path'>('sloc');
  let sortAsc = $state(false);

  let totalFiles = $derived(files.length);
  let totalSloc = $derived(files.reduce((sum, f) => sum + f.sloc, 0));
  let languages = $derived(() => {
    const counts = new Map<string, number>();
    for (const f of files) {
      const lang = f.language ?? 'unknown';
      counts.set(lang, (counts.get(lang) ?? 0) + f.sloc);
    }
    return [...counts.entries()]
      .map(([lang, sloc]) => ({ lang, sloc, pct: totalSloc > 0 ? (sloc / totalSloc) * 100 : 0 }))
      .sort((a, b) => b.sloc - a.sloc);
  });
  let uniqueLanguages = $derived(new Set(files.map((f) => f.language).filter(Boolean)).size);

  let sortedFiles = $derived(() => {
    const sorted = [...files];
    sorted.sort((a, b) => {
      if (sortKey === 'sloc') {
        return sortAsc ? a.sloc - b.sloc : b.sloc - a.sloc;
      }
      return sortAsc ? a.path.localeCompare(b.path) : b.path.localeCompare(a.path);
    });
    return sorted.slice(0, 20);
  });

  function toggleSort(key: 'sloc' | 'path') {
    if (sortKey === key) {
      sortAsc = !sortAsc;
    } else {
      sortKey = key;
      sortAsc = key === 'path';
    }
  }

  function langColor(lang: string): string {
    return LANG_COLORS[lang.toLowerCase()] ?? '#6b7280';
  }

  onMount(async () => {
    files = await fetchFiles();
    loading = false;
  });
</script>

<div class="p-6 max-w-5xl mx-auto">
  <h1 class="text-2xl font-bold mb-6">Metrics Dashboard</h1>

  {#if loading}
    <p class="text-gray-400 animate-pulse">Loading...</p>
  {:else if files.length === 0}
    <div class="text-center py-16">
      <p class="text-gray-400 text-lg mb-2">No metrics available</p>
      <p class="text-gray-500">Run <code class="text-indigo-400 font-mono">codeilus analyze ./repo</code> first</p>
    </div>
  {:else}
    <!-- Stats row -->
    <div class="grid grid-cols-3 gap-4 mb-8">
      <div class="stat-card">
        <div class="text-3xl font-bold text-indigo-400">{totalFiles}</div>
        <div class="text-sm text-gray-400">Total Files</div>
      </div>
      <div class="stat-card">
        <div class="text-3xl font-bold text-indigo-400">{totalSloc.toLocaleString()}</div>
        <div class="text-sm text-gray-400">Total SLOC</div>
      </div>
      <div class="stat-card">
        <div class="text-3xl font-bold text-indigo-400">{uniqueLanguages}</div>
        <div class="text-sm text-gray-400">Languages</div>
      </div>
    </div>

    <!-- Language distribution -->
    <h2 class="text-lg font-semibold mb-3">Language Distribution</h2>
    <div class="space-y-2 mb-8">
      {#each languages() as { lang, sloc, pct }}
        <div class="flex items-center gap-3">
          <span class="w-24 text-sm text-gray-300 truncate text-right">{lang}</span>
          <div class="flex-1 bg-gray-800 rounded-full h-5 overflow-hidden">
            <div
              class="h-full rounded-full transition-all"
              style="width: {Math.max(pct, 1)}%; background: {langColor(lang)}"
            ></div>
          </div>
          <span class="text-sm text-gray-400 w-20 text-right">{pct.toFixed(1)}%</span>
          <span class="text-xs text-gray-500 w-16 text-right">{sloc.toLocaleString()}</span>
        </div>
      {/each}
    </div>

    <!-- Top files table -->
    <h2 class="text-lg font-semibold mb-3">Top Files by SLOC</h2>
    <div class="bg-gray-900 border border-gray-800 rounded-lg overflow-hidden">
      <table class="w-full text-sm">
        <thead>
          <tr class="border-b border-gray-800">
            <th class="text-left p-3 text-gray-400 cursor-pointer hover:text-gray-200" onclick={() => toggleSort('path')}>
              Path {sortKey === 'path' ? (sortAsc ? '\u2191' : '\u2193') : ''}
            </th>
            <th class="text-left p-3 text-gray-400 w-28">Language</th>
            <th class="text-right p-3 text-gray-400 w-24 cursor-pointer hover:text-gray-200" onclick={() => toggleSort('sloc')}>
              SLOC {sortKey === 'sloc' ? (sortAsc ? '\u2191' : '\u2193') : ''}
            </th>
          </tr>
        </thead>
        <tbody>
          {#each sortedFiles() as file}
            <tr class="border-b border-gray-800/50 hover:bg-gray-800/50">
              <td class="p-3 font-mono text-gray-200 truncate max-w-md" title={file.path}>{file.path}</td>
              <td class="p-3">
                {#if file.language}
                  <span class="text-xs px-2 py-0.5 rounded" style="background: {langColor(file.language)}20; color: {langColor(file.language)}">{file.language}</span>
                {/if}
              </td>
              <td class="p-3 text-right text-gray-300 font-mono">{file.sloc.toLocaleString()}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<style>
  @reference "tailwindcss";
  .stat-card {
    @apply p-4 bg-gray-900 border border-gray-800 rounded-lg text-center;
  }
</style>
