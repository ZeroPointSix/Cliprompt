use std::mem::size_of;
use windows::Win32::Foundation::{BOOL, HWND};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
    VIRTUAL_KEY, VK_CONTROL, VK_V,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AttachThreadInput, GetForegroundWindow, GetWindowThreadProcessId, SetFocus, SetForegroundWindow,
};

pub fn capture_foreground_window() -> Option<isize> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0 == 0 {
            None
        } else {
            Some(hwnd.0)
        }
    }
}

pub fn focus_window(hwnd: isize) -> Result<(), String> {
    unsafe {
        let hwnd = HWND(hwnd);
        if hwnd.0 == 0 {
            return Err("no active window captured".to_string());
        }

        let target_thread = GetWindowThreadProcessId(hwnd, std::ptr::null_mut());
        let current_thread = GetCurrentThreadId();
        let _ = AttachThreadInput(current_thread, target_thread, BOOL(1));
        let _ = SetForegroundWindow(hwnd);
        let _ = SetFocus(hwnd);
        let _ = AttachThreadInput(current_thread, target_thread, BOOL(0));
    }
    Ok(())
}

pub fn send_ctrl_v() -> Result<(), String> {
    unsafe {
        let inputs = [
            key_input(VK_CONTROL, KEYBD_EVENT_FLAGS(0)),
            key_input(VK_V, KEYBD_EVENT_FLAGS(0)),
            key_input(VK_V, KEYEVENTF_KEYUP),
            key_input(VK_CONTROL, KEYEVENTF_KEYUP),
        ];
        let sent = SendInput(&inputs, size_of::<INPUT>() as i32);
        if sent == 0 {
            return Err("SendInput failed".to_string());
        }
    }
    Ok(())
}

fn key_input(key: VIRTUAL_KEY, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}
