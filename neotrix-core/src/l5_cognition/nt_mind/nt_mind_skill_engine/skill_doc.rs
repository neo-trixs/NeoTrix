//! skill_doc — 从 `nt_mind_skill_engine.rs` 拆分 (行为零变更).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use super::skill_attribution::parse_array_field;

/// A single skill entry parsed from a markdown file with YAML frontmatter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDocEntry {
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

#[deprecated(note = "Use `SkillDocEntry` instead")]
pub type SkillEntry = SkillDocEntry;

impl SkillDocEntry {
    pub(crate) fn from_file(path: &Path) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        Self::from_content(path, &content)
    }

    pub(crate) fn from_content(path: &Path, content: &str) -> Option<Self> {
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
/// R-P42 强化现有 SkillDocEntry 解析路径, 不新建平行解析器 (复用同一 frontmatter 切片逻辑)。
pub fn _validate_agent_skills_standard(content: &str) -> Result<(), Vec<String>> {
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

impl crate::l0_substrate::nt_core_self_test::SelfTest for AgentSkillsStandardSelfTest {
    fn name(&self) -> &str {
        "agent_skills_standard"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        // 合规: 含 name + description → 通过
        let good = "---\nname: foo\ndescription: does foo when bar\nlicense: Apache-2.0\nallowed-tools: Read, Edit\n---\nbody";
        if _validate_agent_skills_standard(good).is_err() {
            return Err(vec!["agent_skills_standard: compliant skill wrongly rejected".into()]);
        }
        // 违规: 缺 name → 必须拒绝
        let bad = "---\ndescription: no name field\n---\nbody";
        if _validate_agent_skills_standard(bad).is_ok() {
            return Err(vec!["agent_skills_standard: missing-name skill wrongly accepted".into()]);
        }
        // 违规: 无 frontmatter → 必须拒绝
        let no_fm = "# just markdown\nno frontmatter";
        if _validate_agent_skills_standard(no_fm).is_ok() {
            return Err(vec!["agent_skills_standard: frontmatter-less skill wrongly accepted".into()]);
        }
        Ok(())
    }
}

/// 注册 Agent Skills 标准 SelfTest 到全局注册表 (T2)。
pub fn register_skill_standard_self_tests(registry: &mut crate::l0_substrate::nt_core_self_test::SelfTestRegistry) {
    registry.register(Box::new(AgentSkillsStandardSelfTest));
}


// ────────────────────────────────────────────────────────────────
// A5 吸收 (SkillNet, zjunlp/SkillNet): 技能五维质量评估。
// SkillNet 把技能当软件资产, 五维评估 = Safety / Completeness /
// Executability / Maintainability / Cost-awareness。注入 SkillDocEntry
// 作为生产质量门: 新技能入库前评分, 低于阈值的标记低质量 (R-P55 对接
// 质量门禁语义)。纯确定性启发式, 无 LLM 依赖。
// ────────────────────────────────────────────────────────────────
