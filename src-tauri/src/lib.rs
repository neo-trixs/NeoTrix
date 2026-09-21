#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]
#![warn(clippy::all, clippy::pedantic, clippy::dbg_macro, clippy::print_stdout, clippy::print_stderr)]
#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    reason = "pedantic false-positives in Tauri plugin boilerplate"
)]

pub(crate) mod agent_identity;
pub mod app_error;
pub(crate) mod atomic_io;
pub mod autostart;
pub(crate) mod bot;
pub mod browser_host;
pub mod commands;
pub mod config;
pub mod constants;
pub mod db_pool;
pub(crate) mod debouncer;
pub mod desktop;
pub mod domain;
pub(crate) mod engine;
pub mod error_codes;
pub mod health;
pub mod ipc;
pub mod logger;
pub mod market;
pub mod notifications;
pub mod recovery;
pub mod service;
pub mod stub;
pub mod util;
pub(crate) mod validated;
pub(crate) mod vault;

use tauri::{Emitter, Manager};

/// Build a native app menu (macOS-style) so keyboard shortcuts like Cmd+C/V,
/// Cmd+Q and standard roles behave like a first-class desktop app. Menu events
/// that matter to NeoTrix (check updates, new session, settings) are forwarded
/// to the frontend as window events; the rest use Tauri predefined roles.
pub fn setup_menu(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::menu::{MenuBuilder, PredefinedMenuItem, SubmenuBuilder};

    let check_updates =
        tauri::menu::MenuItemBuilder::with_id("check_updates", "Check for Updates…")
            .accelerator("CmdOrCtrl+Shift+U")
            .build(app)?;
    let new_session = tauri::menu::MenuItemBuilder::with_id("new_session", "New Session")
        .accelerator("CmdOrCtrl+N")
        .build(app)?;
    let open_settings = tauri::menu::MenuItemBuilder::with_id("open_settings", "Settings…")
        .accelerator("CmdOrCtrl+,")
        .build(app)?;
    let cmd_palette = tauri::menu::MenuItemBuilder::with_id("cmd_palette", "Command Palette…")
        .accelerator("CmdOrCtrl+K")
        .build(app)?;

    let app_menu = SubmenuBuilder::new(app, "NeoTrix")
        .item(&PredefinedMenuItem::about(app, Some("NeoTrix"), None)?)
        .separator()
        .item(&check_updates)
        .separator()
        .item(&PredefinedMenuItem::hide(app, Some("Hide NeoTrix"))?)
        .item(&PredefinedMenuItem::hide_others(app, Some("Hide Others"))?)
        .item(&PredefinedMenuItem::show_all(app, Some("Show All"))?)
        .separator()
        .item(&PredefinedMenuItem::quit(app, Some("Quit NeoTrix"))?)
        .build()?;

    let file_menu = SubmenuBuilder::new(app, "File")
        .item(&new_session)
        .item(&open_settings)
        // P2-2: no close_window item here — its default CmdOrCtrl+W accelerator
        // would be captured by the native menu and prevent the webview's
        // "⌘W 删除会话" handler (delete-session confirm) from ever firing.
        .build()?;

    let edit_menu = SubmenuBuilder::new(app, "Edit")
        .item(&PredefinedMenuItem::undo(app, Some("Undo"))?)
        .item(&PredefinedMenuItem::redo(app, Some("Redo"))?)
        .separator()
        .item(&PredefinedMenuItem::cut(app, Some("Cut"))?)
        .item(&PredefinedMenuItem::copy(app, Some("Copy"))?)
        .item(&PredefinedMenuItem::paste(app, Some("Paste"))?)
        .item(&PredefinedMenuItem::select_all(app, Some("Select All"))?)
        .build()?;

    let view_menu = SubmenuBuilder::new(app, "View")
        .item(&cmd_palette)
        .separator()
        .item(&PredefinedMenuItem::fullscreen(
            app,
            Some("Enter Full Screen"),
        )?)
        .build()?;

    let window_menu = SubmenuBuilder::new(app, "Window")
        .item(&PredefinedMenuItem::minimize(app, Some("Minimize"))?)
        .item(&PredefinedMenuItem::maximize(app, Some("Maximize"))?)
        .item(&PredefinedMenuItem::close_window(
            app,
            Some("Close Window"),
        )?)
        .build()?;

    let help_menu = SubmenuBuilder::new(app, "Help")
        .item(&check_updates)
        .build()?;

    let menu = MenuBuilder::new(app)
        .items(&[
            &app_menu,
            &file_menu,
            &edit_menu,
            &view_menu,
            &window_menu,
            &help_menu,
        ])
        .build()?;

    app.set_menu(menu)?;

    app.on_menu_event(|app, event| match event.id().as_ref() {
        "check_updates" => {
            if let Err(e) = app.emit("neotrix_check_updates", ()) {
                tracing::trace!("emit check_updates: {e}");
            }
        }
        "new_session" => {
            if let Err(e) = app.emit("neotrix_new_session", ()) {
                tracing::trace!("emit new_session: {e}");
            }
        }
        "open_settings" => {
            if let Err(e) = app.emit("open_settings", ()) {
                tracing::trace!("emit open_settings: {e}");
            }
        }
        "cmd_palette" => {
            if let Err(e) = app.emit("neotrix_open_palette", ()) {
                tracing::trace!("emit open_palette: {e}");
            }
        }
        _ => {}
    });

    Ok(())
}

