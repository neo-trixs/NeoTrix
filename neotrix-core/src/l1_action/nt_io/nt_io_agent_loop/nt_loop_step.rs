//! NT-IO AgentLoop step — 单步（纯搬移，行为零变更）。
//!
//! 内容：`distill_output` 可逆蒸馏 + 单步 helpers（`emit_final`/`task_intent`/
//! `build_request`/`output_budget_for`/`execute_tools`/`call_tool`/
//! `action_type_for_tool`/`truncate`/`trim_tool_output`/`check_tool_approval`/
//! `trim_history`/`maybe_compact_context`）。跨模块调用者改为 `pub(crate)`，行为不变。

use std::collections::HashMap;

use serde_json::Value;

use super::nt_loop_types::{AgentLoop, ToolInvocation, COMPACTION_MIN_MESSAGES, COMPACTION_SUMMARY_INPUT_MAX_TOKENS, COMPACTION_SUMMARY_MAX_TOKENS, COMPACTION_THRESHOLD_RATIO};
use crate::l0_substrate::nt_core_traits::{SecretRiskLevel, ToolOutput};
use crate::l1_action::nt_io::nt_io_output_style::{OutputStyleId, OutputStyleRegistry};
use crate::l1_action::nt_io::nt_io_provider::context_budget::{apply_context_budget, estimate_messages_tokens, estimate_tokens};
use crate::l1_action::nt_io::nt_io_provider::generation_classifier::{GenerationClassifier, TaskType};
use crate::l1_action::nt_io::nt_io_provider::types::{LlmError, LlmRequest, Message, Role, ToolCallInfo};
use crate::l1_action::nt_action_facade::nt_approval::{ActionType, PendingAction};

