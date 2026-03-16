<script lang="ts">
  import { page } from '$app/stores';

  let segments = $derived.by(() => {
    const path = $page.url.pathname;
    if (path === '/') return [];
    const parts = path.split('/').filter(Boolean);
    return parts.map((part, i) => ({
      label: part.charAt(0).toUpperCase() + part.slice(1).replace(/-/g, ' '),
      href: '/' + parts.slice(0, i + 1).join('/'),
      current: i === parts.length - 1,
    }));
  });
</script>

{#if segments.length > 0}
<nav class="flex items-center gap-1.5 text-xs text-[var(--c-text-muted)] mb-4">
  <a href="/" class="hover:text-[var(--c-text-secondary)] transition-colors">Home</a>
  {#each segments as seg}
    <span>/</span>
    {#if seg.current}
      <span class="text-[var(--c-text-secondary)]">{seg.label}</span>
    {:else}
      <a href={seg.href} class="hover:text-[var(--c-text-secondary)] transition-colors">{seg.label}</a>
    {/if}
  {/each}
</nav>
{/if}
