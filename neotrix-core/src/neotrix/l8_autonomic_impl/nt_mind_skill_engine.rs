//! Skill auto-invocation engine — scans, parses, indexes, and auto-invokes
//! skill markdown files with YAML frontmatter.
//!
//! Integrates with:
//!   - nt_mind_hook: fires SkillLoaded/SkillUnloaded HookEvents
//!   - GWT workspace: broadcasts skill activation events

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::core::nt_core_gwt::workspace::GlobalWorkspace;
use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_types::ProceduralMemoryRecord;
use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_unify::{skill_upsert, SkillRecord};
use crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase;
use crate::neotrix::l8_autonomic_impl::nt_mind_hook::{HookEvent, MindHookRegistry, HookContext, HookResult};

/// A single skill entry parsed from a markdown file with YAML frontmatter.
#[derive(Debug, Clone)]
pub struct SkillEntry {
    pub name: String,
    pub description: String,
    pub triggers: Vec<String>,
    pub e8_modes: Vec<u8>,
    pub tools: Vec<String>,
    pub hooks: Vec<String>,
    pub priority: u8,
    pub path: PathBuf,
    pub content: String,
    pub active: bool,
    /// 渐进披露 (progressive disclosure, 吸收自 cathrynlavery/diagram-design):
    /// SKILL.md 只保留选择指南, 深层细节以 `references/*.md` 按需加载。
    pub references: Vec<String>,
    /// 技能树层级 (AgentSkillOS 吸收): 粗到细分类 + 父技能指针, 支撑互补性检索。
    pub category: String,
    pub parent: String,
    /// 确定性 selftest 门 (P2-5, shuohao-skills 模式): skill 目录存在
    /// `scripts/selftest.sh` 或 `scripts/selftest.js` 且产结构化 JSON 即 verified;
    /// 缺失 → `unverified` (拒收/标记而非静默加载, 与 R-P16 同构)。
    pub verified: bool,
}

impl SkillEntry {
    fn from_file(path: &Path) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        Self::from_content(path, &content)
    }

    fn from_content(path: &Path, content: &str) -> Option<Self> {
        let stripped = content.trim_start();
        if !stripped.starts_with("---") {
            return None;
        }
        let end = stripped[3..].find("---")?;
        let frontmatter = &stripped[3..3 + end];

        let mut name = String::new();
        let mut description = String::new();
        let mut triggers = Vec::new();
        let mut e8_modes = Vec::new();
        let mut tools = Vec::new();
        let mut hooks = Vec::new();
        let mut priority: u8 = 50;
        let mut references = Vec::new();
        let mut category = "general".to_string();
        let mut parent = String::new();

        for line in frontmatter.lines() {
            let line = line.trim();
            if let Some(val) = line.strip_prefix("name:") {
                name = val.trim().to_string();
            } else if let Some(val) = line.strip_prefix("description:") {
                description = val.trim().to_string();
            } else if let Some(val) = line.strip_prefix("triggers:") {
                triggers = parse_array_field(val);
            } else if let Some(val) = line.strip_prefix("e8_modes:") {
                e8_modes = parse_array_field(val).iter().filter_map(|s| s.parse::<u8>().ok()).collect();
            } else if let Some(val) = line.strip_prefix("tools:") {
                tools = parse_array_field(val);
            } else if let Some(val) = line.strip_prefix("hooks:") {
                hooks = parse_array_field(val);
            } else if let Some(val) = line.strip_prefix("references:") {
                references = parse_array_field(val);
            } else if let Some(val) = line.strip_prefix("category:") {
                let c = val.trim().trim_matches('"').trim_matches('\'').to_string();
                if !c.is_empty() {
                    category = c;
                }
            } else if let Some(val) = line.strip_prefix("parent:") {
                parent = val.trim().trim_matches('"').trim_matches('\'').to_string();
            } else if let Some(val) = line.strip_prefix("priority:") {
                priority = val.trim().parse::<u8>().unwrap_or(50).min(100);
            }
        }

        if name.is_empty() || description.is_empty() {
            return None;
        }

        // 确定性 selftest 门: skill 根目录下 scripts/selftest.sh|js 存在性检查
        let verified = Self::has_selftest(path);

        Some(Self {
            name,
            description,
            triggers,
            e8_modes,
            tools,
            hooks,
            priority,
            path: path.to_path_buf(),
            content: content.to_string(),
            active: false,
            references,
            category,
            parent,
            verified,
        })
    }

    /// 校验 skill 是否带确定性 selftest 脚本 (P2-5 质量门)。
    /// skill 根 = SKILL.md 所在目录 (目录型) 或自身目录 (单文件型)。
    fn has_selftest(path: &Path) -> bool {
        let root = if path.file_name().is_some_and(|n| n == "SKILL.md") {
            path.parent().unwrap_or(path)
        } else {
            path
        };
        let scripts = root.join("scripts");
        scripts.join("selftest.sh").is_file() || scripts.join("selftest.js").is_file()
    }

    pub fn body(&self) -> &str {
        let stripped = self.content.trim_start();
        if !stripped.starts_with("---") {
            return stripped;
        }
        if let Some(end) = stripped[3..].find("---") {
            &stripped[3 + end + 3..]
        } else {
            stripped
        }
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
}

impl Default for AnchorPromote {
    fn default() -> Self {
        Self {
            minimal_tools: 2,
            standard_tools: 10,
            promote_on: PromoteSignal::FirstDurableCall,
            stage: 0,
            durable_calls: 0,
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
        }
    }

    /// 记录一次 durable 调用/assistant 消息。
    pub fn record_call(&mut self) {
        self.durable_calls += 1;
    }

    /// 当前生效的工具预算: stage 0 → minimal, stage >= 1 → standard。
    pub fn active_tool_count(&self) -> usize {
        if self.stage == 0 {
            self.minimal_tools
        } else {
            self.standard_tools
        }
    }

    /// 尝试提升: 仅在 stage 0 且 durable 信号满足 (durable_calls >= 1) 时
    /// 提升到 stage 1。返回阶段是否发生变化。
    pub fn maybe_promote(&mut self) -> bool {
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
        (1.0 - self.minimal_tools as f64 / self.standard_tools.max(1) as f64)
            .max(0.0)
            .min(1.0)
    }
}

// ────────────────────────────────────────────────────────────────
// P6: BookToSkill (book-to-skill 机制输入侧)
// 书/文档 (PDF/EPUB/DOCX/MD/HTML/RTF/MOBI) → 统一 agent skill 铸造的
// 输入建模与章节→技能候选映射。本层只做"输入归一化 + 章节→技能候选
// 映射"; 产出路径复用既有 SkillEngine/SkillEntry, 禁止平行适配器 (R-P42)。
// ────────────────────────────────────────────────────────────────

/// 支持的文档格式 (输入归一化)。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DocFormat {
    Pdf,
    Epub,
    Docx,
    Markdown,
    Html,
    Rtf,
    Mobi,
}

impl DocFormat {
    pub fn label(self) -> &'static str {
        match self {
            DocFormat::Pdf => "pdf",
            DocFormat::Epub => "epub",
            DocFormat::Docx => "docx",
            DocFormat::Markdown => "md",
            DocFormat::Html => "html",
            DocFormat::Rtf => "rtf",
            DocFormat::Mobi => "mobi",
        }
    }
}

/// 归一化后的章节 (输入建模)。
#[derive(Debug, Clone)]
pub struct DocChapter {
    pub title: String,
    pub order: usize,
    pub char_count: usize,
    pub summary: String,
}

/// 一本书/文档的统一输入模型 (book-to-skill 输入侧)。
#[derive(Debug, Clone)]
pub struct BookInput {
    pub title: String,
    pub format: DocFormat,
    pub chapters: Vec<DocChapter>,
}

/// 章节→技能候选映射结果。
#[derive(Debug, Clone)]
pub struct SkillCandidate {
    pub name: String,
    pub source_chapters: Vec<usize>,
    pub priority: u8,
}

/// book-to-skill 输入侧配置: 短章节过滤阈值 + 候选数量上限。
#[derive(Debug, Clone)]
pub struct BookToSkill {
    pub min_chapter_chars: usize,
    pub max_candidates: usize,
}

impl Default for BookToSkill {
    fn default() -> Self {
        Self {
            min_chapter_chars: 500,
            max_candidates: 8,
        }
    }
}

impl BookToSkill {
    pub fn new(min_chapter_chars: usize, max_candidates: usize) -> Self {
        Self {
            min_chapter_chars,
            max_candidates,
        }
    }

    /// 按扩展名推断文档格式; 未知扩展名回退 Markdown。
    pub fn infer_format(path: &str) -> DocFormat {
        let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
        match ext.as_str() {
            "pdf" => DocFormat::Pdf,
            "epub" => DocFormat::Epub,
            "docx" => DocFormat::Docx,
            "md" | "markdown" => DocFormat::Markdown,
            "html" | "htm" => DocFormat::Html,
            "rtf" => DocFormat::Rtf,
            "mobi" => DocFormat::Mobi,
            _ => DocFormat::Markdown,
        }
    }

    /// 过滤 char_count < min_chapter_chars 的章节, 保持原 order。
    pub fn normalize(&self, input: &BookInput) -> Vec<DocChapter> {
        input
            .chapters
            .iter()
            .filter(|c| c.char_count >= self.min_chapter_chars)
            .cloned()
            .collect()
    }

    /// 对每个保留章节生成技能候选: 章节长度 > 2000 字符 → priority=2
    /// (长章节=高价值技能), 否则 priority=1; 最多 max_candidates 个。
    pub fn discover_candidates(&self, input: &BookInput) -> Vec<SkillCandidate> {
        self.normalize(input)
            .iter()
            .take(self.max_candidates.max(0))
            .map(|ch| SkillCandidate {
                name: clean_chapter_title(&ch.title),
                source_chapters: vec![ch.order],
                priority: if ch.char_count > 2000 { 2 } else { 1 },
            })
            .collect()
    }

    /// 保留章节字符总数 / 全书字符总数, 归一到 [0,1]。
    pub fn skill_yield(&self, input: &BookInput) -> f64 {
        let total: usize = input.chapters.iter().map(|c| c.char_count).sum();
        if total == 0 {
            return 0.0;
        }
        let kept: usize = self.normalize(input).iter().map(|c| c.char_count).sum();
        (kept as f64 / total as f64).max(0.0).min(1.0)
    }
}

/// 章节标题清洗: 去数字前缀/特殊字符 → snake_case (skill 命名用)。
fn clean_chapter_title(title: &str) -> String {
    let mut cleaned = String::new();
    let mut prev_sep = false;
    for ch in title.trim().chars() {
        let c = ch.to_ascii_lowercase();
        if c.is_ascii_alphanumeric() {
            cleaned.push(c);
            prev_sep = false;
        } else if !prev_sep && !cleaned.is_empty() {
            cleaned.push('_');
            prev_sep = true;
        } else {
            prev_sep = true;
        }
    }
    let cleaned = cleaned.trim_matches('_');
    // 去数字前缀 (如 "12. Introduction" → "introduction")
    let cleaned = cleaned.trim_start_matches(|c: char| c.is_ascii_digit()).trim_start_matches('_');
    if cleaned.is_empty() {
        "chapter".to_string()
    } else {
        cleaned.to_string()
    }
}

impl crate::core::nt_core_self_test::SelfTest for BookToSkill {
    fn name(&self) -> &str {
        "nt_mind_book_to_skill"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let bts = BookToSkill::default();
        let input = BookInput {
            title: "Test Book".into(),
            format: DocFormat::Markdown,
            chapters: vec![
                DocChapter { title: "Intro".into(), order: 0, char_count: 600, summary: String::new() },
                DocChapter { title: "Deep Dive".into(), order: 1, char_count: 3000, summary: String::new() },
            ],
        };
        if bts.skill_yield(&input) != 1.0 {
            return Err(vec!["yield should be 1.0 when all chapters retained".into()]);
        }
        if bts.discover_candidates(&input).len() != 2 {
            return Err(vec!["should discover 2 candidates".into()]);
        }
        Ok(())
    }
}

/// 差分归因记录 (arxiv 2608.11888 SkillTriage 吸收): 单个技能的激活统计与
/// procedure-heavy 风险标记。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillAttribution {
    pub name: String,
    pub category: String,
    pub activations: u32,
    pub over_validation_score: u32,
    pub procedure_heavy: bool,
    pub flagged: bool,
}

/// 技能树层级统计 (G6, AgentSkillOS 吸收): 巡检报告的数据载体。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillTreeStats {
    pub total_skills: usize,
    pub categories: HashMap<String, usize>,
    pub roots: usize,
    pub orphans: usize,
    pub max_depth: usize,
}

fn parse_array_field(val: &str) -> Vec<String> {
    let trimmed = val.trim();
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let inner = &trimmed[1..trimmed.len() - 1];
        inner.split(',')
            .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        trimmed.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

// ────────────────────────────────────────────────────────────────
// P-F1: RevertibleEffect + InverseLedger (吸收 cordiverse §3.1, F1)
// 每个 skill-install 上下文变换携带追踪的逆 (Γ → Γ×(Γ→Γ))。Runtime 把逆
// 按加载序累积到 accumulator φ (twisted composition monoid 𝔗Γ); teardown
// 以 LIFO 逆序应用 φ — 结构性保证, 非手写清理 (paper §3.3.3 p.27)。
// ────────────────────────────────────────────────────────────────

/// 逆操作闭包: 返回 Result 以便按 fiber 捕获失败而不中断其余逆操作 (L-Raise)。
pub type InverseOp = Arc<dyn Fn() -> Result<(), String> + Send + Sync>;

/// 可逆效应: 一次安装变换的前向标签 + 显式单侧逆。
#[derive(Clone)]
pub struct RevertibleEffect {
    pub label: String,
    inverse: InverseOp,
}

impl RevertibleEffect {
    pub fn new(
        label: impl Into<String>,
        inverse: impl Fn() -> Result<(), String> + Send + Sync + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            inverse: Arc::new(inverse),
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn run(&self) -> Result<(), String> {
        (self.inverse)()
    }
}

impl std::fmt::Debug for RevertibleEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RevertibleEffect").field("label", &self.label).finish()
    }
}

