<script lang="ts">
  import { onMount } from 'svelte';
  import { searchSymbols } from '$lib/api';
  import { connectWebSocket, isConnected } from '$lib/stores/events.svelte';
  import type { SymbolRow } from '$lib/types';

  let query = $state('');
  let suggestions = $state<SymbolRow[]>([]);
  let context = $state<SymbolRow[]>([]);
  let toast = $state('');
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  function onInput() {
    if (debounceTimer) clearTimeout(debounceTimer);
    if (query.trim().length < 2) {
      suggestions = [];
      return;
    }
    debounceTimer = setTimeout(async () => {
      suggestions = await searchSymbols(query.trim());
    }, 300);
  }

  function addToContext(sym: SymbolRow) {
    if (!context.find((s) => s.id === sym.id)) {
      context = [...context, sym];
    }
    suggestions = [];
    query = '';
  }

  function removeFromContext(id: number) {
    context = context.filter((s) => s.id !== id);
  }

  function handleAsk() {
    toast = 'LLM not connected \u2014 coming soon';
    setTimeout(() => { toast = ''; }, 3000);
  }

  function kindColor(kind: string): string {
    switch (kind.toLowerCase()) {
      case 'function': return 'bg-indigo-600';
      case 'class': return 'bg-pink-600';
      case 'method': return 'bg-teal-600';
      case 'struct': return 'bg-amber-600';
      default: return 'bg-gray-600';
    }
  }

  onMount(() => {
    if (!isConnected()) {
      connectWebSocket();
    }
  });
</script>

<div class="p-8 max-w-3xl mx-auto">
  <div class="flex items-center gap-3 mb-4">
    <h1 class="text-2xl font-bold">Ask About the Code</h1>
    <span class="text-xs px-2 py-0.5 rounded-full {isConnected() ? 'bg-green-900 text-green-400' : 'bg-red-900 text-red-400'}">
      {isConnected() ? 'WS Connected' : 'WS Disconnected'}
    </span>
  </div>

  <!-- Context area -->
  {#if context.length > 0}
    <div class="mb-4 p-3 bg-gray-900 border border-gray-800 rounded-lg">
      <div class="text-xs text-gray-500 mb-2">Context</div>
      <div class="flex flex-wrap gap-2">
        {#each context as sym}
          <span class="inline-flex items-center gap-1 text-sm px-2 py-1 bg-gray-800 rounded">
            <span class="text-xs px-1 py-0.5 rounded {kindColor(sym.kind)} text-white">{sym.kind}</span>
            <span class="font-mono text-gray-200">{sym.name}</span>
            <button class="text-gray-500 hover:text-gray-300 ml-1" onclick={() => removeFromContext(sym.id)}>&times;</button>
          </span>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Chat area -->
  <div class="bg-gray-900 border border-gray-800 rounded-lg p-4 mb-4 h-96 overflow-auto">
    <p class="text-gray-500 text-center mt-32">Ask anything about the codebase...</p>
  </div>

  <!-- Input area -->
  <div class="relative">
    <div class="flex gap-2">
      <input
        type="text"
        placeholder="Search for symbols or ask a question..."
        class="flex-1 bg-gray-900 border border-gray-800 rounded px-4 py-2 text-gray-100 focus:border-indigo-500 outline-none"
        bind:value={query}
        oninput={onInput}
      />
      <button
        class="bg-indigo-600 px-6 py-2 rounded hover:bg-indigo-500 transition-colors"
        onclick={handleAsk}
      >Ask</button>
    </div>

    <!-- Suggestions dropdown -->
    {#if suggestions.length > 0}
      <div class="absolute left-0 right-16 mt-1 bg-gray-900 border border-gray-700 rounded-lg shadow-lg z-10 max-h-48 overflow-auto">
        {#each suggestions as sym}
          <button
            class="w-full text-left px-3 py-2 hover:bg-gray-800 flex items-center gap-2 text-sm transition-colors"
            onclick={() => addToContext(sym)}
          >
            <span class="text-xs px-1.5 py-0.5 rounded {kindColor(sym.kind)} text-white">{sym.kind}</span>
            <span class="font-mono text-gray-200">{sym.name}</span>
            <span class="text-xs text-gray-500 ml-auto">L{sym.start_line}</span>
          </button>
        {/each}
      </div>
    {/if}
  </div>
</div>

<!-- Toast -->
{#if toast}
  <div class="fixed bottom-6 right-6 bg-gray-800 border border-gray-700 text-gray-200 px-4 py-2 rounded-lg shadow-lg text-sm">
    {toast}
  </div>
{/if}
