<script lang="ts">
  interface Props {
    message: string;
    variant?: 'error' | 'warning' | 'info';
    dismissible?: boolean;
  }

  let { message, variant = 'error', dismissible = true }: Props = $props();
  let visible = $state(true);

  const colors = {
    error: { bg: 'bg-red-900/20', border: 'border-red-500/30', text: 'text-red-300' },
    warning: { bg: 'bg-amber-900/20', border: 'border-amber-500/30', text: 'text-amber-300' },
    info: { bg: 'bg-blue-900/20', border: 'border-blue-500/30', text: 'text-blue-300' },
  };

  let c = $derived(colors[variant]);
</script>

{#if visible}
  <div class="flex items-center justify-between gap-3 px-4 py-3 rounded-lg border {c.bg} {c.border}">
    <p class="text-sm {c.text}">{message}</p>
    {#if dismissible}
      <button
        class="text-[var(--c-text-muted)] hover:text-[var(--c-text-secondary)] text-lg leading-none"
        onclick={() => visible = false}
      >&times;</button>
    {/if}
  </div>
{/if}
