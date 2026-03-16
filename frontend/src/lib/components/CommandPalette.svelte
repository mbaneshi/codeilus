<script lang="ts">
  import { goto } from '$app/navigation';

  let open = $state(false);
  let query = $state('');

  const commands = [
    { label: 'Dashboard', path: '/', keywords: 'home overview' },
    { label: 'Graph Explorer', path: '/explore/graph', keywords: 'dependencies nodes edges' },
    { label: 'File Tree', path: '/explore/tree', keywords: 'files folders' },
    { label: 'Metrics', path: '/explore/metrics', keywords: 'complexity sloc' },
    { label: 'Diagrams', path: '/explore/diagrams', keywords: 'architecture' },
    { label: 'Learn', path: '/learn', keywords: 'chapters curriculum' },
    { label: 'Ask AI', path: '/ask', keywords: 'chat question llm' },
    { label: 'Settings', path: '/settings', keywords: 'preferences config' },
  ];

  let filtered = $derived(
    query.length === 0
      ? commands
      : commands.filter(c =>
          c.label.toLowerCase().includes(query.toLowerCase()) ||
          c.keywords.includes(query.toLowerCase())
        )
  );

  let selectedIndex = $state(0);

  function handleKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
      e.preventDefault();
      open = !open;
      query = '';
      selectedIndex = 0;
    }
    if (!open) return;
    if (e.key === 'Escape') { open = false; }
    if (e.key === 'ArrowDown') { e.preventDefault(); selectedIndex = Math.min(selectedIndex + 1, filtered.length - 1); }
    if (e.key === 'ArrowUp') { e.preventDefault(); selectedIndex = Math.max(selectedIndex - 1, 0); }
    if (e.key === 'Enter' && filtered[selectedIndex]) {
      goto(filtered[selectedIndex].path);
      open = false;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
<div class="fixed inset-0 bg-black/50 z-50 flex items-start justify-center pt-[20vh]" onclick={() => open = false}>
  <div class="bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl w-full max-w-md shadow-2xl overflow-hidden" onclick={(e) => e.stopPropagation()}>
    <input
      type="text"
      bind:value={query}
      placeholder="Search commands..."
      class="w-full px-4 py-3 bg-transparent text-[var(--c-text-primary)] text-sm border-b border-[var(--c-border)] outline-none placeholder-[var(--c-text-muted)]"
      autofocus
    />
    <div class="max-h-64 overflow-y-auto py-1">
      {#each filtered as cmd, i}
        <button
          class="w-full px-4 py-2.5 text-left text-sm flex items-center gap-2 transition-colors {i === selectedIndex ? 'bg-[var(--c-accent)]/10 text-[var(--c-accent)]' : 'text-[var(--c-text-secondary)] hover:bg-[var(--surface-2)]'}"
          onclick={() => { goto(cmd.path); open = false; }}
        >
          {cmd.label}
        </button>
      {/each}
      {#if filtered.length === 0}
        <div class="px-4 py-3 text-sm text-[var(--c-text-muted)]">No results</div>
      {/if}
    </div>
    <div class="px-4 py-2 border-t border-[var(--c-border)] text-xs text-[var(--c-text-muted)] flex gap-3">
      <span><kbd class="bg-[var(--surface-3)] px-1 rounded">&#8593;&#8595;</kbd> navigate</span>
      <span><kbd class="bg-[var(--surface-3)] px-1 rounded">&#8629;</kbd> select</span>
      <span><kbd class="bg-[var(--surface-3)] px-1 rounded">esc</kbd> close</span>
    </div>
  </div>
</div>
{/if}
