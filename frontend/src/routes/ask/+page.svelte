<script lang="ts">
  import { searchSymbols, fetchLlmStatus, askQuestion } from '$lib/api';
  import type { SymbolRow } from '$lib/types';
  import { MessageCircle, Send, Search, X, AlertCircle, Bot, User } from 'lucide-svelte';
  import Markdown from '$lib/Markdown.svelte';

  interface Message {
    role: 'user' | 'assistant' | 'error';
    content: string;
  }

  let query = $state('');
  let suggestions = $state<SymbolRow[]>([]);
  let context = $state<SymbolRow[]>([]);
  let messages = $state<Message[]>([]);
  let llmAvailable = $state(false);
  let streaming = $state(false);
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;
  let chatContainer: HTMLDivElement | undefined = $state();

  function scrollToBottom() {
    if (chatContainer) chatContainer.scrollTop = chatContainer.scrollHeight;
  }

  function onInput() {
    if (debounceTimer) clearTimeout(debounceTimer);
    if (query.trim().length < 2) {
      suggestions = [];
      return;
    }
    debounceTimer = setTimeout(async () => {
      suggestions = await searchSymbols(query.trim());
    }, 300);
  }

  function addToContext(sym: SymbolRow) {
    if (!context.find((s) => s.id === sym.id)) {
      context = [...context, sym];
    }
    suggestions = [];
    query = '';
  }

  function removeFromContext(id: number) {
    context = context.filter((s) => s.id !== id);
  }

  function kindColor(kind: string): string {
    switch (kind.toLowerCase()) {
      case 'function': return 'text-indigo-400 bg-indigo-500/10';
      case 'class': return 'text-pink-400 bg-pink-500/10';
      case 'method': return 'text-teal-400 bg-teal-500/10';
      case 'struct': return 'text-amber-400 bg-amber-500/10';
      default: return 'text-[var(--c-text-secondary)] bg-[var(--surface-3)]';
    }
  }

  async function handleAsk() {
    const q = query.trim();
    if (!q || streaming) return;
    if (suggestions.length > 0) { suggestions = []; return; }

    messages = [...messages, { role: 'user', content: q }];
    query = '';
    suggestions = [];
    streaming = true;
    messages = [...messages, { role: 'assistant', content: '' }];
    scrollToBottom();

    await askQuestion(
      q,
      context.map((s) => s.id),
      (delta) => {
        const last = messages[messages.length - 1];
        if (last && last.role === 'assistant') {
          last.content += delta;
          messages = [...messages.slice(0, -1), last];
        }
        scrollToBottom();
      },
      () => { streaming = false; scrollToBottom(); },
      (error) => {
        messages = [...messages.slice(0, -1), { role: 'error', content: error }];
        streaming = false;
        scrollToBottom();
      },
    );
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); handleAsk(); }
  }

  if (typeof window !== 'undefined') {
    fetchLlmStatus().then((s) => { llmAvailable = s.available; });
  }
</script>

