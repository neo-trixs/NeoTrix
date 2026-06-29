/// DeepAbsorptionPipeline — 吸收能力元进化
///
/// # 自身缺陷分析
/// 当前"吸收" == 单次提取 → 子弹笔记摘要。无：
/// - 结构模型构建（组件/连接/因果图）
/// - 深度门控（检测未理解的gap）
/// - 交叉授粉（跨吸收物关联映射）
/// - 缺陷映射（每个发现对应已知缺陷）
/// - 进化设计（架构变更规格说明书）
///
/// # 架构位置
/// T2(经验)层: 与 repo_understanding.rs 同级
/// 参考: PaperWise 6-section 管线, Self-Reasoning RALM (AAAI 2025),
///       Feynman AI gap detection, Dify Knowledge Pipeline
///
/// # 8阶段管线
/// 1. SurfaceScan  — 首遍阅读 + 关键声明提取
/// 2. StructuralModel — 因果/结构心智模型建立
/// 3. FeynmanGate — 深度验证（用最简语言重解释, 检测gap）
/// 4. CrossPollinate — 跨知识源连接映射
/// 5. DefectMap — 映射到已知NeoTrix缺陷列表
/// 6. EvolutionDesign — 生成架构变更规格
/// 7. Implement — 执行代码变更
/// 8. Consolidate — 写入经验树 + 更新todo
///
/// 费曼门控: 阶段3失败 → 退回阶段1重新扫描更多细节
use serde::{Deserialize, Serialize};

// ─── 核心数据结构 ───

/// 吸收目标 — 知道"我们在吸收什么"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsorptionTarget {
    /// 名称（如 "Theater of Mind / GWA 语义熵"）
    pub name: String,
    /// 源类型: paper/article/repo/concept
    pub source_type: String,
    /// 源标识符（arXiv ID / URL / GitHub repo）
    pub source_id: String,
    /// 一句话摘要
    pub tagline: String,
}

/// 吸收管线状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepAbsorptionPipeline {
    /// 吸收目标
    pub target: AbsorptionTarget,
    /// 阶段完成标志
    pub stage_surface: bool,
    pub stage_structural: bool,
    pub stage_feynman: bool,
    pub stage_cross: bool,
    pub stage_defect_map: bool,
    pub stage_evolution: bool,
    pub stage_implement: bool,
    pub stage_consolidate: bool,
    /// 最大迭代轮次（FeynmanGate失败时重试）
    pub max_iterations: usize,
    /// 当前迭代计数
    pub current_iteration: usize,

    // ── 各阶段产出 ──
    /// 阶段1: 关键声明集合
    pub key_claims: Vec<Claim>,
    /// 阶段2: 结构模型
    pub structural_model: Option<StructuralModel>,
    /// 阶段3: 费曼门检测到的gap
    pub feynman_gaps: Vec<String>,
    /// 阶段3: 通过深度(0.0-1.0)
    pub feynman_depth: f64,
    /// 阶段4: 交叉连接
    pub cross_connections: Vec<CrossConnection>,
    /// 阶段5: 缺陷映射
    pub defect_impacts: Vec<DefectImpact>,
    /// 阶段6: 进化规格
    pub evolution_specs: Vec<EvolutionSpec>,
    /// 阶段8: 经验树节点
    pub consolidated_nodes: Vec<AbsorptionNode>,

    // ── 综合度量 ──
    /// 整体理解深度 (0.0-1.0)
    pub comprehension_depth_total: f64,
    /// 结构完整性 (0.0-1.0)
    pub structural_completeness: f64,
    /// 识别到的gap数量
    pub gap_count: usize,
}

/// 关键声明 — 从源中提取的核心断言/机制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub statement: String,
    pub confidence: f64,
    pub source_quote: Option<String>,
    pub is_verified: bool,
}

