//! status — 从 `entry/mod.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。


use super::info;

/// `json=true` 时直出 `ServerProxy::status()` 原始对象（与面板同源，零字段差）。
pub fn show_status(json: bool) {
    let status = neotrix::l1_action::nt_io::nt_io_proxy_server::ServerProxy::status();
    if json {
        // 机器可读通道：不染色、不套框，`serde_json` 紧凑单行（jq 友好）。
        println!("{}", status);
        return;
    }
    // ⚠️ 2026-10-05 迁移到 nt_term_viz::panel。
    //
    // 首版缺陷：**每一行的补空格数都不同**，而顶/底边是手数横线。
    // 实测（可见宽，ANSI 已剥）：
    //   │ Brain dimensions:  4 / 23 active   │  标签 18 列 + 固定补白
    //   │ Extensions:  0                     │
    //   │ Knowledge store:   123 bytes  │      ← 多一个空格
    //   │ KB graph:  12 nodes / 34 edges │
    //   顶边/底边 vis=42，两串横线互不相关
    // ⇒ 数值一变（`123` → `1234567`）补白就错位。
    //
    // 现按「标签 + 值」成行交给 render_panel：内容宽由最长行决定，
    // 数值变长时框自动变宽，不再需要手算补白。
    let nodes = status["knowledge_nodes"].as_i64().unwrap_or(0);
    let edges = status["knowledge_edges"].as_i64().unwrap_or(0);
    let rows = nt_term_viz::panel::render_panel(
        "NeoTrix Status",
        &[
            &format!(
                "{}  {} / {}  {}",
                info("Brain dimensions:"),
                status["brain_dims"].as_i64().unwrap_or(0),
                status["total_dims"].as_i64().unwrap_or(23),
                info("active")
            ),
            &format!(
                "{}  {}",
                info("Extensions:"),
                status["brain_extension"].as_i64().unwrap_or(0)
            ),
            &format!(
                "{}  {}  {}",
                info("Knowledge store:"),
                status["knowledge_store_bytes"].as_i64().unwrap_or(0),
                info("bytes")
            ),
            &format!(
                "{}  {} {} / {} {}",
                info("KB graph:"),
                nodes,
                info("nodes"),
                edges,
                info("edges")
            ),
        ],
    );
    for r in rows {
        // 染色整行（含边框）：宽度不受影响，且与首版视觉一致。
        println!("{}", info(r));
    }
}

pub fn generate_completions(shell: &str, cmd: &mut clap::Command) {
    use clap_complete::Shell;
    let shell = match shell {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        "powershell" => Shell::PowerShell,
        "elvish" => Shell::Elvish,
        other => {
            eprintln!(
                "error: unsupported shell '{}'. Use: bash, zsh, fish, powershell, elvish",
                other
            );
            std::process::exit(1);
        }
    };
    // EPIPE 容错：clap_complete 内部对写失败是 `.expect(...)`（实测
    // `neotrix completions bash | head` 直接 panic 出 BrokenPipe）。改为先在
    // 内存里生成完毕，再自己写 stdout —— 下游提前关管（head/管道截断）时
    // 按 Unix 惯例静默退 0，不把 panic 甩给用户。
    let mut buf: Vec<u8> = Vec::new();
    clap_complete::generate(shell, cmd, "neotrix", &mut buf);
    use std::io::Write as _;
    if let Err(err) = std::io::stdout().write_all(&buf) {
        if err.kind() == std::io::ErrorKind::BrokenPipe {
            std::process::exit(0);
        }
        eprintln!("error: failed to write completion: {err}");
        std::process::exit(1);
    }
}
