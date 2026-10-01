//! macOS 原生菜单栏（上游 `dsh-harness-desktop` 0.19.1 的
//! `src-tauri/src/desktop/builder.rs::install_macos_menu` 移植，MIT）。
//!
//! ## 为什么必须有它（这是「1:1 还原」里最容易漏的一处）
//!
//! 壳层 `layout/components/navbar.tsx`（本仓**逐字未改**）在 macOS 上把
//! 「文件」「运行」「帮助」**整组隐藏**：
//!
//! ```text
//! <If cond={!IS_MACOS}> …文件/运行/帮助… </If>
//! <If cond={!IS_MACOS}> …最小化/最大化/关闭… </If>
//! ```
//!
//! 它们假定这些入口由**原生菜单栏**承载（`navbar.tsx` 里那段注释写明
//! 「macOS：「文件」「帮助」在 macOS 上由原生菜单栏承载，见
//! `desktop/builder.rs` 的 `install_macos_menu`」）。
//!
//! ⛔ 本仓此前**从未安装过菜单** ⇒ macOS 上那两组按钮隐藏之后，配置、关于、
//! 运行日志、检查更新、文档、新建窗口/新聊天/打开文件夹**全部无入口**，
//! 而前端 `useListen('macos-menu-action')` 那一整段分发逻辑（本仓同样逐字
//! 未改）永远是死代码。
//!
//! ## 编辑菜单不是可选项（上游 issue #85）
//!
//! macOS 一旦挂上主菜单，⌘X/⌘C/⌘V/⌘A 会先经菜单的 key-equivalent 路由；
//! 不挂载标准编辑项，WebView 里的输入框**无法剪切/复制/粘贴**。
//! 所以编辑菜单与文件菜单同级，不是「顺手加的」。
//!
//! ## 语言
//!
//! 文案取自上游 `config/i18n.rs` 的同名键（逐条抄，包括 `menu.close` 的 ⌘W）。

use std::sync::atomic::{AtomicU8, Ordering};

/// 语言。默认中文，与上游 `Lang::Zh` 默认值一致。
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Zh,
    En,
}

static CURRENT_LANG: AtomicU8 = AtomicU8::new(0); // 0 = zh, 1 = en

/// 设置原生菜单的语言。
pub fn set_language(lang: Lang) {
    CURRENT_LANG.store(
        match lang {
            Lang::Zh => 0,
            Lang::En => 1,
        },
        Ordering::SeqCst,
    );
}

fn lang() -> Lang {
    if CURRENT_LANG.load(Ordering::SeqCst) == 1 {
        Lang::En
    } else {
        Lang::Zh
    }
}

/// 文案表。**逐条抄自上游 `src-tauri/src/config/i18n.rs`**，未改一字。
///
/// ⛔ 只放菜单用得到的键：上游那张表还兼着运行时/安装错误文案，
/// 那些在本仓由别的错误路径产出（`install_dependencies` 等），抄过来
/// 会变成没人调用的僵尸文案。
pub fn t(key: &str) -> String {
    let (zh, en): (&str, &str) = match key {
        "menu.application" => ("应用", "Application"),
        "menu.help" => ("帮助", "Help"),
        "menu.file" => ("文件", "File"),
        "menu.new_window" => ("新建窗口", "New Window"),
        "menu.new_chat" => ("新聊天", "New Chat"),
        "menu.open_folder" => ("打开文件夹", "Open Folder"),
        "menu.close" => ("关闭", "Close"),
        "menu.quit" => ("退出", "Quit"),
        "menu.documentation" => ("文档", "Documentation"),
        "menu.settings" => ("设置…", "Settings…"),
        "menu.enter_fullscreen" => ("进入全屏幕", "Enter Full Screen"),
        "menu.exit_fullscreen" => ("退出全屏幕", "Exit Full Screen"),
        "menu.about" => ("关于 Desktop", "About Desktop"),
        "menu.run_logs" => ("运行日志", "Run Logs"),
        "menu.check_update" => ("检查更新", "Check for Updates"),
        "menu.restart" => ("重启", "Restart"),
        "menu.edit" => ("编辑", "Edit"),
        "menu.undo" => ("撤销", "Undo"),
        "menu.redo" => ("重做", "Redo"),
        "menu.cut" => ("剪切", "Cut"),
        "menu.copy" => ("复制", "Copy"),
        "menu.paste" => ("粘贴", "Paste"),
        "menu.select_all" => ("全选", "Select All"),
        _ => (key, key),
    };
    match lang() {
        Lang::Zh => zh.to_owned(),
        Lang::En => en.to_owned(),
    }
}

