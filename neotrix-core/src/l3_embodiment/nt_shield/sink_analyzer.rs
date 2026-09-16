//! SinkAnalyzer — Identify dangerous API calls (sinks) in binaries/code

use std::collections::HashMap;

/// A dangerous API sink identified in the code
#[derive(Debug, Clone)]
pub struct Sink {
    pub name: String,
    pub category: SinkCategory,
    pub severity: SinkSeverity,
    pub location: String,
    pub parameters: Vec<String>,
}

/// Category of a dangerous sink
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SinkCategory {
    FileIO,
    Network,
    ProcessExecution,
    MemoryOperation,
    Encryption,
    Reflection,
    Injection,
    Persistence,
    PrivilegeEscalation,
    Obfuscation,
}

/// Severity of a sink
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SinkSeverity {
    Low,
    Medium,
    High,
    Critical,
}

pub struct SinkCategoryResult {
    pub category: SinkCategory,
    pub sinks: Vec<Sink>,
    pub count: usize,
}

/// Analyzes code/binaries to identify dangerous API calls (sinks)
pub struct SinkAnalyzer {
    sinks: Vec<Sink>,
    sink_patterns: HashMap<SinkCategory, Vec<String>>,
}

impl SinkAnalyzer {
    /// Create a new SinkAnalyzer with built-in sink patterns
    pub fn new() -> Self {
        let mut sink_patterns = HashMap::new();

        sink_patterns.insert(
            SinkCategory::FileIO,
            vec![
                "File.open".to_string(),
                "fopen".to_string(),
                "open".to_string(),
                "readFile".to_string(),
                "writeFile".to_string(),
                "CreateFile".to_string(),
            ],
        );

        sink_patterns.insert(
            SinkCategory::Network,
            vec![
                "socket".to_string(),
                "connect".to_string(),
                "send".to_string(),
                "recv".to_string(),
                "URL.openConnection".to_string(),
                "HttpClient.send".to_string(),
            ],
        );

        sink_patterns.insert(
            SinkCategory::ProcessExecution,
            vec![
                "Runtime.exec".to_string(),
                "ProcessBuilder.start".to_string(),
                "system".to_string(),
                "exec".to_string(),
                "CreateProcess".to_string(),
            ],
        );

        sink_patterns.insert(
            SinkCategory::MemoryOperation,
            vec![
                "malloc".to_string(),
                "memcpy".to_string(),
                "memmove".to_string(),
                "strcpy".to_string(),
                "sprintf".to_string(),
                "scanf".to_string(),
            ],
        );

        sink_patterns.insert(
            SinkCategory::Encryption,
            vec![
                "encrypt".to_string(),
                "decrypt".to_string(),
                "Cipher.getInstance".to_string(),
                "hash".to_string(),
            ],
        );

        sink_patterns.insert(
            SinkCategory::Reflection,
            vec![
                "Class.forName".to_string(),
                "Method.invoke".to_string(),
                "getMethod".to_string(),
            ],
        );

        sink_patterns.insert(
            SinkCategory::Injection,
            vec![
                "eval".to_string(),
                "exec".to_string(),
                "system".to_string(),
                "Runtime.getRuntime".to_string(),
            ],
        );

        sink_patterns.insert(
            SinkCategory::Persistence,
            vec![
                "Registry.setValue".to_string(),
                "setAutoStart".to_string(),
                "cron".to_string(),
            ],
        );

        sink_patterns.insert(
            SinkCategory::PrivilegeEscalation,
            vec![
                "setuid".to_string(),
                "setgid".to_string(),
                "chmod".to_string(),
                "sudo".to_string(),
            ],
        );

        sink_patterns.insert(
            SinkCategory::Obfuscation,
            vec![
                "obfuscate".to_string(),
                "encode".to_string(),
                "Base64.encode".to_string(),
            ],
        );

        Self {
            sinks: Vec::new(),
            sink_patterns,
        }
    }

    /// Analyze the given code/binary content and identify all sinks
    pub fn analyze(&mut self, code: &str) -> &[Sink] {
        self.sinks.clear();
        self.find_sinks(code);
        &self.sinks
    }

    /// Find all dangerous sinks in the provided code text
    pub fn find_sinks(&mut self, code: &str) -> Vec<&Sink> {
        let mut results = Vec::new();

        for (category, patterns) in &self.sink_patterns {
            for pattern in patterns {
                if code.contains(pattern) {
                    let sink = Sink {
                        name: pattern.clone(),
                        category: *category,
                        severity: self.estimate_severity(category),
                        location: format!("line_unknown"),
                        parameters: vec![],
                    };
                    self.sinks.push(sink);
                }
            }
        }

        for sink in &self.sinks {
            results.push(sink);
        }
        results
    }

    /// Categorize all found sinks by their category
    pub fn categorize(&self) -> Vec<SinkCategoryResult> {
        let mut categories: HashMap<SinkCategory, Vec<Sink>> = HashMap::new();

        for sink in &self.sinks {
            categories
                .entry(sink.category)
                .or_default()
                .push(sink.clone());
        }

        let mut results: Vec<SinkCategoryResult> = categories
            .into_iter()
            .map(|(category, sinks)| {
                let count = sinks.len();
                SinkCategoryResult {
                    category,
                    sinks,
                    count,
                }
            })
            .collect();

        results.sort_by(|a, b| b.count.cmp(&a.count));
        results
    }

    fn estimate_severity(&self, category: &SinkCategory) -> SinkSeverity {
        match category {
            SinkCategory::Injection | SinkCategory::PrivilegeEscalation => SinkSeverity::Critical,
            SinkCategory::ProcessExecution | SinkCategory::Network => SinkSeverity::High,
            SinkCategory::FileIO | SinkCategory::MemoryOperation => SinkSeverity::Medium,
            SinkCategory::Encryption | SinkCategory::Reflection | SinkCategory::Persistence | SinkCategory::Obfuscation => SinkSeverity::Low,
        }
    }
}

impl Default for SinkAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_sinks() {
        let mut analyzer = SinkAnalyzer::new();
        let code = "Runtime.exec(\"ls\"); socket.connect(\"host\");";
        let results = analyzer.analyze(code);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_categorize() {
        let mut analyzer = SinkAnalyzer::new();
        let code = "Runtime.exec(\"ls\");";
        analyzer.analyze(code);
        let categorized = analyzer.categorize();
        assert!(!categorized.is_empty());
    }

    #[test]
    fn test_empty_code() {
        let mut analyzer = SinkAnalyzer::new();
        let results = analyzer.analyze("safe code here");
        assert!(results.is_empty());
    }
}
