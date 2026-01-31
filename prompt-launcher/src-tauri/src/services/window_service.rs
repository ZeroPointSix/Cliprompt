use crate::constants::{EVENT_LAUNCHER_SHOWN, MAIN_WINDOW_LABEL};
use crate::lifecycle::GateDecision;
use crate::state::AppState;
#[cfg(target_os = "windows")]
use crate::win;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};
use tauri::Emitter;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

pub struct WindowService;

impl WindowService {
    pub fn frontend_ready(app: &AppHandle, state: &Arc<AppState>) -> Result<(), String> {
        let should_show = state.launcher_gate.lock().unwrap().set_ui_ready();
        if should_show {
            show_main_window(app)?;
        }
        Ok(())
    }

    pub fn capture_active_window(state: &Arc<AppState>) -> Result<(), String> {
        store_active_window(state)
    }

    pub fn focus_last_window(state: &Arc<AppState>, auto_paste: bool) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            let hwnd = *state.last_active_hwnd.lock().unwrap();
            if let Some(hwnd) = hwnd {
                win::focus_window(hwnd)?;
                if auto_paste {
                    thread::sleep(Duration::from_millis(30));
                    win::send_ctrl_v()?;
                }
            }
            Ok(())
        }
        #[cfg(not(target_os = "windows"))]
        {
            Ok(())
        }
    }

    pub fn update_hotkey_registration(
        app: &AppHandle,
        state: &Arc<AppState>,
        hotkey: &str,
    ) -> Result<(), String> {
        let current = state.registered_hotkey.lock().unwrap().clone();
        if current.as_deref() == Some(hotkey) {
            return Ok(());
        }

        register_global_hotkey(app, state, hotkey)?;

        if let Some(previous) = current {
            let _ = app.global_shortcut().unregister(previous.as_str());
        }

        *state.registered_hotkey.lock().unwrap() = Some(hotkey.to_string());
        Ok(())
    }

    pub fn request_show(app: &AppHandle, state: &Arc<AppState>) -> Result<(), String> {
        let should_show = state.launcher_gate.lock().unwrap().request_show();
        if !should_show {
            let _ = store_active_window(state);
            return Ok(());
        }
        let _ = store_active_window(state);
        show_main_window(app)
    }

    pub fn request_toggle(app: &AppHandle, state: &Arc<AppState>) -> Result<(), String> {
        let decision = state
            .launcher_gate
            .lock()
            .unwrap()
            .allow_toggle(Instant::now());
        match decision {
            GateDecision::Debounced => Ok(()),
            GateDecision::DeferShow => {
                let _ = store_active_window(state);
                Ok(())
            }
            GateDecision::Proceed => toggle_main_window(app, state),
        }
    }

    pub fn init_tray(app: &tauri::App) -> Result<(), String> {
        let show = MenuItem::with_id(app, "show", "显示", true, None::<&str>)
            .map_err(|e| format!("menu item failed: {e}"))?;
        let hide = MenuItem::with_id(app, "hide", "隐藏", true, None::<&str>)
            .map_err(|e| format!("menu item failed: {e}"))?;
        let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)
            .map_err(|e| format!("menu item failed: {e}"))?;
        let menu = Menu::with_items(app, &[&show, &hide, &quit])
            .map_err(|e| format!("menu build failed: {e}"))?;
        let icon = app
            .default_window_icon()
            .cloned()
            .ok_or_else(|| "missing default window icon".to_string())?;

        TrayIconBuilder::new()
            .icon(icon)
            .menu(&menu)
            .on_menu_event(|app, event| match event.id().as_ref() {
                "show" => {
                    let state = app.state::<Arc<AppState>>();
                    let _ = WindowService::request_show(app, state.inner());
                }
                "hide" => {
                    let _ = hide_main_window(app);
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            })
            .on_tray_icon_event(|tray, event| {
                if let TrayIconEvent::Click {
                    button,
                    button_state,
                    ..
                } = event
                {
                    if button == MouseButton::Left && button_state == MouseButtonState::Up {
                        let app = tray.app_handle();
                        let state = app.state::<Arc<AppState>>();
                        let _ = WindowService::request_toggle(&app, state.inner());
                    }
                }
            })
            .build(app)
            .map_err(|e| format!("tray init failed: {e}"))?;

        Ok(())
    }
}

fn store_active_window(state: &Arc<AppState>) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        if let Some(hwnd) = win::capture_foreground_window() {
            *state.last_active_hwnd.lock().unwrap() = Some(hwnd);
            Ok(())
        } else {
            Err("no active window detected".to_string())
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(())
    }
}

fn register_global_hotkey(
    app: &AppHandle,
    state: &Arc<AppState>,
    hotkey: &str,
) -> Result<(), String> {
    let state_handle = state.clone();
    app.global_shortcut()
        .on_shortcut(hotkey, move |app_handle, _, event| {
            if event.state == ShortcutState::Pressed {
                let _ = WindowService::request_toggle(app_handle, &state_handle);
            }
        })
        .map_err(|e| format!("register hotkey failed: {e}"))
}

fn main_window(app: &AppHandle) -> Result<tauri::WebviewWindow, String> {
    app.get_webview_window(MAIN_WINDOW_LABEL)
        .ok_or_else(|| "main window not found".to_string())
}

fn show_main_window(app: &AppHandle) -> Result<(), String> {
    let window = main_window(app)?;
    window
        .show()
        .map_err(|e| format!("show window failed: {e}"))?;
    window
        .set_focus()
        .map_err(|e| format!("focus window failed: {e}"))?;
    let _ = window.emit(EVENT_LAUNCHER_SHOWN, ());
    Ok(())
}

fn hide_main_window(app: &AppHandle) -> Result<(), String> {
    let window = main_window(app)?;
    window
        .hide()
        .map_err(|e| format!("hide window failed: {e}"))?;
    Ok(())
}

fn toggle_main_window(app: &AppHandle, state: &Arc<AppState>) -> Result<(), String> {
    let window = main_window(app)?;
    let visible = window
        .is_visible()
        .map_err(|e| format!("query window failed: {e}"))?;
    if visible {
        hide_main_window(app)
    } else {
        let _ = store_active_window(state);
        show_main_window(app)
    }
}
