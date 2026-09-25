//! skill_compose — 从 `nt_mind_skill_engine.rs` 拆分 (行为零变更).

use super::skill_doc::SkillDocEntry;
use super::book_to_skill::{CLAIM_RE, COVERAGE_RE, INNER_ONLY, MARKERS, REPEAT_RE};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillRelationship {
    Complement,
    Handoff,
    Substitute,
    Unrelated,
}

impl SkillRelationship {
    pub fn label(self) -> &'static str {
        match self {
            SkillRelationship::Complement => "complement",
            SkillRelationship::Handoff => "handoff",
            SkillRelationship::Substitute => "substitute",
            SkillRelationship::Unrelated => "unrelated",
        }
    }
}

/// 技能组合推断器 — 从类别/工具/触发器重叠推导两技能间关系。
pub struct SkillComposer;

impl SkillComposer {
    /// 交集系数: 两集合的交集大小 / 较小集合大小 (Jaccard-ish, 0..1)。
    fn overlap(a: &[String], b: &[String]) -> f64 {
        if a.is_empty() || b.is_empty() {
            return 0.0;
        }
        let sa: std::collections::HashSet<&str> = a.iter().map(|s| s.as_str()).collect();
        let inter = b.iter().filter(|x| sa.contains(x.as_str())).count();
        inter as f64 / a.len().min(b.len()).max(1) as f64
    }

    /// 判定两技能关系。
    pub fn compose(a: &SkillDocEntry, b: &SkillDocEntry) -> SkillRelationship {
        let tool_overlap = Self::overlap(&a.tools, &b.tools);
        let trig_overlap = Self::overlap(&a.triggers, &b.triggers);
        let same_category = !a.category.is_empty() && a.category == b.category;

        if same_category && tool_overlap >= 0.75 && trig_overlap < 0.5 {
            return SkillRelationship::Substitute;
        }
        if same_category && (trig_overlap >= 0.5 || tool_overlap >= 0.5) {
            return SkillRelationship::Complement;
        }
        if tool_overlap >= 0.5 || (tool_overlap > 0.0 && trig_overlap > 0.0) {
            return SkillRelationship::Handoff;
        }
        SkillRelationship::Unrelated
    }
}

// ────────────────────────────────────────────────────────────────
// P4: AnchorPromote (dsh-anchored-standard 吸收)
// 渐进披露阶梯 (progressive disclosure ladder) 应用到工具预算: agent
// session 首个模型请求锚定 Minimal 工具集 (真实 schema, 无自动注入上下文),
// 一旦会话 durable (首个 durable 工具/调用 或 assistant 消息) 即 promote
// 到更重的 Standard 工具集。
// ────────────────────────────────────────────────────────────────

/// 披露阶段描述: stage 0 = Minimal (锚定), stage >= 1 = Standard (提升后)。
#[derive(Debug, Clone)]
pub struct DisclosureStage {
    pub stage: u8,
    pub label: String,
    pub tool_count: usize,
    pub durable: bool,
}

impl Default for DisclosureStage {
    fn default() -> Self {
        Self {
            stage: 0,
            label: "Minimal".to_string(),
            tool_count: 2,
            durable: false,
        }
    }
}

/// 触发 promote 的 durable 信号 (dsh-anchored-standard 吸收)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromoteSignal {
    FirstDurableCall,
    FirstAssistantMessage,
    Both,
}

/// J-Space 三级门控 pass (j-space: SKILL.md §gate) — 任务分类决定加载多少机制。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskPass {
    /// 一眼可核验的单步任务 — 不加载额外机制, 不暴露工具。
    Fast,
    /// 有限多步任务, 一个可交付物 — Minimal 工具集, 只加载相关模块。
    Full,
    /// 多阶段/多文件/多轮或需持久状态 — 提升到 Standard 工具集,
    /// 启用账本 + checkpoint + 恢复。
    Loop,
}

impl TaskPass {
    pub const ALL: [TaskPass; 3] = [TaskPass::Fast, TaskPass::Full, TaskPass::Loop];

    pub fn label(self) -> &'static str {
        match self {
            TaskPass::Fast => "fast",
            TaskPass::Full => "full",
            TaskPass::Loop => "loop",
        }
    }

    /// J-Space: pass 决定披露强度 — fast 零工具, full 保持 Minimal,
    /// loop 需要 Standard (长程状态必须能带 checkpoint/恢复机制)。
    pub fn needs_standard(self) -> bool {
        matches!(self, TaskPass::Loop)
    }

    /// J-Space 地板: 无法一眼核验的答案就不是 fast。
    pub fn is_verifiable_in_glance(self) -> bool {
        matches!(self, TaskPass::Fast)
    }
}

/// 锚定-然后-promote 状态机: 先以 Minimal 工具集锚定会话, session 一旦
/// durable (首个 durable 工具/调用 或 assistant 消息) 即提升到 Standard
/// 工具集。minimal/standard 预算可配, 披露节省量 = 1 - minimal/standard。
#[derive(Debug, Clone)]
pub struct AnchorPromote {
    pub minimal_tools: usize,
    pub standard_tools: usize,
    pub promote_on: PromoteSignal,
    pub stage: u8,
    pub durable_calls: usize,
    /// J-Space pass 分层: 当前任务的推理深度门 (fast/full/loop)。
    pub pass: TaskPass,
}

