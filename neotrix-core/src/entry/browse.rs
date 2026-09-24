//! browse — 从 `entry/mod.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。


use colored::Colorize;
use super::{err, info, success, warn};

pub fn run_browse(url: &str) {
    use neotrix::l1_action::nt_io::nt_io_browser_engine::{
        AuthConfig, BackendKind, BrowserAction, BrowserConfig, BrowserEngine,
    };
    // 后端选择（默认 Http，行为不变）：
    // NT_BROWSE_BACKEND=http|chrome|cdp|mock；Cdp 需已登录 profile 接管时配 NT_BROWSE_PROFILE。
    let backend = std::env::var("NT_BROWSE_BACKEND")
        .ok()
        .map(|s| {
            let name = s.trim().to_ascii_lowercase();
            match name.as_str() {
                "mock" => BackendKind::Mock,
                "chrome" | "chrome-headless" | "headless" => BackendKind::ChromeHeadless,
                "cdp" => BackendKind::Cdp,
                _ => BackendKind::Http,
            }
        })
        .unwrap_or(BackendKind::Http);
    let profile = std::env::var("NT_BROWSE_PROFILE")
        .ok()
        .filter(|s| !s.trim().is_empty());
    println!("{}", info("╭─ NeoTrix Browser ──────────────────────────╮"));
    println!("│ {} {}", info("Fetching:"), url);
    println!("│ {} {:?}", info("Backend:"), backend);
    if backend == BackendKind::Cdp {
        match profile.as_deref() {
            Some(p) => println!("│ {} {} (先退出占用它的 Chrome)", info("Profile:"), p),
            None => println!("│ {}", info("Profile: fresh (未登录态)")),
        }
    }
    println!(
        "{}",
        info("╰────────────────────────────────────────────────╯")
    );
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("{}: runtime: {}", err("Error"), e);
            return;
        }
    };
    rt.block_on(async {
        let engine = BrowserEngine::new(BrowserConfig {
            backend,
            profile_dir: profile,
            timeout_ms: 30_000,
            ..Default::default()
        });
        let session_id = match engine.create_session().await {
            Ok(id) => id,
            Err(e) => {
                eprintln!("{}: {}", err("Error"), e);
                return;
            }
        };
        // 认证（可选）：优先级 站点名 > token 文件 > 内存 token。
        // 例：NEOTRIX_AUTH_SITE=lingee neotrix browse <url>
        // 例：NEOTRIX_AUTH_TOKEN_FILE=~/.config/neotrix/lingee.token neotrix browse <url>
        let auth_site = std::env::var("NEOTRIX_AUTH_SITE")
            .ok()
            .filter(|s| !s.trim().is_empty());
        let token_file = std::env::var("NEOTRIX_AUTH_TOKEN_FILE")
            .ok()
            .filter(|s| !s.trim().is_empty());
        let token_literal = std::env::var("NEOTRIX_AUTH_TOKEN")
            .ok()
            .filter(|s| !s.trim().is_empty());
        if let Some(site) = auth_site {
            if let Err(e) = engine
                .set_session_auth_by_site(&session_id, site.trim())
                .await
            {
                eprintln!("{}: auth: {}", err("Error"), e);
                return;
            }
            println!("│ {} {}", info("Auth:"), info("site (auth.toml)"));
        } else if let Some(path) = token_file {
            // 默认 7 天有效期的站（如 Lingee）可直接用；已知过期点可再配
            if let Err(e) = engine
                .set_session_auth(&session_id, AuthConfig::file(path))
                .await
            {
                eprintln!("{}: auth: {}", err("Error"), e);
                return;
            }
            println!("│ {} {}", info("Auth:"), info("token file (hot-reload)"));
        } else if let Some(token) = token_literal {
            if let Err(e) = engine
                .set_session_auth(&session_id, AuthConfig::literal(token))
                .await
            {
                eprintln!("{}: auth: {}", err("Error"), e);
                return;
            }
            println!("│ {} {}", info("Auth:"), info("inline token"));
        }
        match engine
            .execute(
                &session_id,
                BrowserAction::Navigate {
                    url: url.to_string(),
                },
            )
            .await
        {
            Ok(result) if result.success => {
                let lines: Vec<&str> = result.output.lines().collect();
                if let Some(title) = result.title {
                    println!("{} {}", info("Title:"), title);
                }
                println!(
                    "\n{} ({} lines, ~{} chars):",
                    info("Content"),
                    lines.len(),
                    result.output.len()
                );
                for line in lines.iter().take(60) {
                    println!("  {}", line);
                }
                if lines.len() > 60 {
                    println!(
                        "  {} ({})",
                        info("..."),
                        info(format!("{} more lines", lines.len() - 60))
                    );
                }
            }
            Ok(result) => eprintln!(
                "{}: {}",
                err("Error"),
                result.error.unwrap_or_else(|| "unknown".to_string())
            ),
            Err(e) => eprintln!("{}: {}", err("Error"), e),
        }
    });
}

