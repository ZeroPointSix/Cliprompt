# Architectural Violations

### UI layer owns domain logic + IO orchestration
**Observation**: `+page.svelte` handles search state, tag logic, and direct command calls alongside rendering.
**Location**:
- `prompt-launcher/src/routes/+page.svelte:1`
- `prompt-launcher/src/routes/+page.svelte:195`
- `prompt-launcher/src/routes/+page.svelte:1324`
**Impact**: Medium - Blurs UI/domain boundaries and limits reuse.
**Recommendation**: Move command calls and stateful logic into Svelte stores/services.

### Composition root doubles as feature implementation
**Observation**: `lib.rs` contains command handlers, filesystem logic, window/tray control, and watcher setup.
**Location**:
- `prompt-launcher/src-tauri/src/lib.rs:47`
- `prompt-launcher/src-tauri/src/lib.rs:559`
- `prompt-launcher/src-tauri/src/lib.rs:980`
**Impact**: Medium - Makes the composition root a high-risk change hotspot.
**Recommendation**: Extract command handlers into `commands/` and move IO/watcher logic into services.

### Layering inconsistency across prompt operations
**Observation**: Only prompt creation uses a domain/usecase/repository stack; indexing/search/update bypass it.
**Location**:
- `prompt-launcher/src-tauri/src/usecase/create_prompt_file.rs:5`
- `prompt-launcher/src-tauri/src/prompts.rs:19`
- `prompt-launcher/src-tauri/src/lib.rs:117`
**Impact**: Medium - Mixed layering reduces architectural clarity.
**Recommendation**: Either extend the usecase pattern for prompt operations or simplify by collapsing layers.

### Stringly-typed command/event contracts
**Observation**: Command names and event names are repeated as raw strings across UI and backend.
**Location**:
- `prompt-launcher/src/routes/+page.svelte:195`
- `prompt-launcher/src/routes/+page.svelte:235`
- `prompt-launcher/src-tauri/src/lib.rs:45`
**Impact**: Medium - Runtime-only failures if a name changes.
**Recommendation**: Centralize command and event names in shared constants/clients.
