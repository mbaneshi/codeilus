<div class="p-8 max-w-3xl mx-auto">
  <h1 class="text-2xl font-bold mb-2">Explore</h1>
  <p class="text-gray-400 mb-6">Browse the codebase structure, relationships, and metrics.</p>
  <div class="grid grid-cols-2 gap-4">
    <a href="/explore/tree" class="card">
      <div class="text-lg font-semibold mb-1">File Tree</div>
      <div class="text-sm text-gray-400">Browse files, folders, and symbols</div>
    </a>
    <a href="/explore/graph" class="card">
      <div class="text-lg font-semibold mb-1">Knowledge Graph</div>
      <div class="text-sm text-gray-400">Interactive dependency visualization</div>
    </a>
    <a href="/explore/metrics" class="card">
      <div class="text-lg font-semibold mb-1">Metrics</div>
      <div class="text-sm text-gray-400">SLOC, language distribution, top files</div>
    </a>
    <a href="/explore/diagrams" class="card">
      <div class="text-lg font-semibold mb-1">Diagrams</div>
      <div class="text-sm text-gray-400">Architecture modules and process flows</div>
    </a>
  </div>
</div>

<style>
  @reference "tailwindcss";
  .card {
    @apply block p-6 bg-gray-900 border border-gray-800 rounded-lg hover:border-indigo-500 transition-colors;
  }
</style>
