<script lang="ts">
  import { onMount } from 'svelte';
  import { fetchCommunities } from '$lib/api';
  import type { Community } from '$lib/types';

  let loading = $state(true);
  let chapters = $state<Community[]>([]);

  onMount(async () => {
    chapters = await fetchCommunities();
    loading = false;
  });
</script>

<div class="p-6 max-w-3xl mx-auto">
  <h1 class="text-2xl font-bold mb-2">Learning Path</h1>
  <p class="text-gray-400 mb-6">Work through the codebase one community at a time.</p>

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
        <div class="card">
          <div class="flex items-start gap-4">
            <div class="w-10 h-10 rounded-full bg-indigo-600/20 text-indigo-400 flex items-center justify-center text-lg font-bold shrink-0">
              {i + 1}
            </div>
            <div class="flex-1 min-w-0">
              <h3 class="text-base font-semibold text-gray-100 mb-1">{chapter.label}</h3>
              <p class="text-sm text-gray-400 mb-3">{chapter.member_count} topics</p>

              <!-- Progress bar placeholder -->
              <div class="flex items-center gap-2 mb-3">
                <div class="flex-1 bg-gray-800 rounded-full h-1.5 overflow-hidden">
                  <div class="h-full rounded-full bg-gray-700" style="width: 0%"></div>
                </div>
                <span class="text-xs text-gray-500">0%</span>
              </div>

              <button class="text-sm px-4 py-1.5 bg-indigo-600 rounded hover:bg-indigo-500 transition-colors text-white">
                Start
              </button>
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
    @apply p-4 bg-gray-900 border border-gray-800 rounded-lg hover:border-indigo-500 transition-colors;
  }
</style>
