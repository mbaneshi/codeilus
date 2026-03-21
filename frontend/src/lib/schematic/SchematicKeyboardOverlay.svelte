<script lang="ts">
  let {
    visible = false,
    onclose,
  }: {
    visible: boolean;
    onclose: () => void;
  } = $props();

  const shortcuts = [
    { key: '⌘ K', desc: 'Search files & symbols' },
    { key: '⌘ /', desc: 'Ask AI about selected node' },
    { key: '1', desc: 'Tree mode' },
    { key: '2', desc: 'Graph mode' },
    { key: 'F', desc: 'Fit all in viewport' },
    { key: 'Esc', desc: 'Close panel / menu' },
    { key: '?', desc: 'Toggle this overlay' },
    { key: 'Del', desc: 'Hide selected node' },
  ];
</script>

{#if visible}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="fixed inset-0 z-[80] bg-black/50 flex items-center justify-center" onclick={onclose}>
    <div class="bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl shadow-2xl p-6 w-[340px]" onclick={(e) => e.stopPropagation()}>
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-sm font-semibold text-[var(--c-text-primary)]">Keyboard Shortcuts</h2>
        <button onclick={onclose} class="text-[var(--c-text-muted)] hover:text-[var(--c-text-primary)]">✕</button>
      </div>
      <div class="space-y-2">
        {#each shortcuts as s}
          <div class="flex items-center justify-between">
            <kbd class="px-2 py-0.5 bg-[var(--surface-2)] border border-[var(--c-border)] rounded text-[10px] font-mono text-[var(--c-text-primary)] min-w-[44px] text-center">{s.key}</kbd>
            <span class="text-xs text-[var(--c-text-secondary)]">{s.desc}</span>
          </div>
        {/each}
      </div>
    </div>
  </div>
{/if}
