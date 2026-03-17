# Codeilus

!!! warning "Status: Pre-release (v0.1.0-alpha)"
    All core features functional. Some narrative types may fall back to placeholders if LLM times out.

**Turn any codebase into an interactive learning experience.**

<div class="grid cards" markdown>

-   :material-rocket-launch:{ .lg .middle } __Get Started in 30 Seconds__

    ---

    ```bash
    git clone https://github.com/mbaneshi/codeilus.git
    cd codeilus && cargo build --release
    ./target/release/codeilus analyze ./any-repo
    ./target/release/codeilus serve
    # Open http://localhost:4174
    ```

    [:octicons-arrow-right-24: Installation](getting-started/installation.md)

-   :material-book-open-variant:{ .lg .middle } __Guided Learning Path__

    ---

    Auto-generated chapters ordered by dependency.
    Quizzes, XP, badges, and progress tracking.

    [:octicons-arrow-right-24: Learning Path](guide/learning-path.md)

-   :material-graph:{ .lg .middle } __Interactive Graph Explorer__

    ---

    Navigate the knowledge graph with colored communities,
    call chains, and impact analysis.

    [:octicons-arrow-right-24: Graph Explorer](guide/graph-explorer.md)

-   :material-robot:{ .lg .middle } __AI-Powered Understanding__

    ---

    8 types of pre-generated narratives plus
    streaming Q&A powered by Claude Code.

    [:octicons-arrow-right-24: Ask AI](guide/ask-ai.md)

</div>

---

## What Codeilus Does

| Before Codeilus | After Codeilus |
|---|---|
| Grep around, hope you find the right file | Guided reading order: "read these 3 files, understand 80%" |
| Stare at a class and guess what it does | LLM-generated explanations with graph context |
| No idea how modules connect | Interactive architecture diagram from real data |
| Onboarding takes weeks | Gamified learning path: chapters, quizzes, progress |
| README is the only overview | 8 types of pre-generated narrative content |

## Two Modes, One Engine

### Interactive Local Server

```bash
./target/release/codeilus analyze ./any-repo
# Analyzing... 342 files, 2847 symbols, 12 communities
# Narrating... 8 sections generated

./target/release/codeilus serve
# Open http://localhost:4174
```

Full SvelteKit 5 UI with guided learning, graph explorer, metrics dashboard, architecture diagrams, streaming Q&A, and gamification.

### Static Page Publishing

```bash
codeilus harvest --trending
codeilus export --all-harvested --output ./output
codeilus deploy ./output --cloudflare
```

Self-contained HTML pages (~300KB) that load in under 1 second. Daily automation via GitHub Actions.

---

## Architecture at a Glance

```
codeilus (single binary, ~15MB)
+-- Rust backend (Axum HTTP + WebSocket)
+-- SQLite database (WAL mode, zero config)
+-- SvelteKit 5 frontend (embedded via rust-embed)
+-- Claude Code integration (subprocess, stream-json)
```

No Docker. No PostgreSQL. No Redis. No Node.js runtime. Just one binary + Claude Code.

[:octicons-arrow-right-24: Full architecture](architecture/overview.md)
