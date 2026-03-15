<script lang="ts">
  import { marked } from 'marked';

  let { content = '' }: { content: string } = $props();

  // Configure marked for safe, simple rendering
  marked.setOptions({
    breaks: true,
    gfm: true,
  });

  let html = $derived(marked.parse(content || '', { async: false }) as string);
</script>

<div class="markdown-content">
  {@html html}
</div>

<style>
  @reference "tailwindcss";
  .markdown-content {
    @apply text-sm text-[var(--c-text-secondary)] leading-relaxed;
  }
  .markdown-content :global(h1) {
    @apply text-lg font-bold text-[var(--c-text-primary)] mt-4 mb-2;
  }
  .markdown-content :global(h2) {
    @apply text-base font-semibold text-[var(--c-text-primary)] mt-4 mb-2;
  }
  .markdown-content :global(h3) {
    @apply text-sm font-semibold text-[var(--c-text-primary)] mt-3 mb-1.5;
  }
  .markdown-content :global(p) {
    @apply mb-3 last:mb-0;
  }
  .markdown-content :global(strong) {
    @apply font-semibold text-[var(--c-text-primary)];
  }
  .markdown-content :global(em) {
    @apply italic;
  }
  .markdown-content :global(code) {
    @apply font-mono text-xs bg-[var(--surface-3)] text-[var(--c-accent)] px-1.5 py-0.5 rounded;
  }
  .markdown-content :global(pre) {
    @apply bg-[var(--surface-3)] border border-[var(--c-border)] rounded-lg p-3 mb-3 overflow-x-auto;
  }
  .markdown-content :global(pre code) {
    @apply bg-transparent px-0 py-0 text-[var(--c-text-secondary)];
  }
  .markdown-content :global(ul) {
    @apply list-disc list-inside mb-3 space-y-1;
  }
  .markdown-content :global(ol) {
    @apply list-decimal list-inside mb-3 space-y-1;
  }
  .markdown-content :global(li) {
    @apply text-sm;
  }
  .markdown-content :global(hr) {
    @apply border-[var(--c-border)] my-4;
  }
  .markdown-content :global(a) {
    @apply text-[var(--c-accent)] hover:underline;
  }
  .markdown-content :global(blockquote) {
    @apply border-l-2 border-[var(--c-accent)]/30 pl-3 italic text-[var(--c-text-muted)] mb-3;
  }
</style>
