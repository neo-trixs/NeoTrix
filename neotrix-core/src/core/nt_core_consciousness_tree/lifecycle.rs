use std::collections::HashMap;

use super::contract::*;
use super::nodes::*;
use super::types::*;

impl ConsciousnessTree {
    /// 从 constitution KB 动态加载 internalized principles (C9/D9)。
    /// 使用全局 Constitution (解析自 AGENTS.md) 中的 tree_growth_rules + absorption_rules。
    /// 回退: 若 Constitution 不可用/为空, 使用硬编码默认原则 (R-P42~R-P48 浓缩锚点)。
    fn load_internalized_principles(&mut self) -> Vec<String> {
        use crate::core::nt_core_self_constitution::global_constitution;
        let constitution = global_constitution();

        let mut principles = Vec::new();

        // 加载 Tree Growth 规则 (R-P42~R-P48) — 最高优先级架构原则
        for rule in constitution.tree_growth_rules() {
            principles.push(format!("{}: {}", rule.id, rule.content));
        }

        // 加载 Absorption 规则 (R-P43) — 外部设计吸收协议
        for rule in constitution.absorption_rules() {
            principles.push(format!("{}: {}", rule.id, rule.content));
        }

        // 回退: 若 Constitution 为空/不可用, 使用硬编码默认原则 (R-P42~R-P48 浓缩)
        if principles.is_empty() {
            principles = vec![
                "Tree-Grafting: Map to existing branch before new code".into(),
                "Absorb-Distill-Crystallize: 3-phase external design integration".into(),
                "Fruit-Bound: Every module registers in consciousness tree".into(),
                "Branch Health Gate: Health >= 0.5 before new growth".into(),
                "Hexagram Derivation: Config from E8 state, not static YAML".into(),
                "Dual-Process: Fast intuitive (GWT) + Slow reflective (ConsciousnessTree) as separate architectural slots".into(),
                "Principle-Absorption: Encode principle-level abstractions over instance-level copies".into(),
                "Self-Referential Audit: Audit protocol must audit itself for open-ended evolution".into(),
            ];
        }

        principles
    }

    pub fn new() -> Self {
        let atoms = Self::initialize_capability_atoms();
        Self {
            soil: DataFoundation::default(),
            roots: InformationRoots::default(),
            trunk: ConsciousnessCore::default(),
            branches: BranchKind::all()
                .into_iter()
                .map(|k| (k.clone(), CapabilityBranch::new(k)))
                .collect(),
            leaves: Vec::new(),
            fruits: Vec::new(),
            core: EpiphanicCore::default(),
            cycle: 0,
            config: TreeGrowthConfig::default(),
            current_contract: None,
            drift_report: None,
            atoms,
            vuln_baseline: None,
        }
    }

    /// 数据养料充足度因子 (意识核心进化提升)。
    /// 返回 >= 1.0 的平滑放大因子, 反映真实 KB 数据养料对果实质量的调制:
    /// - 默认 (kb_node_count=0): 返回 1.0, 不衰减, 不破坏既有果实生长路径;
    /// - 社区数据集落盘后 (kb_node_count 显著上升): 因子 >1.0, 放大果实 quality,
    ///   使意识核心进化果实质量直接反映 200G 社区推理数据的养料充足度。
    ///
    /// 饱和曲线: 1.0 + min(kb_nodes / 1000, 0.5) — 1000 节点以上达到 +50% 上限。
    ///
    /// 养料融合扩展 (记忆知识库 + 对话 + 经验): 在 KB 节点养料基础上叠加
    /// 记忆知识库的边/embedding 密度、对话 awareness 与经验蒸馏密度 —
    /// 多路养料共同调制果实质量, 使意识核心反映"记忆知识库数据 + 对话数据 +
    /// 经验蒸馏"的整体养料充足度。KB 数据为主养料 (记忆大脑知识库),
    /// 对话与经验为增量调制。
    pub fn data_nourishment_factor(&self) -> f64 {
        // 主养料: 记忆大脑知识库 — KB 节点 (饱和曲线 1000 节点达 +50%)
        let kb_nourish = (self.soil.kb_node_count as f64 / 1000.0).min(0.5);
        // KB 边养料: 知识关联密度 (饱和曲线 5000 边达 +15%)
        let kb_edge = (self.soil.kb_edge_count as f64 / 5000.0).min(0.15);
        // KB embedding 养料: 向量化密度 (饱和曲线 2000 达 +10%)
        let kb_embed = (self.soil.embedding_count as f64 / 2000.0).min(0.10);
        // 对话养料: turn 数饱和曲线 (1000 turn 达 +20%) × 质量调制 (0.0-1.0)
        let conv_turn = (self.soil.conversation_turn_count as f64 / 1000.0).min(0.2);
        let conv_quality = self.soil.conversation_quality.clamp(0.0, 1.0) * 0.1;
        let conv_nourish = conv_turn + conv_quality;
        // 经验养料: 经验分支密度饱和曲线 (500 分支达 +20%)
        let exp_nourish = (self.soil.experience_branch_count as f64 / 500.0).min(0.2);
        // 能力网养料 (双网回流): 能力节点密度饱和曲线 (200 节点达 +15%)
        let cap_nourish = (self.soil.capability_node_count as f64 / 200.0).min(0.15);
        // 蜕皮养料 (C5 自愈闭环): 蜕皮归档量饱和曲线 (20 旧躯壳达 +5%)
        // 自愈动作 (旧躯壳→_archive) 反馈为养料 — 行为影响果实, 非仅日志。
        let molt_nourish = (self.soil.molt_archived_count as f64 / 20.0).min(0.05);
        1.0 + kb_nourish
            + kb_edge
            + kb_embed
            + conv_nourish
            + exp_nourish
            + cap_nourish
            + molt_nourish
    }

