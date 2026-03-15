<script lang="ts">
  import { fetchHealth, fetchLlmStatus } from '$lib/api';
  import { Settings, Server, Cpu, Palette, Info, RotateCcw } from 'lucide-svelte';

  let healthStatus = $state('checking...');
  let llmAvailable = $state<boolean | null>(null);
  let llmProvider = $state('');
  let apiBase = $state('/api/v1');
  let showResetConfirm = $state(false);

  if (typeof window !== 'undefined') {
    Promise.all([fetchHealth(), fetchLlmStatus()]).then(([health, llm]) => {
      healthStatus = health.status;
      llmAvailable = llm.available;
      llmProvider = llm.provider || '';
    });
  }

  function resetOnboarding() {
    localStorage.removeItem('codeilus-onboarded');
    showResetConfirm = true;
    setTimeout(() => { showResetConfirm = false; }, 2000);
  }
</script>

<div class="p-8 max-w-2xl mx-auto">
  <div class="flex items-center gap-3 mb-8">
    <div class="w-10 h-10 rounded-xl bg-[var(--surface-2)] flex items-center justify-center">
      <Settings size={20} class="text-[var(--c-text-secondary)]" />
    </div>
    <div>
      <h1 class="text-2xl font-bold tracking-tight">Settings</h1>
      <p class="text-sm text-[var(--c-text-secondary)]">Configuration and system status</p>
    </div>
  </div>

  <!-- System Status -->
  <section class="mb-8">
    <h2 class="section-title">
      <Server size={15} />
      System Status
    </h2>
    <div class="bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl divide-y divide-[var(--c-border)] overflow-hidden">
      <div class="flex items-center justify-between px-5 py-4">
        <div>
          <div class="text-sm font-medium text-[var(--c-text-primary)]">API Server</div>
          <div class="text-xs text-[var(--c-text-muted)] mt-0.5">Backend connection status</div>
        </div>
        <div class="flex items-center gap-2">
          <span class="w-2 h-2 rounded-full {healthStatus === 'ok' ? 'bg-emerald-400' : 'bg-red-400'}"></span>
          <span class="text-sm {healthStatus === 'ok' ? 'text-emerald-400' : 'text-red-400'}">
            {healthStatus === 'ok' ? 'Connected' : healthStatus}
          </span>
        </div>
      </div>
      <div class="flex items-center justify-between px-5 py-4">
        <div>
          <div class="text-sm font-medium text-[var(--c-text-primary)]">LLM Provider{llmProvider ? ` (${llmProvider})` : ''}</div>
          <div class="text-xs text-[var(--c-text-muted)] mt-0.5">Required for Ask and narrative generation</div>
        </div>
        <div class="flex items-center gap-2">
          {#if llmAvailable === null}
            <span class="w-2 h-2 rounded-full bg-gray-500 animate-pulse"></span>
            <span class="text-sm text-[var(--c-text-muted)]">Checking...</span>
          {:else}
            <span class="w-2 h-2 rounded-full {llmAvailable ? 'bg-emerald-400' : 'bg-amber-400'}"></span>
            <span class="text-sm {llmAvailable ? 'text-emerald-400' : 'text-amber-400'}">
              {llmAvailable ? 'Available' : 'Not Found'}
            </span>
          {/if}
        </div>
      </div>
      <div class="flex items-center justify-between px-5 py-4">
        <div>
          <div class="text-sm font-medium text-[var(--c-text-primary)]">API Endpoint</div>
          <div class="text-xs text-[var(--c-text-muted)] mt-0.5">Base URL for backend API</div>
        </div>
        <code class="text-xs bg-[var(--surface-2)] px-2.5 py-1 rounded-lg text-[var(--c-text-secondary)] font-mono">{apiBase}</code>
      </div>
    </div>
  </section>

  <!-- LLM Setup -->
  {#if llmAvailable === false}
    <section class="mb-8">
      <h2 class="section-title">
        <Cpu size={15} />
        LLM Setup
      </h2>
      <div class="bg-amber-400/5 border border-amber-400/20 rounded-xl p-5">
        <p class="text-sm text-amber-400 font-medium mb-3">Claude Code CLI is not detected</p>
        <ol class="text-sm text-[var(--c-text-secondary)] space-y-2.5 list-decimal list-inside">
          <li>Install: <code class="text-xs font-mono bg-[var(--surface-2)] px-1.5 py-0.5 rounded">npm install -g @anthropic-ai/claude-code</code></li>
          <li>Verify: <code class="text-xs font-mono bg-[var(--surface-2)] px-1.5 py-0.5 rounded">claude --version</code></li>
          <li>Restart the Codeilus server</li>
        </ol>
        <p class="text-xs text-[var(--c-text-muted)] mt-4">Analysis, graphs, and metrics work without LLM. Only Q&A and narratives require it.</p>
      </div>
    </section>
  {/if}

  <!-- Appearance -->
  <section class="mb-8">
    <h2 class="section-title">
      <Palette size={15} />
      Preferences
    </h2>
    <div class="bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl divide-y divide-[var(--c-border)] overflow-hidden">
      <div class="flex items-center justify-between px-5 py-4">
        <div>
          <div class="text-sm font-medium text-[var(--c-text-primary)]">Theme</div>
          <div class="text-xs text-[var(--c-text-muted)] mt-0.5">Color scheme for the UI</div>
        </div>
        <select class="bg-[var(--surface-2)] border border-[var(--c-border)] rounded-lg px-3 py-1.5 text-sm text-[var(--c-text-primary)] outline-none focus:border-[var(--c-accent)] cursor-pointer">
          <option value="dark">Dark</option>
          <option value="light" disabled>Light (coming soon)</option>
        </select>
      </div>
      <div class="flex items-center justify-between px-5 py-4">
        <div>
          <div class="text-sm font-medium text-[var(--c-text-primary)]">Onboarding</div>
          <div class="text-xs text-[var(--c-text-muted)] mt-0.5">Show the welcome screen on next visit</div>
        </div>
        <button
          onclick={resetOnboarding}
          class="flex items-center gap-1.5 text-sm text-[var(--c-accent)] hover:text-[var(--c-accent-hover)] transition-colors"
        >
          <RotateCcw size={14} />
          {showResetConfirm ? 'Reset! Reload to see it.' : 'Reset onboarding'}
        </button>
      </div>
    </div>
  </section>

  <!-- About -->
  <section>
    <h2 class="section-title">
      <Info size={15} />
      About
    </h2>
    <div class="bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl p-5">
      <h3 class="text-sm font-semibold text-[var(--c-text-primary)] mb-1">Codeilus</h3>
      <p class="text-sm text-[var(--c-text-secondary)] leading-relaxed mb-3">
        A single binary that analyzes any codebase and transforms it into a guided, interactive learning experience.
      </p>
      <div class="flex items-center gap-2 flex-wrap">
        {#each ['Rust', 'Axum', 'SvelteKit 5', 'SQLite', 'Claude Code'] as tech}
          <span class="text-[11px] px-2 py-1 rounded-md bg-[var(--surface-2)] text-[var(--c-text-muted)] font-medium">{tech}</span>
        {/each}
      </div>
    </div>
  </section>
</div>

<style>
  @reference "tailwindcss";
  .section-title {
    @apply flex items-center gap-2 text-xs font-semibold uppercase tracking-wider text-[var(--c-text-muted)] mb-3;
  }
</style>
