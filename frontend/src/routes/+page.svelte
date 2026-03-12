<script lang="ts">
  import { onMount } from 'svelte';

  let health = $state<string>('checking...');

  onMount(async () => {
    try {
      const res = await fetch('/api/v1/health');
      const data = await res.json();
      health = data.status;
    } catch {
      health = 'disconnected';
    }
  });
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

  <div class="text-sm text-gray-500">
    Server: <span class="text-indigo-400">{health}</span>
  </div>
</div>

<style>
  @reference "tailwindcss";
  .card {
    @apply block p-4 bg-gray-900 border border-gray-800 rounded-lg hover:border-indigo-500 transition-colors;
  }
</style>