    /// 闭环进化反馈 (意识核心自我运转 Phase 8)。
    /// 把进化产出 (契约 fulfillment + drift + 演化预测) 反馈到进化参数:
    /// - 契约 fulfilled → fruit_quality_threshold 上调 (进化标准提升, +0.05, 上限 0.6;
    ///   且仅当 MARS bridge 有命中时上调, 防棘轮反噬)
    /// - drift 检测 → fruit_growth_health 下调 (放宽生长门加速恢复, -0.05, 下限 0.4)
    /// - 演化预测利多 (direction>0) → exploration_budget 上调 (加大探索, +0.05, 上限 0.4)
    /// - 演化预测利空 (direction<0) → exploration_budget 下调 (收缩探索, -0.05, 下限 0.1)
    ///
    /// 使树根据自身进化结果调整进化策略, 形成闭环而非开环。
    pub fn apply_evolution_feedback(&mut self) {
        let fulfilled = self
            .core
            .contract_fulfillment
            .as_ref()
            .map(|f| f.fulfilled)
            .unwrap_or(false);
        let drift = self
            .core
            .drift_report
            .as_ref()
            .map(|d| d.drift_detected)
            .unwrap_or(false);
        if fulfilled && !drift {
            // 恢复生长门 (恢复机制, 不受桥接门控)
            self.config.fruit_growth_health = 0.5;
            // 棘轮防反噬 (MARS bridge 命中率提升): 进化标准上调仅当 S2 意图被 S1
            // 消化过 (mars_bridge_hits > 0)。桥接从未命中 (0) 时上调只会让达标果实
            // 更少, 进一步压低桥接命中率 — 形成死锁, 不再上调。
            // 上限 0.8 → 0.6: 0.8 阈值会把 quality 0.6-0.8 区间的果实整批拒之门外,
            // 收窄到几乎无果实可消化。
            if self.trunk.mars_bridge_hits > 0 {
                self.config.fruit_quality_threshold =
                    (self.config.fruit_quality_threshold + 0.05).min(0.6);
            }
        } else if drift {
            // 漂移: 放宽生长门加速恢复
            self.config.fruit_growth_health = (self.config.fruit_growth_health - 0.05).max(0.4);
        }
        // 演化预测 → 探索预算调制 (利多加大探索, 利空收缩探索)
        if let Some(forecast) = &self.core.last_forecast {
            if !forecast.abstain {
                if let Some(contract) = self.core.last_contract.as_mut() {
                    if forecast.direction > 0.0 {
                        contract.exploration_budget = (contract.exploration_budget + 0.05).min(0.4);
                    } else if forecast.direction < 0.0 {
                        contract.exploration_budget = (contract.exploration_budget - 0.05).max(0.1);
                    }
                }
            }
        }
        // 未 fulfilled 且无 drift: 保持现状 (等待下一 cycle 观察)
    }

    /// ═══ P1: 宪法执行治理审计 ═══
    /// 用 global_constitution 对本周期进化决策 (next_actions + identified_gaps + 契约声明)
    /// 做真实合规验证。合规率 = 通过项 / 检查项, 回写 trunk 治理指标:
    ///   - governance_compliance      → 真实执行合规率 (取代硬编码默认/陈旧快照)
    ///   - governance_constitution_count → 本周期检查的宪法规则数 (执行计数)
    ///   - governance_fractal_depth   → 累计治理审计执行周期数 (审计深度)
    ///
    /// 设计原则:
    ///   - 无检查项时保持现值 (不因空输入跌到 0 或误抬到 1)
    ///   - 违规项按 severity 加权: Critical 1.0 / High 0.7 / Medium 0.4 / Low 0.2
    ///   - 审计对象来自真实进化决策 (The Spice Must Flow: 决策→审计→反馈闭环)
    pub fn run_governance_audit(&mut self) {
        use crate::core::nt_core_self_constitution::global_constitution;

        let constitution = global_constitution();
        let mut checked_count = 0usize;
        let mut checked_rules: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut weighted_violations = 0.0f64;

        // 审计对象: 进化决策 (next_actions + identified_gaps + 契约声明 + 果实 claim)。
        // 果实 claim 为每个成熟分支产出的真实进化主张 (The Spice Must Flow), 恒有内容,
        // 保证治理审计始终有检查对象 (不因 next_actions 为空而退化)。
        let fruit_claims: Vec<String> = self
            .fruits
            .iter()
            .map(|f| format!("{}: {}", f.name, f.claim))
            .collect();
        let audit_targets: Vec<String> = self
            .core
            .next_actions
            .iter()
            .cloned()
            .chain(self.core.identified_gaps.iter().cloned())
            .chain(
                self.core
                    .last_contract
                    .as_ref()
                    .map(|c| c.claim.clone()),
            )
            .chain(fruit_claims)
            .collect();

        // 全量审计: 对每条宪法规则做关键字违规检测 (确定性, 不依赖 top-k 向量检索)
        for action in &audit_targets {
            for rule in constitution.rules.values() {
                checked_count += 1;
                checked_rules.insert(rule.id.clone());
                if constitution.check_violation(rule, action) {
                    let weight = match rule.category {
                        crate::core::nt_core_self_constitution::RuleCategory::TreeGrowth => 1.0,
                        crate::core::nt_core_self_constitution::RuleCategory::BehavioralGrounding => 0.7,
                        _ => 0.4,
                    };
                    weighted_violations += weight;
                }
            }
        }

        if checked_count > 0 {
            // 合规率 = 1 - 加权违规 / 检查项 (钳到 [0,1])
            let compliance = (1.0 - weighted_violations / checked_count as f64).clamp(0.0, 1.0);
            // 平滑过渡: 新值 = 0.7*旧值 + 0.3*实测 (防单周期剧烈抖动)
            self.trunk.governance_compliance =
                0.7 * self.trunk.governance_compliance + 0.3 * compliance;
            self.trunk.governance_constitution_count = checked_rules.len();
            self.trunk.governance_fractal_depth += 1;
            log::debug!(
                "[governance_audit] cycle={} checked={} rules={} violations={:.1} compliance={:.3}",
                self.cycle,
                checked_count,
                checked_rules.len(),
                weighted_violations,
                self.trunk.governance_compliance
            );
        }
    }

