//! 迭代验证 Agent
//! 
//! 核心循环：状态快照 → 漏洞探测 → 缺陷分类 → 补丁生成 → 应用补丁 → 验证通过 → 反馈学习

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use super::probes::{ProbeEngine, Gap, GapType, GapSeverity};
use super::patches::{PatchGenerator, Patch, PatchAction};
use super::convergence::ConvergenceChecker;
use super::state::StateSnapshot;

/// 迭代验证 Agent
pub struct IterationAgent {
    /// 当前周期
    pub cycle: u32,
    /// 最大周期数
    pub max_cycles: u32,
    /// 收敛阈值：连续N次无新漏洞
    pub convergence_threshold: u32,
    /// 当前状态快照
    pub state_snapshot: StateSnapshot,
    /// 漏洞注册表
    pub gap_registry: GapRegistry,
    /// 补丁历史
    pub patch_history: Vec<PatchRecord>,
    /// 元模式
    pub meta_patterns: Vec<MetaPattern>,
    /// 探测引擎
    pub probe_engines: Vec<Box<dyn ProbeEngine>>,
    /// 补丁生成器
    pub patch_generators: Vec<Box<dyn PatchGenerator>>,
    /// 验证器
    pub checker: ConvergenceChecker,
    /// 统计
    pub stats: IterationStats,
}

/// 漏洞注册表
#[derive(Debug, Clone, Default)]
pub struct GapRegistry {
    /// 按维度分组的漏洞
    pub gaps_by_dimension: HashMap<String, Vec<Gap>>,
    /// 按严重程度分组
    pub gaps_by_severity: HashMap<GapSeverity, Vec<Gap>>,
    /// 总漏洞数
    pub total_gaps: u32,
    /// 已修复漏洞数
    pub fixed_gaps: u32,
}

impl GapRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, gap: Gap) {
        self.total_gaps += 1;
        
        self.gaps_by_dimension
            .entry(gap.dimension.clone())
            .or_insert_with(Vec::new)
            .push(gap.clone());
        
        self.gaps_by_severity
            .entry(gap.severity.clone())
            .or_insert_with(Vec::new)
            .push(gap);
    }

    pub fn mark_fixed(&mut self, gap_id: &str) {
        self.fixed_gaps += 1;
        // 从注册表中移除已修复的漏洞
        for gaps in self.gaps_by_dimension.values_mut() {
            gaps.retain(|g| g.id != gap_id);
        }
    }

    pub fn coverage_rate(&self) -> f64 {
        if self.total_gaps == 0 {
            return 1.0;
        }
        self.fixed_gaps as f64 / self.total_gaps as f64
    }

    pub fn dimensions(&self) -> Vec<String> {
        self.gaps_by_dimension.keys().cloned().collect()
    }
}

/// 补丁记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchRecord {
    /// 周期
    pub cycle: u32,
    /// 应用的补丁
    pub patches: Vec<Patch>,
    /// 新发现的漏洞数
    pub new_gaps_found: u32,
    /// 修复的漏洞数
    pub gaps_fixed: u32,
    /// 时间戳
    pub timestamp: String,
    /// 耗时（毫秒）
    pub duration_ms: u64,
}

/// 元模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaPattern {
    /// 模式ID
    pub id: String,
    /// 模式描述
    pub description: String,
    /// 出现频率
    pub frequency: u32,
    /// 关联维度
    pub dimensions: Vec<String>,
    /// 补丁策略
    pub patch_strategy: String,
}

/// 迭代统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IterationStats {
    /// 总周期数
    pub total_cycles: u32,
    /// 总漏洞发现数
    pub total_gaps_found: u32,
    /// 总漏洞修复数
    pub total_gaps_fixed: u32,
    /// 总补丁数
    pub total_patches: u32,
    /// 平均每周期耗时
    pub avg_cycle_duration_ms: u64,
    /// 收敛状态
    pub converged: bool,
    /// 收敛周期
    pub convergence_cycle: Option<u32>,
}

