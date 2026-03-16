<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    text: string;
    children?: Snippet;
  }
  let { text, children }: Props = $props();
  let visible = $state(false);
</script>

<span class="relative inline-flex items-center">
  {#if children}{@render children()}{/if}
  <button
    class="ml-1 w-4 h-4 rounded-full bg-[var(--surface-3)] text-[var(--c-text-muted)] text-xs flex items-center justify-center hover:bg-[var(--c-accent)]/20 hover:text-[var(--c-accent)] transition-colors"
    onmouseenter={() => visible = true}
    onmouseleave={() => visible = false}
    onfocus={() => visible = true}
    onblur={() => visible = false}
  >
    ?
  </button>
  {#if visible}
    <div class="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-3 py-2 bg-[var(--surface-3)] border border-[var(--c-border)] rounded-lg text-xs text-[var(--c-text-secondary)] whitespace-nowrap z-50 shadow-lg">
      {text}
      <div class="absolute top-full left-1/2 -translate-x-1/2 -mt-px border-4 border-transparent border-t-[var(--surface-3)]"></div>
    </div>
  {/if}
</span>
