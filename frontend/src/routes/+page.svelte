<script lang="ts">
  import { fetchHealth, fetchFiles } from '$lib/api';
  import type { FileRow } from '$lib/types';

  let health = $state<string>('checking...');
  let files = $state<FileRow[]>([]);
  let totalFiles = $derived(files.length);
  let totalSloc = $derived(files.reduce((sum, f) => sum + f.sloc, 0));
  let languageCount = $derived(new Set(files.map((f) => f.language).filter(Boolean)).size);

  if (typeof window !== 'undefined') {
    Promise.all([fetchHealth(), fetchFiles()]).then(([healthData, fileData]) => {
      health = healthData.status;
      files = fileData;
    });
  }
</script>

<div class="p-8 max-w-3xl mx-auto">
  <h1 class="text-4xl font-bold mb-4">Welcome to Codeilus</h1>
  <p class="text-gray-400 text-lg mb-8">
    Turn any codebase into an interactive learning experience.
  </p>

  <div class="grid grid-cols-2 gap-4 mb-8">
    <a href="/learn" class="card">
      <h3 class="text-lg font-semibold mb-1">Learn</h3>
      <p class="text-sm text-gray-400">Guided chapters with quizzes and progress tracking</p>
    </a>
    <a href="/explore" class="card">
      <h3 class="text-lg font-semibold mb-1">Explore</h3>
      <p class="text-sm text-gray-400">File tree, graph, metrics, and diagrams</p>
    </a>
    <a href="/ask" class="card">
      <h3 class="text-lg font-semibold mb-1">Ask</h3>
      <p class="text-sm text-gray-400">Q&A powered by Claude Code</p>
    </a>
    <div class="card opacity-50">
      <h3 class="text-lg font-semibold mb-1">Settings</h3>
      <p class="text-sm text-gray-400">Coming soon</p>
    </div>
  </div>

  {#if totalFiles > 0}
    <div class="grid grid-cols-3 gap-4 mb-6">
      <div class="stat-badge">
        <div class="text-xl font-bold text-indigo-400">{totalFiles}</div>
        <div class="text-xs text-gray-500">Files</div>
      </div>
      <div class="stat-badge">
        <div class="text-xl font-bold text-indigo-400">{totalSloc.toLocaleString()}</div>
        <div class="text-xs text-gray-500">SLOC</div>
      </div>
      <div class="stat-badge">
        <div class="text-xl font-bold text-indigo-400">{languageCount}</div>
        <div class="text-xs text-gray-500">Languages</div>
      </div>
    </div>
  {:else if health !== 'checking...'}
    <div class="mb-6 p-4 bg-gray-900 border border-gray-800 rounded-lg text-center">
      <p class="text-gray-400 text-sm">Get started: run <code class="text-indigo-400 font-mono">codeilus analyze ./path</code></p>
    </div>
  {/if}

  <div class="text-sm text-gray-500">
    Server: <span class="text-indigo-400">{health}</span>
  </div>
</div>

<style>
  @reference "tailwindcss";
  .card {
    @apply block p-4 bg-gray-900 border border-gray-800 rounded-lg hover:border-indigo-500 transition-colors;
  }
  .stat-badge {
    @apply p-3 bg-gray-900 border border-gray-800 rounded-lg text-center;
  }
</style>
