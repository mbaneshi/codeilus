<script lang="ts">
  import { fetchHealth, fetchFiles, fetchCommunities, fetchNarrative, fetchChapters } from '$lib/api';
  import type { FileRow, Community, NarrativeResponse, Chapter } from '$lib/types';
  import { BookOpen, Compass, MessageCircle, ArrowRight, Zap, GitBranch, BarChart3, FileText, Map, Users } from 'lucide-svelte';
  import Markdown from '$lib/Markdown.svelte';
  import { OnboardingBanner } from '$lib/components';

  let health = $state<string>('checking...');
  let files = $state<FileRow[]>([]);
  let communities = $state<Community[]>([]);
  let overview = $state<NarrativeResponse | null>(null);
  let architecture = $state<NarrativeResponse | null>(null);
  let readingOrder = $state<NarrativeResponse | null>(null);
  let chapters = $state<Chapter[]>([]);
  let showOnboarding = $state(false);
  let loaded = $state(false);
  let totalFiles = $derived(files.length);
  let totalSloc = $derived(files.reduce((sum, f) => sum + f.sloc, 0));
  let languageCount = $derived(new Set(files.map((f) => f.language).filter(Boolean)).size);
  let topLanguages = $derived.by(() => {
    // @ts-ignore
    const counts = new Map();
    for (const f of files) {
      const lang = f.language ?? 'unknown';
      counts.set(lang, (counts.get(lang) ?? 0) + f.sloc);
    }
    return [...counts.entries()]
      .sort((a, b) => b[1] - a[1])
      .slice(0, 4)
      .map(([lang]) => lang);
  });

  if (typeof window !== 'undefined') {
    if (!localStorage.getItem('codeilus-onboarded')) {
      showOnboarding = true;
    }

    // Load each independently so one failure doesn't block all data
    fetchHealth().then((h) => { health = h.status; }).catch(() => { health = 'disconnected'; });
    fetchFiles().then((f) => { files = f; }).catch(() => {});
    fetchCommunities().then((c) => { communities = c; }).catch(() => {});
    fetchNarrative('overview').then((o) => { overview = o; }).catch(() => {});
    fetchNarrative('architecture').then((a) => { architecture = a; }).catch(() => {});
    fetchNarrative('reading_order').then((r) => { readingOrder = r; }).catch(() => {});
    fetchChapters().then((ch) => { chapters = ch; loaded = true; }).catch(() => { loaded = true; });
  }

  function dismissOnboarding() {
    showOnboarding = false;
    localStorage.setItem('codeilus-onboarded', 'true');
  }
</script>

