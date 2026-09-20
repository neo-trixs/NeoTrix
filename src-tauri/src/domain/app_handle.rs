use std::sync::OnceLock;
use tauri::AppHandle;

static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

pub fn set_app_handle(app: AppHandle) {
    if let Err(e) = APP_HANDLE.set(app) {
        tracing::warn!("app_handle: already set: {e}");
    }
}

pub fn get_app_handle() -> Option<&'static AppHandle> {
    APP_HANDLE.get()
}