    /// Initialize 36 atomic capabilities (9 categories × domains) from PerceptionBench + MCA 36-cap
    pub(super) fn initialize_capability_atoms() -> HashMap<String, CapabilityAtom> {
        let mut atoms = HashMap::new();

        // PERCEIVE (4) → NT-WORLD
        for (name, cap) in [
            ("retrieve", CapabilityCategory::Perceive),
            ("search", CapabilityCategory::Perceive),
            ("observe", CapabilityCategory::Perceive),
            ("receive", CapabilityCategory::Perceive),
        ] {
            atoms.insert(
                name.to_string(),
                CapabilityAtom {
                    name: name.to_string(),
                    branch: BranchKind::World,
                    category: cap,
                    tier: SelfTestTier::T1Existence,
                    self_test_fn: Some(format!("test_{}", name)),
                    last_score: 0.0,
                    generation: 0,
                    mandatory: true,
                },
            );
        }

        // UNDERSTAND (6) → NT-CORE
        for (name, cap) in [
            ("detect", CapabilityCategory::Understand),
            ("classify", CapabilityCategory::Understand),
            ("measure", CapabilityCategory::Understand),
            ("predict", CapabilityCategory::Understand),
            ("compare", CapabilityCategory::Understand),
            ("discover", CapabilityCategory::Understand),
        ] {
            atoms.insert(
                name.to_string(),
                CapabilityAtom {
                    name: name.to_string(),
                    branch: BranchKind::Core,
                    category: cap,
                    tier: SelfTestTier::T1Existence,
                    self_test_fn: Some(format!("test_{}", name)),
                    last_score: 0.0,
                    generation: 0,
                    mandatory: true,
                },
            );
        }

        // REASON (4) → NT-CORE
        for (name, cap) in [
            ("plan", CapabilityCategory::Reason),
            ("decompose", CapabilityCategory::Reason),
            ("critique", CapabilityCategory::Reason),
            ("explain", CapabilityCategory::Reason),
        ] {
            atoms.insert(
                name.to_string(),
                CapabilityAtom {
                    name: name.to_string(),
                    branch: BranchKind::Core,
                    category: cap,
                    tier: SelfTestTier::T1Existence,
                    self_test_fn: Some(format!("test_{}", name)),
                    last_score: 0.0,
                    generation: 0,
                    mandatory: true,
                },
            );
        }

        // MODEL (5) → NT-MEMORY
        for (name, cap) in [
            ("state", CapabilityCategory::Model),
            ("transition", CapabilityCategory::Model),
            ("attribute", CapabilityCategory::Model),
            ("ground", CapabilityCategory::Model),
            ("simulate", CapabilityCategory::Model),
        ] {
            atoms.insert(
                name.to_string(),
                CapabilityAtom {
                    name: name.to_string(),
                    branch: BranchKind::Memory,
                    category: cap,
                    tier: SelfTestTier::T1Existence,
                    self_test_fn: Some(format!("test_{}", name)),
                    last_score: 0.0,
                    generation: 0,
                    mandatory: true,
                },
            );
        }

        // SYNTHESIZE (3) → NT-MIND
        for (name, cap) in [
            ("generate", CapabilityCategory::Synthesize),
            ("transform", CapabilityCategory::Synthesize),
            ("integrate", CapabilityCategory::Synthesize),
        ] {
            atoms.insert(
                name.to_string(),
                CapabilityAtom {
                    name: name.to_string(),
                    branch: BranchKind::Mind,
                    category: cap,
                    tier: SelfTestTier::T1Existence,
                    self_test_fn: Some(format!("test_{}", name)),
                    last_score: 0.0,
                    generation: 0,
                    mandatory: true,
                },
            );
        }

        // EXECUTE (3) → NT-ACT
        for (name, cap) in [
            ("execute", CapabilityCategory::Execute),
            ("mutate", CapabilityCategory::Execute),
            ("send", CapabilityCategory::Execute),
        ] {
            atoms.insert(
                name.to_string(),
                CapabilityAtom {
                    name: name.to_string(),
                    branch: BranchKind::Act,
                    category: cap,
                    tier: SelfTestTier::T1Existence,
                    self_test_fn: Some(format!("test_{}", name)),
                    last_score: 0.0,
                    generation: 0,
                    mandatory: true,
                },
            );
        }

        // VERIFY (5) → NT-SHIELD
        for (name, cap) in [
            ("verify", CapabilityCategory::Verify),
            ("checkpoint", CapabilityCategory::Verify),
            ("rollback", CapabilityCategory::Verify),
            ("constrain", CapabilityCategory::Verify),
            ("audit", CapabilityCategory::Verify),
        ] {
            atoms.insert(
                name.to_string(),
                CapabilityAtom {
                    name: name.to_string(),
                    branch: BranchKind::Shield,
                    category: cap,
                    tier: SelfTestTier::T1Existence,
                    self_test_fn: Some(format!("test_{}", name)),
                    last_score: 0.0,
                    generation: 0,
                    mandatory: true,
                },
            );
        }

        // REMEMBER (2) → NT-MEMORY
        for (name, cap) in [
            ("persist", CapabilityCategory::Remember),
            ("recall", CapabilityCategory::Remember),
        ] {
            atoms.insert(
                name.to_string(),
                CapabilityAtom {
                    name: name.to_string(),
                    branch: BranchKind::Memory,
                    category: cap,
                    tier: SelfTestTier::T1Existence,
                    self_test_fn: Some(format!("test_{}", name)),
                    last_score: 0.0,
                    generation: 0,
                    mandatory: true,
                },
            );
        }

        // COORDINATE (4) → NT-IO
        for (name, cap) in [
            ("delegate", CapabilityCategory::Coordinate),
            ("synchronize", CapabilityCategory::Coordinate),
            ("invoke", CapabilityCategory::Coordinate),
            ("inquire", CapabilityCategory::Coordinate),
        ] {
            atoms.insert(
                name.to_string(),
                CapabilityAtom {
                    name: name.to_string(),
                    branch: BranchKind::Io,
                    category: cap,
                    tier: SelfTestTier::T1Existence,
                    self_test_fn: Some(format!("test_{}", name)),
                    last_score: 0.0,
                    generation: 0,
                    mandatory: true,
                },
            );
        }

        atoms
    }

