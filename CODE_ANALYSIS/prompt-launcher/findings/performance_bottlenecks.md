# Performance Bottlenecks

## Scope
- App: prompt-launcher (SvelteKit UI + Tauri/Rust backend)
- Focus: search, indexing, UI rendering, file I/O

## High Severity
- Full re-index on every filesystem event: `refresh_prompts` calls `index_prompts`, which walks the entire prompts directory and reads every file into memory each time a single file changes. This is O(total files + total bytes) per change and can thrash on large prompt collections or rapid save events. References: `src-tauri/src/lib.rs` (refresh_prompts/start_watcher), `src-tauri/src/prompts.rs` (index_prompts/read_prompt).
- Search recomputes normalized text per prompt per query: `score_prompt` builds `tag_text`, `title_text`, and a `full_text` string with `to_lowercase` for every prompt on every keystroke. This is O(N * text length) with heavy allocation pressure. Reference: `src-tauri/src/prompts.rs`.

## Medium Severity
- UI recomputes tag metadata on every search result update: the main `$effect` rebuilds `allTags` by scanning `allPrompts` whenever `filtered` changes, causing O(N) work per keystroke even though `allPrompts` is unchanged. Reference: `src/routes/+page.svelte`.
- Highlight/snippet work per row is repeated: `getRowPreviewHtml` -> `extractTerms` -> `highlightSnippet` builds a regex and escapes text for each row render. This repeats across rows and re-renders, adding CPU overhead. Reference: `src/routes/+page.svelte`.

## Low Severity
- Window resize effect triggers `setSize` on every filtered-length change even when the computed height is unchanged, plus verbose logging. This adds IPC chatter and can introduce UI jitter. Reference: `src/routes/+page.svelte`.
- Tag suggestions lower-case each tag on every keystroke (`tag.toLowerCase()` in a filter), which is small but scales with large tag lists. Reference: `src/routes/+page.svelte`.