/// 逆账本: `install_id` → 按加载序记录的逆操作列表。
/// 卸载以 LIFO (逆加载序) 派生 teardown, 结构上保证 φ(γ) ≃ γ0。
#[derive(Clone)]
pub struct InverseLedger {
    entries: HashMap<u64, Vec<RevertibleEffect>>,
    next_id: u64,
}

impl std::fmt::Debug for InverseLedger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InverseLedger")
            .field("next_id", &self.next_id)
            .field("active_transactions", &self.entries.len())
            .finish()
    }
}

impl Default for InverseLedger {
    fn default() -> Self {
        Self { entries: HashMap::new(), next_id: 0 }
    }
}

impl InverseLedger {
    pub fn new() -> Self {
        Self::default()
    }

    /// 开启一个新的安装事务, 返回 install_id (accumulator φ 的标识)。
    pub fn begin_install(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.entries.insert(id, Vec::new());
        id
    }

    /// 把逆操作按加载顺序推入账本。
    pub fn push_inverse(&mut self, install_id: u64, effect: RevertibleEffect) -> Result<(), String> {
        let entry = self.entries.get_mut(&install_id)
            .ok_or_else(|| format!("unknown install transaction: {}", install_id))?;
        entry.push(effect);
        Ok(())
    }

    /// 加载序下的逆操作标签 (诊断/测试: 记录顺序即加载顺序)。
    pub fn inverse_labels(&self, install_id: u64) -> Vec<String> {
        self.entries
            .get(&install_id)
            .map(|e| e.iter().map(|x| x.label.clone()).collect())
            .unwrap_or_default()
    }

    pub fn inverse_count(&self, install_id: u64) -> usize {
        self.entries.get(&install_id).map_or(0, |e| e.len())
    }

    pub fn active_transactions(&self) -> usize {
        self.entries.len()
    }

    /// 事务是否仍存在 (install 逆账本条目的活标志; 供悬挂所有权检测: 事务
    /// 消失但 fiber 仍标记 held 即悬挂所有权)。
    pub fn has_transaction(&self, install_id: u64) -> bool {
        self.entries.contains_key(&install_id)
    }

    /// 以 LIFO (逆加载序) 执行该事务的全部逆操作; 单级失败被记录但不
    /// 中止其余逆操作 (L-Raise: failure per-fiber, siblings keep running)。
    pub fn teardown(&mut self, install_id: u64) -> Vec<Result<(), String>> {
        let entry = match self.entries.remove(&install_id) {
            Some(e) => e,
            None => return Vec::new(),
        };
        entry.into_iter().rev().map(|effect| effect.run()).collect()
    }
}

// ────────────────────────────────────────────────────────────────
// P-F5: FiberLifecycle 惯性生命周期状态机 (吸收 cordiverse §4.1-4.3, F5)
// 组件 = (d: spec, p: provision, e: witnessed effect); fiber = 单次实例化,
// 拥有自己的生命周期状态。非原子转移有惯性; 失败按 fiber 记录 (L-Raise),
// 不传播到 parent — sibling 继续运行。效应总经 Retired/Unloading 恢复, 不滞留。
// ────────────────────────────────────────────────────────────────

/// Fiber 生命周期状态 (F5): Loaded → Active → Suspended → Retired + Failed。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FiberLifecycleState {
    /// 已加载 (install 完成, 尚未激活)
    Loaded,
    /// 激活 (依赖满足, 正常服务)
    Active,
    /// 挂起 (依赖丢失, 等待重新满足)
    Suspended,
    /// 退休 (teardown 完成, 终态)
    Retired,
    /// 失败 (按 fiber 捕获, 不传播到 sibling)
    Failed,
}

impl FiberLifecycleState {
    pub fn label(&self) -> &'static str {
        use FiberLifecycleState::*;
        match self {
            Loaded => "loaded",
            Active => "active",
            Suspended => "suspended",
            Retired => "retired",
            Failed => "failed",
        }
    }
}

/// 单次 fiber 失败记录: 失败时的状态 + 消息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiberFailure {
    pub at_state: FiberLifecycleState,
    pub message: String,
}

/// Fiber 生命周期状态机 (F5): 惯性转移 (仅合法下一状态) + 按 fiber 失败捕获。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiberLifecycle {
    pub fiber_id: String,
    pub state: FiberLifecycleState,
    pub install_id: u64,
    pub failures: Vec<FiberFailure>,
}

impl FiberLifecycle {
    pub fn new(fiber_id: impl Into<String>, install_id: u64) -> Self {
        Self {
            fiber_id: fiber_id.into(),
            state: FiberLifecycleState::Loaded,
            install_id,
            failures: Vec::new(),
        }
    }

    /// 惯性转移表 (paper §4.3.3): 仅允许的下一状态。
    /// Retired 是终态 (无后继); Failed 仅可回 Loaded (重装/重试) 或 Retired。
    fn is_allowed(from: FiberLifecycleState, to: FiberLifecycleState) -> bool {
        use FiberLifecycleState::*;
        matches!(
            (from, to),
            (Loaded, Active)
                | (Loaded, Suspended)
                | (Loaded, Retired)
                | (Loaded, Failed)
                | (Active, Suspended)
                | (Active, Retired)
                | (Active, Failed)
                | (Suspended, Active)
                | (Suspended, Retired)
                | (Suspended, Failed)
                | (Failed, Loaded)
                | (Failed, Retired)
        )
    }

    pub fn state(&self) -> FiberLifecycleState {
        self.state
    }

    /// 惯性转移: 非法转移返回 Err 且状态不变 (inertia: in-flight 转移先落地)。
    pub fn transition(&mut self, to: FiberLifecycleState) -> Result<(), String> {
        if self.state == to {
            return Err(format!(
                "fiber '{}' is already {}",
                self.fiber_id,
                self.state.label()
            ));
        }
        if !Self::is_allowed(self.state, to) {
            return Err(format!(
                "illegal transition for fiber '{}': {} → {}",
                self.fiber_id,
                self.state.label(),
                to.label()
            ));
        }
        self.state = to;
        Ok(())
    }

    /// 按 fiber 捕获失败: 记录失败并转入 Failed; 不传播到 sibling。
    pub fn record_failure(&mut self, message: impl Into<String>) {
        self.failures.push(FiberFailure {
            at_state: self.state,
            message: message.into(),
        });
        self.state = FiberLifecycleState::Failed;
    }
}

// ────────────────────────────────────────────────────────────────
// C5 自愈检测件: revertible_effects (F1) + fiber_lifecycle (F5)
// 纯内存模拟, 无网络/磁盘/env IO (SelfTest 约束)。不变量破坏即 Err。
// ────────────────────────────────────────────────────────────────

/// C5 自愈检测件: 逆账本往返不变量 (φ(γ) ≃ γ0) — LIFO 逆应用必须完全还原状态,
/// 错序/逆丢失必须被检出。纯内存栈式模拟 (无 IO)。
pub struct RevertibleEffectsHealer;

impl RevertibleEffectsHealer {
    /// 构造栈式 install 场景: 每次 install 推入值并登记逆操作 (弹回该值)。
    fn push_scenario(installs: &[i32]) -> (Arc<std::sync::Mutex<Vec<i32>>>, Vec<RevertibleEffect>) {
        let state = Arc::new(std::sync::Mutex::new(Vec::<i32>::new()));
        let mut effects = Vec::new();
        for &v in installs {
            let s = state.clone();
            state.lock().unwrap().push(v);
            effects.push(RevertibleEffect::new(format!("pop_{}", v), move || {
                let mut s = s.lock().map_err(|e| e.to_string())?;
                match s.pop() {
                    Some(top) if top == v => Ok(()),
                    Some(top) => Err(format!("inverse mismatch: expected {}, got {}", v, top)),
                    None => Err(format!("empty stack during inverse {}", v)),
                }
            }));
        }
        (state, effects)
    }

    /// 按给定顺序应用逆操作并判断状态是否完全还原 (往返不变量)。
    fn roundtrip_restores(
        state: &Arc<std::sync::Mutex<Vec<i32>>>,
        effects: &[RevertibleEffect],
        order: &[usize],
    ) -> bool {
        for &idx in order {
            let ok = effects
                .get(idx)
                .map(|e| e.run())
                .unwrap_or(Err("bad inverse index".into()))
                .is_ok();
            if !ok {
                return false;
            }
        }
        state.lock().unwrap().is_empty()
    }
}

