<script lang="ts">
  import { fetchHealth } from '$lib/api';

  let status = $state<'loading' | 'connected' | 'disconnected'>('loading');
  let llmAvailable = $state(false);

  async function check() {
    try {
      const h = await fetchHealth();
      status = h.status === 'ok' ? 'connected' : 'disconnected';
      llmAvailable = (h as Record<string, unknown>).llm_available as boolean ?? false;
    } catch {
      status = 'disconnected';
    }
  }

  $effect(() => {
    check();
    const interval = setInterval(check, 30000);
    return () => clearInterval(interval);
  });
</script>

<div class="flex items-center gap-3 px-3 py-2 text-[11px]">
  <div class="flex items-center gap-1.5">
    {#if status === 'loading'}
      <span class="w-2 h-2 rounded-full bg-gray-400 animate-pulse"></span>
      <span class="text-[var(--c-text-muted)]">Connecting...</span>
    {:else if status === 'connected'}
      <span class="w-2 h-2 rounded-full bg-emerald-400"></span>
      <span class="text-emerald-400">Connected</span>
    {:else}
      <span class="w-2 h-2 rounded-full bg-red-400"></span>
      <span class="text-red-400">Disconnected</span>
    {/if}
  </div>
  {#if status === 'connected'}
    <div class="flex items-center gap-1.5">
      <span class="w-1 h-1 rounded-full {llmAvailable ? 'bg-emerald-400' : 'bg-amber-400'}"></span>
      <span class="{llmAvailable ? 'text-emerald-400' : 'text-amber-400'}">
        LLM: {llmAvailable ? 'Available' : 'Disabled'}
      </span>
    </div>
  {/if}
</div>
