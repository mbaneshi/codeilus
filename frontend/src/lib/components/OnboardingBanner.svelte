<script lang="ts">
  interface Props {
    show: boolean;
  }
  let { show }: Props = $props();
</script>

{#if show}
<div class="bg-gradient-to-r from-[var(--c-accent)]/10 to-transparent border border-[var(--c-accent)]/30 rounded-xl p-6 mb-6">
  <h2 class="text-lg font-semibold text-[var(--c-text-primary)] mb-3">Welcome to Codeilus</h2>
  <p class="text-sm text-[var(--c-text-secondary)] mb-4">
    Get started by analyzing a codebase. Codeilus will parse your code, build a knowledge graph,
    and create an interactive learning experience.
  </p>
  <div class="space-y-2 text-sm text-[var(--c-text-secondary)]">
    <div class="flex items-start gap-2">
      <span class="text-[var(--c-accent)] font-mono mt-0.5">1.</span>
      <span>Run <code class="bg-[var(--surface-3)] px-2 py-0.5 rounded text-xs">codeilus analyze ./your-repo</code> to parse and analyze</span>
    </div>
    <div class="flex items-start gap-2">
      <span class="text-[var(--c-accent)] font-mono mt-0.5">2.</span>
      <span>Run <code class="bg-[var(--surface-3)] px-2 py-0.5 rounded text-xs">codeilus serve</code> to start the web UI</span>
    </div>
    <div class="flex items-start gap-2">
      <span class="text-[var(--c-accent)] font-mono mt-0.5">3.</span>
      <span>Explore the graph, learn chapters, and take quizzes!</span>
    </div>
  </div>
</div>
{/if}