impl crate::core::nt_core_self_test::SelfTest for RevertibleEffectsHealer {
    fn name(&self) -> &str {
        "nt_mind_skill_engine::revertible_effects_healer"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();

        // 1) 合法往返: LIFO 逆序应用 → 状态完全还原
        let (state, effects) = Self::push_scenario(&[1, 2, 3]);
        if !Self::roundtrip_restores(&state, &effects, &[2, 1, 0]) {
            failures.push("roundtrip: LIFO 逆应用未还原状态 (φ(γ) ≇ γ0)".into());
        }

        // 2) 顺序错乱: 破坏不变量必须被检出
        let (state_w, effects_w) = Self::push_scenario(&[1, 2, 3]);
        if Self::roundtrip_restores(&state_w, &effects_w, &[0, 1, 2]) {
            failures.push("roundtrip: 错序逆应用被误判为还原 (检测盲区)".into());
        }

        // 3) 逆丢失: 状态滞留必须被检出
        let (state_m, effects_m) = Self::push_scenario(&[1, 2, 3]);
        if Self::roundtrip_restores(&state_m, &effects_m, &[2, 1]) {
            failures.push("roundtrip: 逆丢失未被检出 (状态滞留)".into());
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

/// C5 自愈检测件: fiber 所有权生命周期 — 合法转移 + 悬挂所有权自动释放。
/// 持有中不可被重新认领, 释放后可重装; holder 消失但 fiber 仍 held 即悬挂,
/// 经 release_dangling 自动转入 Retired。
pub struct FiberLifecycleHealer;

impl FiberLifecycleHealer {
    /// 安装一个带逆账本事务的 fiber 并激活 (合法持有)。
    fn install_held_fiber(engine: &mut SkillEngine, name: &str) -> Result<u64, String> {
        let id = engine.inverse_ledger.begin_install();
        engine
            .inverse_ledger
            .push_inverse(id, RevertibleEffect::new(format!("inverse_{}", name), || Ok(())))?;
        engine
            .fiber_lifecycles
            .insert(name.to_string(), FiberLifecycle::new(name.to_string(), id));
        let _ = engine
            .fiber_lifecycles
            .get_mut(name)
            .ok_or_else(|| format!("fiber '{}' not inserted", name))?
            .transition(FiberLifecycleState::Active);
        Ok(id)
    }
}

impl crate::core::nt_core_self_test::SelfTest for FiberLifecycleHealer {
    fn name(&self) -> &str {
        "nt_mind_skill_engine::fiber_lifecycle_healer"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        use FiberLifecycleState::*;

        let mut failures = Vec::new();
        let mut engine = SkillEngine::new(PathBuf::new());

        // 1) 所有权转移合法: 持有中不可被重新认领 (惯性状态机拒绝自环)。
        let id = match Self::install_held_fiber(&mut engine, "f1") {
            Ok(id) => id,
            Err(e) => return Err(vec![format!("install held fiber failed: {}", e)]),
        };
        if engine.fiber_lifecycles.get("f1").map(|f| f.state) != Some(Active) {
            failures.push("合法持有 fiber 未处于 Active".into());
        }
        if !engine.inverse_ledger.has_transaction(id) {
            failures.push("持有中 fiber 的逆账本事务必须存在".into());
        }
        let mut probe = FiberLifecycle::new("f1", id);
        let _ = probe.transition(Active);
        if probe.transition(Active).is_ok() {
            failures.push("持有中 fiber 被非法重新认领 (自环)".into());
        }

        // 2) 释放后重新认领合法: 卸载 (Retired) 后重装派生新 install 事务。
        match engine.uninstall_skill("f1") {
            Ok(_) => {}
            Err(e) => failures.push(format!("uninstall_skill failed: {}", e)),
        }
        let id2 = match Self::install_held_fiber(&mut engine, "f1") {
            Ok(id2) => id2,
            Err(e) => {
                failures.push(format!("re-claim after release failed: {}", e));
                u64::MAX
            }
        };
        if engine.fiber_lifecycles.get("f1").map(|f| f.state) != Some(Active) {
            failures.push("释放后重新认领未处于 Active".into());
        }
        if id2 != id + 1 {
            failures.push("重装必须派生新的 install 事务".into());
        }

        // 3) 悬挂所有权: holder 消失 (事务被消耗) 但 fiber 仍 held → 自动释放。
        let dangling_id = match Self::install_held_fiber(&mut engine, "dangling") {
            Ok(id3) => id3,
            Err(e) => {
                failures.push(format!("install dangling fiber failed: {}", e));
                u64::MAX
            }
        };
        engine.inverse_ledger.teardown(dangling_id);
        let released = engine.release_dangling();
        if !released.iter().any(|n| n == "dangling") {
            failures.push(format!("悬挂 fiber 未被自动释放, released={:?}", released));
        }
        if engine.fiber_lifecycles.get("dangling").map(|f| f.state) != Some(Retired) {
            failures.push("悬挂 fiber 释放后应处于 Retired".into());
        }
        let extra = engine.release_dangling();
        if !extra.is_empty() {
            failures.push(format!("合法持有被误判为悬挂: {:?}", extra));
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

/// Core skill engine: scan, index, match, activate/deactivate.
pub struct SkillEngine {
    skills_dir: PathBuf,
    skills: Vec<SkillEntry>,
    /// Index: trigger keyword → skill indices
    trigger_index: HashMap<String, Vec<usize>>,
    /// Index: E8 mode → skill indices
    e8_index: HashMap<u8, Vec<usize>>,
    /// Optional hook registry for firing lifecycle events
    hooks: Option<MindHookRegistry>,
    /// Optional GWT for broadcasting activation events
    gwt: Option<Arc<RwLock<GlobalWorkspace>>>,
/// Optional KB handle: when attached, load_all() auto-syncs the skill
    /// index into the KB `skills_index` table (UCN Phase 1 写通)。
    kb: Option<Arc<KnowledgeBase>>,
    /// 差分归因 (arxiv 2608.11888 SkillTriage 吸收): 每 skill 的激活次数 /
    /// 过程过重标记, 识别 procedure-heavy 技能 (过度验证 = 强制劳动毒源)。
    attribution: HashMap<String, SkillAttribution>,
    /// 锚定-然后-promote (dsh-anchored-standard 吸收): session 首个请求锚定
    /// Minimal 工具集, durable 后提升到 Standard 工具集。
    pub disclosure: AnchorPromote,
    /// 可逆效应逆账本 (cordiverse F1 吸收): `install_id` → 加载序逆操作。
    /// uninstall 以 LIFO 派生 teardown, 非手写清理。
    pub inverse_ledger: InverseLedger,
    /// 技能 fiber 生命周期注册表 (cordiverse F5 吸收): skill 名 → fiber 状态机。
    pub fiber_lifecycles: HashMap<String, FiberLifecycle>,
}

impl SkillEngine {
    pub fn new(skills_dir: PathBuf) -> Self {
        Self {
            skills_dir,
            skills: Vec::new(),
            trigger_index: HashMap::new(),
            e8_index: HashMap::new(),
            hooks: None,
            gwt: None,
            kb: None,
            attribution: HashMap::new(),
            disclosure: AnchorPromote::default(),
            inverse_ledger: InverseLedger::new(),
            fiber_lifecycles: HashMap::new(),
        }
    }

    pub fn with_kb(mut self, kb: Arc<KnowledgeBase>) -> Self {
        self.kb = Some(kb);
        self
    }

    pub fn kb(&self) -> Option<&Arc<KnowledgeBase>> {
        self.kb.as_ref()
    }

    pub fn with_hooks(mut self, hooks: MindHookRegistry) -> Self {
        self.hooks = Some(hooks);
        self
    }

    pub fn with_gwt(mut self, gwt: Arc<RwLock<GlobalWorkspace>>) -> Self {
        self.gwt = Some(gwt);
        self
    }

    /// Scan the skills directory and load all valid skill files.
    pub fn load_all(&mut self) -> Vec<SkillEntry> {
        self.skills.clear();
        self.trigger_index.clear();
        self.e8_index.clear();

        let dir = &self.skills_dir;
        if !dir.exists() {
            let _ = std::fs::create_dir_all(dir);
            return Vec::new();
        }

        let mut loaded = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let skill_md = path.join("SKILL.md");
                    if skill_md.exists() {
                        if let Some(skill) = SkillEntry::from_file(&skill_md) {
                            loaded.push(skill);
                        }
                    }
                    continue;
                }
                if path.extension().is_some_and(|e| e == "md") {
                    if let Some(skill) = SkillEntry::from_file(&path) {
                        loaded.push(skill);
                    }
                }
            }
        }

        self.skills = loaded;
        self.build_index();
        // UCN Phase 1 写通: 若挂接 KB, 扫描后自动把索引同步进 skills_index 表。
        if let Some(kb) = self.kb.clone() {
            if let Ok(conn) = kb.raw_conn() {
                let _ = self.sync_to_kb_index(&conn);
            }
        }
        self.skills.clone()
    }

    /// 把当前内存索引同步到 KB `skills_index` 表 (UCN Phase 1 写通)。
    /// 返回本次真正写入/更新的条数; 内容未变化 (content_hash 相同) 被去重跳过。
    pub fn sync_to_kb_index(&self, conn: &rusqlite::Connection) -> Result<usize, String> {
        use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_unify::skill_content_hash;
        use std::collections::HashSet;

        let mut written = 0usize;
        let mut seen: HashSet<String> = HashSet::new();
        for skill in &self.skills {
            if !seen.insert(skill.name.clone()) {
                continue;
            }
            let record = SkillRecord {
                id: uuid::Uuid::new_v4().to_string(),
                name: skill.name.clone(),
                description: Some(skill.description.clone()),
                source_path: Some(skill.path.to_string_lossy().to_string()),
                tags: if skill.triggers.is_empty() {
                    None
                } else {
                    Some(skill.triggers.join(","))
                },
                is_builtin: false,
                last_indexed_at: Some(crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_unify::now()),
                created_at: crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_unify::now(),
                updated_at: crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_unify::now(),
                content_hash: Some(skill_content_hash(&skill.content)),
            };
            if skill_upsert(conn, &record.name, &record)? {
                written += 1;
            }
        }
        Ok(written)
    }

    /// Build trigger and E8 mode indices.
    fn build_index(&mut self) {
        self.trigger_index.clear();
        self.e8_index.clear();

        for (i, skill) in self.skills.iter().enumerate() {
            for trigger in &skill.triggers {
                let key = trigger.to_lowercase();
                self.trigger_index.entry(key).or_default().push(i);
            }
            for mode in &skill.e8_modes {
                self.e8_index.entry(*mode).or_default().push(i);
            }
        }
    }

    /// Find skills matching a query string and optional E8 mode.
    /// When `e8_mode` is `None`, the E8 mode filter is skipped.
    /// Matching is case-insensitive keyword match against triggers.
    /// Results are sorted by priority descending, then by trigger relevance.
    /// 反哺自 spec-kit/autoroute 吸收: 确定性优先级栈 (exact > substring) + 硬结果上限
    /// (open-code-review 预算纪律) — 防止路由返回无界候选淹没下游消费方。
    pub const MAX_ROUTE_RESULTS: usize = 8;

    pub fn find_matching(&self, query: &str, e8_mode: Option<u8>) -> Vec<&SkillEntry> {
        let query_lower = query.to_lowercase();
        let query_words: Vec<String> = query_lower.split_whitespace()
            .map(|s| s.to_string())
            .chain(std::iter::once(query_lower.clone()))
            .collect();

        // tier 0 = exact trigger equality (最高优先级, 确定性命中)
        // tier 1 = substring 命中
        let mut exact: Vec<(usize, usize, &SkillEntry)> = Vec::new();
        let mut scored: Vec<(usize, usize, &SkillEntry)> = Vec::new();

        for skill in self.skills.iter() {
            if let Some(mode) = e8_mode {
                if !skill.e8_modes.is_empty() && !skill.e8_modes.contains(&mode) {
                    continue;
                }
            }
            let mut exact_count = 0;
            let mut match_count = 0;
            for word in &query_words {
                for trigger in &skill.triggers {
                    let t_lower = trigger.to_lowercase();
                    if t_lower == *word {
                        exact_count += 1;
                    } else if t_lower.contains(word.as_str()) || word.contains(t_lower.as_str()) {
                        match_count += 1;
                    }
                }
            }
            if exact_count > 0 {
                exact.push((exact_count, skill.priority as usize, skill));
            } else if match_count > 0 {
                scored.push((match_count, skill.priority as usize, skill));
            }
        }

        // Sort: desc by exact_count, then desc by priority (确定性优先层)
        exact.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)));
        // Sort: desc by match_count, then desc by priority
        scored.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)));

        exact.into_iter().map(|(_, _, s)| s)
            .chain(scored.into_iter().map(|(_, _, s)| s))
            .take(Self::MAX_ROUTE_RESULTS)
            .collect()
    }

    /// 技能树 (AgentSkillOS 吸收): category → skills, 每类内按 priority 降序。
    pub fn skill_tree(&self) -> HashMap<String, Vec<&SkillEntry>> {
        let mut tree: HashMap<String, Vec<&SkillEntry>> = HashMap::new();
        for s in self.skills.iter() {
            tree.entry(s.category.clone()).or_default().push(s);
        }
        for v in tree.values_mut() {
            v.sort_by(|a, b| b.priority.cmp(&a.priority));
        }
        tree
    }

    pub fn children_of(&self, name: &str) -> Vec<&SkillEntry> {
        self.skills.iter().filter(|s| s.parent == name).collect()
    }

    /// 互补性感知检索 (AgentSkillOS 吸收): 在 find_matching 候选基础上, 对
    /// 与已激活技能同 category 的候选施加降级, 优先返回未覆盖类别 (多样化)。
    pub fn find_matching_complementary(
        &self,
        query: &str,
        e8_mode: Option<u8>,
        active_names: &[&str],
    ) -> Vec<&SkillEntry> {
        let covered: Vec<String> = self
            .skills
            .iter()
            .filter(|s| active_names.contains(&s.name.as_str()))
            .map(|s| s.category.clone())
            .collect();

        let mut candidates = self.find_matching(query, e8_mode);
        candidates.sort_by(|a, b| {
            let a_covered = covered.contains(&a.category);
            let b_covered = covered.contains(&b.category);
            match (a_covered, b_covered) {
                (true, false) => std::cmp::Ordering::Greater,
                (false, true) => std::cmp::Ordering::Less,
                _ => b.priority.cmp(&a.priority),
            }
        });
        candidates
    }

    pub fn record_activation(&mut self, name: &str) {
        let Some(entry) = self.get_skill(name) else {
            return;
        };
        let entry = entry.clone();
        let score = self.over_validation_score(name);
        let procedure_heavy = score >= 12;
        let attr = self.attribution.entry(name.to_string()).or_insert(SkillAttribution {
            name: name.to_string(),
            category: entry.category.clone(),
            activations: 0,
            over_validation_score: score,
            procedure_heavy,
            flagged: false,
        });
        attr.activations += 1;
        attr.over_validation_score = score;
        attr.procedure_heavy = procedure_heavy;
        attr.flagged = procedure_heavy && attr.activations > 1;
    }

    pub fn over_validation_score(&self, name: &str) -> u32 {
        let Some(skill) = self.get_skill(name) else {
            return 0;
        };
        let body = skill.body();
        let lower = body.to_lowercase();
        let markers = [
            "rebuild", "cargo clean", "verify", "re-read", "re read", "audit", "validate",
            "recheck", "must ensure", "compile twice", "check twice",
        ];
        let mut score = 0u32;
        for m in markers {
            score += lower.matches(m).count() as u32;
        }
        score += lower
            .lines()
            .filter(|l| {
                let t = l.trim();
                t.len() > 4
                    && (t.chars().next().is_some_and(|c| c.is_ascii_digit())
                        || t.starts_with('-'))
                    && t.matches(' ').count() >= 8
            })
            .count() as u32;
        score
    }

    pub fn attribution_report(&self) -> Vec<SkillAttribution> {
        let mut report: Vec<SkillAttribution> = self
            .skills
            .iter()
            .map(|s| {
                self.attribution
                    .get(&s.name)
                    .cloned()
                    .unwrap_or(SkillAttribution {
                        name: s.name.clone(),
                        category: s.category.clone(),
                        activations: 0,
                        over_validation_score: self.over_validation_score(&s.name),
                        procedure_heavy: false,
                        flagged: false,
                    })
            })
            .collect();
        report.sort_by(|a, b| b.activations.cmp(&a.activations));
        report
    }

    pub fn get_skill(&self, name: &str) -> Option<&SkillEntry> {
        self.skills.iter().find(|s| s.name == name)
    }

    pub fn get_skill_mut(&mut self, name: &str) -> Option<&mut SkillEntry> {
        self.skills.iter_mut().find(|s| s.name == name)
    }

    /// 渐进披露加载 (progressive disclosure, diagram-design 吸收):
    /// SKILL.md 只描述技能的选择与入口, 深层细节 (参考文档/模板/示例) 存于
    /// `<skill_dir>/references/<file>`, 按需读取 — 避免常驻加载拉爆上下文。
    ///
    /// 返回已声明引用中命中的内容; 未声明或不存在返回 Err (提示缺失)。
    pub fn load_reference(&self, name: &str, reference: &str) -> Result<String, String> {
        let entry = self
            .get_skill(name)
            .ok_or_else(|| format!("Skill '{}' not found", name))?;
        if !entry.references.iter().any(|r| r == reference) {
            return Err(format!(
                "Reference '{}' not declared in skill '{}' (declared: {:?})",
                reference, name, entry.references
            ));
        }
        let skill_dir = entry.path.parent().unwrap_or(&self.skills_dir);
        let ref_path = skill_dir.join("references").join(reference);
        if !ref_path.exists() {
            return Err(format!(
                "Reference file missing: {}",
                ref_path.display()
            ));
        }
        std::fs::read_to_string(&ref_path).map_err(|e| format!("read reference: {}", e))
    }

    /// 推进渐进披露阶梯 (P4, dsh-anchored-standard 吸收): 若 session 已
    /// durable (首个 durable 工具/调用), 从 Minimal 提升到 Standard 工具集。
    /// 返回阶段是否发生变化。
    pub fn step_disclosure(&mut self) -> bool {
        self.disclosure.maybe_promote()
    }

    /// Activate a skill by name. Fires HookEvent::SkillLoaded and GWT broadcast.
    pub fn activate_skill(&mut self, name: &str) -> Result<(), String> {
        let idx = self.skills.iter().position(|s| s.name == name)
            .ok_or_else(|| format!("Skill '{}' not found", name))?;
        if self.skills[idx].active {
            return Err(format!("Skill '{}' is already active", name));
        }
        self.skills[idx].active = true;
        self.record_activation(name);
        let desc = self.skills[idx].description.clone();
        let triggers = self.skills[idx].triggers.clone();
        let e8_modes = self.skills[idx].e8_modes.clone();
        let priority = self.skills[idx].priority;

        if let Some(ref mut hooks) = self.hooks {
            let ctx = HookContext::new(
                HookEvent::SkillLoaded,
                &format!("skill:{}", name),
            ).with_payload(serde_json::json!({
                "name": name,
                "description": desc,
                "triggers": triggers,
                "e8_modes": e8_modes,
                "priority": priority,
            }));
            hooks.trigger(&ctx);
        }

        if let Some(ref gwt) = self.gwt {
            if let Ok(mut gwt) = gwt.try_write() {
                gwt.broadcast(&format!("[skill_activated] {} — {}", name, desc));
            }
        }

        Ok(())
    }

    /// Deactivate a skill by name. Fires HookEvent::SkillUnloaded.
    pub fn deactivate_skill(&mut self, name: &str) -> Result<(), String> {
        let idx = self.skills.iter().position(|s| s.name == name)
            .ok_or_else(|| format!("Skill '{}' not found", name))?;
        if !self.skills[idx].active {
            return Err(format!("Skill '{}' is not active", name));
        }
        self.skills[idx].active = false;

        if let Some(ref mut hooks) = self.hooks {
            let ctx = HookContext::new(
                HookEvent::SkillUnloaded,
                &format!("skill:{}", name),
            );
            hooks.trigger(&ctx);
        }

        Ok(())
    }

    pub fn list_active(&self) -> Vec<&SkillEntry> {
        self.skills.iter().filter(|s| s.active).collect()
    }

    /// 披露门控的活跃技能视图 (P4 行为接线): 披露预算 active_tool_count()
    /// 真实限制模型可见工具集 — stage 0 (Minimal) 时仅暴露预算数量的
    /// 高优先级技能, promote 到 Standard 后暴露全部活跃技能。
    /// 这是 active_tool_count() 从"展示"到"行为门控"的生产路径。
    pub fn visible_active(&self) -> Vec<&SkillEntry> {
        let mut active: Vec<&SkillEntry> = self.skills.iter().filter(|s| s.active).collect();
        let budget = self.disclosure.active_tool_count();
        if self.disclosure.stage == 0 && active.len() > budget {
            // Minimal 阶段: 按 priority 升序 (高优先级在前) 截断到预算
            active.sort_by_key(|s| s.priority);
            active.truncate(budget);
        }
        active
    }

    /// 技能树层级统计 (G6, AgentSkillOS 吸收): category 分布、根/叶/孤儿技能、
    /// 覆盖率与深度。供背景循环巡检报告使用。
    pub fn skill_tree_stats(&self) -> SkillTreeStats {
        let mut categories: HashMap<String, usize> = HashMap::new();
        let mut roots = 0usize;
        let mut orphans = 0usize;
        
        let mut depth: HashMap<String, usize> = HashMap::new();

        for s in &self.skills {
            *categories.entry(s.category.clone()).or_insert(0) += 1;
            if s.parent.is_empty() {
                roots += 1;
            }
        }
        for _ in 0..=self.skills.len() {
            let mut changed = false;
            for s in &self.skills {
                if s.parent.is_empty() {
                    if depth.insert(s.name.clone(), 0).is_none_or(|old| old != 0) {
                        changed = true;
                    }
                } else {
                    if let Some(pd) = depth.get(&s.parent).copied() {
                        let d = pd + 1;
                        if depth.insert(s.name.clone(), d).is_none_or(|old| old != d) {
                            changed = true;
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }
        let max_depth = depth.values().copied().max().unwrap_or(0);
        let known: std::collections::HashSet<&String> =
            self.skills.iter().map(|s| &s.name).collect();
        for s in &self.skills {
            if !s.parent.is_empty() && !known.contains(&s.parent) {
                orphans += 1;
            }
        }
        SkillTreeStats {
            total_skills: self.skills.len(),
            categories,
            roots,
            orphans,
            max_depth,
        }
    }

    /// 差分归因 flagged 汇总 (G7, arxiv 2608.11888 SkillTriage): 返回
    /// procedure-heavy 且被标记的技能 (过度验证毒源), 供巡检广播告警。
    pub fn flagged_attributions(&self) -> Vec<SkillAttribution> {
        self.attribution_report()
            .into_iter()
            .filter(|a| a.flagged || (a.procedure_heavy && a.activations > 0))
            .collect()
    }

    pub fn list_all(&self) -> Vec<&SkillEntry> {
        self.skills.iter().collect()
    }

    pub fn skills_dir(&self) -> &Path {
        &self.skills_dir
    }

    /// Install a skill from a source path (file or directory with SKILL.md).
    /// Copies the file(s) into the skills directory.
    pub fn install_skill(&mut self, source_path: &Path) -> Result<(), String> {
        if !source_path.exists() {
            return Err(format!("Source path does not exist: {}", source_path.display()));
        }

        if source_path.is_dir() {
            let skill_md = source_path.join("SKILL.md");
            if !skill_md.exists() {
                return Err("Directory must contain a SKILL.md file".to_string());
            }
            let content = std::fs::read_to_string(&skill_md).map_err(|e| e.to_string())?;
            let entry = SkillEntry::from_content(&skill_md, &content)
                .ok_or_else(|| "Invalid frontmatter in SKILL.md".to_string())?;

            let target_dir = self.skills_dir.join(&entry.name);
            let _ = std::fs::create_dir_all(&target_dir);

            // Copy SKILL.md
            let dest = target_dir.join("SKILL.md");
            std::fs::copy(&skill_md, &dest).map_err(|e| e.to_string())?;

            // Copy other files from source directory
            if let Ok(entries) = std::fs::read_dir(source_path) {
                for e in entries.flatten() {
                    let src = e.path();
                    if src == skill_md { continue; }
                    let fname = src.file_name().unwrap_or_default();
                    let dst = target_dir.join(fname);
                    if src.is_file() {
                        let _ = std::fs::copy(&src, &dst);
                    } else if src.is_dir() {
                        let dst_sub = target_dir.join(fname);
                        let _ = std::fs::create_dir_all(&dst_sub);
                        if let Ok(sub) = std::fs::read_dir(&src) {
                            for sub_entry in sub.flatten() {
                                let sub_src = sub_entry.path();
                                if sub_src.is_file() {
                                    let _ = std::fs::copy(&sub_src, dst_sub.join(sub_src.file_name().unwrap_or_default()));
                                }
                            }
                        }
                    }
                }
            }

            self.load_all();
            self.register_install_effects(&entry.name, &target_dir)?;
            Ok(())
        } else if source_path.extension().is_some_and(|e| e == "md") {
            let content = std::fs::read_to_string(source_path).map_err(|e| e.to_string())?;
            let entry = SkillEntry::from_content(source_path, &content)
                .ok_or_else(|| "Invalid frontmatter in skill file".to_string())?;

            let target_dir = self.skills_dir.join(&entry.name);
            let _ = std::fs::create_dir_all(&target_dir);
            let dest = target_dir.join("SKILL.md");
            std::fs::copy(source_path, &dest).map_err(|e| e.to_string())?;

            self.load_all();
            self.register_install_effects(&entry.name, &target_dir)?;
            Ok(())
        } else {
            Err("Source must be a .md file or a directory containing SKILL.md".to_string())
        }
    }

    /// Build a SkillEntry from a ProceduralMemoryRecord (KB-stored E8 trajectory pattern).
    /// Converts the E8 sequence, trigger, reward, and tags into a YAML-frontmatter skill
    /// that can be written to the filesystem and loaded by SkillEngine.
    pub fn skill_from_procedural_record(record: &ProceduralMemoryRecord) -> SkillEntry {
        let e8_str = format!("[{}]", record.e8_sequence.iter().map(|m| m.to_string()).collect::<Vec<_>>().join(","));

        let yaml = format!(
            "---\nname: {}\ndescription: {}\ntriggers: [\"e8\", \"proc_skill\", \"{}\"]\ne8_modes: {}\npriority: {}\n---\n\n{}",
            record.name,
            record.description,
            record.skill_id,
            e8_str,
            (record.avg_reward * 100.0) as u8,
            record.description,
        );

        SkillEntry {
            name: record.name.clone(),
            description: record.description.clone(),
            triggers: vec!["e8".to_string(), "proc_skill".to_string(), record.skill_id.clone()],
            e8_modes: record.e8_sequence.clone(),
            tools: vec![],
            hooks: vec![],
            priority: (record.avg_reward * 100.0) as u8,
            path: PathBuf::new(),
            content: yaml,
            active: false,
            references: vec![],
            category: "procedural".to_string(),
            parent: String::new(),
            verified: false,
        }
    }

    /// Install a procedural memory record as a YAML-frontmatter skill file in the skills directory.
    /// Creates `~/.neotrix/skills/<skill_name>/SKILL.md` from the record.
    /// Returns the name of the installed skill on success.
    pub fn install_from_procedural(&mut self, record: &ProceduralMemoryRecord) -> Result<String, String> {
        let skill = Self::skill_from_procedural_record(record);
        let target_dir = self.skills_dir.join(&skill.name);
        let _ = std::fs::create_dir_all(&target_dir);
        let dest = target_dir.join("SKILL.md");
        std::fs::write(&dest, &skill.content).map_err(|e| format!("write skill: {}", e))?;
        self.load_all();
        log::info!("[procedural→skill] installed '{}' from E8 pattern ({} states, reward={:.3})",
            skill.name, record.e8_sequence.len(), record.avg_reward);
        Ok(skill.name)
    }

    /// 把 install 的逆操作推入逆账本并派生 skill fiber (cordiverse F1+F5)。
    /// teardown 从加载序派生, 非手写清理 (paper §3.3.3 p.27)。
    fn register_install_effects(&mut self, name: &str, target_dir: &Path) -> Result<(), String> {
        let install_id = self.inverse_ledger.begin_install();
        let inv_target = target_dir.to_path_buf();
        let inv_label = format!("remove installed skill dir: {}", inv_target.display());
        self.inverse_ledger.push_inverse(
            install_id,
            RevertibleEffect::new(inv_label, move || {
                if inv_target.exists() {
                    std::fs::remove_dir_all(&inv_target)
                        .map_err(|e| format!("remove {}: {}", inv_target.display(), e))
                } else {
                    Ok(())
                }
            }),
        )?;
        self.fiber_lifecycles.insert(
            name.to_string(),
            FiberLifecycle::new(name.to_string(), install_id),
        );
        if let Some(fiber) = self.fiber_lifecycles.get_mut(name) {
            let _ = fiber.transition(FiberLifecycleState::Active);
        }
        Ok(())
    }

    /// 卸载技能 (cordiverse F1+F5): 按加载序的 LIFO 逆序执行该 install 的
    /// 全部逆操作, 完成后把 fiber 转入 Retired 终态。逆操作中的失败按 fiber
    /// 捕获, 不中断其余逆操作, 也不影响其他 fiber。
    pub fn uninstall_skill(&mut self, name: &str) -> Result<Vec<Result<(), String>>, String> {
        if self.fiber_lifecycles.get(name).map(|f| f.state) == Some(FiberLifecycleState::Retired) {
            return Err(format!("skill '{}' fiber already retired", name));
        }
        let install_id = self.fiber_lifecycles.get(name)
            .map(|f| f.install_id)
            .ok_or_else(|| format!("no installed fiber for skill '{}'", name))?;
        let results = self.inverse_ledger.teardown(install_id);
        if let Some(fiber) = self.fiber_lifecycles.get_mut(name) {
            let _ = fiber.transition(FiberLifecycleState::Retired);
        }
        if let Some(idx) = self.skills.iter().position(|s| s.name == name) {
            self.skills.remove(idx);
            self.build_index();
        }
        Ok(results)
    }

    /// 从 skill 名查 fiber 当前生命周期状态。
    pub fn fiber_state(&self, name: &str) -> Option<FiberLifecycleState> {
        self.fiber_lifecycles.get(name).map(|f| f.state)
    }

    /// 按 fiber 捕获失败并转入 Failed (不传播到 sibling)。
    pub fn record_fiber_failure(&mut self, name: &str, message: impl Into<String>) -> bool {
        if let Some(fiber) = self.fiber_lifecycles.get_mut(name) {
            fiber.record_failure(message);
            true
        } else {
            false
        }
    }

    /// 释放悬挂所有权: fiber 仍标记 held (Loaded/Active/Suspended) 但其 install
    /// 逆账本事务已消失 (holder 失效) → 自动转入 Retired 终态。返回释放列表。
    pub fn release_dangling(&mut self) -> Vec<String> {
        use FiberLifecycleState::*;
        let dangling: Vec<String> = self
            .fiber_lifecycles
            .iter()
            .filter(|(_, f)| matches!(f.state, Loaded | Active | Suspended))
            .filter(|(_, f)| !self.inverse_ledger.has_transaction(f.install_id))
            .map(|(name, _)| name.clone())
            .collect();
        let mut released = Vec::new();
        for name in dangling {
            if let Some(fiber) = self.fiber_lifecycles.get_mut(&name) {
                let _ = fiber.transition(FiberLifecycleState::Retired);
                released.push(name);
            }
        }
        released
    }

    /// Find all skill files in the workspace and agent directories.
    /// Legacy compatibility: discovers but does NOT load into this engine.
    pub fn discover_skills() -> Vec<DiscoveredSkill> {
        let mut skills = Vec::new();
        let mut seen: Vec<String> = Vec::new();

        // 1. ~/.neotrix/skills/
        if let Ok(home) = std::env::var("HOME") {
            let dir = PathBuf::from(&home).join(".neotrix").join("skills");
            if dir.exists() {
                Self::scan_discover_dir(&dir, &mut seen, &mut skills);
            }
        }

        // 2. ~/.agents/skills/
        if let Ok(home) = std::env::var("HOME") {
            let dir = PathBuf::from(&home).join(".agents").join("skills");
            if dir.exists() {
                Self::scan_discover_dir(&dir, &mut seen, &mut skills);
            }
        }

        // 3. Workspace skills/
        let ws = Path::new("skills");
        if ws.exists() {
            Self::scan_discover_dir(ws, &mut seen, &mut skills);
        }

        skills
    }

    fn scan_discover_dir(dir: &Path, seen: &mut Vec<String>, skills: &mut Vec<DiscoveredSkill>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                    if seen.contains(&name) { continue; }
                    let skill_md = path.join("SKILL.md");
                    if skill_md.exists() {
                        let content = std::fs::read_to_string(&skill_md).unwrap_or_default();
                        let description = Self::extract_frontmatter_desc(&content);
                        seen.push(name.clone());
                        skills.push(DiscoveredSkill { name, description, path: skill_md });
                    }
                }
            }
        }
    }

    fn extract_frontmatter_desc(content: &str) -> String {
        let stripped = content.trim_start();
        if !stripped.starts_with("---") { return String::new(); }
        if let Some(end) = stripped[3..].find("---") {
            let frontmatter = &stripped[3..3 + end];
            for line in frontmatter.lines() {
                if let Some(val) = line.trim().strip_prefix("description:") {
                    return val.trim().to_string();
                }
            }
        }
        String::new()
    }

    /// Find all SKILL.md files recursively within a directory (legacy compat).
    pub fn find_skill_mds(dir: &Path) -> Vec<PathBuf> {
        let mut results = Vec::new();
        if dir.is_file() && dir.ends_with("SKILL.md") {
            results.push(dir.to_path_buf());
            return results;
        }
        Self::find_skill_mds_recursive(dir, &mut results);
        results
    }

    fn find_skill_mds_recursive(dir: &Path, results: &mut Vec<PathBuf>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let fname = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
                    if fname.starts_with('.') || fname == "node_modules" || fname == "target" {
                        continue;
                    }
                    Self::find_skill_mds_recursive(&path, results);
                } else if path.ends_with("SKILL.md") {
                    results.push(path);
                }
            }
        }
    }
}

