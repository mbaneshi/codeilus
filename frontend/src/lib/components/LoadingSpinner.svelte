<script lang="ts">
  interface Props {
    size?: number;
    message?: string;
  }

  let { size = 24, message }: Props = $props();
</script>

<div class="flex flex-col items-center justify-center gap-3 py-12">
  <svg
    width={size}
    height={size}
    viewBox="0 0 24 24"
    fill="none"
    class="animate-spin text-[var(--c-accent)]"
  >
    <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" opacity="0.2" />
    <path d="M12 2a10 10 0 0 1 10 10" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
  </svg>
  {#if message}
    <p class="text-sm text-[var(--c-text-secondary)]">{message}</p>
  {/if}
</div>
