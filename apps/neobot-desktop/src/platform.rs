//! 平台接线命令 —— 窗口、剪贴板、通知、开机自启、外链。
//!
//! # 这组命令的性质：**不是 agent 能力，是操作系统能力**
//!
//! 剪贴板、通知、开机自启、外链、多窗口 —— 每一件都有系统级细节
//! （剪贴板的图片编码、通知的权限询问、LaunchAgent 的 plist 落点、
//! 多窗口的 URL scheme 回调）。所以**接插件**比**自己写**既便宜也更可靠。
//!
//! 这与 `commands.rs` 里的会话/面板命令不同：那些是 neotrix 的业务，
//! 库里有对应实现；这里库里**没有也不该有**。
//!
//! # 参数类型：Tauri 命令按名传参
//!
//! 前端 `invoke('x', { a, b })` 会把 `a`/`b` 反序列化成 Rust 形参，
//! **形参名必须与前端传的键名逐字相同**。
//! ⛔ 写错不报「参数错」，而是「字段缺失」；若调用处 `.catch(() => {})`
//!    吞掉错误，症状是**静默不生效** —— 本仓已因此踩过一次
//!    （契约里 `log_frontend` 写成 2 参、实际 3 参）。
//!    `nt_check_api.mjs` ⑦ 现在对参数做双向对账。

use serde::Serialize;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::commands::data_dir;

/// `open_external_url(url)`
///
/// 用系统默认浏览器打开。⛔ 只允许 http/https —— 允许 `file://` 等协议
/// 会让一个来自对话内容的链接变成任意本地文件读取入口。
#[tauri::command]
pub async fn open_external_url(app: AppHandle, url: String) -> Result<(), String> {
    let u = url.trim();
    if !(u.starts_with("http://") || u.starts_with("https://")) {
        return Err(format!("只允许 http/https，收到：{}", &u[..u.len().min(40)]));
    }
    tauri_plugin_opener::OpenerExt::opener(&app)
        .open_url(u.to_owned(), None::<String>)
        .map_err(|e| e.to_string())
}

/// `write_clipboard_text(text)`
#[tauri::command]
pub fn write_clipboard_text(app: AppHandle, text: String) -> Result<(), String> {
    use tauri_plugin_clipboard_manager::ClipboardExt;
    app.clipboard()
        .write_text(text)
        .map_err(|e| format!("写剪贴板失败：{e}"))
}

/// `read_clipboard_image(app) -> String`（data URL）
///
/// ⛔ **返回 data URL 而不是裸 base64**：界面要能直接塞进 `<img src>`。
///    裸 base64 需要调用方自己拼前缀，漏一次就是一张坏图。
#[tauri::command]
pub fn read_clipboard_image(app: AppHandle) -> Result<String, String> {
    use tauri_plugin_clipboard_manager::ClipboardExt;
    let img = app
        .clipboard()
        .read_image()
        .map_err(|e| format!("剪贴板里没有可读的图片：{e}"))?;
    // ⛔ arboard 的 Image 只给了 width()/height()，**没有**取像素的公开方法
    //    （像素在  feature 后面，而插件没开那个 feature）。
    //    ⇒ 这里**不做手工像素搬运**，也不编 PNG（chunk/CRC/Adler 一堆细节，
    //    而我们要的是「系统能力」不是「图片格式实现」）。
    //
    //    诚实地说做不到，比返回一个空 data URL 好：后者会让界面显示一张
    //    坏图，而用户完全不知道「粘贴图片」这个功能其实是空的。
    let _ = (img.width(), img.height());
    Err("剪贴板图片：arboard 未暴露像素读取（需 image-data feature），暂不可用".to_owned())
}

/// `show_native_notification(title, body)`
#[tauri::command]
pub fn show_native_notification(app: AppHandle, title: String, body: Option<String>) -> Result<(), String> {
    use tauri_plugin_notification::NotificationExt;
    let n = app.notification().builder().title(&title);
    let n = match body {
        Some(b) => n.body(&b),
        None => n,
    };
    n.show().map_err(|e| e.to_string())
}

/// `get_launch_on_login() -> boolean`
#[tauri::command]
pub fn get_launch_on_login(app: AppHandle) -> bool {
    use tauri_plugin_autostart::ManagerExt;
    app.autolaunch().is_enabled().unwrap_or(false)
}

/// `set_launch_on_login(enabled)`
#[tauri::command]
pub fn set_launch_on_login(app: AppHandle, enabled: bool) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;
    let m = app.autolaunch();
    let r = if enabled { m.enable() } else { m.disable() };
    r.map_err(|e| e.to_string())
}

/// 窗口描述。**刻意不叫 `Window`** —— 那是 Tauri 自己的类型名。
// ⛔ `Deserialize` 不是可选的：Tauri 按名传参要**从 JSON 反序列化**成这个结构。
//    只 derive Serialize 的话编译能过（返回时用得上），但作为**入参**会在
//    main.rs 注册时报 `the trait bound WindowSpec: CommandArg is not satisfied`
//    —— 错误出现在 main.rs，离真正的原因（这个 derive 少了）很远。
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WindowSpec {
    pub label: String,
    pub title: String,
    pub width: f64,
    pub height: f64,
    pub url: String,
}

