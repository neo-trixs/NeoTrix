//! `nt_agent` — 有界多跳 agent loop.
//!
//! 编排移植 OpenMuse `engine/{conversation,model,worker}.ts`
//! (bounded steps + lease 心跳思想) 与 cumora `turn.ts` hop loop,
//! 工具收敛为 `bash + set_turn_status + 3×FS` (cumora 极简 schema),
//! 每个动作必经 `nt_policy` 网关 + `nt_audit` 先写后执.

use std::path::Path;

use chrono::Utc;
use uuid::Uuid;

use crate::nt_audit::{AuditDecision, AuditEvent};
use crate::nt_config::{NeobotConfig, PolicyMode};
use crate::nt_engine::EngineAdapter;
use crate::nt_error::NtBotError;
use crate::nt_policy::{Actor, PolicyContext, PolicyDecision, evaluate_policy};
use crate::nt_store::NeobotStore;
use crate::nt_types::{AgentTask, TaskStatus, ToolName, ToolResult, TurnStatus};

const READ_CAP: u64 = 512 * 1024;
const WRITE_CAP: usize = 2 * 1024 * 1024;
const OUTPUT_CAP: usize = 8 * 1024;

/// 跑一轮本地任务 (创建 task → 有界 loop → 落库), 返回终态.
pub fn run_local_turn(
    store: &NeobotStore,
    config: &NeobotConfig,
    engine: &dyn EngineAdapter,
    title: &str,
    user_text: &str,
) -> Result<TurnStatus, NtBotError> {
    run_local_turn_inner(store, config, engine, title, user_text, None)
}

/// 流式版 — 模型增量内容经 `on_delta` 回调 (SSE 真流式引擎).
pub fn run_local_turn_stream(
    store: &NeobotStore,
    config: &NeobotConfig,
    engine: &dyn EngineAdapter,
    title: &str,
    user_text: &str,
    on_delta: &mut dyn FnMut(&str),
) -> Result<TurnStatus, NtBotError> {
    run_local_turn_inner(store, config, engine, title, user_text, Some(on_delta))
}

fn run_local_turn_inner(
    store: &NeobotStore,
    config: &NeobotConfig,
    engine: &dyn EngineAdapter,
    title: &str,
    user_text: &str,
    on_delta: Option<&mut dyn FnMut(&str)>,
) -> Result<TurnStatus, NtBotError> {
    let now = Utc::now().to_rfc3339();
    let task = AgentTask {
        id: Uuid::new_v4().to_string(),
        title: title.to_owned(),
        status: TaskStatus::Running,
        created_at: now.clone(),
        updated_at: now,
    };
    store.save_task(&task)?;
    let status = run_loop(store, config, engine, &task.id, user_text, on_delta)?;
    let finished = AgentTask {
        status: match status {
            TurnStatus::Done => TaskStatus::Done,
            TurnStatus::Blocked => TaskStatus::Failed,
            TurnStatus::Continue | TurnStatus::NeedsClarification | TurnStatus::Waiting => {
                TaskStatus::Pending
            }
        },
        updated_at: Utc::now().to_rfc3339(),
        ..task
    };
    store.save_task(&finished)?;
    store.enqueue_outbox(
        &Uuid::new_v4().to_string(),
        "CH_MESSAGE_NEW",
        &serde_json::json!({"task_id": finished.id, "status": finished.status.as_str()}).to_string(),
    )?;
    Ok(status)
}

