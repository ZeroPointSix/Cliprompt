<script lang="ts">
  import { onDestroy, onMount, tick } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { register, unregister } from "@tauri-apps/plugin-global-shortcut";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";
  import { openPath } from "@tauri-apps/plugin-opener";

  type PromptEntry = {
    id: string;
    title: string;
    body: string;
    preview: string;
    tags: string[];
    path: string;
  };

  type AppConfig = {
    prompts_dir: string;
    auto_paste: boolean;
    hotkey: string;
    auto_start: boolean;
    favorites: string[];
  };

  const appWindow = getCurrentWindow();
  const maxResults = 8;

  let searchInput: HTMLInputElement | null = null;
  let query = $state<string>("");
  let config = $state<AppConfig>({
    prompts_dir: "",
    auto_paste: true,
    hotkey: "Alt+Space",
    auto_start: false,
    favorites: []
  });
  let selectedIndex = $state<number>(0);
  let status = $state<string>("");
  let hotkeyDraft = $state<string>("");
  let hotkeyError = $state<string>("");
  let settingsError = $state<string>("");
  let showSettings = $state<boolean>(false);
  let showFavorites = $state<boolean>(false);
  let currentHotkey = "";

  let filtered = $state<PromptEntry[]>([]);
  let activePrompt = $state<PromptEntry | null>(null);
  let favoritesList = $state<{ prompt: PromptEntry; index: number }[]>([]);
  let regularList = $state<{ prompt: PromptEntry; index: number }[]>([]);
  let searchTimer: ReturnType<typeof setTimeout> | null = null;
  let searchToken = 0;

  let unlistenPrompts: UnlistenFn | null = null;
  let unlistenFocus: UnlistenFn | null = null;

  $effect(() => {
    const favorites: { prompt: PromptEntry; index: number }[] = [];
    const regular: { prompt: PromptEntry; index: number }[] = [];
    filtered.forEach((prompt, index) => {
      if (isFavorite(prompt)) {
        favorites.push({ prompt, index });
      } else {
        regular.push({ prompt, index });
      }
    });
    favoritesList = favorites;
    regularList = regular;
    activePrompt = filtered[selectedIndex] ?? null;
  });

  onMount(async () => {
    config = await invoke<AppConfig>("get_config");
    hotkeyDraft = config.hotkey;
    await registerHotkey(config.hotkey);
    await refreshResults();

    unlistenPrompts = await listen("prompts-updated", () => {
      selectedIndex = 0;
      void refreshResults();
    });

    unlistenFocus = await appWindow.onFocusChanged(({ payload }) => {
      if (!payload) {
        appWindow.hide();
      } else {
        focusSearch();
      }
    });
  });

  onDestroy(async () => {
    if (unlistenPrompts) {
      unlistenPrompts();
    }
    if (unlistenFocus) {
      unlistenFocus();
    }
    if (currentHotkey) {
      await unregister(currentHotkey).catch(() => {});
    }
  });

  async function registerHotkey(hotkey: string) {
    hotkeyError = "";
    if (currentHotkey) {
      await unregister(currentHotkey).catch(() => {});
    }
    try {
      await register(hotkey, toggleWindow);
      currentHotkey = hotkey;
    } catch (error) {
      hotkeyError = `Hotkey failed: ${error}`;
    }
  }

  async function toggleWindow() {
    const visible = await appWindow.isVisible();
    if (visible) {
      await appWindow.hide();
      return;
    }
    await invoke("capture_active_window").catch(() => {});
    await appWindow.show();
    await appWindow.setFocus();
    await tick();
    focusSearch();
  }

  function focusSearch() {
    if (searchInput) {
      searchInput.focus();
      searchInput.select();
    }
  }

  async function chooseFolder() {
    const result = await openDialog({
      directory: true,
      multiple: false,
      title: "Select prompts folder"
    });
    if (!result) {
      return;
    }
    const dir = Array.isArray(result) ? result[0] : result;
    await invoke("set_prompts_dir", { path: dir });
    config = { ...config, prompts_dir: dir };
    status = "Folder updated";
    await refreshResults();
  }

  async function applyHotkey() {
    if (!hotkeyDraft.trim()) {
      hotkeyError = "Hotkey cannot be empty";
      return;
    }
    await registerHotkey(hotkeyDraft);
    if (!hotkeyError) {
      await invoke("set_hotkey", { hotkey: hotkeyDraft });
      config = { ...config, hotkey: hotkeyDraft };
      status = "Hotkey saved";
      settingsError = "";
    }
  }

  async function toggleAutoPaste() {
    const nextValue = !config.auto_paste;
    config = { ...config, auto_paste: nextValue };
    await invoke("set_auto_paste", { autoPaste: nextValue });
  }

  async function toggleAutoStart() {
    const nextValue = !config.auto_start;
    config = { ...config, auto_start: nextValue };
    try {
      await invoke("set_auto_start", { autoStart: nextValue });
      settingsError = "";
    } catch (error) {
      config = { ...config, auto_start: !nextValue };
      settingsError = `Auto-start failed: ${error}`;
    }
  }

  async function toggleFavorite(prompt: PromptEntry | null | undefined) {
    if (!prompt) {
      return;
    }
    const favorites = await invoke<string[]>("toggle_favorite", { id: prompt.id });
    config = { ...config, favorites: favorites ?? [] };
    void refreshResults();
  }

  function toggleSelectedFavorite() {
    if (!activePrompt) {
      return;
    }
    void toggleFavorite(activePrompt);
  }

  function isFavorite(prompt: PromptEntry) {
    return config.favorites.includes(prompt.id);
  }

  function toggleFavoritesFilter() {
    showFavorites = !showFavorites;
    selectedIndex = 0;
    void refreshResults();
  }

  async function usePrompt(prompt: PromptEntry | null | undefined) {
    if (!prompt) {
      return;
    }
    await appWindow.hide();
    await writeText(prompt.body);
    await invoke("focus_last_window", { autoPaste: config.auto_paste });
    query = "";
    selectedIndex = 0;
    void refreshResults();
  }

  async function copyPrompt(prompt: PromptEntry | null | undefined) {
    if (!prompt) {
      return;
    }
    await writeText(prompt.body);
    status = "Copied to clipboard";
  }

  async function copyTitle(prompt: PromptEntry | null | undefined) {
    if (!prompt) {
      return;
    }
    await writeText(prompt.title);
    status = "Title copied";
  }

  async function openPrompt(prompt: PromptEntry | null | undefined) {
    if (!prompt) {
      return;
    }
    await openPath(prompt.path);
  }

  async function openFolder() {
    if (!config.prompts_dir) {
      return;
    }
    await openPath(config.prompts_dir);
  }

  function toggleSettings() {
    showSettings = !showSettings;
  }

  function onSearchInput(event: Event) {
    const target = event.target as HTMLInputElement | null;
    query = target?.value ?? "";
    selectedIndex = 0;
    scheduleSearch();
  }

  function onSearchKeydown(event: KeyboardEvent) {
    if (
      event.ctrlKey &&
      event.shiftKey &&
      event.key.toLowerCase() === "f"
    ) {
      event.preventDefault();
      toggleSelectedFavorite();
      return;
    }
    if (filtered.length === 0) {
      if (event.key === "Escape") {
        event.preventDefault();
        appWindow.hide();
      }
      return;
    }
    if (event.key === "ArrowDown") {
      event.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, filtered.length - 1);
      return;
    }
    if (event.key === "ArrowUp") {
      event.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
      return;
    }
    if (event.key === "Enter") {
      event.preventDefault();
      usePrompt(filtered[selectedIndex]);
      return;
    }
    if (event.key === "Escape") {
      event.preventDefault();
      appWindow.hide();
    }
  }

  function scheduleSearch() {
    if (searchTimer) {
      clearTimeout(searchTimer);
    }
    searchTimer = setTimeout(() => {
      void refreshResults();
    }, 40);
  }

  async function refreshResults() {
    const token = ++searchToken;
    const results = await invoke<PromptEntry[]>("search_prompts", {
      query,
      limit: maxResults,
      favoritesOnly: showFavorites
    });
    if (token !== searchToken) {
      return;
    }
    filtered = results ?? [];
    if (selectedIndex >= filtered.length) {
      selectedIndex = 0;
    }
    activePrompt = filtered[selectedIndex] ?? null;
  }