<div class="flex flex-col h-full">
  <!-- Header -->
  <div class="p-5 border-b border-[var(--c-border)]">
    <div class="max-w-3xl mx-auto flex items-center gap-3">
      <div class="w-10 h-10 rounded-xl bg-amber-500/10 flex items-center justify-center">
        <MessageCircle size={20} class="text-amber-400" />
      </div>
      <div class="flex-1">
        <h1 class="text-lg font-semibold tracking-tight">Ask About the Code</h1>
        <p class="text-xs text-[var(--c-text-muted)]">Ask questions about the codebase — powered by Claude</p>
      </div>
      <div class="flex items-center gap-2">
        <span class="w-2 h-2 rounded-full {llmAvailable ? 'bg-emerald-400' : 'bg-amber-400'}"></span>
        <span class="text-xs text-[var(--c-text-muted)]">{llmAvailable ? 'LLM Ready' : 'LLM Unavailable'}</span>
      </div>
    </div>
  </div>

  {#if !llmAvailable}
    <div class="max-w-3xl mx-auto w-full px-5 pt-4">
      <div class="flex items-start gap-3 p-4 bg-amber-400/5 border border-amber-400/20 rounded-xl">
        <AlertCircle size={16} class="text-amber-400 shrink-0 mt-0.5" />
        <div>
          <p class="text-sm text-amber-400 font-medium">Claude Code CLI not detected</p>
          <p class="text-xs text-[var(--c-text-muted)] mt-1">Install with <code class="font-mono bg-[var(--surface-2)] px-1.5 py-0.5 rounded">npm install -g @anthropic-ai/claude-code</code></p>
        </div>
      </div>
    </div>
  {/if}

  <!-- Context chips -->
  {#if context.length > 0}
    <div class="max-w-3xl mx-auto w-full px-5 pt-4">
      <div class="flex items-center gap-2 flex-wrap">
        <span class="text-xs text-[var(--c-text-muted)] font-medium">Context:</span>
        {#each context as sym}
          <span class="inline-flex items-center gap-1.5 text-xs px-2.5 py-1 bg-[var(--surface-1)] border border-[var(--c-border)] rounded-lg">
            <span class="font-medium {kindColor(sym.kind).split(' ')[0]}">{sym.kind}</span>
            <span class="font-mono text-[var(--c-text-primary)]">{sym.name}</span>
            <button class="text-[var(--c-text-muted)] hover:text-red-400 ml-0.5 transition-colors" onclick={() => removeFromContext(sym.id)}>
              <X size={12} />
            </button>
          </span>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Chat area -->
  <div bind:this={chatContainer} class="flex-1 overflow-auto px-5 py-6">
    <div class="max-w-3xl mx-auto">
      {#if messages.length === 0}
        <div class="text-center pt-24">
          <div class="w-16 h-16 rounded-2xl bg-[var(--surface-1)] border border-[var(--c-border)] flex items-center justify-center mx-auto mb-5">
            <MessageCircle size={28} class="text-[var(--c-text-muted)]" />
          </div>
          <p class="text-[var(--c-text-secondary)] font-medium mb-2">Ask anything about the codebase</p>
          <p class="text-sm text-[var(--c-text-muted)]">Search for symbols to add context for more precise answers</p>
        </div>
      {:else}
        <div class="space-y-5">
          {#each messages as msg}
            {#if msg.role === 'user'}
              <div class="flex gap-3 justify-end">
                <div class="max-w-[75%] bg-[var(--c-accent)]/10 border border-indigo-500/20 rounded-2xl rounded-tr-md px-4 py-3">
                  <p class="text-sm text-[var(--c-text-primary)] whitespace-pre-wrap leading-relaxed">{msg.content}</p>
                </div>
                <div class="w-8 h-8 rounded-full bg-[var(--c-accent)]/15 flex items-center justify-center shrink-0">
                  <User size={14} class="text-[var(--c-accent)]" />
                </div>
              </div>
            {:else if msg.role === 'assistant'}
              <div class="flex gap-3">
                <div class="w-8 h-8 rounded-full bg-[var(--surface-2)] border border-[var(--c-border)] flex items-center justify-center shrink-0">
                  <Bot size={14} class="text-[var(--c-text-secondary)]" />
                </div>
                <div class="max-w-[75%] bg-[var(--surface-1)] border border-[var(--c-border)] rounded-2xl rounded-tl-md px-4 py-3">
                  {#if msg.content}
                    <Markdown content={msg.content} />
                  {:else}
                    <div class="flex items-center gap-2 text-[var(--c-text-muted)] text-sm">
                      <span class="inline-block w-1.5 h-1.5 rounded-full bg-[var(--c-accent)] animate-pulse"></span>
                      <span>Thinking...</span>
                    </div>
                  {/if}
                </div>
              </div>
            {:else}
              <div class="flex gap-3">
                <div class="w-8 h-8 rounded-full bg-red-400/15 flex items-center justify-center shrink-0">
                  <AlertCircle size={14} class="text-red-400" />
                </div>
                <div class="max-w-[75%] bg-red-400/5 border border-red-400/20 rounded-2xl rounded-tl-md px-4 py-3">
                  <p class="text-sm text-red-400">{msg.content}</p>
                </div>
              </div>
            {/if}
          {/each}
        </div>
      {/if}
    </div>
  </div>

  <!-- Input area -->
  <div class="border-t border-[var(--c-border)] p-5">
    <div class="max-w-3xl mx-auto relative">
      <div class="flex gap-2.5 items-end">
        <div class="flex-1 relative">
          <input
            type="text"
            placeholder="Type a question or search symbols..."
            class="w-full bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl px-4 py-3 text-sm text-[var(--c-text-primary)] placeholder:text-[var(--c-text-muted)] focus:border-[var(--c-accent)] focus:ring-1 focus:ring-[var(--c-accent)]/30 outline-none transition-all"
            bind:value={query}
            oninput={onInput}
            onkeydown={handleKeydown}
            disabled={streaming}
          />

          <!-- Suggestions dropdown -->
          {#if suggestions.length > 0}
            <div class="absolute left-0 right-0 bottom-full mb-2 bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl shadow-xl shadow-black/30 z-10 overflow-hidden">
              <div class="text-[10px] uppercase tracking-wider text-[var(--c-text-muted)] font-medium px-3 py-2 border-b border-[var(--c-border)]">
                Add to context
              </div>
              {#each suggestions as sym}
                <button
                  class="w-full text-left px-3 py-2.5 hover:bg-[var(--surface-2)] flex items-center gap-2.5 text-sm transition-colors"
                  onclick={() => addToContext(sym)}
                >
                  <span class="text-[10px] font-medium uppercase tracking-wider px-1.5 py-0.5 rounded {kindColor(sym.kind)}">{sym.kind}</span>
                  <span class="font-mono text-[var(--c-text-primary)]">{sym.name}</span>
                  <span class="text-xs text-[var(--c-text-muted)] ml-auto">L{sym.start_line}</span>
                </button>
              {/each}
            </div>
          {/if}
        </div>
        <button
          class="bg-[var(--c-accent)] hover:bg-[var(--c-accent-hover)] disabled:opacity-40 disabled:cursor-not-allowed text-white p-3 rounded-xl transition-colors shrink-0"
          onclick={handleAsk}
          disabled={streaming || !query.trim()}
        >
          <Send size={18} />
        </button>
      </div>
      <p class="text-[11px] text-[var(--c-text-muted)] mt-2">
        Press Enter to send. Search for symbols to add context for focused answers.
      </p>
    </div>
  </div>
</div>
