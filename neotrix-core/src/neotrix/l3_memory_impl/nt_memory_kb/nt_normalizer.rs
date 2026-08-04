//! nt_normalizer — NeoTrix Knowledge Base Text Normalization Module
//!
//! 忠实移植自 `scripts/nt_normalizer.py`。基于 2024-2026 知识图谱构建、实体解析与
//! 数据标准化文献实践。所有函数与 Python 数据结构保持一致，输出逐字节对齐。

use html_escape::decode_html_entities;
use regex::Regex;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::OnceLock;
use unicode_normalization::UnicodeNormalization;

// ── 惰性正则 ─────────────────────────────────────────────────────
// Markdown 剥离规则表 (pattern, replacement)。
// - 顺序即 Python `re.sub` 的调用顺序，必须保持一致。
// - `(?m)` = `re.MULTILINE`，`(?s)` = `re.DOTALL`。
// - 由 OnceLock 在首次调用时编译一次，避免每次重建 `Regex`。
const MD_RULES: &[(&str, &str)] = &[
    (r"!\[.*?\]\(.*?\)", ""),                // 图片
    (r"\[([^\]]*)\]\(.*?\)", "$1"),          // 链接 -> 保留文本
    (r"(?s)```[\s\S]*?```", ""),             // 代码围栏
    (r"`([^`]+)`", "$1"),                    // 行内代码
    (r"\*\*([^*]+)\*\*", "$1"),              // 粗体
    (r"__([^_]+)__", "$1"),                  // 粗体 (替代)
    (r"\*([^*]+)\*", "$1"),                  // 斜体
    (r"_([^_]+)_", "$1"),                    // 斜体 (替代)
    (r"(?m)^[#]+ ", ""),                     // 标题
    (r"(?m)^[>\|] ", ""),                    // 引用/表格
    (r"(?m)^[-*+]\s+", ""),                  // 列表项
    (r"(?m)^\d+\.\s+", ""),                  // 有序列表
];

fn md_rules() -> &'static Vec<(Regex, &'static str)> {
    static RULES: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();
    RULES.get_or_init(|| {
        MD_RULES
            .iter()
            .map(|(pat, rep)| (Regex::new(pat).expect("静态 Markdown 正则必须合法"), *rep))
            .collect()
    })
}

/// 语言规范名映射表 (Python `LANG_NORM_MAP`)。
/// 将大小写/别名变体 → 规范形式，供 FTS5 精确匹配与图谱聚合。
fn lang_map() -> &'static HashMap<&'static str, &'static str> {
    static MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    MAP.get_or_init(|| {
        [
            ("javascript", "JavaScript"),
            ("typescript", "TypeScript"),
            ("python", "Python"),
            ("python3", "Python"),
            ("go", "Go"),
            ("golang", "Go"),
            ("rust", "Rust"),
            ("rs", "Rust"),
            ("c", "C"),
            ("c++", "C++"),
            ("cpp", "C++"),
            ("cplusplus", "C++"),
            ("c#", "C#"),
            ("csharp", "C#"),
            ("java", "Java"),
            ("kotlin", "Kotlin"),
            ("swift", "Swift"),
            ("objective-c", "Objective-C"),
            ("objc", "Objective-C"),
            ("dart", "Dart"),
            ("scala", "Scala"),
            ("ruby", "Ruby"),
            ("rb", "Ruby"),
            ("php", "PHP"),
            ("perl", "Perl"),
            ("lua", "Lua"),
            ("haskell", "Haskell"),
            ("clojure", "Clojure"),
            ("elixir", "Elixir"),
            ("erlang", "Erlang"),
            ("shell", "Shell"),
            ("bash", "Shell"),
            ("zsh", "Shell"),
            ("sh", "Shell"),
            ("markdown", "Markdown"),
            ("md", "Markdown"),
            ("html", "HTML"),
            ("css", "CSS"),
            ("sass", "SCSS"),
            ("less", "Less"),
            ("solidity", "Solidity"),
            ("vyper", "Vyper"),
            ("dockerfile", "Dockerfile"),
            ("makefile", "Makefile"),
            ("cmake", "CMake"),
            ("jupyter notebook", "Jupyter"),
            ("jupyter", "Jupyter"),
            ("tex", "LaTeX"),
            ("latex", "LaTeX"),
            ("vue", "Vue"),
            ("jsx", "JSX"),
            ("tsx", "TSX"),
            ("svelte", "Svelte"),
            ("rust-lang/rust", "Rust"),
            ("golang/go", "Go"),
            ("python/cpython", "Python"),
            ("microsoft/typescript", "TypeScript"),
            ("apple/swift", "Swift"),
            ("microsoft/visual-studio-code", "TypeScript"),
        ]
        .into_iter()
        .collect()
    })
}

