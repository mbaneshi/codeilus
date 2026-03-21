<script lang="ts">
  import type { SchematicNode, SourceResponse } from '$lib/types';
  import { fetchFileSource, fetchSymbol } from '$lib/api';
  import { LoadingSpinner } from '$lib/components';

  let {
    node,
    x = 300,
    y = 200,
    onclose,
    onviewfull,
  }: {
    node: SchematicNode;
    x: number;
    y: number;
    onclose: () => void;
    onviewfull?: () => void;
  } = $props();

  let sourceData = $state<SourceResponse | null>(null);
  let loading = $state(true);

  // Load source
  if (node.file_id) {
    (async () => {
      let start: number | undefined;
      let end: number | undefined;
      if (node.symbol_id) {
        const sym = await fetchSymbol(node.symbol_id);
        if (sym) { start = Math.max(1, sym.start_line - 3); end = sym.end_line + 3; }
      }
      sourceData = await fetchFileSource(node.file_id!, start, end);
      loading = false;
    })();
  } else {
    loading = false;
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div class="fixed inset-0 z-[55]" onclick={onclose}>
  <div
    class="absolute bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl shadow-2xl overflow-hidden"
    style="left: {Math.min(x, globalThis.innerWidth - 500)}px; top: {Math.min(y, globalThis.innerHeight - 420)}px; width: 480px; max-height: 400px;"
    onclick={(e) => e.stopPropagation()}
  >
    <!-- Header -->
    <div class="flex items-center justify-between px-3 py-2 border-b border-[var(--c-border)] bg-[var(--surface-2)]">
      <div class="flex items-center gap-2 truncate">
        <span class="text-[10px] uppercase text-[var(--c-text-muted)]">{node.kind || node.type}</span>
        <span class="text-xs font-semibold text-[var(--c-text-primary)] font-mono truncate">{node.label}</span>
      </div>
      <div class="flex items-center gap-2 shrink-0">
        {#if onviewfull}
          <button onclick={onviewfull} class="text-[10px] text-[var(--c-accent)] hover:underline">Full source</button>
        {/if}
        <button onclick={onclose} class="text-[var(--c-text-muted)] hover:text-[var(--c-text-primary)]">✕</button>
      </div>
    </div>

    <!-- Source -->
    <div class="overflow-auto" style="max-height: 350px;">
      {#if loading}
        <div class="p-6"><LoadingSpinner /></div>
      {:else if sourceData}
        <pre class="text-[11px] font-mono p-3 text-[var(--c-text-secondary)] leading-relaxed">{sourceData.lines.map(l => `${String(l.number).padStart(4)} │ ${l.content}`).join('\n')}</pre>
      {:else}
        <p class="p-4 text-sm text-[var(--c-text-muted)]">No source available.</p>
      {/if}
    </div>
  </div>
</div>
