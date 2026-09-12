//! # NT-REPAIR-HANZI-VIDEO — 汉字拆字视频生成修复引擎 (hbg-hanzi-chaizi-video 吸收, 2026-08-27)
//!
//! 吸收自 [Mr-funny/hbg-hanzi-chaizi-video](https://github.com/Mr-funny/hbg-hanzi-chaizi-video)
//! 的汉字组件视频生成方法论, 转化为 NT-REPAIR 域的 Rust 生产骨架:
//!
//!   1. **HanziChaiziEngine** — 汉字拆字 (chaizi): 将目标汉字递归分解为组件序列,
//!      每个组件带层级与笔画占位, 作为视频分镜的"骨架"。
//!   2. **VideoClipPlanner** — 组件序列 → 视频片段生成计划 (`VideoClipPlan`):
//!      每个组件映射为一个视频片段 (clip), 携带时间轴/层级/渲染提示, 供下游
//!      ffmpeg 渲染 (本模块仅产出计划, 不集成 ffmpeg, 满足 stub 契约)。
//!   3. **RepairHarness** — 修复入口: 校验拆分链完整性 (组件可达根/无悬空),
//!      不变量被破坏即报修复信号 (NT-REPAIR 自愈语义)。
//!
//! 接线契约 (R-P79): 本模块实现 `SelfTest` (T1), 注册名 `nt_repair_hanzi_video`,
//! 其结果流入 ConsciousnessTree 分支健康 (T3 生产接线)。
//!
//! ## 证据阶梯 (hbg-hanzi-chaizi-video → C0-C6 映射)
//!
//! | 阶梯 | 本模块对应 | 条件 |
//! |------|-----------|------|
//! | C0 身份映射 | `HanziChar` / `HanziComponent` 建模 | 编译 |
//! | C1 拆字 | `HanziChaiziEngine::decompose` | 单测 |
//! | C2 组件序列 | `HanziChaiziEngine::_component_sequence` | 单测 |
//! | C3 视频计划 | `VideoClipPlanner::plan` | 单测 |
//! | C4 契约输出 | `RepairHarness::repair` 聚合 | 集成 (SelfTest) |
//! | C5 跨面复用 | 修复注册 | 生产接线 (T3) |

#![forbid(unsafe_code)]

use crate::core::nt_core_self_test::SelfTest;

// ────────────────────────────────────────────────────────────────
// 汉字组件 — 拆字结果的基本单元
// ────────────────────────────────────────────────────────────────

/// 单个汉字组件 (chaizi 产物): 携带字形 + 层级 + 笔画占位。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HanziComponent {
    /// 组件字形 (如 "木" / "子" / "李" 的某层拆出件)。
    pub glyph: String,
    /// 层级: 0 = 叶子组件 (不可再拆), 越大越接近根汉字。
    pub level: u8,
    /// 笔画数占位 (用于视频渲染时长估算)。
    pub strokes: u8,
}

impl HanziComponent {
    pub fn new(glyph: impl Into<String>, level: u8, strokes: u8) -> Self {
        Self {
            glyph: glyph.into(),
            level,
            strokes,
        }
    }

    /// 叶子组件: level 0。
    pub fn leaf(glyph: impl Into<String>, strokes: u8) -> Self {
        Self::new(glyph, 0, strokes)
    }
}

/// 一个汉字的拆字结果: 组件树被展平成带层级的组件序列。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HanziChar {
    /// 原汉字 (根)。
    pub root: String,
    /// 拆出的组件序列 (叶子在前, 根在后; 即视频分镜顺序)。
    pub components: Vec<HanziComponent>,
}

impl HanziChar {
    pub fn new(root: impl Into<String>, components: Vec<HanziComponent>) -> Self {
        Self {
            root: root.into(),
            components,
        }
    }

    /// 组件序列是否叶子优先 (level 0 在首, 根在末)。
    pub fn is_well_formed(&self) -> bool {
        if self.components.is_empty() {
            return false;
        }
        // 末组件应为根字形
        if self.components.last().map(|c| c.glyph.as_str()) != Some(self.root.as_str()) {
            return false;
        }
        // 层级应单调非减 (叶子→根)
        self.components
            .windows(2)
            .all(|w| w[0].level <= w[1].level)
    }
}

