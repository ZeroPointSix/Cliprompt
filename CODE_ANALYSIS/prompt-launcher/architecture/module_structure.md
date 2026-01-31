# Module Structure

## Top-Level Layout

### Monorepo-style layout (frontend + tauri backend)
**Observation**: UI and native backend live under `prompt-launcher/`, with SvelteKit in `src/` and Rust/Tauri in `src-tauri/`.
**Location**:
- `prompt-launcher/src/app.html:1`
- `prompt-launcher/src-tauri/Cargo.toml:1`
**Impact**: Low - Clear separation of technology stacks.
**Recommendation**: Add a short architecture summary in docs to explain cross-boundary contracts.

## Backend Modules (prompt-launcher/src-tauri/src)

### Composition root + feature modules
**Observation**: `lib.rs` wires app setup and invokes modules for config, prompts, lifecycle, and OS integration.
**Location**:
- `prompt-launcher/src-tauri/src/lib.rs:47`
- `prompt-launcher/src-tauri/src/lib.rs:932`
**Impact**: Medium - `lib.rs` is a change hotspot.
**Recommendation**: Split command handlers into a `commands` module to reduce churn in `lib.rs`.

### Domain/usecase/infrastructure islands
**Observation**: A small clean-architecture slice exists for file creation, but other prompt flows bypass it.
**Location**:
- `prompt-launcher/src-tauri/src/domain/prompt_filename.rs:26`
- `prompt-launcher/src-tauri/src/usecase/create_prompt_file.rs:5`
- `prompt-launcher/src-tauri/src/prompts.rs:19`
**Impact**: Medium - Layering is inconsistent across features.
**Recommendation**: Either expand the pattern across prompt operations or simplify by removing unused layers.

## Frontend Modules (prompt-launcher/src/routes)

### Single-page UI module
**Observation**: UI logic, state, and rendering are concentrated in `+page.svelte`.
**Location**:
- `prompt-launcher/src/routes/+page.svelte:1`
- `prompt-launcher/src/routes/+page.svelte:1324`
**Impact**: Medium - Low cohesion and limited reuse of UI logic.
**Recommendation**: Extract a `src/lib/stores/` or `src/lib/services/` layer and split UI components.

## Layering Strategy

### Direct UI -> backend command coupling
**Observation**: UI calls backend commands directly with string names and payloads.
**Location**:
- `prompt-launcher/src/routes/+page.svelte:195`
- `prompt-launcher/src/routes/+page.svelte:330`
- `prompt-launcher/src-tauri/src/lib.rs:980`
**Impact**: Medium - Contract changes are runtime-only failures.
**Recommendation**: Introduce a typed command client module on the frontend.
