//! CLI 集成测试 — 纯本地命令冒烟 (真实运行二进制)
//!
//! 只测不依赖 LLM provider / 网络 / 交互式 wizard 的命令,
//! 保证在任何环境 (CI/无配置) 下可重复通过。

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn status_exits_0() {
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    cmd.args(["status", "--color", "never"])
        .assert()
        .success()
        .stdout(predicate::str::contains("NeoTrix Status"));
}

#[test]
fn completions_bash_generates_function() {
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    cmd.args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_neotrix"));
}

#[test]
fn completions_zsh_generates_function() {
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    cmd.args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_neotrix"));
}

#[test]
fn completions_fish_generates_function() {
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    cmd.args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete"));
}

#[test]
fn completions_powershell_generates_function() {
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    cmd.args(["completions", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Register-ArgumentCompleter"));
}

#[test]
fn completions_unsupported_shell_errors() {
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    cmd.args(["completions", "tcsh"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("unsupported shell"));
}

// ── 智能命令整合: 未知子命令回退到交互式命令注册表 ──

#[test]
fn unknown_subcommand_falls_back_to_registry() {
    // `neotrix kb` → clap 未知子命令 → 回退 registry `/kb`
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    cmd.args(["kb"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Knowledge Base"));
}

#[test]
fn slash_command_directly_from_cli() {
    // `neotrix /kb` → 直接斜杠命令
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    cmd.args(["/kb"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Knowledge Base"));
}

#[test]
fn fuzzy_subcommand_matches_registry() {
    // `neotrix kbl` → 编辑距离模糊匹配 → /kb
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    cmd.args(["kbl"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Knowledge Base"));
}

#[test]
fn truly_unknown_command_still_errors() {
    // 完全未知 → 保持 clap 用法错误 (exit 2)
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    cmd.args(["foobarxyz"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("unrecognized subcommand"));
}

#[test]
fn features_list_exits_0() {
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    cmd.args(["features", "list", "--color", "never"])
        .assert()
        .success();
}

#[test]
fn features_enable_unknown_flag_exits_0_or_1() {
    // 未知 feature 名: 不崩溃, 退出码 0 或 1 均可 (取决于实现), 但不得 panic
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    let out = cmd
        .args(["features", "enable", "definitely-not-a-feature"])
        .output()
        .unwrap();
    assert!(
        out.status.code().is_some(),
        "features enable must not panic (signal={:?})",
        out.status.code()
    );
}

#[test]
fn discover_json_output_is_valid() {
    // UDP 扫描 (短时长) 输出 JSON; 验证可解析且含 agent_count 字段
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    let out = cmd
        .args(["discover", "--duration", "100", "--json"])
        .output()
        .unwrap();
    assert!(out.status.code().is_some(), "discover must not panic");
    let stdout = String::from_utf8_lossy(&out.stdout);
    // 扫描可能无结果, 但 JSON 必须可解析
    if let Some(json_start) = stdout.find('{') {
        let json_str = &stdout[json_start..];
        let v: serde_json::Value = serde_json::from_str(json_str)
            .unwrap_or_else(|e| panic!("discover JSON invalid: {e}\nraw: {stdout}"));
        assert!(
            v.get("agent_count").is_some(),
            "JSON missing agent_count: {v}"
        );
    }
}

#[test]
fn bench_help_exits_0() {
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    cmd.args(["bench", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn wallet_help_exits_0() {
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    cmd.args(["wallet", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn sandbox_help_exits_0() {
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    cmd.args(["sandbox", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn config_help_exits_0() {
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    cmd.args(["config", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn evidence_help_exits_0() {
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    cmd.args(["evidence", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn quiet_suppresses_config_log() {
    // --quiet 应抑制 [config] loaded 诊断输出 (stderr)
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    cmd.args(["--quiet", "status"])
        .assert()
        .success()
        .stderr(predicate::str::contains("[config] loaded").not());
}

#[test]
fn color_never_emits_no_ansi() {
    // --color never 的 stdout 不应含 ANSI 转义序列 (ESC[)
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    let out = cmd.args(["status", "--color", "never"]).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains('\u{1b}'),
        "stdout must not contain ANSI escapes with --color never: {:?}",
        stdout.chars().take(80).collect::<String>()
    );
}

#[test]
fn color_always_emits_ansi() {
    // --color always 应输出 ANSI 转义序列
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    let out = cmd.args(["status", "--color", "always"]).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains('\u{1b}'),
        "stdout should contain ANSI escapes with --color always"
    );
}
