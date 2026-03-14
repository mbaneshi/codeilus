<script lang="ts">
  import { onMount } from 'svelte';
  import { fetchCommunities, fetchProcesses } from '$lib/api';
  import type { Community, ProcessFlow } from '$lib/types';

  let loading = $state(true);
  let communities = $state<Community[]>([]);
  let processes = $state<ProcessFlow[]>([]);
  let expandedProcesses = $state<Set<number>>(new Set());

  function toggleProcess(id: number) {
    const next = new Set(expandedProcesses);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    expandedProcesses = next;
  }

  onMount(async () => {
    const [c, p] = await Promise.all([fetchCommunities(), fetchProcesses()]);
    communities = c;
    processes = p;
    loading = false;
  });
</script>

<div class="p-6 max-w-5xl mx-auto">
  <h1 class="text-2xl font-bold mb-6">Diagrams</h1>

  {#if loading}
    <p class="text-gray-400 animate-pulse">Loading...</p>
  {:else if communities.length === 0 && processes.length === 0}
    <div class="text-center py-16">
      <p class="text-gray-400 text-lg mb-2">No diagram data</p>
      <p class="text-gray-500">Run <code class="text-indigo-400 font-mono">codeilus analyze ./repo</code> first</p>
    </div>
  {:else}
    <!-- Communities -->
    {#if communities.length > 0}
      <h2 class="text-lg font-semibold mb-3">Communities</h2>
      <div class="grid grid-cols-2 gap-4 mb-8">
        {#each communities as community}
          <div class="card">
            <h3 class="text-base font-semibold text-gray-100 mb-2">{community.label}</h3>
            <div class="flex items-center gap-3 mb-2">
              <span class="text-sm text-gray-400">{community.member_count} members</span>
            </div>
            <div class="flex items-center gap-2">
              <span class="text-xs text-gray-500">Cohesion</span>
              <div class="flex-1 bg-gray-800 rounded-full h-2 overflow-hidden">
                <div
                  class="h-full rounded-full bg-indigo-500"
                  style="width: {Math.min(community.cohesion * 100, 100)}%"
                ></div>
              </div>
              <span class="text-xs text-gray-400">{(community.cohesion * 100).toFixed(0)}%</span>
            </div>
          </div>
        {/each}
      </div>
    {/if}

    <!-- Processes -->
    {#if processes.length > 0}
      <h2 class="text-lg font-semibold mb-3">Process Flows</h2>
      <div class="space-y-3">
        {#each processes as process}
          <div class="card">
            <button
              class="w-full text-left flex items-center gap-2"
              onclick={() => toggleProcess(process.id)}
            >
              <span class="text-gray-500">{expandedProcesses.has(process.id) ? '\u25BE' : '\u25B8'}</span>
              <h3 class="text-base font-semibold text-gray-100">{process.name}</h3>
              <span class="text-xs text-gray-500 ml-auto">{process.steps.length} steps</span>
            </button>

            {#if expandedProcesses.has(process.id)}
              <div class="mt-3 ml-6 border-l-2 border-gray-700 pl-4 space-y-3">
                {#each process.steps.sort((a, b) => a.order - b.order) as step}
                  <div class="flex items-start gap-3">
                    <span class="text-xs text-gray-500 mt-0.5 font-mono w-6 shrink-0">{step.order}.</span>
                    <div>
                      <div class="text-sm font-mono text-indigo-400">{step.symbol_name}</div>
                      <div class="text-xs text-gray-400">{step.description}</div>
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  {/if}
</div>

<style>
  @reference "tailwindcss";
  .card {
    @apply p-4 bg-gray-900 border border-gray-800 rounded-lg hover:border-indigo-500 transition-colors;
  }
</style>
