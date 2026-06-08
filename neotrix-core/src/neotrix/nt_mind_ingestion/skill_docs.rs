//! SKILL 文档系统 — 以 YAML 结构化文档存储意识能力,
//! 可被发现、加载并链接到 HyperCube VSA 知识空间.
//!
//! 每个 `.skill.yaml` 文件定义了一个可被意识核心按需激活的能力.
//! 文件格式设计为纯 YAML，无需外部解析器依赖。

/// SKILL 文档的 YAML 模式设计。
///
/// ```yaml
/// # .skill.yaml — 意识能力文档
///
/// skill_id: str (required)
///   唯一标识符，用于 HyperCube 检索和依赖引用。
///   命名约定: `{domain}_{subdomain}_{name}`，例如 `code_review_syntax_parser`
///
/// name: str (required)
///   人类可读的能力名称。
///
/// version: str (required)
///   语义化版本号 (semver)，例如 "1.2.0"
///
/// description: str (optional)
///   能力功能的自然语言描述。
///
/// triggers: list of trigger (optional)
///   激活此技能的条件。
///   每个 trigger:
///     event: str (optional) — 触发事件，例如 "git.pre_commit"
///     pattern: str (optional) — 触发模式/关键词，例如 "review:"
///   √ 至少提供一个 event 或 pattern。
///
/// io: mapping (required)
///   VSA 输入/输出规范。
///     input_vsa_type: str — 输入 VSA 向量类型名，例如 "CodeDiff"
///     output_vsa_type: str — 输出 VSA 向量类型名，例如 "ReviewResult"
///     input_dim: int (default 4096) — 输入 VSA 维度
///     output_dim: int (default 4096) — 输出 VSA 维度
///
/// e8_mode: mapping (optional)
///   E8 晶格六十四卦模式。
///     hexagram: int (0-63) — I Ching 卦象索引
///     description: str (optional) — 卦象描述
///
/// quality_threshold: float (0.0-1.0, default 0.7)
///   激活该技能所需的最低质量评分。
///
/// dependencies: list of str (optional)
///   此技能依赖的其他 skill_id 列表。
///
/// version_history: list of entry (optional)
///   版本演化记录。
///   每个 entry:
///     version: str — 版本号
///     date: str — ISO 8601 日期
///     changes: str — 变更描述
///
/// vsa_tag: mapping (optional)
///   自身-世界边界标签。
///     domain: str — "Self" | "World"
///     subdomain: str — "Thought" | "Memory" | "Plan" | "Skill"
/// ```
pub const SKILL_SCHEMA_YAML: &str = "见上方模块文档";

// ============================================================================
// 类型定义
// ============================================================================

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 技能触发条件
#[derive(Debug, Clone, PartialEq)]
pub struct SkillTrigger {
    /// 触发事件名，例如 "git.pre_commit", "pull_request.opened"
    pub event: Option<String>,
    /// 触发模式/关键词，例如 "review:"
    pub pattern: Option<String>,
}

/// VSA 输入/输出规范
#[derive(Debug, Clone, PartialEq)]
pub struct SkillIoSpec {
    /// 输入 VSA 向量类型
    pub input_vsa_type: String,
    /// 输出 VSA 向量类型
    pub output_vsa_type: String,
    /// 输入 VSA 维度（默认 4096）
    pub input_dim: usize,
    /// 输出 VSA 维度（默认 4096）
    pub output_dim: usize,
}

impl Default for SkillIoSpec {
    fn default() -> Self {
        Self {
            input_vsa_type: String::new(),
            output_vsa_type: String::new(),
            input_dim: 4096,
            output_dim: 4096,
        }
    }
}

/// E8 六十四卦模式
#[derive(Debug, Clone, PartialEq)]
pub struct SkillE8Mode {
    /// I Ching 卦象索引 (0-63)
    pub hexagram: u8,
    /// 可选卦象描述
    pub description: Option<String>,
}

/// 版本历史条目
#[derive(Debug, Clone, PartialEq)]
pub struct SkillVersionEntry {
    /// 版本号 (semver)
    pub version: String,
    /// ISO 8601 日期
    pub date: String,
    /// 变更描述
    pub changes: String,
}

/// VSA 自身-世界边界标签
#[derive(Debug, Clone, PartialEq)]
pub struct SkillVsaTag {
    /// 域: "Self" 或 "World"
    pub domain: String,
    /// 子域: "Thought", "Memory", "Plan", "Skill" 等
    pub subdomain: String,
}

impl Default for SkillVsaTag {
    fn default() -> Self {
        Self {
            domain: "Self".into(),
            subdomain: "Skill".into(),
        }
    }
}

/// 完整的 SKILL 文档定义。
///
/// 对应一个 `.skill.yaml` 文件，表示意识核心的一个可加载能力。
#[derive(Debug, Clone, PartialEq)]
pub struct SkillDefinition {
    /// 唯一标识符
    pub skill_id: String,
    /// 人类可读名称
    pub name: String,
    /// 语义化版本
    pub version: String,
    /// 可选描述
    pub description: Option<String>,
    /// 触发条件列表
    pub triggers: Vec<SkillTrigger>,
    /// VSA 输入/输出规范
    pub io: SkillIoSpec,
    /// E8 六十四卦模式
    pub e8_mode: Option<SkillE8Mode>,
    /// 激活质量阈值 (0.0-1.0)
    pub quality_threshold: f64,
    /// 依赖的其他 skill_id 列表
    pub dependencies: Vec<String>,
    /// 版本历史
    pub version_history: Vec<SkillVersionEntry>,
    /// VSA 自身-世界边界标签
    pub vsa_tag: SkillVsaTag,
    /// 原始文件路径
    pub source_path: Option<PathBuf>,
}

