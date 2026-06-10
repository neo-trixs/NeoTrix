use tauri::{Emitter, Manager};

pub fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder};
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
