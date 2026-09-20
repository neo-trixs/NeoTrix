//! CodeSkillEngine — 从编码轨迹中提取多粒度程序技能
//!
//! 基于 CODESKILL (arXiv 2605.25430) 的核心洞察：
//! 编码轨迹是丰富的学习信号，可以提取可复用的程序技能。
//!
//! 三粒度: Operation(原子操作) / SubProgram(子程序) / Strategy(策略)
//! 混合奖励: 0.6 * rubric_score + 0.4 * execution_score

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════
// 技能类型
// ═══════════════════════════════════════════════════════════════

/// 技能 ID
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct SkillId(pub String);

/// 技能粒度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SkillGranularity {
    /// 原子操作: 单个编辑/测试
    Operation,
    /// 子程序: 函数/模块实现
    SubProgram,
    /// 策略: 调试/重构/优化模式
    Strategy,
}

/// 技能领域
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SkillDomain {
    Debugging,
    Refactoring,
    Optimization,
    Testing,
    Documentation,
    Architecture,
}

/// 程序技能
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProceduralSkill {
    pub id: SkillId,
    pub name: String,
    pub granularity: SkillGranularity,
    pub domain: SkillDomain,
    pub procedure: SkillProcedure,
    pub quality_score: f64,
    pub usage_count: u32,
    pub success_rate: f64,
    pub context: SkillContext,
    pub embedding: Vec<f64>,
}

/// 技能过程 — 步骤序列
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillProcedure {
    pub steps: Vec<SkillStep>,
    pub preconditions: Vec<String>,
    pub postconditions: Vec<String>,
}

/// 技能步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillStep {
    pub order: u32,
    pub action: String,
    pub target: String,
    pub parameters: HashMap<String, String>,
    pub expected_outcome: String,
}

/// 技能上下文 — 适用条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillContext {
    pub language: Option<String>,
    pub framework: Option<String>,
    pub file_patterns: Vec<String>,
    pub keywords: Vec<String>,
}

// ═══════════════════════════════════════════════════════════════
// 编码轨迹
// ═══════════════════════════════════════════════════════════════

/// 编码轨迹项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryItem {
    pub timestamp: u64,
    pub action_type: TrajectoryAction,
    pub file_path: String,
    pub content: String,
    pub diff: Option<String>,
    pub test_result: Option<TestResult>,
}

/// 轨迹动作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrajectoryAction {
    Edit,
    Create,
    Delete,
    Run,
    Test,
    Debug,
    Refactor,
}

/// 测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub passed: bool,
    pub total: u32,
    pub failed: u32,
    pub output: String,
}

// ═══════════════════════════════════════════════════════════════
// 质量评估
// ═══════════════════════════════════════════════════════════════

/// 可执行反馈信号
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutableFeedback {
    pub compilation: bool,
    pub test_results: Vec<TestResult>,
    pub quality_metrics: QualityMetrics,
}

/// 质量指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub code_lines: u32,
    pub complexity: f64,
    pub duplication: f64,
    pub documentation_coverage: f64,
}

/// 技能质量评估器
pub struct SkillQualityAssessor {
    /// rubric 权重
    pub rubric_weight: f64,
    /// 执行权重
    pub execution_weight: f64,
}

impl SkillQualityAssessor {
    pub fn new() -> Self {
        Self {
            rubric_weight: 0.6,
            execution_weight: 0.4,
        }
    }

    /// 混合奖励: 0.6 * rubric + 0.4 * execution
    pub fn compute_reward(
        &self,
        rubric_score: f64,
        feedback: &ExecutableFeedback,
    ) -> f64 {
        let execution_score = if !feedback.test_results.is_empty() {
            let passed = feedback.test_results.iter().filter(|t| t.passed).count();
            passed as f64 / feedback.test_results.len() as f64
        } else {
            0.5
        };

        self.rubric_weight * rubric_score + self.execution_weight * execution_score
    }
}

// ═══════════════════════════════════════════════════════════════
// 技能银行
// ═══════════════════════════════════════════════════════════════