/// 迭代报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationReport {
    /// 统计
    pub stats: IterationStats,
    /// 最终状态
    pub final_state: StateSnapshot,
    /// 漏洞注册表
    pub gap_registry: GapRegistry,
    /// 元模式
    pub meta_patterns: Vec<MetaPattern>,
    /// 补丁历史摘要
    pub patch_history_summary: Vec<PatchRecord>,
    /// 收敛证明
    pub convergence_proof: ConvergenceProof,
}

/// 收敛证明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceProof {
    /// 连续无漏洞次数
    pub consecutive_no_gap_cycles: u32,
    /// 覆盖的维度数
    pub covered_dimensions: u32,
    /// 总维度数
    pub total_dimensions: u32,
    /// 意识指标
    pub consciousness_phi: f64,
    pub consciousness_coherence: f64,
    /// 是否收敛
    pub converged: bool,
}

impl IterationAgent {
    /// 创建新的迭代验证 Agent
    pub fn new(max_cycles: u32, convergence_threshold: u32) -> Self {
        Self {
            cycle: 0,
            max_cycles,
            convergence_threshold,
            state_snapshot: StateSnapshot::default(),
            gap_registry: GapRegistry::new(),
            patch_history: Vec::new(),
            meta_patterns: Vec::new(),
            probe_engines: Vec::new(),
            patch_generators: Vec::new(),
            checker: ConvergenceChecker::new(convergence_threshold),
            stats: IterationStats::default(),
        }
    }

    /// 添加探测引擎
    pub fn with_probe_engine(mut self, engine: Box<dyn ProbeEngine>) -> Self {
        self.probe_engines.push(engine);
        self
    }

    /// 添加补丁生成器
    pub fn with_patch_generator(mut self, generator: Box<dyn PatchGenerator>) -> Self {
        self.patch_generators.push(generator);
        self
    }

    /// 运行迭代验证
    pub fn run(&mut self) -> IterationReport {
        let start_time = std::time::Instant::now();

        while self.cycle < self.max_cycles {
            let cycle_start = std::time::Instant::now();

            // ① 状态快照
            self.take_snapshot();

            // ② 漏洞探测
            let gaps = self.probe_gaps();

            // ③ 漏洞分类
            let classified_gaps = self.classify_gaps(gaps);

            // ④ 补丁生成
            let patches = self.generate_patches(&classified_gaps);

            // ⑤ 应用补丁
            let gaps_fixed = self.apply_patches(&patches);

            // ⑥ 验证收敛
            let new_gaps = self.count_new_gaps();
            let duration = cycle_start.elapsed().as_millis() as u64;

            let record = PatchRecord {
                cycle: self.cycle,
                patches: patches.clone(),
                new_gaps_found: new_gaps,
                gaps_fixed,
                timestamp: chrono::Utc::now().to_rfc3339(),
                duration_ms: duration,
            };
            self.patch_history.push(record);

            // 更新统计
            self.stats.total_cycles += 1;
            self.stats.total_gaps_found += new_gaps;
            self.stats.total_gaps_fixed += gaps_fixed;
            self.stats.total_patches += patches.len() as u32;

            // ⑦ 元学习
            self.learn_meta_patterns();

            // 检查收敛
            if self.checker.check(&self.patch_history, &self.gap_registry, &self.state_snapshot) {
                self.stats.converged = true;
                self.stats.convergence_cycle = Some(self.cycle);
                break;
            }

            self.cycle += 1;
        }

        // 计算平均耗时
        if self.stats.total_cycles > 0 {
            self.stats.avg_cycle_duration_ms = 
                start_time.elapsed().as_millis() as u64 / self.stats.total_cycles;
        }

        self.generate_report()
    }

    /// 状态快照
    fn take_snapshot(&mut self) {
        // TODO: 从实际系统获取状态
        self.state_snapshot = StateSnapshot::new(
            self.cycle,
            0.5 + (self.cycle as f64 * 0.001).min(0.5),
            0.5 + (self.cycle as f64 * 0.0005).min(0.5),
        );
    }

