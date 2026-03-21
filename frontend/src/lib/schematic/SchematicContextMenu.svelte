<script lang="ts">
  import type { SchematicNode } from '$lib/types';

  export type ContextAction =
    | 'copy-name' | 'focus-here' | 'hide'
    | 'view-source' | 'ask-ai' | 'add-note'
    | 'show-call-chain' | 'start-quiz'
    | 'start-learning' | 'show-members' | 'filter-community';

  let {
    node,
    x,
    y,
    onaction,
    onclose,
  }: {
    node: SchematicNode;
    x: number;
    y: number;
    onaction: (action: ContextAction) => void;
    onclose: () => void;
  } = $props();

  interface MenuItem { action: ContextAction; label: string; icon: string; }

  let items = $derived.by<MenuItem[]>(() => {
    const common: MenuItem[] = [
      { action: 'copy-name', label: 'Copy name', icon: '📋' },
      { action: 'focus-here', label: 'Focus here', icon: '🎯' },
      { action: 'hide', label: 'Hide from view', icon: '👁' },
    ];

    if (node.type === 'file') {
      return [
        { action: 'view-source', label: 'View source', icon: '📄' },
        { action: 'ask-ai', label: 'Ask AI about this', icon: '🤖' },
        { action: 'add-note', label: 'Add annotation', icon: '📝' },
        ...common,
      ];
    }

    if (node.type === 'symbol') {
      const symItems: MenuItem[] = [
        { action: 'view-source', label: 'View source', icon: '📄' },
        { action: 'show-call-chain', label: 'Show connections', icon: '🔗' },
        { action: 'ask-ai', label: 'Ask AI about this', icon: '🤖' },
        { action: 'add-note', label: 'Add annotation', icon: '📝' },
      ];
      if (node.chapter_id) {
        symItems.push({ action: 'start-quiz', label: 'Start quiz', icon: '❓' });
      }
      return [...symItems, ...common];
    }

    if (node.type === 'community') {
      return [
        { action: 'start-learning', label: 'Start learning', icon: '📚' },
        { action: 'show-members', label: 'Show all members', icon: '👥' },
        { action: 'filter-community', label: 'Filter to this', icon: '🔍' },
        ...common,
      ];
    }

    return common;
  });

  function handleAction(action: ContextAction) {
    onaction(action);
    onclose();
  }
</script>

<svelte:window onpointerdown={onclose} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  class="fixed z-[70] bg-[var(--surface-1)] border border-[var(--c-border)] rounded-lg shadow-2xl py-1 min-w-[180px]"
  style="left: {Math.min(x, globalThis.innerWidth - 200)}px; top: {Math.min(y, globalThis.innerHeight - 300)}px;"
  onpointerdown={(e) => e.stopPropagation()}
>
  {#each items as item}
    <button
      class="w-full text-left px-3 py-1.5 text-xs flex items-center gap-2 hover:bg-[var(--surface-2)] transition-colors text-[var(--c-text-secondary)] hover:text-[var(--c-text-primary)]"
      onclick={() => handleAction(item.action)}
    >
      <span class="text-[10px] w-4 text-center">{item.icon}</span>
      {item.label}
    </button>
  {/each}
</div>
