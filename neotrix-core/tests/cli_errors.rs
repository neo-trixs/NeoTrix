//! CLI 集成测试 — 错误处理与退出码 (对标主流 CLI 约定)
//!
//! 退出码约定: 0=成功 / 1=运行时错误 / 2=用法错误 (clap 标准)。
//! 错误信息统一输出到 stderr, 前缀 `error:`。

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn unknown_subcommand_exits_2() {
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    cmd.arg("nonexistent-command")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("unrecognized subcommand"));
}

#[test]
fn unknown_flag_exits_2() {
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    cmd.arg("--definitely-not-a-flag")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("unexpected argument"));
}

#[test]
fn exec_without_prompt_exits_2() {
    // 用法错误 (无 prompt/file/pipe) → 退出码 2, 对标 clap 约定
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    cmd.arg("exec")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error: no prompt provided"));
}

#[test]
fn exec_without_prompt_error_on_stderr() {
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    cmd.arg("exec")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Usage: neotrix exec"));
}

#[test]
fn unknown_subcommand_error_on_stderr() {
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    cmd.arg("nonexistent-command")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Usage: neotrix"));
}

#[test]
fn missing_required_arg_exits_2() {
    // search 需要 query 参数; 缺失 → clap 用法错误
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    cmd.arg("search")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("required"));
}

#[test]
fn invalid_value_exits_2() {
    // discover --port 需要数字; 非法值 → clap 解析错误
    let mut cmd = Command::cargo_bin("neotrix").unwrap();
    cmd.args(["discover", "--port", "not-a-number"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("invalid value"));
}
