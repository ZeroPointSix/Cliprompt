# Quick Wins

1) Debounce filesystem refreshes
- Add a short debounce in the watcher callback before calling `refresh_prompts` to reduce repeated full scans during save storms.

2) Split heavy UI effects
- Move `allTags` building into its own `$effect` that depends only on `allPrompts`.
- Keep `topTags` calculation tied only to its data source to avoid recomputation on unrelated state changes.

3) Memoize search terms/regex in UI
- Create a derived `searchTerms` array once per query.
- Build a single regex once per query and reuse in `highlightSnippet` for all rows.

4) Avoid redundant window resizes
- Track last applied height and skip `appWindow.setSize` if unchanged.
- Remove verbose console logs in the resize effect (or guard behind debug flag).

5) Precompute lowercase tags list for suggestions
- Maintain `allTagsLower = allTags.map(t => t.toLowerCase())` to avoid lowering each tag on every keypress.