/// 结构模型 — 组件-连接-因果图
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralModel {
    /// 核心组件
    pub components: Vec<Component>,
    /// 组件间连接
    pub connections: Vec<Connection>,
    /// 因果流
    pub causal_flow: Vec<String>,
    /// 输入/输出/状态
    pub io_state: IoState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    pub role: String,
    pub sub_components: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub from: String,
    pub to: String,
    pub mechanism: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoState {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub internal_state: Vec<String>,
}

/// 交叉连接 — 与已知知识的关联
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossConnection {
    pub target: String,
    pub neotrix_subsystem: String,
    pub relationship: String, // isomorphic / complementary / conflicting
    pub action: String,
}

/// 缺陷映射 — 此吸收物能修复哪些已知缺陷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectImpact {
    pub defect_id: String,
    pub defect_name: String,
    pub impact_level: String, // resolves / mitigates / informs
    pub description: String,
}

/// 进化规格 — 架构变更方案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionSpec {
    pub title: String,
    pub priority: String, // P0 / P1 / P2
    pub subsystem: String,
    pub file_path: Option<String>,
    pub description: String,
    pub key_interfaces: Vec<String>,
    pub estimated_lines: usize,
    pub status: String,
}

/// 经验树节点 — 吸收结果持久化
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsorptionNode {
    pub name: String,
    pub depth_score: f64,
    pub key_insights: Vec<String>,
    pub connections: Vec<String>,
    pub defect_impacts: Vec<String>,
    pub evolutions: Vec<String>,
    pub source: String,
}

impl DeepAbsorptionPipeline {
    /// 创建新的吸收管线
    pub fn new(target: AbsorptionTarget) -> Self {
        Self {
            target,
            stage_surface: false,
            stage_structural: false,
            stage_feynman: false,
            stage_cross: false,
            stage_defect_map: false,
            stage_evolution: false,
            stage_implement: false,
            stage_consolidate: false,
            max_iterations: 3,
            current_iteration: 0,
            key_claims: Vec::new(),
            structural_model: None,
            feynman_gaps: Vec::new(),
            feynman_depth: 0.0,
            cross_connections: Vec::new(),
            defect_impacts: Vec::new(),
            evolution_specs: Vec::new(),
            consolidated_nodes: Vec::new(),
            comprehension_depth_total: 0.0,
            structural_completeness: 0.0,
            gap_count: 0,
        }
    }

    /// 阶段1: 表面扫描 — 提取关键声明
    pub fn run_surface_scan(&mut self, claims: Vec<Claim>) {
        self.key_claims = claims;
        self.stage_surface = true;
        self.current_iteration += 1;
    }

    /// 阶段2: 结构模型构建
    pub fn run_structural_model(&mut self, model: StructuralModel) {
        self.structural_model = Some(model);
        self.stage_structural = true;
    }

    /// 阶段3: 费曼门控 — 检测理解gap
    /// 返回gap列表，为空则通过
    pub fn run_feynman_gate(&mut self, gaps: Vec<String>) -> bool {
        self.feynman_gaps = gaps;
        self.gap_count = self.feynman_gaps.len();

        let pass = self.feynman_gaps.is_empty() || self.current_iteration >= self.max_iterations;
        if pass {
            self.feynman_depth = if self.feynman_gaps.is_empty() {
                1.0
            } else {
                // 迭代耗尽但仍有gap, 标记深度
                let coverage = 1.0 - (self.feynman_gaps.len() as f64).min(10.0) / 10.0;
                coverage.max(0.3)
            };
            self.stage_feynman = true;
            true
        } else {
            self.feynman_depth = 0.5;
            false
        }
    }

    /// 阶段4: 交叉授粉
    pub fn run_cross_pollinate(&mut self, connections: Vec<CrossConnection>) {
        self.cross_connections = connections;
        self.stage_cross = true;
    }

    /// 阶段5: 缺陷映射
    pub fn run_defect_map(&mut self, impacts: Vec<DefectImpact>) {
        self.defect_impacts = impacts;
        self.stage_defect_map = true;
    }

    /// 阶段6: 进化设计 — 生成架构变更规格
    pub fn run_evolution_design(&mut self, specs: Vec<EvolutionSpec>) {
        self.evolution_specs = specs;
        self.stage_evolution = true;
    }

    /// 阶段7: 标记实现完成
    pub fn mark_implemented(&mut self) {
        self.stage_implement = true;
    }

    /// 阶段8: 合并入经验树
    pub fn run_consolidate(&mut self, nodes: Vec<AbsorptionNode>) {
        self.consolidated_nodes = nodes;
        self.stage_consolidate = true;
    }