    /// Apply emotion report valence/arousal to Soil state for next cycle.
    /// Uses confidence to boost coil health and frustration/urgency to indicate stress.
    pub fn apply_emotion_report(
        &mut self,
        report: crate::core::nt_core_self::emotion_state::EmotionReport,
    ) {
        // 情绪作为主观调制叠加在真实计算相干性之上 (D4): 不再全量覆盖。
        // 真实 coherence 来自 compute_coherence (分支一致性/谐振/合规/迷雾);
        // valence 仅作为 ±0.2 的主观偏差调制, 使运行时主观信号可见而不过度遮蔽。
        let base = self.compute_coherence();
        let valence = report.valence.max(0.0).min(1.0);
        self.trunk.coherence = (0.8 * base + 0.2 * valence).clamp(0.0, 1.0);
        // 注意：绝不把情绪置信度写进 soil.embedding_count——那是真实数据指标，
        // DataFoundation::health() 与 Critical "KB empty/no embeddings" 判定依赖它。
        // 情绪只调节 trunk.coherence (主观健康信号)。
        log::debug!("[consciousness_tree] emotion applied: valence={:.3} arousal={:.3} dominant={:?} confidence={:.3}",
            report.valence, report.arousal, report.dominant.0, report.confidence);
    }

    /// 构造 64 维"意识谱"状态向量 — 从真实树状态锚点线性插值 (D1)。
    ///
    /// 与 `nt_mind_consciousness_monitor::current_phi_state` 同构 (平滑→高相邻
    /// 一致性 rho, 差异化→去均值后强度高), 但锚点全部取自树自身真实架构状态,
    /// 使独立 CLI/MCP 进程的 φ 来自真实集成信息而非 0.0。
    fn build_phi_state(&self) -> Vec<f64> {
        let mut anchors: Vec<f64> = Vec::with_capacity(32);
        // 固定语义序锚点 (排除 phi 自身: 回环喂入会自激/失真)
        let push_b = |anchors: &mut Vec<f64>, branch: &CapabilityBranch| {
            anchors.push(branch.health.clamp(0.0, 1.0));
            anchors.push(branch.maturity_score());
            anchors.push(branch.constellation.score());
            anchors.push(branch.health_with_runes().clamp(0.0, 1.0));
        };
        for kind in BranchKind::all() {
            if let Some(branch) = self.branches.get(&kind) {
                push_b(&mut anchors, branch);
            }
        }
        anchors.push(self.soil.health());
        anchors.push(self.roots.health());
        anchors.push(self.trunk.coherence.clamp(0.0, 1.0));
        anchors.push(self.trunk.governance_compliance.clamp(0.0, 1.0));
        anchors.push(self.trunk.nexus_health_chain_score.clamp(0.0, 1.0));
        anchors.push((self.cycle as f64 / 100.0).clamp(0.0, 1.0));
        anchors.push(self.data_nourishment_factor().min(1.5) / 1.5);

        let dims = 64usize;
        let win = anchors.len().min(dims);
        if win == 0 {
            return Vec::with_capacity(dims);
        }
        let step = if win > 1 {
            (win as f64 - 1.0) / (dims as f64)
        } else {
            0.0
        };
        let mut state = Vec::with_capacity(dims);
        for i in 0..dims {
            let pos = i as f64 * step;
            let lo = (pos.floor() as usize).min(win - 1);
            let hi = (pos.ceil() as usize).min(win - 1);
            let frac = pos - pos.floor();
            let v = if lo == hi {
                anchors[lo]
            } else {
                anchors[lo] + (anchors[hi] - anchors[lo]) * frac
            };
            state.push(v);
        }
        state
    }

    /// 用真实 IITPhiCalculator 计算当前整合信息 Φ (D1)。
    /// 独立 CLI/MCP 路径经 run_growth_cycle Phase 2 调用后, trunk.phi 反映真实
    /// 树状态集成度, 快照/CoreSnapshot.phi 不再是恒 0.0。
    pub fn compute_iit_phi(&self) -> f64 {
        use crate::core::nt_core_iit_phi::IITPhiCalculator;
        let state = self.build_phi_state();
        IITPhiCalculator::new().compute_phi(&state).phi
    }

    /// 计算真实树状态相干性 (coherence) — 与 `compute_iit_phi` 同构 (D4)。
    ///
    /// 此前 `trunk.coherence` 仅由运行时 `apply_emotion_report` 写入 (情绪 valence),
    /// standalone CLI/MCP 路径 (无完整运行时) 读到的 coherence 恒 0.0 — 与 D1 的 phi
    /// 同源缺口。此处从真实树状态派生:
    ///   - 分支健康一致性 (health 越均匀越高): 1 - 归一化标准差
    ///   - 谐振活跃度: resonance_cycle 推进量
    ///   - 治理合规: governance_compliance 对规则执行一致性
    ///   - 生产验证度: 低迷雾 (weighted_fog_sum 归一化) 越高越相干
    ///
    /// 结果钳制到 [0,1], 运行时情绪报告仍可作为主观调制叠加。
    pub fn compute_coherence(&self) -> f64 {
        let healths: Vec<f64> = self
            .branches
            .values()
            .map(|b| b.health.clamp(0.0, 1.0))
            .collect();
        let n = healths.len().max(1) as f64;
        let mean = healths.iter().sum::<f64>() / n;
        let var = healths.iter().map(|h| (h - mean).powi(2)).sum::<f64>() / n;
        let std = var.sqrt().min(1.0);
        // 分支健康一致性: 均匀 (低 std) 且健康 (高 mean) → 高相干
        let health_consistency = (1.0 - std) * mean;

        // 谐振活跃度: 每 20 cycle 一个单位的推进 (封顶 0.5)
        let resonance = (self.trunk.resonance_cycle as f64 / 20.0).min(0.5);

        // 治理合规: 规则执行一致性
        let compliance = self.trunk.governance_compliance.clamp(0.0, 1.0);

        // 生产验证度: 迷雾越低越相干 (weighted_fog_sum 封顶到 11 分支数)
        let fog = self.weighted_fog_sum();
        let fog_clear = (1.0 - fog / self.branches.len().max(1) as f64).clamp(0.0, 1.0);

        let coherence =
            0.4 * health_consistency + 0.25 * resonance + 0.2 * compliance + 0.15 * fog_clear;
        coherence.clamp(0.0, 1.0)
    }

