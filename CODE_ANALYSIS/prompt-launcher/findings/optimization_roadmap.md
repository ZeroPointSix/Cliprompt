# Optimization Roadmap

## Phase 0 - Baseline (0.5 day)
- Add timing logs around indexing and search paths in Rust.
- Measure search latency for a representative prompt set (e.g., 1k, 5k, 10k files).
- Capture UI re-render frequency while typing in the search box.

## Phase 1 - Quick Wins (1-2 days)
- Debounce filesystem refreshes and skip redundant window resizes.
- Split heavy `$effect` blocks in the UI to avoid O(N) work on every keystroke.
- Memoize query terms and highlight regex once per query.

## Phase 2 - Medium Effort (3-5 days)
- Add an in-memory cache keyed by path to avoid full re-indexing.
- Precompute searchable blobs (lowercased title/tags/body) on load to reduce per-query allocations.
- Reduce prompt body transfers during search by returning snippet/preview only when possible.

## Phase 3 - Long Term (1-2 weeks)
- Introduce a search index (SQLite FTS or Tantivy).
- Persist the index to disk for faster startup and incremental updates.
- Expand instrumentation dashboards and add automated performance regression checks.

## Validation
- Target: search results within <50ms for 1k prompts, <120ms for 10k prompts.
- Target: indexing updates <300ms for single-file changes.
- Confirm no UI jank during fast typing (no noticeable frame drops).
