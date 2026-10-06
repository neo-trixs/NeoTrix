//! `neotrix web fetch` —— 用 NeoTrix 自带的 `UniversalBrowser` 抓一个 URL.
//!
//! **本文件是「补缺口」，不是新能力。**
//! `UniversalBrowser::fetch()` 早已存在于
//! `neotrix::l1_action::nt_io::universal_browser`，但此前
//! **没有任何 CLI 或 example 能触达它**
//! （`rg 'UniversalBrowser' neotrix-core/src/bin crates/*/src/bin` ⇒ 零命中）
//! ⇒ **能力等于不存在**。
//!
//! ⚠️ **凭据边界（重要，请先读）**
//! · 本命令**只发匿名请求** —— 不读 Chrome、不导 cookie、不碰本机凭据。
//! · 因此需要登录的站点（飞书 / Google / Meta 等）**只会拿到登录墙**。
//!   这是**预期行为**，不是 bug：不该由工具悄悄替你带上你的身份。
//! · `BrowserResult.cookies` 是**响应带来的**，不是本机的；打印时
//!   `CookieEntry` 的 `Debug` 已 redact（`value` ⇒ `<redacted>`），
//!   故本命令**不会**把会话值写进终端或日志。
//! · 若确需带自己的会话，请**自行**把 cookie 放进
//!   `~/.neotrix/cookies/<id>.json` —— 那是**凭据操作，应由本人执行**。
//!
//! ⛔ 历史上这份能力曾被误放到 `neotrix-neobot` 的 CLI：
//! 该 crate **刻意不依赖 `neotrix-core`**（它按「同版本」镜像依赖而非依赖），
//! 放进去会**破坏这条架构边界**，且编译直接失败
//! （`unresolved module or unlinked crate 'neotrix'`）。
//! ⇒ 能力在 `neotrix-core`，其 CLI 出口就属于 `neotrix`。

use neotrix::l1_action::nt_io::universal_browser::{PlatformConfig, UniversalBrowser};

/// `neotrix web fetch <url>`
pub fn run_web_fetch(url: &str, quiet: bool) {
    let trimmed = url.trim();
    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        eprintln!("⛔ URL 必须以 http:// 或 https:// 开头：{trimmed}");
        std::process::exit(2);
    }

    // `neotrix` 主二进制此前没有 async 运行时先例 ⇒ 按需建一个 current-thread。
    let rt = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("⛔ 建 tokio 运行时失败：{e}");
            std::process::exit(1);
        }
    };

    // ⚠️ `fetch` 内部会 `new_page()`，它要求浏览器**已launch**
    //（否则返回 `Err("Browser not launched")` —— 本命令首次实现时实测踩到）。
    // `launch` 取 `&mut self` 且需要 `PlatformConfig` ⇒ 此处必须用 `mut`。
    let mut browser = UniversalBrowser::new();
    if let Err(e) = rt.block_on(browser.launch(PlatformConfig::fetch())) {
        eprintln!(
            "⛔ 启动浏览器失败：{e}\n\
             ⭐ 首次启动可能需要下载 Chromium；若被沙箱/无显示环境挡住，\n\
               可改用已运行的浏览器后端（见 docs/LOCAL-LLAMA 与 browser_engine 配置）。"
        );
        std::process::exit(1);
    }

    let result = match rt.block_on(browser.fetch(trimmed)) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("⛔ 抓取失败：{e}");
            std::process::exit(1);
        }
    };

    if !quiet {
        eprintln!(
            "[neotrix] fetch {trimmed} => success={} duration={}ms{}",
            result.success,
            result.duration_ms,
            if result.cookies.is_empty() {
                String::new()
            } else {
                // 只报**条数**，绝不打印内容（CookieEntry 的 Debug 已 redact）
                format!("（响应携带 {} 条 cookie）", result.cookies.len())
            }
        );
    }

    match (result.success, result.content) {
        (true, Some(content)) => println!("{content}"),
        (true, None) => {
            eprintln!(
                "⛔ 抓取成功但正文为空。\n\
                 ⭐ 最可能的原因：**该 URL 需要登录**，而本命令**只发匿名请求**。\n\
                 \x20 若是 SPA 外壳，则页面正文由 JS 渲染，静态抓取拿不到。\n\
                 \x20 需要自己的会话时，请**自行**把 cookie 放进 ~/.neotrix/cookies/<id>.json。"
            );
            std::process::exit(1);
        }
        (false, _) => {
            eprintln!(
                "⛔ 抓取失败：{}",
                result.error.unwrap_or_else(|| "未提供错误信息".to_string())
            );
            std::process::exit(1);
        }
    }
}