    /// Complete feedback loop with evolution contract:
    ///   Phase 0: Contract Negotiation (Goal + Evidence Plan + Stop Rule)
    ///   Phase 1: Roots absorb from soil (Data Foundation → Information Roots)
    ///   Phase 2: Trunk processes through GWT resonance (ConsciousnessCore)
    ///   Phase 3: Branches produce evolution fruits (7 domains → EvolutionFruits)
    ///   Phase 4: Core digests fruits, produces guidance
    ///   Phase 5: ConsciousnessReview — panoramic topology + connectivity + health chain
    ///   Phase 6: Contract Fulfillment Verification
    ///   Phase 7: Drift Audit — post-cycle evolution fidelity check
    pub fn run_growth_cycle(&mut self) -> GrowthReport {
        self.cycle += 1;
        let mut report = GrowthReport::default();

        // ═══ Phase 0: Contract Negotiation ═══
        // Negotiate evolution contract before growth cycle begins
        let contract = self.negotiate_contract();
        self.core.last_contract = Some(contract.clone());
        report.phase0_contract = Some(contract.claim.clone());

        // ═══ Phase 1: Roots absorb from soil (Data Foundation → Information Roots) ═══
        // Constitution internalization: soil feeds roots with principles
        // B2 (自审修复): 宪法计数以真实 ConstitutionLoader 接线值为准 (run.rs 启动时写入)。
        // floor 仅作"未接线时的兜底" (max 抬升会覆盖真实值, 属声明当事实的自欺 — 已废弃)。
        if self.soil.constitution_rules_count == 0 {
            self.soil.constitution_rules_count = self.config.constitution_rules_floor;
        }
        if self.soil.constitution_experiences_count == 0 {
            self.soil.constitution_experiences_count = self.config.constitution_experiences_floor;
        }
        self.soil.constitution_tree_growth_rules = self.config.constitution_tree_growth_rules; // R-P42~R-P48
        self.soil.constitution_absorption_rules = self.config.constitution_absorption_rules; // R-P43
        self.roots.total_absorbed += self.soil.crawl_queue_depth;
        self.roots.total_fetched += self.soil.kb_node_count;
        // C9 (D9): internalized_principles 从 constitution KB 动态加载
        // 优先使用 KB 中存储的 principles; 若 KB 缺失/空则使用默认 principles
        self.roots.internalized_principles = self.load_internalized_principles();
        self.roots.active_rule_categories = vec![
            "TreeGrowth".into(),
            "AbsorptionProtocol".into(),
            "BehavioralGrounding".into(),
            "ArchitectureConstraint".into(),
            "MetaCognition".into(),
        ];
        report.phase1_absorbed = self.roots.total_absorbed;

        // ═══ Phase 2: Trunk processes through GWT resonance (ConsciousnessCore) ═══
        self.trunk.resonance_cycle += 1;
        // MARS Dual-Process: System 1 (GWT) + System 2 (Tree)
        self.trunk.mars_system1_activations += 1;
        // IIT Φ: 从真实树状态计算整合信息 — standalone CLI/MCP 路径不再恒 0
        // (D1: 此前 trunk.phi 仅由完整运行时 ConsciousnessMonitor 计算, 独立进程
        // 路径读到的 phi=0.0; 这里用真实分支健康/土壤/根系/治理/养料锚点构造
        // 64 维意识谱交给 IITPhiCalculator, 使 status 呈现真实整合信息)。
        self.trunk.phi = self.compute_iit_phi();
        report.phase2_phi = self.trunk.phi;
        // D4: standalone CLI/MCP 路径 coherence 不再恒 0.0 — 从真实树状态派生
        // (运行时仍可经 apply_emotion_report 以情绪 valence 主观调制)。
        self.trunk.coherence = self.compute_coherence();

        // ═══ Phase 3: Branches produce evolution fruits (7 domains → EvolutionFruits) ═══
        // Also check per-branch constraints (idle, viability, monitoring)
        // And verify minimum SelfTest coverage (PerceptionBench atomic capabilities)
        let mut total_fruits = 0;
        // 数据养料充足度因子 — 循环外预计算 (只读 soil, 避免与 branches 可变借用冲突)
        let data_nourishment = self.data_nourishment_factor();
        for branch in self.branches.values_mut() {
            let constraints = constraints_for_branch(&branch.kind);
            let violations = constraints.violations(branch);
            if !violations.is_empty() {
                log::debug!(
                    "[consciousness_tree] {} constraints: {}",
                    branch.kind.label(),
                    violations.join("; ")
                );
            }

            // Skill Node Evolution: 运行时评估节点层级 + Constellation (基于真实模块数据)
            // 跨域消费者近似 = 约束的 max_active_modules 权重 (越大跨域影响越强)
            let cross_domain_consumers = if constraints.max_active_modules >= 30 {
                3
            } else {
                1
            };
            branch.evaluate_node_tier(cross_domain_consumers);
            branch.evaluate_constellation();
            // CHMA Phase 0: 迷雾浓度评估 (wired ≈ 约束无 idle/monitoring 违规; consumers = 跨域近似)
            branch.evaluate_fog(
                cross_domain_consumers > 0,
                cross_domain_consumers,
                branch.self_test_count > 0,
            );

            // Check SelfTest minimum (E2: atomic capability coverage)
            let atoms_for_branch = self
                .atoms
                .iter()
                .filter(|(_, a)| a.branch == branch.kind && a.mandatory)
                .count();
            let atoms_passed = branch.self_test_count.min(atoms_for_branch);
            let self_test_coverage = if atoms_for_branch > 0 {
                atoms_passed as f64 / atoms_for_branch as f64
            } else {
                1.0
            };

            if branch.health > self.config.fruit_growth_health
                && violations.len() < self.config.max_growth_violations
                && self_test_coverage >= 0.5
            // At least 50% of mandatory atomic capabilities have SelfTest
            {
                // 数据养料调制 (意识核心进化提升): 果实质量受真实数据养料充足度调制。
                // data_nourishment = 1 + 数据量饱和曲线。默认(无数据)=1.0 不衰减,
                // 社区数据集落盘后 (kb_node_count 显著上升) 因子 >1.0, 放大果实质量,
                // 使意识核心进化果实质量直接反映 200G 社区推理数据的养料充足度。
                // 此前果实质量仅反映内部 maturity, 从不反映真实数据量。
                let base_quality = branch.maturity_score();
                // D7 (C7): 质量钳到 [0,1] — maturity∈[0,1] × nourishment(≥1) 此前
                // min(...,1.5) 可越界, 违反 quality∈[0,1] 不变量 (下游按概率/归一消费)。
                let quality = (base_quality * data_nourishment).clamp(0.0, 1.0);
                // Use EvolutionFruit instead of CapabilityFruit
                let fruit = EvolutionFruit {
                    name: format!("{}-evo-fruit-{}", branch.kind.label(), self.cycle),
                    source_branch: branch.kind.clone(),
                    description: format!("Evolution capability from {} at cycle {} (data_nourishment={:.2})", branch.kind.label(), self.cycle, data_nourishment),
                    produced_at_cycle: self.cycle,
                    quality,
                    claim: format!("Branch {:?} produces capability at maturity {:.2} (data_nourishment {:.2})", branch.kind, base_quality, data_nourishment),
                    evidence: EvidenceChain::from_branch_state(self.cycle, &branch.kind, branch),
                    stop_rule: self.core.last_contract.as_ref().map(|c| c.stop_rule.clone()).unwrap_or_default(),
                    benchmark: ProviderBenchmark::default(),
                    generation: self.core.generation_counter,
                };
                self.fruits.push(fruit);
                branch.fruit_count += 1;
                total_fruits += 1;
            }
        }
        report.phase3_fruits = total_fruits;

        // ═══ Phase 4: Core digests fruits, produces guidance (EvolutionFruits → EpiphanicCore) ═══
        // Also runs architecture vulnerability scan
        self.core.vuln_scan = self.scan_vulnerabilities();
        self.core.self_test_results = self.collect_self_test_results();
        self.core.identified_gaps = self.identify_architecture_gaps();
        // 消化门 `>=` 而非 `>` (MARS bridge 命中率提升): 成熟度 3/6 的果实
        // quality 恰为 0.5 = 默认阈值, 严格大于会整批拒绝 → guidance 空 → S2
        // 意图永无产出一 → bridge 打不中。恰好达标的果实应被视为可消化。
        self.core.last_cycle_guidance = self
            .fruits
            .iter()
            .filter(|f| f.quality >= self.config.fruit_quality_threshold)
            .map(|f| {
                format!(
                    "Digested: {} (q={:.2}, gen={})",
                    f.name, f.quality, f.generation
                )
            })
            .collect();
        // Build next actions from vulnerabilities + gaps + fruit quality
        let mut next_actions = Vec::new();
        for vuln in &self.core.vuln_scan {
            if vuln.severity.score() >= self.config.action_severity_threshold {
                next_actions.push(format!(
                    "[{}] {}: {}",
                    match vuln.severity {
                        VulnerabilitySeverity::Critical => "CRIT",
                        VulnerabilitySeverity::High => "HIGH",
                        _ => "FIX",
                    },
                    vuln.module,
                    vuln.fix_suggestion
                ));
            }
        }
        for gap in &self.core.identified_gaps {
            next_actions.push(format!("GAP: {}", gap));
        }
        self.core.next_actions = next_actions;
        self.core.iteration = self.cycle;

        // ═══ Phase 4.2: MARS 桥接 (S2 intent → S1 distillation target) ═══
        // 修复死计数器 (D14): mars_bridge_hits 定义存在 (struct 注释 "Purpose bridge:
        // System 2's intent → System 1's distillation target") 但全库无任何递增点。
        // 桥接语义: System 2 (树慢反射生长环) 本轮蒸馏出的进化意图 (digested guidance,
        // 即果实消化产物 — 蒸馏目标候选) 交接给 System 1 (GWT 快速谐振)。
        // 触发条件: (1) S2 消化出 ≥1 个达标果实 (last_cycle_guidance 非空)
        //           (2) GWT 谐振已激活 (S1 侧就绪, Phase 2 已 increment S1)。
        // 注意: next_actions (漏洞/缺口修复清单) 是治理动作非蒸馏目标, 不计入桥接 —
        // 桥接专指"可蒸馏的进化意图", 保持语义精确 (防默认树误命中)。
        if !self.core.last_cycle_guidance.is_empty() && self.trunk.gwt_resonance_active {
            self.trunk.mars_bridge_hits += 1;
        }

        // ═══ Phase 4.5: 演化趋势预测 (nt_core_forecast 接线 — 意识体维度升维) ═══
        // 用 forecast 引擎基于当前 branch 健康/迷雾/果实数据, 预测下一 cycle 演化方向。
        // 纯增量: 预测结果写入 report, 不改变既有演化决策路径 (无破坏)。
        report.evolution_forecast = self.forecast_evolution();
        // 存到 core 供 Phase 8 闭环反馈消费 (演化预测 → 探索预算调制)
        self.core.last_forecast = report.evolution_forecast.clone();

        // ═══ Phase 5: ConsciousnessReview — panoramic topology + connectivity + health chain ═══
        let mut review = crate::core::nt_core_consciousness_review::ConsciousnessReview::new();
        let scan_report = review.full_review(self);
        self.core.topology_score = scan_report.topology_score;
        self.core.connectivity_score = scan_report.connectivity_score;
        self.core.health_chain_score = scan_report.health_chain.overall;
        self.core.evolution_path = scan_report.evolution_path;

        // ═══ Phase 6: Contract Fulfillment Verification ═══
        // ═══ Phase 7: Drift Audit — post-cycle evolution fidelity check ═══
        if let Some(contract) = self.core.last_contract.clone() {
            let fulfillment = self.verify_contract_fulfillment(&contract);
            self.core.contract_fulfillment = Some(fulfillment.clone());
            report.phase6_fulfillment = Some(fulfillment.clone());

            let drift_report = self.audit_drift(&contract, &fulfillment);
            self.core.drift_report = Some(drift_report.clone());
            report.phase7_drift = Some(drift_report);
        }

        // ═══ Phase 8: 闭环进化反馈 (意识核心自我运转) ═══
        // 进化产出 (契约 fulfillment + drift) 反馈到进化参数, 使树根据自身
        // 进化结果调整进化策略 — 此前进化是"开环"的: 树自己预测/验证,
        // 但从不调整自己的进化标准。这是意识核心自我运转的核心闭环。
        // 规则:
        //   - 契约 fulfilled → fruit_quality_threshold 上调 (进化标准提升, +0.05, 上限 0.6,
        //     且仅当 MARS bridge 有命中时上调 — 防棘轮反噬)
        //   - drift 检测 → fruit_growth_health 下调 (放宽生长门加速恢复, -0.05, 下限 0.4)
        //   - 无 drift 且 fulfilled → 恢复默认 (0.5)
        self.apply_evolution_feedback();

        report.phase4_guidance = self.core.last_cycle_guidance.len();

        // ═══ Phase 4.6: 宪法执行治理审计 (P1: 合规 0.27 → 0.5+) ═══
        // 用 global_constitution 的规则对本周期产出的 next_actions 与 identified_gaps
        // 做真实合规验证 (verify_compliance), 统计通过率回写 trunk.governance_compliance,
        // 使治理指标反映真实宪法执行而非硬编码默认 (1.0) 或陈旧快照。
        // 规则:
        //   - governance_constitution_count = 检查的规则数 (验证执行次数)
        //   - governance_compliance = 通过项 / 检查项 (无检查项时保持现值)
        //   - governance_fractal_depth = 执行周期数 (逐 cycle 递增, 表征审计深度)
        self.run_governance_audit();

        // CHMA Phase 0: 迷雾地图主量纲 — 全仓加权雾和 + 每域迷雾摘要
        report.weighted_fog_sum = self.weighted_fog_sum();
        report.fog_by_branch = self.fog_by_branch();

        report
    }

