# Recommendations

## Priority Recommendations

### 1) Add a typed frontend command client
**Observation**: UI calls are scattered string literals.
**Location**:
- `prompt-launcher/src/routes/+page.svelte:195`
- `prompt-launcher/src/routes/+page.svelte:330`
**Impact**: Medium - High risk of contract drift.
**Recommendation**: Create `prompt-launcher/src/lib/tauriClient.ts` and use it from components.

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

### 2) Split UI logic into stores/services + components
**Observation**: `+page.svelte` concentrates UI, state, and IO.
**Location**:
- `prompt-launcher/src/routes/+page.svelte:1`
- `prompt-launcher/src/routes/+page.svelte:1324`
**Impact**: Medium - Reduced cohesion and harder testing.
**Recommendation**: Move query/prompt logic into Svelte stores and keep view files focused on layout.

### 3) Extract backend command handlers
**Observation**: `lib.rs` mixes command implementations with app wiring and platform integrations.
**Location**:
- `prompt-launcher/src-tauri/src/lib.rs:47`
- `prompt-launcher/src-tauri/src/lib.rs:980`
**Impact**: Medium - Increases coupling in the composition root.
**Recommendation**: Create `prompt-launcher/src-tauri/src/commands/` and move command functions there.

### 4) Normalize architecture around prompts
**Observation**: Prompt creation uses a usecase pattern but indexing/search does not.
**Location**:
- `prompt-launcher/src-tauri/src/usecase/create_prompt_file.rs:5`
- `prompt-launcher/src-tauri/src/prompts.rs:19`
**Impact**: Medium - Inconsistent layering makes evolution harder.
**Recommendation**: Introduce a `PromptsService` interface or fold the usecase into a simpler module strategy.

## Secondary Recommendations

### 5) Centralize event name constants
**Observation**: Event strings are hard-coded in multiple files.
**Location**:
- `prompt-launcher/src-tauri/src/lib.rs:45`
- `prompt-launcher/src/routes/+page.svelte:185`
**Impact**: Low - Mismatches can cause silent failures.
**Recommendation**: Define event names in a backend constants module and mirror them in the frontend client.
