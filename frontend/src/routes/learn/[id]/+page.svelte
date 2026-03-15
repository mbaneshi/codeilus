<script lang="ts">
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import {
    fetchChapter,
    fetchChapters,
    fetchProgress,
    fetchQuiz,
    submitQuizAnswer,
    markSectionComplete,
  } from '$lib/api';
  import type { Chapter, ChapterSection, Progress, QuizQuestion } from '$lib/types';
  import {
    BookOpen, ArrowLeft, ArrowRight, Check, CheckCircle, CircleDot,
    Code2, GitBranch, HelpCircle, Loader2, Trophy, Zap, X, Eye,
  } from 'lucide-svelte';
  import Markdown from '$lib/Markdown.svelte';

  type Tab = 'overview' | 'walkthrough' | 'connections' | 'quiz';

  let loading = $state(true);
  let chapter = $state<Chapter | null>(null);
  let allChapters = $state<Chapter[]>([]);
  let progress = $state<Progress[]>([]);
  let activeTab = $state<Tab>('overview');
  let quiz = $state<QuizQuestion[]>([]);
  let quizLoading = $state(false);

  // Quiz state
  let currentQuestion = $state(0);
  let selectedAnswer = $state<string | null>(null);
  let submitted = $state(false);
  let lastResult = $state<{ correct: boolean; xp_earned: number } | null>(null);
  let xpAnimation = $state(false);
  let totalQuizXp = $state(0);
  let quizFinished = $state(false);

  // Show quiz modal
  let showQuizModal = $state(false);

  let chapterId = $derived(Number($page.params.id));

  let chapterProgress = $derived.by(() => {
    if (!chapter || chapter.sections.length === 0) return 0;
    const completed = progress.filter(p => p.chapter_id === chapterId && p.completed).length;
    return Math.round((completed / chapter.sections.length) * 100);
  });

  function isSectionCompleted(sectionId: number): boolean {
    return progress.some(p => p.chapter_id === chapterId && p.section_id === sectionId && p.completed);
  }

  let nextChapter = $derived(
    allChapters.find(c => chapter && c.order_index === chapter.order_index + 1) ?? null
  );

  let prevChapter = $derived(
    allChapters.find(c => chapter && c.order_index === chapter.order_index - 1) ?? null
  );

  function formatLabel(label: string): string {
    return label
      .replace(/^cluster_/, '')
      .replace(/_/g, ' ')
      .replace(/\b\w/g, (c) => c.toUpperCase());
  }

  function difficultyColor(d: string): string {
    switch (d.toLowerCase()) {
      case 'beginner': return 'text-emerald-400 bg-emerald-500/10 border-emerald-500/20';
      case 'intermediate': return 'text-amber-400 bg-amber-500/10 border-amber-500/20';
      case 'advanced': return 'text-red-400 bg-red-500/10 border-red-500/20';
      default: return 'text-[var(--c-text-muted)] bg-[var(--surface-3)] border-[var(--c-border)]';
    }
  }

  function kindBadgeColor(kind: string): string {
    switch (kind.toLowerCase()) {
      case 'function': return 'text-blue-400 bg-blue-500/10';
      case 'class': return 'text-purple-400 bg-purple-500/10';
      case 'struct': return 'text-teal-400 bg-teal-500/10';
      case 'module': return 'text-amber-400 bg-amber-500/10';
      case 'trait': return 'text-pink-400 bg-pink-500/10';
      case 'impl': return 'text-cyan-400 bg-cyan-500/10';
      default: return 'text-[var(--c-text-muted)] bg-[var(--surface-3)]';
    }
  }

  async function handleMarkComplete(sectionId: number) {
    const success = await markSectionComplete(chapterId, sectionId);
    if (success) {
      progress = [...progress, { chapter_id: chapterId, section_id: sectionId, completed: true, completed_at: new Date().toISOString() }];
    }
  }

  async function loadQuiz() {
    quizLoading = true;
    quiz = await fetchQuiz(chapterId);
    currentQuestion = 0;
    selectedAnswer = null;
    submitted = false;
    lastResult = null;
    totalQuizXp = 0;
    quizFinished = false;
    quizLoading = false;
    showQuizModal = true;
  }

  async function handleSubmitAnswer() {
    if (!selectedAnswer || !quiz[currentQuestion]) return;
    submitted = true;
    lastResult = await submitQuizAnswer(quiz[currentQuestion].id, selectedAnswer);
    if (lastResult.xp_earned > 0) {
      totalQuizXp += lastResult.xp_earned;
      xpAnimation = true;
      setTimeout(() => { xpAnimation = false; }, 1500);
    }
  }

  function nextQuizQuestion() {
    if (currentQuestion < quiz.length - 1) {
      currentQuestion++;
      selectedAnswer = null;
      submitted = false;
      lastResult = null;
    } else {
      quizFinished = true;
    }
  }

  function closeQuiz() {
    showQuizModal = false;
  }

  $effect(() => {
    const id = chapterId;
    loading = true;
    Promise.all([fetchChapter(id), fetchChapters(), fetchProgress()]).then(([ch, all, p]) => {
      chapter = ch;
      allChapters = all;
      progress = p;
      loading = false;
    });
  });

  const tabs: { id: Tab; label: string }[] = [
    { id: 'overview', label: 'Overview' },
    { id: 'walkthrough', label: 'Code Walkthrough' },
    { id: 'connections', label: 'Connections' },
    { id: 'quiz', label: 'Quiz' },
  ];