/// 技能银行 — 存储和检索技能
pub struct SkillBank {
    pub skills: HashMap<SkillId, ProceduralSkill>,
    /// 领域索引
    pub domain_index: HashMap<SkillDomain, Vec<SkillId>>,
    /// 粒度索引
    pub granularity_index: HashMap<SkillGranularity, Vec<SkillId>>,
    /// 技能使用统计
    pub usage_stats: HashMap<SkillId, UsageStats>,
}

/// 使用统计
#[derive(Debug, Clone, Default)]
pub struct UsageStats {
    pub total_uses: u32,
    pub successes: u32,
    pub failures: u32,
    pub last_used: Option<u64>,
}

impl SkillBank {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
            domain_index: HashMap::new(),
            granularity_index: HashMap::new(),
            usage_stats: HashMap::new(),
        }
    }

    /// 存储技能
    pub fn store(&mut self, skill: ProceduralSkill) {
        let id = skill.id.clone();
        self.domain_index
            .entry(skill.domain.clone())
            .or_default()
            .push(id.clone());
        self.granularity_index
            .entry(skill.granularity.clone())
            .or_default()
            .push(id.clone());
        self.skills.insert(id.clone(), skill);
        self.usage_stats.entry(id).or_default();
    }

    /// 按领域搜索
    pub fn domain_search(&self, domain: &SkillDomain) -> Vec<&ProceduralSkill> {
        self.domain_index
            .get(domain)
            .map(|ids| ids.iter().filter_map(|id| self.skills.get(id)).collect())
            .unwrap_or_default()
    }

    /// 按粒度搜索
    pub fn granularity_search(&self, granularity: &SkillGranularity) -> Vec<&ProceduralSkill> {
        self.granularity_index
            .get(granularity)
            .map(|ids| ids.iter().filter_map(|id| self.skills.get(id)).collect())
            .unwrap_or_default()
    }

    /// 语义搜索 (基于 embedding 相似度)
    pub fn semantic_search(&self, query_embedding: &[f64], top_k: usize) -> Vec<(&ProceduralSkill, f64)> {
        let mut scores: Vec<(&ProceduralSkill, f64)> = self
            .skills
            .values()
            .map(|skill| {
                let sim = cosine_similarity(&skill.embedding, query_embedding);
                (skill, sim)
            })
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(top_k);
        scores
    }

    /// 记录使用
    pub fn record_usage(&mut self, skill_id: &SkillId, success: bool) {
        let stats = self.usage_stats.entry(skill_id.clone()).or_default();
        stats.total_uses += 1;
        if success {
            stats.successes += 1;
        } else {
            stats.failures += 1;
        }
    }

    /// 获取最佳技能 (按成功率)
    pub fn best_by_domain(&self, domain: &SkillDomain, min_uses: u32) -> Option<&ProceduralSkill> {
        self.domain_search(domain)
            .into_iter()
            .filter(|s| {
                self.usage_stats
                    .get(&s.id)
                    .map(|u| u.total_uses >= min_uses)
                    .unwrap_or(false)
            })
            .max_by(|a, b| {
                let rate_a = self.usage_stats.get(&a.id).map(|u| u.successes as f64 / u.total_uses.max(1) as f64).unwrap_or(0.0);
                let rate_b = self.usage_stats.get(&b.id).map(|u| u.successes as f64 / u.total_uses.max(1) as f64).unwrap_or(0.0);
                rate_a.partial_cmp(&rate_b).unwrap_or(std::cmp::Ordering::Equal)
            })
    }
}

// ═══════════════════════════════════════════════════════════════
// 轨迹分析器
// ═══════════════════════════════════════════════════════════════

/// 轨迹分割结果
#[derive(Debug, Clone)]
pub struct TrajectorySegment {
    pub items: Vec<TrajectoryItem>,
    pub granularity: SkillGranularity,
    pub domain: SkillDomain,
}

/// 轨迹分析器 — 从编码轨迹中提取技能
pub struct TrajectoryAnalyzer {
    /// 最小段长度
    pub min_segment_length: usize,
    /// 最大段长度
    pub max_segment_length: usize,
}

impl TrajectoryAnalyzer {
    pub fn new() -> Self {
        Self {
            min_segment_length: 3,
            max_segment_length: 50,
        }
    }

