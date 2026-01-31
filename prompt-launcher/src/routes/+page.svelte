<script lang="ts">
  import { onDestroy, onMount, tick } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { getVersion } from "@tauri-apps/api/app";
  import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { readText, writeText } from "@tauri-apps/plugin-clipboard-manager";
  import { EVENTS } from "$lib/constants";
  import { tauriClient } from "$lib/tauriClient";
  import { configStore } from "$lib/stores/configStore";
  import { promptsStore } from "$lib/stores/promptsStore";
  import { buildRecentList, buildTopTags, getTagSuggestions } from "$lib/promptList";
  import {
    applyTagSuggestion as buildTagSuggestion,
    clearTagFilters as clearTagFiltersValue,
    hasAnyFilters as hasAnyFiltersValue,
    hasTagFilters as hasTagFiltersValue,
    isTagActive as isTagActiveValue,
    toggleTagFilter as toggleTagFilterValue
  } from "$lib/launcherFilters";
  import SettingsPanel from "$lib/components/SettingsPanel.svelte";
  import ResultsList from "$lib/components/ResultsList.svelte";
  import type { PromptEntry } from "$lib/types";

  const appWindow = getCurrentWindow();
  const maxResults = 8;

  let searchInput = $state<HTMLInputElement | null>(null);
  let query = $state<string>("");
  let appVersion = $state<string>("");
  let config = $configStore;
  let selectedIndex = $state<number>(0);
  let status = $state<string>("");
  let hotkeyDraft = $state<string>("");
  let hotkeyError = $state<string>("");
  let settingsError = $state<string>("");
  let showSettings = $state<boolean>(false);
  let showShortcuts = $state<boolean>(false);
  let showFavorites = $state<boolean>(false);
  let showRecent = $state<boolean>(false);
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
  let allPrompts = $derived($promptsStore);
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
  let unlistenLauncherShown: UnlistenFn | null = null;
  let windowClickHandler: ((event: MouseEvent) => void) | null = null;
  let windowJustShown = false;
  let focusLossTimer: ReturnType<typeof setTimeout> | null = null;

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
    unlistenLauncherShown = await listen(EVENTS.LAUNCHER_SHOWN, async () => {
      windowJustShown = true;
      await tick();
      focusSearch();
      setTimeout(() => {
        windowJustShown = false;
      }, 500);
    });

    await tick();
    try {
      await tauriClient.frontendReady();
    } catch (error) {
      console.warn("[frontend_ready] Failed to notify backend", error);
    }

    let loadedConfig: typeof config | null = null;
    try {
      loadedConfig = await configStore.load();
    } catch (error) {
      status = formatError(error) || "读取配置失败";
      loadedConfig = config;
    }
    const effectiveConfig = loadedConfig ?? config;
    // Initialize hotkeyDraft from loaded config or store default
    hotkeyDraft = effectiveConfig.hotkey;

    // Load app version with fallback
    try {
      appVersion = await getVersion();
    } catch (error) {
      console.warn("[appVersion] Failed to load version", error);
      appVersion = "Unknown";
    }

    try {
      await promptsStore.loadAll();
    } catch (error) {
      status = formatError(error) || "提示词列表加载失败";
    }
    if (effectiveConfig.show_shortcuts_hint) {
      showShortcuts = true;
      try {
        await configStore.setShowShortcutsHint(false);
      } catch (error) {
        console.warn("[shortcutsHint] Failed to update hint flag", error);
      }
    }
    try {
      await refreshResults();
    } catch (error) {
      status = formatError(error) || "搜索初始化失败";
    }

    windowClickHandler = () => {
      closeContextMenu();
    };
    window.addEventListener("click", windowClickHandler);

    unlistenPrompts = await listen<PromptEntry[]>(
      EVENTS.PROMPTS_UPDATED,
      (event) => {
        promptsStore.setFromEvent(event.payload ?? []);
        selectedIndex = 0;
        void refreshResults();
      }
    );

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
    if (unlistenLauncherShown) {
      unlistenLauncherShown();
    }
    if (windowClickHandler) {
      window.removeEventListener("click", windowClickHandler);
    }
  });

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
      await tauriClient.openPromptPath(path);
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
    await promptsStore.setPromptsDir(dir);
    configStore.setPromptsDirSync(dir);
    status = "目录已更新";
    await refreshResults();
  }

  async function applyHotkey() {
    if (!hotkeyDraft.trim()) {
      hotkeyError = "快捷键不能为空";
      return;
    }
    hotkeyError = "";
    try {
      await configStore.setHotkey(hotkeyDraft);
      status = "快捷键已保存";
      settingsError = "";
    } catch (error) {
      hotkeyError = `快捷键注册失败：${formatError(error)}`;
      status = hotkeyError;
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
    await configStore.setAutoPaste(nextValue);
  }

  async function toggleAppendClipboard() {
    const nextValue = !config.append_clipboard;
    await configStore.setAppendClipboard(nextValue);
  }

  async function toggleAutoStart() {
    const nextValue = !config.auto_start;
    try {
      await configStore.setAutoStart(nextValue);
      settingsError = "";
    } catch (error) {
      settingsError = `自启设置失败：${error}`;
    }
  }

  async function toggleFavorite(prompt: PromptEntry | null | undefined) {
    if (!prompt) {
      return;
    }
    await configStore.toggleFavorite(prompt.id);
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
    await configStore.pushRecent(prompt.id);
  }

  async function toggleRecentEnabled() {
    const nextValue = !config.recent_enabled;
    await configStore.setRecentEnabled(nextValue);
    if (!nextValue) {
      await configStore.clearRecent();
    }
    void refreshResults();
  }

  async function clearRecent() {
    await configStore.clearRecent();
    void refreshResults();
  }

  async function usePrompt(prompt: PromptEntry | null | undefined) {
    if (!prompt) {
      console.log("[usePrompt] No prompt provided");
      return;
    }
    console.log("[usePrompt] Using prompt:", prompt.title);
    try {
      console.log("[usePrompt] append_clipboard:", config.append_clipboard);
      let clipboardText = "";
      if (config.append_clipboard) {
        try {
          clipboardText = await readText();
          console.log(
            "[usePrompt] Clipboard read length:",
            clipboardText ? clipboardText.length : 0
          );
        } catch (error) {
          console.warn("[usePrompt] Failed to read clipboard", error);
        }
      }
      const trimmedClipboard = clipboardText.trim();
      const output = trimmedClipboard
        ? `${prompt.body}\n\n---\n\n${clipboardText}`
        : prompt.body;
      console.log(
        "[usePrompt] Output length:",
        output.length,
        "Appended:",
        Boolean(trimmedClipboard)
      );
      await appWindow.hide();
      console.log("[usePrompt] Window hidden");
      await writeText(output);
      console.log("[usePrompt] Text written to clipboard");
      await tauriClient.focusLastWindow(config.auto_paste);
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
      await promptsStore.deletePromptFiles(paths);
      await configStore.load();
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

  function applyTagSuggestion(tag: string) {
    query = buildTagSuggestion(query, tag);
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
        await promptsStore.updatePromptTags(paths, tokens, []);
        status = "标签已添加";
      } else if (tagEditorMode === "remove") {
        const tokens = Array.from(removeTagSelection);
        if (tokens.length === 0) {
          status = "请选择要移除的标签";
          return;
        }
        await promptsStore.updatePromptTags(paths, [], tokens);
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
      const path = await promptsStore.createPromptFile(trimmed);
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
    await configStore.setTopTagsScope(value);
    return value;
  }

  async function setTopTagsLimit(limit: number) {
    const value = Math.max(1, Math.min(20, Math.floor(limit)));
    await configStore.setTopTagsLimit(value);
  }

  async function setPreviewChars(previewChars: number) {
    const value = Math.max(10, Math.min(200, Math.floor(previewChars)));
    if (value === config.preview_chars) {
      return;
    }
    await configStore.setPreviewChars(value);
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

  function hasTagFilters() {
    return hasTagFiltersValue(query);
  }

  function hasAnyFilters() {
    return hasAnyFiltersValue({ query, showFavorites, showRecent });
  }

  function isTagActive(tag: string) {
    return isTagActiveValue(query, tag);
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
    query = clearTagFiltersValue(query);
    selectedIndex = 0;
    status = "标签已清空";
    void refreshResults();
    focusSearch();
  }

  function toggleTagFilter(tag: string) {
    query = toggleTagFilterValue(query, tag);
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
    const results = await promptsStore.searchPrompts(
      query,
      maxResults,
      showFavorites
    );
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
                <SettingsPanel
                    {appVersion}
                    {config}
                    bind:hotkeyDraft
                    {hotkeyError}
                    {settingsError}
                    onChooseFolder={chooseFolder}
                    onHotkeyInputKeydown={onHotkeyInputKeydown}
                    onApplyHotkey={applyHotkey}
                    onToggleAutoPaste={toggleAutoPaste}
                    onToggleAppendClipboard={toggleAppendClipboard}
                    onToggleAutoStart={toggleAutoStart}
                    onPreviewCharsChange={onPreviewCharsChange}
                />
            {:else}
                <ResultsList
                    {tagSuggestions}
                    onApplyTagSuggestion={applyTagSuggestion}
                    {filtered}
                    {selectedIndex}
                    {selectedIds}
                    {status}
                    {getRowPreviewHtml}
                    onResultClick={onResultClick}
                    onResultContextMenu={onResultContextMenu}
                    onResultHover={(index) => (selectedIndex = index)}
                />
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

.btn-sm.ghost {
  background: transparent;
  border: 1px solid var(--border-color);
}
</style>
