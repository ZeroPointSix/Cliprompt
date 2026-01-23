# Architectural Violations

### View layer contains domain logic and IO
**Observation**: The main Svelte page handles state, data shaping, command invocations, and view rendering together.
**Location**:
- prompt-launcher/src/routes/+page.svelte:42
- prompt-launcher/src/routes/+page.svelte:415
- prompt-launcher/src/routes/+page.svelte:941
**Impact**: Medium - Blurs boundaries between UI and domain logic and complicates reuse.
**Recommendation**: Move domain logic and command calls into stores/services, keep the page focused on rendering.

### Composition root doubles as feature implementation
**Observation**: lib.rs implements commands, window/tray management, filesystem watching, and state updates in one module.
**Location**:
- prompt-launcher/src-tauri/src/lib.rs:25
- prompt-launcher/src-tauri/src/lib.rs:402
- prompt-launcher/src-tauri/src/lib.rs:472
**Impact**: Medium - Increases coupling and makes testing or refactoring risky.
**Recommendation**: Extract command handlers and runtime services into separate modules.

### Stringly-typed command and event contracts
**Observation**: Command names and event names are repeated as raw strings across layers.
**Location**:
- prompt-launcher/src/routes/+page.svelte:156
- prompt-launcher/src/routes/+page.svelte:169
- prompt-launcher/src-tauri/src/lib.rs:472
**Impact**: Medium - Changes can silently break behavior at runtime.
**Recommendation**: Centralize command/event names in a typed frontend client and a backend constants module.

### No explicit domain service boundary for prompts
**Observation**: Command handlers call prompt indexing/search functions directly without a service layer.
**Location**:
- prompt-launcher/src-tauri/src/lib.rs:60
- prompt-launcher/src-tauri/src/prompts.rs:17
**Impact**: Low - Fine for MVP, but layering will be harder to enforce later.
**Recommendation**: Introduce a `prompts_service` module to encapsulate indexing, searching, and updates.
