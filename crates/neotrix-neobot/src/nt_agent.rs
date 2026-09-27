//! `nt_agent` — 有界多跳 agent loop.
//!
//! 有界步数 + 运行租约心跳 + hop 循环；工具收敛为
//! `bash + set_turn_status + 3×FS` 极简 schema；
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
/// 运行租约秒（单轮最长 10 分钟；崩溃后 `recover_stale_running` 凭它回收）。
pub const LEASE_SECS: i64 = 600;

/// 增量回调（流式对话流用）。
pub type DeltaCallback<'a> = &'a mut dyn FnMut(&str);
/// 工具步骤回调（tool 名，成功与否，500 字内摘要）。
pub type StepCallback<'a> = &'a mut dyn FnMut(&str, bool, &str);

/// 一轮运行的只读上下文（rish Env 思想：输入全显式，打包传参；
/// 顺带把 9 参函数压到 clippy type_complexity 线下）。
pub struct RunContext<'a> {
    pub store: &'a NeobotStore,
    pub config: &'a NeobotConfig,
    pub engine: &'a dyn EngineAdapter,
    pub actor: Actor,
    pub actor_name: &'a str,
    pub title: &'a str,
    pub user_text: &'a str,
    pub convo_id: Option<&'a str>,
}

/// 跑一轮本地任务 (创建 task → 有界 loop → 落库), 返回终态.
pub fn run_local_turn(
    store: &NeobotStore,
    config: &NeobotConfig,
    engine: &dyn EngineAdapter,
    title: &str,
    user_text: &str,
) -> Result<TurnStatus, NtBotError> {
    let ctx = RunContext {
        store,
        config,
        engine,
        actor: Actor::Bot,
        actor_name: "bot",
        title,
        user_text,
        convo_id: None,
    };
    run_local_turn_inner(&ctx, None, None)
}

/// 发起方具名版（routine firing 审计记 `routine:<name>`；审计 actor 即发起方）。
/// `convo_id`: 指定会话则任务归属该会话；None 则自动新建群组会话
/// （IM 语义：调用方传选中会话 id 即可追加）。
pub fn run_local_turn_as(
    store: &NeobotStore,
    config: &NeobotConfig,
    engine: &dyn EngineAdapter,
    actor: Actor,
    actor_name: &str,
    title: &str,
    user_text: &str,
    convo_id: Option<&str>,
) -> Result<TurnStatus, NtBotError> {
    let ctx = RunContext {
        store,
        config,
        engine,
        actor,
        actor_name,
        title,
        user_text,
        convo_id,
    };
    run_local_turn_inner(&ctx, None, None)
}

/// 流式版 — 模型增量内容经 `on_delta` 回调 (SSE 真流式引擎).
pub fn run_local_turn_stream(
    store: &NeobotStore,
    config: &NeobotConfig,
    engine: &dyn EngineAdapter,
    title: &str,
    user_text: &str,
    on_delta: DeltaCallback<'_>,
) -> Result<TurnStatus, NtBotError> {
    let ctx = RunContext {
        store,
        config,
        engine,
        actor: Actor::Bot,
        actor_name: "bot",
        title,
        user_text,
        convo_id: None,
    };
    run_local_turn_inner(&ctx, Some(on_delta), None)
}

/// 流式具名版.
pub fn run_local_turn_stream_as(
    store: &NeobotStore,
    config: &NeobotConfig,
    engine: &dyn EngineAdapter,
    actor: Actor,
    actor_name: &str,
    title: &str,
    user_text: &str,
    convo_id: Option<&str>,
    on_delta: DeltaCallback<'_>,
    on_step: Option<StepCallback<'_>>,
) -> Result<TurnStatus, NtBotError> {
    let ctx = RunContext {
        store,
        config,
        engine,
        actor,
        actor_name,
        title,
        user_text,
        convo_id,
    };
    run_local_turn_inner(&ctx, Some(on_delta), on_step)
}

/// 占位标题（自动命名只动这些；用户手改过的永不动）。
fn is_placeholder_title(title: &str) -> bool {
    matches!(title.trim(), "我的私聊" | "新群组" | "新对话" | "")
}

