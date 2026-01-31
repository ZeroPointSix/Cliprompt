# Long-Term Improvements

## Search Indexing
- Introduce a real search index (e.g., Tantivy or SQLite FTS) to support fast, scalable search with ranked results and partial matches without scanning all prompt bodies on every query.
- Store pre-tokenized terms and tag metadata for faster scoring and filtering.

## Incremental Prompt Store
- Maintain an in-memory cache keyed by path with metadata (mtime, size, tags, preview) and only re-read files that changed.
- Persist a lightweight index on disk to accelerate startup (load metadata without reading bodies).

## IO and Memory
- Read prompt bodies lazily: return metadata + preview for list/search, and load full body only when a prompt is opened/copied.
- Consider size limits for prompt content to prevent extreme memory usage when very large files exist in the prompt directory.

## UI Performance
- Virtualize large lists if you increase `maxResults` or add a full list view.
- Move snippet generation to the backend so the frontend only renders precomputed snippets with highlight positions.

## Observability
- Add basic timing instrumentation for indexing and search (e.g., `Instant::now()` in Rust, `performance.now()` in UI) to validate improvements.
