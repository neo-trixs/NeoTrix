use std::path::Path;
use std::time::Instant;

use super::grep::{format_results, search_file_content_with_mode, Match, SearchMode};

pub struct SearchQuery<'a> {
    pub pattern: &'a str,
    pub path: &'a Path,
    pub prefer_exact: bool,
    pub fuzzy_enabled: bool,
    pub max_results: usize,
}

impl<'a> Default for SearchQuery<'a> {
    fn default() -> Self {
        Self {
            pattern: "",
            path: Path::new("."),
            prefer_exact: true,
            fuzzy_enabled: true,
            max_results: 1000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchReport {
    pub mode_used: SearchMode,
    pub total_matches: usize,
    pub elapsed_ms: u64,
    pub had_fallback: bool,
    pub fallback_chain: Vec<SearchMode>,
}

pub struct FileSearchEngine;

impl FileSearchEngine {
    pub fn search(query: &SearchQuery) -> (Vec<Match>, SearchReport) {
        let start = Instant::now();
        let mut chain = Vec::new();

        let mode_order = if query.prefer_exact {
            vec![SearchMode::Plain, SearchMode::Regex, SearchMode::Multi]
        } else {
            vec![SearchMode::Regex, SearchMode::Multi, SearchMode::Plain]
        };

        // Phase 1: try exact modes
        for mode in &mode_order {
            chain.push(mode.clone());
            let results =
                Self::search_with_engine(query.path, query.pattern, mode, query.max_results);
            let n = results.len();
            if n > 0 {
                return (
                    results,
                    SearchReport {
                        mode_used: mode.clone(),
                        total_matches: n,
                        elapsed_ms: start.elapsed().as_millis() as u64,
                        had_fallback: false,
                        fallback_chain: chain,
                    },
                );
            }
        }

        // Phase 2: try multi-pattern (second pass with auto-detect)
        let multi_mode = SearchMode::Multi;
        chain.push(multi_mode.clone());
        let results =
            Self::search_with_engine(query.path, query.pattern, &multi_mode, query.max_results);
        let n = results.len();
        if n > 0 {
            return (
                results,
                SearchReport {
                    mode_used: multi_mode,
                    total_matches: n,
                    elapsed_ms: start.elapsed().as_millis() as u64,
                    had_fallback: false,
                    fallback_chain: chain,
                },
            );
        }

        // Phase 3: fuzzy fallback (if enabled)
        if query.fuzzy_enabled {
            let fuzzy_mode = SearchMode::Fuzzy;
            chain.push(fuzzy_mode.clone());
            let results =
                Self::search_with_engine(query.path, query.pattern, &fuzzy_mode, query.max_results);
            let n = results.len();
            return (
                results,
                SearchReport {
                    mode_used: fuzzy_mode,
                    total_matches: n,
                    elapsed_ms: start.elapsed().as_millis() as u64,
                    had_fallback: n > 0,
                    fallback_chain: chain,
                },
            );
        }

        (
            vec![],
            SearchReport {
                mode_used: SearchMode::Plain,
                total_matches: 0,
                elapsed_ms: start.elapsed().as_millis() as u64,
                had_fallback: false,
                fallback_chain: chain,
            },
        )
    }

    fn search_with_engine(
        path: &Path,
        pattern: &str,
        mode: &SearchMode,
        max_results: usize,
    ) -> Vec<Match> {
        let mut all = Vec::new();
        if path.is_file() {
            all = search_file_content_with_mode(path, pattern, mode);
        } else if path.is_dir() {
            for entry in walkdir::WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if !entry.file_type().is_file() {
                    continue;
                }
                let mut r = search_file_content_with_mode(entry.path(), pattern, mode);
                all.append(&mut r);
                if all.len() >= max_results {
                    break;
                }
            }
        }
        all.truncate(max_results);
        all
    }

    pub fn search_formatted(query: &SearchQuery) -> (String, SearchReport) {
        let (results, report) = Self::search(query);
        if results.is_empty() {
            return (String::new(), report);
        }
        (format_results(&results), report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_file(content: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("neotrix_engine_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join(format!("engine_{}.rs", fastrand::u32(..)));
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "{}", content).unwrap();
        path
    }

    #[test]
    fn test_engine_plain_mode() {
        let p = temp_file("hello world\nfn test() {}");
        let q = SearchQuery {
            pattern: "hello",
            path: &p,
            ..Default::default()
        };
        let (r, report) = FileSearchEngine::search(&q);
        assert!(!r.is_empty());
        assert_eq!(report.mode_used, SearchMode::Plain);
        assert!(!report.had_fallback);
    }

    #[test]
    fn test_engine_fallback_to_fuzzy() {
        let p = temp_file("functionality check");
        let q = SearchQuery {
            pattern: "functinality",
            path: &p,
            fuzzy_enabled: true,
            prefer_exact: true,
            ..Default::default()
        };
        let (r, report) = FileSearchEngine::search(&q);
        assert!(!r.is_empty(), "should find via fuzzy fallback");
        assert_eq!(report.mode_used, SearchMode::Fuzzy);
        assert!(report.had_fallback);
    }

    #[test]
    fn test_engine_no_fallback_when_disabled() {
        let p = temp_file("functionality check");
        let q = SearchQuery {
            pattern: "functinality",
            path: &p,
            fuzzy_enabled: false,
            prefer_exact: true,
            ..Default::default()
        };
        let (r, _) = FileSearchEngine::search(&q);
        assert!(r.is_empty(), "should not find without fuzzy");
    }

    #[test]
    fn test_engine_empty_pattern() {
        let p = temp_file("hello");
        let q = SearchQuery {
            pattern: "",
            path: &p,
            ..Default::default()
        };
        let (r, _) = FileSearchEngine::search(&q);
        assert!(r.is_empty());
    }
}
