<script lang="ts">
  import { fetchChapters, fetchNarrative, fetchProgress, fetchLearnerStats, resetProgress } from '$lib/api';
  import type { Chapter, NarrativeResponse, Progress, LearnerStats, Badge } from '$lib/types';
  import { BookOpen, ChevronDown, ChevronRight, GraduationCap, FileText, Flame, Zap, Trophy, Award, RotateCcw } from 'lucide-svelte';
  import Markdown from '$lib/Markdown.svelte';

  let loading = $state(true);
  let chapters = $state<Chapter[]>([]);
  let expandedId = $state<number | null>(null);
  let overview = $state<NarrativeResponse | null>(null);
  let progress = $state<Progress[]>([]);
  let stats = $state<LearnerStats>({ total_xp: 0, streak_days: 0, last_active: '', chapters_completed: 0, badges: [] });

  function formatLabel(label: string): string {
    return label
      .replace(/^cluster_/, '')
      .replace(/_/g, ' ')
      .replace(/\b\w/g, (c) => c.toUpperCase());
  }

  function difficultyColor(d: string): string {
    switch (d.toLowerCase()) {
      case 'beginner': return 'text-emerald-400 bg-emerald-500/10';
      case 'intermediate': return 'text-amber-400 bg-amber-500/10';
      case 'advanced': return 'text-red-400 bg-red-500/10';
      default: return 'text-[var(--c-text-muted)] bg-[var(--surface-3)]';
    }
  }

  function difficultyIcon(d: string): string {
    switch (d.toLowerCase()) {
      case 'beginner': return '🌱';
      case 'intermediate': return '⚡';
      case 'advanced': return '🔥';
      default: return '📘';
    }
  }

  function chapterProgress(chapterId: number, sectionCount: number): number {
    if (sectionCount === 0) return 0;
    const completed = progress.filter(p => p.chapter_id === chapterId && p.completed).length;
    return Math.round((completed / sectionCount) * 100);
  }

  function toggleChapter(id: number) {
    expandedId = expandedId === id ? null : id;
  }

  let resetting = $state(false);

  async function handleResetProgress() {
    if (!confirm('Reset all progress? This will clear XP, badges, and section completions.')) return;
    resetting = true;
    await resetProgress();
    progress = [];
    stats = { total_xp: 0, streak_days: 0, last_active: '', chapters_completed: 0, badges: [] };
    resetting = false;
  }

  let overallProgress = $derived(
    chapters.length === 0 ? 0 :
    Math.round((stats.chapters_completed / chapters.length) * 100)
  );

  if (typeof window !== 'undefined') {
    Promise.all([
      fetchChapters(),
      fetchNarrative('overview'),
      fetchProgress(),
      fetchLearnerStats(),
    ]).then(([ch, o, p, s]) => {
      chapters = ch;
      overview = o;
      progress = p;
      stats = s;
      loading = false;
    }).catch(() => {
      loading = false;
    });
  }
</script>

