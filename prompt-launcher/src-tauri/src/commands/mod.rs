pub mod config;
pub mod prompts;
pub mod window;

pub use config::{
    clear_recent, get_config, push_recent, set_append_clipboard, set_auto_paste, set_auto_start,
    set_hotkey, set_preview_chars, set_recent_enabled, set_show_shortcuts_hint,
    set_top_tags_limit, set_top_tags_scope, toggle_favorite,
};
pub use prompts::{
    create_prompt_file, delete_prompt_files, list_prompts, open_prompt_path, search_prompts,
    set_prompts_dir, update_prompt_tags,
};
pub use window::{capture_active_window, focus_last_window, frontend_ready};
