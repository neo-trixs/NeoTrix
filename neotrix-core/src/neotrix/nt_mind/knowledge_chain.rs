use std::path::PathBuf;
use serde::{Serialize, Deserialize};

use super::core::{CapabilityVector, PerformanceEvaluator};
use super::self_iterating::ReasoningBrain;
use super::memory::{ReasoningBank, ReasoningMemory};
use super::self_edit::MicroEdit;
use super::knowledge_miner::{KnowledgeMiner, MinedKnowledge};
use crate::neotrix::error::NeoTrixResult;

/// 知识链阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KnowledgeChainPhase {
    /// 发现：识别新的知识来源
    Discovery,
    /// 挖掘：分析来源提取知识
    Mining,
    /// 验证：验证知识质量
    Validation,
    /// 吸收：将知识整合到大脑
    Absorption,
    /// 存储：保存到 ReasoningBank
    Storage,
    /// 报告：生成知识增长报告
    Reporting,
}

/// 知识链 — 连接挖掘→验证→吸收→存储的完整管道
pub struct KnowledgeChain {
    pub miner: KnowledgeMiner,
    pub work_dir: PathBuf,
    pub phase: KnowledgeChainPhase,
    pub chain_history: Vec<ChainRecord>,
    pub validation_threshold: f64,
}

/// 链记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainRecord {
    pub phase: KnowledgeChainPhase,
    pub source: String,
    pub success: bool,
    pub details: String,
    pub timestamp: i64,
    pub reward: f64,
}

/// 知识链运行结果
#[derive(Debug, Clone)]
pub struct ChainRunResult {
    pub discovered: usize,
    pub mined: usize,
    pub validated: usize,
    pub absorbed: usize,
    pub stored: usize,
    pub total_reward: f64,
    pub details: Vec<String>,
}

impl KnowledgeChain {
    pub fn new(work_dir: PathBuf) -> Self {
        Self {
            miner: KnowledgeMiner::new(work_dir.clone()),
            work_dir,
            phase: KnowledgeChainPhase::Discovery,
            chain_history: Vec::new(),
            validation_threshold: 0.6,
        }
    }

    /// 初始化默认发现目标
    pub fn init_default_discovery(&mut self) {
        self.miner.enqueue_default_targets();
        self.phase = KnowledgeChainPhase::Discovery;
    }

    /// 添加自定义发现目标
    pub fn add_discovery_target(&mut self, url: &str) {
        self.miner.enqueue(url);
    }

    /// 运行完整知识链一圈
    /// 1. Discovery → 2. Mining → 3. Validation → 4. Absorption → 5. Storage → 6. Reporting
    pub fn run_chain(&mut self, brain: &mut ReasoningBrain, bank: &mut ReasoningBank) -> NeoTrixResult<ChainRunResult> {
        let mut details = Vec::new();

        // Phase 1: Discovery
        self.phase = KnowledgeChainPhase::Discovery;
        self.miner.enqueue_default_targets();
        let pending = self.miner.discovery_queue.len();
        details.push(format!("发现阶段: {} 个待处理来源", pending));

        // Phase 2: Mining
        self.phase = KnowledgeChainPhase::Mining;
        let round_result = self.miner.mine_round(brain, bank);
        let mined = round_result.mined_count;
        details.push(format!("挖掘阶段: 成功挖掘 {} 个来源", mined));

        // Phase 3: Validation
        self.phase = KnowledgeChainPhase::Validation;
        let (validated, valid_sources) = self.validate_knowledge(&round_result.sources);
        details.push(format!("验证阶段: {}/{} 通过验证", validated, round_result.sources.len()));

        // Phase 4: Absorption (验证通过的来源吸收到 brain)
        self.phase = KnowledgeChainPhase::Absorption;
        let total_reward = self.absorb_knowledge(brain, bank, &valid_sources);
        details.push(format!("吸收阶段: 总奖励 {:.3}", total_reward));

        // Phase 5: Storage (已由 mine_round 完成，补全记录)
        self.phase = KnowledgeChainPhase::Storage;
        details.push(format!("存储阶段: {} 条记忆存入 ReasoningBank", valid_sources.len()));

        // Phase 6: Reporting
        self.phase = KnowledgeChainPhase::Reporting;
        let report = self.miner.generate_report(&valid_sources);
        details.push(format!("报告:\n{}", report));

        Ok(ChainRunResult {
            discovered: pending,
            mined,
            validated,
            absorbed: validated,
            stored: validated,
            total_reward,
            details,
        })
    }