pub fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "stealth-net")]
    use tauri::menu::SubmenuBuilder;
    use tauri::menu::{MenuBuilder, MenuItemBuilder};
    use tauri::tray::TrayIconBuilder;

    let show = MenuItemBuilder::with_id("show", "Show Window").build(app)?;
    let config = MenuItemBuilder::with_id("config", "Open Config").build(app)?;
    let sync_now = MenuItemBuilder::with_id("sync_now", "Sync Now").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

    let proxy_menu = {
        #[cfg(feature = "stealth-net")]
        {
            let geo = MenuItemBuilder::with_id("proxy_geo", "🌍 Geo").build(app)?;
            let stealth = MenuItemBuilder::with_id("proxy_stealth", "🕶 Stealth").build(app)?;
            let tor = MenuItemBuilder::with_id("proxy_tor", "🧅 Tor").build(app)?;
            let off = MenuItemBuilder::with_id("proxy_off", "⛔ Off").build(app)?;
            let status_item = MenuItemBuilder::with_id("proxy_status", "Status...").build(app)?;
            Some(
                SubmenuBuilder::new(app, "Proxy")
                    .item(&geo)
                    .item(&stealth)
                    .item(&tor)
                    .item(&off)
                    .separator()
                    .item(&status_item)
                    .build()?,
            )
        }
        #[cfg(not(feature = "stealth-net"))]
        {
            None as Option<tauri::menu::Submenu<tauri::Wry>>
        }
    };

    let menu = {
        let mut b = MenuBuilder::new(app).item(&show).item(&config);
        if let Some(ref pm) = proxy_menu {
            b = b.separator().item(pm);
        }
        b.separator()
            .item(&sync_now)
            .separator()
            .item(&quit)
            .build()?
    };

    let icon = tauri::include_image!("icons/icon.png");

    TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .menu(&menu)
        .tooltip("NeoTrix Desktop")
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    if let Err(e) = window.show() {
                        tracing::trace!("tray show: {e}");
                    }
                    if let Err(e) = window.set_focus() {
                        tracing::trace!("tray focus: {e}");
                    }
                }
            }
            "config" => {
                if let Err(e) = app.emit("open_settings", ()) {
                    tracing::trace!("emit open_settings: {e}");
                }
            }
            "sync_now" => {
                if let Err(e) = app.emit("sync_trigger", ()) {
                    tracing::trace!("emit sync_trigger: {e}");
                }
            }
            #[cfg(feature = "stealth-net")]
            mode_id @ ("proxy_geo" | "proxy_stealth" | "proxy_tor" | "proxy_off") => {
                let mode = mode_id.strip_prefix("proxy_").unwrap_or("geo");
                if let Err(e) = app.emit("proxy_mode_change", mode) {
                    tracing::trace!("emit proxy_mode_change: {e}");
                }
            }
            #[cfg(feature = "stealth-net")]
            "proxy_status" => {
                if let Err(e) = app.emit("open_proxy_status", ()) {
                    tracing::trace!("emit open_proxy_status: {e}");
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}