/// P2 可逆命令输出蒸馏 (repowise absorbed 2026-08-19, R-P79):
/// 超长工具输出压缩为 errors-first + `[ref#N]` 内联标记, 可展开回原文。
///
/// 蒸馏策略 (对齐 repowise "压缩再给模型读" 机制):
/// 1. **errors-first** — 错误/警告/失败行前置, 模型先看问题。
/// 2. **尾部保留** — 最后几行 (exit code / 摘要) 保留, 结尾状态不失真。
/// 3. **`[ref#N]` 内联标记** — 中间省略区以 `[ref#N]` 占位, 语义上
///    可 expand 回完整原文 (原文全量在 `ToolInvocation.output` / tool_log)。
/// 4. **行级硬预算** — 按行裁剪并逐行累积 `estimate_tokens`, 输出总和
///    严格 ≤ max_tokens (无逐段拼接超支问题, BPE/tiktoken 亦成立)。
///
/// 输出总是 ≤ max_tokens 估算预算; 输入未超限时原样返回 (零开销快路径)。
pub(crate) fn distill_output(content: &str, max_tokens: usize) -> String {
    if estimate_tokens(content) <= max_tokens {
        return content.to_string();
    }

    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return content.to_string();
    }

    // 尾部保留: 最多保留尾部行 (exit code / 摘要)。
    const TAIL_LINES: usize = 3;
    let tail_start = lines.len().saturating_sub(TAIL_LINES);
    let tail = &lines[tail_start..];

    // errors-first: 从全部行 (含尾部区) 提取错误/警告/失败行, 集中前置。
    let is_errorish = |l: &str| {
        let low = l.to_lowercase();
        low.contains("error") || low.contains("fail") || low.contains("denied")
            || low.contains("warning") || low.contains("exception") || low.contains("panic")
            || low.contains("traceback") || low.contains("exit code") || low.contains("fatal")
    };
    let mut error_lines: Vec<&str> = Vec::new();
    let mut plain_lines: Vec<&str> = Vec::new();
    for l in lines.iter() {
        if is_errorish(l) {
            error_lines.push(l);
        } else {
            plain_lines.push(l);
        }
    }
    // body 段取普通行 (不含已提取的尾部行); tail 段仅保留尾部普通行。
    let tail_plain: Vec<&str> = tail.iter().copied().filter(|l| !is_errorish(l)).collect();

    // 行级硬预算: 预算内逐段尽力装入, 段间以结构头分隔; 超支即停。
    // 段优先级: errors 段 (错误行) > tail 段 (exit code/摘要, 预留固定额) >
    // body 段 (用剩余预算, 以 [ref#N] 标记省略)。
    let mut out = String::new();
    let mut spent = 0usize;

    // tail 预留: 尾部状态行享有固定预算, 保证 exit code/摘要不丢。
    let tail_reserve = if tail_plain.is_empty() {
        0
    } else {
        (max_tokens as f64 * 0.20).floor() as usize
    };

    // errors 段: 优先装入全部错误行。
    if !error_lines.is_empty() {
        out.push_str("## errors\n");
        spent += estimate_tokens("## errors\n");
        for l in &error_lines {
            let line_cost = estimate_tokens(l) + 1; // 行尾换行
            if spent + line_cost + tail_reserve > max_tokens {
                break;
            }
            out.push_str(l);
            out.push('\n');
            spent += line_cost;
        }
    }

    // tail 段: 尾部状态行 (exit code / 摘要) 尽预留预算装入。
    if !tail_plain.is_empty() {
        if !out.is_empty() {
            out.push('\n');
            spent += 1;
        }
        out.push_str("## tail\n");
        spent += estimate_tokens("## tail\n");
        for l in &tail_plain {
            let line_cost = estimate_tokens(l) + 1;
            if spent + line_cost > max_tokens {
                break;
            }
            out.push_str(l);
            out.push('\n');
            spent += line_cost;
        }
    }

    // body 段: 用剩余预算装入普通行, 以 `[ref#N]` 标记省略。
    // 标记开销在装入行前预留 — 保证省略标记不会被兜底截断切掉。
    const REF_MARK: &str = "[ref#1] body 中段省略 (原文见 tool_log)";
    let body_plain: Vec<&str> = plain_lines
        .iter()
        .copied()
        .filter(|l| !tail_plain.contains(l))
        .collect();
    if !body_plain.is_empty() && spent < max_tokens {
        if !out.is_empty() {
            out.push('\n');
            spent += 1;
        }
        out.push_str("## body\n");
        spent += estimate_tokens("## body\n");
        // 预留省略标记 (仅当 body 行未全装时)
        let mark_cost = estimate_tokens(REF_MARK) + 1;
        let mut placed = 0usize;
        for l in &body_plain {
            let line_cost = estimate_tokens(l) + 1;
            let will_truncate = placed + 1 < body_plain.len();
            let reserve = if will_truncate { mark_cost } else { 0 };
            if spent + line_cost + reserve > max_tokens {
                break;
            }
            out.push_str(l);
            out.push('\n');
            spent += line_cost;
            placed += 1;
        }
        if placed < body_plain.len() {
            // 省略标记: 原文全量在 tool_log 可展开。
            out.push_str(REF_MARK);
            out.push('\n');
        }
    }

    // 不变量兜底: 任何极端 BPE 情况下仍强制截断到预算。
    // 注意不能依赖 truncate_preserving — 其内部用保守 char 估算,
    // 在 tiktoken 精确计数下可能仍超预算。改为逐字符裁剪直至精确达标。
    if estimate_tokens(&out) > max_tokens {
        // 保留头部信息密度: 从尾部逐步裁减到预算内。
        // ⛔⛔ 死循环修复（2026-10-07，对齐 neobot `nt_output_distill.rs` 已修范式 `8196dd10`）：
        // 原写法「truncate 到 0.7×len 后 push 固定 11 字符标记」在 `len≈50` 形成不动点 ⇒ 永不退出。
        // 修法：每轮强制砍字符 + 收敛 guard，不依赖「乘 0.7」启发式。
        const TAIL_MARK: &str = "…[truncated]…";
        if estimate_tokens(TAIL_MARK) > max_tokens {
            return "…[output exceeds budget, see tool_log]…".to_string();
        }
        let mut clipped = out.clone();
        let mut guard = clipped.len() + 16;
        while estimate_tokens(&clipped) > max_tokens {
            let budget_bytes = max_tokens.saturating_mul(4).max(8);
            if clipped.len() <= budget_bytes && estimate_tokens(&clipped) <= max_tokens {
                break;
            }
            if guard == 0 {
                break;
            }
            guard -= 1;
            let keep = clipped
                .char_indices()
                .nth(clipped.len() * 3 / 4)
                .map(|(i, _)| i)
                .unwrap_or(0);
            if keep == 0 {
                break;
            }
            clipped.truncate(keep);
            clipped.push_str(TAIL_MARK);
        }
        if estimate_tokens(&clipped) > max_tokens {
            // 兜底极端: 预算太小连头部都装不下, 返回省略标记。
            return "…[output exceeds budget, see tool_log]…".to_string();
        }
        clipped
    } else {
        out
    }
}

