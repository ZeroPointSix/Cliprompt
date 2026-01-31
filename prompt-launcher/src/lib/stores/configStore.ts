import { get, writable } from "svelte/store";
import { tauriClient } from "$lib/tauriClient";
import type { AppConfig, RecentState } from "$lib/types";

const defaultConfig: AppConfig = {
  prompts_dir: "",
  auto_paste: true,
  append_clipboard: false,
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
};

const store = writable<AppConfig>(defaultConfig);

const setLocal = (partial: Partial<AppConfig>) => {
  store.update((current) => ({ ...current, ...partial }));
};

const applyRecentState = (recentState: RecentState | undefined) => {
  setLocal({
    recent_ids: recentState?.recent_ids ?? [],
    recent_meta: recentState?.recent_meta ?? {}
  });
};

export const configStore = {
  subscribe: store.subscribe,
  set: store.set,
  update: store.update,
  load: async () => {
    const config = await tauriClient.getConfig();
    store.set(config);
    return config;
  },
  setPromptsDirSync: (path: string) => {
    setLocal({ prompts_dir: path });
  },
  setAutoPaste: async (value: boolean) => {
    setLocal({ auto_paste: value });
    await tauriClient.setAutoPaste(value);
  },
  setAppendClipboard: async (value: boolean) => {
    setLocal({ append_clipboard: value });
    await tauriClient.setAppendClipboard(value);
  },
  setHotkey: async (hotkey: string) => {
    await tauriClient.setHotkey(hotkey);
    setLocal({ hotkey });
  },
  setAutoStart: async (value: boolean) => {
    const previous = get(store).auto_start;
    setLocal({ auto_start: value });
    try {
      await tauriClient.setAutoStart(value);
    } catch (error) {
      setLocal({ auto_start: previous });
      throw error;
    }
  },
  toggleFavorite: async (id: string) => {
    const favorites = await tauriClient.toggleFavorite(id);
    setLocal({ favorites: favorites ?? [] });
    return favorites ?? [];
  },
  pushRecent: async (id: string) => {
    const recentState = await tauriClient.pushRecent(id);
    applyRecentState(recentState);
    return recentState;
  },
  setRecentEnabled: async (value: boolean) => {
    setLocal({ recent_enabled: value });
    await tauriClient.setRecentEnabled(value);
  },
  clearRecent: async () => {
    const recentState = await tauriClient.clearRecent();
    applyRecentState(recentState);
    return recentState;
  },
  setTopTagsScope: async (value: boolean) => {
    setLocal({ top_tags_use_results: value });
    await tauriClient.setTopTagsScope(value);
  },
  setTopTagsLimit: async (value: number) => {
    setLocal({ top_tags_limit: value });
    await tauriClient.setTopTagsLimit(value);
  },
  setPreviewChars: async (value: number) => {
    setLocal({ preview_chars: value });
    await tauriClient.setPreviewChars(value);
  },
  setShowShortcutsHint: async (value: boolean) => {
    setLocal({ show_shortcuts_hint: value });
    await tauriClient.setShowShortcutsHint(value);
  }
};
