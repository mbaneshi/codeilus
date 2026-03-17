<script lang="ts">
  import '../app.css';
  import { searchSymbols } from '$lib/api';
  import type { SymbolRow } from '$lib/types';
  import { Breadcrumbs, CommandPalette, SystemStatus } from '$lib/components';
  import { BookOpen, Compass, MessageCircle, Settings, Search, Layers, Home, Sun, Moon } from 'lucide-svelte';
  import { themeStore } from '$lib/stores/theme.svelte';
  import { page } from '$app/stores';

  let { children } = $props();
  let searchQuery = $state('');
  let searchResults = $state<SymbolRow[]>([]);
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  let currentPath = $derived($page.url.pathname);
  let isFullScreen = $derived(currentPath.startsWith('/explore/graph') || currentPath.startsWith('/explore/schematic'));

  function onSearchInput() {
    if (debounceTimer) clearTimeout(debounceTimer);
    if (searchQuery.trim().length < 2) {
      searchResults = [];
      return;
    }
    debounceTimer = setTimeout(async () => {
      const results = await searchSymbols(searchQuery.trim());
      searchResults = results.slice(0, 5);
    }, 300);
  }

  function clearSearch() {
    searchQuery = '';
    searchResults = [];
  }

  function isActive(path: string): boolean {
    if (path === '/') return currentPath === '/';
    return currentPath.startsWith(path);
  }
</script>

<div class="flex h-screen">
  <!-- Sidebar -->
  <nav class="w-64 shrink-0 bg-[var(--surface-1)] border-r border-[var(--c-border)] flex flex-col overflow-hidden">
    <!-- Logo -->
    <a href="/" class="block px-5 py-5 border-b border-[var(--c-border)] group">
      <div class="flex items-center gap-2.5">
        <div class="w-8 h-8 rounded-lg bg-[var(--c-accent)]/15 flex items-center justify-center">
          <Layers size={18} class="text-[var(--c-accent)]" />
        </div>
        <div>
          <h1 class="text-base font-semibold tracking-tight text-[var(--c-text-primary)]">Codeilus</h1>
          <p class="text-[11px] text-[var(--c-text-muted)] leading-none mt-0.5">Learn any codebase</p>
        </div>
      </div>
    </a>

    <!-- Navigation -->
    <div class="flex-1 px-3 py-4 space-y-1 overflow-auto">
      <a href="/" class="nav-item" class:active={isActive('/')}>
        <Home size={18} />
        <span>Home</span>
      </a>
      <a href="/learn" class="nav-item" class:active={isActive('/learn')}>
        <BookOpen size={18} />
        <span>Learn</span>
      </a>
      <a href="/explore" class="nav-item" class:active={isActive('/explore')}>
        <Compass size={18} />
        <span>Explore</span>
      </a>
      <a href="/ask" class="nav-item" class:active={isActive('/ask')}>
        <MessageCircle size={18} />
        <span>Ask</span>
      </a>

      <!-- Search -->
      <div class="pt-4 mt-4 border-t border-[var(--c-border)]">
        <div class="relative">
          <Search size={14} class="absolute left-2.5 top-1/2 -translate-y-1/2 text-[var(--c-text-muted)]" />
          <input
            type="text"
            placeholder="Search symbols..."
            class="w-full bg-[var(--surface-2)] border border-[var(--c-border)] rounded-lg pl-8 pr-3 py-2 text-sm text-[var(--c-text-primary)] placeholder:text-[var(--c-text-muted)] focus:border-[var(--c-accent)] focus:ring-1 focus:ring-[var(--c-accent)]/30 outline-none transition-all"
            bind:value={searchQuery}
            oninput={onSearchInput}
          />
          {#if searchResults.length > 0}
            <div class="absolute left-0 right-0 mt-1.5 bg-[var(--surface-2)] border border-[var(--c-border)] rounded-lg shadow-xl shadow-black/30 z-20 overflow-hidden">
              {#each searchResults as sym}
                <a
                  href="/explore/tree"
                  class="flex items-center gap-2 px-3 py-2.5 hover:bg-[var(--surface-3)] text-sm transition-colors"
                  onclick={clearSearch}
                >
                  <span class="text-[10px] font-medium uppercase tracking-wider px-1.5 py-0.5 rounded bg-[var(--c-accent)]/15 text-[var(--c-accent)]">{sym.kind}</span>
                  <span class="font-mono text-[var(--c-text-primary)]">{sym.name}</span>
                </a>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    </div>

    <!-- Footer -->
    <div class="px-3 pb-3">
      <button onclick={() => themeStore.toggle()} class="nav-item w-full" title="Toggle theme">
        {#if themeStore.current === 'dark'}
          <Sun size={18} />
          <span>Light mode</span>
        {:else}
          <Moon size={18} />
          <span>Dark mode</span>
        {/if}
      </button>
      <a href="/settings" class="nav-item" class:active={isActive('/settings')}>
        <Settings size={18} />
        <span>Settings</span>
      </a>
      <SystemStatus />
      <div class="px-3 py-2 text-[11px] text-[var(--c-text-muted)]">
        Codeilus v0.1.0
      </div>
    </div>
  </nav>

  <!-- Main content -->
  <main class="flex-1 bg-[var(--surface-0)] {isFullScreen ? 'overflow-hidden' : 'overflow-auto'}">
    {#if isFullScreen}
      <div class="h-full flex flex-col">
        {#if currentPath.startsWith('/explore/graph')}
          <div class="px-6 pt-4 pb-2 shrink-0">
            <Breadcrumbs />
          </div>
        {/if}
        <div class="flex-1 min-h-0">
          {@render children()}
        </div>
      </div>
    {:else}
      <div class="p-6">
        <Breadcrumbs />
        {@render children()}
      </div>
    {/if}
  </main>
</div>

<CommandPalette />

<style>
  @reference "tailwindcss";
  .nav-item {
    @apply flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium text-[var(--c-text-secondary)] hover:text-[var(--c-text-primary)] hover:bg-[var(--surface-2)] transition-all cursor-pointer;
  }
  .nav-item.active {
    @apply bg-[var(--c-accent)]/10 text-[var(--c-accent)];
  }
</style>
