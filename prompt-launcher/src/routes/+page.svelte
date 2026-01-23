<script lang="ts">
  import { onDestroy, onMount, tick } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { getVersion } from "@tauri-apps/api/app";
  import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { register, unregister } from "@tauri-apps/plugin-global-shortcut";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";

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
    recent_ids: string[];
    recent_enabled: boolean;
    recent_meta: Record<string, number>;
    top_tags_use_results: boolean;
    top_tags_limit: number;
    show_shortcuts_hint: boolean;
    preview_chars: number;
  };

  type RecentState = {
    recent_ids: string[];
    recent_meta: Record<string, number>;
  };

  const appWindow = getCurrentWindow();
  const maxResults = 8;

  let searchInput = $state<HTMLInputElement | null>(null);
  let query = $state<string>("");
  let appVersion = $state<string>("");
  let config = $state<AppConfig>({
    prompts_dir: "",
    auto_paste: true,
    hotkey: "Alt+Space",
    auto_start: false,
    favorites: [],
    recent_ids: [],
    recent_enabled: true,
    recent_meta: {},
    top_tags_use_results: false,
    top_tags_limit: 8,
    show_shortcuts_hint: true,
    preview_chars: 50
  });
  let selectedIndex = $state<number>(0);
  let status = $state<string>("");
  let hotkeyDraft = $state<string>("");
  let hotkeyError = $state<string>("");
  let settingsError = $state<string>("");
  let showSettings = $state<boolean>(false);
  let showShortcuts = $state<boolean>(false);
  let showFavorites = $state<boolean>(false);
  let showRecent = $state<boolean>(false);
  let currentHotkey = "";
  let selectedIds = $state<Set<string>>(new Set());
  let contextMenu = $state<{ visible: boolean; x: number; y: number }>({
    visible: false,
    x: 0,
    y: 0
  });
  let contextTarget = $state<PromptEntry | null>(null);
  let tagEditorMode = $state<"add" | "remove" | null>(null);
  let tagInput = $state<string>("");
  let removeTagOptions = $state<string[]>([]);
  let removeTagSelection = $state<Set<string>>(new Set());
  let tagSuggestions = $state<string[]>([]);
  let isComposing = false;
  let compositionEndedAt = 0;

  let filtered = $state<PromptEntry[]>([]);
  let allPrompts = $state<PromptEntry[]>([]);
  let allTags = $state<string[]>([]);
  let topTags = $state<{ tag: string; count: number }[]>([]);
  let topTagsScopeBeforeFilter = $state<boolean | null>(null);
  let activePrompt = $state<PromptEntry | null>(null);
  let recentList = $state<{ prompt: PromptEntry; index: number }[]>([]);
  let favoritesList = $state<{ prompt: PromptEntry; index: number }[]>([]);
  let regularList = $state<{ prompt: PromptEntry; index: number }[]>([]);
  let searchTimer: ReturnType<typeof setTimeout> | null = null;
  let searchToken = 0;

  let unlistenPrompts: UnlistenFn | null = null;
  let unlistenFocus: UnlistenFn | null = null;
  let windowClickHandler: ((event: MouseEvent) => void) | null = null;
  let windowJustShown = false;
  let focusLossTimer: ReturnType<typeof setTimeout> | null = null;
  let lastToggleTime = 0;
  let toggleDebounceMs = 300;

  // Derived state to determine if the dropdown area should be visible
  let hasContent = $derived(
    query.length > 0 ||
    showFavorites ||
    showRecent ||
    showSettings ||
    filtered.length > 0
  );

  $effect(() => {
    const recent = buildRecentList(filtered, config.recent_ids);
    const recentIds = new Set(recent.map((item) => item.prompt.id));
    const favorites: { prompt: PromptEntry; index: number }[] = [];
    const regular: { prompt: PromptEntry; index: number }[] = [];
    filtered.forEach((prompt, index) => {
      if (recentIds.has(prompt.id)) {
        return;
      }
      if (isFavorite(prompt)) {
        favorites.push({ prompt, index });
      } else {
        regular.push({ prompt, index });
      }
    });
    recentList = recent;
    favoritesList = favorites;
    regularList = regular;
    activePrompt = filtered[selectedIndex] ?? null;
    const tagSource = getTagSource();
    const tagLimit = config.top_tags_limit > 0 ? config.top_tags_limit : 8;
    topTags = buildTopTags(tagSource, tagLimit);
    const tagSet = new Set<string>();
    allPrompts.forEach((prompt) => {
      prompt.tags?.forEach((tag) => tagSet.add(tag));
    });
    allTags = Array.from(tagSet).sort((a, b) => a.localeCompare(b));
  });

  $effect(() => {
    tagSuggestions = getTagSuggestions(query, allTags);
  });

  // Dynamic window height adjustment based on content
  $effect(() => {
    const inputHeight = 56;
    const itemHeight = 50;
    const footerHeight = 40;
    const settingsHeight = 360;
    const minHeight = inputHeight;
    const maxHeight = 500;

    let targetHeight = minHeight;

    if (hasContent) {
      if (showSettings) {
        // Settings view: input + settings container
        targetHeight = inputHeight + settingsHeight;
      } else if (filtered.length > 0) {
        // Results view: input + (items * height) + footer
        targetHeight = inputHeight + (filtered.length * itemHeight) + footerHeight;
      } else {
        // Empty state: input + empty message + footer
        targetHeight = inputHeight + 100 + footerHeight;
      }
    }

    // Clamp to min/max bounds
    targetHeight = Math.max(minHeight, Math.min(maxHeight, targetHeight));

    console.log(`[Window Resize] hasContent=${hasContent}, filtered.length=${filtered.length}, showSettings=${showSettings}, targetHeight=${targetHeight}`);

    // Apply the new window size
    appWindow.setSize(new LogicalSize(760, targetHeight)).then(() => {
      console.log(`[Window Resize] Successfully resized to ${targetHeight}px`);
    }).catch((error) => {
      console.error(`[Window Resize] Failed to resize:`, error);
    });
  });

  onMount(async () => {
    document.title = "提示词启动器";
    config = await invoke<AppConfig>("get_config");
    try {
      appVersion = await getVersion();
    } catch (error) {
      console.warn("[appVersion] Failed to load version", error);
    }
    hotkeyDraft = config.hotkey;
    allPrompts = await invoke<PromptEntry[]>("list_prompts");
    if (config.show_shortcuts_hint) {
      showShortcuts = true;
      await invoke("set_show_shortcuts_hint", { showShortcutsHint: false });
      config = { ...config, show_shortcuts_hint: false };
    }
    await registerHotkey(config.hotkey);
    await refreshResults();

    windowClickHandler = () => {
      closeContextMenu();
    };
    window.addEventListener("click", windowClickHandler);

    unlistenPrompts = await listen<PromptEntry[]>("prompts-updated", (event) => {
      allPrompts = event.payload ?? [];
      selectedIndex = 0;
      void refreshResults();
    });

    // Re-add focus listener with improved logic
    unlistenFocus = await appWindow.onFocusChanged(({ payload }) => {
      if (!payload) {
        // Window lost focus
        // Don't hide immediately if window was just shown (within 500ms)
        if (windowJustShown) {
          console.log("[Focus] Window just shown, ignoring focus loss");
          return;
        }

        // Add a delay before hiding to work with debounce mechanism
        if (focusLossTimer) {
          clearTimeout(focusLossTimer);
        }
        focusLossTimer = setTimeout(() => {
          console.log("[Focus] Hiding window due to focus loss");
          appWindow.hide();
        }, 200);
      } else {
        // Window gained focus - cancel any pending hide
        if (focusLossTimer) {
          clearTimeout(focusLossTimer);
          focusLossTimer = null;
        }
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
    if (windowClickHandler) {
      window.removeEventListener("click", windowClickHandler);
    }
    if (currentHotkey) {
      await unregister(currentHotkey).catch(() => {});
    }
  });

  async function registerHotkey(hotkey: string) {
    console.log("[registerHotkey] Attempting to register hotkey:", hotkey);
    hotkeyError = "";
    if (currentHotkey) {
      console.log("[registerHotkey] Unregistering previous hotkey:", currentHotkey);
      await unregister(currentHotkey).catch(() => {});
    }
    try {
      await register(hotkey, toggleWindow);
      currentHotkey = hotkey;
      console.log("[registerHotkey] Successfully registered hotkey:", hotkey);
    } catch (error) {
      console.error("[registerHotkey] Failed to register hotkey:", error);
      hotkeyError = `快捷键注册失败：${error}`;
    }
  }

  async function toggleWindow() {
    // Debounce to prevent multiple rapid toggles
    const now = Date.now();
    if (now - lastToggleTime < toggleDebounceMs) {
      console.log("[toggleWindow] Debounced - ignoring rapid toggle");
      return;
    }
    lastToggleTime = now;

    const visible = await appWindow.isVisible();
    console.log("[toggleWindow] Current visible state:", visible);

    if (visible) {
      console.log("[toggleWindow] Hiding window");
      await appWindow.hide();
      return;
    }

    console.log("[toggleWindow] Showing window");
    await invoke("capture_active_window").catch(() => {});
    windowJustShown = true;
    await appWindow.show();
    await appWindow.setFocus();
    await tick();
    focusSearch();
    // Clear the flag after a short delay to allow normal focus loss handling
    setTimeout(() => {
      windowJustShown = false;
    }, 500);
  }

  function focusSearch() {
    if (searchInput) {
      searchInput.focus();
      // Ensure cursor is at the end or text is selected, depending on preference.
      // Selecting all is standard for launchers.
      searchInput.select();
    }
  }

  function formatError(error: unknown) {
    if (typeof error === "string") {
      return error;
    }
    if (error instanceof Error && error.message) {
      return error.message;
    }
    try {
      return JSON.stringify(error);
    } catch {
      return String(error);
    }
  }

  async function openPathWithFallback(path: string) {
    console.info("[openPromptPath] opening:", path);
    try {
      await invoke("open_prompt_path", { path });
      return { ok: true as const };
    } catch (error) {
      const message = formatError(error) || "未知错误";
      console.error("[openPromptPath] open failed:", message);
      return { ok: false as const, message };
    }
  }

  async function chooseFolder() {
    const result = await openDialog({
      directory: true,
      multiple: false,
      title: "选择提示词目录"
    });
    if (!result) {
      return;
    }
    const dir = Array.isArray(result) ? result[0] : result;
    await invoke("set_prompts_dir", { path: dir });
    config = { ...config, prompts_dir: dir };
    status = "目录已更新";
    await refreshResults();
  }

  async function applyHotkey() {
    if (!hotkeyDraft.trim()) {
      hotkeyError = "快捷键不能为空";
      return;
    }
    await registerHotkey(hotkeyDraft);
    if (!hotkeyError) {
      await invoke("set_hotkey", { hotkey: hotkeyDraft });
      config = { ...config, hotkey: hotkeyDraft };
      status = "快捷键已保存";
      settingsError = "";
    }
  }

  function onHotkeyInputKeydown(event: KeyboardEvent) {
    // Prevent default behavior to avoid typing characters
    event.preventDefault();

    // Ignore modifier keys pressed alone
    if (["Control", "Alt", "Shift", "Meta"].includes(event.key)) {
      return;
    }

    // Build the hotkey string
    const modifiers: string[] = [];
    if (event.ctrlKey) modifiers.push("Ctrl");
    if (event.altKey) modifiers.push("Alt");
    if (event.shiftKey) modifiers.push("Shift");
    if (event.metaKey) modifiers.push("Meta");

    // Get the key name
    let key = event.key;

    // Normalize key names for special keys
    if (key === " ") key = "Space";
    else if (key.length === 1) key = key.toUpperCase();

    // Build the hotkey string (e.g., "Alt+Space", "Ctrl+Shift+A")
    const hotkey = modifiers.length > 0
      ? `${modifiers.join("+")}+${key}`
      : key;

    hotkeyDraft = hotkey;
    hotkeyError = "";
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
      settingsError = `自启设置失败：${error}`;
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

  function buildRecentList(prompts: PromptEntry[], recentIds: string[]) {
    const map = new Map(
      prompts.map((prompt, index) => [prompt.id, { prompt, index }])
    );
    const results: { prompt: PromptEntry; index: number }[] = [];
    for (const id of recentIds) {
      const entry = map.get(id);
      if (entry) {
        results.push(entry);
      }
    }
    return results;
  }

  function toggleFavoritesFilter() {
    showFavorites = !showFavorites;
    if (showFavorites) {
      showRecent = false;
      void applyTopTagsScopeForFilter();
    } else if (!showRecent) {
      void restoreTopTagsScopeAfterFilter();
    }
    selectedIndex = 0;
    void refreshResults();
  }

  function toggleRecentFilter() {
    showRecent = !showRecent;
    if (showRecent) {
      showFavorites = false;
      void applyTopTagsScopeForFilter();
    } else if (!showFavorites) {
      void restoreTopTagsScopeAfterFilter();
    }
    selectedIndex = 0;
    void refreshResults();
  }

  async function markRecent(prompt: PromptEntry) {
    if (!config.recent_enabled) {
      return;
    }
    const recentState = await invoke<RecentState>("push_recent", { id: prompt.id });
    config = {
      ...config,
      recent_ids: recentState?.recent_ids ?? [],
      recent_meta: recentState?.recent_meta ?? {}
    };
  }

  async function toggleRecentEnabled() {
    const nextValue = !config.recent_enabled;
    config = { ...config, recent_enabled: nextValue };
    await invoke("set_recent_enabled", { recentEnabled: nextValue });
    if (!nextValue) {
      const recentState = await invoke<RecentState>("clear_recent");
      config = {
        ...config,
        recent_ids: recentState?.recent_ids ?? [],
        recent_meta: recentState?.recent_meta ?? {},
        recent_enabled: nextValue
      };
    }
    void refreshResults();
  }

  async function clearRecent() {
    const recentState = await invoke<RecentState>("clear_recent");
    config = {
      ...config,
      recent_ids: recentState?.recent_ids ?? [],
      recent_meta: recentState?.recent_meta ?? {}
    };
    void refreshResults();
  }

  async function usePrompt(prompt: PromptEntry | null | undefined) {
    if (!prompt) {
      console.log("[usePrompt] No prompt provided");
      return;
    }
    console.log("[usePrompt] Using prompt:", prompt.title);
    try {
      await appWindow.hide();
      console.log("[usePrompt] Window hidden");
      await writeText(prompt.body);
      console.log("[usePrompt] Text written to clipboard");
      await invoke("focus_last_window", { autoPaste: config.auto_paste });
      console.log("[usePrompt] Focused last window");
      await markRecent(prompt);
      console.log("[usePrompt] Marked as recent");
      query = "";
      selectedIndex = 0;
      void refreshResults();
    } catch (error) {
      console.error("[usePrompt] Error:", error);
    }
  }

  async function copyPrompt(prompt: PromptEntry | null | undefined) {
    if (!prompt) {
      return;
    }
    await writeText(prompt.body);
    await markRecent(prompt);
    status = "已复制到剪贴板";
  }

  async function copyTitle(prompt: PromptEntry | null | undefined) {
    if (!prompt) {
      return;
    }
    await writeText(prompt.title);
    await markRecent(prompt);
    status = "标题已复制";
  }

  async function copyPath(prompt: PromptEntry | null | undefined) {
    if (!prompt) {
      return;
    }
    await writeText(prompt.path);
    await markRecent(prompt);
    status = "路径已复制";
  }

  async function copyTags(prompt: PromptEntry | null | undefined) {
    if (!prompt) {
      return;
    }
    if (!prompt.tags?.length) {
      status = "没有可复制的标签";
      return;
    }
    const tagString = prompt.tags.map((tag) => `#${tag}`).join(" ");
    await writeText(tagString);
    await markRecent(prompt);
    status = "标签已复制";
  }

  async function copySnippet(prompt: PromptEntry | null | undefined) {
    if (!prompt) {
      return;
    }
    const snippet = getRowPreview(prompt);
    if (!snippet) {
      status = "没有可复制的片段";
      return;
    }
    await writeText(snippet);
    await markRecent(prompt);
    status = "片段已复制";
  }

  async function openPrompt(prompt: PromptEntry | null | undefined) {
    if (!prompt) {
      return;
    }
    const result = await openPathWithFallback(prompt.path);
    if (!result.ok) {
      status = `打开失败：${result.message}`;
    }
  }

  async function openFolder() {
    if (!config.prompts_dir) {
      return;
    }
    const result = await openPathWithFallback(config.prompts_dir);
    if (!result.ok) {
      status = `打开失败：${result.message}`;
    }
  }

  async function deleteSelectedPrompts() {
    const selected = getSelectedPrompts();
    if (selected.length === 0) {
      status = "请选择项目";
      return;
    }
    const confirmMessage =
      selected.length === 1
        ? `确定删除 "${selected[0].title}" 吗？此操作不可撤销。`
        : `确定删除 ${selected.length} 个文件吗？此操作不可撤销。`;
    if (!window.confirm(confirmMessage)) {
      return;
    }
    const paths = selected.map((prompt) => prompt.path);
    try {
      const updated = await invoke<PromptEntry[]>("delete_prompt_files", { paths });
      allPrompts = updated ?? [];
      config = await invoke<AppConfig>("get_config");
      selectedIds = new Set();
      selectedIndex = 0;
      status = "删除成功";
    } catch (error) {
      status = `删除失败：${formatError(error)}`;
    } finally {
      await refreshResults();
    }
  }

  function getSelectedPrompts() {
    if (selectedIds.size === 0) {
      return [];
    }
    return filtered.filter((prompt) => selectedIds.has(prompt.id));
  }

  function setSingleSelection(prompt: PromptEntry) {
    selectedIds = new Set([prompt.id]);
  }

  function toggleSelection(prompt: PromptEntry) {
    const next = new Set(selectedIds);
    if (next.has(prompt.id)) {
      next.delete(prompt.id);
    } else {
      next.add(prompt.id);
    }
    selectedIds = next;
  }

  function onResultClick(event: MouseEvent, prompt: PromptEntry, index: number) {
    if (event.ctrlKey) {
      selectedIndex = index;
      toggleSelection(prompt);
      return;
    }
    selectedIndex = index;
    setSingleSelection(prompt);
    void usePrompt(prompt);
  }


  function onResultContextMenu(
    event: MouseEvent,
    prompt: PromptEntry,
    index: number
  ) {
    event.preventDefault();
    selectedIndex = index;
    if (!selectedIds.has(prompt.id)) {
      setSingleSelection(prompt);
    }
    // Keep context menu within window bounds so all items remain visible.
    const menuItemCount = 4;
    const menuItemHeight = 40;
    const menuPadding = 12;
    const menuWidth = 160;
    const menuHeight = menuPadding + menuItemCount * menuItemHeight;
    const padding = 8;
    const maxX = Math.max(padding, window.innerWidth - menuWidth - padding);
    const maxY = Math.max(padding, window.innerHeight - menuHeight - padding);
    const x = Math.min(Math.max(padding, event.clientX), maxX);
    const y = Math.min(Math.max(padding, event.clientY), maxY);
    contextTarget = prompt;
    contextMenu = {
      visible: true,
      x,
      y
    };
  }

  function closeContextMenu() {
    if (!contextMenu.visible) {
      return;
    }
    contextMenu = { ...contextMenu, visible: false };
    contextTarget = null;
  }

  function getTagSuggestions(value: string, tags: string[]) {
    const parts = value.trim().split(/\s+/);
    const last = parts[parts.length - 1] ?? "";
    if (!last.startsWith("#")) {
      return [];
    }
    const keyword = last.slice(1).toLowerCase();
    return tags
      .filter((tag) => tag.toLowerCase().startsWith(keyword))
      .slice(0, 8);
  }

  function applyTagSuggestion(tag: string) {
    const parts = query.trim().split(/\s+/).filter(Boolean);
    if (parts.length === 0) {
      query = `#${tag}`;
    } else {
      parts[parts.length - 1] = `#${tag}`;
      query = parts.join(" ").trim();
    }
    selectedIndex = 0;
    scheduleSearch();
    focusSearch();
  }

  function openTagEditor(mode: "add" | "remove") {
    const selected = getSelectedPrompts();
    if (selected.length === 0) {
      status = "请选择项目";
      return;
    }
    tagEditorMode = mode;
    tagInput = "";
    removeTagSelection = new Set();
    if (mode === "remove") {
      const tagSet = new Set<string>();
      selected.forEach((prompt) => {
        prompt.tags?.forEach((tag) => tagSet.add(tag));
      });
      removeTagOptions = Array.from(tagSet).sort((a, b) => a.localeCompare(b));
    } else {
      removeTagOptions = [];
    }
    closeContextMenu();
  }

  function closeTagEditor() {
    tagEditorMode = null;
    tagInput = "";
    removeTagOptions = [];
    removeTagSelection = new Set();
  }

  function stopPropagation(event: Event) {
    event.stopPropagation();
  }

  function onBackdropKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      closeTagEditor();
    }
  }

  function toggleRemoveTag(tag: string) {
    const next = new Set(removeTagSelection);
    if (next.has(tag)) {
      next.delete(tag);
    } else {
      next.add(tag);
    }
    removeTagSelection = next;
  }

  async function applyTagEditor() {
    const selected = getSelectedPrompts();
    if (selected.length === 0) {
      status = "请选择项目";
      closeTagEditor();
      return;
    }
    const paths = selected.map((prompt) => prompt.path);
    try {
      if (tagEditorMode === "add") {
        const tokens = tagInput
          .split(/\s+/)
          .map((token) => token.trim())
          .filter(Boolean);
        if (tokens.length === 0) {
          status = "标签不能为空";
          return;
        }
        await invoke<PromptEntry[]>("update_prompt_tags", {
          paths,
          add: tokens,
          remove: []
        });
        status = "标签已添加";
      } else if (tagEditorMode === "remove") {
        const tokens = Array.from(removeTagSelection);
        if (tokens.length === 0) {
          status = "请选择要移除的标签";
          return;
        }
        await invoke<PromptEntry[]>("update_prompt_tags", {
          paths,
          add: [],
          remove: tokens
        });
        status = "标签已移除";
      }
      closeTagEditor();
      await refreshResults();
    } catch (error) {
      status = typeof error === "string" ? error : "标签更新失败";
    }
  }

  async function createQuickPrompt() {
    const name = window.prompt("输入文件名");
    const trimmed = name?.trim() ?? "";
    if (!trimmed) {
      return;
    }
    try {
      const path = await invoke<string>("create_prompt_file", { name: trimmed });
      try {
        const result = await openPathWithFallback(path);
        if (result.ok) {
          await appWindow.hide();
          status = "文件已创建并打开";
        } else {
          status = `文件已创建，但打开失败：${result.message}`;
        }
      } catch {
        status = "文件已创建，但打开失败";
      }
      await refreshResults();
    } catch (error) {
      status = formatError(error) || "创建失败";
    }
  }

  async function toggleSettings() {
    showSettings = !showSettings;
    await tick();
    if (!showSettings) {
      focusSearch();
    }
  }

  async function toggleTopTagsScope(nextValue?: boolean) {
    const value =
      typeof nextValue === "boolean"
        ? nextValue
        : !config.top_tags_use_results;
    config = { ...config, top_tags_use_results: value };
    await invoke("set_top_tags_scope", { useResults: value });
    return value;
  }

  async function setTopTagsLimit(limit: number) {
    const value = Math.max(1, Math.min(20, Math.floor(limit)));
    config = { ...config, top_tags_limit: value };
    await invoke("set_top_tags_limit", { limit: value });
  }

  async function setPreviewChars(previewChars: number) {
    const value = Math.max(10, Math.min(200, Math.floor(previewChars)));
    if (value === config.preview_chars) {
      return;
    }
    config = { ...config, preview_chars: value };
    await invoke("set_preview_chars", { previewChars: value });
    await refreshResults();
  }

  function onPreviewCharsChange(event: Event) {
    const target = event.target as HTMLInputElement | null;
    if (!target) {
      return;
    }
    const raw = Number(target.value);
    if (!Number.isFinite(raw)) {
      target.value = String(config.preview_chars);
      return;
    }
    const value = Math.max(10, Math.min(200, Math.floor(raw)));
    if (target.value !== String(value)) {
      target.value = String(value);
    }
    if (value === config.preview_chars) {
      return;
    }
    void setPreviewChars(value);
  }

  async function applyTopTagsScopeForFilter() {
    if (topTagsScopeBeforeFilter === null) {
      topTagsScopeBeforeFilter = config.top_tags_use_results;
    }
    if (!config.top_tags_use_results) {
      await toggleTopTagsScope(true);
    }
  }

  async function restoreTopTagsScopeAfterFilter() {
    if (topTagsScopeBeforeFilter === null) {
      return;
    }
    const previous = topTagsScopeBeforeFilter;
    topTagsScopeBeforeFilter = null;
    if (config.top_tags_use_results !== previous) {
      await toggleTopTagsScope(previous);
    }
  }

  function buildTopTags(prompts: PromptEntry[], limit: number) {
    const counts = new Map<string, number>();
    prompts.forEach((prompt) => {
      prompt.tags?.forEach((tag) => {
        counts.set(tag, (counts.get(tag) ?? 0) + 1);
      });
    });
    return Array.from(counts.entries())
      .sort((a, b) => {
        if (b[1] !== a[1]) {
          return b[1] - a[1];
        }
        return a[0].localeCompare(b[0]);
      })
      .slice(0, limit)
      .map(([tag, count]) => ({ tag, count }));
  }

  function getTagSource() {
    if (config.top_tags_use_results) {
      return filtered;
    }
    if (showFavorites) {
      return allPrompts.filter((prompt) => isFavorite(prompt));
    }
    if (showRecent) {
      return buildRecentList(allPrompts, config.recent_ids).map(
        (item) => item.prompt
      );
    }
    return allPrompts;
  }

  function normalizeTagToken(tag: string) {
    return `#${tag.toLowerCase()}`;
  }

  function hasTagFilters() {
    return query
      .split(/\\s+/)
      .some((part) => part.startsWith("#") && part.length > 1);
  }

  function hasQuery() {
    return query.trim().length > 0;
  }

  function hasAnyFilters() {
    return hasQuery() || showFavorites || showRecent;
  }

  function isTagActive(tag: string) {
    const token = normalizeTagToken(tag);
    return query
      .split(/\\s+/)
      .some((part) => part.toLowerCase() === token);
  }

  function resetAllFilters() {
    showFavorites = false;
    showRecent = false;
    query = "";
    selectedIndex = 0;
    status = "筛选已重置";
    if (topTagsScopeBeforeFilter !== null) {
      void restoreTopTagsScopeAfterFilter();
    }
    void refreshResults();
    focusSearch();
  }

  function clearTagFilters() {
    const parts = query.split(/\\s+/).filter(Boolean);
    const remaining = parts.filter((part) => !part.startsWith("#"));
    query = remaining.join(" ").trim();
    selectedIndex = 0;
    status = "标签已清空";
    void refreshResults();
    focusSearch();
  }

  function toggleTagFilter(tag: string) {
    const token = normalizeTagToken(tag);
    const parts = query.split(/\\s+/).filter(Boolean);
    const hasTag = parts.some((part) => part.toLowerCase() === token);
    const filtered = parts.filter((part) => part.toLowerCase() !== token);
    const next = hasTag ? filtered : [...filtered, token];
    query = next.join(" ").trim();
    selectedIndex = 0;
    void refreshResults();
    focusSearch();
  }

  function getRowPreview(prompt: PromptEntry) {
    const terms = extractTerms(query);
    if (terms.length === 0) {
      return prompt.preview;
    }
    const snippet = makeSnippet(prompt.body, terms);
    return snippet || prompt.preview;
  }

  function getRowPreviewHtml(prompt: PromptEntry) {
    const snippet = getRowPreview(prompt);
    if (!snippet) {
      return "";
    }
    const terms = extractTerms(query);
    if (terms.length === 0) {
      return escapeHtml(snippet);
    }
    return highlightSnippet(snippet, terms);
  }

  function getPreviewBodyHtml(prompt: PromptEntry) {
    const terms = extractTerms(query);
    if (terms.length === 0) {
      return escapeHtml(prompt.body);
    }
    return highlightSnippet(prompt.body, terms);
  }

  function escapeHtml(text: string) {
    return text.replace(/[&<>"']/g, (match) => {
      switch (match) {
        case "&":
          return "&amp;";
        case "<":
          return "&lt;";
        case ">":
          return "&gt;";
        case "\"":
          return "&quot;";
        case "'":
          return "&#39;";
        default:
          return match;
      }
    });
  }

  function escapeRegex(text: string) {
    return text.replace(/[.*+?^${}()|[\\]\\\\]/g, "\\\\$&");
  }

  function highlightSnippet(snippet: string, terms: string[]) {
    const uniqueTerms = Array.from(new Set(terms.filter(Boolean)));
    if (uniqueTerms.length === 0) {
      return escapeHtml(snippet);
    }
    const pattern = uniqueTerms.map(escapeRegex).join("|");
    if (!pattern) {
      return escapeHtml(snippet);
    }
    const regex = new RegExp(`(${pattern})`, "gi");
    let result = "";
    let lastIndex = 0;
    for (const match of snippet.matchAll(regex)) {
      const index = match.index ?? 0;
      const value = match[0] ?? "";
      if (index < lastIndex) {
        continue;
      }
      result += escapeHtml(snippet.slice(lastIndex, index));
      result += `<mark>${escapeHtml(value)}</mark>`;
      lastIndex = index + value.length;
    }
    result += escapeHtml(snippet.slice(lastIndex));
    return result;
  }

  function extractTerms(rawQuery: string) {
    return rawQuery
      .split(/\\s+/)
      .filter((term) => term && !term.startsWith("#"))
      .map((term) => term.toLowerCase());
  }

  function makeSnippet(body: string, terms: string[]) {
    if (!body) {
      return "";
    }
    const compact = body.replace(/\\s+/g, " ").trim();
    const lower = compact.toLowerCase();
    let bestIndex = -1;
    let bestTerm = "";

    for (const term of terms) {
      const index = lower.indexOf(term);
      if (index !== -1 && (bestIndex === -1 || index < bestIndex)) {
        bestIndex = index;
        bestTerm = term;
      }
    }

    if (bestIndex === -1) {
      return "";
    }

    const start = Math.max(0, bestIndex - 40);
    const end = Math.min(compact.length, bestIndex + bestTerm.length + 60);
    let snippet = compact.slice(start, end).trim();
    if (start > 0) {
      snippet = `...${snippet}`;
    }
    if (end < compact.length) {
      snippet = `${snippet}...`;
    }
    return snippet;
  }

  function formatLastUsed(prompt: PromptEntry) {
    const timestamp = config.recent_meta[prompt.id];
    if (!timestamp) {
      return "从未使用";
    }
    const delta = Date.now() - timestamp;
    if (delta < 60000) {
      return "刚刚";
    }
    if (delta < 3600000) {
      return `${Math.floor(delta / 60000)} 分钟前`;
    }
    if (delta < 86400000) {
      return `${Math.floor(delta / 3600000)} 小时前`;
    }
    return `${Math.floor(delta / 86400000)} 天前`;
  }

  function onSearchInput(event: Event) {
    const target = event.target as HTMLInputElement | null;
    query = target?.value ?? "";
    selectedIndex = 0;
    scheduleSearch();
  }

  function onSearchKeydown(event: KeyboardEvent) {
    if (event.isComposing || isComposing) {
      return;
    }
    if (event.key === "Process" || event.keyCode === 229) {
      return;
    }
    if (event.key === "Enter" && Date.now() - compositionEndedAt < 120) {
      return;
    }
    if (event.key === "Escape" && contextMenu.visible) {
      event.preventDefault();
      closeContextMenu();
      return;
    }
    if (event.key === "Escape" && tagEditorMode) {
      event.preventDefault();
      closeTagEditor();
      return;
    }
    if (
      event.ctrlKey &&
      event.shiftKey &&
      event.key.toLowerCase() === "f"
    ) {
      event.preventDefault();
      toggleSelectedFavorite();
      return;
    }
    if (
      event.ctrlKey &&
      event.shiftKey &&
      event.key.toLowerCase() === "g"
    ) {
      event.preventDefault();
      toggleFavoritesFilter();
      status = showFavorites ? "收藏过滤已开启" : "收藏过滤已关闭";
      return;
    }
    if (
      event.ctrlKey &&
      event.shiftKey &&
      event.key.toLowerCase() === "r"
    ) {
      event.preventDefault();
      void clearRecent();
      status = "最近记录已清空";
      return;
    }
    if (
      event.ctrlKey &&
      event.shiftKey &&
      event.key.toLowerCase() === "e"
    ) {
      event.preventDefault();
      toggleRecentFilter();
      status = showRecent ? "最近过滤已关闭" : "最近过滤已开启";
      return;
    }
    if (
      event.ctrlKey &&
      event.shiftKey &&
      event.key.toLowerCase() === "s"
    ) {
      event.preventDefault();
      const nextValue = !config.top_tags_use_results;
      void toggleTopTagsScope(nextValue);
      status = nextValue ? "热门标签：结果" : "热门标签：全部";
      return;
    }
    if (filtered.length === 0 && !showSettings) {
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
      if (!showSettings && filtered.length > 0) {
          usePrompt(filtered[selectedIndex]);
      }
      return;
    }
    if (event.key === "Escape") {
      event.preventDefault();
      if (showSettings) {
          showSettings = false;
          focusSearch();
      } else {
          appWindow.hide();
      }
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
    const baseResults = results ?? [];
    if (showRecent) {
      filtered = buildRecentList(baseResults, config.recent_ids).map(
        (item) => item.prompt
      );
    } else {
      filtered = baseResults;
    }
    if (selectedIndex >= filtered.length) {
      selectedIndex = 0;
    }
    activePrompt = filtered[selectedIndex] ?? null;
    const validIds = new Set(filtered.map((prompt) => prompt.id));
    const nextSelection = new Set(
      Array.from(selectedIds).filter((id) => validIds.has(id))
    );
    if (nextSelection.size === 0 && activePrompt) {
      nextSelection.add(activePrompt.id);
    }
    selectedIds = nextSelection;
  }
</script>

<main class="launcher-root">
    <div class="input-bar" class:expanded={hasContent}>
        <div class="search-icon">
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="11" cy="11" r="8"></circle>
                <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
            </svg>
        </div>
        <input
            bind:this={searchInput}
            class="main-input"
            placeholder="Search prompts..."
            value={query}
            oninput={onSearchInput}
            onkeydown={onSearchKeydown}
            oncompositionstart={() => {
              isComposing = true;
            }}
            oncompositionend={() => {
              isComposing = false;
              compositionEndedAt = Date.now();
            }}
        />
        <button class="add-btn" onclick={createQuickPrompt} title="新建短语">
             <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="12" y1="5" x2="12" y2="19"></line>
                <line x1="5" y1="12" x2="19" y2="12"></line>
             </svg>
        </button>
        <button class="settings-btn" class:active={showSettings} onclick={toggleSettings} title="Settings">
             <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="3"></circle>
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
            </svg>
        </button>
    </div>

    {#if hasContent}
        <div class="results-dropdown">
            {#if showSettings}
                <div class="settings-container">
                    <div class="settings-header-mini">
                        <span>设置</span>
                        <span class="version">{appVersion ? `v${appVersion}` : "v--"}</span>
                    </div>

                    <div class="settings-scroll-area">
                        <div class="settings-section">
                            <div class="section-title">基础配置</div>
                            <div class="setting-item">
                                <span class="label">提示词目录</span>
                                <div class="controls">
                                    <div class="path-display" title={config.prompts_dir}>{config.prompts_dir || "未设置"}</div>
                                    <button class="btn-sm" onclick={chooseFolder}>选择</button>
                                </div>
                            </div>
                            <div class="setting-item">
                                <span class="label">快捷键</span>
                                <div class="controls">
                                    <input class="input-sm" bind:value={hotkeyDraft} onkeydown={onHotkeyInputKeydown} placeholder="按下组合键..." />
                                    <button class="btn-sm" onclick={applyHotkey}>应用</button>
                                </div>
                            </div>
                            {#if hotkeyError}
                                <div class="settings-error">{hotkeyError}</div>
                            {/if}
                        </div>

                        <div class="settings-section">
                             <div class="section-title">行为选项</div>
                             <div class="setting-item">
                                <span class="label">自动粘贴</span>
                                <label class="toggle-switch">
                                    <input type="checkbox" checked={config.auto_paste} onchange={toggleAutoPaste} />
                                    <span class="slider"></span>
                                </label>
                             </div>
                             <div class="setting-item">
                                <span class="label">开机自启</span>
                                <label class="toggle-switch">
                                    <input type="checkbox" checked={config.auto_start} onchange={toggleAutoStart} />
                                    <span class="slider"></span>
                                </label>
                             </div>
                             {#if settingsError}
                                 <div class="settings-error">{settingsError}</div>
                             {/if}
                             <div class="setting-item">
                                <span class="label">预览长度(字)</span>
                                <div class="controls">
                                    <input
                                        class="input-sm"
                                        type="number"
                                        min="10"
                                        max="200"
                                        step="1"
                                        value={config.preview_chars}
                                        onchange={onPreviewCharsChange}
                                    />
                                </div>
                             </div>
                        </div>
                    </div>
                </div>
            {:else}
                {#if tagSuggestions.length > 0}
                    <div class="tag-suggestions">
                        {#each tagSuggestions as tag}
                            <button class="tag-suggestion" onclick={() => applyTagSuggestion(tag)}>
                                #{tag}
                            </button>
                        {/each}
                    </div>
                {/if}
                <div class="results-list">
                    {#if filtered.length === 0}
                         <div class="empty-state">
                             <span class="empty-icon">🔍</span>
                             <span>没有找到匹配的提示词</span>
                         </div>
                    {:else}
                        {#each filtered as prompt, index (prompt.id)}
                            <button
                                class="result-item"
                                class:selected={index === selectedIndex}
                                class:multi-selected={selectedIds.has(prompt.id)}
                                type="button"
                                onclick={(event) => onResultClick(event, prompt, index)}
                                oncontextmenu={(event) => onResultContextMenu(event, prompt, index)}
                                onmouseenter={() => selectedIndex = index}
                            >
                                <div class="result-main">
                                    <div class="result-title">
                                        <span class="title-text">{prompt.title}</span>
                                        {#if prompt.tags?.length}
                                            <span class="tags-inline">
                                                {#each prompt.tags as tag}
                                                    <span class="tag-pill">#{tag}</span>
                                                {/each}
                                            </span>
                                        {/if}
                                    </div>
                                    <div class="result-preview">{@html getRowPreviewHtml(prompt)}</div>
                                </div>
                            </button>
                        {/each}
                    {/if}
                </div>

                <div class="dropdown-footer">
                    {#if status}
                        <span class="status-msg">{status}</span>
                    {:else}
                        <span class="keys-hint">
                            <span class="key">↵</span> 粘贴
                            <span class="key">Esc</span> 返回
                            <span class="key">Tab</span> 选择
                        </span>
                    {/if}
                </div>
            {/if}
        </div>
    {/if}

    {#if contextMenu.visible}
        <div
            class="context-menu"
            style={`top: ${contextMenu.y}px; left: ${contextMenu.x}px;`}
            onpointerdown={stopPropagation}
        >
            <button class="context-item" onclick={() => openTagEditor("add")}>添加标签</button>
            <button class="context-item" onclick={() => openTagEditor("remove")}>移除标签</button>
            <button
                class="context-item"
                onclick={() => {
                  const target = contextTarget;
                  closeContextMenu();
                  void openPrompt(target);
                }}
            >打开文件</button>
            <button
                class="context-item"
                onclick={() => {
                  closeContextMenu();
                  void deleteSelectedPrompts();
                }}
            >删除文件</button>
        </div>
    {/if}

    {#if tagEditorMode}
        <div
            class="modal-backdrop"
            role="button"
            tabindex="0"
            onpointerdown={closeTagEditor}
            onkeydown={onBackdropKeydown}
        ></div>
        <div
            class="modal"
            role="dialog"
            tabindex="-1"
            onpointerdown={stopPropagation}
        >
            {#if tagEditorMode === "add"}
                <div class="modal-title">添加标签</div>
                <input
                    class="modal-input"
                    placeholder="用空格分隔多个标签"
                    value={tagInput}
                    oninput={(event) => {
                      const target = event.target as HTMLInputElement | null;
                      tagInput = target?.value ?? "";
                    }}
                />
            {:else}
                <div class="modal-title">移除标签</div>
                {#if removeTagOptions.length === 0}
                    <div class="modal-hint">当前选择没有可移除标签</div>
                {:else}
                    <div class="modal-options">
                        {#each removeTagOptions as tag}
                            <label class="modal-option">
                                <input
                                    type="checkbox"
                                    checked={removeTagSelection.has(tag)}
                                    onchange={() => toggleRemoveTag(tag)}
                                />
                                <span>#{tag}</span>
                            </label>
                        {/each}
                    </div>
                {/if}
            {/if}
            <div class="modal-actions">
                <button class="btn-sm" onclick={applyTagEditor}>确定</button>
                <button class="btn-sm ghost" onclick={closeTagEditor}>取消</button>
            </div>
        </div>
    {/if}
</main>

<style>
:global(html),
:global(body) {
  margin: 0;
  padding: 0;
  background: transparent !important;
  color: #c9d1d9; /* Light text for dark mode default, or use system adaptive */
  font-family: "Segoe UI", "Roboto", sans-serif;
  overflow: hidden;
  height: 100vh;
  width: 100vw;
}

/* Light theme variables (defaulting to a clean light look for now as per request for "classic") */
:root {
  --bg-color: #ffffff;
  --text-color: #333333;
  --accent-color: #0078d4;
  --selected-bg: #f3f3f3;
  --border-color: #e5e5e5;
  --shadow: none;
  --input-height: 56px;
  --radius: 0;
}

.launcher-root {
  display: flex;
  flex-direction: column;
  padding: 0;
  margin: 0;
  width: 100%;
  height: 100vh;
  box-sizing: border-box;
  align-items: stretch;
  justify-content: flex-start;
  background: transparent !important;
}

.input-bar {
  width: 100%;
  height: var(--input-height);
  background: var(--bg-color);
  border-radius: 0;
  box-shadow: none;
  display: flex;
  align-items: center;
  padding: 0 16px;
  transition: none;
  z-index: 100;
  position: relative;
  margin: 0;
  opacity: 1;
}

.input-bar.expanded {
  border-bottom-left-radius: 0;
  border-bottom-right-radius: 0;
  border-bottom: 1px solid var(--border-color);
}

.search-icon {
  color: #777;
  display: flex;
  align-items: center;
  margin-right: 12px;
}

.main-input {
  flex: 1;
  border: none;
  background: transparent;
  font-size: 20px;
  outline: none;
  color: var(--text-color);
  height: 100%;
}

.settings-btn {
  background: transparent;
  border: none;
  color: #999;
  cursor: pointer;
  padding: 8px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  transition: all 0.2s;
}

.settings-btn:hover, .settings-btn.active {
  background: #f0f0f0;
  color: #333;
}

.results-dropdown {
  width: 100%;
  background: var(--bg-color);
  border-radius: 0;
  box-shadow: none;
  overflow: visible;
  max-height: 400px;
  display: flex;
  flex-direction: column;
  animation: slideDown 0.15s ease-out;
  margin: 0;
  opacity: 1;
}

@keyframes slideDown {
  from { opacity: 0; transform: translateY(-10px); }
  to { opacity: 1; transform: translateY(0); }
}

.results-list {
  max-height: 360px;
  overflow-y: auto;
  padding: 8px 0;
}

.result-item {
  padding: 10px 16px;
  cursor: pointer;
  border-left: 3px solid transparent;
  display: flex;
  align-items: center;
  background: transparent;
  border: none;
  width: 100%;
  text-align: left;
  font: inherit;
}

.result-item.selected {
  background-color: var(--selected-bg);
  border-left-color: var(--accent-color);
}

.result-main {
  flex: 1;
  overflow: hidden;
}

.result-title {
  font-size: 16px;
  font-weight: 500;
  color: var(--text-color);
  display: flex;
  align-items: center;
  gap: 8px;
}

.title-text {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tags-inline {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}

.tag-pill {
  font-size: 11px;
  background: #eef;
  color: #44a;
  padding: 1px 6px;
  border-radius: 12px;
}

.add-btn {
  width: 34px;
  height: 34px;
  border-radius: 8px;
  border: 1px solid var(--border-color);
  background: #ffffff;
  color: #1f2a37;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  margin-right: 8px;
  cursor: pointer;
}

.add-btn:hover {
  background: #f3f5f9;
}

.tag-suggestions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  padding: 8px 16px 0 16px;
  background: var(--bg-color);
  border-left: 1px solid var(--border-color);
  border-right: 1px solid var(--border-color);
}

.tag-suggestion {
  border: 1px solid var(--border-color);
  background: #ffffff;
  color: #1f2a37;
  border-radius: 999px;
  padding: 4px 10px;
  font-size: 12px;
  cursor: pointer;
}

.tag-suggestion:hover {
  background: #f3f5f9;
}

.result-item.multi-selected {
  background: #e9f2ff;
  border-color: #cfe2ff;
}

.context-menu {
  position: fixed;
  background: #ffffff;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 6px;
  box-shadow: 0 10px 30px rgba(15, 23, 42, 0.12);
  z-index: 2000;
  min-width: 140px;
}

.context-item {
  display: block;
  width: 100%;
  text-align: left;
  padding: 8px 10px;
  border: none;
  background: transparent;
  color: #111827;
  border-radius: 6px;
  cursor: pointer;
}

.context-item:hover {
  background: #f3f5f9;
}

.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.28);
  z-index: 1900;
  border: none;
  padding: 0;
}

.modal {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  background: #ffffff;
  border-radius: 12px;
  border: 1px solid var(--border-color);
  padding: 16px;
  min-width: 280px;
  max-width: 360px;
  z-index: 2001;
}

.modal-title {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 12px;
  color: #111827;
}

.modal-input {
  width: 100%;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 8px 10px;
  font-size: 13px;
}

.modal-options {
  display: flex;
  flex-direction: column;
  gap: 6px;
  max-height: 200px;
  overflow-y: auto;
}

.modal-option {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: #1f2a37;
}

.modal-hint {
  font-size: 12px;
  color: #6b7280;
  margin-bottom: 10px;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 14px;
}

.btn-sm.ghost {
  background: transparent;
  border: 1px solid var(--border-color);
}

.result-preview {
  font-size: 13px;
  color: #666;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-top: 2px;
}

:global(.result-preview mark) {
  background: rgba(255, 230, 0, 0.4);
  color: inherit;
  padding: 0;
}

.empty-state {
  padding: 30px;
  text-align: center;
  color: #888;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
}

.empty-icon {
  font-size: 24px;
}

.dropdown-footer {
  padding: 8px 16px;
  background: #fafafa;
  border-top: 1px solid var(--border-color);
  font-size: 12px;
  color: #888;
  display: flex;
  justify-content: space-between;
}

.keys-hint {
  display: flex;
  gap: 12px;
}

.key {
  background: #eee;
  padding: 1px 5px;
  border-radius: 3px;
  font-family: monospace;
  font-size: 11px;
  margin-right: 2px;
}

/* Settings Styles */
.settings-container {
    padding: 16px;
    height: 360px;
    display: flex;
    flex-direction: column;
}

.settings-header-mini {
    font-size: 14px;
    font-weight: bold;
    color: #555;
    margin-bottom: 12px;
    display: flex;
    justify-content: space-between;
}

.settings-scroll-area {
    flex: 1;
    overflow-y: auto;
}

.settings-section {
    margin-bottom: 20px;
}

.section-title {
    font-size: 12px;
    color: var(--accent-color);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 8px;
    font-weight: 600;
}

.setting-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
    font-size: 13px;
    color: var(--text-color);
}

.settings-error {
    font-size: 12px;
    color: #b91c1c;
    margin: 2px 0 8px 0;
}

.controls {
    display: flex;
    gap: 8px;
    align-items: center;
}

.path-display {
    max-width: 150px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: #666;
    font-size: 12px;
    background: #f5f5f5;
    padding: 2px 6px;
    border-radius: 4px;
}

.btn-sm {
    padding: 3px 8px;
    font-size: 12px;
    border: 1px solid #ddd;
    background: white;
    border-radius: 4px;
    cursor: pointer;
}

.btn-sm:hover {
    background: #f0f0f0;
}

.input-sm {
    padding: 3px 6px;
    font-size: 12px;
    border: 1px solid #ddd;
    border-radius: 4px;
    width: 100px;
}

/* Toggle Switch */
.toggle-switch {
  position: relative;
  display: inline-block;
  width: 32px;
  height: 18px;
}

.toggle-switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: #ccc;
  transition: .4s;
  border-radius: 34px;
}

.slider:before {
  position: absolute;
  content: "";
  height: 14px;
  width: 14px;
  left: 2px;
  bottom: 2px;
  background-color: white;
  transition: .4s;
  border-radius: 50%;
}

input:checked + .slider {
  background-color: var(--accent-color);
}

input:checked + .slider:before {
  transform: translateX(14px);
}
</style>
