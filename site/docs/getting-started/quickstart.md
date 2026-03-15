# Quick Start

## Analyze and Explore

Point Codeilus at any repository:

```bash
codeilus ./path/to/repo
```

This runs the full pipeline (parse, graph, metrics, narrate, learn) and starts the server:

```
Analyzing... 342 files, 2847 symbols, 12 communities
Step 1/8: Parsing repository...
Step 2/8: Storing parsed data...
Step 3/8: Building knowledge graph...
Step 4/8: Computing metrics...
Step 5/8: Detecting patterns...
Step 6/8: Generating diagrams...
Step 7/8: Generating narratives...
Step 8/8: Building curriculum...
Analysis complete!
Starting codeilus server on 127.0.0.1:4174
```

Open [http://localhost:4174](http://localhost:4174) in your browser.

## Step by Step

If you prefer to separate analysis from serving:

```bash
# Analyze first
codeilus analyze ./path/to/repo

# Then serve
codeilus serve --port 4174
```

## What You'll See

### Learning Path (`/learn`)
Chapters ordered by dependency, each with sections, quizzes, and progress tracking. Start with Chapter 0 (The Big Picture) and work through the codebase module by module.

### Graph Explorer (`/explore/graph`)
Interactive knowledge graph with nodes colored by community. Click any node to see its callers, callees, and connections.

### Metrics Dashboard (`/explore/metrics`)
Complexity heatmap, fan-in/out analysis, anti-pattern warnings, and hotspot identification.

### File Tree (`/explore/tree`)
Browsable file hierarchy with symbol counts, language detection, and source viewing.

### Ask AI (`/ask`)
Streaming Q&A powered by Claude Code. Ask questions about the codebase and get answers with graph context.

## Without Claude Code

Everything works without Claude Code &mdash; you just get placeholder text instead of AI-generated narratives:

```bash
CODEILUS_SKIP_LLM=1 codeilus ./repo
```

Analysis, graphs, metrics, diagrams, curriculum structure, and quizzes all function normally. Only the narrative text and Q&A require Claude Code.
