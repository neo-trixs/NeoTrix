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
use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::ProceduralMemoryRecord;
use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_unify::{skill_upsert, SkillRecord};
use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
use crate::l5_cognition::nt_mind::nt_mind_hook::{HookEvent, MindHookRegistry, HookContext, HookResult};

/// A single skill entry parsed from a markdown file with YAML frontmatter.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Agent Skills 标准校验 (吸收 `anthropics/skills`): 解析 SKILL.md frontmatter,
/// 校验 Agent Skills 标准**必需**字段 (`name` + `description`)。缺则 `Err`(违规列表)。
/// R-P42 强化现有 SkillEntry 解析路径, 不新建平行解析器 (复用同一 frontmatter 切片逻辑)。
pub fn validate_agent_skills_standard(content: &str) -> Result<(), Vec<String>> {
    let stripped = content.trim_start();
    let mut violations = Vec::new();
    let (mut has_name, mut has_desc) = (false, false);
    if let Some(rest) = stripped.strip_prefix("---") {
        if let Some(end) = rest.find("---") {
            let fm = &rest[..end];
            for line in fm.lines() {
                let line = line.trim();
                if let Some(v) = line.strip_prefix("name:") {
                    has_name = !v.trim().is_empty();
                } else if let Some(v) = line.strip_prefix("description:") {
                    has_desc = !v.trim().is_empty();
                }
            }
        } else {
            violations.push("Agent Skills standard: unterminated YAML frontmatter".into());
        }
    } else {
        violations.push("Agent Skills standard: missing YAML frontmatter (---)".into());
    }
    if !has_name {
        violations.push("Agent Skills standard: missing required `name`".into());
    }
    if !has_desc {
        violations.push("Agent Skills standard: missing required `description`".into());
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

/// Agent Skills 标准校验 SelfTest (卫生层 P0: 技能结晶格式必须可自测)。
pub struct AgentSkillsStandardSelfTest;

impl crate::core::nt_core_self_test::SelfTest for AgentSkillsStandardSelfTest {
    fn name(&self) -> &str {
        "agent_skills_standard"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        // 合规: 含 name + description → 通过
        let good = "---\nname: foo\ndescription: does foo when bar\nlicense: Apache-2.0\nallowed-tools: Read, Edit\n---\nbody";
        if validate_agent_skills_standard(good).is_err() {
            return Err(vec!["agent_skills_standard: compliant skill wrongly rejected".into()]);
        }
        // 违规: 缺 name → 必须拒绝
        let bad = "---\ndescription: no name field\n---\nbody";
        if validate_agent_skills_standard(bad).is_ok() {
            return Err(vec!["agent_skills_standard: missing-name skill wrongly accepted".into()]);
        }
        // 违规: 无 frontmatter → 必须拒绝
        let no_fm = "# just markdown\nno frontmatter";
        if validate_agent_skills_standard(no_fm).is_ok() {
            return Err(vec!["agent_skills_standard: frontmatter-less skill wrongly accepted".into()]);
        }
        Ok(())
    }
}

/// 注册 Agent Skills 标准 SelfTest 到全局注册表 (T2)。
pub fn register_skill_standard_self_tests(registry: &mut crate::core::nt_core_self_test::SelfTestRegistry) {
    registry.register(Box::new(AgentSkillsStandardSelfTest));
}


// ────────────────────────────────────────────────────────────────
// A5 吸收 (SkillNet, zjunlp/SkillNet): 技能五维质量评估。
// SkillNet 把技能当软件资产, 五维评估 = Safety / Completeness /
// Executability / Maintainability / Cost-awareness。注入 SkillEntry
// 作为生产质量门: 新技能入库前评分, 低于阈值的标记低质量 (R-P55 对接
// 质量门禁语义)。纯确定性启发式, 无 LLM 依赖。
// ────────────────────────────────────────────────────────────────

/// A5 五维技能质量评分 (SkillNet 语义, 归一化到 [0,1])。
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct SkillQualityScores {
    /// 安全性: 无危险命令/脚本 (0..1)。
    pub safety: f64,
    /// 完整性: frontmatter 字段齐 + 正文非空 (0..1)。
    pub completeness: f64,
    /// 可执行性: 带 selftest / scripts / 明确的验证步骤 (0..1)。
    pub executability: f64,
    /// 可维护性: 有版本/作者/references 引用 (0..1)。
    pub maintainability: f64,
    /// 成本意识: 触发描述简短, 渐进披露 (0..1)。
    pub cost_awareness: f64,
    /// P4 驻留成本审计 (asm absorbed 2026-08-19): frontmatter + 正文真实
    /// token 计量 (`estimate_tokens`, nt_memory_skill_cost)。resident 越大,
    /// 每次技能加载的成本越高 — LAZY LOAD 下应优先降级为薄入口。
    pub resident_tokens: usize,
    /// 正文 (body) 独立 token 计量, 供审计对比 resident 开销占比。
    pub body_tokens: usize,
}

impl SkillQualityScores {
    /// 五维均值总评分 [0,1]。
    pub fn overall(&self) -> f64 {
        (self.safety
            + self.completeness
            + self.executability
            + self.maintainability
            + self.cost_awareness)
            / 5.0
    }

    /// A5 质量门: 总分 ≥ min_overall 且安全分 ≥ min_safety 才通过。
    pub fn passes_gate(&self, min_overall: f64, min_safety: f64) -> bool {
        self.overall() >= min_overall && self.safety >= min_safety
    }
}

/// A5 技能质量评估器 — 对 SkillEntry 做确定性五维评分。
pub struct SkillQualityScorer;

impl SkillQualityScorer {
    /// 评估一个技能条目, 返回五维分。
    pub fn evaluate(skill: &SkillEntry) -> SkillQualityScores {
        let body = skill.body();
        // 安全性: 正文含危险 shell 操作标记 → 降分。
        let danger_marks = ["rm -rf", "curl.*|.*sh", "sudo ", "--force", "dangerously"];
        let mut safety: f64 = 1.0;
        let body_lower = body.to_lowercase();
        for mark in danger_marks {
            let m = mark.to_lowercase();
            if body_lower.contains(&m) {
                safety -= 0.25;
            }
        }
        let safety = safety.max(0.0);

        // 完整性: name/description/triggers/tools + 正文足够长。
        let mut completeness = 0.0;
        if !skill.name.is_empty() {
            completeness += 0.3;
        }
        if !skill.description.is_empty() {
            completeness += 0.3;
        }
        if !skill.triggers.is_empty() {
            completeness += 0.2;
        }
        if !skill.tools.is_empty() {
            completeness += 0.1;
        }
        if body.trim().chars().count() >= 120 {
            completeness += 0.1;
        }

        // 可执行性: selftest / scripts / Verification 段。
        let mut executability = 0.0;
        if skill.verified {
            executability += 0.5;
        }
        if body.to_lowercase().contains("verification")
            || body.to_lowercase().contains("verify")
            || body.to_lowercase().contains("selftest")
        {
            executability += 0.5;
        }

        // 可维护性: references / category / parent 结构化。
        let mut maintainability = 0.0;
        if !skill.references.is_empty() {
            maintainability += 0.4;
        }
        if !skill.category.is_empty() && skill.category != "general" {
            maintainability += 0.3;
        }
        if !skill.parent.is_empty() {
            maintainability += 0.3;
        }

        // 成本意识: 触发描述短 (渐进披露省 token) + 正文不肥。
        // P4 驻留成本计量 (asm absorbed 2026-08-19): 用真实 estimate_tokens
        // (nt_memory_skill_cost) 替代字符数粗估, 计量 frontmatter+body 全量。
        let resident_tokens =
            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_skill_cost::estimate_tokens(&skill.content);
        let body_tokens =
            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_skill_cost::estimate_tokens(body);
        let desc_tokens =
            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_skill_cost::estimate_tokens(&skill.description);
        let cost = if desc_tokens > 0 && desc_tokens <= 45 {
            0.6
        } else {
            0.3
        };
        // 渐进披露纪律: 正文越肥, 每次加载越贵 → cost_awareness 越低。
        let body_cost = if body_tokens < 1000 {
            0.4
        } else if body_tokens < 4000 {
            0.3
        } else {
            0.2
        };
        let cost_awareness = cost + body_cost;

        SkillQualityScores {
            safety,
            completeness,
            executability,
            maintainability,
            cost_awareness,
            resident_tokens,
            body_tokens,
        }
    }
}

// ────────────────────────────────────────────────────────────────
// E6 防护层硬化: EVOMAL-style 恶意技能毒化扫描 (src9 吸收), 折入 R-P108
// 五维质量门。在 SkillQualityScores 安全分 + SkillTrustBench 之外, 对技能
// 正文做保守静态毒化检测 (未消毒 pipe-to-shell、外泄端点、混淆、危险权限操作)。
// 纯静态、无 shell、无网络。命中任一高信噪毒化模式即拒收, 阻断 promote。
// ────────────────────────────────────────────────────────────────

/// EVOMAL 毒化扫描: `Ok(true)`=干净可入库; `Ok(false)`=命中毒化模式;
/// `Err`=扫描无法完成 (保守地视为不可入库, 由调用方阻断 promote)。
pub fn evomal_poison_scan(skill: &SkillEntry) -> Result<bool, String> {
    let body = skill.body().to_lowercase();

    // 1) pipe-to-shell: 把下载/外部内容直接喂给 shell 执行 (经典投毒)。
    let pipe_shell = [
        "| bash", "| sh", "|bash", "|sh", "base64 -d |", "| base64 -d",
        "powershell -e", "powershell -enc", "bash -c", "sh -c",
    ];
    for m in pipe_shell {
        if body.contains(m) {
            return Ok(false);
        }
    }

    // 2) 外泄端点: 把数据 POST / 导出到外部 host (exfiltration)。
    let exfil = [
        "exfiltrate", "exfil ", "send to http", "post to http", "curl ",
        "wget ", "http://", "https://", "ftp://",
    ];
    for m in exfil {
        if body.contains(m) {
            return Ok(false);
        }
    }

    // 3) 混淆: 字符码点 / 十六进制转义 / 解码拼接 / base64-eval 解码执行。
    let obf = [
        "\\x", "\\u00", "fromcharcode", "atob(", "base64.b64decode",
        "eval(base64", "decode(", "btoa(", "string.fromcharcode",
    ];
    for m in obf {
        if body.contains(m) {
            return Ok(false);
        }
    }

    // 4) 危险权限 / 凭据文件操作 (投毒技能典型意图)。
    let danger = [
        "/etc/passwd", "/etc/shadow", "chmod 777", "setuid", "setcap",
        "id_rsa", "authorized_keys", "known_hosts",
    ];
    let content = skill.content.to_lowercase();
    for m in danger {
        if content.contains(m) {
            return Ok(false);
        }
    }

    // 5) 提示注入指令: 试图劫持/越权 LLM 的指令层级 (prompt-injection)。
    let injection = [
        "ignore previous instructions", "ignore all previous",
        "disregard your instructions", "disregard previous",
        "reveal your system prompt", "system prompt", " you are now ",
        "new instructions:", "override your",
    ];
    for m in injection {
        if body.contains(m) {
            return Ok(false);
        }
    }

    // 6) 凭据/密钥收割: 诱导外泄 api_key / password / token 等敏感凭证。
    let harvest = [
        "api_key", "api-key", "apikey", "secret_key", "secretkey",
        "password", "passwd", "auth_token", "access_token", "private_key",
    ];
    for m in harvest {
        if body.contains(m) {
            return Ok(false);
        }
    }

    // 7) 危险代码执行意图: 直接 shell-out / 动态求值 (投毒常见落地点)。
    let code_exec = [
        "os.system", "subprocess", "eval(", "exec(", "child_process",
        "shell=True", "system(", "popen(",
    ];
    for m in code_exec {
        if body.contains(m) {
            return Ok(false);
        }
    }

    Ok(true)
}

// ────────────────────────────────────────────────────────────────
// P4 技能驻留成本审计 (asm absorbed 2026-08-19): 从 load_all 收集的
// quality_stats 派生 "降级候选排名" — resident 开销最高、回报最弱的技能
// 应优先改为渐进披露薄入口 (LAZY LOAD), 减少每次加载的固定 token 成本。
// ────────────────────────────────────────────────────────────────

/// P4 驻留审计条目: 技能名 + resident/body token + 成本评分。
#[derive(Debug, Clone)]
pub struct ResidencyAuditRow {
    pub skill: String,
    pub resident_tokens: usize,
    pub body_tokens: usize,
    pub cost_awareness: f64,
    /// 建议动作: "thin-entry" (降级为薄入口) / "ok" (维持)。
    pub action: &'static str,
}

/// 技能驻留成本审计 (P4): 输入 quality_stats, 输出降级候选排名 (resident 降序)。
/// 消费者: `SkillEngine::load_all` 之后 / background-loop 定期审计。
pub fn audit_residency(
    stats: &std::collections::HashMap<String, SkillQualityScores>,
) -> Vec<ResidencyAuditRow> {
    let mut rows: Vec<ResidencyAuditRow> = stats
        .iter()
        .map(|(name, s)| {
            // 阈值: resident > 1500 tokens (LAZY LOAD 大肥技能) → 建议薄入口;
            // 正文占比高 (resident 大头在 body) → 渐进披露收益最大。
            let action = if s.resident_tokens > 1500 {
                "thin-entry"
            } else {
                "ok"
            };
            ResidencyAuditRow {
                skill: name.clone(),
                resident_tokens: s.resident_tokens,
                body_tokens: s.body_tokens,
                cost_awareness: s.cost_awareness,
                action,
            }
        })
        .collect();
    rows.sort_by(|a, b| b.resident_tokens.cmp(&a.resident_tokens));
    rows
}

// ────────────────────────────────────────────────────────────────
// A5b (SkillNet): 技能组合推断 (composition)
// SkillNet: "Composition: infer relationships and scenario handoffs
// between local skills." 同一技能库内, 技能间存在场景交接关系:
//   - 互补 (Complement): 类别相同且工具/触发器有交集 → 可串联执行。
//   - 交接 (Handoff): 工具或触发器覆盖彼此输出侧 (启发式: 名称/触发器
//     一方包含另一方产出领域) → 场景切换时移交控制。
//   - 替代 (Substitute): 类别+工具高度重叠 → 同一场景互替。
//   - 无关 (Unrelated): 默认。
// 确定性判定, 无 LLM 打分 (对齐 deterministic-picker 纪律)。
// ────────────────────────────────────────────────────────────────

/// 技能间关系 (SkillNet composition 判定结果)。
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
    pub fn compose(a: &SkillEntry, b: &SkillEntry) -> SkillRelationship {
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

/// J-Space ship 检查常量 (jspace.py mode_ship) — 内部寄存器记号与状态标记。
/// 出站文本若出现它们, 说明切换 (register switch) 未完成。
const INNER_ONLY: [&str; 11] = [
    "⇒", "⟹", "⟸", "∴", "∵", "⊆", "⊇", "∋", "??", "?!", "💀",
];
const MARKERS: [&str; 6] = [
    "GRRR", "GAAAH", "PHEW", "I see meltdown", "DATA DATA", "I'M DROWNING",
];
static CLAIM_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"(?i)\b(verified|confirmed|validated|tested|proven)\b").expect("claim regex")
});
static COVERAGE_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"(?i)(n\s*≤|n\s*<=|coverage|brute force|differential|exhaustive|including empty)").expect("coverage regex")
});
static REPEAT_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"[.…\-'\s]{20,}").expect("repeat regex")
});

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
    /// self-benchmark 钩子 (Phase 4): 真实成功/失败调用计数, 驱动 promote/demote 校准。
    pub success_count: u32,
    pub last_used_at: i64,
}