/// 菜单项 id → 前端动作的映射表。
///
/// 前端 `navbar.tsx` 的 switch 恰好处理这 9 个 id（逐字未改）。这张表是
/// 「Rust 侧发了什么」的单一真源，用它测 —— 而不是把 9 个字面量散在
/// `on_menu_event` 的 match 里（那样加菜单项时会漏掉其中一处，且没人发现）。
pub const FORWARDED_ACTIONS: [&str; 9] = [
    "desktop-config",
    "desktop-about",
    "desktop-copy-run-logs",
    "desktop-check-update",
    "desktop-restart",
    "desktop-documentation",
    "desktop-new-window",
    "desktop-new-chat",
    "desktop-open-folder",
];

/// 该 id 是否要转发给前端（其余由系统自己处理：关闭/退出/全屏/编辑）。
pub fn forwards_to_frontend(id: &str) -> bool {
    FORWARDED_ACTIONS.contains(&id)
}

/// 安装 macOS 原生菜单栏。
///
/// 非 macOS 平台是**空实现**：上游在 Windows/Linux 上把同一组项渲染在
/// 壳层导航栏（`navbar.tsx` 的 `!IS_MACOS` 分支，本仓逐字保留），
/// 在那里再装一份原生菜单就是重复入口。
#[cfg(target_os = "macos")]
pub fn install(app: &tauri::AppHandle<tauri::Wry>) -> Result<(), String> {
    use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
    use tauri::Emitter;

    // —— 系统应用菜单：hide / hide_others / show_all / quit ——
    // macOS 会把首个菜单标题强制显示为应用名，所以它只承载系统动作，
    // 真正可见的「应用」菜单放在其后（上游同款顺序，否则会被系统改名）。
    let hide = PredefinedMenuItem::hide(app, None).map_err(|e| e.to_string())?;
    let hide_others = PredefinedMenuItem::hide_others(app, None).map_err(|e| e.to_string())?;
    let show_all = PredefinedMenuItem::show_all(app, None).map_err(|e| e.to_string())?;
    let sep_a = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let quit_sys = PredefinedMenuItem::quit(app, None).map_err(|e| e.to_string())?;
    let system_application_menu = Submenu::with_id_and_items(
        app,
        "desktop-system-application-menu",
        app.package_info().name.clone(),
        true,
        &[&hide, &hide_others, &show_all, &sep_a, &quit_sys],
    )
    .map_err(|e| e.to_string())?;

    // —— 应用：设置…（⌘,）/ 全屏 ——
    let config = MenuItem::with_id(
        app,
        "desktop-config",
        t("menu.settings"),
        true,
        Some("CmdOrCtrl+,"),
    )
    .map_err(|e| e.to_string())?;
    let sep_b = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let fullscreen = PredefinedMenuItem::fullscreen(app, Some(&t("menu.enter_fullscreen")))
        .map_err(|e| e.to_string())?;
    let application_menu = Submenu::with_id_and_items(
        app,
        "desktop-application-menu",
        t("menu.application"),
        true,
        &[&config, &sep_b, &fullscreen],
    )
    .map_err(|e| e.to_string())?;

    // —— 文件：新建窗口 ⌘N / 新聊天 ⌘⇧N / 打开文件夹 ⌘O / 关闭 ⌘W / 退出 ——
    // 与非 macOS 上导航栏的「文件」下拉同组同序（navbar.tsx 的 !IS_MACOS 分支）。
    let new_window =
        MenuItem::with_id(app, "desktop-new-window", t("menu.new_window"), true, Some("CmdOrCtrl+N"))
            .map_err(|e| e.to_string())?;
    let new_chat =
        MenuItem::with_id(app, "desktop-new-chat", t("menu.new_chat"), true, Some("CmdOrCtrl+Shift+N"))
            .map_err(|e| e.to_string())?;
    let open_folder =
        MenuItem::with_id(app, "desktop-open-folder", t("menu.open_folder"), true, Some("CmdOrCtrl+O"))
            .map_err(|e| e.to_string())?;
    let sep_c = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let close =
        PredefinedMenuItem::close_window(app, Some(&t("menu.close"))).map_err(|e| e.to_string())?;
    let sep_d = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let quit = PredefinedMenuItem::quit(app, Some(&t("menu.quit"))).map_err(|e| e.to_string())?;
    let file_menu = Submenu::with_id_and_items(
        app,
        "desktop-file-menu",
        t("menu.file"),
        true,
        &[&new_window, &new_chat, &open_folder, &sep_c, &close, &sep_d, &quit],
    )
    .map_err(|e| e.to_string())?;

    // —— 编辑：撤销/重做/剪切/复制/粘贴/全选（issue #85，见模块注释）——
    let undo = PredefinedMenuItem::undo(app, Some(&t("menu.undo"))).map_err(|e| e.to_string())?;
    let redo = PredefinedMenuItem::redo(app, Some(&t("menu.redo"))).map_err(|e| e.to_string())?;
    let sep_e = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let cut = PredefinedMenuItem::cut(app, Some(&t("menu.cut"))).map_err(|e| e.to_string())?;
    let copy = PredefinedMenuItem::copy(app, Some(&t("menu.copy"))).map_err(|e| e.to_string())?;
    let paste = PredefinedMenuItem::paste(app, Some(&t("menu.paste"))).map_err(|e| e.to_string())?;
    let sep_f = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let select_all = PredefinedMenuItem::select_all(app, Some(&t("menu.select_all")))
        .map_err(|e| e.to_string())?;
    let edit_menu = Submenu::with_id_and_items(
        app,
        "desktop-edit-menu",
        t("menu.edit"),
        true,
        &[&undo, &redo, &sep_e, &cut, &copy, &paste, &sep_f, &select_all],
    )
    .map_err(|e| e.to_string())?;

    // —— 帮助：运行日志 / 重启 / 检查更新 / — / 文档 / 关于 Desktop ——
    let run_logs = MenuItem::with_id(
        app,
        "desktop-copy-run-logs",
        t("menu.run_logs"),
        true,
        None::<&str>,
    )
    .map_err(|e| e.to_string())?;
    let restart =
        MenuItem::with_id(app, "desktop-restart", t("menu.restart"), true, None::<&str>)
            .map_err(|e| e.to_string())?;
    let check_update = MenuItem::with_id(
        app,
        "desktop-check-update",
        t("menu.check_update"),
        true,
        None::<&str>,
    )
    .map_err(|e| e.to_string())?;
    let sep_g = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let documentation =
        MenuItem::with_id(app, "desktop-documentation", t("menu.documentation"), true, None::<&str>)
            .map_err(|e| e.to_string())?;
    let about =
        MenuItem::with_id(app, "desktop-about", t("menu.about"), true, None::<&str>)
            .map_err(|e| e.to_string())?;
    let help_menu = Submenu::with_id_and_items(
        app,
        "desktop-help-menu",
        t("menu.help"),
        true,
        &[&run_logs, &restart, &check_update, &sep_g, &documentation, &about],
    )
    .map_err(|e| e.to_string())?;

    let menu = Menu::with_items(
        app,
        &[&system_application_menu, &application_menu, &file_menu, &edit_menu, &help_menu],
    )
    .map_err(|e| e.to_string())?;
    app.set_menu(menu).map_err(|e| e.to_string())?;

    // 菜单项点击 → 只发 id，逻辑复用前端已有实现（与上游同款：
    // 壳层不该自己长出一套并行实现）。
    app.on_menu_event(move |app, event| {
        let id = event.id().as_ref();
        if forwards_to_frontend(id) {
            if let Err(e) = app.emit("macos-menu-action", id) {
                eprintln!("[menu] 发 macos-menu-action 失败：{e}");
            }
        }
    });
    Ok(())
}

