<script lang="ts">
  import '../app.css';
  import { searchSymbols } from '$lib/api';
  import type { SymbolRow } from '$lib/types';

  let { children } = $props();
  let searchQuery = $state('');
  let searchResults = $state<SymbolRow[]>([]);
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  function onSearchInput() {
    if (debounceTimer) clearTimeout(debounceTimer);
    if (searchQuery.trim().length < 2) {
      searchResults = [];
      return;
    }
    debounceTimer = setTimeout(async () => {
      const results = await searchSymbols(searchQuery.trim());
      searchResults = results.slice(0, 5);
    }, 300);
  }

  function clearSearch() {
    searchQuery = '';
    searchResults = [];
  }
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

      <!-- Search -->
      <div class="mt-4 pt-3 border-t border-gray-800">
        <div class="relative">
          <input
            type="text"
            placeholder="Search symbols..."
            class="w-full bg-gray-800 border border-gray-700 rounded px-3 py-1.5 text-sm text-gray-100 focus:border-indigo-500 outline-none"
            bind:value={searchQuery}
            oninput={onSearchInput}
          />
          {#if searchResults.length > 0}
            <div class="absolute left-0 right-0 mt-1 bg-gray-900 border border-gray-700 rounded shadow-lg z-20 overflow-hidden">
              {#each searchResults as sym}
                <a
                  href="/explore/tree"
                  class="block px-3 py-2 hover:bg-gray-800 text-sm transition-colors"
                  onclick={clearSearch}
                >
                  <span class="text-xs text-gray-500 mr-1">{sym.kind}</span>
                  <span class="font-mono text-gray-200">{sym.name}</span>
                </a>
              {/each}
            </div>
          {/if}
        </div>
      </div>
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
