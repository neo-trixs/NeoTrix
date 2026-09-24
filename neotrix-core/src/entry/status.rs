//! status — 从 `entry/mod.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。


use super::info;

pub fn show_status() {
    let status = neotrix::l1_action::nt_io::nt_io_proxy_server::ServerProxy::status();
    println!("{}", info("╭─ NeoTrix Status ───────────────────────╮"));
    println!(
        "│ {}  {:<2} / {:<2} {}   │",
        info("Brain dimensions:"),
        status["brain_dims"].as_i64().unwrap_or(0),
        status["total_dims"].as_i64().unwrap_or(23),
        info("active")
    );
    println!(
        "│ {}  {:<4}               │",
        info("Extensions:"),
        status["brain_extension"].as_i64().unwrap_or(0)
    );
    println!(
        "│ {}   {:<8} {}  │",
        info("Knowledge store:"),
        status["knowledge_store_bytes"].as_i64().unwrap_or(0),
        info("bytes")
    );
    let nodes = status["knowledge_nodes"].as_i64().unwrap_or(0);
    let edges = status["knowledge_edges"].as_i64().unwrap_or(0);
    println!(
        "│ {}  {:<6} {} / {:<6} {} │",
        info("KB graph:"),
        nodes,
        info("nodes"),
        edges,
        info("edges")
    );
    println!("{}", info("╰─────────────────────────────────────────╯"));
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
    let mut stdout = std::io::stdout();
    clap_complete::generate(shell, cmd, "neotrix", &mut stdout);
}
