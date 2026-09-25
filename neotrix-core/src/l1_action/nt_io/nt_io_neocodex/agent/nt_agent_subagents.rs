// ── NeoCodex agent subagents: dispatch only, execution reuses super::subagent (split from agent.rs, behavior unchanged) ──

use super::NeoCodexAgent;
use super::super::subagent::{SubagentDispatch, SubagentKind};

impl NeoCodexAgent {
    /// Dispatch parallel subagents (from Claude Code fork/async/sync)
    ///
    /// P2-C2: 把父代理最近的对话 turns 压缩成有界摘要随任务一起派发——
    /// subagent 获得定向父上下文 (信息不丢失), 又不复制全量历史 (token 受控)。
    pub async fn dispatch_subagents(&mut self, tasks: Vec<(SubagentKind, String)>) {
        let cwd = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        let context_hint = SubagentDispatch::compress_context(&self.context.turns, 2000);
        let hint = if context_hint.is_empty() { None } else { Some(context_hint.as_str()) };
        self.subagent_results = SubagentDispatch::run_parallel_with_context(tasks, hint, &cwd).await;
    }
}