<div class="p-8 max-w-4xl mx-auto">
  <!-- Header with XP + Streak -->
  <div class="mb-8">
    <div class="flex items-center justify-between mb-3">
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-xl bg-indigo-500/10 flex items-center justify-center">
          <BookOpen size={20} class="text-indigo-400" />
        </div>
        <div>
          <h1 class="text-2xl font-bold tracking-tight">Learning Path</h1>
          <p class="text-sm text-[var(--c-text-secondary)]">Work through the codebase one module at a time</p>
        </div>
      </div>
      <!-- XP + Streak counters -->
      {#if !loading}
        <div class="flex items-center gap-4">
          <div class="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-amber-500/10 border border-amber-500/20">
            <Zap size={16} class="text-amber-400" />
            <span class="text-sm font-bold text-amber-400">{stats.total_xp} XP</span>
          </div>
          <div class="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-orange-500/10 border border-orange-500/20">
            <Flame size={16} class="text-orange-400" />
            <span class="text-sm font-bold text-orange-400">{stats.streak_days}d streak</span>
          </div>
        </div>
      {/if}
    </div>
  </div>

  {#if loading}
    <div class="space-y-4">
      {#each [1, 2, 3] as _}
        <div class="h-24 bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl animate-pulse"></div>
      {/each}
    </div>
  {:else if chapters.length === 0}
    <div class="text-center py-20 bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl">
      <BookOpen size={40} class="text-[var(--c-text-muted)] mx-auto mb-4" />
      <p class="text-[var(--c-text-secondary)] text-lg font-medium mb-2">No chapters yet</p>
      <p class="text-[var(--c-text-muted)] text-sm">Run <code class="text-[var(--c-accent)] font-mono text-xs bg-[var(--c-accent)]/10 px-1.5 py-0.5 rounded">codeilus analyze ./repo</code> to generate the learning path</p>
    </div>
  {:else}
    <!-- Badge shelf -->
    {#if stats.badges.length > 0}
      <div class="flex items-center gap-3 p-4 bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl mb-4 overflow-x-auto">
        <Award size={16} class="text-[var(--c-text-muted)] shrink-0" />
        <span class="text-xs font-semibold uppercase tracking-wider text-[var(--c-text-muted)] shrink-0">Badges</span>
        <div class="flex items-center gap-2">
          {#each stats.badges as badge}
            <div
              class="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-indigo-500/10 border border-indigo-500/20 shrink-0"
              title="{badge.description}"
            >
              <span class="text-sm">{badge.icon}</span>
              <span class="text-xs font-medium text-indigo-300">{badge.name}</span>
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Overview bar -->
    <div class="flex items-center gap-4 p-4 bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl mb-6">
      <div class="flex-1">
        <div class="flex items-center justify-between mb-2">
          <span class="text-sm font-medium text-[var(--c-text-secondary)]">
            {stats.chapters_completed}/{chapters.length} chapters completed
          </span>
          <span class="text-xs font-medium text-[var(--c-accent)]">{overallProgress}%</span>
        </div>
        <div class="w-full bg-[var(--surface-3)] rounded-full h-2">
          <div class="h-full rounded-full bg-[var(--c-accent)] transition-all duration-500" style="width: {overallProgress}%"></div>
        </div>
      </div>
      <button
        class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-colors border border-red-500/20 bg-red-500/10 text-red-400 hover:bg-red-500/20 disabled:opacity-50 shrink-0"
        onclick={handleResetProgress}
        disabled={resetting}
      >
        <RotateCcw size={14} />
        {resetting ? 'Resetting...' : 'Reset Progress'}
      </button>
    </div>

    <!-- Overview narrative (if available) -->
    {#if overview?.content}
      <div class="bg-[var(--surface-1)] border border-indigo-500/20 rounded-xl p-5 mb-6">
        <div class="flex items-center gap-2 mb-3">
          <FileText size={15} class="text-[var(--c-accent)]" />
          <span class="text-xs font-semibold uppercase tracking-wider text-[var(--c-text-muted)]">Project Overview</span>
        </div>
        <Markdown content={overview.content} />
      </div>
    {/if}

    <!-- Chapters -->
    <div class="space-y-3">
      {#each chapters as chapter, i}
        {@const isExpanded = expandedId === chapter.id}
        {@const pct = chapterProgress(chapter.id, chapter.sections.length)}

        <div class="bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl overflow-hidden hover:border-[var(--c-border-hover)] transition-colors {isExpanded ? 'ring-1 ring-[var(--c-accent)]/20 border-indigo-500/30' : ''}">
          <!-- Chapter header -->
          <button
            class="w-full text-left p-5 flex items-start gap-4"
            onclick={() => toggleChapter(chapter.id)}
          >
            <div class="w-10 h-10 rounded-xl bg-[var(--c-accent)]/10 flex items-center justify-center text-[var(--c-accent)] font-bold text-sm shrink-0 relative">
              {#if pct === 100}
                <Trophy size={18} class="text-amber-400" />
              {:else}
                {chapter.order_index + 1}
              {/if}
            </div>
            <div class="flex-1 min-w-0">
              <div class="flex items-start justify-between gap-4">
                <div class="flex-1">
                  <h3 class="text-base font-semibold text-[var(--c-text-primary)] mb-1">{formatLabel(chapter.title)}</h3>
                  <div class="flex items-center gap-3 text-sm text-[var(--c-text-muted)]">
                    <span class="inline-flex items-center gap-1 text-xs font-medium px-2 py-0.5 rounded-md {difficultyColor(chapter.difficulty)}">
                      {difficultyIcon(chapter.difficulty)} {chapter.difficulty}
                    </span>
                    <span>{chapter.sections.length} sections</span>
                  </div>
                  <!-- Progress bar under title -->
                  {#if chapter.sections.length > 0}
                    <div class="mt-2 flex items-center gap-2">
                      <div class="flex-1 bg-[var(--surface-3)] rounded-full h-1.5">
                        <div
                          class="h-full rounded-full transition-all duration-500 {pct === 100 ? 'bg-emerald-400' : 'bg-[var(--c-accent)]'}"
                          style="width: {pct}%"
                        ></div>
                      </div>
                      <span class="text-xs text-[var(--c-text-muted)] tabular-nums">{pct}%</span>
                    </div>
                  {/if}
                </div>
                <div class="shrink-0 mt-1 text-[var(--c-text-muted)]">
                  {#if isExpanded}
                    <ChevronDown size={18} />
                  {:else}
                    <ChevronRight size={18} />
                  {/if}
                </div>
              </div>
            </div>
          </button>

          <!-- Expanded content -->
          {#if isExpanded}
            <div class="px-5 pb-5 pt-0">
              <div class="border-t border-[var(--c-border)] pt-5">
                <!-- Description -->
                {#if chapter.description}
                  <p class="text-sm text-[var(--c-text-secondary)] mb-5 leading-relaxed">{chapter.description}</p>
                {/if}

                <!-- Narrative content (LLM-generated module summary) -->
                {#if chapter.narrative}
                  <div class="bg-[var(--surface-2)] rounded-xl p-4 mb-5">
                    <div class="flex items-center gap-2 mb-2">
                      <GraduationCap size={14} class="text-[var(--c-accent)]" />
                      <span class="text-xs font-semibold uppercase tracking-wider text-[var(--c-text-muted)]">Module Explanation</span>
                    </div>
                    <Markdown content={chapter.narrative} />
                  </div>
                {/if}

                <!-- Section summary -->
                {#if chapter.sections.length > 0}
                  <div class="flex items-center gap-2 text-xs text-[var(--c-text-muted)] mb-4">
                    <span>{chapter.sections.length} sections:</span>
                    <span class="text-[var(--c-text-secondary)]">{chapter.sections.map(s => s.title).join(' · ')}</span>
                  </div>
                {/if}

                <!-- Open chapter detail link -->
                <a
                  href="/learn/{chapter.id}"
                  class="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-[var(--c-accent)] text-white text-sm font-medium hover:bg-[var(--c-accent)]/80 transition-colors"
                >
                  <BookOpen size={14} />
                  {pct > 0 && pct < 100 ? 'Continue Learning' : pct === 100 ? 'Review Chapter' : 'Start Chapter'}
                </a>
              </div>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>
