# Recommendations

## Priority Recommendations

### 1) Create a typed frontend command client
**Observation**: Commands are invoked by string literals in the Svelte page.
**Location**:
- prompt-launcher/src/routes/+page.svelte:156
- prompt-launcher/src/routes/+page.svelte:916
**Impact**: Medium - Renaming commands risks runtime errors.
**Recommendation**: Add a `src/lib/tauriClient.ts` that centralizes command names and payloads.

Example:
```ts
import { invoke } from "@tauri-apps/api/core";

export const tauriClient = {
  getConfig: () => invoke<AppConfig>("get_config"),
  listPrompts: () => invoke<PromptEntry[]>("list_prompts"),
  searchPrompts: (query: string, limit: number, favoritesOnly: boolean) =>
    invoke<PromptEntry[]>("search_prompts", { query, limit, favoritesOnly }),
  setPromptsDir: (path: string) => invoke("set_prompts_dir", { path }),
};
```

### 2) Split +page.svelte into components and stores
**Observation**: UI, state, and IO logic are all in one file.
**Location**:
- prompt-launcher/src/routes/+page.svelte:42
- prompt-launcher/src/routes/+page.svelte:941
**Impact**: Medium - Reduced cohesion and limited reuse.
**Recommendation**: Extract a store for prompts and a separate component for settings/results lists.

### 3) Introduce backend command modules
**Observation**: lib.rs is both composition root and command implementation.
**Location**:
- prompt-launcher/src-tauri/src/lib.rs:25
- prompt-launcher/src-tauri/src/lib.rs:472
**Impact**: Medium - Increases coupling as features grow.
**Recommendation**: Move command functions into `src-tauri/src/commands/` and keep lib.rs for wiring.

### 4) Centralize event names and command names
**Observation**: Event names like `prompts-updated` are defined as string literals in multiple files.
**Location**:
- prompt-launcher/src-tauri/src/lib.rs:402
- prompt-launcher/src/routes/+page.svelte:169
**Impact**: Medium - High risk of contract drift.
**Recommendation**: Define constants for event names in the backend and mirror them in the frontend client.
