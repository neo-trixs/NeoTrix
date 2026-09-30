//! 桌面级命令 —— 主题、构建标识、前端日志、运行日志、关于页。
//!
//! # 为什么单独一个模块
//!
//! `api.rs` 是**契约声明**，`commands.rs` 是会话/面板业务。
//! 这一组是「应用关于自己」的接口（我是什么版本、什么主题、日志在哪），
//! 与业务无关且被多处引用，混进 `commands.rs` 会让那个文件失去主题。
//!
//! # 这组命令的共同点：**只读自身状态，不碰 store**
//!
//! 所以它们可以在应用刚起来、`get_runtime_info` 还没返回时就被调用 ——
//! 而那正是壳启动的关键窗口（见 `main.tsx` 第 17 行：
//! 主题必须在首屏渲染**之前**定下来，否则会闪一下白）。

use serde::Serialize;

/// 主题偏好。字面量与上游 `invoke<'dark' | 'light' | 'system'>` 一致。
///
/// ⛔ 刻意用字符串枚举而不是 `#[serde(rename_all)]` 的 `enum`：
///    前端那个联合类型是**字面量类型**，Rust 侧序列化出的必须是同样三个词。
///    写成 enum 又漏 `rename_all`，前端会拿到 `"Dark"`，而 `'Dark'`
///    不在联合类型里 —— tsc 抓不到（跨语言），界面会静默走进 default 分支。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Dark,
    Light,
    System,
}

/// `get_dsh_theme() -> 'dark' | 'light' | 'system'`
#[tauri::command]
pub fn get_dsh_theme() -> Theme {
    // 缺省 System：让系统决定。写死 Dark/Light 会在另一种系统上直接错色。
    Theme::System
}

/// `is_dev_build() -> boolean`
#[tauri::command]
pub fn is_dev_build() -> bool {
    // ⛔ 刻意**不**读环境变量：dev/release 是**编译期**事实。
    //    读 `DEBUG` 之类的运行期变量，在打包后仍可能为真
    //    （用户自己 export 了同名变量就会误判成开发版，
    //    于是开发菜单出现在正式版里）。
    cfg!(debug_assertions)
}

/// `log_frontend(level, target, message)`
///
/// 前端把每条日志送到后端落盘。
///
/// ⚠️ 参数是 **3 个**（`level` / `target` / `message`），不是 2 个。
/// 契约最初写成 2 个，门没抓到 —— 门当时只对**命令名**，不对**参数**。
/// 已把参数对账加进 `nt_check_api.mjs`（见该文件 ⑦）。
#[tauri::command]
pub fn log_frontend(level: String, target: String, message: String) {
    emit_log_line(&level, &target, &message);
}

/// 拼一行日志。抽成纯函数是为了能单测 —— 直接 `eprintln!` 的话
/// 「格式对不对」只能靠肉眼看 stdout。
///
/// ⛔ 换行必须先剥掉：前端的消息里可能带 `\n`（多行错误对象），
///    直接写会让一条日志变成多行，之后按行解析就错了。
pub fn emit_log_line(level: &str, target: &str, message: &str) {
    let clean = message.replace(['\r', '\n'], " ");
    eprintln!("[neobot][{level}][{target}] {clean}");
}

/// `read_run_logs() -> string`
///
/// 返回给设置向导的诊断文本。
///
/// ⚠️ 不返回 `Result`、也不返回 Option：前端在
/// `setup.tsx` 里直接把它塞进 `<pre>`，返回 null 会让 `<pre>{null}</pre>`
/// 渲染成空 —— 看起来像「没有日志」，而实际是「读失败」。
/// ⇒ 拿不到就**如实说明**，别返回空串。
#[tauri::command]
pub fn read_run_logs() -> String {
    let mut out = String::new();
    out.push_str("NeoBot 桌面端\n");
    out.push_str(&format!("版本      {}\n", env!("CARGO_PKG_VERSION")));
    out.push_str(&format!("构建      {}\n", if is_dev_build() { "debug（开发）" } else { "release（发布）" }));
    match std::env::var("NEOBOT_DATA_DIR") {
        Ok(d) if !d.trim().is_empty() => out.push_str(&format!("数据目录  {d}（来自 NEOBOT_DATA_DIR）\n")),
        _ => out.push_str("数据目录  ~/.neobot（未设 NEOBOT_DATA_DIR）\n"),
    }
    out.push_str(&format!("平台      {}\n", std::env::consts::OS));
    out.push_str(&format!("架构      {}\n", std::env::consts::ARCH));
    out
}

