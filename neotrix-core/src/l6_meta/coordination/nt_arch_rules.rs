//! 架构规则注册表（裁定 ①A：**开集可注册，但每条必须可证伪 + 有消费者 + 进豁免基线**）。
//!
//! # 为什么不是「规则无限」
//!
//! 用户原话是「规则是无限的」。本模块**驳回这一条**，理由是它与本仓公理
//! **D-16 最短描述**（`NEOTRIX-MASTER-BLUEPRINT.md` §20）直接冲突：
//!
//! > 理解 ＝ 在未见过的数据上描述长度最短；
//! > `free_energy = complexity + accuracy`
//!
//! 「规则无限」= complexity 项趋于无穷 ⇒ 按 D-16 **不是更聪明，是更不自知**。
//! 工程后果更具体：规则越多，每条越无法被证伪，整体越不可裁决 ——
//! 那正是本仓正在治的病（`nt_core_guardian` 1,567 行自称统一 13 机制却零融合；
//! 同一机制 10 份熔断器；同一类型 4 份）。
//!
//! ⇒ 裁定为**开集但有界**（用户已确认 ①A）：
//!
//! | 要求 | 落地 |
//! |---|---|
//! | **可证伪** | 每条必须给出「什么输入会让它为假」；无判据不得注册 |
//! | **有消费者** | 每条必须点名它服务的**模块或门**；无消费者不得注册 |
//! | **进豁免基线** | 违反它的既有债必须登记进 `.neotrix/arch-rules-baseline.txt` |
//!
//! ⛔ **本模块不做自动架构微调。** 2,465 个已分类文件、13 条已知分层债是
//! **跨窗口共享的架构决策**；任何自动改动会撞上别窗正在写的文件，
//! 且产出的改动**没人有资格裁决**（改哪一层是架构判断）。
//! 本模块只做一件事：**给"该往哪调"提供带对抗评分的排序信号**，人裁决。

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// 一条架构规则的**元数据**（不含判定逻辑 —— 逻辑在各自的门/测试里）。
///
/// ⛔ `falsifiable_by` 为空 ⇒ 无法证伪 ⇒ 不得注册（见 `RuleRegistry::register`）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchRuleMeta {
    /// 规则 id（稳定，基线文件按它登记）。
    pub id: String,
    /// 人读的一句话。
    pub statement: String,
    /// ⭐ **什么输入会让这条规则为假**（必填 —— 无判据的规则不可证伪）。
    pub falsifiable_by: String,
    /// ⭐ **这条规则服务的模块或门**（必填 —— 无消费者的规则是装饰）。
    pub serves: Vec<String>,
    /// 规则指向的层（`None` 表示跨层）。
    pub layer: Option<u8>,
}

// ============================================================================
// 三条准入判定（裁定 ①A 的核心）
// ============================================================================

/// 准入失败的原因 —— ⛔ **禁止 panic**（本仓铁律），故用 `Result` 而非 `expect`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RejectReason {
    /// `falsifiable_by` 为空 ⇒ 不可证伪
    NotFalsifiable(String),
    /// `serves` 为空 ⇒ 无消费者
    NoConsumer(String),
    /// id 重复
    DuplicateId(String),
}

impl std::fmt::Display for RejectReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFalsifiable(s) => write!(f, "规则「{s}」没写 falsifiable_by ⇒ 不可证伪"),
            Self::NoConsumer(s) => write!(f, "规则「{s}」没写 serves ⇒ 没有消费者"),
            Self::DuplicateId(s) => write!(f, "规则 id 重复：{s}"),
        }
    }
}

/// 规则注册表 —— **开集**（可随时注册新规则），但每条受三条准入判定约束。
///
/// # 规则注册表（`.neotrix/arch-rules.tsv`）与本结构的对应
///
/// | 规则 | 本文件的实现点 | 门 |
/// |---|---|---|
/// | **R1** 反向层依赖 | `scan_layer_file`（判定 `tgt > src`） | `scripts/check-layer-deps.sh` |
/// | **R2** 层词汇单一真源 | [`ArchLayer::REAL_DIRS`](crate::l6_meta::nt_core_self_review::nt_review_types::ArchLayer::REAL_DIRS) | `scripts/check-arch-rules.sh` |
/// | **R3** 规则三准入 | [`RuleRegistry::register`] 的三条 `RejectReason` | `scripts/check-arch-rules.sh` |
/// | **R4** 对抗不代替裁决 | [`DebtVerdict::needs_human`]（`score_layer_debt` 恒填充） | `scripts/check-arch-rules.sh` |
///
/// ⛔ 改本文件时**同步** `.neotrix/arch-rules.tsv` —— 门会校验
/// 「每个规则 id 在本文件有实现引用」，反向（改了实现没改表）由
/// `arch-rules-baseline.txt` 的双向差集抓住。
#[derive(Debug, Default)]
pub struct RuleRegistry {
    rules: BTreeMap<String, ArchRuleMeta>,
}