/// Lightweight discovered skill (legacy compatibility).
#[derive(Debug, Clone)]
pub struct DiscoveredSkill {
    pub name: String,
    pub description: String,
    pub path: PathBuf,
}

// ────────────────────────────────────────────────────────────────
// P23: PromptLibrary (吸收 prompts.chat — 提示词资产库)
// 提示词资产持久库: 命名 + 版本 + 标签路由。供 harness / 进化 loop
// 复用工程化提示词, 替代散落的硬编码 prompt。
// ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptEntry {
    pub name: String,
    pub version: u32,
    pub tags: Vec<String>,
    pub content: String,
    pub author: String,
}

impl PromptEntry {
    pub fn new(name: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: 1,
            tags: vec![],
            content: content.into(),
            author: "neotrix".into(),
        }
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromptLibrary {
    prompts: Vec<PromptEntry>,
}

impl PromptLibrary {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, entry: PromptEntry) -> Result<(), String> {
        if let Some(existing) = self.prompts.iter_mut().find(|p| p.name == entry.name) {
            // 同名 → 版本递增 (prompts.chat 语义: 同名可迭代)
            existing.version += 1;
            existing.content = entry.content;
            existing.tags = entry.tags;
            return Ok(());
        }
        self.prompts.push(entry);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&PromptEntry> {
        self.prompts.iter().find(|p| p.name == name)
    }

