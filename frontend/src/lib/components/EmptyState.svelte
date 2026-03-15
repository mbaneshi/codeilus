<script lang="ts">
  interface Props {
    title: string;
    description?: string;
    command?: string;
    icon?: string;
  }

  let { title, description, command, icon = '📦' }: Props = $props();
</script>

<div class="flex flex-col items-center justify-center py-20 text-center">
  <div class="text-4xl mb-4">{icon}</div>
  <h2 class="text-lg font-semibold text-[var(--c-text-primary)]">{title}</h2>
  {#if description}
    <p class="mt-2 text-sm text-[var(--c-text-secondary)] max-w-md">{description}</p>
  {/if}
  {#if command}
    <code class="mt-4 px-4 py-2.5 bg-[var(--surface-2)] border border-[var(--c-border)] rounded-lg text-sm font-mono text-[var(--c-text-primary)]">{command}</code>
  {/if}
</div>