impl AgentLoop {
    pub(crate) fn emit_final(&mut self, text: &str) -> Result<String, LlmError> {
        let styled = if self.style == OutputStyleId::Plain {
            text.to_string()
        } else {
            let reg = self
                .style_registry
                .get_or_insert_with(|| std::sync::Arc::new(OutputStyleRegistry::new()));
            reg.apply(self.style, text)
        };
        // G27 输出纪律治理: 每条最终输出经 OutputGovernor 检查, 报告附于本对象供观测。
        let reg = self
            .style_registry
            .get_or_insert_with(|| std::sync::Arc::new(OutputStyleRegistry::new()));
        self.last_governance = Some(reg.govern(&styled));
        self.messages.push(Message::new(Role::Assistant, &styled));
        self.trim_history();
        Ok(styled)
    }

    /// 当前任务意图 (agent-vision-toolkit 吸收): 由最近一条用户消息 + 目标模型
    /// 推导, 注入视觉分析器使其观察贴合当前目标。无历史时退化为模型名。
    pub(crate) fn task_intent(&self) -> String {
        let latest = self
            .messages
            .iter()
            .rev()
            .find(|m| m.role == Role::User)
            .map(|m| m.content.clone())
            .unwrap_or_default();
        let latest = latest.trim();
        if latest.is_empty() {
            format!("model:{}", self.model)
        } else {
            latest.chars().take(200).collect()
        }
    }

    pub(crate) fn build_request(&self) -> LlmRequest {
        let tools = self
            .tools
            .iter()
            .map(|t| {
                let def = t.to_def();
                crate::l1_action::nt_io::nt_io_provider::types::Tool {
                    name: def.name,
                    description: def.description,
                    input_schema: def.input_schema,
                }
            })
            .collect();

        // 上下文 token 预算 (R-P 吸收 Headroom/RTK): 在克隆上压缩, 不污染持久历史。
        // W1.1 (batch3, arxiv 2608.22752 Compaction Cliff): 断崖事件 = 大上下文被
        // 压缩至保留率 <35% — 下游任务成功率坍缩前兆, 生产路径告警留痕。
        let mut messages = self.messages.clone();
        let budget_result =
            apply_context_budget(&mut messages, self.context_token_budget);
        if budget_result.is_cliff {
            log::warn!(
                "compaction cliff: retention={:.1}% evicted={} truncated={} \
                 — 任务定义锚点保留, 建议收窄本轮工具输出或提升预算",
                budget_result.retention_ratio() * 100.0,
                budget_result.messages_evicted,
                budget_result.tool_outputs_truncated
            );
        }

        // P0-4 prefix caching: 稳定前缀 = 除末条 (当前请求) 外的全部历史。
        // ReAct 每轮重发时该前缀命中 provider 缓存, 成本趋近增量。
        let cacheable_prefix_tokens = if messages.len() > 1 {
            Some(estimate_messages_tokens(&messages[..messages.len() - 1]))
        } else {
            messages
                .first()
                .map(|m| estimate_tokens(&m.content))
        };

        LlmRequest {
            model: self.model.clone(),
            messages,
            temperature: Some(0.7),
            // P2-B4: 输出端约束 — 按任务类型派生 max_tokens。
            max_tokens: self.output_budget_for(),
            tools,
            image_data: None,
            thinking_budget: None,
            provider_params: HashMap::new(),
            constraint_json: None,
            structured_output: None,
            cacheable_prefix_tokens,
        }
    }

    /// P2-B4: 输出端约束 — 按任务类型派生输出 token 预算。
    ///
    /// 用 F6 GenerationClassifier 的关键词检测 (确定性, 零 LLM 成本) 对末条用户
    /// 消息分类: 短任务 (摘要/抽取/工具) 不必预留满额预算, 编码任务给足——省输出 token
    /// 同时避免小任务超时。默认仍保持 4096。
    fn output_budget_for(&self) -> u32 {
        let last_user = self
            .messages
            .iter()
            .rev()
            .find(|m| m.role == Role::User)
            .map(|m| m.content.clone())
            .unwrap_or_default();
        if last_user.is_empty() {
            return 4096;
        }
        let cls = GenerationClassifier::new().classify(&last_user, "");
        match cls.task_type {
            TaskType::Summarization | TaskType::Extraction | TaskType::ToolUse => 2048,
            TaskType::Code => 8192,
            TaskType::Reasoning | TaskType::Knowledge => 4096,
            _ => 4096,
        }
    }