/// 非 macOS：不需要原生菜单（导航栏已渲染同一组项）。
#[cfg(not(target_os = "macos"))]
pub fn install(_app: &tauri::AppHandle<tauri::Wry>) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{forwards_to_frontend, set_language, t, Lang, FORWARDED_ACTIONS};

    #[test]
    fn 文案双语与上游一致() {
        set_language(Lang::Zh);
        assert_eq!(t("menu.settings"), "设置…");
        assert_eq!(t("menu.about"), "关于 Desktop");
        assert_eq!(t("menu.select_all"), "全选");
        set_language(Lang::En);
        assert_eq!(t("menu.settings"), "Settings…");
        assert_eq!(t("menu.about"), "About Desktop");
        assert_eq!(t("menu.select_all"), "Select All");
        set_language(Lang::Zh);
        // 未知键回显键名（上游同款：不静默吞掉）。
        assert_eq!(t("menu.nope"), "menu.nope");
    }

    #[test]
    fn 转发表覆盖前端switch的九个id() {
        // ⛔ 这 9 个必须与 `navbar.tsx` 的 `useListen('macos-menu-action')`
        //    switch 分支一一对应。少一个 = 那个菜单项点了没反应。
        for id in [
            "desktop-config",
            "desktop-about",
            "desktop-copy-run-logs",
            "desktop-check-update",
            "desktop-restart",
            "desktop-documentation",
            "desktop-new-window",
            "desktop-new-chat",
            "desktop-open-folder",
        ] {
            assert!(forwards_to_frontend(id), "{id} 必须转发给前端");
        }
        // 系统自带项不转发（转发两次会既执行又发事件）。
        for id in ["fullscreen", "quit", "close_window", "copy", "paste", "hide"] {
            assert!(!forwards_to_frontend(id), "{id} 由系统处理，不该转发");
        }
        assert_eq!(FORWARDED_ACTIONS.len(), 9);
    }
}