// ────────────────────────────────────────────────────────────────
// HanziChaiziEngine — 汉字拆字 (递归分解为组件序列)
// ────────────────────────────────────────────────────────────────

/// 内置拆字表 (stub): 覆盖少量常见汉字的 chaizi 规则。
/// 真实吸收应扩展为 KB-backed 组件字典 (R-P79 后续接线点)。
type ChaiziRule = (u8, &'static [&'static str]);

/// 内置拆字规则: (组件笔画数占位, 子组件序列, 末元素为根字形)。
const CHAIZI_TABLE: &[(&str, ChaiziRule)] = &[
    // 李 = 木 + 子, 木(4) 子(3), 根 李(7)
    ("李", (7, &["木", "子", "李"])),
    // 林 = 木 + 木, 根 林(8)
    ("林", (8, &["木", "木", "林"])),
    // 好 = 女 + 子, 女(3) 子(3), 根 好(6)
    ("好", (6, &["女", "子", "好"])),
    // 明 = 日 + 月, 日(4) 月(4), 根 明(8)
    ("明", (8, &["日", "月", "明"])),
    // 森 = 木 + 林, 林(8) 木(4), 根 森(12)
    ("森", (12, &["林", "木", "森"])),
];

/// 汉字拆字引擎: 将目标汉字递归分解为组件序列。
#[derive(Debug, Clone, Default)]
pub struct HanziChaiziEngine {
    /// 未知汉字的回退笔画占位 (无法查表时视作整体叶子)。
    pub unknown_strokes: u8,
}

impl HanziChaiziEngine {
    pub fn new(unknown_strokes: u8) -> Self {
        Self {
            unknown_strokes: unknown_strokes.max(1),
        }
    }

    /// 查内置拆字表, 返回 (根笔画, 子组件序列), 未命中返回 None。
    fn lookup(&self, glyph: &str) -> Option<ChaiziRule> {
        CHAIZI_TABLE.iter().find(|(g, _)| *g == glyph).map(|(_, r)| *r)
    }

    /// 拆字: 返回带层级的组件序列 (叶子优先, 根为末)。
    ///
    /// 不变量:
    /// - 命中表: 子组件 level 递增, 末为根 (level = 子组件数)。
    /// - 未命中: 整体作为单叶子组件 (level 0, 笔画占位 unknown_strokes)。
    pub fn decompose(&self, glyph: &str) -> HanziChar {
        match self.lookup(glyph) {
            Some((root_strokes, parts)) => {
                let components: Vec<HanziComponent> = parts
                    .iter()
                    .enumerate()
                    .map(|(i, g)| {
                        let level = i as u8;
                        let strokes = if i + 1 == parts.len() {
                            root_strokes
                        } else {
                            self.lookup(g)
                                .map(|(s, _)| s)
                                .unwrap_or(self.unknown_strokes)
                        };
                        HanziComponent::new(*g, level, strokes)
                    })
                    .collect();
                HanziChar::new(glyph, components)
            }
            None => HanziChar::new(
                glyph,
                vec![HanziComponent::leaf(glyph, self.unknown_strokes)],
            ),
        }
    }

    /// 组件序列 (只取字形), 叶子优先。
    pub(crate) fn _component_sequence(&self, glyph: &str) -> Vec<String> {
        self.decompose(glyph)
            .components
            .iter()
            .map(|c| c.glyph.clone())
            .collect()
    }

    /// 是否为已知可拆汉字。
    pub fn is_known(&self, glyph: &str) -> bool {
        self.lookup(glyph).is_some()
    }
}

// ────────────────────────────────────────────────────────────────
// VideoClipPlanner — 组件序列 → 视频片段生成计划
// ────────────────────────────────────────────────────────────────

/// 单个视频片段计划: 一个组件对应一镜。
#[derive(Debug, Clone, PartialEq)]
pub struct VideoClip {
    /// 片段序号 (叶子=0 起始)。
    pub index: u8,
    /// 渲染的组件字形。
    pub glyph: String,
    /// 起始时间 (秒), 累加。
    pub start_sec: f64,
    /// 时长 (秒), 由笔画数占位推导。
    pub duration_sec: f64,
    /// 层级, 供渲染层级缩放。
    pub level: u8,
}

