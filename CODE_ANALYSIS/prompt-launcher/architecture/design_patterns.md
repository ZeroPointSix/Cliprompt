# Design Patterns

## Architectural Patterns

### Tauri desktop + SvelteKit SPA
**Observation**: The frontend is built as a SvelteKit SPA packaged inside a Tauri desktop app.
**Location**:
- prompt-launcher/svelte.config.js:1
- prompt-launcher/svelte.config.js:10
- prompt-launcher/src/routes/+layout.js:5
**Impact**: Low - This is a common, suitable pattern for local desktop utilities.
**Recommendation**: Document the boundary between UI and native services and keep the contract stable as the app grows.

### Command/RPC pattern via Tauri invoke
**Observation**: UI issues commands by string name and backend exposes them via Tauri command handlers.
**Location**:
- prompt-launcher/src/routes/+page.svelte:156
- prompt-launcher/src/routes/+page.svelte:916
- prompt-launcher/src-tauri/src/lib.rs:472
**Impact**: Medium - Stringly-typed commands can drift without compile-time checks.
**Recommendation**: Introduce a typed frontend client module that wraps all command names and payload shapes.

### Observer pattern for prompt updates
**Observation**: Backend watches the prompts directory and emits an event that the UI subscribes to.
**Location**:
- prompt-launcher/src-tauri/src/lib.rs:402
- prompt-launcher/src/routes/+page.svelte:169
**Impact**: Low - Provides reactive refresh behavior without polling.
**Recommendation**: Centralize event names as constants to prevent mismatches across layers.

## Design Pattern Usage

### Shared state (singleton-like) in backend
**Observation**: AppState is a shared, process-wide state object stored in Tauri state.
**Location**:
- prompt-launcher/src-tauri/src/lib.rs:25
**Impact**: Medium - Shared mutable state can become a coordination bottleneck as features grow.
**Recommendation**: Split state into smaller domain services and expose them via command modules.

## Anti-Patterns / Smells

### God component in the Svelte UI
**Observation**: The main page contains state, side effects, data formatting, and rendering in one file.
**Location**:
- prompt-launcher/src/routes/+page.svelte:42
- prompt-launcher/src/routes/+page.svelte:156
- prompt-launcher/src/routes/+page.svelte:941
**Impact**: Medium - Reduced cohesion and harder testability or reuse of logic.
**Recommendation**: Extract stores/services (state + IO) and split UI into smaller Svelte components.

### God module in backend composition root
**Observation**: lib.rs includes app setup, window/tray control, file watching, and command handlers.
**Location**:
- prompt-launcher/src-tauri/src/lib.rs:25
- prompt-launcher/src-tauri/src/lib.rs:402
- prompt-launcher/src-tauri/src/lib.rs:472
**Impact**: Medium - Harder to evolve or test features in isolation.
**Recommendation**: Move commands to a dedicated module and keep lib.rs focused on wiring.
