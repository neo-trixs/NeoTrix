//! sysops — NeoTrix 系统运维统一入口 (NT-REPAIR 域执行层)
//!
//! 由分散 sh 脚本 Rust 化而来 (cycle 207 整合方案):
//!   - install-daemon.sh / uninstall-daemon.sh → daemons install/uninstall
//!   - kb-guard.sh / kb-backup.sh             → guard/backup (复用 nt_mind_guard)
//!   - uninstall.sh                           → uninstall (KbGuard 备份前置防删库)
//!
//! 设计原则:
//!   - launchd plist 的 ProgramArguments 指向当前二进制自身 (current_exe),
//!     不硬编码路径 (修复 deploy/com.neotrix.proxy-daemon.plist 的 3tfire 断链)
//!   - 所有操作幂等: 重复 install 不报错, 重复 uninstall 不报错
//!   - uninstall 前先备份 KB, 拒绝无保护删除

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use colored::Colorize;

fn info(msg: impl AsRef<str>) -> String {
    msg.as_ref().blue().to_string()
}
fn success(msg: impl AsRef<str>) -> String {
    msg.as_ref().green().to_string()
}
fn warn(msg: impl AsRef<str>) -> String {
    msg.as_ref().yellow().to_string()
}
fn err(msg: impl AsRef<str>) -> String {
    msg.as_ref().red().to_string()
}

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn launch_agents_dir() -> PathBuf {
    home_dir().join("Library/LaunchAgents")
}

fn kb_guard_plist_path() -> PathBuf {
    launch_agents_dir().join("com.neotrix.kb-guard.plist")
}

fn kb_backup_plist_path() -> PathBuf {
    launch_agents_dir().join("com.neotrix.kb-backup.plist")
}

fn backup_root() -> PathBuf {
    home_dir().join("Library/Application Support/NeoTrix/backups")
}

/// 当前二进制绝对路径 (plist ProgramArguments 用)
fn current_exe() -> PathBuf {
    std::env::current_exe().unwrap_or_else(|_| PathBuf::from("neotrix"))
}

fn plist(label: &str, args: &[&str], interval_secs: u64, log_path: &Path) -> String {
    let args_xml: String = args
        .iter()
        .map(|a| format!("        <string>{}</string>", a))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
{args_xml}
    </array>
    <key>StartInterval</key>
    <integer>{interval_secs}</integer>
    <key>RunAtLoad</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{log}</string>
    <key>StandardErrorPath</key>
    <string>{log}</string>
</dict>
</plist>
"#,
        label = label,
        args_xml = args_xml,
        interval_secs = interval_secs,
        log = log_path.display()
    )
}

/// 当前用户 UID (launchctl gui 域需要)。UID 环境变量在非 shell 环境可能未导出,
/// 用 `id -u` 兜底。
fn current_uid() -> String {
    if let Ok(uid) = std::env::var("UID") {
        if !uid.is_empty() {
            return uid;
        }
    }
    Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "0".into())
}

fn bootout(label: &str) {
    let uid = current_uid();
    let _ = Command::new("launchctl")
        .arg("bootout")
        .arg(format!("gui/{}/{}", uid, label))
        .output();
}