    /// 执行一组工具调用，把 assistant 调用 + 每个工具结果回填历史。
    pub(crate) async fn execute_tools(&mut self, calls: &[ToolCallInfo]) -> Result<(), LlmError> {
        // 1. 回填 assistant 的 tool_calls 消息（OpenAI 语义要求）。
        let assistant_msg = Message::assistant_with_calls("", calls.to_vec());
        self.messages.push(assistant_msg);

        // 每个 tool call 推进一个金丝雀观察窗口。位置照抄 plur `server.ts:336`
        // 「每次 tool call = 一个 turn」。
        //
        // 放这里才诚实：这是本仓唯一「模型请求了能力并被派发」的汇聚点，
        // 所以窗口推进衡量的是「模型真的在用能力」。
        // 刻意不在 `turn()` 入口打 tick —— 那会把「用户问了一句但模型没调
        // 任何能力」也算成一轮，窗口被无关轮次灌水，阈值形同虚设。
        for _call in calls {
            neotrix_neobot::nt_capability_canary::tick(&self.canary_session);
        }

        // 2. 逐个执行。
        for call in calls {
            let args: Value = serde_json::from_str(&call.arguments).unwrap_or(Value::Null);
            let result = self.call_tool(&call.name, &args);
            let content = match &result {
                Ok(ToolOutput { success, content }) => {
                    if *success {
                        content.clone()
                    } else {
                        format!("TOOL_ERROR: {}", content)
                    }
                }
                Err(e) => format!("TOOL_ERROR: {}", e),
            };
            // 完整输出进 tool_log 供审计, 回填历史经 trim_tool_output 截断。
            let history_content = self.trim_tool_output(&content);
            self.tool_log.push(ToolInvocation {
                name: call.name.clone(),
                arguments: call.arguments.clone(),
                success: result.is_ok(),
                output: content.clone(),
            });
            if let Some(h) = &self.tool_hook {
                h(&call.name, result.is_ok());
            }
            self.messages.push(Message::tool(&history_content, &call.id));
        }
        self.trim_history();
        Ok(())
    }

    pub(crate) fn call_tool(&self, name: &str, args: &Value) -> Result<ToolOutput, String> {
        // P0-3 pre-tool-use secret 扫描 (吸收 sonarqube-cli 防泄漏模式):
        // 工具调用前扫描参数, 发现 Critical/High 凭据即阻断 — 防止密钥/令牌
        // 经工具参数泄漏给外部服务或写入日志。对应 sonarqube-cli
        // "detect secrets before they leak" 的 pre-tool-use hook 语义。
        let args_text = args.to_string();
        // 使用 SecretScanner trait (抽象 L3 Redactor, 消除 L1→L3 直接依赖)
        if let Some(ref scanner) = self.secret_scanner {
            let (risk, hits) = scanner.analyze(&args_text);
            if risk == SecretRiskLevel::Dangerous {
                return Err(format!(
                    "[secret-guard] tool '{}' blocked: potential credential leak in args ({})",
                    name,
                    hits.join(", ")
                ));
            }
        }
        self.tools
            .iter()
            .find(|t| t.id() == name)
            .ok_or_else(|| format!("Unknown tool: {}", name))
            .and_then(|t| t.execute(args))
    }

    /// 工具名 → 审批动作类型（用于权限门禁分类）。
    /// 启发式映射: 命令执行→ShellCommand, git→GitOperation, 文件写→FileWrite/FileEdit,
    /// 其余兜底 Other{tool, args}。
    fn action_type_for_tool(name: &str, args: &Value) -> ActionType {
        let n = name.to_lowercase();
        let args_s = args.to_string();
        if n.contains("shell")
            || n.contains("exec")
            || n.contains("bash")
            || n.contains("run")
            || n.contains("command")
        {
            ActionType::ShellCommand {
                command: Self::truncate(&args_s, 120),
            }
        } else if n.contains("git") {
            ActionType::GitOperation {
                description: Self::truncate(&args_s, 120),
            }
        } else if n.contains("write")
            || n.contains("create")
            || n.contains("edit")
            || n.contains("patch")
            || n.contains("diff")
        {
            ActionType::FileEdit {
                path: name.to_string(),
                diff: Self::truncate(&args_s, 120),
            }
        } else {
            ActionType::Other {
                tool: name.to_string(),
                args: Self::truncate(&args_s, 120),
            }
        }
    }