fn run_loop(
    store: &NeobotStore,
    config: &NeobotConfig,
    engine: &dyn EngineAdapter,
    task_id: &str,
    user_text: &str,
    mut on_delta: Option<&mut dyn FnMut(&str)>,
) -> Result<TurnStatus, NtBotError> {
    use crate::nt_types::{TranscriptItem, TranscriptRole};
    let steps = config.max_steps.max(1);
    let mut history: Vec<TranscriptItem> = Vec::new();
    let mut current = TurnStatus::Continue;
    for n in 0..steps {
        let turn = match on_delta.as_mut() {
            Some(callback) => engine.run_turn_stream(user_text, &history, &mut **callback)?,
            None => engine.run_turn_with_history(user_text, &history)?,
        };
        // 有用量即落账本 (cumora `llm_calls` 本地子集).
        if let Some(usage) = turn.usage.as_ref() {
            store.record_ledger(
                &Uuid::new_v4().to_string(),
                &Utc::now().to_rfc3339(),
                engine.engine_id(),
                engine.model_name(),
                usage.prompt_tokens,
                usage.completion_tokens,
                usage.cost_usd,
            )?;
        }
        history.push(TranscriptItem {
            role: TranscriptRole::Assistant,
            content: turn.assistant_text.clone(),
            tool_calls: turn.tool_calls.clone(),
            tool_call_id: None,
        });
        // 引擎自带 tool_calls 为空时按纯回复处理.
        if turn.tool_calls.is_empty() {
            store.add_step(task_id, i64::from(n), "reply", true, &turn.assistant_text)?;
            current = turn.status;
            break;
        }
        let mut saw_status: Option<TurnStatus> = None;
        for call in &turn.tool_calls {
            let (decision, rule) = gate(config, call)?;
            let allowed = matches!(decision, PolicyDecision::Allow);
            // dry-run: 记录但不执行. 执行错误转失败结果 (模型可见, 可换路),
            // 只有落库/审计失败才 `?` 中断.
            let result = if allowed && config.policy_mode == crate::nt_config::PolicyMode::Enforce {
                match execute_tool(config, call) {
                    Ok(result) => result,
                    Err(err) => ToolResult {
                        ok: false,
                        output: format!("tool error: {err}"),
                        truncated: false,
                    },
                }
            } else {
                ToolResult {
                    ok: false,
                    output: if allowed {
                        "(dry-run: not executed)".to_owned()
                    } else {
                        "(denied)".to_owned()
                    },
                    truncated: false,
                }
            };
            let event = AuditEvent::new(
                "bot",
                call.name.as_str(),
                if allowed {
                    AuditDecision::Allow
                } else {
                    AuditDecision::Deny
                },
                rule,
                &format!("task={task_id} ok={} out={}", result.ok, result.output),
            );
            store.record_audit(&event)?;
            store.add_step(task_id, i64::from(n), call.name.as_str(), result.ok, &result.output)?;
            history.push(TranscriptItem {
                role: TranscriptRole::Tool,
                content: truncate_history(&result.output),
                tool_calls: Vec::new(),
                tool_call_id: Some(call.id.clone()),
            });
            if call.name == ToolName::SetTurnStatus {
                saw_status = parse_status_arg(&call.args);
            }
            if !allowed && config.policy_mode == PolicyMode::Enforce {
                current = TurnStatus::Blocked;
                break;
            }
        }
        if let Some(status) = saw_status {
            current = status;
            if status != TurnStatus::Continue {
                break;
            }
        } else if current != TurnStatus::Continue {
            break;
        }
    }
    Ok(current)
}

/// 历史回填截断 (4KiB/条, 防上下文爆炸; 全量仍在 steps 表).
fn truncate_history(output: &str) -> String {
    const LIMIT: usize = 4096;
    if output.len() <= LIMIT {
        return output.to_owned();
    }
    let mut cut = LIMIT;
    while cut > 0 && !output.is_char_boundary(cut) {
        cut -= 1;
    }
    match output.get(..cut) {
        Some(safe) => format!("{safe}…[truncated]"),
        None => "…[truncated]".to_owned(),
    }
}