    /// 计算综合理解深度
    pub fn compute_depth(&mut self) -> f64 {
        let mut score = 0.0;
        let mut count = 0;

        if self.stage_surface {
            let claim_quality = self.key_claims.iter().map(|c| c.confidence).sum::<f64>()
                / self.key_claims.len().max(1) as f64;
            score += claim_quality * 0.15;
            count += 1;
        }
        if self.stage_structural {
            let model_depth = self.structural_model.as_ref().map_or(0.0, |m| {
                let c = m.components.len().min(10) as f64 / 10.0;
                let conn = m.connections.len().min(10) as f64 / 10.0;
                (c + conn) / 2.0
            });
            score += model_depth * 0.25;
            count += 1;
        }
        if self.stage_feynman {
            score += self.feynman_depth * 0.30;
            count += 1;
        }
        if self.stage_cross {
            let cross_quality = self.cross_connections.len().min(8) as f64 / 8.0;
            score += cross_quality * 0.15;
            count += 1;
        }
        if self.stage_defect_map {
            let defect_quality = self.defect_impacts.len().min(5) as f64 / 5.0;
            score += defect_quality * 0.15;
            count += 1;
        }

        let depth = if count > 0 { score / count as f64 } else { 0.0 };
        self.comprehension_depth_total = depth;
        depth
    }

    /// 报告当前管线状态
    pub fn report(&self) -> AbsorptionReport {
        let done = [
            self.stage_surface,
            self.stage_structural,
            self.stage_feynman,
            self.stage_cross,
            self.stage_defect_map,
            self.stage_evolution,
            self.stage_implement,
            self.stage_consolidate,
        ];
        let stage_names = [
            "SurfaceScan",
            "StructuralModel",
            "FeynmanGate",
            "CrossPollinate",
            "DefectMap",
            "EvolutionDesign",
            "Implement",
            "Consolidate",
        ];
        let stages: Vec<StageStatus> = stage_names
            .iter()
            .zip(done.iter())
            .enumerate()
            .map(|(i, (name, &done))| StageStatus {
                name: name.to_string(),
                done,
                index: i,
            })
            .collect();

        let active_stage = stages.iter().position(|s| !s.done).unwrap_or(8);

        AbsorptionReport {
            target: self.target.clone(),
            stages,
            active_stage,
            comprehension_depth: self.comprehension_depth_total,
            gap_count: self.gap_count,
            feynman_depth: self.feynman_depth,
            evolution_count: self.evolution_specs.len(),
            consolidated_count: self.consolidated_nodes.len(),
        }
    }

    /// 管线是否完全完成
    pub fn is_complete(&self) -> bool {
        self.stage_surface
            && self.stage_structural
            && self.stage_feynman
            && self.stage_cross
            && self.stage_defect_map
            && self.stage_evolution
            && self.stage_implement
            && self.stage_consolidate
    }
}

