# Module Structure

## Top-Level Layout

### Two-module monorepo layout
**Observation**: The repository has a frontend package and a Tauri Rust backend under one root.
**Location**:
- prompt-launcher/package.json:1
- prompt-launcher/src-tauri/Cargo.toml:1
**Impact**: Low - Clear separation between UI and native code.
**Recommendation**: Add a short architecture overview in docs to explain module boundaries and contracts.

## Backend Modules (prompt-launcher/src-tauri/src)

### Composition root plus feature modules
**Observation**: lib.rs wires together commands and OS integrations while feature modules handle specific concerns.
**Location**:
- prompt-launcher/src-tauri/src/lib.rs:1
- prompt-launcher/src-tauri/src/autostart.rs:9
- prompt-launcher/src-tauri/src/config.rs:8
- prompt-launcher/src-tauri/src/prompts.rs:17
- prompt-launcher/src-tauri/src/win.rs:12
**Impact**: Medium - lib.rs holds cross-cutting logic and can become a change hotspot.
**Recommendation**: Move command handlers into a `commands` module and keep platform integrations isolated.

## Frontend Modules (prompt-launcher/src/routes)

### Single-page UI with mixed responsibilities
**Observation**: +page.svelte owns state, command calls, data shaping, and rendering.
**Location**:
- prompt-launcher/src/routes/+page.svelte:42
- prompt-launcher/src/routes/+page.svelte:585
- prompt-launcher/src/routes/+page.svelte:941
**Impact**: Medium - Low cohesion inside the view layer and limited reuse.
**Recommendation**: Split into view components and move behavior into stores/services.

## Layering Strategy

### Direct UI to backend command coupling
**Observation**: The UI calls Tauri commands directly and embeds command names as strings.
**Location**:
- prompt-launcher/src/routes/+page.svelte:156
- prompt-launcher/src/routes/+page.svelte:283
- prompt-launcher/src/routes/+page.svelte:916
**Impact**: Medium - Changing command names or payloads risks runtime breakage.
**Recommendation**: Define a typed client module that centralizes command names and payloads.