impl RuleRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一条规则。**三条准入判定全过才收**（裁定 ①A）。
    ///
    /// ⛔ 这不是"完备性检查"，是**最低门槛** —— 它拦得住"装饰性规则"，
    /// 拦不住"规则本身写错了"。后者要靠红蓝对抗（见
    /// [`score_layer_debt`]）与人工裁决。
    pub fn register(&mut self, meta: ArchRuleMeta) -> Result<(), RejectReason> {
        if meta.id.trim().is_empty() {
            return Err(RejectReason::NotFalsifiable(meta.statement));
        }
        if meta.falsifiable_by.trim().is_empty() {
            return Err(RejectReason::NotFalsifiable(meta.statement));
        }
        if meta.serves.is_empty() {
            return Err(RejectReason::NoConsumer(meta.statement));
        }
        if self.rules.contains_key(&meta.id) {
            return Err(RejectReason::DuplicateId(meta.id));
        }
        self.rules.insert(meta.id.clone(), meta);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&ArchRuleMeta> {
        self.rules.get(id)
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ArchRuleMeta> {
        self.rules.values()
    }
}

// ============================================================================
// 红蓝对抗评分（裁定 ③A：给分层债排序，产出**建议**而非自动改动）
// ============================================================================

/// 一处分层债（由 `check_architecture_layer_depth` 检出）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerDebt {
    pub path: String,
    pub line: u32,
    /// 源层号（0-6）
    pub src_layer: u8,
    /// 目标层号（0-6）
    pub tgt_layer: u8,
    pub snippet: String,
}

/// 红蓝对抗的一票。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    /// 主张「该修」
    Pro,
    /// 主张「容忍 / 暂不动」
    Con,
}

/// 一条债的对抗结论 —— **只是建议，人裁决**。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtVerdict {
    pub debt: LayerDebt,
    /// 跨层距离（|src−tgt|）。**越大越可能是真问题**（同层误报优先）。
    pub layer_gap: i32,
    pub pro: Vec<String>,
    pub con: Vec<String>,
    /// 对抗后收敛分（0-1，越高越该修）
    pub converged_score: f64,
    /// 分歧度（0=蓝队全同意该修，1=完全分歧）
    pub divergence: f64,
    /// ⭐ 裁决所需的人工判断提示（⛔ 本模块不代做）
    pub needs_human: String,
}

