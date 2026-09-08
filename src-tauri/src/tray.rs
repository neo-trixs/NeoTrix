use tauri::{
    image::Image,
    menu::{Menu, MenuEvent, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

pub fn create_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let show_i = MenuItem::with_id(app, "show", "Show NeoTrix", true, None::<&str>)?;
    let health_i = MenuItem::with_id(app, "health", "Health Status", true, None::<&str>)?;
    let models_i = MenuItem::with_id(app, "models", "Model Manager", true, None::<&str>)?;
    let separator = tauri::menu::PredefinedMenuItem::separator(app)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit NeoTrix", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[&show_i, &health_i, &models_i, &separator, &quit_i],
    )?;

    let _tray = TrayIconBuilder::new()
        .icon(Image::from_path("icons/32x32.png")?)
        .tooltip("NeoTrix Desktop")
        .menu(&menu)
        .on_menu_event(move |app, event| {
            handle_menu_event(app, event);
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}

fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
        "show" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "health" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                let _ = window.emit("navigate", "health");
            }
        }
        "models" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                let _ = window.emit("navigate", "models");
            }
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    }
}