    /// CJK 安全截断（按字符，不按字节）。
    fn truncate(s: &str, max_chars: usize) -> String {
        s.chars().take(max_chars).collect()
    }

    /// 工具输出截断 (Headroom/RTK 杠杆): 完整输出进 tool_log 供审计,
    /// 回填历史的仅保留头 60%/尾 40%, 中段折叠 — 防止巨大输出每轮重发。
    pub(crate) fn trim_tool_output(&self, content: &str) -> String {
        if self.max_tool_output_tokens > 0
            && estimate_tokens(content) > self.max_tool_output_tokens
        {
            // P2 可逆输出蒸馏 (repowise absorbed 2026-08-19, R-P79):
            // errors-first + `[ref#N]` 内联标记 — 错误行前置, 尾部保留
            // (exit code/摘要), 中间以标记替代; 原文全量在 tool_log 可还原。
            distill_output(content, self.max_tool_output_tokens)
        } else {
            content.to_string()
        }
    }

    /// 工具执行前审批门槛。
    /// - 无回调 (None)：需审批工具一律跳过，返回错误 "需审批"（默认安全）
    /// - 有回调：回调决策 approve (true) / deny (false)
    ///
    /// 锁纪律：必须在调回调前 drop(guard)（回调可能阻塞等待用户按键），
    ///
    /// 批准后再重新加锁 approve/deny。
    pub(crate) fn check_tool_approval(
        name: &str,
        args: &Value,
        on_approval: Option<&(dyn Fn(&PendingAction) -> bool + Send)>,
    ) -> Result<(), String> {
        let engine = crate::l1_action::nt_action_facade::nt_approval::global_approval();
        let action = Self::action_type_for_tool(name, args);
        let require = {
            let guard = engine.lock().map_err(|e| format!("approval lock: {}", e))?;
            guard.require_approval(&action)
        };
        if !require {
            return Ok(());
        }
        let pending = {
            let mut guard = engine.lock().map_err(|e| format!("approval lock: {}", e))?;
            guard.submit(action)
        };
        let decision = match on_approval {
            Some(cb) => cb(&pending),
            None => false,
        };
        // 批准后重新加锁 approve；deny 则无需回写（PendingAction 自然过期）
        if decision {
            let mut guard = engine.lock().map_err(|e| format!("approval lock: {}", e))?;
            let _ = guard.approve(&pending.id);
        }
        if decision {
            Ok(())
        } else {
            Err(format!("需要审批: {}", pending.description))
        }
    }

    /// 历史裁剪: 先按条数 (`max_history`), 再按 token 预算 (`context_token_budget`)。
    /// 始终保留 System 首条与末条 (当前请求), 丢最旧非 System 消息。
    pub(crate) fn trim_history(&mut self) {
        // 按条数裁剪 (硬上限)。
        while self.messages.len() > self.max_history && self.messages.len() > 2 {
            self.messages.remove(1);
        }
        // 按 token 预算裁剪: 超预算丢最旧非 System 消息, 保留末条。
        if self.context_token_budget > 0 {
            while self.messages.len() > 2
                && estimate_messages_tokens(&self.messages) > self.context_token_budget
            {
                let mut evict_at = None;
                for (idx, m) in self.messages.iter().enumerate() {
                    let is_system_head = idx == 0 && m.role == Role::System;
                    let is_last = idx == self.messages.len() - 1;
                    if !is_system_head && !is_last {
                        evict_at = Some(idx);
                        break;
                    }
                }
                match evict_at {
                    Some(idx) => {
                        self.messages.remove(idx);
                    }
                    None => break,
                }
            }
        }
    }

