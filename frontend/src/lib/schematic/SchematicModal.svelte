<script lang="ts">
  import { X } from 'lucide-svelte';

  let {
    open = false,
    title = '',
    onclose,
    children,
  }: {
    open?: boolean;
    title?: string;
    onclose?: () => void;
    children?: import('svelte').Snippet;
  } = $props();
</script>

{#if open}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="fixed inset-0 z-50 flex justify-end" onclick={onclose}>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="w-[420px] h-full bg-[var(--surface-1)] border-l border-[var(--c-border)] shadow-2xl shadow-black/40 overflow-auto animate-slide-in"
      onclick={(e) => e.stopPropagation()}
    >
      <div class="flex items-center justify-between px-5 py-4 border-b border-[var(--c-border)] sticky top-0 bg-[var(--surface-1)] z-10">
        <h2 class="text-base font-semibold text-[var(--c-text-primary)] truncate">{title}</h2>
        <button onclick={onclose} class="p-1.5 rounded-lg hover:bg-[var(--surface-2)] text-[var(--c-text-muted)] hover:text-[var(--c-text-primary)] transition-colors">
          <X size={16} />
        </button>
      </div>
      <div class="p-5 space-y-4">
        {#if children}
          {@render children()}
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  @keyframes slide-in {
    from { transform: translateX(100%); }
    to { transform: translateX(0); }
  }
  .animate-slide-in {
    animation: slide-in 0.2s ease-out;
  }
</style>