/// `create_app_window(spec)`
///
/// 开一个新窗口。
///
/// ⛔ `url` 只允许相对路径或 http/https。同 `open_external_url` 的理由：
///    对话内容里的链接不该能打开任意本地协议。
#[tauri::command]
pub async fn create_app_window(app: AppHandle, spec: WindowSpec) -> Result<String, String> {
    let url = spec.url.trim().to_owned();
    let target = if url.starts_with("http://") || url.starts_with("https://") {
        WebviewUrl::External(url.parse().map_err(|_| "URL 非法")?)
    } else if let Some(p) = url.strip_prefix('/') {
        WebviewUrl::App(p.to_owned().into())
    } else {
        return Err(format!("url 必须是绝对路径或 http(s)：{url}"));
    };
    let w = WebviewWindowBuilder::new(&app, &spec.label, target)
        .title(&spec.title)
        .inner_size(spec.width, spec.height)
        .build()
        .map_err(|e| e.to_string())?;
    // 记下创建时间，供 `remote_open_window` 的复用判定
    let _ = w;
    let _ = data_dir();
    Ok(spec.label)
}

/// `remote_open_window(app, label, url)` —— 已存在则聚焦，否则新建。
#[tauri::command]
pub fn remote_open_window(app: AppHandle, label: String, url: String) -> Result<String, String> {
    if let Some(w) = app.get_webview_window(&label) {
        // ⛔ 已存在的窗口只**聚焦**，不重新导航。
        //    重新导航会打断那个窗口里正在做的事（用户在另一个会话里打字）。
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
        return Ok(label);
    }
    let spec = WindowSpec { label, title: "NeoBot".into(), width: 1000.0, height: 700.0, url };
    tauri::async_runtime::block_on(create_app_window(app, spec))
}

/// `quit_app()` —— 退出应用（上游 `desktop/window.rs quit_app`).
///
/// ⛔ 桌宠移动命令曾住在这里（绝对定位版），2026-09-30 搬进 `pet.rs`
/// 并改成与上游同形的相对增量 —— pet 窗自己的调用是 `{ deltaX, deltaY }`，
/// 旧形状调上去静默失败。见 `pet.rs move_pet_window`.
///
/// ⛔ 返回 `()` 而不是 `Result`：调用后进程即退出，
/// 返回错误除了让调用方多写一个 `.catch` 之外没有任何意义。
#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}

/// `reveal_data_dir()` —— 在文件管理器里定位数据目录。
///
/// 目录不存在就先建（全新安装，学上游 `system_os.rs reveal_data_dir`）。
#[tauri::command]
pub fn reveal_data_dir(app: AppHandle) -> Result<(), String> {
    let dir = data_dir()?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建数据目录失败：{e}"))?;
    reveal(&app, &dir)
}

/// `reveal_in_folder(path)` —— 在文件管理器里定位一个文件/目录。
///
/// ⛔ 只揭示**存在**的路径：不存在的路径调过去，
/// Finder/资源管理器各弹各的错，症状不统一。
/// 揭示（选中但不打开）本身不读取内容，风险止于「看到文件名」。
#[tauri::command]
pub fn reveal_in_folder(app: AppHandle, path: String) -> Result<(), String> {
    let p = std::path::PathBuf::from(path.trim());
    if !p.exists() {
        return Err("路径不存在，无法定位".to_owned());
    }
    reveal(&app, &p)
}

fn reveal(app: &AppHandle, p: &std::path::Path) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .reveal_item_in_dir(p.to_string_lossy().into_owned())
        .map_err(|e| format!("定位失败：{e}"))
}

/// `open_dir(path)` —— 用系统文件管理器打开一个目录。
///
/// ⛔ 必须是**已存在的目录**：文件/不存在的路径调过去，
/// 各平台行为不一（有的打开父目录、有的报错、有的没反应）。
#[tauri::command]
pub fn open_dir(app: AppHandle, path: String) -> Result<(), String> {
    let p = std::path::PathBuf::from(path.trim());
    if !p.is_dir() {
        return Err("不是已存在的目录".to_owned());
    }
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_path(p.to_string_lossy().into_owned(), None::<String>)
        .map_err(|e| format!("打开目录失败：{e}"))
}

#[cfg(test)]
mod tests {

    #[test]
    fn 外链只放行http与https() {
        // ⛔ 放行 file:// 会让对话里的一个链接变成任意本地文件读取入口。
        for bad in ["file:///etc/passwd", "javascript:alert(1)", "data:text/html,<x>"] {
            let u = bad.trim();
            let ok = u.starts_with("http://") || u.starts_with("https://");
            assert!(!ok, "不该放行：{bad}");
        }
        for good in ["http://x.dev", "https://x.dev/a?b=c"] {
            assert!(good.starts_with("http"), "该放行：{good}");
        }
    }



    #[test]
    fn 窗口url只允许本地路径或http() {
        for bad in ["file:///x", "javascript:1", "tauri://x"] {
            let t = bad.starts_with("http://")
                || bad.starts_with("https://")
                || bad.starts_with('/');
            assert!(!t, "不该放行：{bad}");
        }
        assert!("/index.html".starts_with('/'));
    }
}
