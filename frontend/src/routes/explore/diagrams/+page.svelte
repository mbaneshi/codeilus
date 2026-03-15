<script lang="ts">
  import { fetchCommunities, fetchProcesses } from '$lib/api';
  import type { Community, ProcessFlow } from '$lib/types';
  import { Layers, ChevronDown, ChevronRight, ArrowLeft, Workflow } from 'lucide-svelte';

  let loading = $state(true);
  let error = $state<string | null>(null);
  let communities = $state<Community[]>([]);
  let processes = $state<ProcessFlow[]>([]);
  let expandedProcesses = $state<Set<number>>(new Set());

  function formatLabel(label: string): string {
    return label.replace(/^cluster_/, '').replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());
  }

  function toggleProcess(id: number) {
    const next = new Set(expandedProcesses);
    if (next.has(id)) next.delete(id); else next.add(id);
    expandedProcesses = next;
  }

  if (typeof window !== 'undefined') {
    Promise.all([fetchCommunities(), fetchProcesses()]).then(([c, p]) => {
      communities = c.sort((a, b) => b.member_count - a.member_count);
      processes = p;
      loading = false;
    }).catch((e) => {
      error = `Failed to load diagram data: ${e}`;
      loading = false;
    });
  }
</script>

<div class="p-8 max-w-5xl mx-auto">
  <div class="flex items-center gap-3 mb-8">
    <a href="/explore" class="w-8 h-8 rounded-lg bg-[var(--surface-2)] flex items-center justify-center text-[var(--c-text-muted)] hover:text-[var(--c-text-primary)] hover:bg-[var(--surface-3)] transition-all">
      <ArrowLeft size={16} />
    </a>
    <div class="w-10 h-10 rounded-xl bg-rose-500/10 flex items-center justify-center">
      <Layers size={20} class="text-rose-400" />
    </div>
    <div>
      <h1 class="text-2xl font-bold tracking-tight">Diagrams</h1>
      <p class="text-sm text-[var(--c-text-secondary)]">Architecture modules and process flows</p>
    </div>
  </div>

  {#if loading}
    <div class="space-y-4">
      {#each [1, 2, 3] as _}
        <div class="h-20 bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl animate-pulse"></div>
      {/each}
    </div>
  {:else if error}
    <div class="text-center py-20 bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl">
      <Layers size={40} class="text-red-400 mx-auto mb-4" />
      <p class="text-red-400 text-lg font-medium mb-2">Error loading diagrams</p>
      <p class="text-[var(--c-text-muted)] text-sm">{error}</p>
    </div>
  {:else if communities.length === 0 && processes.length === 0}
    <div class="text-center py-20 bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl">
      <Layers size={40} class="text-[var(--c-text-muted)] mx-auto mb-4" />
      <p class="text-[var(--c-text-secondary)] text-lg font-medium mb-2">No diagram data</p>
      <p class="text-[var(--c-text-muted)] text-sm">Run <code class="text-[var(--c-accent)] font-mono text-xs bg-[var(--c-accent)]/10 px-1.5 py-0.5 rounded">codeilus analyze ./repo</code> first</p>
    </div>
  {:else}
    {#if communities.length > 0}
      <section class="mb-10">
        <h2 class="text-sm font-semibold uppercase tracking-wider text-[var(--c-text-muted)] mb-4">Architecture — {communities.length} Modules</h2>
        <p class="text-sm text-[var(--c-text-secondary)] mb-5">Functional areas detected by community analysis. Higher cohesion means tighter internal coupling.</p>

        <!-- Bar chart -->
        <div class="bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl p-5 mb-6">
          {#each communities.slice(0, 12) as community}
            {@const maxCount = communities[0].member_count}
            <div class="flex items-center gap-3 mb-2.5 last:mb-0">
              <span class="text-sm text-[var(--c-text-secondary)] w-36 truncate font-medium" title={formatLabel(community.label)}>{formatLabel(community.label)}</span>
              <div class="flex-1 bg-[var(--surface-3)] rounded-full h-5 overflow-hidden">
                <div
                  class="h-full rounded-full bg-gradient-to-r from-indigo-500 to-indigo-400 flex items-center justify-end pr-2"
                  style="width: {Math.max(8, (community.member_count / maxCount) * 100)}%"
                >
                  <span class="text-[10px] font-mono text-white/80">{community.member_count}</span>
                </div>
              </div>
              <span class="text-xs text-[var(--c-text-muted)] w-10 text-right font-mono">{(community.cohesion * 100).toFixed(0)}%</span>
            </div>
          {/each}
        </div>

        <!-- Grid -->
        <div class="grid grid-cols-2 gap-3">
          {#each communities as community}
            <div class="p-4 bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl hover:border-[var(--c-border-hover)] transition-colors">
              <h3 class="text-sm font-semibold text-[var(--c-text-primary)] mb-2">{formatLabel(community.label)}</h3>
              <div class="flex items-center gap-3 mb-2.5">
                <span class="text-xs text-[var(--c-text-muted)]">{community.member_count} symbols</span>
              </div>
              <div class="flex items-center gap-2">
                <span class="text-[10px] text-[var(--c-text-muted)] uppercase tracking-wider">Cohesion</span>
                <div class="flex-1 bg-[var(--surface-3)] rounded-full h-1.5 overflow-hidden">
                  <div class="h-full rounded-full bg-[var(--c-accent)]" style="width: {Math.min(community.cohesion * 100, 100)}%"></div>
                </div>
                <span class="text-[11px] text-[var(--c-text-muted)] font-mono">{(community.cohesion * 100).toFixed(0)}%</span>
              </div>
            </div>
          {/each}
        </div>
      </section>
    {/if}

    {#if processes.length > 0}
      <section>
        <h2 class="text-sm font-semibold uppercase tracking-wider text-[var(--c-text-muted)] mb-4">Process Flows — {processes.length} detected</h2>
        <p class="text-sm text-[var(--c-text-secondary)] mb-5">Execution paths through the codebase, starting from entry points.</p>
        <div class="space-y-3">
          {#each processes as process}
            {@const isExpanded = expandedProcesses.has(process.id)}
            <div class="bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl hover:border-[var(--c-border-hover)] transition-colors {isExpanded ? 'ring-1 ring-[var(--c-accent)]/20 border-indigo-500/30' : ''}">
              <button class="w-full text-left p-4 flex items-center gap-3" onclick={() => toggleProcess(process.id)}>
                <div class="w-8 h-8 rounded-lg bg-[var(--surface-2)] flex items-center justify-center text-[var(--c-text-muted)]">
                  <Workflow size={16} />
                </div>
                <h3 class="text-sm font-semibold text-[var(--c-text-primary)] flex-1">{formatLabel(process.name)}</h3>
                <span class="text-xs text-[var(--c-text-muted)] mr-2">{process.steps.length} steps</span>
                {#if isExpanded}
                  <ChevronDown size={16} class="text-[var(--c-text-muted)]" />
                {:else}
                  <ChevronRight size={16} class="text-[var(--c-text-muted)]" />
                {/if}
              </button>

              {#if isExpanded}
                <div class="px-4 pb-4">
                  <div class="ml-4 border-l-2 border-indigo-500/20 pl-5 space-y-4">
                    {#each process.steps.sort((a, b) => a.order - b.order) as step, idx}
                      <div class="relative flex items-start gap-3">
                        <div class="absolute -left-[27px] w-3 h-3 rounded-full bg-[var(--c-accent)]/20 border-2 border-indigo-500"></div>
                        <div>
                          <div class="text-sm font-mono text-[var(--c-accent)] font-medium">{step.symbol_name}()</div>
                          {#if step.description}
                            <div class="text-xs text-[var(--c-text-muted)] mt-0.5">{step.description}</div>
                          {/if}
                        </div>
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}
            </div>
          {/each}
        </div>
      </section>
    {/if}
  {/if}
</div>