    /// 验证知识质量
    fn validate_knowledge(&self, sources: &[MinedKnowledge]) -> (usize, Vec<MinedKnowledge>) {
        let mut valid = Vec::new();
        for source in sources {
            if self.is_valid_knowledge(source) {
                valid.push(source.clone());
            }
        }
        (valid.len(), valid)
    }

    /// 单项知识验证：置信度 + 向量合理性 + 非空
    fn is_valid_knowledge(&self, knowledge: &MinedKnowledge) -> bool {
        if knowledge.confidence < self.validation_threshold {
            return false;
        }
        if knowledge.source_name.is_empty() || knowledge.source_url.is_empty() {
            return false;
        }
        let sum: f64 = knowledge.capability_vector.to_full_vector().iter().sum();
        if sum < 0.01 {
            return false;
        }
        true
    }

    /// 吸收验证通过的知识
    fn absorb_knowledge(
        &mut self,
        brain: &mut ReasoningBrain,
        bank: &mut ReasoningBank,
        sources: &[MinedKnowledge],
    ) -> f64 {
        let mut total_reward = 0.0;
        let before = brain.capability.clone();

        for knowledge in sources {
            // 注册知识来源
            brain.register_knowledge_source(&knowledge.source_name, knowledge.capability_vector.clone());

            // 应用 MicroEdits
            for edit in &knowledge.micro_edits {
                self.apply_edit(brain, edit);
            }
            brain.capability.normalize();

            // 计算奖励
            let after_score = PerformanceEvaluator::evaluate(
                &KnowledgeMiner::domain_to_task_type(&knowledge.domain),
                &brain.capability,
            );
            let before_score = PerformanceEvaluator::evaluate(
                &KnowledgeMiner::domain_to_task_type(&knowledge.domain),
                &before,
            );
            let reward = (after_score - before_score).max(0.0) * knowledge.confidence;
            total_reward += reward;

            // 记录到链历史
            self.chain_history.push(ChainRecord {
                phase: KnowledgeChainPhase::Absorption,
                source: knowledge.source_name.clone(),
                success: reward > 0.01,
                details: format!("domain={}, confidence={:.2}, reward={:.3}", knowledge.domain, knowledge.confidence, reward),
                timestamp: chrono::Utc::now().timestamp(),
                reward,
            });

            // 存储到 ReasoningBank
            let task_type = KnowledgeMiner::domain_to_task_type(&knowledge.domain);
            let memory = ReasoningMemory::new(
                &format!("KnowledgeChain: {} from {}", knowledge.source_name, knowledge.source_url),
                task_type,
                &knowledge.micro_edits,
                reward,
            );
            bank.store(memory);
        }

        total_reward
    }

    fn apply_edit(&self, brain: &mut ReasoningBrain, edit: &MicroEdit) {
        match edit {
            MicroEdit::AdjustDimension(dim, delta) => {
                if let Some(idx) = CapabilityVector::index_from_name(dim) {
                    let val = &mut brain.capability.arr_mut()[idx];
                    *val = (*val + delta).clamp(0.0, 1.0);
                }
            }
            MicroEdit::NormalizeVector => {
                brain.capability.normalize();
            }
            _ => {}
        }
    }

