use std::io;
use std::path::Path;
use winreg::enums::HKEY_CURRENT_USER;
use winreg::RegKey;

const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const APP_NAME: &str = "Prompt Launcher";

pub fn set_auto_start(enabled: bool, exe_path: &Path) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(RUN_KEY)
        .map_err(|e| format!("open run key failed: {e}"))?;

    if enabled {
        let value = exe_path.to_string_lossy().to_string();
        key.set_value(APP_NAME, &value)
            .map_err(|e| format!("set run key failed: {e}"))?;
    } else if let Err(err) = key.delete_value(APP_NAME) {
        if err.kind() != io::ErrorKind::NotFound {
            return Err(format!("delete run key failed: {err}"));
        }
    }

    Ok(())
}
