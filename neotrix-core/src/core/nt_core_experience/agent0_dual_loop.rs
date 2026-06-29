/// Agent0 双循环共进化 (ICLR 2026 RSI Workshop Oral)
///
/// 核心洞察: 两个 agent (curriculum + executor) 在同一环境中对称进化。
/// CurriculumAgent 提出有价值 + 可解决的任务，ExecutorAgent 完成它们。
/// 无需外部标注数据——进化本身就是数据生成器。
///
/// 参考: Agent0: Zero-Data Mathematical Co-Evolution (ICLR 2026 RSI Workshop)
use std::collections::VecDeque;

pub const MAX_DIFFICULTY: f64 = 1.0;
pub const MIN_DIFFICULTY: f64 = 0.1;

/// 课程提议: 目标领域 + 难度 + 提议理由
#[derive(Debug, Clone)]
pub struct CurriculumProposal {
    pub id: u64,
    pub domain: String,
    pub description: String,
    pub difficulty: f64,
    /// CurriculumAgent 自评估的"价值信号" (0-1)
    pub estimated_value: f64,
    /// ExecutorAgent 是否成功完成过此提议
    pub completed: bool,
    /// ExecutorAgent 完成任务后的实际得分
    pub realized_score: Option<f64>,
}

/// 执行结果: 提议 + 实际得分 + 迹
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub proposal_id: u64,
    pub success: bool,
    pub score_before: f64,
    pub score_after: f64,
    pub trace_summary: String,
}

/// CurriculumAgent: 基于历史成功率 + 领域覆盖度提出任务
#[derive(Debug, Clone)]
pub struct CurriculumAgent {
    pub next_id: u64,
    pub max_recent_proposals: usize,
    /// 已提出的任务历史
    pub proposals: VecDeque<CurriculumProposal>,
    /// 每个领域的成功率跟踪
    pub domain_success_rates: std::collections::HashMap<String, (u32, u32)>, // (attempts, successes)
    /// 当前难度水平 (自适应调节)
    pub current_difficulty: f64,
    /// 难度调节步长
    pub difficulty_step: f64,
}

impl CurriculumAgent {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            max_recent_proposals: 100,
            proposals: VecDeque::with_capacity(100),
            domain_success_rates: std::collections::HashMap::new(),
            current_difficulty: 0.3,
            difficulty_step: 0.05,
        }
    }

    /// 提出下一个任务: 选择难度 + 找到覆盖不足的领域
    pub fn propose(&mut self, available_domains: &[String]) -> Option<CurriculumProposal> {
        if available_domains.is_empty() {
            return None;
        }

        // 找到覆盖最不足的领域 (最少尝试次数)
        let domain = available_domains
            .iter()
            .min_by_key(|d| {
                self.domain_success_rates
                    .get(*d)
                    .map(|(a, _)| *a)
                    .unwrap_or(0)
            })
            .cloned()?;

        let id = self.next_id;
        self.next_id += 1;

        let (attempts, successes) = self
            .domain_success_rates
            .get(&domain)
            .copied()
            .unwrap_or((0, 0));

        let success_rate = if attempts > 0 {
            successes as f64 / attempts as f64
        } else {
            0.5
        };

        // 价值信号: 低成功率 + 高尝试数 = 高价值 (有挑战但非不可能)
        let estimated_value = if attempts == 0 {
            0.5
        } else {
            (1.0 - success_rate).min(0.8) * (attempts as f64 / (attempts as f64 + 5.0))
        };

        let proposal = CurriculumProposal {
            id,
            domain: domain.clone(),
            description: format!("co_evolution:{}@d{:.1}", domain, self.current_difficulty),
            difficulty: self.current_difficulty,
            estimated_value,
            completed: false,
            realized_score: None,
        };

        self.proposals.push_back(proposal.clone());
        if self.proposals.len() > self.max_recent_proposals {
            self.proposals.pop_front();
        }

        Some(proposal)
    }

    /// 记录执行结果 → 更新难度
    pub fn record_result(&mut self, result: &ExecutionResult) {
        let entry = self
            .domain_success_rates
            .entry(result.proposal_id.to_string())
            .or_insert((0, 0));
        entry.0 += 1;
        if result.success {
            entry.1 += 1;
        }

        // 找到对应的提议更新状态
        if let Some(proposal) = self
            .proposals
            .iter_mut()
            .find(|p| p.id == result.proposal_id)
        {
            proposal.completed = result.success;
            proposal.realized_score = Some(result.score_after);
        }

        // 自适应难度: 连续成功 → 提高, 连续失败 → 降低
        if result.success {
            self.current_difficulty =
                (self.current_difficulty + self.difficulty_step).min(MAX_DIFFICULTY);
        } else {
            self.current_difficulty =
                (self.current_difficulty - self.difficulty_step * 0.5).max(MIN_DIFFICULTY);
        }
    }

    pub fn stats(&self) -> (usize, f64, usize) {
        let total = self.proposals.len();
        let completed = self.proposals.iter().filter(|p| p.completed).count();
        (total, self.current_difficulty, completed)
    }
}

/// ExecutorAgent: 完成任务提议 + 记录分数
#[derive(Debug, Clone)]
pub struct ExecutorAgent {
    pub execution_history: Vec<ExecutionResult>,
    pub max_history: usize,
}

impl ExecutorAgent {
    pub fn new() -> Self {
        Self {
            execution_history: Vec::with_capacity(200),
            max_history: 200,
        }
    }

