//! book_to_skill — 从 `nt_mind_skill_engine.rs` 拆分 (行为零变更).



/// J-Space ship 检查常量 (jspace.py mode_ship) — 内部寄存器记号与状态标记。
/// 出站文本若出现它们, 说明切换 (register switch) 未完成。
pub(crate) const INNER_ONLY: [&str; 11] = [
    "⇒", "⟹", "⟸", "∴", "∵", "⊆", "⊇", "∋", "??", "?!", "💀",
];
pub(crate) const MARKERS: [&str; 6] = [
    "GRRR", "GAAAH", "PHEW", "I see meltdown", "DATA DATA", "I'M DROWNING",
];
pub(crate) static CLAIM_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"(?i)\b(verified|confirmed|validated|tested|proven)\b").expect("claim regex")
});
pub(crate) static COVERAGE_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"(?i)(n\s*≤|n\s*<=|coverage|brute force|differential|exhaustive|including empty)").expect("coverage regex")
});
pub(crate) static REPEAT_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"[.…\-'\s]{20,}").expect("repeat regex")
});

// ────────────────────────────────────────────────────────────────
// P6: BookToSkill (book-to-skill 机制输入侧)
// 书/文档 (PDF/EPUB/DOCX/MD/HTML/RTF/MOBI) → 统一 agent skill 铸造的
// 输入建模与章节→技能候选映射。本层只做"输入归一化 + 章节→技能候选
// 映射"; 产出路径复用既有 SkillEngine/SkillDocEntry, 禁止平行适配器 (R-P42)。
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
pub struct _SkillCandidate {
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
    pub fn discover_candidates(&self, input: &BookInput) -> Vec<_SkillCandidate> {
        self.normalize(input)
            .iter()
            .take(self.max_candidates.max(0))
            .map(|ch| _SkillCandidate {
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

impl crate::l0_substrate::nt_core_self_test::SelfTest for BookToSkill {
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
