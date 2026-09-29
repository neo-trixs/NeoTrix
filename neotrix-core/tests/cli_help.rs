//! CLI 集成测试 — 帮助系统 (对标主流 CLI: --help / --version / help 子命令)
//!
//! 真实运行 `neotrix` 二进制, 验证帮助输出完整性与格式。

use assert_cmd::Command;
use predicates::prelude::*;

/// 所有顶层命令名 — 与 main.rs `Commands` 枚举保持一致。
/// 新增命令时必须同步更新此列表 (测试即契约)。
const ALL_COMMANDS: &[&str] = &[
    "exec",
    "run",
    "serve",
    "reason",
    "mcp-server",
    "evidence",
    "wiki",
    "todo",
    "search",
    "bench",
    "status",
    "daemon",
    "update",
    "completions",
    "features",
    "config",
    "sysops",
    "browse",
    "login",
    "proxy",
    "discover",
    "sandbox",
    "wallet",
];

#[test]
fn help_lists_all_commands() {
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    let out = cmd.arg("--help").output().unwrap();
    assert!(out.status.success(), "exit={:?}", out.status.code());
    let stdout = String::from_utf8_lossy(&out.stdout);
    for name in ALL_COMMANDS {
        assert!(stdout.contains(name), "help output missing command: {name}");
    }
}

#[test]
fn help_has_usage_and_options() {
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: neotrix"))
        .stdout(predicate::str::contains("--color"))
        .stdout(predicate::str::contains("--profile"))
        .stdout(predicate::str::contains("--help"))
        .stdout(predicate::str::contains("--version"));
}

#[test]
fn version_format_is_semver() {
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"^neotrix \d+\.\d+\.\d+").unwrap());
}

#[test]
fn version_short_flag_works() {
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    cmd.arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("neotrix "));
}

#[test]
fn subcommand_help_shows_usage() {
    // 每个子命令 --help 必须输出 Usage 行 (clap 自动生成, 测试防回归)
    for name in ALL_COMMANDS {
        let mut cmd = Command::cargo_bin("neotrix").unwrap();
        cmd.args([name, "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Usage:"));
    }
}

#[test]
fn help_subcommand_works() {
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    cmd.arg("help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: neotrix"));
}

#[test]
fn help_subcommand_for_specific_command() {
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    cmd.args(["help", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("status"));
}