// ─── 报告数据结构 ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsorptionReport {
    pub target: AbsorptionTarget,
    pub stages: Vec<StageStatus>,
    pub active_stage: usize,
    pub comprehension_depth: f64,
    pub gap_count: usize,
    pub feynman_depth: f64,
    pub evolution_count: usize,
    pub consolidated_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageStatus {
    pub name: String,
    pub done: bool,
    pub index: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_target() -> AbsorptionTarget {
        AbsorptionTarget {
            name: "test-absorption".into(),
            source_type: "paper".into(),
            source_id: "test/1234".into(),
            tagline: "test absorption".into(),
        }
    }

    #[test]
    fn test_pipeline_creation() {
        let p = DeepAbsorptionPipeline::new(sample_target());
        assert!(!p.is_complete());
        assert_eq!(p.target.name, "test-absorption");
    }

    #[test]
    fn test_surface_scan() {
        let mut p = DeepAbsorptionPipeline::new(sample_target());
        p.run_surface_scan(vec![Claim {
            statement: "core claim".into(),
            confidence: 0.8,
            source_quote: None,
            is_verified: true,
        }]);
        assert!(p.stage_surface);
    }

    #[test]
    fn test_structural_model() {
        let mut p = DeepAbsorptionPipeline::new(sample_target());
        p.run_surface_scan(vec![]);
        p.run_structural_model(StructuralModel {
            components: vec![Component {
                name: "C1".into(),
                role: "processor".into(),
                sub_components: vec![],
            }],
            connections: vec![],
            causal_flow: vec![],
            io_state: IoState {
                inputs: vec!["in".into()],
                outputs: vec!["out".into()],
                internal_state: vec![],
            },
        });
        assert!(p.stage_structural);
    }

    #[test]
    fn test_feynman_gate_pass() {
        let mut p = DeepAbsorptionPipeline::new(sample_target());
        p.run_surface_scan(vec![]);
        let result = p.run_feynman_gate(vec![]);
        assert!(result);
        assert!((p.feynman_depth - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_feynman_gate_retry() {
        let mut p = DeepAbsorptionPipeline::new(sample_target());
        p.run_surface_scan(vec![]);
        let result = p.run_feynman_gate(vec!["gap1".into(), "gap2".into()]);
        assert!(!result);
        assert_eq!(p.gap_count, 2);
    }

    #[test]
    fn test_feynman_gate_max_iterations() {
        let mut p = DeepAbsorptionPipeline::new(sample_target());
        p.run_surface_scan(vec![]);
        p.max_iterations = 1; // Already iterated once in run_surface_scan
        let result = p.run_feynman_gate(vec!["gap1".into()]);
        // current_iteration (1) >= max_iterations (1) → pass
        assert!(result);
    }

    #[test]
    fn test_compute_depth() {
        let mut p = DeepAbsorptionPipeline::new(sample_target());
        p.run_surface_scan(vec![
            Claim {
                statement: "c1".into(),
                confidence: 0.9,
                source_quote: None,
                is_verified: true,
            },
            Claim {
                statement: "c2".into(),
                confidence: 0.8,
                source_quote: None,
                is_verified: true,
            },
        ]);
        p.run_structural_model(StructuralModel {
            components: vec![
                Component {
                    name: "A".into(),
                    role: "a".into(),
                    sub_components: vec![],
                },
                Component {
                    name: "B".into(),
                    role: "b".into(),
                    sub_components: vec![],
                },
            ],
            connections: vec![Connection {
                from: "A".into(),
                to: "B".into(),
                mechanism: "signal".into(),
            }],
            causal_flow: vec!["A→B".into()],
            io_state: IoState {
                inputs: vec!["x".into()],
                outputs: vec!["y".into()],
                internal_state: vec!["s".into()],
            },
        });
        p.run_feynman_gate(vec![]);

        let depth = p.compute_depth();
        assert!(depth > 0.0);
        assert!(depth <= 1.0);
    }

    #[test]
    fn test_cross_pollinate_and_defect_map() {
        let mut p = DeepAbsorptionPipeline::new(sample_target());
        p.run_surface_scan(vec![]);
        p.run_cross_pollinate(vec![CrossConnection {
            target: "external".into(),
            neotrix_subsystem: "MemoryLattice".into(),
            relationship: "complementary".into(),
            action: "integrate".into(),
        }]);
        p.run_defect_map(vec![DefectImpact {
            defect_id: "CXV.5".into(),
            defect_name: "吸收元层无反馈".into(),
            impact_level: "resolves".into(),
            description: "closes feedback loop".into(),
        }]);
        assert!(p.stage_cross);
        assert!(p.stage_defect_map);
    }

    #[test]
    fn test_full_pipeline_report() {
        let mut p = DeepAbsorptionPipeline::new(sample_target());
        p.run_surface_scan(vec![]);
        p.run_structural_model(StructuralModel {
            components: vec![],
            connections: vec![],
            causal_flow: vec![],
            io_state: IoState {
                inputs: vec![],
                outputs: vec![],
                internal_state: vec![],
            },
        });
        p.run_feynman_gate(vec![]);
        p.run_cross_pollinate(vec![]);
        p.run_defect_map(vec![]);
        p.run_evolution_design(vec![]);
        p.mark_implemented();
        p.run_consolidate(vec![]);

        assert!(p.is_complete());
        let report = p.report();
        assert_eq!(report.active_stage, 8);
        assert_eq!(report.stages.len(), 8);
        assert!(report.stages.iter().all(|s| s.done));
    }
}
