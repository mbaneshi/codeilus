<script lang="ts">
  import { searchSymbols, fetchLlmStatus, askQuestion } from '$lib/api';
  import type { SymbolRow } from '$lib/types';

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
    if (chatContainer) {
      chatContainer.scrollTop = chatContainer.scrollHeight;
    }
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

  async function handleAsk() {
    const q = query.trim();
    if (!q || streaming) return;

    // If there are suggestions showing, don't submit — user is searching symbols
    if (suggestions.length > 0) {
      suggestions = [];
      return;
    }

    messages = [...messages, { role: 'user', content: q }];
    query = '';
    suggestions = [];
    streaming = true;

    // Add empty assistant message that we'll stream into
    messages = [...messages, { role: 'assistant', content: '' }];
    scrollToBottom();

    const contextIds = context.map((s) => s.id);

    await askQuestion(
      q,
      contextIds,
      (delta) => {
        // Update the last message with streamed content
        const last = messages[messages.length - 1];
        if (last && last.role === 'assistant') {
          last.content += delta;
          messages = [...messages.slice(0, -1), last];
        }
        scrollToBottom();
      },
      () => {
        streaming = false;
        scrollToBottom();
      },
      (error) => {
        // Replace empty assistant message with error
        messages = [...messages.slice(0, -1), { role: 'error', content: error }];
        streaming = false;
        scrollToBottom();
      },
    );
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleAsk();
    }
  }

  function kindColor(kind: string): string {
    switch (kind.toLowerCase()) {
      case 'function': return 'bg-indigo-600';
      case 'class': return 'bg-pink-600';
      case 'method': return 'bg-teal-600';
      case 'struct': return 'bg-amber-600';
      default: return 'bg-gray-600';
    }
  }

  if (typeof window !== 'undefined') {
    fetchLlmStatus().then((status) => {
      llmAvailable = status.available;
    });
  }
</script>

<div class="p-8 max-w-3xl mx-auto">
  <div class="flex items-center gap-3 mb-4">
    <h1 class="text-2xl font-bold">Ask About the Code</h1>
    <span class="text-xs px-2 py-0.5 rounded-full {llmAvailable ? 'bg-green-900 text-green-400' : 'bg-yellow-900 text-yellow-400'}">
      {llmAvailable ? 'LLM Ready' : 'LLM Unavailable'}
    </span>
  </div>

  {#if !llmAvailable}
    <div class="mb-4 p-3 bg-yellow-900/30 border border-yellow-800 rounded-lg text-yellow-300 text-sm">
      Claude Code CLI not detected. Install with: <code class="bg-gray-800 px-1 rounded">npm install -g @anthropic-ai/claude-code</code>
    </div>
  {/if}

  <!-- Context area -->
  {#if context.length > 0}
    <div class="mb-4 p-3 bg-gray-900 border border-gray-800 rounded-lg">
      <div class="text-xs text-gray-500 mb-2">Context ({context.length} symbols)</div>
      <div class="flex flex-wrap gap-2">
        {#each context as sym}
          <span class="inline-flex items-center gap-1 text-sm px-2 py-1 bg-gray-800 rounded">
            <span class="text-xs px-1 py-0.5 rounded {kindColor(sym.kind)} text-white">{sym.kind}</span>
            <span class="font-mono text-gray-200">{sym.name}</span>
            <button class="text-gray-500 hover:text-gray-300 ml-1" onclick={() => removeFromContext(sym.id)}>&times;</button>
          </span>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Chat area -->
  <div bind:this={chatContainer} class="bg-gray-900 border border-gray-800 rounded-lg p-4 mb-4 h-96 overflow-auto">
    {#if messages.length === 0}
      <p class="text-gray-500 text-center mt-32">Ask anything about the codebase...</p>
    {:else}
      <div class="space-y-4">
        {#each messages as msg}
          {#if msg.role === 'user'}
            <div class="flex justify-end">
              <div class="bg-indigo-600/30 border border-indigo-500/30 rounded-lg px-4 py-2 max-w-[80%]">
                <p class="text-gray-100 text-sm whitespace-pre-wrap">{msg.content}</p>
              </div>
            </div>
          {:else if msg.role === 'assistant'}
            <div class="flex justify-start">
              <div class="bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 max-w-[80%]">
                {#if msg.content}
                  <p class="text-gray-200 text-sm whitespace-pre-wrap">{msg.content}</p>
                {:else}
                  <div class="flex items-center gap-2 text-gray-400 text-sm">
                    <span class="animate-pulse">Thinking...</span>
                  </div>
                {/if}
              </div>
            </div>
          {:else}
            <div class="flex justify-start">
              <div class="bg-red-900/30 border border-red-500/30 rounded-lg px-4 py-2 max-w-[80%]">
                <p class="text-red-300 text-sm">{msg.content}</p>
              </div>
            </div>
          {/if}
        {/each}
      </div>
    {/if}
  </div>

  <!-- Input area -->
  <div class="relative">
    <div class="flex gap-2">
      <input
        type="text"
        placeholder="Search symbols with @ or type a question..."
        class="flex-1 bg-gray-900 border border-gray-800 rounded px-4 py-2 text-gray-100 focus:border-indigo-500 outline-none"
        bind:value={query}
        oninput={onInput}
        onkeydown={handleKeydown}
        disabled={streaming}
      />
      <button
        class="bg-indigo-600 px-6 py-2 rounded hover:bg-indigo-500 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        onclick={handleAsk}
        disabled={streaming || !query.trim()}
      >
        {streaming ? 'Streaming...' : 'Ask'}
      </button>
    </div>

    <!-- Suggestions dropdown -->
    {#if suggestions.length > 0}
      <div class="absolute left-0 right-16 mt-1 bg-gray-900 border border-gray-700 rounded-lg shadow-lg z-10 max-h-48 overflow-auto">
        {#each suggestions as sym}
          <button
            class="w-full text-left px-3 py-2 hover:bg-gray-800 flex items-center gap-2 text-sm transition-colors"
            onclick={() => addToContext(sym)}
          >
            <span class="text-xs px-1.5 py-0.5 rounded {kindColor(sym.kind)} text-white">{sym.kind}</span>
            <span class="font-mono text-gray-200">{sym.name}</span>
            <span class="text-xs text-gray-500 ml-auto">L{sym.start_line}</span>
          </button>
        {/each}
      </div>
    {/if}
  </div>

  <p class="text-xs text-gray-600 mt-2">
    Type a question and press Enter. Search for symbols to add as context for more precise answers.
  </p>
</div>
