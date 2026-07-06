use std::path::Path;

#[derive(Debug, Clone)]
pub struct CodeSearchResult {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub content: String,
}

#[derive(Default)]
pub struct CodeSearchEngine;

impl CodeSearchEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn search(query: &str, path: &Path) -> Vec<CodeSearchResult> {
        let output = std::process::Command::new("rg")
            .arg("--line-number")
            .arg("--column")
            .arg("--color")
            .arg("never")
            .arg(query)
            .arg(path)
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                stdout
                    .lines()
                    .filter_map(|line| {
                        let parts: Vec<&str> = line.splitn(4, ':').collect();
                        if parts.len() >= 3 {
                            Some(CodeSearchResult {
                                file: parts[0].to_string(),
                                line: parts[1].parse().unwrap_or(0),
                                column: parts[2].parse().unwrap_or(0),
                                content: parts.get(3).unwrap_or(&"").to_string(),
                            })
                        } else {
                            None
                        }
                    })
                    .collect()
            }
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_code_search_engine_creation() {
        let _engine = CodeSearchEngine::new();
    }

    #[test]
    #[ignore = "flaky: depends on ripgrep installation and filesystem state"]
    fn test_search_returns_results() {
        let dir = std::env::temp_dir().join("neotrix_code_search_test");
        let _ = std::fs::create_dir_all(&dir);
        let file_path = dir.join("test.rs");
        let mut file = std::fs::File::create(&file_path).unwrap();
        writeln!(file, "fn hello() {{}}").unwrap();
        writeln!(file, "// test comment").unwrap();
        writeln!(file, "fn world() {{}}").unwrap();

        let results = CodeSearchEngine::search("hello", &dir);
        assert!(results.len() >= 1);
        assert!(results.iter().any(|r| r.content.contains("hello")));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