fn bootstrap(plist: &Path) -> Result<(), String> {
    let uid = current_uid();
    let out = Command::new("launchctl")
        .arg("bootstrap")
        .arg(format!("gui/{}", uid))
        .arg(plist)
        .output()
        .map_err(|e| format!("launchctl bootstrap: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        let msg = String::from_utf8_lossy(&out.stderr).trim().to_string();
        // bootstrap 已存在时返回 5 (Operation already in progress), 幂等视为成功
        if msg.contains("already") || msg.contains("in progress") {
            Ok(())
        } else {
            Err(format!("launchctl bootstrap: {msg}"))
        }
    }
}

/// 子命令: guard — 手动运行一次 KB 守卫 (备份+健康检查)
fn cmd_guard() {
    use neotrix::neotrix::l8_autonomic_impl::nt_mind_guard::KbGuard;
    let guard = KbGuard::default();
    let report = guard.guard();
    if report.healthy {
        println!("{} KB 健康", success("✓"));
    } else if report.restored {
        println!("{} KB 已从备份恢复", warn("↻"));
    } else {
        println!("{} KB 不健康且无可用备份", err("✗"));
    }
}

/// 子命令: backup — 手动执行一次 KB 快照备份 (并发写入下带重试)
fn cmd_backup() {
    use neotrix::neotrix::l8_autonomic_impl::nt_mind_guard::KbGuard;
    let guard = KbGuard::default();
    let mut last_err = String::new();
    for attempt in 0..3 {
        match guard.backup() {
            Ok(dst) => {
                println!("{} 备份完成: {}", success("✓"), dst.display());
                return;
            }
            Err(e) => {
                last_err = e;
                if attempt < 2 {
                    std::thread::sleep(std::time::Duration::from_millis(500 * (attempt + 1)));
                }
            }
        }
    }
    eprintln!("{} 备份失败 (重试3次): {}", err("✗"), last_err);
}

/// 子命令: status — KB 健康 + 备份 + daemon 状态总览
fn cmd_status() {
    use neotrix::neotrix::l8_autonomic_impl::nt_mind_guard::db_healthy;
    let kb = home_dir().join(".neotrix/knowledge.db");
    let healthy = db_healthy(&kb);
    println!("╭─ NeoTrix SysOps ─────────────────────────╮");
    println!(
        "│ KB   {}  {}",
        if healthy { "✓ 健康".green() } else { "✗ 异常".red() },
        kb.display()
    );
    let bdir = backup_root();
    let backups: Vec<_> = fs::read_dir(&bdir)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| {
            let n = e.file_name().to_string_lossy().into_owned();
            n.starts_with("knowledge-") && n.ends_with(".db")
        })
        .collect();
    println!(
        "│      {} 份备份 @ {}",
        backups.len(),
        bdir.display()
    );
    for (name, path) in [
        ("kb-guard", kb_guard_plist_path()),
        ("kb-backup", kb_backup_plist_path()),
    ] {
        let installed = path.is_file();
        println!(
            "│ {}  {}  {}",
            if installed {
                "✓".green()
            } else {
                "·".dimmed()
            },
            name.to_string().blue(),
            if installed { "installed" } else { "not installed" }
        );
    }
    println!("╰───────────────────────────────────────────╯");
}

/// 子命令: daemons — launchd 任务安装/卸载 (幂等)
fn cmd_daemons(sub: &str) {
    match sub {
        "install" | "setup" | "on" => {
            let exe = current_exe();
            let exe_str = exe.to_string_lossy().into_owned();
            let fs = launch_agents_dir();
            if let Err(e) = fs::create_dir_all(&fs) {
                eprintln!("{} {}", err("Error:"), e);
                return;
            }
            let log_dir = home_dir().join(".neotrix/logs");
            let _ = fs::create_dir_all(&log_dir);

            let guard_plist = plist(
                "com.neotrix.kb-guard",
                &[exe_str.as_str(), "sysops", "guard"],
                600,
                &log_dir.join("kb-guard.log"),
            );
            let backup_plist = plist(
                "com.neotrix.kb-backup",
                &[exe_str.as_str(), "sysops", "backup"],
                21600,
                &log_dir.join("kb-backup.log"),
            );
            fs::write(kb_guard_plist_path(), &guard_plist)
                .map_err(|e| eprintln!("{} write plist: {}", err("Error:"), e))
                .ok();
            fs::write(kb_backup_plist_path(), &backup_plist)
                .map_err(|e| eprintln!("{} write plist: {}", err("Error:"), e))
                .ok();
            bootout("com.neotrix.kb-guard");
            bootout("com.neotrix.kb-backup");
            match bootstrap(&kb_guard_plist_path()) {
                Ok(()) => println!("{} kb-guard daemon installed (600s)", success("✓")),
                Err(e) => eprintln!("{} {}", err("Error:"), e),
            }
            match bootstrap(&kb_backup_plist_path()) {
                Ok(()) => println!("{} kb-backup daemon installed (6h)", success("✓")),
                Err(e) => eprintln!("{} {}", err("Error:"), e),
            }
            println!(
                "{} ProgramArguments 指向 {}",
                info("→"),
                exe.display()
            );
        }
        "uninstall" | "remove" | "off" => {
            bootout("com.neotrix.kb-guard");
            bootout("com.neotrix.kb-backup");
            for p in [kb_guard_plist_path(), kb_backup_plist_path()] {
                if p.is_file() {
                    let _ = fs::remove_file(&p);
                    println!("{} removed {}", success("✓"), p.display());
                }
            }
            println!("{} kb daemons uninstalled", success("✓"));
        }
        "status" => {
            cmd_status();
        }
        other => {
            eprintln!(
                "{} unknown daemons subcommand: {} (expected: install|uninstall|status)",
                err("Error:"),
                other
            );
        }
    }
}

