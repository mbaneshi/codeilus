<script lang="ts">
  import type { SchematicNode, SchematicDetail, Annotation } from '$lib/types';
  import { fetchFileSource } from '$lib/api';
  import type { SourceResponse } from '$lib/types';
  import { LoadingSpinner } from '$lib/components';
  import Markdown from '$lib/Markdown.svelte';

  let {
    node,
    detail = null,
    loading = false,
    annotations = [],
    annotationsLoading = false,
    onclose,
    onnavigate,
    onaskai,
    onannotationcreate,
    onannotationdelete,
    onannotationflag,
  }: {
    node: SchematicNode;
    detail: SchematicDetail | null;
    loading: boolean;
    annotations: Annotation[];
    annotationsLoading: boolean;
    onclose: () => void;
    onnavigate: (nodeId: string) => void;
    onaskai?: (question: string) => void;
    onannotationcreate?: (content: string) => void;
    onannotationdelete?: (id: number) => void;
    onannotationflag?: (id: number) => void;
  } = $props();

  let activeTab = $state<'overview' | 'source' | 'relations' | 'learn' | 'notes'>('overview');
  let sourceData = $state<SourceResponse | null>(null);
  let sourceLoading = $state(false);
  let noteText = $state('');
  let lastNodeId = '';

  function loadSource() {
    if (!node.file_id || sourceLoading) return;
    sourceLoading = true;
    fetchFileSource(node.file_id, 1, 100).then(data => {
      sourceData = data;
      sourceLoading = false;
    });
  }

  function switchTab(tab: typeof activeTab) {
    activeTab = tab;
    if (tab === 'source' && !sourceData && node.file_id) loadSource();
  }

  // Reset on node change (checked manually since $effect may be stripped)
  function checkReset() {
    if (node.id !== lastNodeId) {
      lastNodeId = node.id;
      sourceData = null;
      activeTab = 'overview';
      noteText = '';
    }
  }
  checkReset();

  function submitNote() {
    if (noteText.trim() && onannotationcreate) {
      onannotationcreate(noteText.trim());
      noteText = '';
    }
  }

  const tabs = [
    { id: 'overview' as const, label: 'Overview' },
    { id: 'source' as const, label: 'Source' },
    { id: 'relations' as const, label: 'Relations' },
    { id: 'learn' as const, label: 'Learn' },
    { id: 'notes' as const, label: 'Notes' },
  ];
</script>

