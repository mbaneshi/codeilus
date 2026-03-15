# Codeilus Audit — 2026-03-15

## Summary

This audit was performed by comparing the MkDocs documentation site (served at localhost:8000, built from `site/docs/`) against the actual Rust codebase (16 crates), frontend (SvelteKit 5), database schema, and API layer.

## Findings Overview

| Category | Critical | High | Medium | Low | Total |
|----------|----------|------|--------|-----|-------|
| Doc vs Code Inconsistencies | 3 | 4 | 4 | 3 | **14** |
| Architecture Concerns | 2 | 2 | 4 | — | **8** |
| Code Quality Issues | 1 | 3 | 3 | — | **7** |
| UX Gaps | 1 | 3 | 4 | — | **8** |
| **Total** | **7** | **12** | **15** | **3** | **37** |

## Top 5 Must-Fix Items

1. **Search route double-prefix bug** — `/api/v1/search` is unreachable (5 min fix)
2. **3 missing learning API endpoints** — gamification is non-functional (3 hr fix)
3. **`cargo install codeilus` doesn't work** — first-touch onboarding is broken (docs fix)
4. **DbPool is not a real connection pool** — concurrent requests will fail (2 hr fix)
5. **Path traversal vulnerability** in file source endpoint (30 min fix)

## Documents

| # | File | Contents |
|---|------|----------|
| 01 | [DOCS-VS-CODEBASE-INCONSISTENCIES](./01-DOCS-VS-CODEBASE-INCONSISTENCIES.md) | 14 documented inconsistencies with severity ratings, evidence, and impact analysis |
| 02 | [FIX-PROPOSALS](./02-FIX-PROPOSALS.md) | 12 actionable fixes with code samples, file paths, and implementation timeline (~12 hours total) |
| 03 | [SENIOR-ARCHITECTURE-REVIEW](./03-SENIOR-ARCHITECTURE-REVIEW.md) | 8 architectural concerns, 7 code quality issues, performance/security/testing recommendations |
| 04 | [UX-PRINCIPLES-ADOPTION](./04-UX-PRINCIPLES-ADOPTION.md) | 8 UX principles with wireframes, component proposals, and 4-week implementation roadmap |

## Methodology

1. MkDocs content extracted from `site/docs/` (16 markdown files)
2. Every claim cross-referenced against source code in `crates/`, `frontend/`, `migrations/`
3. Frontend API client (`api.ts`) compared endpoint-by-endpoint against backend routes
4. TypeScript types compared against Rust serde structs
5. Database schema validated against all repo/route usage
6. Security review of file I/O, CORS, subprocess spawning, and input validation