/// 子命令: uninstall — 安全卸载 (先备份 KB, 再删 ~/.neotrix)
fn cmd_uninstall(force: bool) {
    use neotrix::neotrix::l8_autonomic_impl::nt_mind_guard::KbGuard;
    let guard = KbGuard::default();

    println!("{} 卸载前先备份 KB...", info("→"));
    let mut last_err = String::new();
    let mut backed_up = false;
    for attempt in 0..3 {
        match guard.backup() {
            Ok(dst) => {
                println!("{} KB 备份: {}", success("✓"), dst.display());
                backed_up = true;
                break;
            }
            Err(e) => {
                last_err = e;
                if attempt < 2 {
                    std::thread::sleep(std::time::Duration::from_millis(500 * (attempt + 1)));
                }
            }
        }
    }
    if !backed_up {
        if force {
            eprintln!("{} 备份失败但 --force: {}", warn("⚠"), last_err);
        } else {
            eprintln!(
                "{} 备份失败 (重试3次), 中止卸载 (用 --force 覆盖): {}",
                err("✗"),
                last_err
            );
            return;
        }
    }

    cmd_daemons("uninstall");

    let neotrix_dir = home_dir().join(".neotrix");
    println!("{} 删除 {}", warn("→"), neotrix_dir.display());
    match fs::remove_dir_all(&neotrix_dir) {
        Ok(()) => println!("{} removed", success("✓")),
        Err(e) => {
            if force {
                eprintln!("{} 删除失败 (--force 忽略): {e}", warn("⚠"));
            } else {
                eprintln!("{} 删除失败: {e}", err("✗"));
            }
        }
    }
    println!("{} NeoTrix 已卸载 (备份保留在 {})", success("✓"), backup_root().display());
}

/// sysops 主入口: 分发到子命令
pub fn run_sysops(args: &[String]) {
    let sub = args.first().map(String::as_str).unwrap_or("help");
    match sub {
        "help" | "--help" | "-h" => {
            println!(
                "{} NeoTrix SysOps — 统一运维入口 (替代分散 sh 脚本)\n\
                 \n\
                 \x20 usage: neotrix sysops <subcommand>\n\
                 \n\
                 \x20   guard                手动运行一次 KB 守卫 (备份+健康检查)\n\
                 \x20   backup               手动执行一次 KB 快照备份\n\
                 \x20   status               显示 KB 健康/备份/daemon 状态\n\
                 \x20   daemons install     安装 kb-guard + kb-backup launchd 任务 (幂等)\n\
                 \x20   daemons uninstall   卸载 launchd 任务\n\
                 \x20   uninstall [--force] 安全卸载 (先备份 KB 防数据丢失)",
                info("╭─")
            );
        }
        "guard" => cmd_guard(),
        "backup" => cmd_backup(),
        "status" => cmd_status(),
        "daemons" | "daemon" => {
            let subsub = args.get(1).map(String::as_str).unwrap_or("status");
            cmd_daemons(subsub);
        }
        "uninstall" => {
            let force = args.iter().any(|a| a == "--force");
            cmd_uninstall(force);
        }
        other => {
            eprintln!("{} unknown sysops subcommand: {}", err("Error:"), other);
            run_sysops(&[]);
        }
    }
}