/// 按 JSON 动作文件执行连招（Colab 自动操作等场景）：
/// `NT_BROWSE_BACKEND=cdp NT_BROWSE_PROFILE=<dir> neotrix browse-act colab.json`
/// 文件为 BrowserAction 数组（serde 外部标记），例：
/// `[{"Navigate":{"url":"https://colab.research.google.com/"}},{"WaitForElement":{"selector":"...","timeout_ms":30000}}]`
/// 首个失败即停（fail-closed），全程打印进度。
pub fn run_browse_act(path: &str) {
    use neotrix::l1_action::nt_io::nt_io_browser_engine::{
        BackendKind, BrowserAction, BrowserConfig, BrowserEngine,
    };
    let backend = std::env::var("NT_BROWSE_BACKEND")
        .ok()
        .map(|s| {
            let name = s.trim().to_ascii_lowercase();
            match name.as_str() {
                "mock" => BackendKind::Mock,
                "chrome" | "chrome-headless" | "headless" => BackendKind::ChromeHeadless,
                "cdp" => BackendKind::Cdp,
                _ => BackendKind::Http,
            }
        })
        .unwrap_or(BackendKind::Http);
    let profile = std::env::var("NT_BROWSE_PROFILE")
        .ok()
        .filter(|s| !s.trim().is_empty());
    println!(
        "{}",
        info("╭─ NeoTrix Browser Acts ───────────────────────╮")
    );
    println!("│ {} {}", info("File:"), path);
    println!("│ {} {:?}", info("Backend:"), backend);
    println!(
        "{}",
        info("╰────────────────────────────────────────────────╯")
    );
    let data = match std::fs::read_to_string(path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{}: read {}: {}", err("Error"), path, e);
            return;
        }
    };
    let actions: Vec<BrowserAction> = match serde_json::from_str(&data) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}: parse {}: {}", err("Error"), path, e);
            return;
        }
    };
    if actions.is_empty() {
        eprintln!("{}: no actions in {}", err("Error"), path);
        return;
    }
    println!("│ {} {}", info("Actions:"), actions.len());
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("{}: runtime: {}", err("Error"), e);
            return;
        }
    };
    rt.block_on(async {
        let engine = BrowserEngine::new(BrowserConfig {
            backend,
            profile_dir: profile,
            timeout_ms: 30_000,
            ..Default::default()
        });
        let session_id = match engine.create_session().await {
            Ok(id) => id,
            Err(e) => {
                eprintln!("{}: {}", err("Error"), e);
                return;
            }
        };
        let mut done = 0usize;
        for (i, action) in actions.iter().enumerate() {
            let label = format!("{action:?}");
            let label = label.chars().take(120).collect::<String>();
            match engine.execute(&session_id, action.clone()).await {
                Ok(result) if result.success => {
                    done += 1;
                    let out_len = result.output.len();
                    println!("{} [{i}] ok {label} ({out_len} chars)", info("ACT"));
                    // 调试可见：GetContent/GetText 成功时打印前 3000 字符（长页面截断）
                    if out_len > 0 {
                        let shown: String = result.output.chars().take(3000).collect();
                        println!("{shown}");
                        if out_len > 3000 {
                            println!("{} ({} more chars)", info("..."), out_len - 3000);
                        }
                    }
                }
                Ok(result) => {
                    eprintln!(
                        "{}: [{i}] {label}: {}",
                        err("Error"),
                        result.error.unwrap_or_else(|| "unknown".to_string())
                    );
                    break;
                }
                Err(e) => {
                    eprintln!("{}: [{i}] {label}: {e}", err("Error"));
                    break;
                }
            }
        }
        println!("{} {done}/{} actions ok", info("Done:"), actions.len());
    });
}

pub fn run_search(query: &str, count: usize) {
    use neotrix::l2_perception::nt_world::nt_world_search::UnifiedSearch;

    let engine = UnifiedSearch::new();
    println!("{} Searching for: {}", info("🔍"), query);
    println!();

    match engine.search(query, count) {
        Ok(results) => {
            if results.is_empty() {
                println!("{} No results found.", warn("ℹ️"));
                return;
            }
            println!("{}", info(format!("Found {} results:\n", results.len())));
            for (i, result) in results.iter().enumerate() {
                println!("{}. {}", info(format!("{}", i + 1)), result.title.bold());
                println!("   {}", result.url.blue().underline());
                println!("   {}", result.snippet);
                println!();
            }
        }
        Err(e) => {
            eprintln!("{} {}", err("❌ Search error:"), e);
        }
    }
}

pub fn run_login(url: &str) {
    use neotrix::l2_perception::nt_world::nt_world_crawl::BrowserCircuit;
    println!("{}", info("╭─ NeoTrix Login ────────────────────────────╮"));
    println!("│ {}: {}", info("URL"), url);
    println!("│ {}", info("A Chrome window will open. Log in, then"));
    println!("│ {}", info("close the window to save the session."));
    println!(
        "{}",
        info("╰─────────────────────────────────────────────╯")
    );
    let browser = BrowserCircuit::new();
    match browser.login(url) {
        Ok(_) => println!("{}", success("✅ Login session saved.")),
        Err(e) => eprintln!("{}", err(format!("❌ Login error: {}", e))),
    }
}
