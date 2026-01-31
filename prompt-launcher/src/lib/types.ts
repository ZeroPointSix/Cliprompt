export type PromptEntry = {
  id: string;
  title: string;
  body: string;
  preview: string;
  tags: string[];
  path: string;
};

export type AppConfig = {
  prompts_dir: string;
  auto_paste: boolean;
  append_clipboard: boolean;
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

export type RecentState = {
  recent_ids: string[];
  recent_meta: Record<string, number>;
};
