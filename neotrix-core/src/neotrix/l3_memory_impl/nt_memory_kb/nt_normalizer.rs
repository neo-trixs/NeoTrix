//! nt_normalizer — NeoTrix Knowledge Base Text Normalization Module
//! 移植自 scripts/nt_normalizer.py (2024-2026 研究文献实践)

use html_escape::decode_html_entities;
use hex;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use unicode_normalization::UnicodeNormalization;

/// 停用词集（中英混合，用于指纹/关键词提取时过滤）
pub static STOP_WORDS: &str = "的了是在与和就都而这于有也一个我你他她它其之为此对从地向到a an the and or but if then else for while do in on at by to of is it as be this that with from";

/// 单例停用词集合（懒加载）
static STOP_SET: std::sync::OnceLock<HashSet<&'static str>> = std::sync::OnceLock::new();

fn get_stop_set() -> &'static HashSet<&'static str> {
    STOP_SET.get_or_init(|| {
        let mut set = HashSet::new();
        for w in STOP_WORDS.split_whitespace() {
            set.insert(w);
        }
        set
    })
}

/// 三阶段文本归一化管线
/// Stage 1: Unicode NFKC 规范化（规范 + 兼容分解）
/// Stage 2: HTML 实体解码 (& < > 等)
/// Stage 3: 空白折叠 + 去首尾
/// 文献依据: NFKC 防止编码碎片化 (Minimalist Innovation, 2026)
pub fn normalize_text(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    // NFKC: 兼容分解 + 再组合，统一全半角、连字符等
    let nfkc: String = text.nfkc().collect();
    // HTML 实体解码
    let decoded = decode_html_entities(&nfkc);
    // 空白折叠
    let collapsed = decoded.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed
}

/// Markdown 语法剥离，得到纯文本用于 FTS5 索引
/// 移除: 图片、链接(保留文本)、代码围栏、行内代码、粗斜体、标题、引用、表格
/// 文献依据: 原始 MD 语法污染 FTS5 分词器
pub fn strip_markdown(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let mut s = text.to_string();

    // 图片 ![alt](url)
    s = regex::Regex::new(r"!\[.*?\]\(.*?\)")
        .unwrap()
        .replace_all(&s, "")
        .to_string();

    // 链接 [text](url) -> 保留 text
    s = regex::Regex::new(r"\[([^\]]*)\]\(.*?\)")
        .unwrap()
        .replace_all(&s, "$1")
        .to_string();

    // 代码围栏 ```...```
    s = regex::Regex::new(r"(?s)```[\s\S]*?```")
        .unwrap()
        .replace_all(&s, "")
        .to_string();

    // 行内代码 `code`
    s = regex::Regex::new(r"`([^`]+)`")
        .unwrap()
        .replace_all(&s, "$1")
        .to_string();

    // 粗体 **text** / __text__
    s = regex::Regex::new(r"\*\*([^*]+)\*\*")
        .unwrap()
        .replace_all(&s, "$1")
        .to_string();
    s = regex::Regex::new(r"__([^_]+)__")
        .unwrap()
        .replace_all(&s, "$1")
        .to_string();

    // 斜体 *text* / _text_ (不贪婪，避免误杀粗体残留)
    s = regex::Regex::new(r"(?<!\*)\*([^*]+)\*(?!\*)")
        .unwrap()
        .replace_all(&s, "$1")
        .to_string();
    s = regex::Regex::new(r"(?<!_)_([^_]+)_(?!_)")
        .unwrap()
        .replace_all(&s, "$1")
        .to_string();

    // 标题 # ## ###
    s = regex::Regex::new(r"(?m)^#{1,6}\s+")
        .unwrap()
        .replace_all(&s, "")
        .to_string();

    // 引用 >
    s = regex::Regex::new(r"(?m)^>\s?")
        .unwrap()
        .replace_all(&s, "")
        .to_string();

    // 水平线 --- ***
    s = regex::Regex::new(r"(?m)^[-*_]{3,}\s*$")
        .unwrap()
        .replace_all(&s, "")
        .to_string();

    // 表格管道符 |...|
    s = regex::Regex::new(r"^\s*\|.*\|\s*$")
        .unwrap()
        .replace_all(&s, "")
        .to_string();

    // 最后再做一次空白折叠
    normalize_text(&s)
}