    /// P1-B2 双相 compaction: 当估算上下文超过预算 90% 时, 把最旧的
    /// `COMPACTION_MIN_MESSAGES+` 条消息用一次 LLM 调用折成摘要, 替换为单条
    /// Assistant 摘要消息 — 比纯驱逐保留语义, 比全部保留省 token。
    ///
    /// 纯驱逐 (trim_history) 只丢弃, 会丢信息; 本方法在驱逐前抢救语义。
    /// 压缩失败时静默回退驱逐 (不影响本轮)。
    pub(crate) async fn maybe_compact_context(&mut self) {
        if self.context_token_budget == 0 {
            return;
        }
        let budget = self.context_token_budget;
        if estimate_messages_tokens(&self.messages)
            < (budget as f64 * COMPACTION_THRESHOLD_RATIO) as usize
        {
            return;
        }
        // 可压缩区: 跳过 System 头 (idx 0) 与末条 (当前请求)。
        let compact_end = self.messages.len().saturating_sub(1);
        let compactable = compact_end.saturating_sub(1);
        if compactable < COMPACTION_MIN_MESSAGES {
            return;
        }
        // 压缩最旧一半, 保留最近一半细节 + 末条请求。
        let compact_count = compactable / 2;
        let block: Vec<String> = self.messages[1..=compact_count]
            .iter()
            .map(|m| {
                let role = match m.role {
                    Role::System => "system",
                    Role::User => "user",
                    Role::Assistant => "assistant",
                    Role::Tool => "tool",
                };
                format!("{role}: {}", m.content)
            })
            .collect();

        // 预算门：待摘要块超 `COMPACTION_SUMMARY_INPUT_MAX_TOKENS` ⇒
        // **降级为纯驱逐**（退回 `trim_history` 的驱逐路径），并把这次降级
        // 记进 COST_TRACKER —— 不记就等于「压缩白跑一次还花了钱」隐形。
        let summary_input_tokens = estimate_tokens(&block.join("\n---\n"));
        if summary_input_tokens > COMPACTION_SUMMARY_INPUT_MAX_TOKENS {
            let mut ct = match crate::l6_meta::nt_cost_tracker::COST_TRACKER.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            if ct.get_agent_account("agent-loop-compaction").is_none() {
                ct.register_agent("agent-loop-compaction", "AgentLoop compaction", None);
            }
            ct.record_degraded("agent-loop-compaction");
            // 降级 = **明确驱逐**最旧一半 + 留一行可见说明（模型要看得见
            // 「这段被驱逐了」，凭空消失会让人以为没发生过）。
            let evicted = compact_count;
            self.messages.splice(
                1..=compact_count,
                std::iter::once(Message::new(
                    Role::Assistant,
                    &format!(
                        "【上下文驱逐 (预算超限, 未摘要)】最旧 {evicted} 轮已从上下文移除；\
                         摘要被预算门拦下（详见 degraded_count）。"
                    ),
                )),
            );
            return;
        }

        let request = LlmRequest {
            model: self.model.clone(),
            messages: vec![
                Message::new(
                    Role::System,
                    "You are a conversation summarizer for an autonomous coding agent. \
                     Compress the given conversation turns into a concise but complete summary. \
                     Preserve: the overall goal, key decisions, tool results that matter, \
                     and any explicit constraints or user requirements. \
                     Output ONLY the summary, in the same language as the source turns.",
                ),
                Message::new(Role::User, &block.join("\n---\n")),
            ],
            temperature: Some(0.2),
            max_tokens: COMPACTION_SUMMARY_MAX_TOKENS,
            tools: vec![],
            image_data: None,
            thinking_budget: None,
            provider_params: HashMap::new(),
            constraint_json: None,
            structured_output: None,
            cacheable_prefix_tokens: None,
        };

        let summary = match self.backend.complete(&request).await {
            Ok(resp) => {
                // 账本纪律：摘要调用同样消耗 token ⇒ 记入 COST_TRACKER，
                // 否则整轮成本被低报（开销“悄悄加钱”）。cost 未知记 0，
                // 价格口径留给外层 CostPolicy（核心 crate 不持价目表）。
                let mut ct = match crate::l6_meta::nt_cost_tracker::COST_TRACKER.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => poisoned.into_inner(),
                };
                if ct.get_agent_account("agent-loop-compaction").is_none() {
                    ct.register_agent("agent-loop-compaction", "AgentLoop compaction", None);
                }
                ct.record_agent_cost(
                    "agent-loop-compaction",
                    0.0,
                    u64::from(resp.usage.prompt_tokens),
                    u64::from(resp.usage.completion_tokens),
                    0,
                );
                resp.content.trim().to_string()
            }
            Err(_) => {
                // 摘要失败 → 静默回退到纯驱逐。
                return;
            }
        };
        if summary.is_empty() {
            return;
        }

        // 用单条摘要消息替换最旧 block。
        self.messages.splice(
            1..=compact_count,
            std::iter::once(Message::new(
                Role::Assistant,
                &format!("【上下文摘要 (旧轮次压缩)】\n{summary}"),
            )),
        );
    }
}