/// 对一条债跑红蓝对抗，产出**可裁决的排序信号**。
///
/// # 蓝队（Pro：主张修）的论据 —— 全部是**结构事实**，不含 LLM 判断
/// - `layer_gap` 大：跨层跳跃而非相邻引用
/// - 出现在 `layer-deps-baseline.txt` 的已知债里 ⇒ 已登记但未解决
/// - 目标层是**基座**（L0）或**元层**（L6）⇒ 反向依赖影响面最大
///
/// # 红队（Con：主张容忍）的论据 —— 同上，也是结构事实
/// - 走 `facade` 通道 ⇒ 那是本仓**唯一合法**的跨层通道（AGENTS.md §4.2）
/// - 出现在 `#[cfg(test)]` 块内 ⇒ 不影响生产
/// - 基线已登记 ⇒ 已知并接受，不构成新增债
///
/// ⛔ **不做的事**：不判断"这条债该不该修"。那取决于业务语义与跨窗口协调，
/// 只有人能决定（裁定 ①A 的前提：门给信号，人裁决）。
pub fn score_layer_debt(debt: &LayerDebt, in_baseline: bool, is_facade_path: bool) -> DebtVerdict {
    let gap = (debt.src_layer as i32 - debt.tgt_layer as i32).abs();

    let mut pro: Vec<String> = Vec::new();
    let mut con: Vec<String> = Vec::new();

    // ── 蓝队 ──
    pro.push(format!("跨 {gap} 层（{} → {}）", debt.src_layer, debt.tgt_layer));
    if gap >= 4 {
        pro.push("层距 ≥4 ⇒ 与 LAYER-DEBT-TIERS 的 S1/S2 定义吻合".into());
    }
    if debt.tgt_layer == 0 {
        pro.push("目标是基座 L0 ⇒ 基座反向依赖高层，几乎构成循环".into());
    }
    if debt.tgt_layer == 6 {
        pro.push("源层引元层 L6 ⇒ 治理面渗入业务层".into());
    }
    if in_baseline {
        pro.push("已在 layer-deps-baseline.txt 登记 ⇒ 已知但未解决".into());
    }

    // ── 红队 ──
    if is_facade_path {
        con.push("路径经 facade ⇒ AGENTS.md §4.2 认定的唯一合法跨层通道".into());
    }
    if in_baseline {
        con.push("已在基线 ⇒ 属已登记的已知债，不构成新增回归".into());
    }
    con.push("权威裁决门是 check-layer-deps.sh（干净检出口径），本分数仅供参考".into());

    // ── 收敛分：结构分，不用 LLM ──
    let gap_score = (gap as f64 / 6.0).min(1.0);
    let target_weight = if debt.tgt_layer == 0 || debt.tgt_layer == 6 { 1.0 } else { 0.6 };
    let baseline_boost = if in_baseline { 0.15 } else { 0.0 };
    let facade_penalty = if is_facade_path { 0.35 } else { 0.0 };
    let converged = (0.55 * gap_score + 0.45 * target_weight + baseline_boost - facade_penalty)
        .clamp(0.0, 1.0);

    // 分歧度：论据条数差异（Pro 与 Con 势均力敌 ⇒ 分歧高）
    let total = (pro.len() + con.len()) as f64;
    let divergence = if total == 0.0 {
        0.0
    } else {
        (pro.len() as f64 - con.len() as f64).abs() / total
    };

    let needs_human = if is_facade_path {
        "走 facade 通道 ⇒ 先确认它是否属 AGENTS.md §4.2 的豁免范围".to_string()
    } else if gap >= 4 {
        format!(
            "跨 {} 层：需裁决是「消费方自己那层的 facade 用错了方向」还是「真该下沉」",
            gap
        )
    } else {
        "相邻层引用：多为分层粒度问题，优先级最低".to_string()
    };

    DebtVerdict {
        debt: debt.clone(),
        layer_gap: gap,
        pro,
        con,
        converged_score: converged,
        divergence,
        needs_human,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta(id: &str) -> ArchRuleMeta {
        ArchRuleMeta {
            id: id.into(),
            statement: "测试规则".into(),
            falsifiable_by: "喂一个反例".into(),
            serves: vec!["scripts/check-layer-deps.sh".into()],
            layer: Some(1),
        }
    }

    #[test]
    fn register_accepts_complete_rule() {
        let mut r = RuleRegistry::new();
        assert!(r.register(meta("R1")).is_ok());
        assert_eq!(r.len(), 1);
    }

    /// 裁定 ①A 的第一条硬要求：**没写判据的规则不得注册**。
    #[test]
    fn register_rejects_unfalsifiable() {
        let mut r = RuleRegistry::new();
        let mut m = meta("R1");
        m.falsifiable_by = "  ".into();
        assert!(matches!(
            r.register(m),
            Err(RejectReason::NotFalsifiable(_))
        ));
        assert!(r.is_empty(), "被拒的规则不得进注册表");
    }

    /// 第二条硬要求：**没有消费者的规则是装饰**。
    #[test]
    fn register_rejects_without_consumer() {
        let mut r = RuleRegistry::new();
        let mut m = meta("R1");
        m.serves = vec![];
        assert!(matches!(r.register(m), Err(RejectReason::NoConsumer(_))));
        assert!(r.is_empty());
    }

    #[test]
    fn register_rejects_duplicate_id() {
        let mut r = RuleRegistry::new();
        assert!(r.register(meta("R1")).is_ok());
        assert!(matches!(
            r.register(meta("R1")),
            Err(RejectReason::DuplicateId(_))
        ));
        assert_eq!(r.len(), 1);
    }

    fn debt(src: u8, tgt: u8) -> LayerDebt {
        LayerDebt {
            path: format!("neotrix-core/src/l{src}_x/foo.rs"),
            line: 36,
            src_layer: src,
            tgt_layer: tgt,
            snippet: format!("use crate::l{tgt}_y::bar;"),
        }
    }

    /// 跨层越远 ⇒ 收敛分越高（蓝队论据更强）。
    #[test]
    fn score_prefers_bigger_gap() {
        let near = score_layer_debt(&debt(1, 2), false, false);
        let far = score_layer_debt(&debt(1, 5), false, false);
        assert!(
            far.converged_score > near.converged_score,
            "跨 4 层({:.2}) 应高于相邻层({:.2})",
            far.converged_score,
            near.converged_score
        );
    }

    /// 目标 L0/L6 加权 —— 基座/元层反向依赖影响面最大。
    #[test]
    fn score_weights_base_and_meta_targets() {
        let to_l0 = score_layer_debt(&debt(3, 0), false, false);
        let to_l3 = score_layer_debt(&debt(3, 3), false, false);
        assert!(to_l0.converged_score > to_l3.converged_score);
    }

    /// ⭐ facade 通道必须**压低**分数 —— 那是本仓唯一合法的跨层通道。
    #[test]
    fn facade_path_is_penalised() {
        let plain = score_layer_debt(&debt(1, 5), true, false);
        let facade = score_layer_debt(&debt(1, 5), true, true);
        assert!(
            facade.converged_score < plain.converged_score,
            "facade 通道({:.2}) 应低于裸跨层({:.2})",
            facade.converged_score,
            plain.converged_score
        );
        assert!(facade.needs_human.contains("facade"));
    }

    /// ⭐ 分数与「人裁决」必须并存 —— 门给信号，不代替裁决。
    #[test]
    fn verdict_always_names_human_decision() {
        for d in [debt(1, 2), debt(1, 5), debt(0, 6)] {
            let v = score_layer_debt(&d, true, false);
            assert!(!v.needs_human.is_empty(), "每条债都必须提示人工裁决点");
            assert!(!v.pro.is_empty() && !v.con.is_empty(), "红蓝双方都要有论据");
        }
    }

    /// 收敛分恒在 [0,1] —— ⛔ 不许溢出（门报告被污染 = 门不可信）。
    #[test]
    fn score_is_bounded() {
        for src in 0..7u8 {
            for tgt in 0..7u8 {
                for baseline in [false, true] {
                    for facade in [false, true] {
                        let v = score_layer_debt(&debt(src, tgt), baseline, facade);
                        assert!(
                            (0.0..=1.0).contains(&v.converged_score),
                            "src={src} tgt={tgt} baseline={baseline} facade={facade} => {}",
                            v.converged_score
                        );
                        assert!((0.0..=1.0).contains(&v.divergence));
                    }
                }
            }
        }
    }
}
// ============================================================================
// 接线（裁定 ③A）：从真实检出源取债 → 红蓝对抗 → 排序建议
// ============================================================================

use crate::l5_cognition::nt_core_gate::nt_panel_debate::deliberate;
use crate::l5_cognition::nt_core_gate::nt_judge::{JudgeFamily, JudgeOpinion};
use crate::l5_cognition::nt_core_prm::ScoredCriterion;

/// 把一条债转成蓝队/红队各一票 `JudgeOpinion`（接 `deliberate()`）。
///
/// ⭐ **这是「接进门」的关键**：`JudgePanel::deliberate` 是本仓既有的
/// 红蓝对抗入口（`nt_core_gate/nt_panel_debate.rs:608`），此前**只有测试在用**
/// （`tests.rs:177-239`），零生产消费者 ⇒ 属"已建+已测+未接线"。
/// 本函数是它的**第一个真实调用点**，且不新建任何平行模块。
fn to_opinion(side: Side, judge_id: &str, score: f64, argument: String) -> JudgeOpinion {
    let family = match side {
        Side::Pro => JudgeFamily::Analytic,
        Side::Con => JudgeFamily::Heuristic,
    };
    JudgeOpinion {
        judge_id: judge_id.to_string(),
        family,
        raw_score: score,
        debiased_score: score,
        confidence: 0.8,
        criteria: vec![ScoredCriterion {
            name: format!("{side:?} 论据"),
            score,
            rationale: Some(argument.clone()),
        }],
        attribution_tags: vec![format!("{side:?}")],
    }
}

/// 对一批债跑完整红蓝对抗，产出**按收敛分降序**的裁决建议。
///
/// # ⛔ 本函数不做架构微调（裁定 ①A 的前提）
///
/// 返回的是 `DebtVerdict` 列表 —— 每条都带 `needs_human`。
/// 真正的"微调"是改代码，而改跨层依赖是**跨窗口共享的架构决策**：
/// 2,465 个已分类文件 + 13 条已知分层债不是本模块能单方面决定的。
/// ⇒ 本函数定位为**"该往哪调"的排序信号**，人裁决。
pub fn rank_layer_debts(
    debts: &[LayerDebt],
    baseline_reader: &dyn Fn(&str) -> bool,
) -> Vec<DebtVerdict> {
    let mut out: Vec<DebtVerdict> = debts
        .iter()
        .map(|d| {
            let in_baseline = baseline_reader(&d.path);
            let is_facade = d.path.contains("facade");
            let mut v = score_layer_debt(d, in_baseline, is_facade);

            // ── 接既有红蓝对抗引擎 ──
            let opinions = vec![
                to_opinion(
                    Side::Pro,
                    "arch_pro",
                    v.converged_score,
                    v.pro.join("；"),
                ),
                to_opinion(
                    Side::Con,
                    "arch_con",
                    1.0 - v.converged_score,
                    v.con.join("；"),
                ),
            ];
            let report = deliberate(&opinions);
            // ⚠️ deliberate 的收敛分是 0-1 的启发式；与结构分加权合并，
            //    结构分占主导（规则本身是结构事实，对抗分只做微调）。
            v.converged_score =
                (0.75 * v.converged_score + 0.25 * report.converged_score).clamp(0.0, 1.0);
            v.divergence = (0.5 * v.divergence + 0.5 * report.divergence).clamp(0.0, 1.0);
            v
        })
        .collect();
    // 稳定排序：分数降序；同分按层距降序、再按路径（保证可复现）
    out.sort_by(|a, b| {
        b.converged_score
            .partial_cmp(&a.converged_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(b.layer_gap.cmp(&a.layer_gap))
            .then(a.debt.path.cmp(&b.debt.path))
    });
    out
}

#[cfg(test)]
mod rank_tests {
    use super::*;

    fn d(path: &str, src: u8, tgt: u8) -> LayerDebt {
        LayerDebt {
            path: path.into(),
            line: 1,
            src_layer: src,
            tgt_layer: tgt,
            snippet: String::new(),
        }
    }

    /// ⭐ 排序必须**可复现**：同输入两次调用得同序（否则门不可裁决）。
    #[test]
    fn rank_is_deterministic() {
        let debts = vec![
            d("a.rs", 1, 2),
            d("b.rs", 1, 5),
            d("facade/c.rs", 3, 6),
            d("d.rs", 0, 6),
        ];
        let no_base = |_: &str| false;
        let r1: Vec<String> = rank_layer_debts(&debts, &no_base)
            .into_iter()
            .map(|v| v.debt.path.clone())
            .collect();
        let r2: Vec<String> = rank_layer_debts(&debts, &no_base)
            .into_iter()
            .map(|v| v.debt.path.clone())
            .collect();
        assert_eq!(r1, r2, "同输入必须同序");
    }

    /// 基线内 ⇒ 蓝队论据减 1 条但仍有「已知未解决」⇒ 分数应上升。
    #[test]
    fn baseline_raises_score() {
        let x = d("x.rs", 1, 5);
        let a = score_layer_debt(&x, false, false);
        let b = score_layer_debt(&x, true, false);
        assert!(b.converged_score > a.converged_score);
    }

    /// 排序必须把「跨 4 层 + 目标 L0」的排前面。
    #[test]
    fn rank_puts_worst_first() {
        let debts = vec![d("near.rs", 1, 2), d("worst.rs", 0, 5)];
        let no_base = |_: &str| false;
        let r = rank_layer_debts(&debts, &no_base);
        assert_eq!(r[0].debt.path, "worst.rs", "最严重的应排第一");
        assert!(r[0].converged_score > r[1].converged_score);
    }

    /// ⭐ 无论怎么排序，每条都必须带人工裁决提示 —— 门不代替裁决。
    #[test]
    fn every_ranked_item_carries_human_prompt() {
        let no_base = |_: &str| false;
        for v in rank_layer_debts(&[d("a.rs", 1, 5), d("b.rs", 2, 6)], &no_base) {
            assert!(!v.needs_human.is_empty());
            assert!(!v.pro.is_empty());
            assert!(!v.con.is_empty());
        }
    }

    /// 空输入不 panic（本仓铁律：⛔ 禁 panic）。
    #[test]
    fn rank_empty_is_ok() {
        let no_base = |_: &str| false;
        assert!(rank_layer_debts(&[], &no_base).is_empty());
    }
}
