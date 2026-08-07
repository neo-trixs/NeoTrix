#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod browser_host;

use tauri::{Emitter, Manager};

/// Build a native app menu (macOS-style) so keyboard shortcuts like Cmd+C/V,
/// Cmd+Q and standard roles behave like a first-class desktop app. Menu events
/// that matter to NeoCodex (check updates, new session, settings) are forwarded
/// to the frontend as window events; the rest use Tauri predefined roles.
pub fn setup_menu(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::menu::{MenuBuilder, PredefinedMenuItem, SubmenuBuilder};

    let check_updates = tauri::menu::MenuItemBuilder::with_id("check_updates", "Check for Updates…")
        .accelerator("CmdOrCtrl+Shift+U").build(app)?;
    let new_session = tauri::menu::MenuItemBuilder::with_id("new_session", "New Session")
        .accelerator("CmdOrCtrl+N").build(app)?;
    let open_settings = tauri::menu::MenuItemBuilder::with_id("open_settings", "Settings…")
        .accelerator("CmdOrCtrl+,").build(app)?;
    let cmd_palette = tauri::menu::MenuItemBuilder::with_id("cmd_palette", "Command Palette…")
        .accelerator("CmdOrCtrl+K").build(app)?;

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
        .item(&PredefinedMenuItem::fullscreen(app, Some("Enter Full Screen"))?)
        .build()?;

    let window_menu = SubmenuBuilder::new(app, "Window")
        .item(&PredefinedMenuItem::minimize(app, Some("Minimize"))?)
        .item(&PredefinedMenuItem::maximize(app, Some("Maximize"))?)
        .item(&PredefinedMenuItem::close_window(app, Some("Close Window"))?)
        .build()?;

    let help_menu = SubmenuBuilder::new(app, "Help")
        .item(&check_updates)
        .build()?;

    let menu = MenuBuilder::new(app)
        .items(&[&app_menu, &file_menu, &edit_menu, &view_menu, &window_menu, &help_menu])
        .build()?;

    app.set_menu(menu)?;

    app.on_menu_event(|app, event| match event.id().as_ref() {
        "check_updates" => { let _ = app.emit("neocodex-check-updates", ()); }
        "new_session" => { let _ = app.emit("neotrix:new-session", ()); }
        "open_settings" => { let _ = app.emit("open-settings", ()); }
        "cmd_palette" => { let _ = app.emit("neocodex-open-palette", ()); }
        _ => {}
    });

    Ok(())
}

pub fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::menu::{MenuBuilder, MenuItemBuilder};
    #[cfg(feature = "stealth-net")]
    use tauri::menu::SubmenuBuilder;
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
            Some(SubmenuBuilder::new(app, "Proxy")
                .item(&geo).item(&stealth).item(&tor).item(&off)
                .separator().item(&status_item)
                .build()?)
        }
        #[cfg(not(feature = "stealth-net"))]
        { None as Option<tauri::menu::Submenu<tauri::Wry>> }
    };

    let menu = {
        let mut b = MenuBuilder::new(app).item(&show).item(&config);
        if let Some(ref pm) = proxy_menu {
            b = b.separator().item(pm);
        }
        b.separator().item(&sync_now).separator().item(&quit).build()?
    };

    let icon = tauri::include_image!("icons/icon.png");

    TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .menu(&menu)
        .tooltip("NeoTrix Desktop")
        .on_menu_event(|app, event| {
            match event.id().as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "config" => {
                    let _ = app.emit("open-settings", ());
                }
                "sync_now" => {
                    let _ = app.emit("sync-trigger", ());
                }
                #[cfg(feature = "stealth-net")]
                mode_id @ ("proxy_geo" | "proxy_stealth" | "proxy_tor" | "proxy_off") => {
                    let mode = mode_id.strip_prefix("proxy_").unwrap_or("geo");
                    let _ = app.emit("proxy-mode-change", mode);
                }
                #[cfg(feature = "stealth-net")]
                "proxy_status" => {
                    let _ = app.emit("open-proxy-status", ());
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}
