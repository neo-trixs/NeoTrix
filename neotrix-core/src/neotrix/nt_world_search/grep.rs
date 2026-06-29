use aho_corasick::AhoCorasick;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum SearchMode {
    Plain,
    Regex,
    Multi,
    Fuzzy,
}

#[derive(Debug, Clone)]
pub struct Match {
    pub path: String,
    pub line: usize,
    pub content: String,
    pub is_definition: bool,
}

const DEFINITION_PREFIXES: &[&str] = &[
    "fn ",
    "pub fn ",
    "pub async fn ",
    "async fn ",
    "struct ",
    "pub struct ",
    "enum ",
    "pub enum ",
    "trait ",
    "pub trait ",
    "impl ",
    "pub impl ",
    "def ",
    "class ",
    "function ",
    "interface ",
    "type ",
    "pub type ",
    "const ",
    "pub const ",
    "static ",
    "pub static ",
    "macro_rules! ",
    "defmodule ",
    "defprotocol ",
    "defimpl ",
    "func ",
    "func ",
    "defn ",
    "defn- ",
];

fn is_def_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    DEFINITION_PREFIXES.iter().any(|p| trimmed.starts_with(p))
}

fn detect_mode(pattern: &str) -> SearchMode {
    let trimmed = pattern.trim();
    if trimmed.contains(' ')
        && !trimmed.contains(|c: char| c.is_ascii_punctuation() && c != '_' && c != '.')
    {
        let tokens: Vec<&str> = trimmed.split_whitespace().filter(|t| t.len() > 2).collect();
        if tokens.len() > 1 {
            return SearchMode::Multi;
        }
    }
    let has_regex_chars = trimmed.contains(|c: char| {
        matches!(
            c,
            '*' | '+' | '?' | '^' | '$' | '[' | ']' | '(' | ')' | '|' | '\\'
        )
    });
    if has_regex_chars {
        return SearchMode::Regex;
    }
    SearchMode::Plain
}

fn is_smart_case(pattern: &str) -> bool {
    pattern.chars().all(|c| !c.is_uppercase())
}

pub fn search_file_content(path: &Path, pattern: &str) -> Vec<Match> {
    let mode = detect_mode(pattern);
    search_file_content_with_mode(path, pattern, &mode)
}

pub fn search_file_content_with_mode(path: &Path, pattern: &str, mode: &SearchMode) -> Vec<Match> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let smart_case = is_smart_case(pattern);
    let pattern_lower = pattern.to_lowercase();
    let mut results = Vec::new();

    match mode {
        SearchMode::Plain => {
            for (i, line) in content.lines().enumerate() {
                let hit = if smart_case {
                    line.to_lowercase().contains(&pattern_lower)
                } else {
                    line.contains(pattern)
                };
                if hit {
                    results.push(Match {
                        path: path.display().to_string(),
                        line: i + 1,
                        content: line.to_string(),
                        is_definition: is_def_line(line),
                    });
                }
            }
        }
        SearchMode::Regex => {
            let re = match regex::Regex::new(pattern) {
                Ok(r) => r,
                Err(_) => return vec![],
            };
            for (i, line) in content.lines().enumerate() {
                if re.is_match(line) {
                    results.push(Match {
                        path: path.display().to_string(),
                        line: i + 1,
                        content: line.to_string(),
                        is_definition: is_def_line(line),
                    });
                }
            }
        }
        SearchMode::Multi => {
            let tokens: Vec<&str> = pattern.split_whitespace().filter(|t| t.len() > 1).collect();
            let ac = match AhoCorasick::new(&tokens) {
                Ok(ac) => ac,
                Err(_) => return vec![],
            };
            for (i, line) in content.lines().enumerate() {
                if ac.is_match(line) {
                    results.push(Match {
                        path: path.display().to_string(),
                        line: i + 1,
                        content: line.to_string(),
                        is_definition: is_def_line(line),
                    });
                }
            }
        }
        SearchMode::Fuzzy => {
            for (i, line) in content.lines().enumerate() {
                let line_lower = line.to_lowercase();
                let pattern_lower = pattern.to_lowercase();
                let dist = strsim::levenshtein(&pattern_lower, &line_lower);
                let max_dist = (pattern_lower.len() as f64 * 0.4).ceil() as usize;
                if dist <= max_dist.max(2) {
                    results.push(Match {
                        path: path.display().to_string(),
                        line: i + 1,
                        content: line.to_string(),
                        is_definition: is_def_line(line),
                    });
                }
            }
        }
    }

    results
}

pub fn format_results(results: &[Match]) -> String {
    if results.is_empty() {
        return "(no matches)".to_string();
    }
    let mut out = String::new();
    for m in results {
        let def_tag = if m.is_definition { " [def]" } else { "" };
        out.push_str(&format!("{}:{}{}\n", m.path, m.line, def_tag));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_file(content: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("neotrix_grep_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join(format!("test_{}.rs", fastrand::u32(..)));
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "{}", content).unwrap();
        path
    }

    #[test]
    fn test_plain_grep() {
        let p = temp_file("hello world\nfoo bar\nhello again");
        let r = search_file_content(&p, "hello");
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn test_regex_grep() {
        let p = temp_file("abc123\ndef456\nxyz789");
        let r = search_file_content(&p, r"\d+");
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn test_multi_pattern_grep() {
        let p = temp_file("apple\nbanana\ncherry\ndate");
        let r = search_file_content(&p, "apple banana");
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn test_fuzzy_fallback() {
        let p = temp_file("functionality overload");
        let r = search_file_content_with_mode(&p, "functinality", &SearchMode::Fuzzy);
        assert!(!r.is_empty());
    }

    #[test]
    fn test_definition_classifier() {
        let p = temp_file("fn test() {}\nlet x = 1;\npub struct Foo;");
        let r = search_file_content(&p, "fn");
        assert_eq!(r.len(), 1);
        assert!(r[0].is_definition);
    }

    #[test]
    fn test_detect_mode() {
        assert_eq!(detect_mode("hello world"), SearchMode::Multi);
        assert_eq!(detect_mode("hello"), SearchMode::Plain);
        assert_eq!(detect_mode(r"fn\s+\w+"), SearchMode::Regex);
        assert_eq!(detect_mode("apple banana cherry"), SearchMode::Multi);
    }

    #[test]
    fn test_smart_case() {
        assert!(is_smart_case("hello"));
        assert!(!is_smart_case("Hello"));
    }
}