</script>

<div class="p-8 max-w-4xl mx-auto">
  {#if loading}
    <div class="flex items-center justify-center py-20">
      <Loader2 size={24} class="text-[var(--c-accent)] animate-spin" />
    </div>
  {:else if !chapter}
    <div class="text-center py-20 bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl">
      <BookOpen size={40} class="text-[var(--c-text-muted)] mx-auto mb-4" />
      <p class="text-[var(--c-text-secondary)] text-lg font-medium mb-2">Chapter not found</p>
      <a href="/learn" class="text-[var(--c-accent)] text-sm hover:underline">Back to learning path</a>
    </div>
  {:else}
    <!-- Back nav -->
    <a href="/learn" class="inline-flex items-center gap-1.5 text-sm text-[var(--c-text-muted)] hover:text-[var(--c-text-secondary)] transition-colors mb-6">
      <ArrowLeft size={14} />
      Back to Learning Path
    </a>

    <!-- Chapter header -->
    <div class="bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl p-6 mb-6">
      <div class="flex items-start justify-between gap-4 mb-4">
        <div class="flex items-center gap-4">
          <div class="w-12 h-12 rounded-xl bg-[var(--c-accent)]/10 flex items-center justify-center text-[var(--c-accent)] font-bold text-lg shrink-0">
            {chapter.order_index + 1}
          </div>
          <div>
            <h1 class="text-xl font-bold tracking-tight mb-1">{formatLabel(chapter.title)}</h1>
            <div class="flex items-center gap-3">
              <span class="inline-flex items-center gap-1 text-xs font-medium px-2.5 py-1 rounded-md border {difficultyColor(chapter.difficulty)}">
                {chapter.difficulty}
              </span>
              <span class="text-sm text-[var(--c-text-muted)]">{chapter.sections.length} sections</span>
            </div>
          </div>
        </div>
        {#if chapter.community_id !== null}
          <a
            href="/explore/graph"
            class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-colors border"
            style="background: rgba(99,102,241,0.1); color: #818cf8; border-color: rgba(99,102,241,0.2)"
          >
            <GitBranch size={14} />
            View in Graph
          </a>
        {/if}
      </div>

      {#if chapter.description}
        <p class="text-sm text-[var(--c-text-secondary)] mb-4 leading-relaxed">{chapter.description}</p>
      {/if}

      <!-- Progress bar -->
      <div class="flex items-center gap-3">
        <div class="flex-1 bg-[var(--surface-3)] rounded-full h-2.5">
          <div
            class="h-full rounded-full transition-all duration-500 {chapterProgress === 100 ? 'bg-emerald-400' : 'bg-[var(--c-accent)]'}"
            style="width: {chapterProgress}%"
          ></div>
        </div>
        <span class="text-sm font-medium text-[var(--c-text-secondary)] tabular-nums">{chapterProgress}%</span>
        {#if chapterProgress === 100}
          <Trophy size={16} class="text-amber-400" />
        {/if}
      </div>
    </div>

    <!-- Tabs -->
    <div class="flex items-center gap-1 mb-6 p-1 bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl">
      {#each tabs as tab}
        <button
          class="flex-1 py-2 px-3 text-sm font-medium rounded-lg transition-colors {
            activeTab === tab.id
              ? 'bg-[var(--c-accent)] text-white'
              : 'text-[var(--c-text-muted)] hover:text-[var(--c-text-secondary)] hover:bg-[var(--surface-2)]'
          }"
          onclick={() => { activeTab = tab.id; }}
        >
          {tab.label}
        </button>
      {/each}
    </div>

    <!-- Tab content -->
    <div class="bg-[var(--surface-1)] border border-[var(--c-border)] rounded-xl p-6">
      {#if activeTab === 'overview'}
        <!-- Overview: narrative + section content -->
        {#if chapter.narrative}
          <div class="mb-6">
            <Markdown content={chapter.narrative} />
          </div>
        {/if}

        {@const overviewSection = chapter.sections.find(s => s.kind === 'overview')}
        {@const keyConcepts = chapter.sections.find(s => s.kind === 'key_concepts')}
        {@const diagramSection = chapter.sections.find(s => s.kind === 'diagram')}

        {#if overviewSection?.content}
          <div class="prose prose-invert max-w-none mb-6">
            <Markdown content={overviewSection.content} />
          </div>
        {/if}

        {#if keyConcepts?.content}
          <h3 class="text-sm font-semibold uppercase tracking-wider text-[var(--c-text-muted)] mb-3">Key Concepts</h3>
          <div class="prose prose-invert max-w-none mb-6 bg-[var(--surface-2)] rounded-lg p-4">
            <Markdown content={keyConcepts.content} />
          </div>
        {/if}

        {#if diagramSection?.content}
          <h3 class="text-sm font-semibold uppercase tracking-wider text-[var(--c-text-muted)] mb-3">Architecture</h3>
          <div class="bg-[var(--surface-2)] rounded-lg p-4 font-mono text-xs overflow-x-auto whitespace-pre text-[var(--c-text-secondary)]">{diagramSection.content}</div>
        {/if}

      {:else if activeTab === 'walkthrough'}
        <!-- Code Walkthrough: render code_walkthrough section content, or fall back to section list -->
        {@const walkthroughSection = chapter.sections.find(s => s.kind === 'code_walkthrough')}

        {#if walkthroughSection?.content}
          <div class="prose prose-invert max-w-none">
            <Markdown content={walkthroughSection.content} />
          </div>
        {:else}
          <h3 class="text-sm font-semibold uppercase tracking-wider text-[var(--c-text-muted)] mb-4">Work through each section</h3>
          <div class="space-y-2">
            {#each chapter.sections as section, idx}
              {@const completed = isSectionCompleted(section.id)}
              <div class="flex items-start gap-3 p-3 rounded-lg border transition-colors {completed ? 'bg-emerald-500/5 border-emerald-500/20' : 'bg-[var(--surface-2)] border-transparent hover:border-[var(--c-border)]'}">
                <span class="text-xs font-mono text-[var(--c-text-muted)] w-6 text-right shrink-0 pt-0.5">{idx + 1}.</span>
                <button
                  class="w-5 h-5 rounded border-2 flex items-center justify-center shrink-0 transition-colors mt-0.5 {
                    completed
                      ? 'bg-emerald-500 border-emerald-500'
                      : 'border-[var(--c-border)] hover:border-[var(--c-accent)]'
                  }"
                  onclick={() => { if (!completed) handleMarkComplete(section.id); }}
                  disabled={completed}
                >
                  {#if completed}
                    <Check size={12} class="text-white" />
                  {/if}
                </button>
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2 mb-1">
                    <span class="text-sm font-medium {completed ? 'text-[var(--c-text-muted)] line-through' : 'text-[var(--c-text-primary)]'}">{section.title}</span>
                    <span class="text-xs font-medium px-2 py-0.5 rounded-md {kindBadgeColor(section.kind)}">{section.kind}</span>
                  </div>
                  {#if section.content}
                    <div class="text-xs text-[var(--c-text-secondary)] leading-relaxed">
                      <Markdown content={section.content} />
                    </div>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        {/if}

      {:else if activeTab === 'connections'}
        <!-- Connections: render connections section content + prev/next chapter links -->
        {@const connectionsSection = chapter.sections.find(s => s.kind === 'connections')}

        {#if connectionsSection?.content}
          <div class="prose prose-invert max-w-none mb-6">
            <Markdown content={connectionsSection.content} />
          </div>
        {/if}

        <h3 class="text-sm font-semibold uppercase tracking-wider text-[var(--c-text-muted)] mb-4">Navigate</h3>

        {#if prevChapter}
          <div class="mb-4">
            <span class="text-xs text-[var(--c-text-muted)] mb-2 block">Prerequisite</span>
            <a
              href="/learn/{prevChapter.id}"
              class="flex items-center gap-3 p-4 bg-[var(--surface-2)] rounded-lg border border-[var(--c-border)] hover:border-[var(--c-border-hover)] transition-colors"
            >
              <ArrowLeft size={14} class="text-[var(--c-text-muted)]" />
              <div>
                <div class="text-sm font-medium text-[var(--c-text-primary)]">{formatLabel(prevChapter.title)}</div>
                <div class="text-xs text-[var(--c-text-muted)]">Chapter {prevChapter.order_index + 1}</div>
              </div>
            </a>
          </div>
        {/if}

        {#if nextChapter}
          <div>
            <span class="text-xs text-[var(--c-text-muted)] mb-2 block">Next Up</span>
            <a
              href="/learn/{nextChapter.id}"
              class="flex items-center gap-3 p-4 bg-[var(--surface-2)] rounded-lg border border-[var(--c-border)] hover:border-[var(--c-border-hover)] transition-colors"
            >
              <div class="flex-1">
                <div class="text-sm font-medium text-[var(--c-text-primary)]">{formatLabel(nextChapter.title)}</div>
                <div class="text-xs text-[var(--c-text-muted)]">Chapter {nextChapter.order_index + 1}</div>
              </div>
              <ArrowRight size={14} class="text-[var(--c-text-muted)]" />
            </a>
          </div>
        {/if}

        {#if !prevChapter && !nextChapter && !connectionsSection?.content}
          <p class="text-sm text-[var(--c-text-muted)]">This is the only chapter in the learning path.</p>
        {/if}

      {:else if activeTab === 'quiz'}
        <!-- Quiz launcher -->
        <div class="text-center py-8">
          <HelpCircle size={40} class="text-[var(--c-accent)] mx-auto mb-4" />
          <h3 class="text-lg font-semibold mb-2">Test Your Knowledge</h3>
          <p class="text-sm text-[var(--c-text-muted)] mb-6">Answer questions about this chapter to earn XP</p>
          <button
            class="inline-flex items-center gap-2 px-5 py-2.5 rounded-lg bg-[var(--c-accent)] text-white text-sm font-medium hover:bg-[var(--c-accent)]/80 transition-colors disabled:opacity-50"
            onclick={loadQuiz}
            disabled={quizLoading}
          >
            {#if quizLoading}
              <Loader2 size={16} class="animate-spin" />
              Loading...
            {:else}
              <Zap size={16} />
              Start Quiz
            {/if}
          </button>
        </div>
      {/if}
    </div>

    <!-- Next chapter button (when 100% complete) -->
    {#if chapterProgress === 100 && nextChapter}
      <div class="mt-6 text-center">
        <a
          href="/learn/{nextChapter.id}"
          class="inline-flex items-center gap-2 px-6 py-3 rounded-xl bg-emerald-500 text-white font-medium hover:bg-emerald-600 transition-colors"
        >
          Next Chapter: {formatLabel(nextChapter.title)}
          <ArrowRight size={16} />
        </a>
      </div>
    {/if}
  {/if}
</div>

<!-- Quiz Modal -->
{#if showQuizModal}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4">
    <!-- Backdrop -->
    <button class="absolute inset-0 bg-black/60 backdrop-blur-sm" onclick={closeQuiz} aria-label="Close quiz"></button>

    <!-- Modal -->
    <div class="relative bg-[var(--surface-1)] border border-[var(--c-border)] rounded-2xl p-6 w-full max-w-lg shadow-2xl">
      <!-- Close button -->
      <button
        class="absolute top-4 right-4 text-[var(--c-text-muted)] hover:text-[var(--c-text-primary)] transition-colors"
        onclick={closeQuiz}
      >
        <X size={18} />
      </button>

      {#if quiz.length === 0}
        <div class="text-center py-8">
          <HelpCircle size={32} class="text-[var(--c-text-muted)] mx-auto mb-3" />
          <p class="text-[var(--c-text-secondary)]">No quiz questions available for this chapter yet.</p>
        </div>
      {:else if quizFinished}
        <!-- Quiz complete -->
        <div class="text-center py-6">
          <Trophy size={48} class="text-amber-400 mx-auto mb-4" />
          <h3 class="text-xl font-bold mb-2">Quiz Complete!</h3>
          <div class="flex items-center justify-center gap-2 mb-4">
            <Zap size={20} class="text-amber-400" />
            <span class="text-2xl font-bold text-amber-400">+{totalQuizXp} XP</span>
          </div>
          <button
            class="px-5 py-2 rounded-lg bg-[var(--c-accent)] text-white text-sm font-medium hover:bg-[var(--c-accent)]/80 transition-colors"
            onclick={closeQuiz}
          >
            Done
          </button>
        </div>
      {:else}
        <!-- Question -->
        {@const q = quiz[currentQuestion]}
        <div class="mb-1">
          <span class="text-xs text-[var(--c-text-muted)]">Question {currentQuestion + 1} of {quiz.length}</span>
        </div>
        <!-- Progress dots -->
        <div class="flex items-center gap-1 mb-5">
          {#each quiz as _, qi}
            <div class="flex-1 h-1 rounded-full {qi <= currentQuestion ? 'bg-[var(--c-accent)]' : 'bg-[var(--surface-3)]'}"></div>
          {/each}
        </div>

        <h3 class="text-base font-semibold mb-5">{q.question}</h3>

        <div class="space-y-2 mb-5">
          {#each q.options as option, oi}
            {@const isSelected = selectedAnswer === option}
            {@const isCorrectAnswer = submitted && lastResult?.correct && isSelected}
            {@const isWrongAnswer = submitted && !lastResult?.correct && isSelected}
            <button
              class="w-full text-left p-3 rounded-lg border-2 transition-all text-sm {
                isCorrectAnswer
                  ? 'border-emerald-500 bg-emerald-500/10 text-emerald-300'
                  : isWrongAnswer
                    ? 'border-red-500 bg-red-500/10 text-red-300'
                    : isSelected
                      ? 'border-[var(--c-accent)] bg-[var(--c-accent)]/10'
                      : 'border-[var(--c-border)] hover:border-[var(--c-border-hover)]'
              }"
              onclick={() => { if (!submitted) selectedAnswer = option; }}
              disabled={submitted}
            >
              <span class="inline-flex items-center justify-center w-6 h-6 rounded-full text-xs font-bold mr-2 {
                isSelected ? 'bg-[var(--c-accent)] text-white' : 'bg-[var(--surface-3)] text-[var(--c-text-muted)]'
              }">
                {String.fromCharCode(65 + oi)}
              </span>
              {option}
            </button>
          {/each}
        </div>

        <!-- XP animation -->
        {#if xpAnimation}
          <div class="text-center mb-3 animate-bounce">
            <span class="text-lg font-bold text-amber-400">+{lastResult?.xp_earned} XP!</span>
          </div>
        {/if}

        <!-- Submit / Next -->
        <div class="flex justify-end gap-3">
          {#if !submitted}
            <button
              class="px-4 py-2 rounded-lg bg-[var(--c-accent)] text-white text-sm font-medium hover:bg-[var(--c-accent)]/80 transition-colors disabled:opacity-40"
              onclick={handleSubmitAnswer}
              disabled={!selectedAnswer}
            >
              Submit
            </button>
          {:else}
            <button
              class="px-4 py-2 rounded-lg bg-[var(--c-accent)] text-white text-sm font-medium hover:bg-[var(--c-accent)]/80 transition-colors"
              onclick={nextQuizQuestion}
            >
              {currentQuestion < quiz.length - 1 ? 'Next Question' : 'See Results'}
            </button>
          {/if}
        </div>
      {/if}
    </div>
  </div>
{/if}