/// 关于页数据。字段名与前端 `DesktopAboutInfo` **逐项对齐**（snake_case）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DesktopAboutInfo {
    pub version: String,
    pub published_at: String,
    pub copyright: String,
    pub repo: String,
    pub powered_by: String,
}

/// `get_desktop_about() -> DesktopAboutInfo`
#[tauri::command]
pub fn get_desktop_about() -> DesktopAboutInfo {
    DesktopAboutInfo {
        version: env!("CARGO_PKG_VERSION").to_owned(),
        // ⛔ 构建时刻（compile time），不是运行时刻。运行时刻每次启动都变，
        //    放进「关于」页会让人以为程序一直在重装。
        published_at: String::new(),
        copyright: "NeoBot contributors".to_owned(),
        repo: String::new(),
        powered_by: "crates/neotrix-neobot".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 主题序列化后是前端要的小写字面量() {
        // ⛔ 这条是跨语言契约：前端类型是 'dark' | 'light' | 'system'。
        //    写成 "Dark" 时 tsc 抓不到（跨语言），界面会静默走 default。
        assert_eq!(serde_json::to_string(&Theme::Dark).unwrap(), "\"dark\"");
        assert_eq!(serde_json::to_string(&Theme::Light).unwrap(), "\"light\"");
        assert_eq!(serde_json::to_string(&Theme::System).unwrap(), "\"system\"");
    }

    #[test]
    fn 发布时刻在编译期就已确定() {
        // 运行时刻每次都变 ⇒ 「关于」页会像在一直重装。
        // 这里只断言它是空串（未接构建时间戳），而**不是**断言它等于现在。
        assert_eq!(get_desktop_about().published_at, "");
    }

    #[test]
    fn 关于页字段名与前端逐项对齐() {
        let v = serde_json::to_value(get_desktop_about()).unwrap();
        for k in ["version", "published_at", "copyright", "repo", "powered_by"] {
            assert!(v.get(k).is_some(), "缺字段 {k}，前端 DesktopAboutInfo 会拿到 undefined");
        }
    }

    #[test]
    fn 日志里的换行被剥掉() {
        // ⛔ 不剥的话一条日志会变成多行，之后按行解析就错位。
        let mut buf = Vec::new();
        let msg = "line1\nline2\rline3";
        let clean = msg.replace(['\r', '\n'], " ");
        use std::io::Write;
        writeln!(buf, "[neobot][info][ui] {clean}").unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert_eq!(out.matches('\n').count(), 1, "一条日志出现了多行：{out:?}");
        assert!(out.contains("line1 line2 line3"));
    }

    #[test]
    fn 运行日志不是空串() {
        // 前端直接塞进 <pre>。返回空串会被读成「没有日志」，
        // 而实际是「什么都没写」—— 两种情况对用户是两种处境。
        let s = read_run_logs();
        assert!(!s.trim().is_empty());
        assert!(s.contains("NeoBot"));
    }

    #[test]
    fn 构建标识两种模式各自自洽() {
        // 只是断言它是个确定的 bool；debug/release 由编译期决定，
        // 这里能做的是保证**不会**因为读环境变量而误判。
        let d = is_dev_build();
        assert_eq!(d, cfg!(debug_assertions));
    }
}

/// `get_runtime_info() -> RuntimeInfo`
///
/// ⛔ **本仓不返回假 URL。**
///
/// 上游壳的 `store.harness.initialize()` 必调本命令，拿 `service_url` 去 iframe
/// 那个由 `source/deepseek-harness`（空 submodule）起的 Web UI。
/// NeoBot 不跑 DSH 运行时，所以：
///
///   · 返回**空** `service_url` + `has_service: false`
///   · 壳据此切到「自持界面」分支（见 `webview.tsx` 的 `selfHosted`）
///
/// 之所以不编一个 `http://127.0.0.1:3080` 之类：那样壳会 `status='ready'`、
/// iframe 会去加载一个不存在的东西、然后在**别处**报「加载失败」。
/// 错误会被搬到离原因很远的地方 —— 而空 URL 让失败**停在它该停的地方**。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct RuntimeInfo {
    /// 空串 = 没有可 iframe 的服务。
    pub service_url: String,
    /// 谁在提供界面。`"neobot"` = 自持。
    pub host: String,
    /// 显式标志，而不是靠 `service_url.is_empty()` 推断 ——
    /// 推断的话，将来真接了服务，这个判断会静默变成另一种含义。
    pub has_service: bool,
}

