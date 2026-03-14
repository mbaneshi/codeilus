<script lang="ts">
  import { fetchCommunities, fetchGraph } from '$lib/api';
  import type { Community, GraphNode } from '$lib/types';

  let loading = $state(true);
  let chapters = $state<Community[]>([]);
  let expandedId = $state<number | null>(null);
  let symbolMap = $state<Map<number, GraphNode>>(new Map());

  function formatLabel(label: string): string {
    return label
      .replace(/^cluster_/, '')
      .replace(/_/g, ' ')
      .replace(/\b\w/g, (c) => c.toUpperCase());
  }

  function kindBadge(kind: string): string {
    switch (kind.toLowerCase()) {
      case 'function': return 'bg-indigo-600';
      case 'class': return 'bg-pink-600';
      case 'method': return 'bg-teal-600';
      case 'struct': return 'bg-amber-600';
      case 'trait': return 'bg-purple-600';
      case 'impl': return 'bg-cyan-600';
      default: return 'bg-gray-600';
    }
  }

  function toggleChapter(id: number) {
    expandedId = expandedId === id ? null : id;
  }

  function getMemberSymbols(members: number[]): GraphNode[] {
    return members
      .map((id) => symbolMap.get(id))
      .filter((s): s is GraphNode => s !== undefined)
      .sort((a, b) => {
        // Sort: functions first, then by name
        const kindOrder = ['Function', 'Method', 'Struct', 'Class', 'Trait', 'Impl'];
        const ai = kindOrder.indexOf(a.kind);
        const bi = kindOrder.indexOf(b.kind);
        if (ai !== bi) return (ai === -1 ? 99 : ai) - (bi === -1 ? 99 : bi);
        return a.name.localeCompare(b.name);
      });
  }

  if (typeof window !== 'undefined') {
    Promise.all([fetchCommunities(), fetchGraph()]).then(([comms, graph]) => {
      chapters = comms.sort((a, b) => b.member_count - a.member_count);
      // Build symbol lookup from graph nodes
      const map = new Map<number, GraphNode>();
      for (const node of graph.nodes) {
        map.set(node.id, node);
      }
      symbolMap = map;
      loading = false;
    });
  }
</script>

<div class="p-6 max-w-4xl mx-auto">
  <h1 class="text-2xl font-bold mb-2">Learning Path</h1>
  <p class="text-gray-400 mb-6">Work through the codebase one module at a time. Each chapter covers a functional area.</p>

  {#if loading}
    <p class="text-gray-400 animate-pulse">Loading...</p>
  {:else if chapters.length === 0}
    <div class="text-center py-16">
      <p class="text-gray-400 text-lg mb-2">No chapters yet</p>
      <p class="text-gray-500">Run <code class="text-indigo-400 font-mono">codeilus analyze ./repo</code> first</p>
    </div>
  {:else}
    <div class="space-y-4">
      {#each chapters as chapter, i}
        {@const members = getMemberSymbols(chapter.members || [])}
        {@const kindCounts = members.reduce((acc, m) => { acc[m.kind] = (acc[m.kind] || 0) + 1; return acc; }, {} as Record<string, number>)}
        <div class="card">
          <div class="flex items-start gap-4">
            <div class="w-10 h-10 rounded-full bg-indigo-600/20 text-indigo-400 flex items-center justify-center text-lg font-bold shrink-0">
              {i + 1}
            </div>
            <div class="flex-1 min-w-0">
              <h3 class="text-base font-semibold text-gray-100 mb-1">{formatLabel(chapter.label)}</h3>
              <div class="flex items-center gap-3 text-sm text-gray-400 mb-3">
                <span>{chapter.member_count} symbols</span>
                <span class="text-gray-600">|</span>
                <span>Cohesion {(chapter.cohesion * 100).toFixed(0)}%</span>
                {#if Object.keys(kindCounts).length > 0}
                  <span class="text-gray-600">|</span>
                  {#each Object.entries(kindCounts).slice(0, 3) as [kind, count]}
                    <span class="text-xs px-1.5 py-0.5 rounded {kindBadge(kind)} text-white">{count} {kind}{count > 1 ? 's' : ''}</span>
                  {/each}
                {/if}
              </div>

              <!-- Progress bar -->
              <div class="flex items-center gap-2 mb-3">
                <div class="flex-1 bg-gray-800 rounded-full h-1.5 overflow-hidden">
                  <div class="h-full rounded-full bg-indigo-500" style="width: 0%"></div>
                </div>
                <span class="text-xs text-gray-500">0%</span>
              </div>

              <button
                class="text-sm px-4 py-1.5 bg-indigo-600 rounded hover:bg-indigo-500 transition-colors text-white"
                onclick={() => toggleChapter(chapter.id)}
              >
                {expandedId === chapter.id ? 'Collapse' : 'Explore'}
              </button>

              {#if expandedId === chapter.id}
                <div class="mt-4 pt-4 border-t border-gray-800">
                  <p class="text-sm text-gray-300 mb-4">
                    The <strong>{formatLabel(chapter.label)}</strong> module contains {chapter.member_count} symbols
                    that work together (cohesion: {(chapter.cohesion * 100).toFixed(1)}%).
                  </p>

                  {#if members.length > 0}
                    <!-- Group by kind -->
                    {#each Object.entries(kindCounts) as [kind, count]}
                      <div class="mb-3">
                        <h4 class="text-xs font-semibold text-gray-500 uppercase mb-2">{kind}s ({count})</h4>
                        <div class="flex flex-wrap gap-2">
                          {#each members.filter((m) => m.kind === kind).slice(0, 15) as sym}
                            <span class="inline-flex items-center gap-1.5 text-xs px-2.5 py-1 bg-gray-800 rounded-md border border-gray-700 hover:border-indigo-500 transition-colors cursor-default">
                              <span class="w-1.5 h-1.5 rounded-full {kindBadge(sym.kind)}"></span>
                              <span class="font-mono text-gray-200">{sym.name}</span>
                            </span>
                          {/each}
                          {#if members.filter((m) => m.kind === kind).length > 15}
                            <span class="text-xs px-2 py-1 text-gray-500">+{members.filter((m) => m.kind === kind).length - 15} more</span>
                          {/if}
                        </div>
                      </div>
                    {/each}
                  {:else}
                    <p class="text-sm text-gray-500">Symbol details not available.</p>
                  {/if}
                </div>
              {/if}
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  @reference "tailwindcss";
  .card {
    @apply p-4 bg-gray-900 border border-gray-800 rounded-lg hover:border-indigo-500/50 transition-colors;
  }
</style>