    /// 漏洞探测
    fn probe_gaps(&self) -> Vec<Gap> {
        let mut all_gaps = Vec::new();
        
        for engine in &self.probe_engines {
            let gaps = engine.probe(&self.state_snapshot, &self.gap_registry);
            all_gaps.extend(gaps);
        }
        
        all_gaps
    }

    /// 漏洞分类
    fn classify_gaps(&self, gaps: Vec<Gap>) -> Vec<Gap> {
        // 按严重程度排序：Critical > High > Medium > Low
        let mut classified = gaps;
        classified.sort_by(|a, b| b.severity.cmp(&a.severity));
        classified
    }

    /// 补丁生成
    fn generate_patches(&self, gaps: &[Gap]) -> Vec<Patch> {
        let mut patches = Vec::new();
        
        for gap in gaps {
            for generator in &self.patch_generators {
                if let Some(patch) = generator.generate(gap) {
                    patches.push(patch);
                }
            }
        }
        
        patches
    }

    /// 应用补丁
    fn apply_patches(&mut self, patches: &[Patch]) -> u32 {
        let mut fixed = 0;
        
        for patch in patches {
            if patch.confidence >= 0.7 {
                // TODO: 实际应用补丁
                // patch.apply();
                self.gap_registry.mark_fixed(&patch.gap_id);
                fixed += 1;
            }
        }
        
        fixed
    }

    /// 统计新发现的漏洞数
    fn count_new_gaps(&self) -> u32 {
        self.patch_history.last()
            .map(|r| r.new_gaps_found)
            .unwrap_or(0)
    }

    /// 元学习
    fn learn_meta_patterns(&mut self) {
        // 分析最近的补丁历史，发现元模式
        if self.patch_history.len() < 10 {
            return;
        }

        let recent = &self.patch_history[self.patch_history.len()-10..];
        
        // 统计各维度出现频率
        let mut dimension_freq: HashMap<String, u32> = HashMap::new();
        for record in recent {
            for patch in &record.patches {
                *dimension_freq.entry(patch.dimension.clone()).or_insert(0) += 1;
            }
        }

        // 发现高频模式
        for (dim, freq) in dimension_freq {
            if freq >= 5 {
                let pattern = MetaPattern {
                    id: format!("meta_{}", dim),
                    description: format!("高频漏洞维度: {}", dim),
                    frequency: freq,
                    dimensions: vec![dim],
                    patch_strategy: "预补丁".to_string(),
                };
                
                // 避免重复
                if !self.meta_patterns.iter().any(|p| p.id == pattern.id) {
                    self.meta_patterns.push(pattern);
                }
            }
        }
    }

    /// 生成报告
    fn generate_report(self) -> IterationReport {
        let proof = ConvergenceProof {
            consecutive_no_gap_cycles: self.checker.consecutive_no_gap,
            covered_dimensions: self.gap_registry.dimensions().len() as u32,
            total_dimensions: 10, // D1-D10
            consciousness_phi: self.state_snapshot.phi,
            consciousness_coherence: self.state_snapshot.coherence,
            converged: self.stats.converged,
        };

        IterationReport {
            stats: self.stats,
            final_state: self.state_snapshot,
            gap_registry: self.gap_registry,
            meta_patterns: self.meta_patterns,
            patch_history_summary: self.patch_history.into_iter().rev().take(100).collect(),
            convergence_proof: proof,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gap_registry() {
        let mut registry = GapRegistry::new();
        assert_eq!(registry.total_gaps, 0);
        assert_eq!(registry.fixed_gaps, 0);
        assert_eq!(registry.coverage_rate(), 1.0);
    }

    #[test]
    fn test_iteration_agent_creation() {
        let agent = IterationAgent::new(1000, 10);
        assert_eq!(agent.cycle, 0);
        assert_eq!(agent.max_cycles, 1000);
        assert_eq!(agent.convergence_threshold, 10);
    }
}