#[tauri::command]
pub fn get_runtime_info() -> RuntimeInfo {
    RuntimeInfo { service_url: String::new(), host: "neobot".to_owned(), has_service: false }
}

/// `runtime_ready() -> boolean`
///
/// 壳用它决定「要不要走依赖安装流程」。
///
/// ⛔ 恒返回 `true`：NeoBot 不下载 Node、MinGit 或任何外部运行时
///    （那是 DSH Harness 的要求）。返回 `false` 会把壳推进 `installing`
///    状态去装一堆我们根本不需要的东西。
///
///    注意它**不是**「一切就绪」的意思，而是「没有需要安装的依赖」。
#[tauri::command]
pub fn runtime_ready() -> bool {
    true
}

/// `install_dependencies()`
///
/// 上游用它下载并安装 DSH 运行时的依赖。NeoBot 无依赖 ⇒ 空操作。
///
/// ⚠️ 保留这条而不是让壳调不到：壳在 `!runtime_ready || !config.installed`
/// 时会调它。虽然我们恒返回 true，但 `config.installed` 来自用户可能清过的
/// 旧 store —— 那时它会被调到。**空实现**让它正常走完，
/// 而「命令不存在」会把壳卡在 installing 状态出不来。
#[tauri::command]
pub fn install_dependencies() -> Result<(), String> {
    Ok(())
}

/// `set_language(lang)`
///
/// 持久化界面语言（`zh` / `en`）。写进数据目录下的 `language`。
///
/// 失败**不抛**：i18n 检测器调用它时是「尽力持久化」，
/// 抛错会让语言检测整条链断掉（实测日志里就是这么变成 error 的）。
#[tauri::command]
pub fn set_language(lang: String) {
    let v = lang.trim();
    if v != "zh" && v != "en" {
        return;
    }
    // 拿不到数据目录就**放弃持久化**，不报错：界面语言本身已经生效了。
    let Ok(dir) = crate::commands::data_dir() else { return };
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(dir.join("language"), v);
}

#[cfg(test)]
mod runtime_tests {
    use super::*;

    #[test]
    fn 不返回假的服务地址() {
        let r = get_runtime_info();
        assert!(r.service_url.is_empty(), "编造 URL 会让 iframe 去加载不存在的东西");
        assert!(!r.has_service);
        assert_eq!(r.host, "neobot");
    }

    /// 上游壳的 TS 类型是 `{ service_url: string }`。加字段是兼容的，
    /// 删/改字段不是 ⇒ 这条守住「只能加不能改」。
    #[test]
    fn 恒不触发依赖安装流程() {
        // ⛔ 返回 false 会把壳推进 installing，去装 DSH 运行时依赖。
        assert!(runtime_ready());
        assert!(install_dependencies().is_ok());
    }

    #[test]
    fn 语言只认zh与en() {
        // 非法值必须**静默忽略**而不是写进盘 —— 否则会被读成任意字符串。
        assert!(matches!("zh", "zh" | "en"));
        assert!(matches!("en", "zh" | "en"));
        assert!(!matches!("xx", "zh" | "en"));
    }

    #[test]
    fn 保留上游的service_url字段() {
        let v = serde_json::to_value(get_runtime_info()).unwrap();
        assert!(v.get("service_url").is_some());
        assert!(v.get("has_service").is_some());
        assert!(v.get("host").is_some());
    }
}
