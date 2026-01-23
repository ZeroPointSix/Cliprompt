# Dependency Graph

## High-Level Flow

UI (Svelte) -> Tauri API -> Rust command handlers -> OS/FS

## Frontend -> Backend

### UI invokes Tauri commands
**Observation**: The frontend depends on command names and payloads implemented in Rust.
**Location**:
- prompt-launcher/src/routes/+page.svelte:156
- prompt-launcher/src/routes/+page.svelte:916
- prompt-launcher/src-tauri/src/lib.rs:472
**Impact**: Medium - Contract changes require synchronized updates.
**Recommendation**: Centralize command definitions in a shared client module on the frontend.

## Backend Internal Dependencies

### lib.rs depends on feature modules
**Observation**: The application composition root imports and calls config, prompts, win, and autostart modules.
**Location**:
- prompt-launcher/src-tauri/src/lib.rs:1
- prompt-launcher/src-tauri/src/lib.rs:81
**Impact**: Low - Acceptable for a small codebase but increases coordination cost as it grows.
**Recommendation**: Split command handlers into a module that depends on these services rather than all logic living in lib.rs.

### prompts.rs depends on filesystem and WalkDir
**Observation**: Prompt indexing reads prompt files directly from disk with directory traversal.
**Location**:
- prompt-launcher/src-tauri/src/prompts.rs:3
- prompt-launcher/src-tauri/src/prompts.rs:5
**Impact**: Low - Direct and efficient for a local tool.
**Recommendation**: Consider abstracting FS access if you later add tests or alternate storage.

## OS Integrations

### Windows-specific modules
**Observation**: win.rs calls Windows APIs and autostart.rs edits the registry.
**Location**:
- prompt-launcher/src-tauri/src/win.rs:12
- prompt-launcher/src-tauri/src/autostart.rs:9
**Impact**: Medium - OS-specific code should remain isolated to keep portability manageable.
**Recommendation**: Keep platform code in separate modules and guard with feature flags if adding other platforms.