/// 置信度加权语言签名 (Python `_LANG_SIGNATURES`)。weight=0 的条目用于占位跳过。
const LANG_SIGNATURES: &[(&str, &str, f64)] = &[
    (r"```(?:python|py)\b", "Python", 0.9),
    (r"```(?:javascript|js)\b", "JavaScript", 0.9),
    (r"```typescript\b", "TypeScript", 0.9),
    (r"```(?:rust|rs)\b", "Rust", 0.9),
    (r"```go\b", "Go", 0.9),
    (r"```(?:c|cpp|c\+\+|cxx)\b", "C++", 0.9),
    (r"```c#|```csharp\b", "C#", 0.9),
    (r"```java\b", "Java", 0.9),
    (r"```(?:kotlin|kt)\b", "Kotlin", 0.9),
    (r"```swift\b", "Swift", 0.9),
    (r"```(?:ruby|rb)\b", "Ruby", 0.9),
    (r"```php\b", "PHP", 0.9),
    (r"```scala\b", "Scala", 0.9),
    (r"```dart\b", "Dart", 0.9),
    (r"```lua\b", "Lua", 0.9),
    (r"```haskell\b", "Haskell", 0.9),
    (r"```(?:shell|bash|zsh|sh)\b", "Shell", 0.9),
    (r"```sql\b", "SQL", 0.9),
    (r"```r\b", "R", 0.9),
    (r"```(?:matlab|octave)\b", "MATLAB", 0.9),
    (r"```julia\b", "Julia", 0.9),
    (r"built with python|python library|python package", "Python", 0.6),
    (r"built with rust|rust library|rust crate", "Rust", 0.6),
    (r"built with go|golang library", "Go", 0.6),
    (r"built with typescript", "TypeScript", 0.6),
    (r"built with (?:javascript|js)", "JavaScript", 0.6),
    (r"built with (?:react|vue|angular)", "JavaScript", 0.6),
];

