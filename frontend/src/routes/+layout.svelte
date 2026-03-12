<script lang="ts">
  import '../app.css';
  let { children } = $props();
</script>

<!-- Sidebar + main content -->
<div class="flex h-screen">
  <!-- Sidebar -->
  <nav class="w-60 bg-gray-900 border-r border-gray-800 flex flex-col">
    <div class="p-4 border-b border-gray-800">
      <h1 class="text-xl font-bold text-indigo-400">Codeilus</h1>
      <p class="text-xs text-gray-500 mt-1">Learn any codebase</p>
    </div>
    <div class="flex-1 p-2 space-y-1">
      <a href="/" class="nav-item">Home</a>
      <a href="/learn" class="nav-item">Learn</a>
      <a href="/explore" class="nav-item">Explore</a>
      <a href="/ask" class="nav-item">Ask</a>
    </div>
    <div class="p-4 border-t border-gray-800 text-xs text-gray-600">
      v0.1.0
    </div>
  </nav>

  <!-- Main content -->
  <main class="flex-1 overflow-auto">
    {@render children()}
  </main>
</div>

<style>
  @reference "tailwindcss";
  .nav-item {
    @apply block px-3 py-2 rounded text-sm text-gray-300 hover:bg-gray-800 hover:text-white transition-colors;
  }
</style>