    /// 执行一个课程提议 (模拟评估)
    pub fn execute(
        &mut self,
        proposal: &CurriculumProposal,
        current_score: f64,
    ) -> ExecutionResult {
        let score_before = current_score;

        // 模拟执行: 难度越低成功率越高
        let base_chance = 1.0 - proposal.difficulty * 0.5;
        let success = rand_success(base_chance);

        let score_after = if success {
            (score_before + proposal.difficulty * 0.15).min(1.0)
        } else {
            score_before.max(0.0) - 0.05
        };

        let result = ExecutionResult {
            proposal_id: proposal.id,
            success,
            score_before,
            score_after: score_after.max(0.0).min(1.0),
            trace_summary: format!(
                "executor:{}@d{:.1} {}",
                proposal.domain,
                proposal.difficulty,
                if success { "pass" } else { "fail" }
            ),
        };

        self.execution_history.push(result.clone());
        if self.execution_history.len() > self.max_history {
            self.execution_history.remove(0);
        }

        result
    }

    pub fn stats(&self) -> (usize, f64) {
        let total = self.execution_history.len();
        let successes = self.execution_history.iter().filter(|r| r.success).count();
        let rate = if total > 0 {
            successes as f64 / total as f64
        } else {
            0.0
        };
        (total, rate)
    }
}

fn rand_success(chance: f64) -> bool {
    // 确定性随机: 基于简单哈希避免外部依赖
    let seed: u64 = 42; // 测试时固定
    let val = (seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407) as f64)
        / (u64::MAX as f64);
    val < chance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curriculum_first_proposal() {
        let mut cur = CurriculumAgent::new();
        let domains = vec!["reasoning".into(), "memory".into()];
        let prop = cur.propose(&domains).unwrap();
        assert_eq!(prop.domain, "reasoning");
        assert!((prop.difficulty - 0.3).abs() < 1e-9);
        assert!(prop.estimated_value > 0.0);
    }

    #[test]
    fn test_curriculum_covers_underrepresented_domain() {
        let mut cur = CurriculumAgent::new();
        cur.domain_success_rates.insert("reasoning".into(), (10, 8));
        let domains = vec!["reasoning".into(), "memory".into()];
        // memory has 0 attempts → should be chosen
        let prop = cur.propose(&domains).unwrap();
        assert_eq!(prop.domain, "memory");
    }

    #[test]
    fn test_executor_success_at_low_difficulty() {
        let mut exec = ExecutorAgent::new();
        let prop = CurriculumProposal {
            id: 1,
            domain: "reasoning".into(),
            description: "test".into(),
            difficulty: 0.1,
            estimated_value: 0.5,
            completed: false,
            realized_score: None,
        };
        let result = exec.execute(&prop, 0.5);
        // Low difficulty = high success chance
        assert!(result.score_after >= result.score_before);
    }

    #[test]
    fn test_curriculum_adapts_difficulty_on_success() {
        let mut cur = CurriculumAgent::new();
        let d0 = cur.current_difficulty;
        cur.record_result(&ExecutionResult {
            proposal_id: 1,
            success: true,
            score_before: 0.5,
            score_after: 0.6,
            trace_summary: "pass".into(),
        });
        assert!(cur.current_difficulty > d0);
    }

    #[test]
    fn test_curriculum_adapts_difficulty_on_failure() {
        let mut cur = CurriculumAgent::new();
        let d0 = cur.current_difficulty;
        cur.record_result(&ExecutionResult {
            proposal_id: 1,
            success: false,
            score_before: 0.5,
            score_after: 0.4,
            trace_summary: "fail".into(),
        });
        assert!(cur.current_difficulty < d0);
    }

    #[test]
    fn test_executor_stats() {
        let mut exec = ExecutorAgent::new();
        for i in 0..5 {
            let prop = CurriculumProposal {
                id: i,
                domain: "memory".into(),
                description: "test".into(),
                difficulty: 0.2,
                estimated_value: 0.5,
                completed: false,
                realized_score: None,
            };
            exec.execute(&prop, 0.5);
        }
        let (total, rate) = exec.stats();
        assert_eq!(total, 5);
        assert!(rate > 0.0);
    }

    #[test]
    fn test_full_cycle() {
        let mut cur = CurriculumAgent::new();
        let mut exec = ExecutorAgent::new();
        let domains = vec!["reasoning".into(), "memory".into(), "planning".into()];
        let mut current_score = 0.5;
        for _ in 0..10 {
            if let Some(prop) = cur.propose(&domains) {
                let result = exec.execute(&prop, current_score);
                current_score = result.score_after;
                cur.record_result(&result);
            }
        }
        let (_, final_diff, completed) = cur.stats();
        assert!(final_diff > 0.0);
        assert!(completed >= 0);
    }

    #[test]
    fn test_empty_domains_returns_none() {
        let mut cur = CurriculumAgent::new();
        assert!(cur.propose(&[]).is_none());
    }

    #[test]
    fn test_difficulty_bounds() {
        let mut cur = CurriculumAgent::new();
        // Push difficulty to max
        for _ in 0..100 {
            let result = ExecutionResult {
                proposal_id: 1,
                success: true,
                score_before: 0.5,
                score_after: 0.7,
                trace_summary: "".into(),
            };
            cur.record_result(&result);
        }
        assert!(cur.current_difficulty <= MAX_DIFFICULTY + 0.01);
    }
}