/// 完整视频片段生成计划。
#[derive(Debug, Clone, PartialEq)]
pub struct VideoClipPlan {
    pub root: String,
    pub clips: Vec<VideoClip>,
}

impl VideoClipPlan {
    pub fn total_duration(&self) -> f64 {
        self.clips.iter().map(|c| c.duration_sec).sum()
    }
}

/// 视频片段规划器: 组件序列 → 时间轴片段计划 (stub, 不集成 ffmpeg)。
#[derive(Debug, Clone)]
pub struct VideoClipPlanner {
    /// 每个笔画对应的秒数 (渲染时长推导系数)。
    pub secs_per_stroke: f64,
    /// 最小片段时长, 避免过短。
    pub min_clip_sec: f64,
}

impl Default for VideoClipPlanner {
    fn default() -> Self {
        Self {
            secs_per_stroke: 0.4,
            min_clip_sec: 0.3,
        }
    }
}

impl VideoClipPlanner {
    pub fn new(secs_per_stroke: f64, min_clip_sec: f64) -> Self {
        Self {
            secs_per_stroke: secs_per_stroke.max(0.05),
            min_clip_sec: min_clip_sec.max(0.1),
        }
    }

    /// 由拆字结果生成视频片段计划。组件序列叶子优先 → 时间轴正向累加。
    pub fn plan(&self, decomposed: &HanziChar) -> VideoClipPlan {
        let mut start = 0.0_f64;
        let clips: Vec<VideoClip> = decomposed
            .components
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let dur = (c.strokes as f64 * self.secs_per_stroke).max(self.min_clip_sec);
                let clip = VideoClip {
                    index: i as u8,
                    glyph: c.glyph.clone(),
                    start_sec: start,
                    duration_sec: dur,
                    level: c.level,
                };
                start += dur;
                clip
            })
            .collect();
        VideoClipPlan {
            root: decomposed.root.clone(),
            clips,
        }
    }
}

// ────────────────────────────────────────────────────────────────
// RepairHarness — 修复入口 (聚合 + 完整性校验)
// ────────────────────────────────────────────────────────────────

/// 修复结果: 完整性 + 视频计划。
#[derive(Debug, Clone)]
pub struct HanziRepairResult {
    pub glyph: String,
    pub well_formed: bool,
    pub plan: VideoClipPlan,
    pub warnings: Vec<String>,
}

/// 修复执行器: 拆字 → 计划 → 完整性校验 (NT-REPAIR 自愈语义)。
#[derive(Debug, Clone)]
pub struct RepairHarness {
    engine: HanziChaiziEngine,
    planner: VideoClipPlanner,
}

impl Default for RepairHarness {
    fn default() -> Self {
        Self {
            engine: HanziChaiziEngine::default(),
            planner: VideoClipPlanner::default(),
        }
    }
}

impl RepairHarness {
    pub fn new(engine: HanziChaiziEngine, planner: VideoClipPlanner) -> Self {
        Self { engine, planner }
    }

    /// 修复单字: 拆字 + 计划 + 校验。
    pub fn repair(&self, glyph: &str) -> HanziRepairResult {
        let mut warnings = Vec::new();
        let decomposed = self.engine.decompose(glyph);
        if !decomposed.is_well_formed() {
            warnings.push(format!("malformed decomposition for '{}'", glyph));
        }
        if !self.engine.is_known(glyph) {
            warnings.push(format!("unknown glyph '{}' — treated as single leaf", glyph));
        }
        let plan = self.planner.plan(&decomposed);
        if plan.clips.is_empty() {
            warnings.push("empty clip plan".into());
        }
        HanziRepairResult {
            glyph: glyph.to_string(),
            well_formed: decomposed.is_well_formed(),
            plan,
            warnings,
        }
    }
}

// ────────────────────────────────────────────────────────────────
// SelfTest (T1) + 生产接线契约
// ────────────────────────────────────────────────────────────────

/// 汉字拆字视频修复引擎的 SelfTest — 注册名 `nt_repair_hanzi_video` (T2),
/// 结果流入 ConsciousnessTree 分支健康 (T3)。
#[derive(Debug, Clone, Copy, Default)]
pub struct HanziVideoSelfTest;

