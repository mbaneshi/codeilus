# Perplexity Research Brief — Codeilus

Copy-paste this into Perplexity Pro for targeted research.

---

## PROMPT 1: MCP Server Best Practices

```
I'm building "Codeilus" — a Rust-based codebase analysis tool (single binary) that parses any repo using tree-sitter, builds a knowledge graph (petgraph), computes metrics, detects patterns, and generates narratives via Claude CLI.

I want to make it serve as a downstream MCP (Model Context Protocol) server that AI tools like Claude Code, Cursor, Windsurf, Cline, Google ADK agents, and Claude Cowork can connect to.

Current state:
- 8 MCP tools implemented via rmcp (Rust MCP SDK) over stdio transport
- Tools: query_symbols, query_graph, get_context, get_impact, get_diagram, get_metrics, get_learning_status, explain_symbol
- Want to add: SSE and HTTP Streamable transports
- Want to add: MCP Resources (expose graph, communities, metrics as resources)
- Want to add: MCP Prompts (PR review, onboarding, feature planning)

Research for me:
1. What are the best practices for designing MCP tool schemas that AI agents can use effectively? (parameter design, response structure, error handling)
2. How do Claude Code, Cursor, and other AI IDEs discover and use MCP servers? What config format do they expect?
3. What's the current state of MCP transport options? (stdio vs SSE vs HTTP Streamable) Which should I prioritize for widest compatibility?
4. What are examples of production MCP servers that serve as "intelligence layers" (not just wrappers around APIs)?
5. How can MCP Resources and Prompts enhance the developer experience beyond just tools?
6. How does Google ADK connect to MCP servers? Any special considerations?
7. What is "Claude Cowork" and "Antigravity" in the context of AI development tools with MCP support?
```

---

## PROMPT 2: Codebase Intelligence Architecture

```
I'm building a codebase intelligence engine in Rust that analyzes any repository and produces:
- Knowledge graph (symbols as nodes, CALLS/IMPORTS/EXTENDS/IMPLEMENTS as edges)
- Community detection (Louvain algorithm)
- Metrics (cyclomatic complexity, fan-in/out, modularity, TF-IDF)
- Anti-pattern detection (god classes, long methods, circular deps, security hotspots)
- Entry point scoring
- Execution flow tracing

Current challenge: My graph has 875 symbols and 2540 edges, but 99.9% are CALLS edges. IMPORTS resolution is weak (only 2 edges). Heritage (EXTENDS/IMPLEMENTS) is sparse (1 edge).

Research for me:
1. How do tools like Sourcegraph, CodeScene, and Understand build rich dependency graphs? What edge types beyond CALLS and IMPORTS are most valuable?
2. For Rust codebases specifically: how should I resolve `use crate::module::Type` statements to build IMPORTS edges? What about re-exports (`pub use`)?
3. What additional edge types would be most valuable for AI agents trying to understand a codebase? (USES_TYPE, READS_FIELD, WRITES_FIELD, TESTS, CONTAINS?)
4. What graph algorithms beyond Louvain community detection are useful for codebase understanding? (PageRank for importance? Betweenness centrality for bottlenecks?)
5. How should I label communities semantically — not by directory name but by purpose? What heuristics work?
6. What's the state of the art for "impact analysis" — given a symbol, determine what would break if it changes?
```

---

## PROMPT 3: Static Codebase Knowledge Pages

```
I want to build a pipeline that:
1. Scrapes GitHub trending repos daily
2. Analyzes each repo (parse, graph, metrics, narratives)
3. Generates a single self-contained HTML page per repo (<500KB)
4. Deploys to a static site (Cloudflare Pages)

The page should let someone "grasp" a repo in 5 minutes:
- 30-second overview (what it does, for who, why it matters)
- Architecture diagram (auto-generated from real graph data)
- Key files to read first (ranked by graph importance)
- Entry points
- How it works (architecture in plain English)
- Extension points
- Contribution guide
- Metrics snapshot

Research for me:
1. What existing tools generate "repo summary" or "repo understanding" pages? (e.g., GitDiagram, Repomix, CodeSee, Gitingest) What do they do well/poorly?
2. For a single self-contained HTML page: what's the best approach for embedding interactive visualizations (graph, diagrams) without a framework runtime? (vanilla JS + WebGL? D3? Mermaid?)
3. What's the optimal information hierarchy for a 5-minute repo understanding page? What do developers look at first?
4. How should I generate the "key files to read first" ranking? What signals from the graph/metrics best predict learning value?
5. Are there academic papers or blog posts on "pedagogical ordering" of code — teaching a codebase in the right sequence?
6. What's the cheapest way to deploy 25+ static pages daily? (Cloudflare Pages free tier limits, GitHub Pages, Netlify, Vercel?)
```

---

## PROMPT 4: Making Codeilus Production-Ready

```
I have a Rust CLI tool ("Codeilus") with 16 internal crates, 25K lines of Rust, 243 passing tests, zero clippy warnings. It:
- Parses codebases with tree-sitter (6 languages)
- Builds knowledge graphs with petgraph
- Computes metrics
- Serves a SvelteKit 5 frontend embedded via rust-embed
- Exposes an MCP server for AI agent integration
- Can scrape GitHub trending, analyze repos, and generate static HTML pages

I want to make it production-ready for open-source release. Research for me:
1. What's the best strategy for cross-platform binary releases from Rust? (GitHub Actions, cargo-dist, cross?)
2. How should I handle the "single binary with embedded frontend" pattern for distribution? Any gotchas with rust-embed at scale?
3. What makes a great developer tool README/landing page? Examples of open-source CLI tools with excellent docs?
4. What's the best way to do auto-updates for a CLI tool? (homebrew tap, cargo-binstall, self-update?)
5. How should I handle the Claude CLI dependency gracefully? (codeilus works without it, but LLM features need it)
6. What telemetry/analytics should an open-source CLI tool collect (opt-in) to understand usage patterns?
```
