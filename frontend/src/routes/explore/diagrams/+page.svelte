<script lang="ts">
  import { fetchCommunities, fetchProcesses } from '$lib/api';
  import type { Community, ProcessFlow } from '$lib/types';

  let loading = $state(true);
  let communities = $state<Community[]>([]);
  let processes = $state<ProcessFlow[]>([]);
  let expandedProcesses = $state<Set<number>>(new Set());

  function formatLabel(label: string): string {
    return label
      .replace(/^cluster_/, '')
      .replace(/_/g, ' ')
      .replace(/\b\w/g, (c) => c.toUpperCase());
  }

  function toggleProcess(id: number) {
    const next = new Set(expandedProcesses);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    expandedProcesses = next;
  }

  if (typeof window !== 'undefined') {
    Promise.all([fetchCommunities(), fetchProcesses()]).then(([c, p]) => {
      communities = c.sort((a, b) => b.member_count - a.member_count);
      processes = p;
      loading = false;
    });
  }
</script>

<div class="p-6 max-w-5xl mx-auto">
  <div class="flex items-center gap-3 mb-6">
    <a href="/explore" class="text-gray-500 hover:text-gray-300 transition-colors">&larr;</a>
    <h1 class="text-2xl font-bold">Diagrams</h1>
  </div>

  {#if loading}
    <p class="text-gray-400 animate-pulse">Loading...</p>
  {:else if communities.length === 0 && processes.length === 0}
    <div class="text-center py-16">
      <p class="text-gray-400 text-lg mb-2">No diagram data</p>
      <p class="text-gray-500">Run <code class="text-indigo-400 font-mono">codeilus analyze ./repo</code> first</p>
    </div>
  {:else}
    <!-- Architecture Overview -->
    {#if communities.length > 0}
      <h2 class="text-lg font-semibold mb-3">Architecture — {communities.length} Modules</h2>
      <p class="text-sm text-gray-400 mb-4">Functional areas detected by community analysis. Higher cohesion means tighter internal coupling.</p>

      <!-- Top modules bar chart -->
      <div class="mb-6 bg-gray-900 border border-gray-800 rounded-lg p-4">
        {#each communities.slice(0, 10) as community}
          {@const maxCount = communities[0].member_count}
          <div class="flex items-center gap-3 mb-2 last:mb-0">
            <span class="text-sm text-gray-300 w-40 truncate" title={formatLabel(community.label)}>{formatLabel(community.label)}</span>
            <div class="flex-1 bg-gray-800 rounded-full h-4 overflow-hidden">
              <div
                class="h-full rounded-full bg-indigo-500 flex items-center justify-end pr-2"
                style="width: {Math.max(5, (community.member_count / maxCount) * 100)}%"
              >
                <span class="text-[10px] text-white font-mono">{community.member_count}</span>
              </div>
            </div>
            <span class="text-xs text-gray-500 w-12 text-right">{(community.cohesion * 100).toFixed(0)}%</span>
          </div>
        {/each}
      </div>

      <!-- Community grid -->
      <div class="grid grid-cols-2 gap-4 mb-8">
        {#each communities as community}
          <div class="card">
            <h3 class="text-base font-semibold text-gray-100 mb-2">{formatLabel(community.label)}</h3>
            <div class="flex items-center gap-3 mb-2">
              <span class="text-sm text-gray-400">{community.member_count} symbols</span>
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
      <h2 class="text-lg font-semibold mb-3">Process Flows — {processes.length} detected</h2>
      <p class="text-sm text-gray-400 mb-4">Execution paths through the codebase, starting from entry points.</p>
      <div class="space-y-3">
        {#each processes as process}
          <div class="card">
            <button
              class="w-full text-left flex items-center gap-2"
              onclick={() => toggleProcess(process.id)}
            >
              <span class="text-gray-500 text-xs">{expandedProcesses.has(process.id) ? '\u25BE' : '\u25B8'}</span>
              <h3 class="text-base font-semibold text-gray-100">{formatLabel(process.name)}</h3>
              <span class="text-xs text-gray-500 ml-auto">{process.steps.length} steps</span>
            </button>

            {#if expandedProcesses.has(process.id)}
              <div class="mt-3 ml-6 border-l-2 border-indigo-500/30 pl-4 space-y-3">
                {#each process.steps.sort((a, b) => a.order - b.order) as step, idx}
                  <div class="flex items-start gap-3">
                    <div class="w-6 h-6 rounded-full bg-indigo-600/20 text-indigo-400 flex items-center justify-center text-xs font-bold shrink-0 mt-0.5">
                      {idx + 1}
                    </div>
                    <div>
                      <div class="text-sm font-mono text-indigo-400">{step.symbol_name}()</div>
                      {#if step.description}
                        <div class="text-xs text-gray-400 mt-0.5">{step.description}</div>
                      {/if}
                    </div>
                    {#if idx < process.steps.length - 1}
                      <span class="text-gray-600 ml-auto">&darr;</span>
                    {/if}
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
    @apply p-4 bg-gray-900 border border-gray-800 rounded-lg hover:border-indigo-500/50 transition-colors;
  }
</style>
