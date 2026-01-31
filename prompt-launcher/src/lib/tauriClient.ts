import { invoke } from "@tauri-apps/api/core";
import type { AppConfig, PromptEntry, RecentState } from "./types";

export const tauriClient = {
  getConfig: () => invoke<AppConfig>("get_config"),
  listPrompts: () => invoke<PromptEntry[]>("list_prompts"),
  searchPrompts: (query: string, limit: number, favoritesOnly: boolean) =>
    invoke<PromptEntry[]>("search_prompts", { query, limit, favoritesOnly }),
  setPromptsDir: (path: string) =>
    invoke<PromptEntry[]>("set_prompts_dir", { path }),
  createPromptFile: (name: string) =>
    invoke<string>("create_prompt_file", { name }),
  openPromptPath: (path: string) => invoke("open_prompt_path", { path }),
  deletePromptFiles: (paths: string[]) =>
    invoke<PromptEntry[]>("delete_prompt_files", { paths }),
  updatePromptTags: (paths: string[], add: string[], remove: string[]) =>
    invoke<PromptEntry[]>("update_prompt_tags", { paths, add, remove }),
  setAutoPaste: (autoPaste: boolean) =>
    invoke("set_auto_paste", { autoPaste }),
  setAppendClipboard: (appendClipboard: boolean) =>
    invoke("set_append_clipboard", { appendClipboard }),
  setHotkey: (hotkey: string) => invoke("set_hotkey", { hotkey }),
  setAutoStart: (autoStart: boolean) =>
    invoke("set_auto_start", { autoStart }),
  toggleFavorite: (id: string) => invoke<string[]>("toggle_favorite", { id }),
  pushRecent: (id: string) => invoke<RecentState>("push_recent", { id }),
  setRecentEnabled: (recentEnabled: boolean) =>
    invoke("set_recent_enabled", { recentEnabled }),
  setTopTagsScope: (useResults: boolean) =>
    invoke("set_top_tags_scope", { useResults }),
  setTopTagsLimit: (limit: number) =>
    invoke("set_top_tags_limit", { limit }),
  setPreviewChars: (previewChars: number) =>
    invoke("set_preview_chars", { previewChars }),
  setShowShortcutsHint: (showShortcutsHint: boolean) =>
    invoke("set_show_shortcuts_hint", { showShortcutsHint }),
  clearRecent: () => invoke<RecentState>("clear_recent"),
  captureActiveWindow: () => invoke("capture_active_window"),
  focusLastWindow: (autoPaste: boolean) =>
    invoke("focus_last_window", { autoPaste }),
  frontendReady: () => invoke("frontend_ready")
};