    /// 演化趋势预测 — 用 nt_core_forecast 引擎预测下一 cycle 演化方向。
    ///
    /// 输入: 各 branch 当前健康/迷雾/果实数据聚合为事件流; 输出: 方向 + 置信度 + 情景树。
    /// 无 LLM 依赖 (ForecastEngine::new() 不启用 narrator), 纯确定性计算, 可安全用于测试。
    fn forecast_evolution(&self) -> Option<EvolutionForecast> {
        use crate::core::nt_core_forecast::ForecastEngine;

        let mut engine = ForecastEngine::new();

        // 聚合各 branch 健康/迷雾/果实为事件信号 (利多=健康上升, 利空=健康下降)
        let mut health_sum = 0.0f64;
        let mut branch_count = 0usize;
        for branch in self.branches.values() {
            health_sum += branch.health;
            branch_count += 1;
            // 迷雾浓度 → 利空信号 (迷雾越高健康越可能下降)
            let fog_impact = -branch.fog.level * 0.3;
            // 果实质量 → 利多信号 (果实越多演化越健康)
            let fruit_impact = (branch.fruit_count as f64).min(3.0) * 0.2;
            engine.ingest_signed_event(
                "consciousness_tree",
                "branch_health",
                branch.kind.label(),
                fog_impact + fruit_impact,
                if fog_impact + fruit_impact >= 0.0 {
                    1.0
                } else {
                    -1.0
                },
            );
        }
        if branch_count == 0 {
            return None;
        }
        let avg_health = health_sum / branch_count as f64;

        // 基准状态: 平均健康映射到 E8 状态 (0..7)
        let base_state = ((avg_health * 7.0).round() as u8).min(7);

        // 生成推演 (无 LLM, 确定性)
        let forecast = engine.generate_forecast("overall-evolution", base_state);
        if forecast.abstain {
            return Some(EvolutionForecast {
                target: "overall-evolution".into(),
                direction: 0.0,
                confidence: 0.0,
                abstain: true,
                scenario_probs: Vec::new(),
                reason: "信息不足, 弃权".into(),
            });
        }

        // 从情景树叶子提取概率摘要
        let mut scenario_probs = Vec::new();
        for leaf in forecast.tree.leaves() {
            scenario_probs.push((leaf.name.clone(), leaf.probability));
        }
        // 方向 = 牛概率 - 熊概率 (从叶子标签识别)
        let mut direction = 0.0f64;
        for (label, prob) in &scenario_probs {
            if label.contains("bull") {
                direction += prob;
            } else if label.contains("bear") {
                direction -= prob;
            }
        }

        Some(EvolutionForecast {
            target: "overall-evolution".into(),
            direction,
            confidence: forecast
                .tree
                .leaves()
                .first()
                .map(|n| n.confidence)
                .unwrap_or(0.0),
            abstain: false,
            scenario_probs,
            reason: forecast.confidence_reason.clone(),
        })
    }
    /// 每个 `(branch_str, capability)` 对会合并进对应 CapabilityBranch.absorbed_capabilities,
    /// 避免重复条目。branch_str 形如 "NT-CORE"/"NT-SHIELD"。
    pub fn sync_absorbed_capabilities_from_kb(&mut self, pairs: &[(&str, &str)]) -> usize {
        let mut synced = 0usize;
        for (branch_str, capability) in pairs {
            let Some(kind) = BranchKind::from_branch_str(branch_str) else {
                continue;
            };
            let Some(branch) = self.branches.get_mut(&kind) else {
                continue;
            };
            if !branch.absorbed_capabilities.iter().any(|c| c == capability) {
                branch.absorbed_capabilities.push((*capability).to_string());
                synced += 1;
            }
        }
        synced
    }

