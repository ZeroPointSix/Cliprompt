# Optimization Opportunities

## Search/Indexing (Backend)
- Precompute and cache normalized fields for search: store lowercased title/tags/body or a single `search_blob` in memory to avoid per-query `to_lowercase` and `format!` allocations. Keep this updated when prompts are reloaded.
- Replace full re-index with incremental updates: update only changed paths from `notify` events. Use a `HashMap<path, PromptEntry>` and update entries on create/modify/delete instead of walking the entire tree.
- Debounce watcher events: batch rapid changes (save storms) into a single refresh using a short timer (e.g., 150-300ms) to reduce repeated full scans.
- Avoid reading full files for previews: use streaming reads (e.g., `BufRead`) to collect the first N characters for `preview`, and only read full body when needed for view/copy.

## UI Rendering (Frontend)
- Split derived effects: compute `allTags` only when `allPrompts` changes; compute `topTags` only when its data source changes; avoid tying both to every `filtered` update.
- Memoize query terms and regex: derive `searchTerms` once per query and reuse for snippet/highlight across all rows to avoid redundant regex creation and string allocations.
- Compute snippets in backend search results: return snippet + matched indices from Rust so the UI only renders and highlights, avoiding repeated per-row `makeSnippet` and `toLowerCase`.
- Skip `setSize` calls if the height is unchanged; throttle resize requests to reduce IPC overhead.

## Data/State
- Cache favorites/recent sets: store `favorites` and `recent` as `HashSet` in state to avoid repeated conversions inside `search_prompts` and UI filtering.
- Consider pagination for very large prompt lists (beyond current limit 8), especially when showing all prompts without a query.
