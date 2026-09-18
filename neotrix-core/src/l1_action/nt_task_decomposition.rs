/// Task Decomposition trait — L1 抽象，解耦 L5 实现
///
/// 定义任务拆解的接口，L5 提供具体实现。
/// 遵循依赖倒置原则：L1 定义 trait，L5 实现 trait。

/// 任务拆解建议
#[derive(Debug, Clone)]
pub struct DecomposeSuggestion {
    pub subtask: String,
    pub reasoning: String,
}

/// Task Decomposer trait — 任务拆解抽象
pub trait TaskDecomposerTrait: Send + Sync {
    /// 分析任务并返回拆解建议
    ///
    /// # Arguments
    /// * `task` - 要拆解的任务描述
    /// * `aggression` - 拆解激进度 (0.0-1.0)，值越高拆解越细
    ///
    /// # Returns
    /// * `Some(Vec<DecomposeSuggestion>)` - 拆解建议列表
    /// * `None` - 无需拆解
    fn analyze(&self, task: &str, aggression: f64) -> Option<Vec<DecomposeSuggestion>>;
}

/// 默认实现 — 简单的基于关键词的拆解
pub struct DefaultTaskDecomposer;

impl TaskDecomposerTrait for DefaultTaskDecomposer {
    fn analyze(&self, task: &str, aggression: f64) -> Option<Vec<DecomposeSuggestion>> {
        if task.is_empty() || aggression <= 0.0 {
            return None;
        }

        let lower = task.to_lowercase();
        let mut suggestions = Vec::new();

        // 简单的连词分割
        if aggression > 0.3 {
            for (conj, label) in [
                (" and ", "analysis"),
                (" but ", "contrast"),
                (" or ", "option"),
            ] {
                if let Some(pos) = lower.find(conj) {
                    let part = &task[..std::cmp::min(pos + conj.len() - 1, task.len())];
                    let rest = &task[pos + conj.len().min(task.len() - pos)..];
                    if part.len() > 10 && rest.len() > 10 {
                        suggestions.push(DecomposeSuggestion {
                            subtask: format!("Phase 1: {}", part),
                            reasoning: format!("Split on '{}' to reduce scope", label),
                        });
                        suggestions.push(DecomposeSuggestion {
                            subtask: format!("Phase 2: {}", rest),
                            reasoning: format!("Complete after Phase 1 ({})", label),
                        });
                        break;
                    }
                }
            }
        }

        if suggestions.is_empty() {
            None
        } else {
            Some(suggestions)
        }
    }
}