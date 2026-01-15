# Prompt Launcher (MVP)

Minimal Windows prompt manager that stays local, is file driven, and pastes into any app.

## Features

- Global hotkey toggle (default `Alt+Space`)
- Instant search over `.md` / `.txt` files
- Auto paste with optional copy-only mode
- Folder watcher for hot reload
- Right click to open source file
- Quick open for the prompts folder
- Tray menu with show/hide/quit
- Optional start with Windows
- Rust-side fuzzy ranking for large prompt sets
- Copy-only quick action in preview
- Copy-title action for snippet titles
- Compact settings drawer with hotkey guidance
- Favorites toggle with quick filter
- Favorite toggle shortcut: Ctrl+Shift+F
- Favorites count displayed in the header
- Favorites section pinned at the top of the list
- Improved fuzzy ranking with term and boundary scoring
- Filter mode chip in the header
- Copy-path action in the preview panel

## Quickstart

Prereqs: Node.js, Rust toolchain, and the Tauri Windows prerequisites.

```
npm install
npm run tauri dev
```

## Data model

- Each prompt is a single file.
- Title = filename (no extension).
- Tags can be encoded in filenames with `#tag` or `[Tag]`.
- Folder names become tags automatically.
- Search matches filename, tags, preview, and full body.

## Defaults

- Prompts folder: `Documents/PromptLauncher/Prompts`
- Config file: `AppConfig/config.json` (Tauri app config directory)
- Empty prompt folders are seeded with a few sample files.

## Notes

- Enter = paste (or copy when auto paste is off).
- Right click = open the file.
- Double click = paste.