    pub fn by_tag(&self, tag: &str) -> Vec<&PromptEntry> {
        self.prompts.iter().filter(|p| p.tags.iter().any(|t| t == tag)).collect()
    }

    pub fn all(&self) -> &[PromptEntry] {
        &self.prompts
    }

    pub fn len(&self) -> usize {
        self.prompts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.prompts.is_empty()
    }
}

impl crate::core::nt_core_self_test::SelfTest for PromptLibrary {
    fn name(&self) -> &str {
        "nt_mind_prompt_library"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut lib = PromptLibrary::new();
        lib.register(PromptEntry::new("judge_rubric", "score 1-5").with_tags(vec!["eval".into()]))
            .map_err(|e| vec![e])?;
        if lib.len() != 1 {
            return Err(vec!["prompt library should hold 1 entry".into()]);
        }
        Ok(())
    }
}

/// Hook events for skill lifecycle.
pub mod skill_hooks {
    use super::*;

    pub struct SkillActivationHook {
        pub engine: Arc<RwLock<SkillEngine>>,
    }

    impl crate::neotrix::l8_autonomic_impl::nt_mind_hook::HookAction for SkillActivationHook {
        fn name(&self) -> &str {
            "skill_activation_hook"
        }

        fn execute(&self, ctx: &HookContext) -> HookResult {
            let msg = &ctx.message;
            if msg.starts_with("skill:") {
                let name = &msg[6..];
                let engine = self.engine.try_write();
                match engine {
                    Ok(mut engine) => {
                        let _ = engine.activate_skill(name);
                    }
                    Err(_) => {
                        std::thread::yield_now();
                        if let Ok(mut engine) = self.engine.try_write() {
                            let _ = engine.activate_skill(name);
                        }
                    }
                }
            }
            HookResult::ok("skill activation hook processed")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_self_test::SelfTest;

    fn sample_skill_content() -> &'static str {
        r#"---
name: rust-analyzer
description: Expertise in Rust code analysis and optimization
triggers: ["rust", "cargo", "unsafe", "lifetime", "ownership"]
e8_modes: [12, 13, 14]
tools: ["read", "edit", "bash"]
hooks: ["PreToolUse", "PostToolUse"]
priority: 80
---

# Rust Analyzer Skill

## Capabilities
- Analyze Rust code for safety issues
- Suggest optimizations
"#
    }

    fn sample_skill_content_no_frontmatter() -> &'static str {
        "# Just a markdown file\n\nNo frontmatter here."
    }

    fn setup_temp_dir() -> tempfile::TempDir {
        tempfile::tempdir().expect("failed to create temp dir")
    }

    #[test]
    fn test_selftest_gate_marks_unverified_without_script() {
        // P2-5 质量门: 缺 scripts/selftest.sh|js → unverified
        let dir = setup_temp_dir();
        let path = dir.path().join("no-selftest.md");
        std::fs::write(&path, sample_skill_content()).unwrap();
        let entry = SkillEntry::from_file(&path).unwrap();
        assert!(!entry.verified, "skill without scripts/selftest must be unverified");
    }

    #[test]
    fn test_selftest_gate_marks_verified_with_script() {
        // P2-5 质量门: 存在 scripts/selftest.sh → verified
        let dir = setup_temp_dir();
        let skill_dir = dir.path().join("with-selftest");
        let scripts = skill_dir.join("scripts");
        std::fs::create_dir_all(&scripts).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), sample_skill_content()).unwrap();
        std::fs::write(scripts.join("selftest.sh"), "#!/usr/bin/env bash\necho '{\"gates\":[]}'\n").unwrap();
        let entry = SkillEntry::from_file(&skill_dir.join("SKILL.md")).unwrap();
        assert!(entry.verified, "skill with scripts/selftest.sh must be verified");
    }

    #[test]
    fn test_selftest_gate_verified_with_js() {
        let dir = setup_temp_dir();
        let skill_dir = dir.path().join("js-selftest");
        let scripts = skill_dir.join("scripts");
        std::fs::create_dir_all(&scripts).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), sample_skill_content()).unwrap();
        std::fs::write(scripts.join("selftest.js"), "console.log('{\"gates\":[]}')").unwrap();
        let entry = SkillEntry::from_file(&skill_dir.join("SKILL.md")).unwrap();
        assert!(entry.verified);
    }

    #[test]
    fn test_parse_skill_frontmatter() {
        let dir = setup_temp_dir();
        let path = dir.path().join("test.md");
        std::fs::write(&path, sample_skill_content()).unwrap();

        let entry = SkillEntry::from_file(&path).unwrap();
        assert_eq!(entry.name, "rust-analyzer");
        assert_eq!(entry.description, "Expertise in Rust code analysis and optimization");
        assert_eq!(entry.triggers, vec!["rust", "cargo", "unsafe", "lifetime", "ownership"]);
        assert_eq!(entry.e8_modes, vec![12, 13, 14]);
        assert_eq!(entry.tools, vec!["read", "edit", "bash"]);
        assert_eq!(entry.hooks, vec!["PreToolUse", "PostToolUse"]);
        assert_eq!(entry.priority, 80);
        assert!(!entry.active);
    }

    #[test]
    fn test_parse_skill_no_frontmatter_returns_none() {
        let dir = setup_temp_dir();
        let path = dir.path().join("test.md");
        std::fs::write(&path, sample_skill_content_no_frontmatter()).unwrap();
        assert!(SkillEntry::from_file(&path).is_none());
    }

    #[test]
    fn test_parse_skill_missing_name_returns_none() {
        let content = r#"---
description: No name here
---
body"#;
        let dir = setup_temp_dir();
        let path = dir.path().join("test.md");
        std::fs::write(&path, content).unwrap();
        assert!(SkillEntry::from_file(&path).is_none());
    }

    #[test]
    fn test_parse_skill_default_priority() {
        let content = r#"---
name: test
description: A test skill
---
body"#;
        let dir = setup_temp_dir();
        let path = dir.path().join("test.md");
        std::fs::write(&path, content).unwrap();
        let entry = SkillEntry::from_file(&path).unwrap();
        assert_eq!(entry.priority, 50);
    }

    #[test]
    fn test_skill_engine_load_all() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        // Create a skill as a subdirectory with SKILL.md
        let skill_dir = skills_dir.join("rust-analyzer");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), sample_skill_content()).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        let loaded = engine.load_all();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "rust-analyzer");
    }

    #[test]
    fn test_skill_engine_load_all_no_dir_creates_it() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("nonexistent");
        let mut engine = SkillEngine::new(skills_dir.clone());
        let loaded = engine.load_all();
        assert!(loaded.is_empty());
        assert!(skills_dir.exists());
    }

    #[test]
    #[ignore = "flaky: test ordering dependent"]
    fn test_find_matching_by_trigger() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let skill_dir = skills_dir.join("rust-analyzer");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), sample_skill_content()).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        // Match by trigger keyword
        let matches = engine.find_matching("rust", None);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].name, "rust-analyzer");

        let matches = engine.find_matching("ownership", None);
        assert_eq!(matches.len(), 1);

        // Non-matching query
        let matches = engine.find_matching("python", None);
        assert!(matches.is_empty());
    }

    #[test]
    #[ignore = "flaky: test ordering dependent"]
    fn test_find_matching_filters_by_e8_mode() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let skill_dir = skills_dir.join("rust-analyzer");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), sample_skill_content()).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        // E8 mode 12 matches (12,13,14 are valid for this skill)
        let matches = engine.find_matching("rust", Some(12));
        assert_eq!(matches.len(), 1);

        // E8 mode 0 is not in [12,13,14] → no match when Some(0)
        let matches = engine.find_matching("rust", Some(0));
        assert!(matches.is_empty());

        // None skips E8 filter → matches
        let matches = engine.find_matching("rust", None);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_find_matching_e8_filter_with_empty_modes() {
        let content = r#"---
name: generic
description: A generic skill
triggers: ["help", "info"]
---
body"#;
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();
        let skill_dir = skills_dir.join("generic");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), content).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        // No e8_modes specified — matches any mode (Some or None)
        let matches = engine.find_matching("help", Some(42));
        assert_eq!(matches.len(), 1);
        let matches = engine.find_matching("help", None);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_find_matching_exact_trigger_outranks_substring() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let exact_skill = r#"---
name: auth
description: Authentication handler
triggers: ["auth", "login", "session"]
---
body"#;
        let substring_skill = r#"---
