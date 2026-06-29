use std::sync::Mutex;
use tauri::{command, State};
use crate::plugins::manager::{PluginManager, PluginInfo};

#[command]
pub fn plugin_list(manager: State<'_, Mutex<PluginManager>>) -> Vec<PluginInfo> {
    let mut mgr = manager.lock().expect("PluginManager mutex poisoned");
    mgr.discover()
}

#[command]
pub fn plugin_load(manager: State<'_, Mutex<PluginManager>>, name: String) -> Result<(), String> {
    let mut mgr = manager.lock().expect("PluginManager mutex poisoned");
    mgr.load(&name)
}

#[command]
pub fn plugin_unload(manager: State<'_, Mutex<PluginManager>>, name: String) -> Result<(), String> {
    let mut mgr = manager.lock().expect("PluginManager mutex poisoned");
    mgr.unload(&name)
}

#[command]
pub fn plugin_uninstall(manager: State<'_, Mutex<PluginManager>>, name: String) -> Result<(), String> {
    let mut mgr = manager.lock().expect("PluginManager mutex poisoned");
    mgr.uninstall(&name)
}

#[command]
pub fn plugin_install_from_zip(manager: State<'_, Mutex<PluginManager>>, zip_path: String) -> Result<PluginInfo, String> {
    let mut mgr = manager.lock().expect("PluginManager mutex poisoned");
    mgr.install_from_zip(&zip_path)
}

#[command]
pub fn plugin_get_info(manager: State<'_, Mutex<PluginManager>>, name: String) -> Option<PluginInfo> {
    let mgr = manager.lock().expect("PluginManager mutex poisoned");
    mgr.get_plugin_info(&name).cloned()
}

#[command]
pub fn plugin_write_data(manager: State<'_, Mutex<PluginManager>>, name: String, key: String, value: String) -> Result<(), String> {
    let mgr = manager.lock().expect("PluginManager mutex poisoned");
    mgr.write_plugin_data(&name, &key, &value)
}

#[command]
pub fn plugin_read_data(manager: State<'_, Mutex<PluginManager>>, name: String, key: String) -> Result<Option<String>, String> {
    let mgr = manager.lock().expect("PluginManager mutex poisoned");
    mgr.read_plugin_data(&name, &key)
}