impl SelfTest for HanziVideoSelfTest {
    fn name(&self) -> &str {
        "nt_repair_hanzi_video"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();

        // C1: 拆字 — 已知汉字分解为组件序列
        let engine = HanziChaiziEngine::default();
        let li = engine.decompose("李");
        if li.components.len() != 3 {
            failures.push(format!("expected 3 components for 李, got {}", li.components.len()));
        }
        if !engine.is_known("李") {
            failures.push("李 should be known".into());
        }

        // C2: 组件序列 — 叶子优先, 末为根
        let seq = engine._component_sequence("李");
        if seq.first().map(|s| s.as_str()) != Some("木") {
            failures.push(format!("expected 木 first, got {:?}", seq.first()));
        }
        if seq.last().map(|s| s.as_str()) != Some("李") {
            failures.push(format!("expected 李 last, got {:?}", seq.last()));
        }

        // 未命中回退: 整体叶子
        let unknown = engine.decompose("𰻞");
        if unknown.components.len() != 1 {
            failures.push(format!(
                "unknown glyph should be single leaf, got {}",
                unknown.components.len()
            ));
        }

        // C3: 视频计划 — 时间轴累加 + 总时长 > 0
        let planner = VideoClipPlanner::default();
        let plan = planner.plan(&li);
        if plan.clips.is_empty() {
            failures.push("clip plan should be non-empty".into());
        } else if plan.clips[0].start_sec != 0.0 {
            failures.push("first clip must start at 0.0s".into());
        }
        if plan.total_duration() <= 0.0 {
            failures.push("total duration must be positive".into());
        }

        // C4: 修复聚合 — well_formed + 警告语义
        let harness = RepairHarness::default();
        let r_li = harness.repair("李");
        if !r_li.well_formed {
            failures.push("李 repair should be well_formed".into());
        }
        if !r_li.warnings.is_empty() {
            failures.push(format!("李 repair should have no warnings, got {:?}", r_li.warnings));
        }
        let r_unknown = harness.repair("𰻞");
        if r_unknown.warnings.is_empty() {
            failures.push("unknown glyph repair should warn".into());
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompose_known_char() {
        let engine = HanziChaiziEngine::default();
        let li = engine.decompose("李");
        assert_eq!(li.components.len(), 3);
        assert_eq!(li.components.last().unwrap().glyph, "李");
        assert_eq!(li.components.first().unwrap().glyph, "木");
    }

    #[test]
    fn test_component_sequence_order() {
        let engine = HanziChaiziEngine::default();
        let seq = engine._component_sequence("明");
        assert_eq!(seq, vec!["日", "月", "明"]);
    }

    #[test]
    fn test_unknown_glyph_fallback() {
        let engine = HanziChaiziEngine::default();
        let unknown = engine.decompose("𰻞");
        assert_eq!(unknown.components.len(), 1);
        assert!(!engine.is_known("𰻞"));
    }

    #[test]
    fn test_video_clip_plan_timeline() {
        let engine = HanziChaiziEngine::default();
        let planner = VideoClipPlanner::default();
        let plan = planner.plan(&engine.decompose("林"));
        assert_eq!(plan.clips[0].start_sec, 0.0);
        let mut prev_end = 0.0_f64;
        for c in &plan.clips {
            assert_eq!(c.start_sec, prev_end);
            prev_end += c.duration_sec;
        }
        assert!(plan.total_duration() > 0.0);
    }

    #[test]
    fn test_repair_harness_well_formed() {
        let harness = RepairHarness::default();
        let r = harness.repair("森");
        assert!(r.well_formed);
        assert!(r.warnings.is_empty());
        assert!(!r.plan.clips.is_empty());
    }

    #[test]
    fn test_repair_unknown_warns() {
        let harness = RepairHarness::default();
        let r = harness.repair("𰻞");
        assert!(r.warnings.iter().any(|w| w.contains("unknown")));
    }

    #[test]
    fn test_hanzi_video_self_test() {
        let st = HanziVideoSelfTest;
        assert!(
            st.self_test().is_ok(),
            "self-test failed: {:?}",
            st.self_test().err()
        );
    }
}
