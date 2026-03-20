<script lang="ts">
  import { FolderTree, Network, BarChart3, Layers, Compass, GitBranch, Workflow } from 'lucide-svelte';
</script>

<div class="p-8 max-w-3xl mx-auto">
  <div class="flex items-center gap-3 mb-8">
    <div class="w-10 h-10 rounded-xl bg-teal-500/10 flex items-center justify-center">
      <Compass size={20} class="text-teal-400" />
    </div>
    <div>
      <h1 class="text-2xl font-bold tracking-tight">Explore</h1>
      <p class="text-sm text-[var(--c-text-secondary)]">Browse the codebase structure, relationships, and metrics</p>
    </div>
  </div>

  <div class="grid grid-cols-2 lg:grid-cols-3 gap-4">
    <a href="/explore/tree" class="explore-card group">
      <div class="w-10 h-10 rounded-xl bg-emerald-500/10 flex items-center justify-center mb-3 group-hover:bg-emerald-500/20 transition-colors">
        <FolderTree size={20} class="text-emerald-400" />
      </div>
      <h3 class="text-base font-semibold text-[var(--c-text-primary)] mb-1">File Tree</h3>
      <p class="text-sm text-[var(--c-text-secondary)] leading-relaxed">Browse files, folders, and the symbols within each file</p>
    </a>

    <a href="/explore/graph" class="explore-card group">
      <div class="w-10 h-10 rounded-xl bg-violet-500/10 flex items-center justify-center mb-3 group-hover:bg-violet-500/20 transition-colors">
        <Network size={20} class="text-violet-400" />
      </div>
      <h3 class="text-base font-semibold text-[var(--c-text-primary)] mb-1">Knowledge Graph</h3>
      <p class="text-sm text-[var(--c-text-secondary)] leading-relaxed">Interactive dependency graph with community detection</p>
    </a>

    <a href="/explore/metrics" class="explore-card group">
      <div class="w-10 h-10 rounded-xl bg-sky-500/10 flex items-center justify-center mb-3 group-hover:bg-sky-500/20 transition-colors">
        <BarChart3 size={20} class="text-sky-400" />
      </div>
      <h3 class="text-base font-semibold text-[var(--c-text-primary)] mb-1">Metrics</h3>
      <p class="text-sm text-[var(--c-text-secondary)] leading-relaxed">SLOC, language distribution, and largest files</p>
    </a>

    <a href="/explore/schematic" class="explore-card group">
      <div class="w-10 h-10 rounded-xl bg-amber-500/10 flex items-center justify-center mb-3 group-hover:bg-amber-500/20 transition-colors">
        <Workflow size={20} class="text-amber-400" />
      </div>
      <h3 class="text-base font-semibold text-[var(--c-text-primary)] mb-1">Schematic Explorer</h3>
      <p class="text-sm text-[var(--c-text-secondary)] leading-relaxed">Unified tree + graph with lazy loading, communities, and learning links</p>
    </a>

    <a href="/explore/diagrams" class="explore-card group">
      <div class="w-10 h-10 rounded-xl bg-rose-500/10 flex items-center justify-center mb-3 group-hover:bg-rose-500/20 transition-colors">
        <Layers size={20} class="text-rose-400" />
      </div>
      <h3 class="text-base font-semibold text-[var(--c-text-primary)] mb-1">Diagrams</h3>
      <p class="text-sm text-[var(--c-text-secondary)] leading-relaxed">Architecture modules and process flows</p>
    </a>
  </div>
</div>

<style>
  @reference "tailwindcss";
  .explore-card {
    @apply block p-5 bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl hover:border-[var(--c-border-hover)] hover:bg-[var(--surface-2)] transition-all;
  }
</style>