impl SkillDefinition {
    /// 检查此技能是否匹配给定的触发事件。
    pub fn matches_event(&self, event: &str) -> bool {
        self.triggers.iter().any(|t| {
            t.event.as_deref() == Some(event) || t.pattern.as_deref().map_or(false, |p| event.contains(p))
        })
    }

    /// 检查此技能是否匹配给定的文本模式。
    pub fn matches_pattern(&self, text: &str) -> bool {
        self.triggers.iter().any(|t| {
            t.pattern.as_deref().map_or(false, |p| text.contains(p))
        })
    }

    /// 检查质量评分是否满足激活阈值。
    pub fn meets_quality(&self, score: f64) -> bool {
        score >= self.quality_threshold
    }
}

// ============================================================================
// 错误类型
// ============================================================================

/// SKILL 文档加载过程中可能发生的错误。
#[derive(Debug, Clone)]
pub enum SkillLoadError {
    /// 目录读取失败
    Io(String),
    /// 文件读取失败
    ReadFile { path: PathBuf, detail: String },
    /// YAML 语法错误
    ParseError { path: PathBuf, detail: String },
    /// 缺失必填字段
    MissingField { path: PathBuf, field: String },
    /// 字段值无效
    InvalidValue { path: PathBuf, field: String, detail: String },
    /// 重复的 skill_id
    DuplicateSkillId { path: PathBuf, skill_id: String },
}

impl std::fmt::Display for SkillLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(detail) => write!(f, "IO error: {}", detail),
            Self::ReadFile { path, detail } => {
                write!(f, "cannot read {}: {}", path.display(), detail)
            }
            Self::ParseError { path, detail } => {
                write!(f, "parse error in {}: {}", path.display(), detail)
            }
            Self::MissingField { path, field } => {
                write!(f, "missing required field '{}' in {}", field, path.display())
            }
            Self::InvalidValue { path, field, detail } => {
                write!(f, "invalid value for '{}' in {}: {}", field, path.display(), detail)
            }
            Self::DuplicateSkillId { path, skill_id } => {
                write!(f, "duplicate skill_id '{}' in {}", skill_id, path.display())
            }
        }
    }
}

impl std::error::Error for SkillLoadError {}

// ============================================================================
// SKILL 文档加载器
// ============================================================================

/// 扫描 `skills/` 目录中的 `.skill.yaml` 文件并解析为 `SkillDefinition` 列表。
pub struct SkillDocLoader {
    /// 已发现的技能定义 (skill_id → SkillDefinition)
    discovered: HashMap<String, SkillDefinition>,
}

impl Default for SkillDocLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillDocLoader {
    pub fn new() -> Self {
        Self {
            discovered: HashMap::new(),
        }
    }

    /// 扫描指定目录下的所有 `.skill.yaml` 文件并解析。
    ///
    /// 返回成功解析的 `SkillDefinition` 列表。错误会按 skill_id 去重。
    pub fn scan_skills(dir: &Path) -> Result<Vec<SkillDefinition>, Vec<SkillLoadError>> {
        let mut skills = Vec::new();
        let mut errors = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        let entries = match std::fs::read_dir(dir) {
            Ok(rd) => rd,
            Err(e) => {
                return Err(vec![SkillLoadError::Io(format!(
                    "cannot read directory {}: {}",
                    dir.display(),
                    e
                ))]);
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // 递归扫描子目录
                match Self::scan_skills(&path) {
                    Ok(sub_skills) => {
                        for s in sub_skills {
                            if seen_ids.insert(s.skill_id.clone()) {
                                skills.push(s);
                            } else {
                                errors.push(SkillLoadError::DuplicateSkillId {
                                    path: path.clone(),
                                    skill_id: s.skill_id.clone(),
                                });
                            }
                        }
                    }
                    Err(sub_errors) => errors.extend(sub_errors),
                }
                continue;
            }

            if !path
                .extension()
                .and_then(|e| e.to_str())
                .map_or(false, |e| e == "yaml")
            {
                continue;
            }
            if !path
                .file_stem()
                .and_then(|s| s.to_str())
                .map_or(false, |s| s.ends_with(".skill"))
            {
                continue;
            }

            match Self::parse_skill_file(&path) {
                Ok(def) => {
                    if seen_ids.insert(def.skill_id.clone()) {
                        skills.push(def);
                    } else {
                        errors.push(SkillLoadError::DuplicateSkillId {
                            path,
                            skill_id: String::new(),
                        });
                    }
                }
                Err(e) => errors.push(e),
            }
        }

        if errors.is_empty() {
            Ok(skills)
        } else {
            Err(errors)
        }
    }

    /// 解析单个 `.skill.yaml` 文件。
    fn parse_skill_file(path: &Path) -> Result<SkillDefinition, SkillLoadError> {
        let content = std::fs::read_to_string(path).map_err(|e| SkillLoadError::ReadFile {
            path: path.to_path_buf(),
            detail: e.to_string(),
        })?;

        let raw = parse_yaml(&content).map_err(|e| SkillLoadError::ParseError {
            path: path.to_path_buf(),
            detail: e,
        })?;

        Self::build_definition(&raw, path)
    }

