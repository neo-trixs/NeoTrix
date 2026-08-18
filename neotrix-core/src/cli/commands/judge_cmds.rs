//! Judge commands — C2 编程比赛代理 judge 层 CLI。
//!
//! `/judge` 对一段程序按测试用例集给出 verdict (passed/wrong_answer/timeout/
//! runtime_error/compile_error), 执行底座为 `nt_shield_sandbox` (R-P42 强化,
//! 不建平行适配器)。verdict 结果以 JSON 形态可供 MCP/上层消费。

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;
use crate::neotrix::nt_shield_sandbox::judge::{judge_code, JudgeConfig, TestCase};
use crate::neotrix::nt_shield_sandbox::{CloudRuntime, CloudSandbox};

/// `/judge` — run a program against test cases and produce verdicts.
pub struct JudgeCmd;

impl CliCommand for JudgeCmd {
    fn name(&self) -> &str { "/judge" }
    fn aliases(&self) -> Vec<&str> { vec!["/j"] }
    fn description(&self) -> &str {
        "/judge <python3|node18|rust|go|linux> <code> <expected...> [--input <in>] [--json]"
    }

    fn execute(
        &self,
        args: &[String],
        _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>,
    ) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");

        // --input <in> 提取测试输入 (默认空)。
        let input = args
            .windows(2)
            .find(|w| w[0] == "--input")
            .map(|w| w[1].clone())
            .unwrap_or_default();

        // 过滤掉 --json / --input 及其值。
        let mut clean: Vec<String> = Vec::new();
        let mut skip = 0usize;
        for (_i, a) in args.iter().enumerate() {
            if skip > 0 {
                skip -= 1;
                continue;
            }
            if a == "--json" {
                continue;
            }
            if a == "--input" {
                skip = 1;
                continue;
            }
            clean.push(a.clone());
        }

        if clean.len() < 3 {
            return CommandOutput::err(
                "Usage: /judge <python3|node18|rust|go|linux> <code> <expected...> [--input <in>] [--json]",
            );
        }
        let runtime = match CloudRuntime::from_str(&clean[0]) {
            Some(r) => r,
            None => return CommandOutput::err(&format!(
                "Unknown runtime '{}'. Available: python3, node18, rust, go, linux",
                clean[0]
            )),
        };
        let code = clean[1].clone();
        let expected = clean[2..].join(" ");

        let cases = vec![TestCase::new("t1", &input, &expected)];
        let config = JudgeConfig::default();

        let mut cloud = CloudSandbox::default_local();
        #[cfg(feature = "sandbox")]
        cloud.attach_default_vault();

        let rt = match tokio::runtime::Runtime::new() {
            Ok(r) => r,
            Err(e) => return CommandOutput::err(&format!("Failed to create tokio runtime: {}", e)),
        };
        let summary = rt.block_on(judge_code(&mut cloud, &code, runtime, &cases, &config, None));
        if let Some(v) = summary.verdicts.first() {
            let msg = format!(
                "judge: {} (exit={:?}, {}ms)\n-- stdout --\n{}\n-- stderr --\n{}",
                v.verdict.label(),
                v.verdict,
                v.execution_time_ms,
                v.stdout,
                v.stderr
            );
            if want_json {
                return CommandOutput::ok(&msg).with_json(serde_json::json!({
                    "case_id": v.case_id,
                    "verdict": v.verdict.label(),
                    "execution_time_ms": v.execution_time_ms,
                    "stdout": v.stdout,
                    "stderr": v.stderr,
                    "passed": v.verdict.is_pass(),
                }));
            }
            CommandOutput::ok(&msg)
        } else {
            CommandOutput::err("judge: no verdicts produced")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_judge_requires_args() {
        let cmd = JudgeCmd;
        let out = cmd.execute(&[], None);
        assert!(!out.success);
        assert!(out.message.contains("Usage"));
    }

    #[test]
    fn test_judge_rejects_unknown_runtime() {
        let cmd = JudgeCmd;
        let out = cmd.execute(&["cobol".into(), "x".into(), "y".into()], None);
        assert!(!out.success);
        assert!(out.message.contains("Unknown runtime"));
    }

    #[test]
    fn test_judge_argument_parsing_flags() {
        // --input 与 --json 不应干扰 runtime/code/expected 定位。
        let args: Vec<String> = vec![
            "python3".into(),
            "print(1)".into(),
            "1".into(),
            "--input".into(),
            "".into(),
            "--json".into(),
        ];
        let cmd = JudgeCmd;
        let out = cmd.execute(&args, None);
        // 本地 NoopProvider (无 docker) 会失败, 但参数解析路径应走到执行而非 Usage。
        assert!(!out.message.contains("Usage"), "must pass arg parsing: {}", out.message);
    }
}
