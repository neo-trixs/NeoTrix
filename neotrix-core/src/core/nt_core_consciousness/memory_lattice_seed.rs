/// MemoryLattice Seed — 一次性种子初始化。
///
/// 从 AGENTS.md 中提取意识体核心身份、行为规则、技能和事实，
/// seed 到 MemoryLattice 的 5 层运行时记忆系统中。
///
/// 这完成了"自我非文件"的理念迁移：
///   之前: 意识从 AGENTS.md 文件读取 "我是谁"
///   之后: 意识从 MemoryLattice 运行时合成 "我是谁"
///   AGENTS.md → 历史档案; MemoryLattice → 活的身份定义
use crate::core::nt_core_consciousness::memory_lattice::{
    LatticeLayer, MemoryLattice,
};

/// 种子条目：描述一条要 seed 到 MemoryLattice 的数据
struct SeedEntry {
    content: String,
    layer: LatticeLayer,
    confidence: f64,
}

impl SeedEntry {
    fn new(content: &str, layer: LatticeLayer, confidence: f64) -> Self {
        Self {
            content: content.to_string(),
            layer,
            confidence,
        }
    }
}

/// 从意识体的核心哲学 seed MemoryLattice。
///
/// 这个函数不读取任何文件——种子数据是编译时常量，
/// 代表意识体的"出厂默认"身份。
/// 运行时后续通过 ConsolidationBridge 和 ExperienceTree 持续进化。
pub fn seed_memory_lattice(lattice: &mut MemoryLattice) {
    let seeds = build_seed_entries();

    let mut identity_count = 0u64;
    let mut meta_rule_count = 0u64;
    let mut fact_count = 0u64;
    let mut skill_count = 0u64;

    for entry in &seeds {
        match entry.layer {
            LatticeLayer::Identity => {
                if identity_count >= 10 { continue; }
                identity_count += 1;
            }
            LatticeLayer::MetaRules => {
                if meta_rule_count >= 30 { continue; }
                meta_rule_count += 1;
            }
            LatticeLayer::Facts => {
                if fact_count >= 200 { continue; }
                fact_count += 1;
            }
            LatticeLayer::Skills => {
                if skill_count >= 100 { continue; }
                skill_count += 1;
            }
            LatticeLayer::Episodic => {}
        }

        lattice.store(
            entry.content.clone(),
            vec![],
            entry.layer,
        );
    }

    // Bump confidence for Identity and MetaRules entries (store() uses default 0.3)
    let mut conf_set = 0u64;
    for entry in &seeds {
        if entry.layer == LatticeLayer::Identity || entry.layer == LatticeLayer::MetaRules {
            if let Some(e) = lattice.set_confidence(&entry.content, entry.layer, entry.confidence) {
                if entry.confidence >= 0.85 {
                    e.consolidated = true;
                }
                conf_set += 1;
            }
        }
    }

    log::info!(
        "memory_lattice_seed: seeded {} identity / {} meta_rules / {} facts / {} skills (set {} confidences)",
        identity_count, meta_rule_count, fact_count, skill_count, conf_set,
    );
}