impl Default for AnchorPromote {
    fn default() -> Self {
        Self {
            minimal_tools: 2,
            standard_tools: 10,
            promote_on: PromoteSignal::FirstDurableCall,
            stage: 0,
            durable_calls: 0,
            pass: TaskPass::Full,
        }
    }
}

impl AnchorPromote {
    pub fn new(minimal_tools: usize, standard_tools: usize, promote_on: PromoteSignal) -> Self {
        Self {
            minimal_tools,
            standard_tools,
            promote_on,
            stage: 0,
            durable_calls: 0,
            pass: TaskPass::Full,
        }
    }

    /// 设置 J-Space pass (fast/full/loop)。切换 pass 是显式交换 — J-Space
    /// "THE SWAP": 说清换了什么, 而不是悄悄丢。
    pub fn with_pass(mut self, pass: TaskPass) -> Self {
        self.pass = pass;
        self
    }

    /// 记录一次 durable 调用/assistant 消息。
    pub fn record_call(&mut self) {
        self.durable_calls += 1;
    }

    /// 当前生效的工具预算。
    /// J-Space pass 分层: fast 不暴露工具; full 保持 Minimal 锚定;
    /// loop 直接需要 Standard (长程必须带完整工具 + checkpoint/恢复)。
    pub fn active_tool_count(&self) -> usize {
        match self.pass {
            TaskPass::Fast => 0,
            TaskPass::Full => {
                if self.stage == 0 {
                    self.minimal_tools
                } else {
                    self.standard_tools
                }
            }
            TaskPass::Loop => self.standard_tools,
        }
    }

    /// 尝试提升: 仅在 stage 0 且 durable 信号满足 (durable_calls >= 1) 时
    /// 提升到 stage 1。返回阶段是否发生变化。J-Space: loop pass 无需等待
    /// durable — 长程任务从一开始就是完整披露。
    pub fn maybe_promote(&mut self) -> bool {
        if self.pass == TaskPass::Loop {
            self.stage = 1;
            return true;
        }
        if self.stage != 0 {
            return false;
        }
        if self.durable_calls >= 1 {
            self.stage = 1;
            true
        } else {
            false
        }
    }

    /// 披露节省量: Minimal 相对 Standard 省下的工具预算比例, 归一到 [0,1]。
    pub fn disclosure_savings(&self) -> f64 {
        match self.pass {
            TaskPass::Fast => 1.0,
            TaskPass::Loop => 0.0,
            TaskPass::Full => {
                (1.0 - self.minimal_tools as f64 / self.standard_tools.max(1) as f64)
                    .max(0.0)
                    .min(1.0)
            }
        }
    }

    /// J-Space ship 出站寄存器检查 (jspace.py mode_ship): 出站文本不得泄漏
    /// 内部寄存器记号 — 内稠密外清洁 (register switch 是总数: 扩写为干净语言
    /// 才能放出)。返回发现的泄漏, 空 = clean。
    pub fn register_check(&self, text: &str) -> Vec<String> {
        let mut findings = Vec::new();
        if text.is_empty() {
            return findings;
        }
        // 1. inner-register 记号泄漏
        let leaked: Vec<&str> = INNER_ONLY
            .iter()
            .filter(|s| text.contains(**s))
            .copied()
            .collect();
        if !leaked.is_empty() {
            findings.push(format!(
                "inner-register notation in outgoing text: {}",
                leaked.join(" ")
            ));
        }
        // 2. 状态标记泄漏
        let hot: Vec<&str> = MARKERS
            .iter()
            .filter(|m| text.to_lowercase().contains(&m.to_lowercase()))
            .copied()
            .collect();
        if !hot.is_empty() {
            findings.push(format!("state markers in outgoing text: {}", hot.join(", ")));
        }
        // 3. "verified" 未声明 coverage ("verified without stated coverage is
        //    a mood, not a result")
        for (n, line) in text.lines().enumerate() {
            if CLAIM_RE.is_match(line) && !COVERAGE_RE.is_match(line) {
                findings.push(format!(
                    "line {}: \"verified\" with no stated coverage",
                    n + 1
                ));
                break;
            }
        }
        // 4. 重复行 / 长字符 run (回环)
        let lines: Vec<&str> = text.lines().collect();
        let mut run = 1usize;
        for pair in lines.windows(2) {
            let a = pair[0].trim();
            let b = pair[1].trim();
            if !a.is_empty() && a == b {
                run += 1;
                if run >= 3 {
                    findings.push("repetition loop: a line repeats three times or more".to_string());
                    break;
                }
            } else {
                run = 1;
            }
        }
        if REPEAT_RE.is_match(text) {
            findings.push("repetition loop: a character run of 20 or more".to_string());
        }
        findings
    }
}
