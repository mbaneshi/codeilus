<script lang="ts">
  import { Search, X } from 'lucide-svelte';
  import type { SchematicNode } from './types';

  let {
    nodes = [],
    onfocus,
    onhighlight,
  }: {
    nodes?: SchematicNode[];
    onfocus?: (nodeId: string) => void;
    onhighlight?: (ids: Set<string>) => void;
  } = $props();

  let query = $state('');
  let matches = $state<SchematicNode[]>([]);
  let matchIndex = $state(0);

  function doSearch() {
    if (query.trim().length < 2) {
      matches = [];
      onhighlight?.(new Set());
      return;
    }
    const q = query.toLowerCase();
    matches = nodes.filter(n => n.label.toLowerCase().includes(q));
    matchIndex = 0;
    onhighlight?.(new Set(matches.map(m => m.id)));
    if (matches.length > 0) {
      onfocus?.(matches[0].id);
    }
  }

  function nextMatch() {
    if (matches.length === 0) return;
    matchIndex = (matchIndex + 1) % matches.length;
    onfocus?.(matches[matchIndex].id);
  }

  function clear() {
    query = '';
    matches = [];
    matchIndex = 0;
    onhighlight?.(new Set());
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      nextMatch();
    }
    if (e.key === 'Escape') {
      clear();
    }
  }
</script>

<div class="absolute top-3 right-3 z-20 flex items-center gap-2">
  <div class="relative">
    <Search size={14} class="absolute left-2.5 top-1/2 -translate-y-1/2 text-[var(--c-text-muted)]" />
    <input
      type="text"
      placeholder="Search..."
      class="bg-[var(--surface-1)] border border-[var(--c-border)] rounded-lg pl-8 pr-8 py-1.5 text-sm text-[var(--c-text-primary)] placeholder:text-[var(--c-text-muted)] focus:border-[var(--c-accent)] focus:ring-1 focus:ring-[var(--c-accent)]/30 outline-none w-56"
      bind:value={query}
      oninput={doSearch}
      onkeydown={onKeydown}
    />
    {#if query}
      <button onclick={clear} class="absolute right-2 top-1/2 -translate-y-1/2 text-[var(--c-text-muted)] hover:text-[var(--c-text-primary)]">
        <X size={14} />
      </button>
    {/if}
  </div>
  {#if matches.length > 0}
    <span class="text-xs text-[var(--c-text-muted)] tabular-nums">{matchIndex + 1}/{matches.length}</span>
  {/if}
</div>