/// 网关门控: `Unknown` 工具也进策略 (一律拒绝, 原名进审计).
fn gate(config: &NeobotConfig, call: &crate::nt_types::ToolCall) -> Result<(PolicyDecision, Option<String>), NtBotError> {
    let file_path = ["path", "file"]
        .iter()
        .find_map(|key| call.args.get(*key))
        .and_then(|value| value.as_str())
        .map(str::to_owned);
    let command = call
        .args
        .get("command")
        .and_then(|value| value.as_str())
        .map(str::to_owned);
    let computer_action = call
        .args
        .get("action")
        .and_then(|value| value.as_str())
        .map(str::to_owned);
    let computer_target = call
        .args
        .get("target")
        .and_then(|value| value.as_str())
        .map(str::to_owned);
    let ctx = PolicyContext {
        tool: call.name.clone(),
        actor: Actor::Bot,
        human_has_control: config.human_has_control,
        file_path,
        command,
        computer_action,
        computer_target,
        computer_allow: config.computer_allow.clone(),
        computer_hosts: config.computer_hosts.clone(),
    };
    let decision = evaluate_policy(&ctx);
    let rule = match &decision {
        PolicyDecision::Allow => None,
        PolicyDecision::Deny { rule, .. } => Some(rule.clone()),
    };
    Ok((decision, rule))
}

fn parse_status_arg(args: &serde_json::Value) -> Option<TurnStatus> {
    args.get("status")
        .and_then(|value| value.as_str())
        .and_then(TurnStatus::parse)
}

fn execute_tool(config: &NeobotConfig, call: &crate::nt_types::ToolCall) -> Result<ToolResult, NtBotError> {
    match &call.name {
        ToolName::SetTurnStatus => Ok(ToolResult {
            ok: parse_status_arg(&call.args).is_some(),
            output: "status recorded".to_owned(),
            truncated: false,
        }),
        ToolName::Bash => execute_bash(config, call),
        ToolName::ReadFile => execute_read(config, call),
        ToolName::WriteFile => execute_write(config, call),
        ToolName::EditFile => execute_edit(config, call),
        ToolName::ComputerAct => execute_computer(call),
        ToolName::Unknown(raw) => Err(NtBotError::Invalid(format!("unknown tool '{raw}'"))),
    }
}

/// computer 执行 — 当前 Noop 后端诚实失败 (调用方转失败结果回填模型).
fn execute_computer(call: &crate::nt_types::ToolCall) -> Result<ToolResult, NtBotError> {
    use crate::nt_computer::{ComputerBackend as _, NoopBackend, parse_computer_call};
    let parsed = parse_computer_call(&call.args)?;
    let output = NoopBackend.execute(&parsed)?;
    Ok(ToolResult {
        ok: true,
        output,
        truncated: false,
    })
}

fn execute_bash(config: &NeobotConfig, call: &crate::nt_types::ToolCall) -> Result<ToolResult, NtBotError> {
    let Some(command) = call.args.get("command").and_then(|v| v.as_str()) else {
        return Err(NtBotError::Invalid("bash requires {command}".to_owned()));
    };
    if command.trim().is_empty() {
        return Err(NtBotError::Invalid("bash command is empty".to_owned()));
    }
    let output = std::process::Command::new("bash")
        .arg("-c")
        .arg(command)
        .current_dir(&config.workspace_dir)
        .output()?;
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        text.push_str(&stderr);
    }
    Ok(truncate_output(text, output.status.success()))
}

fn execute_read(config: &NeobotConfig, call: &crate::nt_types::ToolCall) -> Result<ToolResult, NtBotError> {
    let path = required_path(&call.args)?;
    let full = join_workspace(&config.workspace_dir, &path)?;
    let meta = std::fs::metadata(&full)?;
    if meta.len() > READ_CAP {
        return Err(NtBotError::Invalid(format!(
            "file too large ({} > 512KiB)",
            meta.len()
        )));
    }
    let content = std::fs::read_to_string(&full)?;
    Ok(truncate_output(content, true))
}

fn execute_write(config: &NeobotConfig, call: &crate::nt_types::ToolCall) -> Result<ToolResult, NtBotError> {
    let path = required_path(&call.args)?;
    let Some(content) = call.args.get("content").and_then(|v| v.as_str()) else {
        return Err(NtBotError::Invalid("write_file requires {content}".to_owned()));
    };
    if content.len() > WRITE_CAP {
        return Err(NtBotError::Invalid("content exceeds 2MiB".to_owned()));
    }
    let full = join_workspace(&config.workspace_dir, &path)?;
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&full, content)?;
    Ok(ToolResult {
        ok: true,
        output: format!("wrote {} bytes", content.len()),
        truncated: false,
    })
}

