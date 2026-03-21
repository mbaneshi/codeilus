# Codeilus — Optimization & Performance Plan

> Audit date: 2026-03-18

## Backend (Rust/Axum)

### 1. N+1 Query Problems — HIGH PRIORITY

| Location | Issue | Fix |
|----------|-------|-----|
| `crates/codeilus-api/src/routes/graph.rs` — `list_communities()` | 1 query for communities + N queries for members (one per community) | Single query with JOIN or batch `IN (...)` clause |
| `crates/codeilus-api/src/routes/graph.rs` — `list_processes()` | 1 query for processes + N queries for steps | Same: JOIN or batch load |
| `crates/codeilus-api/src/routes/ask.rs` — `ask_stream()` | Loads context symbols one-by-one | Batch with `WHERE id IN (...)` |
| `crates/codeilus-api/src/routes/learning.rs` — `get_learner_stats()` | 3 separate queries (stats, completed chapters, badges) | Single query with JOINs |
| `crates/codeilus-api/src/routes/learning.rs` — `skip_chapter()` | Nested loop: fetch sections, check each, record each | Batch insert + aggregate completion check |

### 2. Missing Database Indexes — HIGH PRIORITY

Add to a new migration:

```sql
CREATE INDEX IF NOT EXISTS idx_community_members_community ON community_members(community_id);
CREATE INDEX IF NOT EXISTS idx_community_members_symbol ON community_members(symbol_id);
CREATE INDEX IF NOT EXISTS idx_process_steps_process ON process_steps(process_id);
CREATE INDEX IF NOT EXISTS idx_chapters_community ON chapters(community_id);
CREATE INDEX IF NOT EXISTS idx_chapter_sections_chapter ON chapter_sections(chapter_id);
CREATE INDEX IF NOT EXISTS idx_progress_chapter ON progress(chapter_id);
```

### 3. Missing Pagination — HIGH PRIORITY

These endpoints return all rows with no limit. Add `limit` (default 50, max 200) and `offset` query params:

- `GET /api/v1/symbols`
- `GET /api/v1/files`
- `GET /api/v1/narratives`
- `GET /api/v1/chapters`
- `GET /api/v1/communities`
- `GET /api/v1/processes`
- `GET /api/v1/annotations`

Already paginated (no change needed): `/api/v1/graph`, `/api/v1/search`.

### 4. Expand Moka Cache — MEDIUM PRIORITY

Current state: moka cache with 100-entry capacity and 10-minute TTL covers only graph, narratives, and chapters.

**Add caching to:**

| Endpoint | Cache Key | Suggested TTL |
|----------|-----------|---------------|
| `GET /api/v1/symbols` | `"symbols:l={limit}:o={offset}"` | 10 min |
| `GET /api/v1/files` | `"files:l={limit}:o={offset}"` | 10 min |
| `GET /api/v1/search?q=...` | `"search:q={query}:l={limit}"` | 5 min |
| `GET /api/v1/learner/stats` | `"learner:stats"` | 2 min |
| `GET /api/v1/files/:id/source` | `"file:source:{id}"` | 10 min |
| `GET /api/v1/chapters/:id/quiz` | `"quiz:{id}"` | 10 min |

Also increase capacity from 100 to 500 entries.

### 5. HTTP Caching Headers — MEDIUM PRIORITY

Add middleware or per-route headers for read-only endpoints:

```
Cache-Control: public, max-age=300, stale-while-revalidate=60
```

For mutable endpoints (progress, quiz answers):

```
Cache-Control: no-store
```

Consider adding `ETag` based on last-modified timestamp from the pipeline run.

### 6. Cache Invalidation

Current `invalidate_all()` is too coarse. Add key-prefix invalidation so that a pipeline re-run only clears stale keys (e.g., all keys starting with `"graph:"` or `"symbols:"`).

---

## Frontend (SvelteKit)

### 7. Client-Side API Cache — MEDIUM PRIORITY

No caching exists in `frontend/src/lib/api.ts`. Every page visit re-fetches all data.

**Approach:** Add a simple in-memory cache with TTL in `api.ts`:

```typescript
const cache = new Map<string, { data: unknown; expires: number }>();
const CACHE_TTL = 5 * 60 * 1000; // 5 minutes

async function cachedFetch<T>(url: string): Promise<T> {
  const entry = cache.get(url);
  if (entry && entry.expires > Date.now()) return entry.data as T;
  const data = await apiFetch<T>(url);
  cache.set(url, { data, expires: Date.now() + CACHE_TTL });
  return data;
}
```

Use for: files, symbols, communities, narratives, chapters (read-heavy, rarely change).

### 8. Lazy-Load Shiki Languages — MEDIUM PRIORITY

Currently loads all language packs upfront in the tree page. Instead, load only the language needed for the selected file:

```typescript
const highlighter = await getHighlighter({
  themes: ['github-dark'],
  langs: [detectedLanguage], // not all languages
});
```

### 9. Vite Compression — MEDIUM PRIORITY

Add brotli/gzip compression for production builds:

```bash
pnpm add -D vite-plugin-compression
```

```typescript
// vite.config.ts
import compression from 'vite-plugin-compression';
export default defineConfig({
  plugins: [compression({ algorithm: 'brotli' })],
});
```

### 10. Virtual Scrolling for Large Lists — MEDIUM PRIORITY

File trees, symbol lists, and metric tables render all items in the DOM. For codebases with 1000+ files this will degrade.

**Candidates:**
- File tree in `/routes/explore/tree/+page.svelte`
- Symbol search results in layout sidebar
- Metrics tables in `/routes/explore/metrics/+page.svelte`

Use `svelte-virtual-list` or a simple windowed renderer.

### 11. Bundle Size Reduction — LOW PRIORITY

| Asset | Size | Action |
|-------|------|--------|
| `elk.bundled.js` | 1.5 MB | Serve with brotli compression (~300 KB), add loading indicator |
| `3d-force-graph` + `force-graph` | Both in deps | Confirm both are needed; remove unused one |
| Shiki language packs | ~1 MB+ | Lazy-load per language (see item 8) |

### 12. Manual Chunk Splitting — LOW PRIORITY

Add to `vite.config.ts` to isolate heavy deps into separate chunks:

```typescript
build: {
  rollupOptions: {
    output: {
      manualChunks: {
        'three': ['three'],
        '3d-graph': ['3d-force-graph'],
        'shiki': ['shiki'],
      },
    },
  },
},
```

---

## Implementation Order

| Phase | Items | Impact | Effort |
|-------|-------|--------|--------|
| **Phase 1** | N+1 queries (#1), DB indexes (#2) | High | Low |
| **Phase 2** | Pagination (#3), expand moka cache (#4) | High | Medium |
| **Phase 3** | HTTP headers (#5), client-side cache (#7) | Medium | Low |
| **Phase 4** | Shiki lazy-load (#8), vite compression (#9) | Medium | Low |
| **Phase 5** | Virtual scrolling (#10), bundle splitting (#11, #12) | Medium | Medium |
| **Phase 6** | Cache invalidation (#6) | Low | Medium |