/// 首行片段（去空白，截 24 字；纯附件系统行/空返回 None，不命名）。
fn title_snippet(text: &str) -> Option<String> {
    let first = text.lines().map(str::trim).find(|line| !line.is_empty())?;
    if first.starts_with('[') && first.ends_with(']') {
        return None;
    }
    let snippet: String = first.chars().take(24).collect();
    if snippet.is_empty() {
        None
    } else {
        Some(snippet)
    }
}

fn run_local_turn_inner(
    ctx: &RunContext<'_>,
    on_delta: Option<DeltaCallback<'_>>,
    on_step: Option<StepCallback<'_>>,
) -> Result<TurnStatus, NtBotError> {
    let store = ctx.store;
    let title = ctx.title;
    let convo_id = ctx.convo_id;
    let now = Utc::now().to_rfc3339();
    // 起点两扫（单机，一次 UPDATE 级代价）：崩溃残留回收 + 过期认领释放。
    // best-effort：扫失败不挡本轮（下次起点再扫）。
    let _recovered: usize = store.recover_stale_running(&now).unwrap_or(0);
    let _swept: usize = store
        .sweep_stale_claims(&now, crate::nt_store::CLAIM_TTL_SECS)
        .unwrap_or(0);
    let lease_until =
        (Utc::now() + chrono::Duration::seconds(LEASE_SECS)).to_rfc3339();
    // 会话归属：指定即用（不存在则报错，不静默建）；缺省自动建群组会话。
    let convo_id = match convo_id {
        Some(id) => {
            let found = store
                .list_conversations()?
                .into_iter()
                .find(|c| c.id == id);
            let Some(convo) = found else {
                return Err(NtBotError::Store(format!("no such conversation '{id}'")));
            };
            // 自动标题：占位名会话 + 用户首条有效输入 → 取首行片段命名（仅一次；
            // 用户手改过的标题永不动；失败不挡本轮）。
            if is_placeholder_title(&convo.title) {
                if let Some(snippet) = title_snippet(ctx.user_text) {
                    let _renamed: Option<()> =
                        store.rename_conversation(&convo.id, &snippet).ok();
                }
            }
            id.to_owned()
        }
        None => store.create_conversation("group", title, &[])?,
    };
    let task = AgentTask {
        id: Uuid::new_v4().to_string(),
        title: title.to_owned(),
        status: TaskStatus::Running,
        created_at: now.clone(),
        updated_at: now,
        claimed_by: None,
        claimed_at: None,
        visibility: crate::nt_types::default_visibility(),
        lease_id: Some(Uuid::new_v4().to_string()),
        lease_until: Some(lease_until),
        attempts: 1,
        error: None,
        conversation_id: Some(convo_id),
    };
    store.save_task(&task)?;
    let status = run_loop(ctx, &task.id, on_delta, on_step);
    // turn 级错误（落库失败等）记终态 Failed + error 后原错返回，不吞错。
    let status = match status {
        Ok(status) => status,
        Err(err) => {
            let failed = AgentTask {
                status: TaskStatus::Failed,
                updated_at: Utc::now().to_rfc3339(),
                lease_id: None,
                lease_until: None,
                error: Some(err.to_string()),
                ..task.clone()
            };
            // 已在错误处理中：落库再败也无处可记，静默丢弃。
            let _saved: Result<(), NtBotError> = store.save_task(&failed);
            return Err(err);
        }
    };
    let finished = AgentTask {
        status: match status {
            TurnStatus::Done => TaskStatus::Done,
            TurnStatus::Blocked => TaskStatus::Failed,
            TurnStatus::Continue | TurnStatus::NeedsClarification | TurnStatus::Waiting => {
                TaskStatus::Pending
            }
        },
        updated_at: Utc::now().to_rfc3339(),
        lease_id: None,
        lease_until: None,
        error: if status == TurnStatus::Done {
            None
        } else {
            Some(format!("turn ended as {}", status.as_str()))
        },
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
    ctx: &RunContext<'_>,
    task_id: &str,
    mut on_delta: Option<DeltaCallback<'_>>,
    mut on_step: Option<StepCallback<'_>>,
) -> Result<TurnStatus, NtBotError> {
    use crate::nt_types::{TranscriptItem, TranscriptRole};
    let store = ctx.store;
    let config = ctx.config;
    let engine = ctx.engine;
    let actor = ctx.actor;
    let actor_name = ctx.actor_name;
    let user_text = ctx.user_text;
    let steps = config.max_steps.max(1);
    let mut history: Vec<TranscriptItem> = Vec::new();
    let mut current = TurnStatus::Continue;
    // 单轮累计写入（rish 嵌套预算中层）。
    let mut turn_written: usize = 0;
    for n in 0..steps {
        // 最后一步且已有工具活动: 提醒收尾 (防跑满 max_steps 仍无终态).
        if n + 1 == steps && !history.is_empty() {
            history.push(TranscriptItem {
                role: TranscriptRole::User,
                content: format!(
                    "提醒: 这是最后一步 (max_steps={steps})。请用已有信息直接回复, \
                     并调用 set_turn_status(done) 收尾, 不要再调工具。"
                ),
                tool_calls: Vec::new(),
                tool_call_id: None,
            });
        }
        let hop_started = std::time::Instant::now();
        let turn = match on_delta.as_mut() {
            Some(callback) => engine.run_turn_stream(user_text, &history, &mut **callback)?,
            None => engine.run_turn_with_history(user_text, &history)?,
        };
        let hop_latency_ms = hop_started.elapsed().as_millis().min(i64::MAX as u128) as i64;
        // 有用量即落账本（purpose + measured + status + latency）。
        if let Some(usage) = turn.usage.as_ref() {
            let policy = crate::nt_cost::CostPolicy::from_env();
            let (cost_usd, measured) = if policy.is_configured()
                || crate::nt_cost::price_for(engine.model_name()).is_some()
            {
                crate::nt_cost::cost_for(
                    policy,
                    engine.model_name(),
                    usage.prompt_tokens,
                    usage.completion_tokens,
                )
            } else if usage.cost_usd > 0.0 {
                (usage.cost_usd, true)
            } else {
                (0.0, false)
            };
            store.record_ledger(&crate::nt_store::LedgerEntry {
                id: Uuid::new_v4().to_string(),
                at: Utc::now().to_rfc3339(),
                engine: engine.engine_id().to_owned(),
                model: engine.model_name().to_owned(),
                actor: actor_name.to_owned(),
                purpose: "agent-turn".to_owned(),
                in_tokens: usage.prompt_tokens,
                out_tokens: usage.completion_tokens,
                cost_usd,
                measured,
                status: turn.status.as_str().to_owned(),
                latency_ms: hop_latency_ms,
                error: None,
            })?;
        }
        // CLI 副作用回传 → step 行（落库前复核：kind 非空才记）。
        for effect in &turn.side_effects {
            if effect.kind.trim().is_empty() {
                continue;
            }
            let detail = serde_json::to_string(&effect.payload).unwrap_or_default();
            store.add_step(
                task_id,
                i64::from(n),
                &format!("side-effect:{}", effect.kind),
                true,
                &detail,
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
            let (decision, rule) = gate(config, actor, call)?;
            let allowed = matches!(decision, PolicyDecision::Allow);
            // 网关律：先写审计行（无论放行与否），再执行。
            // 崩溃也不丢“谁动了什么”的记录；执行结果只进 steps 行。
            let intent = call.name.intent();
            let pre_event = AuditEvent::new(
                actor_name,
                call.name.as_str(),
                if allowed {
                    AuditDecision::Allow
                } else {
                    AuditDecision::Deny
                },
                rule.clone(),
                &format!("task={task_id} intent={intent}"),
            );
            store.record_audit(&pre_event)?;
            // dry-run: 记录但不执行. 执行错误转失败结果 (模型可见, 可换路),
            // 只有落库/审计失败才 `?` 中断.
            let result = if allowed && config.policy_mode == crate::nt_config::PolicyMode::Enforce {
                match execute_tool(config, call, &mut turn_written) {
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
            let reason_note = status_reason(&call.args)
                .map(|reason| format!(" reason={reason}"))
                .unwrap_or_default();
            store.add_step(
                task_id,
                i64::from(n),
                call.name.as_str(),
                result.ok,
                &format!("{}{reason_note}", result.output),
            )?;
            // 工作流事件外发（流式对话流用；500 字截断，明细仍在 steps 表）。
            if let Some(emit) = on_step.as_mut() {
                let mut snippet = result.output.clone();
                if snippet.len() > 500 {
                    let mut cut = 500;
                    while cut > 0 && !snippet.is_char_boundary(cut) {
                        cut -= 1;
                    }
                    snippet.truncate(cut);
                    snippet.push('…');
                }
                emit(call.name.as_str(), result.ok, &snippet);
            }
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
        enforce_transcript_budget(&mut history);
    }
    // 跑满仍无终态 (模型一直行动不收尾) → Waiting (任务 Pending, 人可接手).
    if current == TurnStatus::Continue {
        current = TurnStatus::Waiting;
    }
    Ok(current)
}

/// 转录预算（rish 转录上限思想本地值：总量 256KiB / 200 条）。
/// 超限从旧往新丢 Tool 结果行（用户原文与 assistant 回复保留；
/// 全量仍在 steps 表，可追溯）。
const TRANSCRIPT_CAP_BYTES: usize = 256 * 1024;
const TRANSCRIPT_CAP_ITEMS: usize = 200;

fn enforce_transcript_budget(history: &mut Vec<crate::nt_types::TranscriptItem>) {
    while history.len() > TRANSCRIPT_CAP_ITEMS {
        let Some(pos) = history
            .iter()
            .position(|item| item.role == crate::nt_types::TranscriptRole::Tool)
        else {
            break;
        };
        history.remove(pos);
    }
    let mut bytes: usize = history.iter().map(|item| item.content.len()).sum();
    while bytes > TRANSCRIPT_CAP_BYTES {
        let Some(pos) = history
            .iter()
            .position(|item| item.role == crate::nt_types::TranscriptRole::Tool)
        else {
            break;
        };
        bytes = bytes.saturating_sub(
            history
                .get(pos)
                .map(|item| item.content.len())
                .unwrap_or(0),
        );
        if history.get(pos).is_none() {
            break;
        }
        history.remove(pos);
    }
}

/// 历史回填截断 (4KiB/条, 防上下文爆炸; 全量仍在 steps 表).
fn truncate_history(output: &str) -> String {    const LIMIT: usize = 4096;
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
fn gate(
    config: &NeobotConfig,
    actor: Actor,
    call: &crate::nt_types::ToolCall,
) -> Result<(PolicyDecision, Option<String>), NtBotError> {
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
        actor,
        human_has_control: config.human_has_control,
        file_path: file_path.clone(),
        command: command.clone(),
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
    if rule.is_none() {
        // operator 自写 deny：base 放行后才看。
        if let Some((extra_rule, reason)) = crate::nt_policy::evaluate_extra_deny(
            &config.extra_deny,
            &call.name,
            actor,
            command.as_deref(),
            file_path.as_deref(),
        ) {
            return Ok((
                PolicyDecision::Deny {
                    rule: extra_rule.clone(),
                    reason,
                },
                Some(extra_rule),
            ));
        }
    }
    Ok((decision, rule))
}

/// `set_turn_status` 的 reason（status/reason/next_step 三件套；
/// reason 进 step 行，方便复盘“为什么停”）。
fn status_reason(args: &serde_json::Value) -> Option<String> {
    args.get("reason")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|reason| !reason.is_empty())
        .map(|reason| {
            let mut cut = reason.len().min(200);
            while cut > 0 && !reason.is_char_boundary(cut) {
                cut -= 1;
            }
            reason.get(..cut).unwrap_or("").to_owned()
        })
}

fn parse_status_arg(args: &serde_json::Value) -> Option<TurnStatus> {
    args.get("status")
        .and_then(|value| value.as_str())
        .and_then(TurnStatus::parse)
}

fn execute_tool(
    config: &NeobotConfig,
    call: &crate::nt_types::ToolCall,
    turn_written: &mut usize,
) -> Result<ToolResult, NtBotError> {
    match &call.name {
        ToolName::SetTurnStatus => Ok(ToolResult {
            ok: parse_status_arg(&call.args).is_some(),
            output: "status recorded".to_owned(),
            truncated: false,
        }),
        ToolName::Bash => execute_bash(config, call),
        ToolName::ReadFile => execute_read(config, call),
        ToolName::WriteFile => execute_write(config, call, turn_written),
        ToolName::EditFile => execute_edit(config, call, turn_written),
        ToolName::ComputerAct => execute_computer(call),
        ToolName::WebSearch => execute_web_search(call),
        ToolName::WebFetch => execute_web_fetch(call),
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

/// 联网搜索执行（客户端直调；count 越界钳制 1-10，缺 query 直接 Invalid）。
fn execute_web_search(call: &crate::nt_types::ToolCall) -> Result<ToolResult, NtBotError> {
    let Some(query) = call.args.get("query").and_then(|v| v.as_str()) else {
        return Err(NtBotError::Invalid("web_search requires {query}".to_owned()));
    };
    let count = call
        .args
        .get("count")
        .and_then(|v| v.as_u64())
        .unwrap_or(5) as usize;
    let output = crate::nt_web::web_search(query, count)?;
    Ok(ToolResult { ok: true, output, truncated: true })
}

/// 网页抓取执行（scheme 门控在 nt_web 内，fail-closed）。
fn execute_web_fetch(call: &crate::nt_types::ToolCall) -> Result<ToolResult, NtBotError> {
    let Some(url) = call.args.get("url").and_then(|v| v.as_str()) else {
        return Err(NtBotError::Invalid("web_fetch requires {url}".to_owned()));
    };
    let output = crate::nt_web::web_fetch(url)?;
    Ok(ToolResult { ok: true, output, truncated: true })
}

/// agent 侧 bash 执行（P0 审计 F2 加固版）：///
/// - 60s 超时杀（此前 `.output()` 死等，一句 `sleep 999` 卡死整轮）；
/// - 环境脱敏：`env_clear` + 最小白名单，不把宿主 secrets 递进 bash
///   （此前 `env` 回显直达模型 = key 送提供方）；
/// - 管道读数走独立线程，避免大输出撑爆 pipe 导致假死。
fn execute_bash(config: &NeobotConfig, call: &crate::nt_types::ToolCall) -> Result<ToolResult, NtBotError> {
    use std::io::Read as _;
    use std::process::Stdio;
    use std::time::{Duration, Instant};

    const BASH_TIMEOUT: Duration = Duration::from_secs(60);
    /// 递进子进程的最小环境白名单（PATH/HOME 等够用即可；`*KEY*` 类一律不传）。
    const ENV_ALLOW: &[&str] = &[
        "PATH", "HOME", "USER", "LOGNAME", "LANG", "LC_ALL", "TMPDIR", "TEMP", "TERM",
    ];

    let Some(command) = call.args.get("command").and_then(|v| v.as_str()) else {
        return Err(NtBotError::Invalid("bash requires {command}".to_owned()));
    };
    if command.trim().is_empty() {
        return Err(NtBotError::Invalid("bash command is empty".to_owned()));
    }
    let mut cmd = std::process::Command::new("bash");
    cmd.arg("-c")
        .arg(command)
        .current_dir(&config.workspace_dir)
        .env_clear()
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for key in ENV_ALLOW {
        if let Some(value) = std::env::var_os(key) {
            cmd.env(key, value);
        }
    }
    let mut child = cmd.spawn().map_err(|e| NtBotError::Io(format!("spawn bash: {e}")))?;
    let stdout_handle = child.stdout.take().map(|mut pipe| {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _drained: Option<usize> = pipe.read_to_end(&mut buf).ok();
            buf
        })
    });
    let stderr_handle = child.stderr.take().map(|mut pipe| {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _drained: Option<usize> = pipe.read_to_end(&mut buf).ok();
            buf
        })
    });
    let deadline = Instant::now() + BASH_TIMEOUT;
    let status = loop {
        match child.try_wait().map_err(|e| NtBotError::Io(format!("wait bash: {e}")))? {
            Some(status) => break status,
            None if Instant::now() >= deadline => {
                let _killed: Option<()> = child.kill().ok();
                let _waited: Option<std::process::ExitStatus> = child.wait().ok();
                let mut text = String::from_utf8_lossy(&stdout_handle.and_then(|h| h.join().ok()).unwrap_or_default()).into_owned();
                text.push_str("\n[neobot] bash timed out after 60s and was killed");
                return Ok(truncate_output(text, false));
            }
            None => std::thread::sleep(Duration::from_millis(50)),
        }
    };
    let mut text = String::from_utf8_lossy(
        &stdout_handle.and_then(|h| h.join().ok()).unwrap_or_default(),
    )
    .into_owned();
    if !status.success() {
        let stderr_bytes = stderr_handle.and_then(|h| h.join().ok()).unwrap_or_default();
        let stderr = String::from_utf8_lossy(&stderr_bytes);
        text.push_str(&stderr);
    }
    Ok(truncate_output(text, status.success()))
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

fn execute_write(
    config: &NeobotConfig,
    call: &crate::nt_types::ToolCall,
    turn_written: &mut usize,
) -> Result<ToolResult, NtBotError> {
    let path = required_path(&call.args)?;
    let Some(content) = call.args.get("content").and_then(|v| v.as_str()) else {
        return Err(NtBotError::Invalid("write_file requires {content}".to_owned()));
    };
    // 嵌套预算：单次上限 + 单轮累计上限（rish 律）。
    let budget = config.write_budget;
    if content.len() > budget.max_single_write_bytes {
        return Err(NtBotError::Invalid(format!(
            "content {} bytes exceeds single-write budget {}",
            content.len(),
            budget.max_single_write_bytes
        )));
    }
    if content.len() > WRITE_CAP {
        return Err(NtBotError::Invalid("content exceeds 2MiB".to_owned()));
    }
    *turn_written = turn_written.saturating_add(content.len());
    if *turn_written > budget.max_turn_write_bytes {
        return Err(NtBotError::Denied {
            rule: "write-budget".to_owned(),
            reason: format!(
                "turn wrote {} bytes, budget {}",
                turn_written, budget.max_turn_write_bytes
            ),
        });
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

fn execute_edit(
    config: &NeobotConfig,
    call: &crate::nt_types::ToolCall,
    turn_written: &mut usize,
) -> Result<ToolResult, NtBotError> {
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
    let budget = config.write_budget;
    if updated.len() > WRITE_CAP {
        return Err(NtBotError::Invalid("result exceeds 2MiB".to_owned()));
    }
    *turn_written = turn_written.saturating_add(new.len());
    if *turn_written > budget.max_turn_write_bytes {
        return Err(NtBotError::Denied {
            rule: "write-budget".to_owned(),
            reason: format!(
                "turn wrote {} bytes, budget {}",
                turn_written, budget.max_turn_write_bytes
            ),
        });
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
    use super::{enforce_transcript_budget, run_local_turn};
    use crate::nt_config::NeobotConfig;
    use crate::nt_engine::LocalEchoEngine;
    use crate::nt_store::NeobotStore;
    use crate::nt_types::{TranscriptItem, TranscriptRole};

    #[test]
    fn transcript_budget_drops_oldest_tool_rows_first() {
        let mut history = vec![
            TranscriptItem {
                role: TranscriptRole::User,
                content: "keep me".to_owned(),
                tool_calls: Vec::new(),
                tool_call_id: None,
            },
            TranscriptItem {
                role: TranscriptRole::Tool,
                content: "x".repeat(300 * 1024),
                tool_calls: Vec::new(),
                tool_call_id: Some("old".to_owned()),
            },
            TranscriptItem {
                role: TranscriptRole::Assistant,
                content: "keep me too".to_owned(),
                tool_calls: Vec::new(),
                tool_call_id: None,
            },
        ];
        enforce_transcript_budget(&mut history);
        // 300KiB 的旧 Tool 行被丢，用户原文与回复保留。
        assert_eq!(history.len(), 2);
        assert!(history.iter().all(|item| item.role != TranscriptRole::Tool));
        assert!(history.iter().any(|item| item.content == "keep me"));
    }

    #[test]
    fn bash_env_scrubbed_and_path_kept() {
        use super::execute_bash;
        use crate::nt_types::{ToolCall, ToolName};
        // 审计 F2：宿主 secrets 不得递进 bash；白名单 PATH 得留（找得到 wc）。
        std::env::set_var("NEOBOT_TEST_ONLY_SECRET", "s3cr3t-marker");
        let dir = std::env::temp_dir().join("neobot-bash-test");
        let _ = std::fs::create_dir_all(dir.join("workspace"));
        let config = NeobotConfig {
            data_dir: dir.clone(),
            workspace_dir: dir.join("workspace"),
            policy_mode: crate::nt_config::PolicyMode::Enforce,
            human_has_control: false,
            max_steps: 4,
            engine: crate::nt_config::EngineKind::Echo,
            computer_allow: Vec::new(),
            computer_hosts: Vec::new(),
            extra_deny: Vec::new(),
            write_budget: crate::nt_config::WriteBudget {
                max_single_write_bytes: 1024,
                max_turn_write_bytes: 1024,
                max_task_write_bytes: 1024,
            },
        };
        let call = ToolCall {
            id: "c1".to_owned(),
            name: ToolName::Bash,
            args: serde_json::json!({"command": "echo $NEOBOT_TEST_ONLY_SECRET | wc -c"}),
        };
        let res = execute_bash(&config, &call).expect("run");
        std::env::remove_var("NEOBOT_TEST_ONLY_SECRET");
        assert!(res.ok, "PATH whitelist must keep wc working: {}", res.output);
        assert!(
            !res.output.contains("s3cr3t-marker"),
            "env must be scrubbed: {}",
            res.output
        );
    }

    #[test]
    fn turn_write_budget_denies_overrun() {
        let dir = std::env::temp_dir().join("neobot-budget-test");
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
            extra_deny: Vec::new(),
            write_budget: crate::nt_config::WriteBudget {
                max_single_write_bytes: 10,
                max_turn_write_bytes: 15,
                max_task_write_bytes: 1024,
            },
        };
        config.validate().expect("validate");
        // 单次超限拒
        let big = serde_json::json!({"path": "a.txt", "content": "0123456789ABCDEF"});
        let mut turn_written = 0usize;
        let err = super::execute_write(&config, &crate::nt_types::ToolCall {
            id: "w1".to_owned(),
            name: crate::nt_types::ToolName::WriteFile,
            args: big,
        }, &mut turn_written)
        .expect_err("single over budget must fail");
        assert!(err.to_string().contains("single-write"), "{err}");
        // 两次小写累计超轮预算拒
        let small = serde_json::json!({"path": "b.txt", "content": "0123456789"});
        let call = crate::nt_types::ToolCall {
            id: "w2".to_owned(),
            name: crate::nt_types::ToolName::WriteFile,
            args: small,
        };
        super::execute_write(&config, &call, &mut turn_written).expect("first small write");
        let err = super::execute_write(&config, &call, &mut turn_written)
            .expect_err("turn over budget must fail");
        assert!(err.to_string().contains("write-budget"), "{err}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn placeholder_convo_auto_titles_from_first_text() {
        let dir = std::env::temp_dir().join("neobot-autotitle-test");
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
            extra_deny: Vec::new(),
            write_budget: crate::nt_config::default_write_budget(),
        };
        config.validate().expect("validate");
        let store = NeobotStore::open(":memory:").expect("open");
        store.upsert_member("neo", "human").expect("member");
        let dm = store
            .create_conversation("dm", "我的私聊", &["neo".to_owned()])
            .expect("dm");
        let run = |text: &str, convo: &str| {
            super::run_local_turn_as(
                &store,
                &config,
                &LocalEchoEngine,
                crate::nt_policy::Actor::Bot,
                "bot",
                "t",
                text,
                Some(convo),
            )
            .expect("run")
        };
        let title_of = |id: &str| {
            store
                .list_conversations()
                .expect("list")
                .into_iter()
                .find(|c| c.id == id)
                .expect("convo")
                .title
        };
        // 首条有效输入命名
        run("帮我写一份周报总结\n第二行不进标题", &dm);
        assert_eq!(title_of(&dm), "帮我写一份周报总结");
        // 仅一次：第二轮不再改名
        run("随便聊点别的什么内容", &dm);
        assert_eq!(title_of(&dm), "帮我写一份周报总结");
        // 纯附件系统行不命名
        let g = store.create_conversation("group", "新群组", &[]).expect("group");
        run("[附件：a.png]", &g);
        assert_eq!(title_of(&g), "新群组");
        // 超长截 24 字
        let g2 = store.create_conversation("group", "新群组", &[]).expect("group2");
        run("这是一条超过二十四个字的超长输入内容用来测试截断行为是否正确", &g2);
        assert_eq!(title_of(&g2).chars().count(), 24);
        let _ = std::fs::remove_dir_all(&dir);
    }

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
            extra_deny: Vec::new(),
            write_budget: crate::nt_config::default_write_budget(),
        };
        config.validate().expect("validate");
        let store = NeobotStore::open(":memory:").expect("open");
        let status = run_local_turn(&store, &config, &LocalEchoEngine, "t", "hello").expect("run");
        assert_eq!(status, crate::nt_types::TurnStatus::Done);
        assert_eq!(store.list_tasks(10).expect("list").len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 永远只调工具不收尾的引擎 — 验证耗尽转 Waiting + 最后一步 nudge.
    struct LoopForever {
        seen_nudge: std::sync::Mutex<bool>,
    }

    impl crate::nt_engine::EngineAdapter for LoopForever {
        fn engine_id(&self) -> &str {
            "loop"
        }

        fn probe(&self) -> Result<String, crate::NtBotError> {
            Ok("loop".to_owned())
        }

        fn run_turn(
            &self,
            _prompt: &str,
            _inbox: &[String],
        ) -> Result<crate::nt_engine::EngineTurn, crate::NtBotError> {
            Ok(crate::nt_engine::EngineTurn {
                assistant_text: String::new(),
                status: crate::nt_types::TurnStatus::Continue,
                tool_calls: vec![crate::nt_types::ToolCall {
                    id: "l1".to_owned(),
                    name: crate::nt_types::ToolName::Bash,
                    args: serde_json::json!({"command": "echo x"}),
                }],
                usage: None,
                side_effects: Vec::new(),
            })
        }

        fn run_turn_with_history(
            &self,
            prompt: &str,
            history: &[crate::nt_types::TranscriptItem],
        ) -> Result<crate::nt_engine::EngineTurn, crate::NtBotError> {
            if history.iter().any(|item| item.content.contains("最后一步")) {
                if let Ok(mut seen) = self.seen_nudge.lock() {
                    *seen = true;
                }
            }
            self.run_turn(prompt, &[])
        }
    }

    #[test]
    fn exhaustion_becomes_waiting_with_nudge() {
        let dir = std::env::temp_dir().join("neobot-agent-loop-test");
        let _ = std::fs::remove_dir_all(&dir);
        let config = NeobotConfig {
            data_dir: dir.clone(),
            workspace_dir: dir.join("workspace"),
            policy_mode: crate::nt_config::PolicyMode::Enforce,
            human_has_control: false,
            max_steps: 3,
            engine: crate::nt_config::EngineKind::Echo,
            computer_allow: Vec::new(),
            computer_hosts: Vec::new(),
            extra_deny: Vec::new(),
            write_budget: crate::nt_config::default_write_budget(),
        };
        config.validate().expect("validate");
        let store = NeobotStore::open(":memory:").expect("open");
        let engine = LoopForever {
            seen_nudge: std::sync::Mutex::new(false),
        };
        let status = run_local_turn(&store, &config, &engine, "loop", "go").expect("run");
        assert_eq!(status, crate::nt_types::TurnStatus::Waiting);
        assert!(engine.seen_nudge.lock().map(|seen| *seen).unwrap_or(false));
        // 3 跳 bash 全执行.
        let audits = store.list_audit(20).expect("audits");
        assert_eq!(
            audits.iter().filter(|event| event.tool == "bash").count(),
            3
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
