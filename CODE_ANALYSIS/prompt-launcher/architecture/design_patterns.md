# Design Patterns

## Architectural Patterns

### Tauri desktop + SvelteKit SPA
**Observation**: UI is a SvelteKit SPA bundled into a Tauri desktop shell; backend exposes native/FS capabilities via commands.
**Location**:
- `prompt-launcher/src/routes/+layout.js:1`
- `prompt-launcher/src-tauri/src/lib.rs:932`
**Impact**: Low - Suitable for a local desktop launcher.
**Recommendation**: Keep the UI/native command contract explicit and stable as features grow.

### Command/RPC via Tauri invoke
**Observation**: Frontend calls backend commands by string names; backend exposes them via `invoke_handler`.
**Location**:
- `prompt-launcher/src/routes/+page.svelte:195`
- `prompt-launcher/src/routes/+page.svelte:330`
- `prompt-launcher/src-tauri/src/lib.rs:980`
**Impact**: Medium - Stringly-typed contracts can drift without compile-time checks.
**Recommendation**: Introduce a typed frontend client that centralizes command names/payloads.

### Pub/Sub eventing between backend and UI
**Observation**: Backend emits events when prompts change; UI listens for updates.
**Location**:
- `prompt-launcher/src-tauri/src/lib.rs:830`
- `prompt-launcher/src/routes/+page.svelte:235`
**Impact**: Low - Clean event-driven refresh without polling.
**Recommendation**: Centralize event names as constants to avoid mismatches.

### Clean-architecture slice (Domain -> UseCase -> Infrastructure)
**Observation**: Prompt file creation follows Domain (filename rules) -> UseCase -> Repository implementation.
**Location**:
- `prompt-launcher/src-tauri/src/domain/prompt_filename.rs:26`
- `prompt-launcher/src-tauri/src/usecase/create_prompt_file.rs:5`
- `prompt-launcher/src-tauri/src/infrastructure/fs_prompt_file_repository.rs:6`
**Impact**: Low - Good separation for this feature.
**Recommendation**: Use the same layering for other prompt-related operations if they grow.

### Shared state (singleton-like) via Tauri State
**Observation**: `AppState` is a process-wide mutable state accessed by all commands.
**Location**:
- `prompt-launcher/src-tauri/src/lib.rs:47`
**Impact**: Medium - Central state can become a coordination hotspot as features increase.
**Recommendation**: Split `AppState` into smaller services and inject where needed.

## Anti-patterns / Smells

### God component in UI
**Observation**: The main Svelte page owns state, side effects, data shaping, and rendering.
**Location**:
- `prompt-launcher/src/routes/+page.svelte:1`
- `prompt-launcher/src/routes/+page.svelte:1324`
- `prompt-launcher/src/routes/+page.svelte:1578`
**Impact**: Medium - Low cohesion and difficult testability/reuse.
**Recommendation**: Extract stores/services and split view into smaller components.

### God module in backend composition root
**Observation**: `lib.rs` mixes app wiring with command handlers, IO, window management, and watchers.
**Location**:
- `prompt-launcher/src-tauri/src/lib.rs:47`
- `prompt-launcher/src-tauri/src/lib.rs:980`
**Impact**: Medium - Increases coupling and slows refactoring.
**Recommendation**: Move command handlers into a `commands` module and keep `lib.rs` as a wiring layer.