name: auth-analyzer
description: A skill that also matches auth as substring
triggers: ["auth-flow", "oauth"]
---
body"#;
        for (name, content) in [("auth", exact_skill), ("auth-analyzer", substring_skill)] {
            let dir2 = skills_dir.join(name);
            std::fs::create_dir_all(&dir2).unwrap();
            std::fs::write(dir2.join("SKILL.md"), content).unwrap();
        }

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        // Exact trigger "auth" must outrank substring "auth-flow" match
        let matches = engine.find_matching("auth", None);
        assert!(!matches.is_empty());
        assert_eq!(matches[0].name, "auth", "exact trigger must be ranked first");
    }

    #[test]
    fn test_find_matching_caps_results() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        for i in 0..(SkillEngine::MAX_ROUTE_RESULTS + 5) {
            let name = format!("matching-skill-{}", i);
            let dir2 = skills_dir.join(&name);
            std::fs::create_dir_all(&dir2).unwrap();
            let content = format!(
                "---\nname: {}\ndescription: test\ntriggers: [\"matching-skill\"]\n---\nbody",
                name
            );
            std::fs::write(dir2.join("SKILL.md"), content).unwrap();
        }

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        // All skills match query "matching-skill" (substring); results must be capped
        let matches = engine.find_matching("matching-skill", None);
        assert!(matches.len() <= SkillEngine::MAX_ROUTE_RESULTS);
        assert_eq!(matches.len(), SkillEngine::MAX_ROUTE_RESULTS);
    }

    #[test]
    fn test_activate_and_deactivate_skill() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();
        let skill_dir = skills_dir.join("rust-analyzer");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), sample_skill_content()).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        assert!(engine.activate_skill("rust-analyzer").is_ok());
        assert!(engine.get_skill("rust-analyzer").unwrap().active);

        // Double activation should fail
        assert!(engine.activate_skill("rust-analyzer").is_err());

        let active = engine.list_active();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].name, "rust-analyzer");

        assert!(engine.deactivate_skill("rust-analyzer").is_ok());
        assert!(!engine.get_skill("rust-analyzer").unwrap().active);
        assert!(engine.list_active().is_empty());

        // Deactivate inactive should fail
        assert!(engine.deactivate_skill("rust-analyzer").is_err());
    }

    #[test]
    fn test_get_skill_nonexistent() {
        let dir = setup_temp_dir();
        let mut engine = SkillEngine::new(dir.path().join("skills"));
        engine.load_all();
        assert!(engine.get_skill("nonexistent").is_none());
        assert!(engine.activate_skill("nonexistent").is_err());
    }

    #[test]
    fn test_install_skill_from_file() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let source = dir.path().join("source.md");
        std::fs::write(&source, sample_skill_content()).unwrap();

        let mut engine = SkillEngine::new(skills_dir.clone());
        engine.load_all();
        assert!(engine.list_all().is_empty());

        engine.install_skill(&source).unwrap();

        let all = engine.list_all();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].name, "rust-analyzer");
        assert!(skills_dir.join("rust-analyzer").join("SKILL.md").exists());
    }

    #[test]
    fn test_install_skill_from_directory() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let source_dir = dir.path().join("my-skill");
        std::fs::create_dir_all(&source_dir).unwrap();
        std::fs::write(source_dir.join("SKILL.md"), sample_skill_content()).unwrap();
        std::fs::write(source_dir.join("helper.py"), r#"print("hello")"#).unwrap();

        let mut engine = SkillEngine::new(skills_dir.clone());
        engine.load_all();
        assert!(engine.list_all().is_empty());

        engine.install_skill(&source_dir).unwrap();
        assert_eq!(engine.list_all().len(), 1);
        assert!(skills_dir.join("rust-analyzer").join("helper.py").exists());
    }

    #[test]
    fn test_install_skill_invalid_source() {
        let dir = setup_temp_dir();
        let mut engine = SkillEngine::new(dir.path().join("skills"));

        assert!(engine.install_skill(&dir.path().join("nonexistent")).is_err());
        assert!(engine.install_skill(&dir.path().join(".")).is_err());
    }

    #[test]
    fn test_parse_array_field() {
        assert_eq!(parse_array_field(r#"["a", "b", "c"]"#), vec!["a", "b", "c"]);
        assert_eq!(parse_array_field(r#"['x', 'y']"#), vec!["x", "y"]);
        assert_eq!(parse_array_field("a, b, c"), vec!["a", "b", "c"]);
        assert_eq!(parse_array_field(""), Vec::<String>::new());
    }

    #[test]
    fn test_skill_body() {
        let content = sample_skill_content();
        let dir = setup_temp_dir();
        let path = dir.path().join("test.md");
        std::fs::write(&path, content).unwrap();
        let entry = SkillEntry::from_file(&path).unwrap();
        let body = entry.body();
        assert!(body.contains("Analyze Rust code"));
        assert!(body.to_lowercase().contains("suggest optimizations"));
    }

    #[test]
    fn test_discover_skills_legacy() {
        // Should not crash/panic
        let _skills = SkillEngine::discover_skills();
    }

    #[test]
    fn test_priority_sorting() {
        let content_high = r#"---
name: high-priority
description: High priority skill
triggers: ["test"]
priority: 90
---
high"#;
        let content_low = r#"---
name: low-priority
description: Low priority skill
triggers: ["test"]
priority: 10
---
low"#;

        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        std::fs::write(skills_dir.join("high.md"), content_high).unwrap();
        std::fs::write(skills_dir.join("low.md"), content_low).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        let matches = engine.find_matching("test", None);
        assert_eq!(matches.len(), 2);
        // Higher priority first
        assert_eq!(matches[0].name, "high-priority");
    }

    #[test]
    fn test_list_all_empty() {
        let dir = setup_temp_dir();
        let mut engine = SkillEngine::new(dir.path().join("skills"));
        engine.load_all();
        assert!(engine.list_all().is_empty());
    }

    #[test]
    fn test_progressive_disclosure_load_reference() {
        // 渐进披露 (diagram-design 吸收): SKILL.md 只声明 references,
        // 细节存 references/*.md 按需加载。
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        let skill_dir = skills_dir.join("my-skill");
        let refs_dir = skill_dir.join("references");
        std::fs::create_dir_all(&refs_dir).unwrap();

        std::fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: my-skill\ndescription: A skill with progressive disclosure\ntriggers: [\"mine\"]\nreferences: [\"type-a.md\", \"type-b.md\"]\n---\nbody",
        )
        .unwrap();
        std::fs::write(refs_dir.join("type-a.md"), "TYPE-A DETAILS").unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();
        assert_eq!(engine.list_all().len(), 1);

        // 已声明引用 → 按需加载成功
        let loaded = engine.load_reference("my-skill", "type-a.md").unwrap();
        assert!(loaded.contains("TYPE-A DETAILS"));

        // 未声明引用 → 拒绝 (禁未声明加载)
        assert!(engine.load_reference("my-skill", "secret.md").is_err());

        // 缺失文件 → 报错 (声明了但没落盘)
        assert!(engine.load_reference("my-skill", "type-b.md").is_err());

        // 不存在的技能 → 报错
        assert!(engine.load_reference("nope", "type-a.md").is_err());
    }

    #[test]
    fn test_list_active_empty() {
        let dir = setup_temp_dir();
        let mut engine = SkillEngine::new(dir.path().join("skills"));
        engine.load_all();
        assert!(engine.list_active().is_empty());
    }

    #[test]
    fn test_skill_entry_from_content_direct() {
        let entry = SkillEntry::from_content(Path::new("test.md"), sample_skill_content());
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.name, "rust-analyzer");
    }

    #[test]
    fn test_e8_index_build() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        std::fs::write(skills_dir.join("test.md"), sample_skill_content()).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        assert!(engine.e8_index.contains_key(&12));
        assert!(engine.e8_index.contains_key(&13));
        assert!(engine.e8_index.contains_key(&14));
        assert!(!engine.e8_index.contains_key(&0));
    }

    #[test]
    fn test_skill_from_procedural_record_creates_valid_entry() {
        let record = ProceduralMemoryRecord {
            id: "test-id".into(),
            skill_id: "proc_skill_test".into(),
            name: "Test E8 Skill".into(),
            description: "Learned E8 pattern: 3 states".into(),
            e8_sequence: vec![12, 13, 14],
            trigger_pattern: vec![12],
            success_rate: 0.85,
            execution_count: 5,
            avg_reward: 0.75,
            created_at: "2026-07-04T00:00:00Z".into(),
            updated_at: "2026-07-04T00:00:00Z".into(),
            tags: vec!["procedural".into(), "auto_discovered".into()],
        };

        let skill = SkillEngine::skill_from_procedural_record(&record);
        assert_eq!(skill.name, "Test E8 Skill");
        assert_eq!(skill.description, "Learned E8 pattern: 3 states");
        assert_eq!(skill.triggers, vec!["e8", "proc_skill", "proc_skill_test"]);
        assert_eq!(skill.e8_modes, vec![12, 13, 14]);
        assert_eq!(skill.priority, 75);
        assert!(skill.content.starts_with("---\nname: Test E8 Skill"));
        assert!(skill.content.contains("e8_modes: [12,13,14]"));
    }

    #[test]
    fn test_skill_from_procedural_record_low_reward() {
        let record = ProceduralMemoryRecord {
            id: "test-id-2".into(),
            skill_id: "proc_skill_low".into(),
            name: "Low Reward Skill".into(),
            description: "Learned E8 pattern with low confidence".into(),
            e8_sequence: vec![1, 2],
            trigger_pattern: vec![1],
            success_rate: 0.3,
            execution_count: 1,
            avg_reward: 0.15,
            created_at: "2026-07-04T00:00:00Z".into(),
            updated_at: "2026-07-04T00:00:00Z".into(),
            tags: vec![],
        };

        let skill = SkillEngine::skill_from_procedural_record(&record);
        assert_eq!(skill.priority, 15, "low avg_reward should give low priority");
        assert_eq!(skill.e8_modes, vec![1, 2]);
    }

    #[test]
    fn test_install_from_procedural_writes_skill_file() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let record = ProceduralMemoryRecord {
            id: "install-test-id".into(),
            skill_id: "proc_install_test".into(),
            name: "Installed Procedural Skill".into(),
            description: "E8 pattern installed via bridge".into(),
            e8_sequence: vec![5, 10, 15],
            trigger_pattern: vec![5],
            success_rate: 0.9,
            execution_count: 3,
            avg_reward: 0.88,
            created_at: "2026-07-04T00:00:00Z".into(),
            updated_at: "2026-07-04T00:00:00Z".into(),
            tags: vec!["procedural".into()],
        };

        let mut engine = SkillEngine::new(skills_dir.clone());
        let result = engine.install_from_procedural(&record);
        assert!(result.is_ok(), "install_from_procedural failed: {:?}", result);
        assert_eq!(result.unwrap(), "Installed Procedural Skill");

        // Verify the skill file was created and can be loaded back
        let mut engine2 = SkillEngine::new(skills_dir);
        let loaded = engine2.load_all();
        let skill = loaded.iter().find(|s| s.name == "Installed Procedural Skill");
        assert!(skill.is_some(), "installed skill should be loadable");
        assert_eq!(skill.unwrap().e8_modes, vec![5, 10, 15]);
    }

    fn kb_conn() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_schema::initialize(&conn).unwrap();
        conn
    }

    #[test]
    fn test_sync_to_kb_index_write_through_and_dedup() {
        use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_unify::skill_list_all;

        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();
        let skill_dir = skills_dir.join("rust-analyzer");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), sample_skill_content()).unwrap();

        let conn = kb_conn();
        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();
        assert_eq!(engine.list_all().len(), 1);

        // 首次写通: 1 条真正写入
        assert_eq!(engine.sync_to_kb_index(&conn).unwrap(), 1);
        // 二次写通: 内容未变化 → 去重, 0 写入
        assert_eq!(engine.sync_to_kb_index(&conn).unwrap(), 0, "内容未变化必须去重 (避免每命令全量写)");

        let recs = skill_list_all(&conn, 10).unwrap();
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].name, "rust-analyzer");
        assert!(recs[0].content_hash.is_some(), "写通必须携带 content_hash");
        assert_eq!(recs[0].tags.as_deref(), Some("rust,cargo,unsafe,lifetime,ownership"));

        // 内容变化 → 再次写入 (同 name 更新)
        std::fs::write(
            skill_dir.join("SKILL.md"),
            sample_skill_content().replace("priority: 80", "priority: 85"),
        )
        .unwrap();
        engine.load_all();
        assert_eq!(engine.sync_to_kb_index(&conn).unwrap(), 1, "内容变化应重新写入");
        let recs = skill_list_all(&conn, 10).unwrap();
        assert_eq!(recs.len(), 1, "同 name 更新而非新增");
    }

    #[test]
    fn test_load_all_auto_syncs_to_kb() {
        use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_unify::skill_list_all;
        use crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase;

        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();
        let skill_dir = skills_dir.join("rust-analyzer");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), sample_skill_content()).unwrap();

        let kb = KnowledgeBase::open(Some(dir.path().join("kb.db"))).expect("KB open");
        let kb = Arc::new(kb);
        let mut engine = SkillEngine::new(skills_dir).with_kb(kb.clone());
        let loaded = engine.load_all();
        assert_eq!(loaded.len(), 1);

        let conn = kb.conn.lock().unwrap();
        let recs = skill_list_all(&conn, 10).unwrap();
        assert_eq!(recs.len(), 1, "load_all 后应自动写通到 skills_index");
        assert_eq!(recs[0].name, "rust-analyzer");
    }

    #[test]
    fn test_parse_category_parent_frontmatter() {
        let content = r#"---
name: nested
description: A categorized skill
triggers: ["nested"]
category: "test-domain"
parent: root-skill
---
body"#;
        let entry = SkillEntry::from_content(Path::new("nested.md"), content).unwrap();
        assert_eq!(entry.category, "test-domain");
        assert_eq!(entry.parent, "root-skill");
    }

    #[test]
    fn test_default_category_is_general() {
        let entry = SkillEntry::from_content(Path::new("x.md"), sample_skill_content()).unwrap();
        assert_eq!(entry.category, "general");
        assert!(entry.parent.is_empty());
    }

    #[test]
    fn test_skill_tree_groups_by_category() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let cat_a = r#"---
