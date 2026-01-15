# Prompt Launcher (MVP)

Minimal Windows prompt manager that stays local, is file driven, and pastes into any app.

## Features

- Global hotkey toggle (default `Alt+Space`)
- Instant search over `.md` / `.txt` files
- Auto paste with optional copy-only mode
- Folder watcher for hot reload
- Right click to open source file
- Quick open for the prompts folder

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
- Search matches filename, tags, preview, and full body.

## Defaults

- Prompts folder: `Documents/PromptLauncher/Prompts`
- Config file: `AppConfig/config.json` (Tauri app config directory)

## Notes

- Enter = paste (or copy when auto paste is off).
- Right click = open the file.
- Double click = paste.