    /// 从解析的 YAML 映射构建 `SkillDefinition`。
    fn build_definition(
        map: &YamlMap,
        path: &Path,
    ) -> Result<SkillDefinition, SkillLoadError> {
        let skill_id = map
            .get_str("skill_id")
            .ok_or_else(|| SkillLoadError::MissingField {
                path: path.to_path_buf(),
                field: "skill_id".into(),
            })?
            .to_string();

        let name = map
            .get_str("name")
            .ok_or_else(|| SkillLoadError::MissingField {
                path: path.to_path_buf(),
                field: "name".into(),
            })?
            .to_string();

        let version = map
            .get_str("version")
            .ok_or_else(|| SkillLoadError::MissingField {
                path: path.to_path_buf(),
                field: "version".into(),
            })?
            .to_string();

        let description = map.get_str("description").map(|s| s.to_string());

        let triggers = Self::parse_triggers(map, path)?;
        let io = Self::parse_io(map, path)?;
        let e8_mode = Self::parse_e8_mode(map, path).ok().flatten();
        let quality_threshold = map.get_f64("quality_threshold").unwrap_or(0.7);

        if quality_threshold < 0.0 || quality_threshold > 1.0 {
            return Err(SkillLoadError::InvalidValue {
                path: path.to_path_buf(),
                field: "quality_threshold".into(),
                detail: format!("must be between 0.0 and 1.0, got {}", quality_threshold),
            });
        }

        let dependencies = map
            .get_list("dependencies")
            .map(|l| {
                l.iter()
                    .filter_map(|v| extract_scalar_value(v))
                    .collect()
            })
            .unwrap_or_default();

        let version_history = Self::parse_version_history(map, path)?;

        let vsa_tag = Self::parse_vsa_tag(map, path).ok().flatten().unwrap_or_default();

        Ok(SkillDefinition {
            skill_id,
            name,
            version,
            description,
            triggers,
            io,
            e8_mode,
            quality_threshold,
            dependencies,
            version_history,
            vsa_tag,
            source_path: Some(path.to_path_buf()),
        })
    }

    fn parse_triggers(map: &YamlMap, path: &Path) -> Result<Vec<SkillTrigger>, SkillLoadError> {
        let mut triggers = Vec::new();
        if let Some(list) = map.get_list("triggers") {
            for item in list {
                match item {
                    YamlValue::Map(m) => {
                        let event = m.get_str("event").map(|s| s.to_string());
                        let pattern = m.get_str("pattern").map(|s| s.to_string());
                        if event.is_none() && pattern.is_none() {
                            return Err(SkillLoadError::InvalidValue {
                                path: path.to_path_buf(),
                                field: "triggers[].event".into(),
                                detail: "trigger must have at least 'event' or 'pattern'".into(),
                            });
                        }
                        triggers.push(SkillTrigger { event, pattern });
                    }
                    _ => {
                        return Err(SkillLoadError::InvalidValue {
                            path: path.to_path_buf(),
                            field: "triggers[]".into(),
                            detail: "each trigger must be a mapping".into(),
                        });
                    }
                }
            }
        }
        Ok(triggers)
    }

    fn parse_io(map: &YamlMap, path: &Path) -> Result<SkillIoSpec, SkillLoadError> {
        let io_map = map.get_map("io").ok_or_else(|| SkillLoadError::MissingField {
            path: path.to_path_buf(),
            field: "io".into(),
        })?;

        let input_vsa_type =
            io_map
                .get_str("input_vsa_type")
                .ok_or_else(|| SkillLoadError::MissingField {
                    path: path.to_path_buf(),
                    field: "io.input_vsa_type".into(),
                })?
                .to_string();

        let output_vsa_type =
            io_map
                .get_str("output_vsa_type")
                .ok_or_else(|| SkillLoadError::MissingField {
                    path: path.to_path_buf(),
                    field: "io.output_vsa_type".into(),
                })?
                .to_string();

        let input_dim = io_map.get_int("input_dim").unwrap_or(4096) as usize;
        let output_dim = io_map.get_int("output_dim").unwrap_or(4096) as usize;

        Ok(SkillIoSpec {
            input_vsa_type,
            output_vsa_type,
            input_dim,
            output_dim,
        })
    }

    fn parse_e8_mode(
        map: &YamlMap,
        path: &Path,
    ) -> Result<Option<SkillE8Mode>, SkillLoadError> {
        let e8_map = match map.get_map("e8_mode") {
            Some(m) => m,
            None => return Ok(None),
        };

        let hexagram = e8_map.get_int("hexagram").ok_or_else(|| {
            SkillLoadError::MissingField {
                path: path.to_path_buf(),
                field: "e8_mode.hexagram".into(),
            }
        })?;

        if hexagram < 0 || hexagram > 63 {
            return Err(SkillLoadError::InvalidValue {
                path: path.to_path_buf(),
                field: "e8_mode.hexagram".into(),
                detail: format!("must be 0-63, got {}", hexagram),
            });
        }

        let description = e8_map.get_str("description").map(|s| s.to_string());

        Ok(Some(SkillE8Mode {
            hexagram: hexagram as u8,
            description,
        }))
    }

    fn parse_version_history(
        map: &YamlMap,
        _path: &Path,
    ) -> Result<Vec<SkillVersionEntry>, SkillLoadError> {
        let mut history = Vec::new();
        if let Some(list) = map.get_list("version_history") {
            for item in list {
                if let YamlValue::Map(m) = item {
                    let version = m.get_str("version").unwrap_or("?").to_string();
                    let date = m.get_str("date").unwrap_or("?").to_string();
                    let changes = m.get_str("changes").unwrap_or("").to_string();
                    history.push(SkillVersionEntry {
                        version,
                        date,
                        changes,
                    });
                }
            }
        }
        Ok(history)
    }

    fn parse_vsa_tag(
        map: &YamlMap,
        _path: &Path,
    ) -> Result<Option<SkillVsaTag>, SkillLoadError> {
        let tag_map = match map.get_map("vsa_tag") {
            Some(m) => m,
            None => return Ok(None),
        };

        let domain = tag_map.get_str("domain").unwrap_or("Self").to_string();
        if domain != "Self" && domain != "World" {
            return Err(SkillLoadError::InvalidValue {
                path: _path.to_path_buf(),
                field: "vsa_tag.domain".into(),
                detail: format!("must be 'Self' or 'World', got '{}'", domain),
            });
        }

        let subdomain = tag_map.get_str("subdomain").unwrap_or("Skill").to_string();

        Ok(Some(SkillVsaTag { domain, subdomain }))
    }

