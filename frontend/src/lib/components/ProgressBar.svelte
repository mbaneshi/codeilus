<script lang="ts">
  interface Props {
    value: number;
    max?: number;
    label?: string;
    showPercent?: boolean;
  }

  let { value, max = 100, label, showPercent = true }: Props = $props();
  let percent = $derived(Math.min(100, Math.round((value / max) * 100)));
</script>

<div class="space-y-1.5">
  {#if label || showPercent}
    <div class="flex justify-between text-xs">
      {#if label}<span class="text-[var(--c-text-secondary)]">{label}</span>{/if}
      {#if showPercent}<span class="text-[var(--c-text-muted)]">{percent}%</span>{/if}
    </div>
  {/if}
  <div class="h-2 bg-[var(--surface-3)] rounded-full overflow-hidden">
    <div
      class="h-full bg-[var(--c-accent)] rounded-full transition-all duration-500"
      style="width: {percent}%"
    ></div>
  </div>
</div>
