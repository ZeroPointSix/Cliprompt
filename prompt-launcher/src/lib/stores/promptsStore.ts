import { writable } from "svelte/store";
import { tauriClient } from "$lib/tauriClient";
import type { PromptEntry } from "$lib/types";

const store = writable<PromptEntry[]>([]);

export const promptsStore = {
  subscribe: store.subscribe,
  set: store.set,
  update: store.update,
  loadAll: async () => {
    const prompts = await tauriClient.listPrompts();
    store.set(prompts ?? []);
    return prompts ?? [];
  },
  setFromEvent: (prompts: PromptEntry[]) => {
    store.set(prompts ?? []);
  },
  setPromptsDir: async (path: string) => {
    const prompts = await tauriClient.setPromptsDir(path);
    store.set(prompts ?? []);
    return prompts ?? [];
  },
  deletePromptFiles: async (paths: string[]) => {
    const prompts = await tauriClient.deletePromptFiles(paths);
    store.set(prompts ?? []);
    return prompts ?? [];
  },
  updatePromptTags: async (
    paths: string[],
    add: string[],
    remove: string[]
  ) => {
    const prompts = await tauriClient.updatePromptTags(paths, add, remove);
    store.set(prompts ?? []);
    return prompts ?? [];
  },
  createPromptFile: (name: string) => tauriClient.createPromptFile(name),
  searchPrompts: (query: string, limit: number, favoritesOnly: boolean) =>
    tauriClient.searchPrompts(query, limit, favoritesOnly)
};