    /// 获取知识链状态报告
    pub fn get_status(&self) -> KnowledgeChainStatus {
        let total = self.chain_history.len();
        let success = self.chain_history.iter().filter(|r| r.success).count();
        KnowledgeChainStatus {
            total_chain_runs: total,
            success_count: success,
            total_reward: self.chain_history.iter().map(|r| r.reward).sum(),
            current_phase: self.phase,
            pending_sources: self.miner.discovery_queue.len(),
        }
    }

    /// 检查是否存在待处理的来源
    pub fn has_pending(&self) -> bool {
        !self.miner.discovery_queue.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct KnowledgeChainStatus {
    pub total_chain_runs: usize,
    pub success_count: usize,
    pub total_reward: f64,
    pub current_phase: KnowledgeChainPhase,
    pub pending_sources: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_chain_new() {
        let chain = KnowledgeChain::new(PathBuf::from("/tmp/test"));
        assert_eq!(chain.phase, KnowledgeChainPhase::Discovery);
        assert_eq!(chain.validation_threshold, 0.6);
        assert!(chain.chain_history.is_empty());
    }

    #[test]
    fn test_init_default_discovery() {
        let mut chain = KnowledgeChain::new(PathBuf::from("/tmp/test"));
        chain.init_default_discovery();
        assert!(chain.miner.discovery_queue.len() >= 10);
    }

    #[test]
    fn test_add_discovery_target() {
        let mut chain = KnowledgeChain::new(PathBuf::from("/tmp/test"));
        chain.add_discovery_target("https://github.com/foo/bar");
        assert_eq!(chain.miner.discovery_queue.len(), 1);
    }

    #[test]
    fn test_validate_knowledge_empty() {
        let chain = KnowledgeChain::new(PathBuf::from("/tmp/test"));
        let (count, valid) = chain.validate_knowledge(&[]);
        assert_eq!(count, 0);
        assert!(valid.is_empty());
    }

    #[test]
    fn test_is_valid_knowledge_low_confidence() {
        let chain = KnowledgeChain::new(PathBuf::from("/tmp/test"));
        let knowledge = MinedKnowledge {
            source_url: "url".to_string(),
            source_name: "name".to_string(),
            domain: "backend".to_string(),
            capability_vector: CapabilityVector::default(),
            confidence: 0.3,
            micro_edits: vec![],
            tech_stack: vec![],
            insights: vec![],
        };
        assert!(!chain.is_valid_knowledge(&knowledge));
    }

    #[test]
    fn test_is_valid_knowledge_valid() {
        let chain = KnowledgeChain::new(PathBuf::from("/tmp/test"));
        let mut cv = CapabilityVector::default();
        cv.arr_mut()[0] = 0.5;
        let knowledge = MinedKnowledge {
            source_url: "https://github.com/test/repo".to_string(),
            source_name: "test-repo".to_string(),
            domain: "backend".to_string(),
            capability_vector: cv,
            confidence: 0.85,
            micro_edits: vec![],
            tech_stack: vec!["Rust".to_string()],
            insights: vec!["测试洞察".to_string()],
        };
        assert!(chain.is_valid_knowledge(&knowledge));
    }

    #[test]
    fn test_get_status_empty() {
        let chain = KnowledgeChain::new(PathBuf::from("/tmp/test"));
        let status = chain.get_status();
        assert_eq!(status.total_chain_runs, 0);
        assert_eq!(status.total_reward, 0.0);
    }

    #[test]
    fn test_has_pending() {
        let mut chain = KnowledgeChain::new(PathBuf::from("/tmp/test"));
        assert!(!chain.has_pending());
        chain.add_discovery_target("https://github.com/foo/bar");
        assert!(chain.has_pending());
    }

    #[test]
    fn test_knowledge_chain_phase_ordering() {
        use KnowledgeChainPhase::*;
        let phases = vec![Discovery, Mining, Validation, Absorption, Storage, Reporting];
        for (i, phase) in phases.iter().enumerate() {
            if i > 0 {
                // 验证顺序存在
                assert_ne!(*phase as isize, phases[i-1] as isize);
            }
        }
    }
}