    /// 获取所有已发现的技能。
    pub fn discovered(&self) -> Vec<&SkillDefinition> {
        self.discovered.values().collect()
    }

    /// 按 skill_id 查找技能。
    pub fn get(&self, skill_id: &str) -> Option<&SkillDefinition> {
        self.discovered.get(skill_id)
    }

    /// 按名称模糊搜索技能。
    pub fn search(&self, query: &str) -> Vec<&SkillDefinition> {
        let q = query.to_lowercase();
        self.discovered
            .values()
            .filter(|s| {
                s.skill_id.to_lowercase().contains(&q)
                    || s.name.to_lowercase().contains(&q)
                    || s.description.as_deref().map_or(false, |d| d.to_lowercase().contains(&q))
            })
            .collect()
    }
}

// ============================================================================
// 极简 YAML 解析器（无外部依赖）
// ============================================================================

/// YAML 值的内部表示。
#[derive(Debug, Clone, PartialEq)]
enum YamlValue {
    String(String),
    Number(f64),
    Bool(bool),
    Map(YamlMap),
    List(Vec<YamlValue>),
}

/// YAML 映射 (String → YamlValue)。
#[derive(Debug, Clone, PartialEq)]
struct YamlMap {
    entries: HashMap<String, YamlValue>,
}

impl YamlMap {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    fn get(&self, key: &str) -> Option<&YamlValue> {
        self.entries.get(key)
    }

