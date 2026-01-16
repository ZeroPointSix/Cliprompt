# Plan

## Iteration 1 (MVP polish)

- [x] Include full prompt body in search matching.
- [x] Show current prompts folder and quick-open it.
- [x] Keep docs and dev log aligned with changes.
- [x] Run `npm run check` after code updates.

## Iteration 2 (Quality of life)

- [x] Add directory tags from folder names.
- [x] Seed sample prompts when the folder is empty.
- [x] Run `npm run check` after code updates.

## Iteration 3 (Tray integration)

- [x] Add a system tray menu with show/hide/quit.
- [x] Toggle window on tray click while capturing active focus.
- [x] Run `npm run check` after code updates.

## Iteration 4 (Auto-start)

- [x] Add start-with-Windows toggle in UI.
- [x] Persist auto-start in config and apply to registry.
- [x] Run `npm run check` after code updates.

## Iteration 5 (Rust search)

- [x] Move fuzzy ranking to Rust for large prompt sets.
- [x] Query backend on input with simple debounce.
- [x] Run `npm run check` after code updates.

## Iteration 6 (Startup diagnostics)

- [x] Report auto-start registration failures in the UI.
- [x] Run `npm run check` after code updates.

## Iteration 7 (Quick actions)

- [x] Add a copy-only action in the preview pane.
- [x] Run `npm run check` after code updates.

## Iteration 8 (Title copy)

- [x] Add a copy-title quick action in the preview pane.
- [x] Run `npm run check` after code updates.

## Iteration 9 (Settings drawer)

- [x] Add a compact settings drawer with hotkey guidance and status.
- [x] Run `npm run check` after code updates.

## Iteration 10 (Favorites)

- [x] Add a favorites toggle per prompt and a favorites-only filter.
- [x] Run `npm run check` after code updates.

## Iteration 11 (Favorites shortcut)

- [x] Add a keyboard shortcut to toggle favorites for the active prompt.
- [x] Run `npm run check` after code updates.

## Iteration 12 (Favorites badge)

- [x] Show favorites count and active filter state in the header.
- [x] Run `npm run check` after code updates.

## Iteration 13 (Favorites section)

- [x] Show a favorites section pinned at the top of the list.
- [x] Run `npm run check` after code updates.

## Iteration 14 (Fuzzy matcher)

- [x] Improve Rust scoring with term matching and word-boundary bias.
- [x] Run `npm run check` after code updates.

## Iteration 15 (Filter chip)

- [x] Add a filter mode chip in the header.
- [x] Run `npm run check` after code updates.

## Iteration 16 (Copy path)

- [x] Add a copy-path action in the preview pane.
- [x] Run `npm run check` after code updates.

## Iteration 17 (Recent prompts)

- [x] Track recently used prompts and show a recent section.
- [x] Run `npm run check` after code updates.

## Iteration 18 (Recent controls)

- [x] Add recent filter and enable/clear controls.
- [x] Run `npm run check` after code updates.

## Iteration 19 (Snippets)

- [x] Show matched snippets in list rows while searching.
- [x] Run `npm run check` after code updates.

## Iteration 20 (Recent shortcut)

- [x] Add a shortcut to clear recent history.
- [x] Run `npm run check` after code updates.

## Iteration 21 (Recent toggle shortcut)

- [x] Add a shortcut to toggle the recent filter.
- [x] Run `npm run check` after code updates.

## Iteration 22 (Last used)

- [x] Show last-used timestamp in the preview panel.
- [x] Run `npm run check` after code updates.

## Iteration 23 (Copy snippet)

- [x] Add a copy-snippet action in the preview panel.
- [x] Run `npm run check` after code updates.

## Iteration 24 (Favorites shortcut)

- [x] Add a shortcut to toggle the favorites filter.
- [x] Run `npm run check` after code updates.

## Iteration 25 (Snippet highlights)

- [x] Highlight matched terms in list snippets.
- [x] Run `npm run check` after code updates.

## Iteration 26 (Tag filters)

- [x] Make tag chips clickable to toggle #tag filters.
- [x] Run `npm run check` after code updates.

## Iteration 27 (Preview highlights)

- [x] Highlight matched terms inside the preview pane.
- [x] Run `npm run check` after code updates.

## Iteration 28 (Recent filter)

- [x] Apply the recent filter to search results.
- [x] Run `npm run check` after code updates.

## Iteration 29 (Top tags)

- [x] Add a top-tags bar for quick #tag filtering.
- [x] Run `npm run check` after code updates.

## Iteration 30 (Clear tag filters)

- [x] Add a clear button to remove all #tag filters.
- [x] Run `npm run check` after code updates.

## Iteration 31 (Top tags scope)

- [x] Add a scope toggle for top-tags (all vs results).
- [x] Run `npm run check` after code updates.

## Iteration 32 (Top tags scope persistence)

- [x] Persist the top-tags scope toggle in config.
- [x] Run `npm run check` after code updates.

## Iteration 33 (Top tags scope shortcut)

- [x] Add a shortcut to toggle the top-tags scope.
- [x] Run `npm run check` after code updates.

## Iteration 34 (Reset search)

- [x] Add a reset button to clear the search query and tag filters.
- [x] Run `npm run check` after code updates.

## Iteration 35 (Auto tag scope)

- [x] Auto-switch top-tags scope when Favorites/Recent filters are active.
- [x] Run `npm run check` after code updates.

## Iteration 36 (Auto scope indicator)

- [x] Show an auto indicator when top-tags scope is overridden by filters.
- [x] Run `npm run check` after code updates.

## Iteration 37 (Top tags count)

- [x] Allow configuring how many top tags are shown.
- [x] Run `npm run check` after code updates.

## Iteration 38 (Shortcuts legend)

- [x] Add a collapsible shortcuts legend under the search bar.
- [x] Run `npm run check` after code updates.

## Iteration 39 (Copy tags)

- [x] Add a copy-tags action in the preview pane.
- [x] Run `npm run check` after code updates.

## Iteration 40 (Reset filters)

- [x] Add a reset action that clears search, tags, and list filters.
- [x] Run `npm run check` after code updates.

## Iteration 41 (Shortcuts hint)

- [x] Auto-expand the shortcuts legend on first launch.
- [x] Run `npm run check` after code updates.

## Iteration 42 (Windows build fixes)

- [x] Fix Tauri build errors (tray feature, watcher typing, Windows API imports).
- [x] Run `npm run check` after code updates.

## Iteration 43 (Chinese UI)

- [x] Translate UI strings and seed prompts to Chinese.
- [x] Run `npm run check` after code updates.