</script>

<main class="shell">
  <section class="panel">
    <header class="panel-header">
      <div class="title">
        <span class="name">Prompt Launcher</span>
        <span class="meta">Hotkey: {config.hotkey}</span>
        <span class="meta">Folder: {config.prompts_dir || "Not set"}</span>
        <span class="meta">Favorites: {config.favorites.length}</span>
      </div>
      <div class="actions">
        <button class="ghost" type="button" onclick={chooseFolder}>
          Change Folder
        </button>
        <button class="ghost" type="button" onclick={openFolder}>
          Open Folder
        </button>
        <button class="ghost" class:active={showFavorites} type="button" onclick={toggleFavoritesFilter}>
          {showFavorites ? "All prompts" : `Favorites (${config.favorites.length})`}
        </button>
        <label class="toggle">
          <input type="checkbox" checked={config.auto_paste} onchange={toggleAutoPaste} />
          <span>Auto paste</span>
        </label>
        <label class="toggle">
          <input type="checkbox" checked={config.auto_start} onchange={toggleAutoStart} />
          <span>Auto start</span>
        </label>
      </div>
    </header>

    <div class="search">
      <span class="search-icon">/</span>
      <input
        bind:this={searchInput}
        class="search-input"
        placeholder="Search prompts, use #tag"
        value={query}
        oninput={onSearchInput}
        onkeydown={onSearchKeydown}
      />
      <span class="count">{filtered.length}</span>
    </div>

    <div class="content">
      <div class="list">
        {#if filtered.length === 0}
          <div class="empty">
            <span>No matches yet</span>
            <span class="hint">Try a tag like #sql</span>
          </div>
        {:else if !showFavorites && favoritesList.length > 0}
          <div class="section-label">Favorites</div>
          {#each favoritesList as item (item.prompt.id)}
            <div
              class:selected={item.index === selectedIndex}
              class="row"
              style={`--i: ${item.index}`}
              role="button"
              tabindex="0"
              onclick={() => (selectedIndex = item.index)}
              ondblclick={() => usePrompt(item.prompt)}
              onkeydown={(event) => {
                if (event.key === "Enter" || event.key === " ") {
                  event.preventDefault();
                  usePrompt(item.prompt);
                }
              }}
              oncontextmenu={(event) => {
                event.preventDefault();
                openPrompt(item.prompt);
              }}
            >
              <div class="row-title">
                <div class="row-heading">
                  <span>{item.prompt.title}</span>
                  {#if item.prompt.tags?.length}
                    <div class="tags">
                      {#each item.prompt.tags as tag}
                        <span class="tag">#{tag}</span>
                      {/each}
                    </div>
                  {/if}
                </div>
                <div class="row-actions">
                  <button
                    class="fav-toggle"
                    class:active={isFavorite(item.prompt)}
                    type="button"
                    aria-pressed={isFavorite(item.prompt)}
                    onclick={(event) => {
                      event.stopPropagation();
                      toggleFavorite(item.prompt);
                    }}
                  >
                    Fav
                  </button>
                </div>
              </div>
              <div class="row-preview">{item.prompt.preview}</div>
            </div>
          {/each}
          <div class="section-label">All prompts</div>
          {#each regularList as item (item.prompt.id)}
            <div
              class:selected={item.index === selectedIndex}
              class="row"
              style={`--i: ${item.index}`}
              role="button"
              tabindex="0"
              onclick={() => (selectedIndex = item.index)}
              ondblclick={() => usePrompt(item.prompt)}
              onkeydown={(event) => {
                if (event.key === "Enter" || event.key === " ") {
                  event.preventDefault();
                  usePrompt(item.prompt);
                }
              }}
              oncontextmenu={(event) => {
                event.preventDefault();
                openPrompt(item.prompt);
              }}
            >
              <div class="row-title">
                <div class="row-heading">
                  <span>{item.prompt.title}</span>
                  {#if item.prompt.tags?.length}
                    <div class="tags">
                      {#each item.prompt.tags as tag}
                        <span class="tag">#{tag}</span>
                      {/each}
                    </div>
                  {/if}
                </div>
                <div class="row-actions">
                  <button
                    class="fav-toggle"
                    class:active={isFavorite(item.prompt)}
                    type="button"
                    aria-pressed={isFavorite(item.prompt)}
                    onclick={(event) => {
                      event.stopPropagation();
                      toggleFavorite(item.prompt);
                    }}
                  >
                    Fav
                  </button>
                </div>
              </div>
              <div class="row-preview">{item.prompt.preview}</div>
            </div>
          {/each}
        {:else}
          {#each filtered as prompt, index (prompt.id)}
            <div
              class:selected={index === selectedIndex}
              class="row"
              style={`--i: ${index}`}
              role="button"
              tabindex="0"
              onclick={() => (selectedIndex = index)}
              ondblclick={() => usePrompt(prompt)}
              onkeydown={(event) => {
                if (event.key === "Enter" || event.key === " ") {
                  event.preventDefault();
                  usePrompt(prompt);
                }
              }}
              oncontextmenu={(event) => {
                event.preventDefault();
                openPrompt(prompt);
              }}
            >
              <div class="row-title">
                <div class="row-heading">
                  <span>{prompt.title}</span>
                  {#if prompt.tags?.length}
                    <div class="tags">
                      {#each prompt.tags as tag}
                        <span class="tag">#{tag}</span>
                      {/each}
                    </div>
                  {/if}
                </div>
                <div class="row-actions">
                  <button
                    class="fav-toggle"
                    class:active={isFavorite(prompt)}
                    type="button"
                    aria-pressed={isFavorite(prompt)}
                    onclick={(event) => {
                      event.stopPropagation();
                      toggleFavorite(prompt);
                    }}
                  >
                    Fav
                  </button>
                </div>
              </div>
              <div class="row-preview">{prompt.preview}</div>
            </div>
          {/each}
        {/if}
      </div>

      <aside class="preview">
        {#if activePrompt}
          <div class="preview-title">{activePrompt.title}</div>
          <div class="preview-body">{activePrompt.body}</div>
          <div class="preview-actions">
            <button type="button" onclick={() => usePrompt(activePrompt)}>
              Paste
            </button>
            <button class="ghost" type="button" onclick={() => copyPrompt(activePrompt)}>
              Copy
            </button>
            <button class="ghost" type="button" onclick={() => copyTitle(activePrompt)}>
              Copy Title
            </button>
            <button class="ghost" type="button" onclick={() => openPrompt(activePrompt)}>
              Open File
            </button>
          </div>
        {:else}
          <div class="preview-empty">
            <span>Pick a prompt to preview</span>
          </div>
        {/if}
      </aside>
    </div>

    <footer class="panel-footer">
      <div class="hotkey">
        <span>Change hotkey</span>
        <input class="hotkey-input" bind:value={hotkeyDraft} />
        <button type="button" onclick={applyHotkey}>Apply</button>
        <button class="ghost" type="button" onclick={toggleSettings}>
          {showSettings ? "Hide settings" : "Settings"}
        </button>
      </div>
      <div class="status">
        {#if hotkeyError}
          <span class="error">{hotkeyError}</span>
        {:else if settingsError}
          <span class="error">{settingsError}</span>
        {:else if status}
          <span>{status}</span>
        {:else}
          <span>Enter to paste, right click to open, Ctrl+Shift+F to favorite</span>
        {/if}
      </div>
    </footer>

    {#if showSettings}
      <section class="settings">
        <div class="settings-title">Settings info</div>
        <div class="settings-row">
          <span class="settings-label">Hotkey format</span>
          <span>Use modifiers like Ctrl, Alt, Shift. Example: Ctrl+Shift+P</span>
        </div>
        <div class="settings-row">
          <span class="settings-label">Current hotkey</span>
          <span>{config.hotkey}</span>
        </div>
        <div class="settings-row">
          <span class="settings-label">Auto start</span>
          <span>{config.auto_start ? "Enabled" : "Disabled"}</span>
        </div>
        <div class="settings-row">
          <span class="settings-label">Favorites</span>
          <span>Toggle with Ctrl+Shift+F</span>
        </div>
        {#if hotkeyError}
          <div class="settings-row error">{hotkeyError}</div>
        {:else if settingsError}
          <div class="settings-row error">{settingsError}</div>
        {/if}
      </section>
    {/if}
  </section>
</main>

<style>
:global(body) {
  margin: 0;
  background: transparent;
  color: #1e2320;
  font-family: "Bahnschrift", "Segoe UI", sans-serif;
  letter-spacing: 0.01em;
}

:global(*),
:global(*::before),
:global(*::after) {
  box-sizing: border-box;
}

.shell {
  min-height: 100vh;
  display: grid;
  place-items: center;
  padding: 24px;
  background: radial-gradient(circle at 10% 10%, rgba(255, 255, 255, 0.55), rgba(255, 255, 255, 0)) ,
    radial-gradient(circle at 90% 0%, rgba(186, 227, 218, 0.35), rgba(186, 227, 218, 0)),
    linear-gradient(135deg, rgba(246, 238, 228, 0.95), rgba(230, 242, 236, 0.96));
}

.panel {
  width: min(820px, 92vw);
  background: rgba(255, 255, 255, 0.76);
  border-radius: 22px;
  padding: 24px;
  box-shadow:
    0 30px 80px rgba(36, 57, 46, 0.18),
    inset 0 1px 0 rgba(255, 255, 255, 0.6);
  backdrop-filter: blur(18px);
  animation: rise 0.35s ease;
  position: relative;
  overflow: hidden;
}

.panel::before {
  content: "";
  position: absolute;
  inset: -120px auto auto -120px;
  width: 260px;
  height: 260px;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(175, 224, 201, 0.45), rgba(175, 224, 201, 0));
}

.panel::after {
  content: "";
  position: absolute;
  inset: auto -80px -120px auto;
  width: 240px;
  height: 240px;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(224, 197, 150, 0.4), rgba(224, 197, 150, 0));
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  position: relative;
  z-index: 1;
}

.title {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.name {
  font-size: 20px;
  font-weight: 700;
}

.meta {
  font-size: 12px;
  color: #607063;
}

.actions {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 12px;
}

.ghost {
  background: transparent;
  border: 1px solid rgba(87, 107, 95, 0.4);
  color: #375046;
}

.ghost.active {
  background: rgba(221, 243, 232, 0.75);
  border-color: rgba(61, 108, 90, 0.6);
  color: #2d6a57;
}

.toggle {
  display: flex;
  align-items: center;
  gap: 6px;
}

.toggle input {
  accent-color: #2c6958;
}

.search {
  margin-top: 18px;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 14px;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.7);
  border: 1px solid rgba(134, 157, 140, 0.3);
  position: relative;
  z-index: 1;
}

.search-icon {
  font-weight: 700;
  color: #7e8f82;
}

.search-input {
  flex: 1;
  border: none;
  background: transparent;
  font-size: 16px;
  outline: none;
}

.count {
  font-size: 12px;
  color: #5f6f63;
}

.content {
  display: grid;
  grid-template-columns: 1.15fr 0.85fr;
  gap: 18px;
  margin-top: 18px;
  position: relative;
  z-index: 1;
}

.list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 240px;
  overflow-y: auto;
  padding-right: 6px;
}

.row {
  padding: 12px;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.5);
  border: 1px solid transparent;
  cursor: pointer;
  transition: 0.2s ease;
  animation: fadeUp 0.35s ease both;
  animation-delay: calc(var(--i) * 40ms);
}

.row.selected {
  border-color: rgba(61, 108, 90, 0.6);
  background: rgba(221, 243, 232, 0.75);
}

.row-title {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  font-weight: 600;
  font-size: 14px;
}

.row-heading {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.row-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.section-label {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: #708075;
  padding: 4px 8px 2px;
}

.row-preview {
  margin-top: 6px;
  font-size: 12px;
  color: #6c7a70;
}

.tags {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}

.tag {
  font-size: 10px;
  background: rgba(52, 92, 78, 0.12);
  color: #3e6b5b;
  padding: 2px 6px;
  border-radius: 999px;
}

.fav-toggle {
  border: 1px solid rgba(87, 107, 95, 0.4);
  background: transparent;
  color: #375046;
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 999px;
  cursor: pointer;
}

.fav-toggle.active {
  background: rgba(221, 243, 232, 0.9);
  border-color: rgba(61, 108, 90, 0.6);
  color: #2d6a57;
}

.fav-toggle:hover {
  transform: none;
  box-shadow: none;
}

.preview {
  background: rgba(255, 255, 255, 0.6);
  border-radius: 16px;
  padding: 14px;
  border: 1px solid rgba(134, 157, 140, 0.25);
  display: flex;
  flex-direction: column;
  gap: 10px;
  max-height: 240px;
}

.preview-title {
  font-weight: 700;
  font-size: 14px;
}

.preview-body {
  font-size: 12px;
  color: #4a5b51;
  overflow-y: auto;
  white-space: pre-wrap;
}

.preview-actions {
  margin-top: auto;
  display: flex;
  gap: 10px;
}

.panel-footer {
  margin-top: 18px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  position: relative;
  z-index: 1;
}

.settings {
  margin-top: 14px;
  padding: 12px 14px;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.65);
  border: 1px solid rgba(134, 157, 140, 0.25);
  position: relative;
  z-index: 1;
}

.settings-title {
  font-weight: 600;
  font-size: 12px;
  margin-bottom: 8px;
}

.settings-row {
  display: flex;
  gap: 12px;
  font-size: 12px;
  color: #5b6a60;
  margin-bottom: 6px;
}

.settings-row:last-child {
  margin-bottom: 0;
}

.settings-label {
  min-width: 110px;
  color: #3f5146;
  font-weight: 600;
}

.settings-row.error {
  color: #a34f3b;
  font-weight: 600;
}

.hotkey {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
}

.hotkey-input {
  width: 140px;
  border-radius: 8px;
  border: 1px solid rgba(87, 107, 95, 0.4);
  padding: 6px 8px;
  font-size: 12px;
  background: rgba(255, 255, 255, 0.85);
}

.status {
  font-size: 12px;
  color: #5b6a60;
}

.error {
  color: #a34f3b;
}

.panel button:not(.row) {
  border: none;
  padding: 8px 14px;
  border-radius: 10px;
  background: #2d6a57;
  color: #f5f3ed;
  font-size: 12px;
  cursor: pointer;
  transition: 0.2s ease;
}

.panel button:not(.row):hover {
  transform: translateY(-1px);
  box-shadow: 0 8px 18px rgba(45, 106, 87, 0.2);
}

.empty,
.preview-empty {
  padding: 16px;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.6);
  border: 1px dashed rgba(134, 157, 140, 0.3);
  text-align: center;
  font-size: 13px;
  color: #6c7a70;
}

.hint {
  display: block;
  margin-top: 6px;
  font-size: 12px;
}

@keyframes rise {
  from {
    opacity: 0;
    transform: translateY(8px) scale(0.98);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

@keyframes fadeUp {
  from {
    opacity: 0;
    transform: translateY(8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@media (max-width: 720px) {
  .content {
    grid-template-columns: 1fr;
  }

  .panel-footer {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
