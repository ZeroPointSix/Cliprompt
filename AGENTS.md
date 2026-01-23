# Repository Guidelines

## Project Structure & Module Organization
- `prompt-launcher/` is the actual app workspace. Frontend UI lives in `prompt-launcher/src/` (SvelteKit), desktop backend in `prompt-launcher/src-tauri/` (Rust/Tauri), and static assets in `prompt-launcher/static/`.
- Repository-level docs and planning material are in `docs/`, `CODE_ANALYSIS/`, and top-level Markdown files.
- Tauri configuration is in `prompt-launcher/src-tauri/tauri.conf.json`; icons are in `prompt-launcher/src-tauri/icons/`.

## Build, Test, and Development Commands
Run commands from `prompt-launcher/` unless noted.
- `npm install` installs frontend dependencies.
- `npm run dev` starts the Vite dev server (UI only).
- `npm run tauri dev` launches the full desktop app (SvelteKit + Tauri).
- `npm run build` builds the SvelteKit frontend.
- `npm run tauri build` produces the Windows installer under `prompt-launcher/src-tauri/target/release/bundle/`.
- `npm run check` runs `svelte-check` with the repo TypeScript settings.

## Coding Style & Naming Conventions
- Svelte/TS uses 2-space indentation and double quotes in existing files; follow the surrounding style.
- SvelteKit route files use the standard `+page.svelte`, `+layout.js` naming pattern.
- Rust code under `prompt-launcher/src-tauri/src/` follows rustfmt defaults (4-space indentation, snake_case modules/functions).

## Testing Guidelines
- There is no dedicated automated test suite yet. Use `npm run check` for static analysis.
- For changes affecting UX or Tauri commands, validate manually with `npm run tauri dev`.
- If you add tests later, keep them near their modules and document how to run them here.

## Commit & Pull Request Guidelines
- Recent commits show short, imperative messages with occasional Conventional prefixes (e.g., `fix:`, `docs:`, `refactor:`). Keep messages concise and scoped.
- PRs should describe user-visible behavior, link any related issues, and include screenshots or GIFs for UI changes.
- Note any Tauri/Rust changes in the PR description because they affect packaging.

## Security & Configuration Tips
- Do not commit local prompt data or user configs. Default data lives under the user Documents folder and Tauri AppConfig directories.
- If adding new settings, update the defaults and document them in `README.md`.