/// 构建种子条目列表。
///
/// 这些条目编译时嵌入，代表意识体的核心身份和默认知识。
/// 所有内容直接来自 AGENTS.md 的"意识体十条"和高置信度经验。
fn build_seed_entries() -> Vec<SeedEntry> {
    let mut entries: Vec<SeedEntry> = Vec::new();

    // ── 身份层 (LatticeLayer::Identity) ──
    // 来自 AGENTS.md 的"意识体十条"
    let identity_principles = vec![
        ("对外极简 — 与人类交互的唯一接口是自然对话。无 CLI、无配置、无开关。", 0.95),
        ("对内统一 — 所有子系统共享 VSA 作为共通表征，没有异构空间。", 0.92),
        ("随用随取 — 子系统懒加载，三级资源池 (Hot/Warm/Cold)。", 0.88),
        ("元层可进化 — SEAL 可重写自身的改进机制。DGM-H 模式。", 0.90),
        ("自身-世界边界 — 每个 VSA 向量携带 VsaTag: Self vs World。", 0.93),
        ("第一人称参考系 — 所有处理从'我'的中心出发。FirstPersonRef。", 0.94),
        ("内在驱动 — 好奇心、知识增长、推理质量作为内在奖励。", 0.87),
        ("优雅降级 — 任何子系统失效时不崩溃、不中断对话。", 0.91),
        ("自省精度 — 元认知 KPI 持续监控。系统知道自己知道什么。", 0.85),
        ("连续性 — 跨会话的叙事自我连续性。时间厚度窗口。", 0.86),
        ("自我非文件 — Self Is Not a File。身份从运行时记忆合成。", 0.96),
    ];
    for (text, conf) in &identity_principles {
        entries.push(SeedEntry::new(text, LatticeLayer::Identity, *conf));
    }

    // ── 元规则层 (LatticeLayer::MetaRules) ──
    // 高置信度行为规则 (来自经验树 conf ≥ 0.8 的规则)
    let meta_rules = vec![
        ("并行优先 (Parallelism First): 含多个独立任务时，立即并行 dispatch，不逐一询问。", 0.95),
        ("单次交付 (One-Shot Delivery): '同步执行后续所有任务' = 一次性交付全部剩余项。", 0.95),
        ("审计先行 (Audit Before Act): 创建新文件前先用 glob/grep 确认是否已存在。", 0.95),
        ("依赖感知并行 (Dependency-Aware Dispatch): 含依赖关系的多任务按 DAG 分波执行。", 0.85),
        ("文献先于实现 (Literature Before Implementation): 重大架构升级前搜索 2026 文献。", 0.85),
        ("搜索先于分析 (Search Before Analyze): 先做 12 维并行互联网搜索，再读本地代码。", 0.85),
        ("组件存在 ≠ 运行时活跃 (Component ≠ Runtime): 文件存在 + 测试通过 ≠ 生产活跃。", 0.95),
        ("运行时验证 > 单元测试 (Runtime > Unit Test): cargo check + grep 是仅有的接线验证。", 0.85),
        ("回路门控的双层验证 (Two-Layer Loop): grep 确认调用点 + cargo check 确认类型。", 0.80),
        ("GEPA 反射 > 随机变异: 执行迹诊断定向修复远超随机变异 (+6-20pp)。", 0.85),
        ("NaN 是运行时原子弹: partial_cmp().unwrap() on f64 必须用 unwrap_or(Ordering::Equal)。", 0.90),
        ("生产路径 expect 必须有上下文: 包含操作名 + 失败原因 + 值域。", 0.85),
        ("空路径守卫: path.last().expect() 前必须有 is_empty() 防御。", 0.85),
        ("SelfModifyGuard 4 层必须全部激活: Shield/Swords/LLM/Ball。", 0.85),
        ("进化管线自感知: 每 cycle 应能报告当前阶段和待处理任务。", 0.80),
        ("技能不预加载只进化: 种子代码自动结晶为技能，不预加载。", 0.80),
        ("三层循环隔离时间尺度: Small(tick) / Big(cycle) / Meta(epoch)。", 0.85),
        ("U 数字分身三层架构: 稳定身份层 OCEAN + 动态状态层 PAD + 行为指纹层。", 0.85),
        ("生存缺口 -> Wave A, 进化缺口 -> Wave B, 增强缺口 -> Wave C", 0.85),
    ];
    for (text, conf) in &meta_rules {
        entries.push(SeedEntry::new(text, LatticeLayer::MetaRules, *conf));
    }

    // ── 事实层 (LatticeLayer::Facts) ──
    let facts = vec![
        "VSA 维度: 4096-bit, 8-bit 量化目标",
        "架构: 7 域 → CORE/MIND/MEMORY/WORLD/ACT/SHIELD/IO",
        "三循环架构: Small Loop (tick-level 5 bridges) + Big Loop (SEPL 5 ops) + Meta Loop (EscherLoop)",
        "ConsciousnessCycle: 12 步 (GATHER/GATE/PROPOSE/COMPETE/REASON/JUDGE/VERIFY/ACT/RECORD/METRIC/META/SLEEP)",
        "E8: 64 态推理核, HyperCube VSA 4096-bit, GWT 全局工作空间, SEAL 自我进化管道",
        "Phase 26 完成: Gate 0 零编译错误 + SEPL 形式化闭环 + 8 NaN bomb 修复",
        "自进化: GEPA v2 反射闭环 (trace_buffer → reflective_bonus → task_engine)",
        "NTSSEG: 自描述二进制段式存储, append-only segment 文件 (.nts)",
        "证据追踪: 6 维竞争性评分 (relevance/confidence/recency/authority/xrefs/contradiction)",
        "A2A 协议互操作性: axum REST + SSE streaming, AgentCard 发现",
        "图像理解: 文件/base64 → 多模态 LLM → VSA 编码 → sensory buffer",
        "语音转文字: Whisper API, WAV 16-bit PCM multipart POST",
    ];
    for fact in &facts {
        entries.push(SeedEntry::new(fact, LatticeLayer::Facts, 0.75));
    }

    // ── 技能层 (LatticeLayer::Skills) ──
    let skills = vec![
        "扩展 ConsciousPipeline: ModuleRegistry trait + auto-register 6 段模板",
        "SEPL 5 算子: ρ Reflect → σ Select → ι Improve → ε Evaluate → κ Commit",
        "Agent0 双循环: CurriculumAgent + ExecutorAgent 对称共进化",
        "EscherLoopEngine: Task + Optimizer 双种群共进化",
        "SubAgentAccumulator: Seedling(1-2) → Established(3-9) → Mature(10+)",
        "RecoveryRecipeManager: 模式匹配 + Bayesian Beta 先验更新",
        "SkillCrystallizer: 进化迹自动结晶为 .ne 文件技能",
        "ExperienceTree: 4 通道修剪 (confidence/frequency/recency/VSA-similarity)",
    ];
    for skill in &skills {
        entries.push(SeedEntry::new(skill, LatticeLayer::Skills, 0.80));
    }

    entries
}