/// 语言检测：启发式判断中英
pub fn normalize_lang(text: &str) -> &'static str {
    if text.is_empty() {
        return "unknown";
    }
    let chinese_chars = text.chars().filter(|c| ('\u{4e00}'..='\u{9fff}').contains(c)).count();
    let total_chars = text.chars().filter(|c| c.is_alphabetic()).count();
    if total_chars == 0 {
        return "unknown";
    }
    if chinese_chars as f32 / total_chars as f32 > 0.3 {
        "zh"
    } else {
        "en"
    }
}

/// 内容指纹：NFKC + 去停用词 + 小写 + SHA256 前 16 字符
/// 用于去重/相似度检测
pub fn content_fingerprint(text: &str) -> String {
    let norm = normalize_text(text);
    let lower = norm.to_lowercase();
    let words: Vec<&str> = lower
        .split_whitespace()
        .filter(|w| w.len() >= 2 && !get_stop_set().contains(*w))
        .collect();
    let joined = words.join(" ");
    let mut hasher = Sha256::new();
    hasher.update(joined.as_bytes());
    let hash = hasher.finalize();
    hex::encode(hash)[..16].to_string()
}

/// 提取关键章节（标题 + 紧随其后的段落）
pub fn extract_key_sections(text: &str) -> Vec<String> {
    let mut sections = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    let header_re = regex::Regex::new(r"(?m)^#{1,3}\s+(.+)$").unwrap();

    for (i, line) in lines.iter().enumerate() {
        if let Some(caps) = header_re.captures(line) {
            let title = caps.get(1).map(|m| m.as_str()).unwrap_or("").trim();
            if title.is_empty() {
                continue;
            }
            let mut content = String::new();
            for next_line in lines.iter().skip(i + 1) {
                if header_re.is_match(next_line) {
                    break;
                }
                let trimmed = next_line.trim();
                if !trimmed.is_empty() {
                    content.push_str(trimmed);
                    content.push(' ');
                }
            }
            if !content.trim().is_empty() {
                sections.push(format!("{}::{}", title, content.trim()));
            }
        }
    }
    sections
}

/// 检测语言 (更精细版本)
pub fn detect_language(text: &str) -> &'static str {
    normalize_lang(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_text_nfkc() {
        // 全半角统一
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
        let md = "**bold** and *italic* and `code`";
        let plain = strip_markdown(md);
        assert!(plain.contains("bold"));
        assert!(plain.contains("italic"));
        assert!(plain.contains("code"));
        assert!(!plain.contains("*"));
        assert!(!plain.contains("`"));
    }

    #[test]
    fn test_strip_markdown_links() {
        let md = "[link](http://example.com) and ![img](http://example.com/img.png)";
        let plain = strip_markdown(md);
        assert!(plain.contains("link"));
        assert!(!plain.contains("http://"));
    }

    #[test]
    fn test_fingerprint_stable() {
        let fp1 = content_fingerprint("The quick brown fox");
        let fp2 = content_fingerprint("the quick brown fox");
        assert_eq!(fp1, fp2, "指纹应大小写不敏感");
    }

    #[test]
    fn test_fingerprint_stops_filtered() {
        let fp1 = content_fingerprint("the quick brown fox");
        let fp2 = content_fingerprint("quick brown fox");
        assert_eq!(fp1, fp2, "停用词应被过滤");
    }

    #[test]
    fn test_extract_key_sections() {
        let text = "# Title\n\nContent here\n\n## Sub\n\nMore content";
        let sections = extract_key_sections(text);
        assert_eq!(sections.len(), 2);
        assert!(sections[0].starts_with("Title::"));
        assert!(sections[1].starts_with("Sub::"));
    }

    #[test]
    fn test_lang_detection() {
        assert_eq!(normalize_lang("Hello world"), "en");
        assert_eq!(normalize_lang("你好世界"), "zh");
        assert_eq!(normalize_lang("Hello 你好"), "zh");
    }
}