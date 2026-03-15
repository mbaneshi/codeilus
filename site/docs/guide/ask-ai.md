# Ask AI

The Ask page (`/ask`) provides streaming Q&A powered by Claude Code.

## How It Works

1. You type a question about the codebase
2. Codeilus builds context from the database (file count, symbol count, languages, selected symbols)
3. The question + context is sent to Claude Code CLI
4. The response streams back via SSE (Server-Sent Events)

## Example Questions

- "What does the `GraphBuilder` struct do?"
- "How are communities detected?"
- "What's the entry point for the CLI?"
- "How does the event bus work?"
- "What patterns does the analyzer detect?"

## Context

Codeilus provides Claude Code with:

- Repository statistics (files, symbols, languages)
- Selected symbol context (if you've pinned symbols from the graph)
- The full question

This means answers are grounded in the actual codebase structure, not generic knowledge.

## Requirements

The Ask feature requires Claude Code CLI to be installed and authenticated:

```bash
npm install -g @anthropic-ai/claude-code
claude auth
```

Without Claude Code, the Ask page shows an error message. All other features (analysis, graphs, metrics, learning path) work without it.

## LLM Provider

Codeilus uses a provider-agnostic LLM architecture. By default, it uses the Claude Code CLI with your subscription. Check the [Settings page](/settings) to see which provider is active.
