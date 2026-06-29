// Tauri 2.0 主入口 - 最简版，只启动窗口加载前端
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[path = "src/projects.rs"]
mod projects;
#[path = "src/git_integration.rs"]
mod git_integration;

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("tauri::Builder::run failed - app crashed at runtime");
}
