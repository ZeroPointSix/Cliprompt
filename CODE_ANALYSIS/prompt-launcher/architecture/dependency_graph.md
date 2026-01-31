# Dependency Graph

## High-Level Flow

UI (SvelteKit) -> Tauri commands -> Core services (config/prompts) -> OS/FS

## Frontend -> Backend

### UI invokes Tauri commands (string contracts)
**Observation**: The Svelte UI depends on command strings and payload shapes in Rust.
**Location**:
- `prompt-launcher/src/routes/+page.svelte:195`
- `prompt-launcher/src/routes/+page.svelte:330`
- `prompt-launcher/src-tauri/src/lib.rs:980`
**Impact**: Medium - Any rename or payload change breaks at runtime.
**Recommendation**: Centralize command names in a typed client module.

### UI subscribes to backend events
**Observation**: UI listens to `prompts-updated` and `launcher-shown` events emitted by Rust.
**Location**:
- `prompt-launcher/src/routes/+page.svelte:185`
- `prompt-launcher/src/routes/+page.svelte:235`
- `prompt-launcher/src-tauri/src/lib.rs:45`
**Impact**: Low - Event-driven updates are appropriate.
**Recommendation**: Define event names as shared constants.

## Backend Internal Dependencies

### Composition root depends on feature modules
**Observation**: `lib.rs` imports config, prompts, lifecycle, tags_meta, win/autostart, and usecase modules.
**Location**:
- `prompt-launcher/src-tauri/src/lib.rs:1`
- `prompt-launcher/src-tauri/src/lib.rs:36`
**Impact**: Low - Normal for a small app but increases coupling over time.
**Recommendation**: Use a `commands` layer to isolate app wiring from feature logic.

### Clean-architecture slice dependencies
**Observation**: `usecase` depends on `domain`, and `infrastructure` implements `usecase` traits.
**Location**:
- `prompt-launcher/src-tauri/src/usecase/create_prompt_file.rs:3`
- `prompt-launcher/src-tauri/src/infrastructure/fs_prompt_file_repository.rs:4`
**Impact**: Low - Dependency direction aligns with clean architecture.
**Recommendation**: Keep this pattern consistent when adding more IO-related use cases.

### Prompts module depends on tags metadata
**Observation**: `prompts.rs` uses `tags_meta` to resolve tags for prompt files.
**Location**:
- `prompt-launcher/src-tauri/src/prompts.rs:7`
- `prompt-launcher/src-tauri/src/tags_meta.rs:40`
**Impact**: Low - Cohesive behavior within prompt indexing.
**Recommendation**: Consider extracting a `prompts_service` if more tag-related logic appears.

## Coupling/Cycles

**Observation**: No circular dependencies detected in `src-tauri/src` based on module imports.
**Location**:
- `prompt-launcher/src-tauri/src/lib.rs:1`
- `prompt-launcher/src-tauri/src/prompts.rs:7`
- `prompt-launcher/src-tauri/src/usecase/create_prompt_file.rs:3`
**Impact**: Low - Current module graph is acyclic.
**Recommendation**: Keep module imports one-way to preserve acyclicity.
