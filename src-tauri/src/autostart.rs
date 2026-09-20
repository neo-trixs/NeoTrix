use anyhow::{Context, Result as AnyhowResult};
use crate::ipc;
use crate::ipc::IpcResponse;
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

pub struct AutoStartManager {
    app: AppHandle,
}

impl AutoStartManager {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    pub fn is_enabled(&self) -> AnyhowResult<bool> {
        let autostart = self.app.autolaunch();
        Ok(autostart.is_enabled().unwrap_or(false))
    }

    pub fn enable(&self) -> AnyhowResult<()> {
        let autostart = self.app.autolaunch();
        autostart
            .enable()
            .context("Failed to enable autostart")
    }

    pub fn disable(&self) -> AnyhowResult<()> {
        let autostart = self.app.autolaunch();
        autostart
            .disable()
            .context("Failed to disable autostart")
    }

    pub fn toggle(&self) -> AnyhowResult<bool> {
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
pub fn autostart_is_enabled(state: tauri::State<'_, AutoStartManager>) -> IpcResponse<bool> {
    match state.is_enabled() {
        Ok(v) => ipc::ok(v),
        Err(e) => ipc::err("AUTOSTART_QUERY_FAILED", format!("{e}")),
    }
}

#[tauri::command]
pub fn autostart_enable(state: tauri::State<'_, AutoStartManager>) -> IpcResponse<()> {
    match state.enable() {
        Ok(()) => ipc::ok(()),
        Err(e) => ipc::err("AUTOSTART_ENABLE_FAILED", format!("{e}")),
    }
}

#[tauri::command]
pub fn autostart_disable(state: tauri::State<'_, AutoStartManager>) -> IpcResponse<()> {
    match state.disable() {
        Ok(()) => ipc::ok(()),
        Err(e) => ipc::err("AUTOSTART_DISABLE_FAILED", format!("{e}")),
    }
}

#[tauri::command]
pub fn autostart_toggle(state: tauri::State<'_, AutoStartManager>) -> IpcResponse<bool> {
    match state.toggle() {
        Ok(v) => ipc::ok(v),
        Err(e) => ipc::err("AUTOSTART_TOGGLE_FAILED", format!("{e}")),
    }
}