fn lang_signatures() -> &'static Vec<(Regex, &'static str, f64)> {
    static SIGS: OnceLock<Vec<(Regex, &'static str, f64)>> = OnceLock::new();
    SIGS.get_or_init(|| {
        LANG_SIGNATURES
            .iter()
            .filter(|(_, lang, _)| !lang.is_empty())
            .map(|(pat, lang, w)| (Regex::new(pat).expect("静态语言签名必须合法"), *lang, *w))
            .collect()
    })
}

// ── 文本规范化 ────────────────────────────────────────────────────

/// 三阶段文本归一化管线:
///   Stage 1: Unicode NFKC 规范化 (规范 + 兼容分解，统一全半角)
///   Stage 2: HTML 实体解码 (`&amp;` `&lt;` 等)
///   Stage 3: 空白折叠 + 去首尾
pub fn normalize_text(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let nfkc: String = text.nfkc().collect();
    let decoded = decode_html_entities(&nfkc);
    decoded.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// 剥离 Markdown 语法标记，得到纯文本用于 FTS5 索引。
/// 移除: 图片、链接(保留文本)、代码围栏、行内代码、粗斜体、标题、引用、表格、列表。
pub fn strip_markdown(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let mut out = text.to_owned();
    for (regex, replacement) in md_rules() {
        out = regex.replace_all(&out, *replacement).into_owned();
    }
    normalize_text(&out)
}

/// 提取语义重要章节 (标题 + 紧随其后的段落)。
///
/// 启发式: 已知信息型标题 (overview/features/description 等) 下的内容通常最有用。
/// 文献依据: KG 构建综述 (arXiv, 2023) 指出文档结构 (标题) 可作为内容蒸馏的弱监督。
pub fn extract_key_sections(text: &str, top_k: usize) -> Vec<(String, String)> {
    const RELEVANT: &[&str] = &[
        "overview",
        "introduction",
        "features",
        "about",
        "description",
        "what is this",
        "getting started",
        "quick start",
        "key features",
        "capabilities",
    ];

    let mut sections: Vec<(String, Vec<String>)> = Vec::new();
    let mut current = String::from("overview");

    for line in text.lines() {
        if let Some(body) = line.strip_prefix("## ") {
            current = body.trim().to_ascii_lowercase();
            sections.push((current.clone(), Vec::new()));
        } else if line.starts_with("# ") {
            continue;
        } else {
            if let Some((_, lines)) = sections.iter_mut().find(|(h, _)| h == &current) {
                lines.push(line.to_owned());
            }
        }
    }

    let mut result = Vec::new();
    for h in RELEVANT.iter().take(top_k) {
        if let Some((_, lines)) = sections.iter().find(|(name, _)| name == h) {
            let content = lines
                .iter()
                .take(20)
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
                .to_owned();
            if content.chars().count() > 50 {
                result.push(((*h).to_string(), content.chars().take(500).collect()));
            }
        }
    }
    result
}

// ── 语言规范 ──────────────────────────────────────────────────────

/// 语言名规范化到标准形式: `"rust"` → `"Rust"`, `"golang"` → `"Go"`。
pub fn normalize_lang(lang: &str) -> String {
    let key = lang.trim().to_ascii_lowercase();
    lang_map()
        .get(key.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| lang.trim().to_owned())
}

// ── 内容指纹 ──────────────────────────────────────────────────────

/// SHA256 内容指纹，取前 32 个十六进制字符，用于跨源去重。
/// 空/平凡内容 (<10 字符) 返回空串，避免误匹配。
pub fn content_fingerprint(content: &str) -> String {
    if content.is_empty() || content.chars().count() < 10 {
        return String::new();
    }
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let digest = hasher.finalize();
    let hex_str = hex::encode(digest);
    hex_str.chars().take(32).collect()
}

// ── 语言检测 ──────────────────────────────────────────────────────

/// 从内容/标题/URL 检测主要编程语言: 置信度加权签名匹配。
/// 1. 代码块围栏标签 (```python → Python, 权重 0.9)
/// 2. 描述关键词 ('built with Rust' → 权重 0.6)
/// 3. 标题命中
///
/// 返回规范语言名；未检测到→ `"en"` (默认 KB 语言)。
pub fn detect_language(text: &str, title: &str) -> String {
    let mut scores: HashMap<&'static str, f64> = HashMap::new();

    for (re, lang, weight) in lang_signatures() {
        let count = re.find_iter(text).count().min(3);
        if count > 0 {
            let e = scores.entry(lang).or_insert(0.0);
            *e += weight * count as f64;
        }
    }

    if !title.is_empty() {
        let lower = title.to_ascii_lowercase();
        for (needle, lang) in [
            ("python", "Python"),
            ("rust", "Rust"),
            ("go-", "Go"),
            ("golang", "Go"),
            ("typescript", "TypeScript"),
            ("ts-", "TypeScript"),
            ("javascript", "JavaScript"),
            ("js-", "JavaScript"),
        ] {
            if lower.contains(needle) {
                let e = scores.entry(lang).or_insert(0.0);
                *e = e.max(0.5);
            }
        }
    }

    scores
        .iter()
        .filter(|(_, &score)| score >= 0.5)
        .max_by(|a, b| a.1.partial_cmp(b.1).expect("非 NaN 分数"))
        .map(|(lang, _)| normalize_lang(lang))
        .unwrap_or_else(|| "en".to_owned())
}

// ── 质量度量 ──────────────────────────────────────────────────────

/// 计算 KB 节点知识质量分 [0, 1]。
///   内容深度: 日志缩放内容长度 (10K 字符封顶)
///   摘要存在: +0.15,  URL 存在: +0.10
///   边连通性: 每条 +0.05 (封顶 +0.30)
pub fn compute_quality_score(
    content_length: usize,
    has_summary: bool,
    has_url: bool,
    edge_count: usize,
) -> f64 {
    let mut score = 0.0;
    if content_length > 0 {
        score += (content_length as f64 / 10_000.0).min(1.0) * 0.45;
    }
    if has_summary {
        score += 0.15;
    }
    if has_url {
        score += 0.10;
    }
    score += (edge_count as f64 * 0.05).min(0.30);
    score.min(1.0)
}

// ── Schema 校验 ───────────────────────────────────────────────────

pub const NODE_TYPES: &[&str] = &[
    "Repository", "Resource", "Concept", "Article", "Insight", "CodeSnippet",
    "Framework", "Organization", "Paper", "Theory", "Tutorial", "Tool",
    "Project", "Book", "Course", "Video", "Audio", "Image", "Dataset",
    "API", "Standard",
];

pub const RELATION_TYPES: &[&str] = &[
    "contains", "related_to", "references", "depends_on", "part_of",
    "implements", "developed_by", "authored_by", "supports", "uses",
    "similar_to", "translates_to",
];

/// 校验并规范化节点类型。
pub fn validate_node_type(t: &str) -> &'static str {
    NODE_TYPES
        .iter()
        .find(|valid| **valid == t)
        .copied()
        .unwrap_or("Concept")
}

/// 校验并规范化关系类型。
pub fn validate_relation_type(r: &str) -> &'static str {
    RELATION_TYPES
        .iter()
        .find(|valid| **valid == r)
        .copied()
        .unwrap_or("references")
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_text_nfkc() {
        assert_eq!(normalize_text("Ｈｅｌｌｏ"), "Hello");
        assert_eq!(normalize_text("ａ＋ｂ＝ｃ"), "a+b=c");
    }

    #[test]
    fn test_normalize_text_html() {
        assert_eq!(normalize_text("a&nbsp;b"), "a b");
        assert_eq!(normalize_text("a<b>c"), "a<b>c");
    }

    #[test]
    fn test_strip_markdown_basic() {
        let plain = strip_markdown("**bold** and *italic* and `code`");
        assert!(plain.contains("bold"));
        assert!(plain.contains("italic"));
        assert!(plain.contains("code"));
        assert!(!plain.contains('*'));
        assert!(!plain.contains('`'));
    }

    #[test]
    fn test_strip_markdown_links() {
        let plain = strip_markdown("[link](http://example.com) and ![img](http://e.com/i.png)");
        assert!(plain.contains("link"));
        assert!(!plain.contains("http://"));
    }

    #[test]
    fn test_strip_markdown_list() {
        let plain = strip_markdown("- item one\n2. item two");
        assert!(plain.contains("item one"));
        assert!(plain.contains("item two"));
        assert!(!plain.contains('2'));
    }

    #[test]
    fn test_extract_key_sections() {
        let text = "# Title\nSkip me\n\n## Overview\n\nLong content here that is certainly\nlonger than fifty characters total";
        let sections = extract_key_sections(text, 5);
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].0, "overview");
    }

    #[test]
    fn test_normalize_lang_canonical() {
        assert_eq!(normalize_lang("rust"), "Rust");
        assert_eq!(normalize_lang("golang"), "Go");
        assert_eq!(normalize_lang("python3"), "Python");
    }

    #[test]
    fn test_content_fingerprint_empty() {
        assert_eq!(content_fingerprint("short"), "");
        assert_eq!(content_fingerprint(""), "");
    }

    #[test]
    fn test_content_fingerprint_first32() {
        let fp = content_fingerprint("the quick brown fox jumps over the lazy dog");
        assert_eq!(fp.len(), 32);
        assert!(fp.chars().all(|c| c.is_ascii_hexdigit()));
        // 与 hashlib.sha256(...).hexdigest()[:32] 逐字节一致
        assert_eq!(fp, "05c6e08f1d9fdafa03147fcb8f82f124");
    }

    #[test]
    fn test_detect_language_from_fence() {
        assert_eq!(detect_language("```rust\nfn main() {}\n```", ""), "Rust");
        assert_eq!(detect_language("```python\nprint(1)\n```", ""), "Python");
    }

    #[test]
    fn test_detect_language_default_en() {
        assert_eq!(detect_language("just some plain prose text here", ""), "en");
    }

    #[test]
    fn test_detect_language_from_title() {
        assert_eq!(detect_language("", "my-golang-tool"), "Go");
    }

    #[test]
    fn test_quality_score_bounds() {
        assert!(compute_quality_score(0, false, false, 0) >= 0.0);
        assert!(compute_quality_score(1_000_000, true, true, 100) <= 1.0);
    }

    #[test]
    fn test_validate_types() {
        assert_eq!(validate_node_type("Paper"), "Paper");
        assert_eq!(validate_node_type("Bogus"), "Concept");
        assert_eq!(validate_relation_type("uses"), "uses");
        assert_eq!(validate_relation_type("bogus"), "references");
    }
}