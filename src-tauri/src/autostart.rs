use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

pub struct AutoStartManager {
    app: AppHandle,
}

impl AutoStartManager {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    pub fn is_enabled(&self) -> Result<bool, String> {
        let autostart = self.app.autolaunch();
        Ok(autostart.is_enabled().unwrap_or(false))
    }

    pub fn enable(&self) -> Result<(), String> {
        let autostart = self.app.autolaunch();
        autostart
            .enable()
            .map_err(|e| format!("Failed to enable autostart: {}", e))
    }

    pub fn disable(&self) -> Result<(), String> {
        let autostart = self.app.autolaunch();
        autostart
            .disable()
            .map_err(|e| format!("Failed to disable autostart: {}", e))
    }

    pub fn toggle(&self) -> Result<bool, String> {
        if self.is_enabled()? {
            self.disable()?;
            Ok(false)
        } else {
            self.enable()?;
            Ok(true)
        }
    }
}

#[tauri::command]
pub fn autostart_is_enabled(state: tauri::State<'_, AutoStartManager>) -> Result<bool, String> {
    state.is_enabled()
}

#[tauri::command]
pub fn autostart_enable(state: tauri::State<'_, AutoStartManager>) -> Result<(), String> {
    state.enable()
}

#[tauri::command]
pub fn autostart_disable(state: tauri::State<'_, AutoStartManager>) -> Result<(), String> {
    state.disable()
}

#[tauri::command]
pub fn autostart_toggle(state: tauri::State<'_, AutoStartManager>) -> Result<bool, String> {
    state.toggle()
}