fn execute_edit(config: &NeobotConfig, call: &crate::nt_types::ToolCall) -> Result<ToolResult, NtBotError> {
    let path = required_path(&call.args)?;
    let (Some(old), Some(new)) = (
        call.args.get("old").and_then(|v| v.as_str()),
        call.args.get("new").and_then(|v| v.as_str()),
    ) else {
        return Err(NtBotError::Invalid("edit_file requires {old,new}".to_owned()));
    };
    let full = join_workspace(&config.workspace_dir, &path)?;
    let content = std::fs::read_to_string(&full)?;
    let matches = content.matches(old).count();
    if matches != 1 {
        return Err(NtBotError::Invalid(format!(
            "edit needs exactly 1 match, found {matches}"
        )));
    }
    let updated = content.replacen(old, new, 1);
    if updated.len() > WRITE_CAP {
        return Err(NtBotError::Invalid("result exceeds 2MiB".to_owned()));
    }
    std::fs::write(&full, updated)?;
    Ok(ToolResult {
        ok: true,
        output: "edited 1 occurrence".to_owned(),
        truncated: false,
    })
}

fn required_path(args: &serde_json::Value) -> Result<String, NtBotError> {
    args.get("path")
        .or_else(|| args.get("file"))
        .and_then(|v| v.as_str())
        .map(str::to_owned)
        .ok_or_else(|| NtBotError::Invalid("file tool requires {path}".to_owned()))
}

/// workspace jail 双保险 (策略层已判一次, 执行层再拼一次, 防 TOCTOU 式误用).
fn join_workspace(workspace: &Path, rel: &str) -> Result<std::path::PathBuf, NtBotError> {
    if rel.trim().is_empty() || rel.starts_with('/') || rel.starts_with('~') || rel.contains("..") {
        return Err(NtBotError::Denied {
            rule: "workspace-jail".to_owned(),
            reason: "path escapes workspace".to_owned(),
        });
    }
    Ok(workspace.join(rel))
}

fn truncate_output(text: String, ok: bool) -> ToolResult {
    if text.len() <= OUTPUT_CAP {
        return ToolResult {
            ok,
            output: text,
            truncated: false,
        };
    }
    let mut cut = OUTPUT_CAP;
    while cut > 0 && !text.is_char_boundary(cut) {
        cut -= 1;
    }
    let output = if let Some(safe) = text.get(..cut) {
        format!("{safe}…[truncated]")
    } else {
        "…[truncated]".to_owned()
    };
    ToolResult {
        ok,
        output,
        truncated: true,
    }
}

#[cfg(test)]
mod tests {
    use super::run_local_turn;
    use crate::nt_config::NeobotConfig;
    use crate::nt_engine::LocalEchoEngine;
    use crate::nt_store::NeobotStore;

    #[test]
    fn echo_run_completes_and_persists() {
        let dir = std::env::temp_dir().join("neobot-agent-test");
        let _ = std::fs::remove_dir_all(&dir);
        let config = NeobotConfig {
            data_dir: dir.clone(),
            workspace_dir: dir.join("workspace"),
            policy_mode: crate::nt_config::PolicyMode::Enforce,
            human_has_control: false,
            max_steps: 4,
            engine: crate::nt_config::EngineKind::Echo,
            computer_allow: Vec::new(),
            computer_hosts: Vec::new(),
        };
        config.validate().expect("validate");
        let store = NeobotStore::open(":memory:").expect("open");
        let status = run_local_turn(&store, &config, &LocalEchoEngine, "t", "hello").expect("run");
        assert_eq!(status, crate::nt_types::TurnStatus::Done);
        assert_eq!(store.list_tasks(10).expect("list").len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