<div class="w-[360px] shrink-0 border-l border-[var(--c-border)] bg-[var(--surface-1)] overflow-auto flex flex-col h-full">
  <!-- Header -->
  <div class="px-4 py-3 border-b border-[var(--c-border)] sticky top-0 bg-[var(--surface-1)] z-10">
    <div class="flex items-center justify-between">
      <div class="truncate">
        <span class="text-[10px] text-[var(--c-text-muted)] uppercase">{node.type}</span>
        <h2 class="text-sm font-semibold text-[var(--c-text-primary)] truncate">{node.label}</h2>
      </div>
      <button onclick={onclose} class="text-[var(--c-text-muted)] hover:text-[var(--c-text-primary)] text-sm shrink-0 ml-2">✕</button>
    </div>

    <!-- Tab bar -->
    <div class="flex gap-0.5 mt-2 bg-[var(--surface-2)] rounded-lg p-0.5">
      {#each tabs as tab}
        <button
          class="flex-1 px-2 py-1 text-[10px] font-medium rounded-md transition-colors {activeTab === tab.id ? 'bg-[var(--c-accent)] text-white' : 'text-[var(--c-text-muted)] hover:text-[var(--c-text-primary)]'}"
          onclick={() => switchTab(tab.id)}
        >{tab.label}</button>
      {/each}
    </div>
  </div>

  <!-- Content -->
  <div class="flex-1 overflow-auto p-4">
    {#if loading}
      <LoadingSpinner />
    {:else if detail}

      <!-- Overview Tab -->
      {#if activeTab === 'overview'}
        <div class="space-y-3">
          <div class="flex flex-wrap gap-1.5 text-xs">
            {#if node.language}
              <span class="px-2 py-0.5 rounded bg-[var(--c-accent)]/10 text-[var(--c-accent)]">{node.language}</span>
            {/if}
            {#if node.kind}
              <span class="px-2 py-0.5 rounded bg-[var(--surface-2)] text-[var(--c-text-muted)]">{node.kind}</span>
            {/if}
            {#if node.sloc}
              <span class="px-2 py-0.5 rounded bg-[var(--surface-2)] text-[var(--c-text-muted)]">{node.sloc} loc</span>
            {/if}
          </div>

          {#if node.community_label}
            <div class="flex items-center gap-2 px-2 py-1.5 rounded bg-[var(--surface-2)]">
              <span class="w-2.5 h-2.5 rounded-full" style="background: {node.community_color}"></span>
              <span class="text-xs text-[var(--c-text-secondary)]">{node.community_label}</span>
            </div>
          {/if}

          {#if detail.narrative}
            <div>
              <h3 class="text-xs font-medium text-[var(--c-text-muted)] uppercase mb-1">Explanation</h3>
              <div class="text-sm text-[var(--c-text-secondary)] leading-relaxed">
                <Markdown content={detail.narrative} />
              </div>
            </div>
          {/if}

          {#if node.signature}
            <div>
              <h3 class="text-xs font-medium text-[var(--c-text-muted)] uppercase mb-1">Signature</h3>
              <pre class="text-[11px] font-mono bg-[var(--surface-2)] rounded p-2 overflow-x-auto text-[var(--c-text-primary)]">{node.signature}</pre>
            </div>
          {/if}
        </div>

      <!-- Source Tab -->
      {:else if activeTab === 'source'}
        {#if sourceLoading}
          <LoadingSpinner />
        {:else if sourceData}
          <pre class="text-[11px] font-mono bg-[var(--surface-2)] rounded-lg p-3 overflow-auto max-h-[60vh] text-[var(--c-text-secondary)]">{sourceData.lines.map(l => `${String(l.number).padStart(4)} ${l.content}`).join('\n')}</pre>
        {:else if !node.file_id}
          <p class="text-sm text-[var(--c-text-muted)]">No source available for this node.</p>
        {/if}

      <!-- Relations Tab -->
      {:else if activeTab === 'relations'}
        <div class="space-y-4">
          {#if detail.callers.length > 0}
            <div>
              <h3 class="text-xs font-medium text-[var(--c-text-muted)] uppercase mb-1.5">Called by ({detail.callers.length})</h3>
              <div class="space-y-1">
                {#each detail.callers as c}
                  <button onclick={() => onnavigate(c.id)} class="w-full text-left text-xs px-2 py-1.5 rounded bg-[var(--surface-2)] hover:bg-[var(--surface-3)] transition-colors flex justify-between items-center">
                    <span class="font-mono text-[var(--c-text-primary)]">{c.name}</span>
                    <span class="text-[var(--c-text-muted)] text-[10px]">{c.file_path.split('/').pop()}</span>
                  </button>
                {/each}
              </div>
            </div>
          {/if}

          {#if detail.callees.length > 0}
            <div>
              <h3 class="text-xs font-medium text-[var(--c-text-muted)] uppercase mb-1.5">Calls ({detail.callees.length})</h3>
              <div class="space-y-1">
                {#each detail.callees as c}
                  <button onclick={() => onnavigate(c.id)} class="w-full text-left text-xs px-2 py-1.5 rounded bg-[var(--surface-2)] hover:bg-[var(--surface-3)] transition-colors flex justify-between items-center">
                    <span class="font-mono text-[var(--c-text-primary)]">{c.name}</span>
                    <span class="text-[var(--c-text-muted)] text-[10px]">{c.file_path.split('/').pop()}</span>
                  </button>
                {/each}
              </div>
            </div>
          {/if}

          {#if detail.callers.length === 0 && detail.callees.length === 0}
            <p class="text-sm text-[var(--c-text-muted)]">No connections found.</p>
          {/if}
        </div>

      <!-- Learn Tab -->
      {:else if activeTab === 'learn'}
        {#if detail.chapter}
          <div class="space-y-3">
            <a href="/learn/{detail.chapter.id}" class="block px-3 py-2.5 rounded-lg bg-[var(--c-accent)]/10 border border-[var(--c-accent)]/20 hover:bg-[var(--c-accent)]/15 transition-colors">
              <div class="text-xs font-semibold text-[var(--c-accent)]">{detail.chapter.title}</div>
              <div class="flex items-center gap-2 mt-1.5">
                <span class="text-[10px] text-[var(--c-text-muted)] capitalize">{detail.chapter.difficulty}</span>
                <div class="flex-1 h-1.5 bg-[var(--surface-3)] rounded-full overflow-hidden">
                  <div class="h-full bg-[var(--c-success)] rounded-full transition-all" style="width: {(detail.chapter.progress.completed / Math.max(detail.chapter.progress.total, 1)) * 100}%"></div>
                </div>
                <span class="text-[10px] text-[var(--c-text-muted)] tabular-nums">{detail.chapter.progress.completed}/{detail.chapter.progress.total}</span>
              </div>
            </a>
            <a href="/learn/{detail.chapter.id}" class="block text-center text-xs text-[var(--c-accent)] hover:underline">Start Learning &rarr;</a>
          </div>
        {:else}
          <p class="text-sm text-[var(--c-text-muted)]">No learning chapter linked to this node.</p>
        {/if}

      <!-- Notes Tab -->
      {:else if activeTab === 'notes'}
        <div class="space-y-3">
          {#if annotationsLoading}
            <LoadingSpinner />
          {:else}
            {#each annotations as ann}
              <div class="px-3 py-2 rounded-lg bg-[var(--surface-2)] border border-[var(--c-border)]">
                <p class="text-xs text-[var(--c-text-secondary)]">{ann.content}</p>
                <div class="flex items-center gap-2 mt-1.5">
                  <span class="text-[10px] text-[var(--c-text-muted)]">{new Date(ann.created_at).toLocaleDateString()}</span>
                  <button onclick={() => onannotationflag?.(ann.id)} class="text-[10px] {ann.flagged ? 'text-[var(--c-warning)]' : 'text-[var(--c-text-muted)]'} hover:text-[var(--c-warning)]">{ann.flagged ? '★' : '☆'}</button>
                  <button onclick={() => onannotationdelete?.(ann.id)} class="text-[10px] text-[var(--c-text-muted)] hover:text-[var(--c-danger)] ml-auto">Delete</button>
                </div>
              </div>
            {/each}

            {#if annotations.length === 0}
              <p class="text-xs text-[var(--c-text-muted)]">No notes yet.</p>
            {/if}

            <div class="flex gap-2 mt-2">
              <input
                type="text"
                placeholder="Add a note..."
                class="flex-1 bg-[var(--surface-2)] border border-[var(--c-border)] rounded-lg px-3 py-1.5 text-xs text-[var(--c-text-primary)] placeholder:text-[var(--c-text-muted)] focus:border-[var(--c-accent)] outline-none"
                bind:value={noteText}
                onkeydown={(e) => e.key === 'Enter' && submitNote()}
              />
              <button onclick={submitNote} class="px-3 py-1.5 bg-[var(--c-accent)] text-white rounded-lg text-xs font-medium hover:bg-[var(--c-accent-hover)] transition-colors">Add</button>
            </div>
          {/if}
        </div>
      {/if}
    {/if}
  </div>
</div>