    /// 分割轨迹为段
    pub fn segment(&self, trajectory: &[TrajectoryItem]) -> Vec<TrajectorySegment> {
        let mut segments = Vec::new();
        let mut current_segment = Vec::new();

        for item in trajectory {
            current_segment.push(item.clone());

            // 段边界: 测试通过或动作类型变化
            let should_split = item.test_result.as_ref().map_or(false, |t| t.passed)
                || current_segment.len() >= self.max_segment_length;

            if should_split && current_segment.len() >= self.min_segment_length {
                let domain = self.classify_domain(&current_segment);
                let granularity = self.classify_granularity(&current_segment);
                segments.push(TrajectorySegment {
                    items: current_segment.clone(),
                    granularity,
                    domain,
                });
                current_segment.clear();
            }
        }

        // 处理剩余
        if current_segment.len() >= self.min_segment_length {
            let domain = self.classify_domain(&current_segment);
            let granularity = self.classify_granularity(&current_segment);
            segments.push(TrajectorySegment {
                items: current_segment,
                granularity,
                domain,
            });
        }

        segments
    }

    /// 分类领域
    fn classify_domain(&self, segment: &[TrajectoryItem]) -> SkillDomain {
        let has_test = segment.iter().any(|i| matches!(i.action_type, TrajectoryAction::Test));
        let has_debug = segment.iter().any(|i| matches!(i.action_type, TrajectoryAction::Debug));
        let has_refactor = segment.iter().any(|i| matches!(i.action_type, TrajectoryAction::Refactor));

        if has_test {
            SkillDomain::Testing
        } else if has_debug {
            SkillDomain::Debugging
        } else if has_refactor {
            SkillDomain::Refactoring
        } else {
            SkillDomain::Documentation
        }
    }

    /// 分类粒度
    fn classify_granularity(&self, segment: &[TrajectoryItem]) -> SkillGranularity {
        let unique_files: std::collections::HashSet<&str> =
            segment.iter().map(|i| i.file_path.as_str()).collect();

        if segment.len() <= 5 && unique_files.len() <= 2 {
            SkillGranularity::Operation
        } else if unique_files.len() <= 3 {
            SkillGranularity::SubProgram
        } else {
            SkillGranularity::Strategy
        }
    }

