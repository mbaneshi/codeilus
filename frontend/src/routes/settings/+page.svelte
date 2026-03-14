<script lang="ts">
  import { fetchHealth, fetchLlmStatus } from '$lib/api';

  let healthStatus = $state('checking...');
  let llmAvailable = $state<boolean | null>(null);
  let apiBase = $state('/api/v1');
  let theme = $state('dark');

  if (typeof window !== 'undefined') {
    Promise.all([fetchHealth(), fetchLlmStatus()]).then(([health, llm]) => {
      healthStatus = health.status;
      llmAvailable = llm.available;
    });

    // Load saved settings
    const saved = localStorage.getItem('codeilus-settings');
    if (saved) {
      try {
        const parsed = JSON.parse(saved);
        if (parsed.theme) theme = parsed.theme;
      } catch { /* ignore */ }
    }
  }

  function saveSettings() {
    localStorage.setItem('codeilus-settings', JSON.stringify({ theme }));
  }
</script>

<div class="p-8 max-w-2xl mx-auto">
  <h1 class="text-2xl font-bold mb-6">Settings</h1>

  <!-- System Status -->
  <section class="mb-8">
    <h2 class="text-lg font-semibold mb-3 text-gray-300">System Status</h2>
    <div class="bg-gray-900 border border-gray-800 rounded-lg divide-y divide-gray-800">
      <div class="flex items-center justify-between px-4 py-3">
        <div>
          <div class="text-sm font-medium text-gray-200">API Server</div>
          <div class="text-xs text-gray-500">Backend connection status</div>
        </div>
        <span class="text-xs px-2 py-0.5 rounded-full {healthStatus === 'ok' ? 'bg-green-900 text-green-400' : 'bg-red-900 text-red-400'}">
          {healthStatus === 'ok' ? 'Connected' : healthStatus}
        </span>
      </div>
      <div class="flex items-center justify-between px-4 py-3">
        <div>
          <div class="text-sm font-medium text-gray-200">LLM (Claude Code)</div>
          <div class="text-xs text-gray-500">Required for Ask and narrative generation</div>
        </div>
        {#if llmAvailable === null}
          <span class="text-xs px-2 py-0.5 rounded-full bg-gray-700 text-gray-400">Checking...</span>
        {:else}
          <span class="text-xs px-2 py-0.5 rounded-full {llmAvailable ? 'bg-green-900 text-green-400' : 'bg-yellow-900 text-yellow-400'}">
            {llmAvailable ? 'Available' : 'Not Found'}
          </span>
        {/if}
      </div>
      <div class="flex items-center justify-between px-4 py-3">
        <div>
          <div class="text-sm font-medium text-gray-200">API Endpoint</div>
          <div class="text-xs text-gray-500">Base URL for backend API</div>
        </div>
        <code class="text-xs bg-gray-800 px-2 py-1 rounded text-gray-300">{apiBase}</code>
      </div>
    </div>
  </section>

  <!-- LLM Setup -->
  {#if llmAvailable === false}
    <section class="mb-8">
      <h2 class="text-lg font-semibold mb-3 text-gray-300">LLM Setup</h2>
      <div class="bg-yellow-900/20 border border-yellow-800 rounded-lg p-4">
        <p class="text-sm text-yellow-300 mb-3">Claude Code CLI is not detected. To enable AI features:</p>
        <ol class="text-sm text-gray-300 space-y-2 list-decimal list-inside">
          <li>Install Claude Code: <code class="bg-gray-800 px-1 rounded">npm install -g @anthropic-ai/claude-code</code></li>
          <li>Verify it works: <code class="bg-gray-800 px-1 rounded">claude --version</code></li>
          <li>Restart the Codeilus server</li>
        </ol>
        <p class="text-xs text-gray-500 mt-3">All analysis, graphs, and metrics work without LLM. Only Q&A and narrative generation require it.</p>
      </div>
    </section>
  {/if}

  <!-- Appearance -->
  <section class="mb-8">
    <h2 class="text-lg font-semibold mb-3 text-gray-300">Appearance</h2>
    <div class="bg-gray-900 border border-gray-800 rounded-lg divide-y divide-gray-800">
      <div class="flex items-center justify-between px-4 py-3">
        <div>
          <div class="text-sm font-medium text-gray-200">Theme</div>
          <div class="text-xs text-gray-500">Color scheme for the UI</div>
        </div>
        <select
          class="bg-gray-800 border border-gray-700 rounded px-3 py-1 text-sm text-gray-200 outline-none focus:border-indigo-500"
          bind:value={theme}
          onchange={saveSettings}
        >
          <option value="dark">Dark</option>
          <option value="light" disabled>Light (coming soon)</option>
        </select>
      </div>
    </div>
  </section>

  <!-- About -->
  <section class="mb-8">
    <h2 class="text-lg font-semibold mb-3 text-gray-300">About</h2>
    <div class="bg-gray-900 border border-gray-800 rounded-lg px-4 py-3">
      <div class="text-sm text-gray-300 mb-1">Codeilus</div>
      <div class="text-xs text-gray-500">
        A single binary that analyzes any codebase and transforms it into a gamified, interactive learning experience.
      </div>
      <div class="text-xs text-gray-600 mt-2">
        Rust + Axum + SvelteKit 5 + SQLite WAL + Claude Code
      </div>
    </div>
  </section>
</div>