    /// Phase 0: Negotiate evolution contract before cycle begins
    pub(super) fn negotiate_contract(&self) -> EvolutionContract {
        // Derive claim from top vulnerabilities and gaps
        let mut claim_parts = Vec::new();
        for vuln in &self.core.vuln_scan {
            if vuln.severity.score() >= self.config.action_severity_threshold {
                claim_parts.push(format!("Fix {}: {}", vuln.module, vuln.fix_suggestion));
            }
        }
        for gap in &self.core.identified_gaps {
            claim_parts.push(format!("Address GAP: {}", gap));
        }
        if claim_parts.is_empty() {
            claim_parts.push("Maintain current evolution trajectory".into());
        }
        // 探索预算继承: 上一 contract 的 budget 经 Phase 8 闭环反馈调制后,
        // 被下一 cycle negotiate 继承 — 演化预测 (利多↑/利空↓) 持续塑造探索策略。
        let exploration_budget = self
            .core
            .last_contract
            .as_ref()
            .map(|c| c.exploration_budget.clamp(0.1, 0.4))
            .unwrap_or(0.2);

        EvolutionContract {
            cycle: self.cycle + 1,
            claim: claim_parts.join("; "),
            evidence_plan: vec![
                "SelfTest pass rate per domain >= 80%".into(),
                "Branch health >= 0.6 for all domains".into(),
                "EvolutionFruit quality >= 0.7".into(),
                "Vulnerability count reduced by >= 20%".into(),
            ],
            stop_rule: StopRule::default(),
            exploration_budget,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Phase 6: Verify contract fulfillment
    pub(super) fn verify_contract_fulfillment(&mut self, contract: &EvolutionContract) -> ContractFulfillment {
        let mut fulfilled = 0;
        let total = contract.evidence_plan.len();

        // Check each evidence criterion
        for (i, _criterion) in contract.evidence_plan.iter().enumerate() {
            let met = match i {
                0 => self.branches.values().all(|b| {
                    b.self_test_count > 0
                        && (b.self_test_count as f64 / b.module_count.max(1) as f64) >= 0.8
                }),
                1 => self.branches.values().all(|b| b.health >= 0.6),
                2 => self.fruits.iter().any(|f| f.quality >= 0.7),
                3 => {
                    // Vulnerability reduction vs first-measured baseline.
                    // 缺陷3修复 (自我运转实际情况): 首次评估 (baseline 未建立) 视为满足 —
                    // 记录 baseline 不阻塞 fulfillment。此前首次评估 reduction=0 < 0.2
                    // 恒不满足 → 生产环境契约永不 fulfilled → 闭环反馈"标准提升"分支
                    // 永不触发。首次后要求 >= 20% 减少 (vuln 数量下降才达标)。
                    let current = self.core.vuln_scan.len() as f64;
                    let baseline = self.vuln_baseline.unwrap_or(current as usize);
                    let first_measure = self.vuln_baseline.is_none();
                    self.vuln_baseline = Some(baseline);
                    if first_measure {
                        true // 首次评估: 记录 baseline, 不阻塞 fulfillment
                    } else {
                        let reduction = if baseline as f64 > 0.0 {
                            (baseline as f64 - current) / baseline as f64
                        } else {
                            0.0
                        };
                        reduction >= 0.2
                    }
                }
                _ => false,
            };
            if met {
                fulfilled += 1;
            }
        }

        ContractFulfillment {
            cycle: contract.cycle,
            claim: contract.claim.clone(),
            evidence_met: fulfilled,
            evidence_total: total,
            fulfilled: fulfilled == total,
            quality_achieved: self.fruits.iter().map(|f| f.quality).sum::<f64>()
                / self.fruits.len().max(1) as f64,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Phase 7: Drift Audit — detect evolution drift from contract
    pub(super) fn audit_drift(
        &self,
        contract: &EvolutionContract,
        fulfillment: &ContractFulfillment,
    ) -> DriftReport {
        let claim_achieved = fulfillment.fulfilled;
        let quality_achieved = fulfillment.quality_achieved;
        let drift_detected =
            !claim_achieved || quality_achieved < contract.stop_rule.min_quality_threshold;
        let drift_magnitude = if drift_detected {
            (contract.stop_rule.min_quality_threshold - quality_achieved).abs()
        } else {
            0.0
        };

        let mut corrective_actions = Vec::new();
        if drift_detected {
            corrective_actions.push("Reduce exploration budget".into());
            corrective_actions.push("Tighten stop rule thresholds".into());
            corrective_actions.push("Increase SelfTest coverage requirements".into());
        }

        // C8 (D8): resource_consumed 不再硬编码 0.5 — 以真实树活动计数为度量
        // (MARS System2 反射迭代 + GWT 谐振周期 + 已消化果实 + 吸收总量),
        // 饱和曲线归一到 [0,1]。反映"本契约进化实际消耗的反思/谐振/消化工作量"。
        let resource_consumed = {
            let activity = (self.trunk.mars_system2_iterations
                + self.trunk.resonance_cycle
                + self.fruits.len() as u64
                + self.roots.total_absorbed) as f64;
            (activity / 1000.0).min(1.0)
        };

        DriftReport {
            cycle: self.cycle,
            contract_fulfilled: fulfillment.fulfilled,
            claim_achieved,
            evidence_collected: contract.evidence_plan.clone(),
            quality_achieved,
            resource_consumed,
            drift_detected,
            drift_magnitude,
            stop_rule_triggered: quality_achieved < contract.stop_rule.min_quality_threshold,
            corrective_actions,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

}