    fn get_str(&self, key: &str) -> Option<&str> {
        match self.entries.get(key) {
            Some(YamlValue::String(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    fn get_f64(&self, key: &str) -> Option<f64> {
        match self.entries.get(key) {
            Some(YamlValue::Number(n)) => Some(*n),
            Some(YamlValue::String(s)) => s.parse::<f64>().ok(),
            _ => None,
        }
    }

    fn get_int(&self, key: &str) -> Option<i64> {
        match self.entries.get(key) {
            Some(YamlValue::Number(n)) => Some(*n as i64),
            Some(YamlValue::String(s)) => s.parse::<i64>().ok(),
            _ => None,
        }
    }

    fn get_map(&self, key: &str) -> Option<&YamlMap> {
        match self.entries.get(key) {
            Some(YamlValue::Map(m)) => Some(m),
            _ => None,
        }
    }

    fn get_list(&self, key: &str) -> Option<&Vec<YamlValue>> {
        match self.entries.get(key) {
            Some(YamlValue::List(l)) => Some(l),
            Some(YamlValue::Map(m)) => {
                match m.entries.get("_list") {
                    Some(YamlValue::List(l)) => Some(l),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

/// 将 YAML 字符串解析为顶层映射。
fn parse_yaml(input: &str) -> Result<YamlMap, String> {
    let lines: Vec<&str> = input.lines().collect();
    let mut root = YamlMap::new();
    parse_block(&lines, 0, 0, &mut root, 0)?;
    Ok(root)
}

/// 递归解析一个缩进块，所有缩进 >= base_indent 的行属于当前块。
/// 返回下一个未处理的行号。
fn parse_block(
    lines: &[&str],
    start: usize,
    base_indent: usize,
    map: &mut YamlMap,
    _depth: usize,
) -> Result<usize, String> {
    let mut i = start;
    while i < lines.len() {
        let raw = lines[i];
        let line = trim_trailing_comment(raw);
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            i += 1;
            continue;
        }

        let indent = line.len() - line.trim_start().len();
        if indent < base_indent {
            break;
        }

        // 列表项: "- key: value" 或 "- value"
        if trimmed.starts_with("- ") {
            i = parse_list_item(lines, i, indent, base_indent, map)?;
            continue;
        }

        // 普通键值对
        if let Some(idx) = trimmed.find(':') {
            let key = trimmed[..idx].trim().to_string();
            if key.is_empty() {
                return Err(format!("empty key at line {}", i + 1));
            }
            let rest = trimmed[idx + 1..].trim().to_string();

            if rest.is_empty() || rest == "{}" || rest == "{|}" {
                // 值为子映射
                let mut child = YamlMap::new();
                let next_i = parse_block(lines, i + 1, indent + 2, &mut child, _depth + 1)?;
                map.entries.insert(key, YamlValue::Map(child));
                i = next_i;
            } else if rest == "|" || rest == ">" {
                // 多行标量
                let mut captured = Vec::new();
                i += 1;
                while i < lines.len() {
                    let nl = lines[i];
                    if nl.trim().is_empty() {
                        i += 1;
                        break;
                    }
                    let nl_indent = nl.len() - nl.trim_start().len();
                    if nl_indent <= indent {
                        break;
                    }
                    captured.push(nl.trim().to_string());
                    i += 1;
                }
                map.entries
                    .insert(key, YamlValue::String(captured.join("\n")));
            } else {
                // 普通标量值
                map.entries.insert(key, parse_scalar(&rest));
                i += 1;
            }
        } else {
            return Err(format!("missing ':' at line {}", i + 1));
        }
    }
    Ok(i)
}

/// 解析一个列表项（以 `- ` 开头的行）及其后续的附属键值对。
///
/// 一个列表项可以跨越多行：
/// ```yaml
///   - key1: val1
///     key2: val2
///   - key3: val3
/// ```
/// `key2` 和 `key1` 属于同一个列表项映射。
///
/// 返回下一个未处理的行号。
fn parse_list_item(
    lines: &[&str],
    start: usize,
    dash_indent: usize,
    _base_indent: usize,
    parent: &mut YamlMap,
) -> Result<usize, String> {
    let content_indent = dash_indent + 2;

    // 解析第一行（`- key: value` 或 `- value`）
    let first = trim_trailing_comment(lines[start]);
    let first_trimmed = first.trim();
    let after_dash = first_trimmed.strip_prefix("- ").unwrap();

    // 先检查是否存在带 key 的列表项内容
    let mut item_map = if let Some(idx) = after_dash.find(':') {
        let k = after_dash[..idx].trim().to_string();
        let v = after_dash[idx + 1..].trim().to_string();
        let mut item = YamlMap::new();
        if k.is_empty() {
            // "- : value" — 格式错误，忽略 key
        } else if v.is_empty() || v == "{}" {
            // "- key:" — 后面跟缩进的子映射，由 parse_block 处理
            let mut child = YamlMap::new();
            let next_i = parse_block(lines, start + 1, content_indent, &mut child, 0)?;
            item.entries.insert(k, YamlValue::Map(child));
            // 收集后续附属键值对 (与 k 同级)
            let mut ii = next_i;
            while ii < lines.len() {
                let nl = trim_trailing_comment(lines[ii]);
                let nlt = nl.trim();
                if nlt.is_empty() || nlt.starts_with('#') {
                    ii += 1;
                    continue;
                }
                let ni = nl.len() - nl.trim_start().len();
                if ni < content_indent {
                    break;
                }
                if ni == content_indent && nlt.starts_with("- ") {
                    break;
                }
                if ni == content_indent {
                    // 附属键
                    if let Some(ci) = nlt.find(':') {
                        let ck = nlt[..ci].trim().to_string();
                        let cv = nlt[ci + 1..].trim().to_string();
                        item.entries.insert(ck, parse_scalar(&cv));
                        ii += 1;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
            append_to_list(parent, YamlValue::Map(item));
            return Ok(ii);
        } else {
            item.entries.insert(k, parse_scalar(&v));
        }
        item
    } else {
        let mut item = YamlMap::new();
        item.entries.insert("_value".to_string(), parse_scalar(after_dash));
        item
    };

    // 收集后续附属键值对 (与第一个键值对同缩进级别)
    let mut ii = start + 1;
    while ii < lines.len() {
        let nl = trim_trailing_comment(lines[ii]);
        let nlt = nl.trim();
        if nlt.is_empty() || nlt.starts_with('#') {
            ii += 1;
            continue;
        }
        let ni = nl.len() - nl.trim_start().len();
        if ni < content_indent {
            break;
        }
        // 下一个列表项开始
        if ni == content_indent - 2 && nlt.starts_with("- ") {
            break;
        }
        if ni == content_indent - 2 {
            break;
        }
        if ni == content_indent {
            if nlt.starts_with("- ") {
                break;
            }
            if let Some(ci) = nlt.find(':') {
                let ck = nlt[..ci].trim().to_string();
                let cv = nlt[ci + 1..].trim().to_string();
                if cv.trim().is_empty() {
                    // 附属键也是子映射
                    let mut child = YamlMap::new();
                    let next_i = parse_block(lines, ii + 1, content_indent + 2, &mut child, 0)?;
                    item_map.entries.insert(ck, YamlValue::Map(child));
                    ii = next_i;
                } else {
                    item_map.entries.insert(ck, parse_scalar(&cv));
                    ii += 1;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }

    append_to_list(parent, YamlValue::Map(item_map));
    Ok(ii)
}

/// 向 map 中的 `_list` 键追加一个值。如果 `_list` 不存在则创建。
fn append_to_list(map: &mut YamlMap, value: YamlValue) {
    let list = map
        .entries
        .entry("_list".to_string())
        .or_insert_with(|| YamlValue::List(Vec::new()));
    if let YamlValue::List(ref mut l) = list {
        l.push(value);
    }
}

/// 解析键值对一行，"key: value" 分割。
#[allow(dead_code)]
fn split_key_value(line: &str) -> Result<(String, String), String> {
    let trimmed = line.trim();
    if let Some(idx) = trimmed.find(':') {
        let key = trimmed[..idx].trim().to_string();
        if key.is_empty() {
            return Err(format!("empty key in line: '{}'", line));
        }
        let value = trimmed[idx + 1..].to_string();
        Ok((key, value))
    } else {
        Err(format!("missing ':' in line: '{}'", line))
    }
}

/// 解析标量值。
fn parse_scalar(s: &str) -> YamlValue {
    let s = s.trim();
    if s.is_empty() {
        return YamlValue::String(String::new());
    }

    // 移除引号
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        return YamlValue::String(s[1..s.len() - 1].to_string());
    }

    // 布尔值
    if s == "true" || s == "yes" {
        return YamlValue::Bool(true);
    }
    if s == "false" || s == "no" {
        return YamlValue::Bool(false);
    }

    // 数字
    if let Ok(n) = s.parse::<f64>() {
        return YamlValue::Number(n);
    }

    // 兜底为字符串
    YamlValue::String(s.to_string())
}

/// 将单行键值对解析并插入 map。
#[allow(dead_code)]
fn parse_key_value_line(line: &str, map: &mut YamlMap) -> Result<(), String> {
    let trimmed = trim_trailing_comment(line);
    let trimmed = trimmed.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return Ok(());
    }
    let (key, value_str) = split_key_value(trimmed)?;
    map.entries.insert(key, parse_scalar(value_str.trim()));
    Ok(())
}

/// 解析缩进的子映射。
#[allow(dead_code)]
fn parse_submap(
    lines: &[&str],
    start: usize,
    base_indent: usize,
    next_line: &mut usize,
) -> Result<YamlMap, String> {
    let mut map = YamlMap::new();
    let mut i = start;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = trim_trailing_comment(line);
        let trimmed = trimmed.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            i += 1;
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        if indent <= base_indent {
            break;
        }
        if trimmed.starts_with("- ") {
            // 列表项
            let item = trimmed.strip_prefix("- ").unwrap();
            if let Some(idx) = item.find(':') {
                let k = item[..idx].trim().to_string();
                let v = item[idx + 1..].trim().to_string();
                if v.is_empty() {
                    // 子映射列表项
                    let sub = parse_submap(lines, i + 1, indent + 2, &mut i)?;
                    let sub_entries = sub.entries.clone();
                    let mut entry_map = YamlMap::new();
                    entry_map.entries.insert("_key".into(), YamlValue::String(k));
                    entry_map.entries.insert("_value".into(), YamlValue::Map(sub));
                    // 扁平化
                    for (sk, sv) in sub_entries {
                        entry_map.entries.insert(sk, sv);
                    }
                    let list = map
                        .entries
                        .entry("_list".to_string())
                        .or_insert_with(|| YamlValue::List(Vec::new()));
                    if let YamlValue::List(ref mut l) = list {
                        l.push(YamlValue::Map(entry_map));
                    }
                } else {
                    let list = map
                        .entries
                        .entry("_list".to_string())
                        .or_insert_with(|| YamlValue::List(Vec::new()));
                    if let YamlValue::List(ref mut l) = list {
                        let mut m = YamlMap::new();
                        m.entries.insert(k, parse_scalar(&v));
                        l.push(YamlValue::Map(m));
                    }
                }
            } else {
                let list = map
                    .entries
                    .entry("_list".to_string())
                    .or_insert_with(|| YamlValue::List(Vec::new()));
                if let YamlValue::List(ref mut l) = list {
                    l.push(parse_scalar(item));
                }
            }
            i += 1;
        } else {
            let (k, v) = split_key_value(trimmed)?;
            let v = v.trim().to_string();
            if v.is_empty() || v == "{}" {
                let sub = parse_submap(lines, i + 1, indent + 2, &mut i)?;
                map.entries.insert(k, YamlValue::Map(sub));
            } else {
                map.entries.insert(k, parse_scalar(&v));
                i += 1;
            }
        }
    }
    *next_line = i;
    Ok(map)
}

/// 从列表项的 YAML 值中提取标量字符串。
/// 支持两种形式：
/// - `YamlValue::String(s)` — 直接字符串
/// - `YamlValue::Map(m)` 中的 `_value` 键 — 形如 `["item"]` 的列表项
fn extract_scalar_value(v: &YamlValue) -> Option<String> {
    match v {
        YamlValue::String(s) => Some(s.clone()),
        YamlValue::Map(m) => {
            m.entries.get("_value").and_then(|inner| {
                if let YamlValue::String(s) = inner {
                    Some(s.clone())
                } else {
                    None
                }
            })
        }
        _ => None,
    }
}

/// 去除行尾注释（# 号后的内容）。
fn trim_trailing_comment(line: &str) -> &str {
    if let Some(idx) = line.find(" #") {
        // 检查 # 是否在引号内
        let before = &line[..idx];
        let in_quote_single = before.matches('\'').count() % 2 == 1;
        let in_quote_double = before.matches('"').count() % 2 == 1;
        if !in_quote_single && !in_quote_double {
            return before;
        }
    }
    line
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn sample_skill_yaml() -> &'static str {
        r#"skill_id: "code_review_syntax_parser"
name: "Syntax-Aware Code Reviewer"
version: "1.2.0"
description: "Parses code diffs and provides structured review feedback"

triggers:
  - event: "git.pre_commit"
    pattern: "review:"
  - event: "pull_request.opened"

io:
  input_vsa_type: "CodeDiff"
  output_vsa_type: "ReviewResult"
  input_dim: 4096
  output_dim: 1024

e8_mode:
  hexagram: 23
  description: "风雷益 — 审慎增益"

quality_threshold: 0.75

dependencies:
  - "syntax_parser"
  - "pattern_matcher"

version_history:
  - version: "1.0.0"
    date: "2026-01-15"
    changes: "Initial implementation"
  - version: "1.2.0"
    date: "2026-03-01"
    changes: "Added security analysis module"

vsa_tag:
  domain: "Self"
  subdomain: "Skill"
"#
    }

    #[test]
    fn test_parse_skill_file() {
        let dir = tempfile::tempdir().expect("tempdir should succeed");
        let file_path = dir.path().join("code_review.skill.yaml");
        let mut file = std::fs::File::create(&file_path).expect("create file should succeed");
        file.write_all(sample_skill_yaml().as_bytes())
            .expect("write should succeed");

        let def = SkillDocLoader::parse_skill_file(&file_path).expect("parse should succeed");

        assert_eq!(def.skill_id, "code_review_syntax_parser");
        assert_eq!(def.name, "Syntax-Aware Code Reviewer");
        assert_eq!(def.version, "1.2.0");
        assert_eq!(
            def.description.as_deref(),
            Some("Parses code diffs and provides structured review feedback")
        );
        assert_eq!(def.triggers.len(), 2);
        assert_eq!(
            def.triggers[0].event.as_deref(),
            Some("git.pre_commit")
        );
        assert_eq!(def.triggers[0].pattern.as_deref(), Some("review:"));
        assert_eq!(def.triggers[1].event.as_deref(), Some("pull_request.opened"));

        assert_eq!(def.io.input_vsa_type, "CodeDiff");
        assert_eq!(def.io.output_vsa_type, "ReviewResult");
        assert_eq!(def.io.input_dim, 4096);
        assert_eq!(def.io.output_dim, 1024);

        let e8 = def.e8_mode.expect("e8_mode should be present");
        assert_eq!(e8.hexagram, 23);
        assert_eq!(
            e8.description.as_deref(),
            Some("风雷益 — 审慎增益")
        );

        assert!((def.quality_threshold - 0.75).abs() < 1e-9);
        assert_eq!(def.dependencies, vec!["syntax_parser", "pattern_matcher"]);
        assert_eq!(def.version_history.len(), 2);
        assert_eq!(def.version_history[0].version, "1.0.0");
        assert_eq!(def.version_history[1].changes, "Added security analysis module");
        assert_eq!(def.vsa_tag.domain, "Self");
        assert_eq!(def.vsa_tag.subdomain, "Skill");
    }

    #[test]
    fn test_scan_skills_directory() {
        let dir = tempfile::tempdir().expect("tempdir should succeed");

        // Create first skill file
        let sub_dir = dir.path().join("code");
        std::fs::create_dir_all(&sub_dir).expect("create subdir should succeed");
        let mut f1 = std::fs::File::create(sub_dir.join("syntax.skill.yaml")).expect("create file");
        f1.write_all(sample_skill_yaml().as_bytes()).expect("write");

        // Create second skill file in root
        let mut f2 = std::fs::File::create(dir.path().join("design.skill.yaml")).expect("create file");
        f2.write_all(
            r#"skill_id: "ui_design_system"
name: "UI Design System"
version: "0.5.0"
description: "Design system generator"

triggers:
  - event: "design.system_update"

io:
  input_vsa_type: "DesignToken"
  output_vsa_type: "ComponentSpec"

quality_threshold: 0.6
"#
            .as_bytes(),
        )
        .expect("write");

        // Create non-skill yaml file (should be ignored)
        let mut f3 = std::fs::File::create(dir.path().join("config.yaml")).expect("create file");
        f3.write_all(b"key: value\n").expect("write");

        let skills = SkillDocLoader::scan_skills(dir.path()).expect("scan should succeed");
        assert_eq!(skills.len(), 2);

        let ids: Vec<&str> = skills.iter().map(|s| s.skill_id.as_str()).collect();
        assert!(ids.contains(&"code_review_syntax_parser"));
        assert!(ids.contains(&"ui_design_system"));
    }

    #[test]
    fn test_scan_skills_missing_field_error() {
        let dir = tempfile::tempdir().expect("tempdir should succeed");
        let mut f = std::fs::File::create(dir.path().join("bad.skill.yaml")).expect("create file");
        f.write_all(
            r#"name: "No ID"
version: "1.0.0"
"#
            .as_bytes(),
        )
        .expect("write");

        let result = SkillDocLoader::scan_skills(dir.path());
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
        match &errors[0] {
            SkillLoadError::MissingField { field, .. } => {
                assert_eq!(field, "skill_id");
            }
            _ => panic!("Expected MissingField error for skill_id"),
        }
    }

    #[test]
    fn test_scan_skills_duplicate_id() {
        let dir = tempfile::tempdir().expect("tempdir should succeed");
        let yaml = sample_skill_yaml();
        let mut f1 = std::fs::File::create(dir.path().join("a.skill.yaml")).expect("create file");
        f1.write_all(yaml.as_bytes()).expect("write");
        let mut f2 = std::fs::File::create(dir.path().join("b.skill.yaml")).expect("create file");
        f2.write_all(yaml.as_bytes()).expect("write");

        let result = SkillDocLoader::scan_skills(dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_scan_skills_nonexistent_dir() {
        let result = SkillDocLoader::scan_skills(Path::new("/nonexistent/path"));
        assert!(result.is_err());
        match &result.unwrap_err()[0] {
            SkillLoadError::Io(msg) => {
                assert!(msg.contains("cannot read directory"));
            }
            _ => panic!("Expected Io error"),
        }
    }

    #[test]
    fn test_skill_definition_matches_event() {
        let def = SkillDefinition {
            skill_id: "test".into(),
            name: "Test".into(),
            version: "1.0".into(),
            description: None,
            triggers: vec![SkillTrigger {
                event: Some("git.pre_commit".into()),
                pattern: None,
            }],
            io: SkillIoSpec {
                input_vsa_type: "A".into(),
                output_vsa_type: "B".into(),
                input_dim: 4096,
                output_dim: 4096,
            },
            e8_mode: None,
            quality_threshold: 0.5,
            dependencies: vec![],
            version_history: vec![],
            vsa_tag: SkillVsaTag::default(),
            source_path: None,
        };

        assert!(def.matches_event("git.pre_commit"));
        assert!(!def.matches_event("git.post_commit"));
    }

    #[test]
    fn test_skill_definition_matches_pattern() {
        let def = SkillDefinition {
            skill_id: "test".into(),
            name: "Test".into(),
            version: "1.0".into(),
            description: None,
            triggers: vec![SkillTrigger {
                event: None,
                pattern: Some("review:".into()),
            }],
            io: SkillIoSpec {
                input_vsa_type: "A".into(),
                output_vsa_type: "B".into(),
                input_dim: 4096,
                output_dim: 4096,
            },
            e8_mode: None,
            quality_threshold: 0.5,
            dependencies: vec![],
            version_history: vec![],
            vsa_tag: SkillVsaTag::default(),
            source_path: None,
        };

        assert!(def.matches_pattern("please review: this code"));
        assert!(!def.matches_pattern("nothing here"));
    }

    #[test]
    fn test_skill_definition_meets_quality() {
        let mut def = SkillDefinition {
            skill_id: "test".into(),
            name: "Test".into(),
            version: "1.0".into(),
            description: None,
            triggers: vec![],
            io: SkillIoSpec {
                input_vsa_type: "A".into(),
                output_vsa_type: "B".into(),
                input_dim: 4096,
                output_dim: 4096,
            },
            e8_mode: None,
            quality_threshold: 0.7,
            dependencies: vec![],
            version_history: vec![],
            vsa_tag: SkillVsaTag::default(),
            source_path: None,
        };

        assert!(def.meets_quality(0.8));
        assert!(!def.meets_quality(0.6));
        assert!(def.meets_quality(0.7));

        def.quality_threshold = 0.0;
        assert!(def.meets_quality(0.0));
    }

    #[test]
    fn test_skill_load_error_display() {
        let err = SkillLoadError::MissingField {
            path: PathBuf::from("/test.skill.yaml"),
            field: "skill_id".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("skill_id"));
        assert!(msg.contains("test.skill.yaml"));
    }

    #[test]
    fn test_skill_doc_loader_new_and_empty() {
        let loader = SkillDocLoader::new();
        assert!(loader.discovered().is_empty());
    }

    #[test]
    fn test_skill_doc_loader_search() {
        let mut loader = SkillDocLoader::new();
        let def = SkillDefinition {
            skill_id: "syntax_parser".into(),
            name: "Syntax Parser".into(),
            version: "1.0".into(),
            description: Some("Parses code syntax".into()),
            triggers: vec![],
            io: SkillIoSpec::default(),
            e8_mode: None,
            quality_threshold: 0.5,
            dependencies: vec![],
            version_history: vec![],
            vsa_tag: SkillVsaTag::default(),
            source_path: None,
        };
        loader
            .discovered
            .insert(def.skill_id.clone(), def);

        let results = loader.search("syntax");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].skill_id, "syntax_parser");

        let results = loader.search("code");
        assert_eq!(results.len(), 1);

        let results = loader.search("nonexistent");
        assert!(results.is_empty());
    }

    #[test]
    fn test_e8_mode_invalid_hexagram() {
        let dir = tempfile::tempdir().expect("tempdir should succeed");
        let mut f = std::fs::File::create(dir.path().join("bad_e8.skill.yaml")).expect("create file");
        f.write_all(
            r#"skill_id: "bad_e8"
name: "Bad E8"
version: "1.0.0"
io:
  input_vsa_type: "A"
  output_vsa_type: "B"
e8_mode:
  hexagram: 99
"#
            .as_bytes(),
        )
        .expect("write");

        let result = SkillDocLoader::scan_skills(dir.path());
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|e| matches!(e, SkillLoadError::InvalidValue { field, .. } if field == "e8_mode.hexagram")));
    }

    #[test]
    fn test_quality_threshold_out_of_range() {
        let dir = tempfile::tempdir().expect("tempdir should succeed");
        let mut f =
            std::fs::File::create(dir.path().join("bad_qt.skill.yaml")).expect("create file");
        f.write_all(
            r#"skill_id: "bad_qt"
name: "Bad QT"
version: "1.0.0"
io:
  input_vsa_type: "A"
  output_vsa_type: "B"
quality_threshold: 1.5
"#
            .as_bytes(),
        )
        .expect("write");

        let result = SkillDocLoader::scan_skills(dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_vsa_tag_domain_validation() {
        let dir = tempfile::tempdir().expect("tempdir should succeed");
        let mut f =
            std::fs::File::create(dir.path().join("bad_tag.skill.yaml")).expect("create file");
        f.write_all(
            r#"skill_id: "bad_tag"
name: "Bad Tag"
version: "1.0.0"
io:
  input_vsa_type: "A"
  output_vsa_type: "B"
vsa_tag:
  domain: "InvalidDomain"
"#
            .as_bytes(),
        )
        .expect("write");

        let result = SkillDocLoader::scan_skills(dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_minimal_skill_yaml() {
        let dir = tempfile::tempdir().expect("tempdir should succeed");
        let mut f =
            std::fs::File::create(dir.path().join("minimal.skill.yaml")).expect("create file");
        f.write_all(
            r#"skill_id: "minimal"
name: "Minimal Skill"
version: "0.1.0"
io:
  input_vsa_type: "Text"
  output_vsa_type: "Text"
"#
            .as_bytes(),
        )
        .expect("write");

        let skills =
            SkillDocLoader::scan_skills(dir.path()).expect("minimal skill should parse");
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].skill_id, "minimal");
        assert_eq!(skills[0].dependencies.len(), 0);
        assert_eq!(skills[0].version_history.len(), 0);
        assert!((skills[0].quality_threshold - 0.7).abs() < 1e-9);
        assert_eq!(skills[0].io.input_dim, 4096);
        assert_eq!(skills[0].io.output_dim, 4096);
    }

    #[test]
    fn test_skill_definition_vsa_tag_default() {
        let tag = SkillVsaTag::default();
        assert_eq!(tag.domain, "Self");
        assert_eq!(tag.subdomain, "Skill");
    }

    #[test]
    fn test_yaml_stress_trailing_comment() {
        // Ensure trailing comments don't break parsing
        let yaml = r#"skill_id: "test"  # this is a comment
name: "Test Name"  # another comment
version: "1.0.0"  # version comment
io:
  input_vsa_type: "A"  # io comment
  output_vsa_type: "B"
"#;
        let dir = tempfile::tempdir().expect("tempdir should succeed");
        let mut f = std::fs::File::create(dir.path().join("comment.skill.yaml")).expect("create file");
        f.write_all(yaml.as_bytes()).expect("write");

        let skills = SkillDocLoader::scan_skills(dir.path()).expect("comment yaml should parse");
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].skill_id, "test");
        assert_eq!(skills[0].name, "Test Name");
    }
}
