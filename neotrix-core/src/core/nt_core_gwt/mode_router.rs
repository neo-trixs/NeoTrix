// P3: ModeRouter (吸收 dsh-routing-suite — 任务→模式路由矩阵)
// 按任务特征(推理/代码/工具/记忆/创意)路由到执行模式, 并提供专家建议与
// 历史自校正。融入 GWT 注意力路由: 模式权重经 softmax 归一化后注入
// workspace (与 ModalityRouter 互补 — modality 管内容通道, mode 管执行范式)。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskKind {
    Reasoning,
    Coding,
    ToolUse,
    MemoryRetrieval,
    Creative,
}

impl TaskKind {
    pub fn label(&self) -> &'static str {
        match self {
            TaskKind::Reasoning => "reasoning",
            TaskKind::Coding => "coding",
            TaskKind::ToolUse => "tool-use",
            TaskKind::MemoryRetrieval => "memory",
            TaskKind::Creative => "creative",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// 深度推理: 长时间链 + 自我验证
    DeepReason,
    /// 快速反应: 单跳直接响应
    FastReact,
    /// 代码执行: 编辑/构建/测试
    CodeAgent,
    /// 知识检索: 记忆/KB 查询
    Recall,
    /// 多模态生成: 图像/音频/视频
    Generate,
}

impl ExecutionMode {
    pub fn label(&self) -> &'static str {
        match self {
            ExecutionMode::DeepReason => "deep-reason",
            ExecutionMode::FastReact => "fast-react",
            ExecutionMode::CodeAgent => "code-agent",
            ExecutionMode::Recall => "recall",
            ExecutionMode::Generate => "generate",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeDecision {
    pub task_kind: TaskKind,
    pub mode: ExecutionMode,
    pub confidence: f64,
    pub alternatives: Vec<(ExecutionMode, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeRouter {
    /// 静态路由表: 任务类型 → 候选模式及初始权重
    routes: HashMap<TaskKind, Vec<(ExecutionMode, f64)>>,
    /// 历史反馈: (task_kind, mode) → 成功率
    feedback: HashMap<(TaskKind, ExecutionMode), (u64, u64)>,
}

impl Default for ModeRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl ModeRouter {
    pub fn new() -> Self {
        let mut routes = HashMap::new();
        routes.insert(
            TaskKind::Reasoning,
            vec![
                (ExecutionMode::DeepReason, 0.85),
                (ExecutionMode::FastReact, 0.15),
            ],
        );
        routes.insert(
            TaskKind::Coding,
            vec![
                (ExecutionMode::CodeAgent, 0.90),
                (ExecutionMode::DeepReason, 0.10),
            ],
        );
        routes.insert(
            TaskKind::ToolUse,
            vec![
                (ExecutionMode::CodeAgent, 0.70),
                (ExecutionMode::FastReact, 0.30),
            ],
        );
        routes.insert(
            TaskKind::MemoryRetrieval,
            vec![
                (ExecutionMode::Recall, 0.95),
                (ExecutionMode::FastReact, 0.05),
            ],
        );
        routes.insert(
            TaskKind::Creative,
            vec![
                (ExecutionMode::Generate, 0.80),
                (ExecutionMode::DeepReason, 0.20),
            ],
        );
        Self {
            routes,
            feedback: HashMap::new(),
        }
    }

    /// softmax 归一化候选权重 (温度 T=1)。
    fn softmax(cands: &[(ExecutionMode, f64)]) -> Vec<(ExecutionMode, f64)> {
        let max_w = cands
            .iter()
            .map(|(_, w)| *w)
            .fold(f64::NEG_INFINITY, f64::max);
        let exps: Vec<f64> = cands.iter().map(|(_, w)| (w - max_w).exp()).collect();
        let sum: f64 = exps.iter().sum();
        cands
            .iter()
            .zip(exps.iter())
            .map(|((m, _), e)| (*m, e / sum))
            .collect()
    }

    pub fn route(&self, task: TaskKind) -> ModeDecision {
        let raw = self.routes.get(&task).cloned().unwrap_or_default();
        let normalized = Self::softmax(&raw);
        let top = normalized.first().copied();
        let alternatives = normalized;
        let (mode, confidence) = top.unwrap_or((ExecutionMode::FastReact, 1.0));
        ModeDecision {
            task_kind: task,
            mode,
            confidence,
            alternatives,
        }
    }

    /// 执行后反馈 (dsh 自校正): 成功 → 强化该 (task, mode) 权重。
    pub fn record(&mut self, task: TaskKind, mode: ExecutionMode, success: bool) {
        let entry = self.feedback.entry((task, mode)).or_insert((0, 0));
        entry.0 += 1;
        if success {
            entry.1 += 1;
        }
        // 更新路由表: 成功率 > 0.5 提升权重, 否则衰减
        if let Some(cands) = self.routes.get_mut(&task) {
            if let Some((_, w)) = cands.iter_mut().find(|(m, _)| *m == mode) {
                let rate = entry.1 as f64 / entry.0 as f64;
                *w = (*w + 0.1 * (rate - 0.5)).clamp(0.05, 0.95);
            }
        }
    }

    pub fn success_rate(&self, task: TaskKind, mode: ExecutionMode) -> Option<f64> {
        self.feedback
            .get(&(task, mode))
            .map(|(n, s)| *s as f64 / *n as f64)
    }
}

impl crate::core::nt_core_self_test::SelfTest for ModeRouter {
    fn name(&self) -> &str {
        "nt_core_gwt_mode_router"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let router = ModeRouter::new();
        let d = router.route(TaskKind::Coding);
        if d.mode != ExecutionMode::CodeAgent {
            return Err(vec!["coding tasks should route to CodeAgent".into()]);
        }
        if d.confidence < 0.5 || d.confidence > 1.0 {
            return Err(vec!["confidence out of [0.5, 1.0]".into()]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_self_test::SelfTest;

    #[test]
    fn test_route_reasoning() {
        let router = ModeRouter::new();
        let d = router.route(TaskKind::Reasoning);
        assert_eq!(d.mode, ExecutionMode::DeepReason);
    }

    #[test]
    fn test_route_memory() {
        let router = ModeRouter::new();
        let d = router.route(TaskKind::MemoryRetrieval);
        assert_eq!(d.mode, ExecutionMode::Recall);
    }

    #[test]
    fn test_softmax_normalized() {
        let cands = vec![
            (ExecutionMode::DeepReason, 0.85),
            (ExecutionMode::FastReact, 0.15),
        ];
        let norm = ModeRouter::softmax(&cands);
        let sum: f64 = norm.iter().map(|(_, w)| w).sum();
        assert!((sum - 1.0).abs() < 1e-9);
        assert!(norm[0].1 > norm[1].1);
    }

    #[test]
    fn test_alternatives_present() {
        let router = ModeRouter::new();
        let d = router.route(TaskKind::Creative);
        assert_eq!(d.alternatives.len(), 2);
        assert!(d
            .alternatives
            .iter()
            .any(|(m, _)| *m == ExecutionMode::DeepReason));
    }

    #[test]
    fn test_record_adapts_weight() {
        let mut router = ModeRouter::new();
        let before = router.route(TaskKind::Coding);
        for _ in 0..5 {
            router.record(TaskKind::Coding, ExecutionMode::CodeAgent, true);
        }
        let after = router.route(TaskKind::Coding);
        assert!(
            after.confidence > before.confidence,
            "success feedback should raise weight"
        );
        assert_eq!(
            router.success_rate(TaskKind::Coding, ExecutionMode::CodeAgent),
            Some(1.0)
        );
    }

    #[test]
    fn test_unknown_task_falls_back() {
        // 无未知 task 类型, 保证软回退安全: 所有 TaskKind 都有路由
        let router = ModeRouter::new();
        for kind in [
            TaskKind::Reasoning,
            TaskKind::Coding,
            TaskKind::ToolUse,
            TaskKind::MemoryRetrieval,
            TaskKind::Creative,
        ] {
            let d = router.route(kind);
            assert!(d.confidence > 0.0);
        }
    }

    #[test]
    fn test_selftest() {
        let router = ModeRouter::new();
        assert!(router.self_test().is_ok());
    }
}