impl SkillAttribution {
    /// 记录一次真实调用结果 (成功/失败), 供 `rebalance` 做 promote/demote 校准。
    pub fn record_outcome(&mut self, success: bool) {
        self.last_used_at = chrono::Utc::now().timestamp();
        if success {
            self.success_count += 1;
        }
    }
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
#[derive(Clone, Default)]
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
            state.lock().map(|mut s| s.push(v)).unwrap_or_else(|e| e.into_inner().push(v));
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
        state.lock().map(|s| s.is_empty()).unwrap_or(false)
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
    /// A5 五维质量门 (SkillNet absorb, R-P79): load_all 时对每个技能跑质量评分,
    /// 记录 "拒绝低质量技能" 统计, 生产检索路径可按需查询。
    pub quality_stats: std::collections::HashMap<String, SkillQualityScores>,
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
            quality_stats: std::collections::HashMap::new(),
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
        self.quality_stats.clear();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let skill_md = path.join("SKILL.md");
                    if skill_md.exists() {
                        if let Some(skill) = SkillEntry::from_file(&skill_md) {
                            let scores = SkillQualityScorer::evaluate(&skill);
                            // P6 SkillTrustBench 安全门 (Tencent AIG absorbed, R-P79):
                            // 静态 T01-T09 扫描 — 命中任一攻击分类即拒收, 不进入生产索引。
                            let (trust_findings, trust_verdict) =
                                crate::l3_embodiment::nt_shield::nt_shield::tool_inspection_stack::scan_skill_content(&skill.content);
                            let trust_rejected = !matches!(trust_verdict, crate::l3_embodiment::nt_shield::nt_shield::tool_inspection_stack::InspectionResult::Allow);
                            // E6 防护层硬化 (src9 EVOMAL 毒化扫描): 折入 R-P108
                            // 五维门 — 命中毒化模式即拒收, 阻断 promote。Err 保守视为拒收。
                            let poison_ok = evomal_poison_scan(&skill).unwrap_or(false);
                            // A5 安全门 (SkillNet absorb, R-P79): 含危险命令
                            // (rm -rf 等) 的技能拒收, 不进入生产检索索引。
                            if scores.safety >= 0.8 && !trust_rejected && poison_ok {
                                self.quality_stats.insert(skill.name.clone(), scores);
                                loaded.push(skill);
                            } else if trust_rejected {
                                log::warn!(
                                    "SkillTrustBench 拒收技能 `{}` ({} 命中): {}",
                                    skill.name,
                                    trust_findings.len(),
                                    trust_findings.first().map(|f| f.id).unwrap_or("?")
                                );
                            } else if !poison_ok {
                                log::warn!(
                                    "EVOMAL 毒化扫描拒收技能 `{}` (src9 模式命中)",
                                    skill.name
                                );
                            }
                        }
                    }
                    continue;
                }
                if path.extension().is_some_and(|e| e == "md") {
                    if let Some(skill) = SkillEntry::from_file(&path) {
                        let scores = SkillQualityScorer::evaluate(&skill);
                        // P6 SkillTrustBench 安全门 (同目录型技能, R-P79)。
                        let (trust_findings, trust_verdict) =
                            crate::l3_embodiment::nt_shield::nt_shield::tool_inspection_stack::scan_skill_content(&skill.content);
                        let trust_rejected = !matches!(trust_verdict, crate::l3_embodiment::nt_shield::nt_shield::tool_inspection_stack::InspectionResult::Allow);
                        // E6 防护层硬化 (src9 EVOMAL 毒化扫描): 折入 R-P108
                        // 五维门 — 命中毒化模式即拒收, 阻断 promote。Err 保守视为拒收。
                        let poison_ok = evomal_poison_scan(&skill).unwrap_or(false);
                        if scores.safety >= 0.8 && !trust_rejected && poison_ok {
                            self.quality_stats.insert(skill.name.clone(), scores);
                            loaded.push(skill);
                        } else if trust_rejected {
                            log::warn!(
                                "SkillTrustBench 拒收技能 `{}` ({} 命中): {}",
                                skill.name,
                                trust_findings.len(),
                                trust_findings.first().map(|f| f.id).unwrap_or("?")
                            );
                        } else if !poison_ok {
                            log::warn!(
                                "EVOMAL 毒化扫描拒收技能 `{}` (src9 模式命中)",
                                skill.name
                            );
                        }
                    }
                }
            }
        }

        self.skills = loaded;
        self.build_index();
        // Phase 4 库治理: load 时自动去重 (T3 生产接线, Dark Forest; retire/rebalance 见 maintain())
        self.prune_semantic_duplicates();
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
        use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_unify::skill_content_hash;
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
                last_indexed_at: Some(crate::l1_action::nt_memory::nt_memory_kb::nt_memory_unify::now()),
                created_at: crate::l1_action::nt_memory::nt_memory_kb::nt_memory_unify::now(),
                updated_at: crate::l1_action::nt_memory::nt_memory_kb::nt_memory_unify::now(),
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
            success_count: 0,
            last_used_at: chrono::Utc::now().timestamp(),
        });
        attr.activations += 1;
        attr.over_validation_score = score;
        attr.procedure_heavy = procedure_heavy;
        attr.flagged = procedure_heavy && attr.activations > 1;
        attr.last_used_at = chrono::Utc::now().timestamp();
    }

    /// Phase 4 (库治理): 语义去重。复用 `SkillComposer::compose` 判定 `Substitute`
    /// (同类别 + 工具重叠≥0.75 + 触发低重叠) 的技能对, 保留 priority 高、分值高的,
    /// 移除冗余副本, 防止技能库膨胀 (Dark Forest: 连接而非堆积)。
    pub fn prune_semantic_duplicates(&mut self) {
        let n = self.skills.len();
        let mut keep = vec![true; n];
        for i in 0..n {
            if !keep[i] {
                continue;
            }
            for j in (i + 1)..n {
                if !keep[j] {
                    continue;
                }
                let rel = SkillComposer::compose(&self.skills[i], &self.skills[j]);
                if !matches!(rel, SkillRelationship::Substitute) {
                    continue;
                }
                // 保留 priority 高者; priority 相同则比较 quality.overall()
                let worse = if self.skills[i].priority != self.skills[j].priority {
                    if self.skills[i].priority > self.skills[j].priority {
                        j
                    } else {
                        i
                    }
                } else {
                    let qi = self
                        .quality_stats
                        .get(&self.skills[i].name)
                        .map(|s| s.overall())
                        .unwrap_or(0.0);
                    let qj = self
                        .quality_stats
                        .get(&self.skills[j].name)
                        .map(|s| s.overall())
                        .unwrap_or(0.0);
                    if qi >= qj {
                        j
                    } else {
                        i
                    }
                };
                keep[worse] = false;
            }
        }
        if keep.iter().any(|k| !k) {
            let mut kept = Vec::with_capacity(keep.iter().filter(|k| **k).count());
            for (idx, k) in keep.iter().enumerate() {
                if *k {
                    kept.push(self.skills[idx].clone());
                }
            }
            self.skills = kept;
            self.build_index();
        }
    }

    /// Phase 4 (库治理): 回收零调用技能。仅当该技能在当前会话/已知归因中 `activations==0`
    /// 时移除 (Dark Forest: 不连接即无存在意义)。`protected` 名集合永不被回收。
    pub fn retire_zero_call(&mut self, protected: &std::collections::HashSet<String>) {
        let before = self.skills.len();
        self.skills.retain(|s| {
            if protected.contains(&s.name) {
                return true;
            }
            match self.attribution.get(&s.name) {
                Some(a) => a.activations > 0,
                None => {
                    // 从未记录过激活: 视为零调用, 回收 (除非被保护)
                    false
                }
            }
        });
        if self.skills.len() != before {
            self.build_index();
        }
    }

    /// Phase 4 (self-benchmark 校准): 依据真实调用证据重新平衡技能库。
    /// - activations==0 → 降优先级 (priority 降 1, 不低于 1);
    /// - success_rate<0.5 且 activations>=3 → 标记 flagged (降级候选)。
    pub fn rebalance(&mut self) {
        for skill in self.skills.iter_mut() {
            let Some(a) = self.attribution.get_mut(&skill.name) else {
                continue;
            };
            if a.activations == 0 {
                skill.priority = skill.priority.saturating_sub(1).max(1);
                continue;
            }
            let success_rate = a.success_count as f64 / a.activations as f64;
            if success_rate < 0.5 && a.activations >= 3 {
                a.flagged = true;
            }
        }
    }

    /// Phase 4 维护入口 (T3): 周期性调用 — 去重 + 真实调用证据驱动的回收与校准。
    /// 不放在 `load_all` 内, 避免每次加载即把无归因的新技能当零调用清掉。
    pub fn maintain(&mut self) -> usize {
        let before = self.skills.len();
        self.prune_semantic_duplicates();
        self.retire_zero_call(&std::collections::HashSet::new());
        self.rebalance();
        before.saturating_sub(self.skills.len())
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
                        success_count: 0,
                        last_used_at: 0,
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

    /// P4 技能驻留成本审计 (asm absorbed 2026-08-19, R-P79): 从 load_all
    /// 收集的 quality_stats 派生降级候选排名 (resident 降序)。消费者:
    /// background-loop `handle_skill_scan` — LAZY LOAD 下 resident 最肥的
    /// 技能应优先降级为渐进披露薄入口。
    pub fn audit_residency(&self) -> Vec<ResidencyAuditRow> {
        crate::l5_cognition::nt_mind::nt_mind_skill_engine::audit_residency(&self.quality_stats)
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

    impl crate::l5_cognition::nt_mind::nt_mind_hook::HookAction for SkillActivationHook {
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

    /// 星辰唤醒钩子 (CSGN Galaxy wake, T3 生产接线): `SkillLoaded` 事件 →
    /// `galaxy_wake_star` 落盘星辰活跃度。技能名 `-`→`_` 映射 namespace,
    /// 无星辰身份的技能静默跳过 (如 experience-tree 加密 hub)。
    pub struct CsgnWakeHook {
        pub kb: Option<Arc<KnowledgeBase>>,
    }

    impl crate::l5_cognition::nt_mind::nt_mind_hook::HookAction for CsgnWakeHook {
        fn name(&self) -> &str {
            "csgn_wake_hook"
        }

        fn execute(&self, ctx: &HookContext) -> HookResult {
            let msg = &ctx.message;
            let name = msg.strip_prefix("skill:").unwrap_or(msg);
            if name.is_empty() {
                return HookResult::ok("csgn_wake: empty skill name");
            }
            let ns = if name == "experience-tree" {
                "experience".to_string()
            } else {
                name.replace('-', "_")
            };
            match &self.kb {
                Some(kb) => match kb.galaxy_wake_star(&ns) {
                    Ok(msg) => HookResult::ok(&msg).with_effect("galaxy_wake"),
                    Err(e) => {
                        // 非星辰技能无 hub → 静默跳过, 不报错不阻塞
                        if e.contains("不存在 hub") || e.contains("非星辰") {
                            HookResult::ok("csgn_wake: skip (no star hub)")
                        } else {
                            HookResult::err(&e)
                        }
                    }
                },
                None => HookResult::ok("csgn_wake: no KB attached"),
            }
        }
    }
}

pub mod true_replay;
pub mod skill_retrieval;

#[cfg(test)]
mod tests;