<!-- Onboarding Modal -->
{#if showOnboarding}
  <div class="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center p-4">
    <div class="bg-[var(--surface-1)] border border-[var(--c-border)] rounded-2xl max-w-lg w-full p-8 shadow-2xl shadow-black/50">
      <div class="text-center mb-6">
        <div class="w-14 h-14 rounded-2xl bg-[var(--c-accent)]/15 flex items-center justify-center mx-auto mb-4">
          <Zap size={28} class="text-[var(--c-accent)]" />
        </div>
        <h2 class="text-2xl font-bold tracking-tight mb-2">Welcome to Codeilus</h2>
        <p class="text-[var(--c-text-secondary)] text-sm leading-relaxed">
          Codeilus analyzes any codebase and transforms it into a guided learning experience.
          No more staring at unfamiliar code — learn systematically.
        </p>
      </div>

      <div class="space-y-3 mb-8">
        <div class="flex items-start gap-3 p-3 bg-[var(--surface-2)] rounded-xl">
          <div class="w-8 h-8 rounded-lg bg-indigo-500/15 flex items-center justify-center shrink-0 mt-0.5">
            <BookOpen size={16} class="text-indigo-400" />
          </div>
          <div>
            <h3 class="text-sm font-semibold text-[var(--c-text-primary)]">Learn</h3>
            <p class="text-xs text-[var(--c-text-secondary)] mt-0.5">Follow guided chapters through each module of the codebase</p>
          </div>
        </div>
        <div class="flex items-start gap-3 p-3 bg-[var(--surface-2)] rounded-xl">
          <div class="w-8 h-8 rounded-lg bg-teal-500/15 flex items-center justify-center shrink-0 mt-0.5">
            <Compass size={16} class="text-teal-400" />
          </div>
          <div>
            <h3 class="text-sm font-semibold text-[var(--c-text-primary)]">Explore</h3>
            <p class="text-xs text-[var(--c-text-secondary)] mt-0.5">Browse files, visualize the dependency graph, and review metrics</p>
          </div>
        </div>
        <div class="flex items-start gap-3 p-3 bg-[var(--surface-2)] rounded-xl">
          <div class="w-8 h-8 rounded-lg bg-amber-500/15 flex items-center justify-center shrink-0 mt-0.5">
            <MessageCircle size={16} class="text-amber-400" />
          </div>
          <div>
            <h3 class="text-sm font-semibold text-[var(--c-text-primary)]">Ask</h3>
            <p class="text-xs text-[var(--c-text-secondary)] mt-0.5">Ask questions about the code and get AI-powered answers</p>
          </div>
        </div>
      </div>

      <div class="flex items-center gap-3">
        <button
          onclick={dismissOnboarding}
          class="flex-1 bg-[var(--c-accent)] hover:bg-[var(--c-accent-hover)] text-white font-medium py-2.5 rounded-xl transition-colors text-sm"
        >
          Get Started
        </button>
        <button
          onclick={dismissOnboarding}
          class="px-4 py-2.5 text-[var(--c-text-muted)] hover:text-[var(--c-text-secondary)] text-sm transition-colors"
        >
          Skip
        </button>
      </div>

      <p class="text-center text-[11px] text-[var(--c-text-muted)] mt-4">
        You can reopen this from Settings anytime
      </p>
    </div>
  </div>
{/if}

<!-- Main Content -->
<div class="p-8 max-w-4xl mx-auto">
  <!-- Hero -->
  <div class="mb-8">
    <h1 class="text-3xl font-bold tracking-tight mb-2">
      {#if totalFiles > 0}
        Codebase Overview
      {:else}
        Welcome to Codeilus
      {/if}
    </h1>
    <p class="text-[var(--c-text-secondary)] text-base">
      {#if totalFiles > 0}
        {totalFiles} files across {languageCount} language{languageCount !== 1 ? 's' : ''}, {totalSloc.toLocaleString()} lines of code.
      {:else}
        Turn any codebase into an interactive learning experience.
      {/if}
    </p>
  </div>

  {#if totalFiles > 0}
    <!-- Stats -->
    <div class="grid grid-cols-2 sm:grid-cols-4 gap-4 mb-8">
      <div class="stat-card">
        <div class="stat-icon bg-indigo-500/10">
          <GitBranch size={18} class="text-indigo-400" />
        </div>
        <div class="stat-value">{totalFiles}</div>
        <div class="stat-label">Files</div>
      </div>
      <div class="stat-card">
        <div class="stat-icon bg-teal-500/10">
          <BarChart3 size={18} class="text-teal-400" />
        </div>
        <div class="stat-value">{totalSloc.toLocaleString()}</div>
        <div class="stat-label">Lines of Code</div>
      </div>
      <div class="stat-card">
        <div class="stat-icon bg-amber-500/10">
          <Zap size={18} class="text-amber-400" />
        </div>
        <div class="stat-value">{languageCount}</div>
        <div class="stat-label">Languages</div>
      </div>
      <div class="stat-card">
        <div class="stat-icon bg-pink-500/10">
          <Compass size={18} class="text-pink-400" />
        </div>
        <div class="stat-value">{communities.length}</div>
        <div class="stat-label">Modules</div>
      </div>
    </div>

    <!-- Language pills -->
    {#if topLanguages.length > 0}
      <div class="flex items-center gap-2 mb-8">
        <span class="text-xs text-[var(--c-text-muted)] uppercase tracking-wider font-medium">Languages</span>
        {#each topLanguages as lang}
          <span class="text-xs px-2.5 py-1 rounded-full bg-[var(--surface-2)] border border-[var(--c-border)] text-[var(--c-text-secondary)] font-medium">{lang}</span>
        {/each}
      </div>
    {/if}

    <!-- Overview Narrative -->
    {#if overview?.content}
      <section class="mb-8">
        <div class="flex items-center gap-2 mb-3">
          <FileText size={16} class="text-[var(--c-accent)]" />
          <h2 class="text-sm font-semibold uppercase tracking-wider text-[var(--c-text-muted)]">What This Project Does</h2>
        </div>
        <div class="bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl p-5">
          <Markdown content={overview.content} />
        </div>
      </section>
    {/if}

    <!-- Architecture Narrative -->
    {#if architecture?.content}
      <section class="mb-8">
        <div class="flex items-center gap-2 mb-3">
          <Map size={16} class="text-teal-400" />
          <h2 class="text-sm font-semibold uppercase tracking-wider text-[var(--c-text-muted)]">Architecture</h2>
        </div>
        <div class="bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl p-5">
          <Markdown content={architecture.content} />
        </div>
      </section>
    {/if}

    <!-- Reading Order -->
    {#if readingOrder?.content}
      <section class="mb-8">
        <div class="flex items-center gap-2 mb-3">
          <BookOpen size={16} class="text-amber-400" />
          <h2 class="text-sm font-semibold uppercase tracking-wider text-[var(--c-text-muted)]">Where to Start Reading</h2>
        </div>
        <div class="bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl p-5">
          <Markdown content={readingOrder.content} />
        </div>
      </section>
    {/if}

    <!-- Quick Start: Chapters -->
    {#if chapters.length > 0}
      <section class="mb-8">
        <div class="flex items-center justify-between mb-3">
          <div class="flex items-center gap-2">
            <Users size={16} class="text-pink-400" />
            <h2 class="text-sm font-semibold uppercase tracking-wider text-[var(--c-text-muted)]">Learning Path — {chapters.length} Chapters</h2>
          </div>
          <a href="/learn" class="text-sm text-[var(--c-accent)] hover:text-[var(--c-accent-hover)] font-medium flex items-center gap-1">
            View all <ArrowRight size={14} />
          </a>
        </div>
        <div class="grid grid-cols-2 gap-3">
          {#each chapters.slice(0, 4) as chapter, i}
            <a href="/learn" class="p-4 bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl hover:border-[var(--c-border-hover)] transition-colors group">
              <div class="flex items-start gap-3">
                <div class="w-8 h-8 rounded-lg bg-[var(--c-accent)]/10 flex items-center justify-center text-[var(--c-accent)] font-bold text-sm shrink-0">
                  {chapter.order_index + 1}
                </div>
                <div class="min-w-0">
                  <h3 class="text-sm font-semibold text-[var(--c-text-primary)] mb-1 truncate">{chapter.title}</h3>
                  <p class="text-xs text-[var(--c-text-muted)] line-clamp-2">{chapter.description}</p>
                  <span class="inline-block mt-2 text-[10px] uppercase tracking-wider px-2 py-0.5 rounded-md bg-[var(--surface-2)] text-[var(--c-text-muted)] font-medium">{chapter.difficulty}</span>
                </div>
              </div>
            </a>
          {/each}
        </div>
      </section>
    {/if}
  {/if}

  <!-- Action Cards -->
  <div class="grid grid-cols-1 sm:grid-cols-3 gap-4 mb-8">
    <a href="/learn" class="action-card group">
      <div class="w-10 h-10 rounded-xl bg-indigo-500/10 flex items-center justify-center mb-4 group-hover:bg-indigo-500/20 transition-colors">
        <BookOpen size={20} class="text-indigo-400" />
      </div>
      <h3 class="text-base font-semibold mb-1 text-[var(--c-text-primary)]">Learn</h3>
      <p class="text-sm text-[var(--c-text-secondary)] mb-4 leading-relaxed">Guided chapters with explanations</p>
      <div class="flex items-center gap-1 text-[var(--c-accent)] text-sm font-medium mt-auto">
        <span>Start learning</span>
        <ArrowRight size={14} class="group-hover:translate-x-1 transition-transform" />
      </div>
    </a>

    <a href="/explore" class="action-card group">
      <div class="w-10 h-10 rounded-xl bg-teal-500/10 flex items-center justify-center mb-4 group-hover:bg-teal-500/20 transition-colors">
        <Compass size={20} class="text-teal-400" />
      </div>
      <h3 class="text-base font-semibold mb-1 text-[var(--c-text-primary)]">Explore</h3>
      <p class="text-sm text-[var(--c-text-secondary)] mb-4 leading-relaxed">File tree, graph, metrics, diagrams</p>
      <div class="flex items-center gap-1 text-teal-400 text-sm font-medium mt-auto">
        <span>Browse code</span>
        <ArrowRight size={14} class="group-hover:translate-x-1 transition-transform" />
      </div>
    </a>

    <a href="/ask" class="action-card group">
      <div class="w-10 h-10 rounded-xl bg-amber-500/10 flex items-center justify-center mb-4 group-hover:bg-amber-500/20 transition-colors">
        <MessageCircle size={20} class="text-amber-400" />
      </div>
      <h3 class="text-base font-semibold mb-1 text-[var(--c-text-primary)]">Ask</h3>
      <p class="text-sm text-[var(--c-text-secondary)] mb-4 leading-relaxed">Ask questions, AI-powered answers</p>
      <div class="flex items-center gap-1 text-amber-400 text-sm font-medium mt-auto">
        <span>Ask a question</span>
        <ArrowRight size={14} class="group-hover:translate-x-1 transition-transform" />
      </div>
    </a>
  </div>

  <OnboardingBanner show={totalFiles === 0 && loaded} />

  <!-- Server status -->
  <div class="flex items-center gap-2 mt-6 text-xs text-[var(--c-text-muted)]">
    <span class="w-1.5 h-1.5 rounded-full {health === 'ok' ? 'bg-emerald-400' : 'bg-red-400'}"></span>
    <span>Server {health === 'ok' ? 'connected' : health}</span>
  </div>
</div>

<style>
  @reference "tailwindcss";
  .stat-card {
    @apply p-4 bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl;
  }
  .stat-icon {
    @apply w-9 h-9 rounded-lg flex items-center justify-center mb-3;
  }
  .stat-value {
    @apply text-2xl font-bold tracking-tight text-[var(--c-text-primary)];
  }
  .stat-label {
    @apply text-xs text-[var(--c-text-muted)] font-medium mt-0.5;
  }
  .action-card {
    @apply flex flex-col p-5 bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl hover:border-[var(--c-border-hover)] hover:bg-[var(--surface-2)] transition-all;
  }
</style>
