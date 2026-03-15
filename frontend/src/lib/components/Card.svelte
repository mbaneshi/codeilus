<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    title?: string;
    children: Snippet;
    class?: string;
  }

  let { title, children, class: className = '' }: Props = $props();
</script>

<div class="bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl p-5 {className}">
  {#if title}
    <h3 class="text-sm font-semibold text-[var(--c-text-primary)] mb-3">{title}</h3>
  {/if}
  {@render children()}
</div>