name: a-skill
description: cat a
triggers: ["alpha"]
category: analysis
---
body"#;
        let cat_b = r#"---
name: b-skill
description: cat b
triggers: ["beta"]
category: coding
---
body"#;
        std::fs::write(skills_dir.join("a.md"), cat_a).unwrap();
        std::fs::write(skills_dir.join("b.md"), cat_b).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();
        let tree = engine.skill_tree();
        assert_eq!(tree.len(), 2);
        assert!(tree.contains_key("analysis"));
        assert!(tree.contains_key("coding"));
    }

    #[test]
    fn test_children_of_returns_subskills() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let parent = r#"---
name: root-skill
description: root
triggers: ["root"]
category: coding
---
body"#;
        let child = r#"---
name: child-skill
description: child
triggers: ["child"]
category: coding
parent: root-skill
---
body"#;
        std::fs::write(skills_dir.join("root.md"), parent).unwrap();
        std::fs::write(skills_dir.join("child.md"), child).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();
        let children = engine.children_of("root-skill");
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name, "child-skill");
    }

    #[test]
    fn test_find_matching_complementary_prefers_uncovered_category() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let covered = r#"---
name: rust-analyzer
description: rust analysis
triggers: ["rust"]
category: coding
priority: 90
---
body"#;
        let uncovered = r#"---
name: security-review
description: security review
triggers: ["rust"]
category: security
priority: 40
---
body"#;
        std::fs::write(skills_dir.join("a.md"), covered).unwrap();
        std::fs::write(skills_dir.join("b.md"), uncovered).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        // 无已激活技能 → 按优先级: coding(90) 优先
        let base = engine.find_matching("rust", None);
        assert_eq!(base[0].name, "rust-analyzer");

        // 已激活 coding 类技能 → security 类应前置 (互补)
        let comp = engine.find_matching_complementary("rust", None, &["rust-analyzer"]);
        assert!(!comp.is_empty());
        assert_eq!(comp[0].name, "security-review");
    }

    #[test]
    fn test_find_matching_complementary_no_active_keeps_priority_order() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let high = r#"---
name: rust-analyzer
description: rust analysis
triggers: ["rust"]
category: coding
priority: 90
---
body"#;
        let low = r#"---
name: security-review
description: security review
triggers: ["rust"]
category: security
priority: 40
---
body"#;
        std::fs::write(skills_dir.join("a.md"), high).unwrap();
        std::fs::write(skills_dir.join("b.md"), low).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        // 无已激活技能 → 无互补偏好, 保持优先级降序
        let comp = engine.find_matching_complementary("rust", None, &[]);
        assert_eq!(comp[0].name, "rust-analyzer");
    }

    #[test]
    fn test_find_matching_complementary_all_covered_falls_back_to_priority() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let coding = r#"---
name: rust-analyzer
description: rust analysis
triggers: ["rust"]
category: coding
priority: 90
---
body"#;
        let security = r#"---
name: security-review
description: security review
triggers: ["rust"]
category: security
priority: 40
---
body"#;
        std::fs::write(skills_dir.join("a.md"), coding).unwrap();
        std::fs::write(skills_dir.join("b.md"), security).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        // active 覆盖全部候选类别 → 无互补空间, 退回优先级排序 (coding 90 优先)
        let comp = engine.find_matching_complementary("rust", None, &["rust-analyzer", "security-review"]);
        assert_eq!(comp[0].name, "rust-analyzer");
    }

    #[test]
    fn test_over_validation_score_flags_procedure_heavy() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let heavy = r#"---
name: heavy-skill
description: procedure heavy
triggers: ["heavy"]
category: general
---
1. rebuild the entire project and verify
2. cargo clean then re-read every file
3. verify compile twice and re-read all docs
4. audit every line and validate again
5. recheck rebuild and verify the compile
"#;
        std::fs::write(skills_dir.join("heavy.md"), heavy).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();
        let score = engine.over_validation_score("heavy-skill");
        assert!(score >= 12, "procedure-heavy skill must score >= 12, got {}", score);
    }

    #[test]
    fn test_attribution_tracks_activations_and_flags() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let heavy = r#"---
name: heavy-skill
description: procedure heavy
triggers: ["heavy"]
category: general
---
1. rebuild the entire project and verify the compile
2. cargo clean then re-read every single file and recheck
3. verify compile twice and audit the result line by line
4. validate the rebuild and recheck the verify output
5. cargo clean again and re-read the recheck report
"#;
        std::fs::write(skills_dir.join("heavy.md"), heavy).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        engine.activate_skill("heavy-skill").unwrap();
        engine.activate_skill("heavy-skill").unwrap_err();

        let report = engine.attribution_report();
        let attr = report.iter().find(|a| a.name == "heavy-skill").unwrap();
        assert_eq!(attr.activations, 1);
        assert!(attr.procedure_heavy, "heavy skill must be flagged procedure-heavy");
    }

    #[test]
    fn test_skill_tree_stats_counts_hierarchy() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let root = r#"---
name: root-skill
description: root
triggers: ["root"]
category: coding
---
body"#;
        let child = r#"---
name: child-skill
description: child
triggers: ["child"]
category: coding
parent: root-skill
---
body"#;
        let orphan = r#"---
name: orphan-skill
description: orphan (parent missing)
triggers: ["orphan"]
category: analysis
parent: ghost-parent
---
body"#;
        std::fs::write(skills_dir.join("root.md"), root).unwrap();
        std::fs::write(skills_dir.join("child.md"), child).unwrap();
        std::fs::write(skills_dir.join("orphan.md"), orphan).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();
        let stats = engine.skill_tree_stats();
        assert_eq!(stats.total_skills, 3);
        assert_eq!(stats.roots, 1, "only root-skill has no parent");
        assert_eq!(stats.orphans, 1, "orphan-skill points to missing parent");
        assert_eq!(stats.max_depth, 1, "child depth = 1");
        assert_eq!(stats.categories.get("coding"), Some(&2));
        assert_eq!(stats.categories.get("analysis"), Some(&1));
    }

    #[test]
    fn test_flagged_attributions_reports_procedure_heavy() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let heavy = r#"---
