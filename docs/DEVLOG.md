# Dev Log

## 2026-01-15

- Scaffolded Tauri v2 + SvelteKit project in `prompt-launcher`.
- Implemented prompt indexing, file watching, and focus/paste flow in Rust.
- Built the MVP UI with search, preview, folder picker, and hotkey settings.
- Expanded search to include prompt bodies and added quick open for the prompts folder.
- Added `docs/PLAN.md` to track iteration scope.
- Added folder-name tags and seeded example prompts when the folder is empty.
- Added a tray menu (show/hide/quit) with click-to-toggle behavior.
- Added a start-with-Windows toggle backed by registry updates.
- Moved fuzzy search scoring to Rust with a lightweight debounce on input.
- Added UI feedback when auto-start registration fails.
- Added a copy-only quick action in the preview panel.
- Added a copy-title action for quick snippet reuse.
- Added a compact settings drawer with hotkey guidance and status.