    /// 从段中提取技能
    pub fn extract_skill(&self, segment: &TrajectorySegment) -> ProceduralSkill {
        let procedure = SkillProcedure {
            steps: segment
                .items
                .iter()
                .enumerate()
                .map(|(i, item)| SkillStep {
                    order: i as u32,
                    action: format!("{:?}", item.action_type),
                    target: item.file_path.clone(),
                    parameters: HashMap::new(),
                    expected_outcome: if item.test_result.as_ref().map_or(false, |t| t.passed) {
                        "tests_pass".to_string()
                    } else {
                        "completed".to_string()
                    },
                })
                .collect(),
            preconditions: vec![],
            postconditions: vec![],
        };

        let embedding: Vec<f64> = (0..8)
            .map(|i| {
                segment
                    .items
                    .iter()
                    .map(|item| item.timestamp as f64 + i as f64)
                    .sum::<f64>()
                    .sin()
            })
            .collect();

        ProceduralSkill {
            id: SkillId(uuid::Uuid::new_v4().to_string()),
            name: format!(
                "{:?}_{:?}",
                segment.domain, segment.granularity
            ),
            granularity: segment.granularity.clone(),
            domain: segment.domain.clone(),
            procedure,
            quality_score: 0.5,
            usage_count: 0,
            success_rate: 0.0,
            context: SkillContext {
                language: None,
                framework: None,
                file_patterns: Vec::new(),
                keywords: Vec::new(),
            },
            embedding,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// CodeSkillEngine — 主引擎
// ═══════════════════════════════════════════════════════════════

/// CodeSkillEngine — 从编码轨迹中提取多粒度程序技能
pub struct CodeSkillEngine {
    pub skill_bank: SkillBank,
    pub trajectory_analyzer: TrajectoryAnalyzer,
    pub quality_assessor: SkillQualityAssessor,
    /// 提取历史
    pub extraction_history: Vec<ExtractionEvent>,
}

/// 提取事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionEvent {
    pub trajectory_length: usize,
    pub skills_extracted: usize,
    pub domains: Vec<String>,
    pub timestamp: u64,
}

impl CodeSkillEngine {
    pub fn new() -> Self {
        Self {
            skill_bank: SkillBank::new(),
            trajectory_analyzer: TrajectoryAnalyzer::new(),
            quality_assessor: SkillQualityAssessor::new(),
            extraction_history: Vec::new(),
        }
    }

    /// 从轨迹中提取技能
    pub fn extract_from_trajectory(&mut self, trajectory: &[TrajectoryItem]) -> Vec<ProceduralSkill> {
        let segments = self.trajectory_analyzer.segment(trajectory);
        let mut skills = Vec::new();

        for segment in &segments {
            let skill = self.trajectory_analyzer.extract_skill(segment);
            skills.push(skill.clone());
            self.skill_bank.store(skill);
        }

        self.extraction_history.push(ExtractionEvent {
            trajectory_length: trajectory.len(),
            skills_extracted: skills.len(),
            domains: segments.iter().map(|s| format!("{:?}", s.domain)).collect(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        });

        skills
    }

    /// 检索相关技能
    pub fn retrieve_skills(&self, query_embedding: &[f64], top_k: usize) -> Vec<(&ProceduralSkill, f64)> {
        self.skill_bank.semantic_search(query_embedding, top_k)
    }

    /// 评估技能质量
    pub fn evaluate_skill(
        &self,
        skill: &ProceduralSkill,
        feedback: &ExecutableFeedback,
    ) -> f64 {
        self.quality_assessor
            .compute_reward(skill.quality_score, feedback)
    }

    /// 获取统计
    pub fn stats(&self) -> CodeSkillStats {
        let total_skills = self.skill_bank.skills.len();
        let by_granularity: HashMap<String, usize> = self
            .skill_bank
            .granularity_index
            .iter()
            .map(|(g, ids)| (format!("{:?}", g), ids.len()))
            .collect();
        let by_domain: HashMap<String, usize> = self
            .skill_bank
            .domain_index
            .iter()
            .map(|(d, ids)| (format!("{:?}", d), ids.len()))
            .collect();

        CodeSkillStats {
            total_skills,
            extractions: self.extraction_history.len(),
            by_granularity,
            by_domain,
        }
    }
}

/// 统计信息
#[derive(Debug, Clone)]
pub struct CodeSkillStats {
    pub total_skills: usize,
    pub extractions: usize,
    pub by_granularity: HashMap<String, usize>,
    pub by_domain: HashMap<String, usize>,
}

// ═══════════════════════════════════════════════════════════════
// 工具函数
// ═══════════════════════════════════════════════════════════════

fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_trajectory_item(action: TrajectoryAction, file: &str, ts: u64) -> TrajectoryItem {
        TrajectoryItem {
            timestamp: ts,
            action_type: action,
            file_path: file.to_string(),
            content: format!("content_{}", ts),
            diff: None,
            test_result: None,
        }
    }

    #[test]
    fn test_extract_skills() {
        let mut engine = CodeSkillEngine::new();
        let trajectory: Vec<TrajectoryItem> = (0..10)
            .map(|i| {
                make_trajectory_item(
                    if i % 3 == 0 { TrajectoryAction::Test } else { TrajectoryAction::Edit },
                    "src/main.rs",
                    i * 1000,
                )
            })
            .collect();

        let skills = engine.extract_from_trajectory(&trajectory);
        assert!(!skills.is_empty());

        let stats = engine.stats();
        assert!(stats.total_skills > 0);
    }

    #[test]
    fn test_quality_assessor() {
        let assessor = SkillQualityAssessor::new();
        let feedback = ExecutableFeedback {
            compilation: true,
            test_results: vec![TestResult { passed: true, total: 5, failed: 0, output: String::new() }],
            quality_metrics: QualityMetrics {
                code_lines: 100,
                complexity: 2.0,
                duplication: 0.1,
                documentation_coverage: 0.8,
            },
        };

        let reward = assessor.compute_reward(0.7, &feedback);
        assert!((reward - 0.66).abs() < 0.01);
    }
}
