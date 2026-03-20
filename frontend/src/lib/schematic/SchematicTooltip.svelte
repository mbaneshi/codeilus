<script lang="ts">
  import type { SchematicNode, SchematicCommunity } from '$lib/types';

  let {
    node = null,
    x = 0,
    y = 0,
    communities = [],
  }: {
    node: SchematicNode | null;
    x: number;
    y: number;
    communities: SchematicCommunity[];
  } = $props();

  const KIND_LABELS: Record<string, string> = {
    function: 'Function', method: 'Method', class: 'Class', struct: 'Struct',
    enum: 'Enum', trait: 'Trait', interface: 'Interface', impl: 'Impl', module: 'Module',
  };

  let tipX = $derived(x + 14);
  let tipY = $derived(y + 14);
</script>

{#if node}
  <div
    class="fixed z-[60] pointer-events-none px-3 py-2 rounded-lg shadow-xl border max-w-[280px]"
    style="
      left: {tipX}px; top: {tipY}px;
      background: var(--surface-1); border-color: var(--c-border);
      box-shadow: 0 8px 30px rgba(0,0,0,0.3);
    "
  >
    <!-- Type badge + name -->
    <div class="flex items-center gap-2 mb-1">
      <span class="text-[10px] uppercase tracking-wider px-1.5 py-0.5 rounded font-medium"
        style="background: var(--surface-3); color: var(--c-text-muted);">{node.type}</span>
      <span class="text-xs font-semibold text-[var(--c-text-primary)] truncate">{node.label}</span>
    </div>

    <!-- Details row -->
    <div class="flex flex-wrap gap-x-3 gap-y-0.5 text-[10px] text-[var(--c-text-muted)]">
      {#if node.language}
        <span>{node.language}</span>
      {/if}
      {#if node.sloc}
        <span>{node.sloc} loc</span>
      {/if}
      {#if node.kind}
        <span>{KIND_LABELS[node.kind] || node.kind}</span>
      {/if}
      {#if node.symbol_count}
        <span>{node.symbol_count} symbols</span>
      {/if}
    </div>

    <!-- Community -->
    {#if node.community_label}
      <div class="flex items-center gap-1.5 mt-1.5">
        <span class="w-2 h-2 rounded-full shrink-0" style="background: {node.community_color || 'var(--c-text-muted)'}"></span>
        <span class="text-[10px] text-[var(--c-text-secondary)]">{node.community_label}</span>
      </div>
    {/if}

    <!-- Signature -->
    {#if node.signature}
      <div class="mt-1.5 text-[10px] font-mono text-[var(--c-text-muted)] truncate">{node.signature.slice(0, 60)}</div>
    {/if}
  </div>
{/if}