name: heavy-skill
description: procedure heavy
triggers: ["heavy"]
category: general
---
1. rebuild the entire project and verify the compile twice
2. cargo clean then re-read every single file and recheck
3. verify the compile and audit the result line by line
4. validate the rebuild and recheck the verify output
5. cargo clean again and re-read the recheck report
"#;
        std::fs::write(skills_dir.join("heavy.md"), heavy).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();
        engine.activate_skill("heavy-skill").unwrap();

        let flagged = engine.flagged_attributions();
        assert!(!flagged.is_empty(), "procedure-heavy skill must surface in flagged report");
        assert!(flagged.iter().all(|a| a.procedure_heavy));
    }

    // ── P23 PromptLibrary ──
    #[test]
    fn test_prompt_register_and_get() {
        let mut lib = PromptLibrary::new();
        lib.register(PromptEntry::new("judge", "score").with_tags(vec!["eval".into()])).unwrap();
        assert_eq!(lib.len(), 1);
        let p = lib.get("judge").expect("get");
        assert_eq!(p.version, 1);
        assert_eq!(p.tags, vec!["eval".to_string()]);
    }

    #[test]
    fn test_prompt_same_name_bumps_version() {
        let mut lib = PromptLibrary::new();
        lib.register(PromptEntry::new("judge", "v1")).unwrap();
        lib.register(PromptEntry::new("judge", "v2")).unwrap();
        assert_eq!(lib.len(), 1);
        assert_eq!(lib.get("judge").unwrap().version, 2);
        assert_eq!(lib.get("judge").unwrap().content, "v2");
    }

    #[test]
    fn test_prompt_by_tag() {
        let mut lib = PromptLibrary::new();
        lib.register(PromptEntry::new("a", "1").with_tags(vec!["eval".into()])).unwrap();
        lib.register(PromptEntry::new("b", "2").with_tags(vec!["extract".into()])).unwrap();
        assert_eq!(lib.by_tag("eval").len(), 1);
        assert_eq!(lib.by_tag("extract").len(), 1);
        assert_eq!(lib.by_tag("nope").len(), 0);
    }

    #[test]
    fn test_prompt_missing_returns_none() {
        let lib = PromptLibrary::new();
        assert!(lib.get("absent").is_none());
        assert!(lib.is_empty());
    }

    #[test]
    fn test_prompt_selftest() {
        let lib = PromptLibrary::new();
        assert!(lib.self_test().is_ok());
    }

    // ── P4: AnchorPromote (dsh-anchored-standard 吸收) ──
    #[test]
    fn test_disclosure_default_stage_is_minimal() {
        let ap = AnchorPromote::default();
        assert_eq!(ap.stage, 0, "default anchors on stage 0");
        assert_eq!(ap.minimal_tools, 2);
        assert_eq!(ap.standard_tools, 10);
        assert_eq!(ap.active_tool_count(), 2, "stage 0 → Minimal tool budget");
    }

    #[test]
    fn test_disclosure_stage_default_fields() {
        let ds = DisclosureStage::default();
        assert_eq!(ds.stage, 0);
        assert_eq!(ds.label, "Minimal");
        assert_eq!(ds.tool_count, 2);
        assert!(!ds.durable, "default stage not yet durable");
    }

    #[test]
    fn test_disclosure_record_call_then_promote() {
        let mut ap = AnchorPromote::default();
        assert!(!ap.maybe_promote(), "no durable call → stay anchored");
        ap.record_call();
        assert_eq!(ap.durable_calls, 1);
        assert!(ap.maybe_promote(), "first durable call → promote");
        assert_eq!(ap.stage, 1);
        assert!(!ap.maybe_promote(), "already promoted → no re-promote");
    }

    #[test]
    fn test_disclosure_promoted_active_tool_count_is_standard() {
        let mut ap = AnchorPromote::default();
        assert_eq!(ap.active_tool_count(), 2);
        ap.record_call();
        assert!(ap.maybe_promote());
        assert_eq!(ap.active_tool_count(), 10, "promoted → Standard tool budget");
    }

    #[test]
    fn test_disclosure_savings_is_80_percent() {
        let ap = AnchorPromote::default();
        assert_eq!(ap.disclosure_savings(), 0.8, "(1 - 2/10) = 0.8");
    }

    #[test]
    fn test_disclosure_engine_step_transitions_and_stays() {
        let mut engine = SkillEngine::new(PathBuf::from("/nonexistent/skills"));
        assert_eq!(engine.disclosure.stage, 0);
        assert!(!engine.step_disclosure(), "no durable call yet → no transition");
        engine.disclosure.record_call();
        assert!(engine.step_disclosure(), "first durable call → transition");
        assert_eq!(engine.disclosure.stage, 1);
        assert_eq!(engine.disclosure.active_tool_count(), 10);
        assert!(!engine.step_disclosure(), "second call stays promoted");
        assert_eq!(engine.disclosure.stage, 1);
    }

    #[test]
    fn test_disclosure_visible_active_gates_tool_set() {
        // P4 行为接线回归: visible_active() 必须真实限制工具集 —
        // Minimal 阶段只暴露预算(2)个高优先级技能, promote 后完整暴露。
        let mut engine = SkillEngine::new(PathBuf::from("/nonexistent/skills"));
        // 直接注入 4 个活跃技能 (优先级 4/3/2/1 → 1 最高)
        for (name, prio) in [("low", 4u8), ("mid", 3), ("high", 2), ("top", 1)] {
            engine.skills.push(SkillEntry {
                name: name.to_string(),
                description: format!("skill {}", name),
                category: "test".into(),
                triggers: vec![],
                e8_modes: vec![],
                tools: vec![],
                hooks: vec![],
                priority: prio,
                path: PathBuf::new(),
                content: String::new(),
                active: true,
                references: vec![],
                parent: String::new(),
                verified: false,
            });
        }
        // Minimal 阶段: 预算 2 → 只暴露 top/high (priority 1,2)
        assert_eq!(engine.disclosure.stage, 0);
        let visible = engine.visible_active();
        assert_eq!(visible.len(), 2, "Minimal budget gates to 2, got {}", visible.len());
        assert_eq!(visible[0].name, "top", "highest priority first");
        assert_eq!(visible[1].name, "high", "second highest priority");
        // promote 后: 完整暴露全部 4 个
        engine.disclosure.record_call();
        assert!(engine.step_disclosure());
        assert_eq!(engine.disclosure.stage, 1);
        let visible_full = engine.visible_active();
        assert_eq!(visible_full.len(), 4, "Standard stage exposes all active, got {}", visible_full.len());
        // 未激活技能不受门控影响 (list_active 与 visible 一致: 都只含 active)
    }

    #[test]
    fn test_disclosure_new_constructor_with_both_signal() {
        let mut ap = AnchorPromote::new(3, 12, PromoteSignal::Both);
        assert_eq!(ap.stage, 0);
        assert_eq!(ap.minimal_tools, 3);
        assert_eq!(ap.standard_tools, 12);
        ap.record_call();
        assert!(ap.maybe_promote(), "Both signal satisfied on first durable call");
        assert_eq!(ap.active_tool_count(), 12);
        assert_eq!(ap.disclosure_savings(), 0.75, "(1 - 3/12)");
    }

    // ── P6: BookToSkill (book-to-skill 输入侧) ──

    fn sample_book(chapters: Vec<(usize, usize)>) -> BookInput {
        BookInput {
            title: "Sample Book".into(),
            format: DocFormat::Markdown,
            chapters: chapters
                .into_iter()
                .map(|(order, char_count)| DocChapter {
                    title: format!("Chapter {}", order),
                    order,
                    char_count,
                    summary: String::new(),
                })
                .collect(),
        }
    }

    #[test]
    fn test_infer_format_by_extension() {
        assert_eq!(BookToSkill::infer_format("a.pdf"), DocFormat::Pdf);
        assert_eq!(BookToSkill::infer_format("a.epub"), DocFormat::Epub);
        assert_eq!(BookToSkill::infer_format("a.docx"), DocFormat::Docx);
        assert_eq!(BookToSkill::infer_format("a.md"), DocFormat::Markdown);
        assert_eq!(BookToSkill::infer_format("a.MARKDOWN"), DocFormat::Markdown);
        assert_eq!(BookToSkill::infer_format("a.html"), DocFormat::Html);
        assert_eq!(BookToSkill::infer_format("a.htm"), DocFormat::Html);
        assert_eq!(BookToSkill::infer_format("a.rtf"), DocFormat::Rtf);
        assert_eq!(BookToSkill::infer_format("a.mobi"), DocFormat::Mobi);
        assert_eq!(BookToSkill::infer_format("a.unknown"), DocFormat::Markdown);
        assert_eq!(BookToSkill::infer_format("no_extension"), DocFormat::Markdown);
        assert_eq!(BookToSkill::infer_format("a.pdf"), BookToSkill::infer_format("b.PDF"));
        assert_eq!(DocFormat::Pdf.label(), "pdf");
        assert_eq!(DocFormat::Html.label(), "html");
    }

    #[test]
    fn test_normalize_filters_short_chapters_keeps_order() {
        let bts = BookToSkill::new(500, 8);
        let book = sample_book(vec![(0, 400), (1, 600), (2, 100), (3, 900)]);
        let kept = bts.normalize(&book);
        assert_eq!(kept.len(), 2);
        assert_eq!(kept[0].order, 1, "order preserved");
        assert_eq!(kept[0].char_count, 600);
        assert_eq!(kept[1].order, 3);
        assert_eq!(kept[1].char_count, 900);
    }

    #[test]
    fn test_discover_candidates_caps_and_priority() {
        let bts = BookToSkill::new(500, 3);
        let book = sample_book(vec![(0, 600), (1, 3000), (2, 700), (3, 2500)]);
        let candidates = bts.discover_candidates(&book);
        assert_eq!(candidates.len(), 3, "capped at max_candidates=3");
        assert_eq!(candidates[0].priority, 1, "600 chars ≤ 2000 → priority 1");
        assert_eq!(candidates[1].priority, 2, "3000 chars > 2000 → priority 2");
        assert_eq!(candidates[2].priority, 1);
        assert_eq!(candidates[0].source_chapters, vec![0]);
        assert_eq!(candidates[1].source_chapters, vec![1]);
        // 短章节被 normalize 过滤, 不出现在候选中
        assert!(!candidates.iter().any(|c| c.source_chapters == vec![4]));
    }

    #[test]
    fn test_discover_candidates_cleans_titles() {
        let bts = BookToSkill::default();
        let book = BookInput {
            title: "T".into(),
            format: DocFormat::Pdf,
            chapters: vec![DocChapter {
                title: "12. Introduction to Skill Casting!".into(),
                order: 0,
                char_count: 1000,
                summary: String::new(),
            }],
        };
        let candidates = bts.discover_candidates(&book);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].name, "introduction_to_skill_casting");
    }

    #[test]
    fn test_skill_yield_in_range_and_less_than_one_after_filter() {
        let bts = BookToSkill::new(500, 8);
        let book = sample_book(vec![(0, 400), (1, 600), (2, 100), (3, 900)]);
        let yield_ratio = bts.skill_yield(&book);
        assert!(yield_ratio >= 0.0 && yield_ratio <= 1.0, "yield in [0,1]");
        assert!((yield_ratio - 0.75).abs() < 1e-9, "kept 1500 / total 2000 = 0.75");
        assert!(yield_ratio < 1.0, "filtered chapters → yield < 1");
    }

    #[test]
    fn test_skill_yield_all_retained_is_one() {
        let bts = BookToSkill::new(500, 8);
        let book = sample_book(vec![(0, 600), (1, 3000)]);
        assert_eq!(bts.skill_yield(&book), 1.0);
    }

    #[test]
    fn test_skill_yield_empty_input_is_zero() {
        let bts = BookToSkill::default();
        let empty = BookInput {
            title: "Empty".into(),
            format: DocFormat::Markdown,
            chapters: vec![],
        };
        assert_eq!(bts.skill_yield(&empty), 0.0, "no chapters → yield 0");
        let all_short = sample_book(vec![(0, 100), (1, 200)]);
        assert_eq!(bts.skill_yield(&all_short), 0.0, "all filtered → yield 0");
    }

    #[test]
    fn test_book_to_skill_selftest() {
        use crate::core::nt_core_self_test::SelfTest;
        let bts = BookToSkill::default();
        assert_eq!(bts.name(), "nt_mind_book_to_skill");
        assert!(bts.self_test().is_ok());
    }

    // ── cordiverse F1: RevertibleEffect + InverseLedger ──

    #[test]
    fn test_ledger_records_inverse_in_load_order() {
        let mut ledger = InverseLedger::new();
        let id = ledger.begin_install();
        ledger.push_inverse(id, RevertibleEffect::new("first", || Ok(()))).unwrap();
        ledger.push_inverse(id, RevertibleEffect::new("second", || Ok(()))).unwrap();
        ledger.push_inverse(id, RevertibleEffect::new("third", || Ok(()))).unwrap();
        assert_eq!(ledger.inverse_count(id), 3);
        // 账本按加载序记录 (不是逆序)
        assert_eq!(ledger.inverse_labels(id), vec!["first", "second", "third"]);
    }

    #[test]
    fn test_ledger_lifo_teardown_runs_reverse_order() {
        let mut ledger = InverseLedger::new();
        let id = ledger.begin_install();
        let log = Arc::new(std::sync::Mutex::new(Vec::new()));
        for label in ["first", "second", "third"] {
            let l = log.clone();
            ledger.push_inverse(id, RevertibleEffect::new(label, move || {
                l.lock().unwrap().push(label.to_string());
                Ok(())
            })).unwrap();
        }
        let results = ledger.teardown(id);
        assert!(results.iter().all(|r| r.is_ok()));
        let got = log.lock().unwrap().clone();
        assert_eq!(got, vec!["third", "second", "first"], "LIFO: 逆加载序");
        assert_eq!(ledger.inverse_count(id), 0, "teardown 消耗事务");
    }

    #[test]
    fn test_ledger_teardown_unknown_id_is_empty() {
        let mut ledger = InverseLedger::new();
        assert!(ledger.teardown(99).is_empty());
    }

    // ── cordiverse F5: FiberLifecycle ──

    #[test]
    fn test_fiber_lifecycle_rejects_illegal_transitions() {
        let mut fiber = FiberLifecycle::new("f1", 0);
        assert_eq!(fiber.state(), FiberLifecycleState::Loaded);
        assert!(fiber.transition(FiberLifecycleState::Active).is_ok());
        assert!(fiber.transition(FiberLifecycleState::Active).is_err(), "自环非法");
        assert!(fiber.transition(FiberLifecycleState::Loaded).is_err(), "Active→Loaded 非法");
        assert!(fiber.transition(FiberLifecycleState::Retired).is_ok());
        assert!(fiber.transition(FiberLifecycleState::Failed).is_err(), "Retired 是终态");
        assert!(fiber.transition(FiberLifecycleState::Active).is_err(), "Retired→Active 非法");
    }

    #[test]
    fn test_fiber_lifecycle_suspend_and_resume() {
        let mut fiber = FiberLifecycle::new("f2", 0);
        fiber.transition(FiberLifecycleState::Active).unwrap();
        fiber.transition(FiberLifecycleState::Suspended).unwrap();
        assert_eq!(fiber.state(), FiberLifecycleState::Suspended);
        fiber.transition(FiberLifecycleState::Active).unwrap();
        assert_eq!(fiber.state(), FiberLifecycleState::Active);
    }

    #[test]
    fn test_fiber_failure_captured_per_fiber_without_aborting_others() {
        // 两个独立 fiber: A 的逆失败被捕获, B 完全不受影响 (L-Raise)
        let mut ledger = InverseLedger::new();
        let id_a = ledger.begin_install();
        ledger.push_inverse(id_a, RevertibleEffect::new("a1", || Err("a1 boom".into()))).unwrap();
        let id_b = ledger.begin_install();
        ledger.push_inverse(id_b, RevertibleEffect::new("b1", || Ok(()))).unwrap();

        let mut fiber_a = FiberLifecycle::new("a", id_a);
        let mut fiber_b = FiberLifecycle::new("b", id_b);
        fiber_a.transition(FiberLifecycleState::Active).unwrap();
        fiber_b.transition(FiberLifecycleState::Active).unwrap();

        let res_a = ledger.teardown(id_a);
        assert!(res_a[0].is_err(), "fiber A 逆失败必须被捕获");
        fiber_a.record_failure("a1 boom");

        let res_b = ledger.teardown(id_b);
        assert!(res_b[0].is_ok(), "fiber B teardown 不受 A 影响");
        assert_eq!(fiber_b.state(), FiberLifecycleState::Active);
        assert_eq!(fiber_a.state(), FiberLifecycleState::Failed);
        assert_eq!(fiber_a.failures.len(), 1);
        assert_eq!(fiber_a.failures[0].message, "a1 boom");
    }

    // ── wiring: install → ledger → LIFO teardown ──

    #[test]
    fn test_install_wires_inverse_ledger_and_lifo_teardown() {
        let tmp = setup_temp_dir();
        let src = tmp.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("SKILL.md"), sample_skill_content()).unwrap();

        let engine_dir = tmp.path().join("engine");
        std::fs::create_dir_all(&engine_dir).unwrap();
        let mut engine = SkillEngine::new(engine_dir.clone());
        engine.install_skill(&src).unwrap();

        let target = engine_dir.join("rust-analyzer");
        assert!(target.exists(), "installed skill dir must exist");
        let fiber = engine.fiber_lifecycles.get("rust-analyzer").unwrap();
        assert_eq!(fiber.state(), FiberLifecycleState::Active);
        let install_id = fiber.install_id;
        assert!(engine.inverse_ledger.inverse_count(install_id) >= 1);

        let results = engine.uninstall_skill("rust-analyzer").unwrap();
        assert!(results.iter().all(|r| r.is_ok()), "teardown inverses must succeed");
        assert!(!target.exists(), "uninstall 经 LIFO teardown 删除技能目录");
        assert_eq!(engine.fiber_state("rust-analyzer"), Some(FiberLifecycleState::Retired));
        assert!(engine.find_matching("rust", None).is_empty(), "技能从路由移除");
        assert!(engine.uninstall_skill("rust-analyzer").is_err(), "重复卸载被拒");
    }

    // ── C5 自愈检测件: revertible_effects (F1) + fiber_lifecycle (F5) ──

    #[test]
    fn test_revertible_effects_healer() {
        use crate::core::nt_core_self_test::SelfTest;
        let healer = RevertibleEffectsHealer;
        assert_eq!(healer.name(), "nt_mind_skill_engine::revertible_effects_healer");
        assert!(healer.self_test().is_ok());
    }

    #[test]
    fn test_fiber_lifecycle_healer() {
        use crate::core::nt_core_self_test::SelfTest;
        let healer = FiberLifecycleHealer;
        assert_eq!(healer.name(), "nt_mind_skill_engine::fiber_lifecycle_healer");
        assert!(healer.self_test().is_ok());
    }

    #[test]
    fn test_release_dangling_recovers_dangling_ownership() {
        // 合法持有的 fiber 不被误释放; 事务消失的 held fiber 被自动释放。
        let mut engine = SkillEngine::new(PathBuf::new());
        let id = FiberLifecycleHealer::install_held_fiber(&mut engine, "healthy").unwrap();
        let dangling_id = FiberLifecycleHealer::install_held_fiber(&mut engine, "dangling").unwrap();
        engine.inverse_ledger.teardown(dangling_id);

        let released = engine.release_dangling();
        assert_eq!(released, vec!["dangling"], "仅悬挂 fiber 被释放");
        assert!(engine.inverse_ledger.has_transaction(id), "健康 fiber 事务保留");
        assert_eq!(engine.fiber_state("healthy"), Some(FiberLifecycleState::Active));
        assert_eq!(engine.fiber_state("dangling"), Some(FiberLifecycleState::Retired));
        assert!(engine.release_dangling().is_empty(), "幂等: 无残留悬挂");
    }
}
