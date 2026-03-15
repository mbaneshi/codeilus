<script lang="ts">
  import { fetchFiles } from '$lib/api';
  import type { FileRow } from '$lib/types';
  import { BarChart3, ArrowLeft } from 'lucide-svelte';

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
  let error = $state<string | null>(null);
  let files = $state<FileRow[]>([]);
  let sortKey = $state<'sloc' | 'path'>('sloc');
  let sortAsc = $state(false);

  let totalFiles = $derived(files.length);
  let totalSloc = $derived(files.reduce((sum, f) => sum + f.sloc, 0));
  let languages = $derived.by(() => {
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
  let avgSloc = $derived(totalFiles > 0 ? Math.round(totalSloc / totalFiles) : 0);

  let sortedFiles = $derived.by(() => {
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

  if (typeof window !== 'undefined') {
    fetchFiles().then((data) => {
      files = data;
      loading = false;
    }).catch((e) => {
      error = `Failed to load files: ${e}`;
      loading = false;
    });
  }
</script>

<div class="p-6 max-w-5xl mx-auto">
  <div class="flex items-center gap-3 mb-8">
    <a href="/explore" class="w-8 h-8 rounded-lg bg-[var(--surface-2)] flex items-center justify-center text-[var(--c-text-muted)] hover:text-[var(--c-text-primary)] hover:bg-[var(--surface-3)] transition-all">
      <ArrowLeft size={16} />
    </a>
    <div class="w-10 h-10 rounded-xl bg-sky-500/10 flex items-center justify-center">
      <BarChart3 size={20} class="text-sky-400" />
    </div>
    <div>
      <h1 class="text-2xl font-bold tracking-tight">Metrics Dashboard</h1>
      <p class="text-sm text-[var(--c-text-secondary)]">SLOC, language distribution, and file analysis</p>
    </div>
  </div>

  {#if loading}
    <div class="space-y-4">
      {#each [1, 2, 3] as _}
        <div class="h-20 bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl animate-pulse"></div>
      {/each}
    </div>
  {:else if error}
    <div class="text-center py-16">
      <p class="text-red-400 text-lg mb-2">Error loading metrics</p>
      <p class="text-[var(--c-text-muted)] text-sm">{error}</p>
    </div>
  {:else if files.length === 0}
    <div class="text-center py-16">
      <p class="text-gray-400 text-lg mb-2">No metrics available</p>
      <p class="text-gray-500">Run <code class="text-indigo-400 font-mono">codeilus analyze ./repo</code> first</p>
    </div>
  {:else}
    <!-- Stats row -->
    <div class="grid grid-cols-2 sm:grid-cols-4 gap-4 mb-8">
      <div class="stat-card">
        <div class="text-3xl font-bold text-indigo-400">{totalFiles}</div>
        <div class="text-sm text-[var(--c-text-muted)]">Total Files</div>
      </div>
      <div class="stat-card">
        <div class="text-3xl font-bold text-teal-400">{totalSloc.toLocaleString()}</div>
        <div class="text-sm text-[var(--c-text-muted)]">Total SLOC</div>
      </div>
      <div class="stat-card">
        <div class="text-3xl font-bold text-amber-400">{avgSloc}</div>
        <div class="text-sm text-[var(--c-text-muted)]">Avg SLOC/File</div>
      </div>
      <div class="stat-card">
        <div class="text-3xl font-bold text-pink-400">{uniqueLanguages}</div>
        <div class="text-sm text-[var(--c-text-muted)]">Languages</div>
      </div>
    </div>

    <!-- Language distribution -->
    <h2 class="text-lg font-semibold mb-3">Language Distribution</h2>
    <div class="space-y-2 mb-8">
      {#each languages as { lang, sloc, pct }}
        <div class="flex items-center gap-3">
          <span class="w-24 text-sm text-[var(--c-text-secondary)] truncate text-right">{lang}</span>
          <div class="flex-1 bg-[var(--surface-3)] rounded-full h-5 overflow-hidden">
            <div
              class="h-full rounded-full transition-all"
              style="width: {Math.max(pct, 1)}%; background: {langColor(lang)}"
            ></div>
          </div>
          <span class="text-sm text-[var(--c-text-muted)] w-20 text-right">{pct.toFixed(1)}%</span>
          <span class="text-xs text-[var(--c-text-muted)] w-16 text-right">{sloc.toLocaleString()}</span>
        </div>
      {/each}
    </div>

    <!-- Top files table -->
    <h2 class="text-lg font-semibold mb-3">Top Files by SLOC</h2>
    <div class="bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl overflow-hidden">
      <table class="w-full text-sm">
        <thead>
          <tr class="border-b border-[var(--c-border)]">
            <th class="text-left p-3 text-[var(--c-text-muted)] cursor-pointer hover:text-[var(--c-text-primary)] transition-colors" onclick={() => toggleSort('path')}>
              Path {sortKey === 'path' ? (sortAsc ? '\u2191' : '\u2193') : ''}
            </th>
            <th class="text-left p-3 text-[var(--c-text-muted)] w-28">Language</th>
            <th class="text-right p-3 text-[var(--c-text-muted)] w-24 cursor-pointer hover:text-[var(--c-text-primary)] transition-colors" onclick={() => toggleSort('sloc')}>
              SLOC {sortKey === 'sloc' ? (sortAsc ? '\u2191' : '\u2193') : ''}
            </th>
          </tr>
        </thead>
        <tbody>
          {#each sortedFiles as file}
            <tr class="border-b border-[var(--c-border)]/50 hover:bg-[var(--surface-2)]">
              <td class="p-3 font-mono text-[var(--c-text-primary)] truncate max-w-md" title={file.path}>{file.path}</td>
              <td class="p-3">
                {#if file.language}
                  <span class="text-xs px-2 py-0.5 rounded" style="background: {langColor(file.language)}20; color: {langColor(file.language)}">{file.language}</span>
                {/if}
              </td>
              <td class="p-3 text-right text-[var(--c-text-secondary)] font-mono">{file.sloc.toLocaleString()}</td>
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
    @apply p-4 bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl text-center;
  }
</style>
