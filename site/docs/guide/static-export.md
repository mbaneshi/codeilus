# Static Export

Codeilus can export analyzed repositories as self-contained HTML pages.

## Generate Static Pages

```bash
# Analyze a repo
codeilus analyze ./repo

# Export to HTML
codeilus export ./repo --output ./output
```

## What's Included

Each exported HTML file (~200-500KB) contains:

- Project overview and 1-line purpose
- Architecture diagram (Mermaid rendered inline)
- Key files reading order
- Entry points
- How it works (architecture narrative)
- How to extend
- How to contribute
- Why it's trending
- Metrics snapshot
- Community summaries

All data is inlined as JSON in `<script>` tags. CSS is inlined. No external dependencies.

## Harvest Pipeline

For automated daily processing of trending repos:

```bash
# Scrape GitHub trending and analyze
codeilus harvest --trending

# Export all harvested repos
codeilus export --all-harvested --date 2026-03-15 --output ./output

# Deploy to CDN
codeilus deploy ./output --cloudflare
```

## Constraints

- **200-500KB** per HTML page
- **< 1 second** load time on mobile
- **Zero server** needed &mdash; pure static files
- **Vanilla JS** &mdash; no framework runtime
