use std::collections::VecDeque;

use crate::core::nt_core_traits::ToolExecutor;

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum DetectedIntent {
    WebSearch(String),
    WebFetch(String),
    FileRead(String),
    FileWrite(String, String),
    FileEdit(String, String, String),
    Bash(String),
    Glob(String),
    Grep(String, String),
    Reasoning(String),
    Greeting,
    Status,
    Translate(String, String), // (text, target_language_code)
    Unknown(String),
}

impl DetectedIntent {
    pub fn label(&self) -> &str {
        match self {
            DetectedIntent::WebSearch(_) => "web_search",
            DetectedIntent::WebFetch(_) => "web_fetch",
            DetectedIntent::FileRead(_) => "file_read",
            DetectedIntent::FileWrite(_, _) => "file_write",
            DetectedIntent::FileEdit(_, _, _) => "file_edit",
            DetectedIntent::Bash(_) => "bash",
            DetectedIntent::Glob(_) => "glob",
            DetectedIntent::Grep(_, _) => "grep",
            DetectedIntent::Reasoning(_) => "reasoning",
            DetectedIntent::Greeting => "greeting",
            DetectedIntent::Status => "status",
            DetectedIntent::Translate(_, _) => "translate",
            DetectedIntent::Unknown(_) => "unknown",
        }
    }

    pub fn input_text(&self) -> &str {
        match self {
            DetectedIntent::WebSearch(s) => s,
            DetectedIntent::WebFetch(s) => s,
            DetectedIntent::FileRead(s) => s,
            DetectedIntent::FileWrite(s, _) => s,
            DetectedIntent::FileEdit(s, _, _) => s,
            DetectedIntent::Bash(s) => s,
            DetectedIntent::Glob(s) => s,
            DetectedIntent::Grep(s, _) => s,
            DetectedIntent::Reasoning(s) => s,
            DetectedIntent::Greeting => "greeting",
            DetectedIntent::Status => "status",
            DetectedIntent::Translate(s, _) => s,
            DetectedIntent::Unknown(s) => s,
        }
    }
}

pub struct ToolOrchestrator {
    execution_history: VecDeque<String>,
    max_history: usize,
    executor: Box<dyn ToolExecutor>,
}

impl ToolOrchestrator {
    pub fn new(executor: Box<dyn ToolExecutor>) -> Self {
        Self {
            execution_history: VecDeque::new(),
            max_history: 50,
            executor,
        }
    }

    pub fn detect_intent(&self, input: &str) -> DetectedIntent {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return DetectedIntent::Unknown(String::new());
        }

        let lower = trimmed.to_lowercase();

        let greeting_keywords = [
            "hello",
            "hi ",
            "hey",
            "greetings",
            "good morning",
            "good afternoon",
            "good evening",
        ];
        if greeting_keywords.iter().any(|k| lower.starts_with(k)) {
            return DetectedIntent::Greeting;
        }

        if lower == "status" || lower == "stats" || lower == "what's your status" {
            return DetectedIntent::Status;
        }

        if lower.starts_with("search ")
            || lower.starts_with("find ")
            || lower.starts_with("look up ")
        {
            let query = trimmed
                .strip_prefix("search ")
                .or_else(|| trimmed.strip_prefix("find "))
                .or_else(|| trimmed.strip_prefix("look up "))
                .unwrap_or(trimmed)
                .trim();
            return DetectedIntent::WebSearch(query.to_string());
        }

        if lower.starts_with("fetch ")
            || lower.starts_with("read url")
            || lower.starts_with("get url")
        {
            let url = trimmed
                .strip_prefix("fetch ")
                .or_else(|| trimmed.strip_prefix("read url "))
                .or_else(|| trimmed.strip_prefix("get url "))
                .unwrap_or(trimmed)
                .trim();
            return DetectedIntent::WebFetch(url.to_string());
        }

        if lower.starts_with("read ") || lower.starts_with("open ") || lower.starts_with("cat ") {
            let path = trimmed
                .strip_prefix("read ")
                .or_else(|| trimmed.strip_prefix("open "))
                .or_else(|| trimmed.strip_prefix("cat "))
                .unwrap_or(trimmed)
                .trim();
            return DetectedIntent::FileRead(path.to_string());
        }

        if lower.starts_with("write ") || lower.starts_with("save ") {
            let rest = trimmed
                .strip_prefix("write ")
                .or_else(|| trimmed.strip_prefix("save "))
                .unwrap_or(trimmed)
                .trim();
            if let Some((path, content)) = rest.split_once(' ') {
                return DetectedIntent::FileWrite(path.to_string(), content.to_string());
            }
            return DetectedIntent::Unknown(trimmed.to_string());
        }

        if lower.starts_with("run ") || lower.starts_with("execute ") || lower.starts_with("bash ")
        {
            let cmd = trimmed
                .strip_prefix("run ")
                .or_else(|| trimmed.strip_prefix("execute "))
                .or_else(|| trimmed.strip_prefix("bash "))
                .unwrap_or(trimmed)
                .trim();
            return DetectedIntent::Bash(cmd.to_string());
        }

        if lower.starts_with("glob ") {
            let pattern = trimmed.strip_prefix("glob ").unwrap_or(trimmed).trim();
            return DetectedIntent::Glob(pattern.to_string());
        }

        if lower.starts_with("grep ") {
            let rest = trimmed.strip_prefix("grep ").unwrap_or(trimmed).trim();
            if let Some((pattern, path)) = rest.split_once(' ') {
                return DetectedIntent::Grep(pattern.to_string(), path.to_string());
            }
            return DetectedIntent::Grep(String::new(), rest.to_string());
        }

        if lower.starts_with("translate ") {
            return Self::parse_translate_intent(trimmed, &lower);
        }

        DetectedIntent::Reasoning(trimmed.to_string())
    }

    fn parse_translate_intent(input: &str, lower: &str) -> DetectedIntent {
        // Patterns: "translate X to LANG", "translate X into LANG", "translate X"
        let body = lower.strip_prefix("translate ").unwrap_or(lower).trim();

        // Language name → ISO code mapping
        let lang_map: Vec<(&str, &str)> = vec![
            ("chinese", "zh"),
            ("mandarin", "zh"),
            ("english", "en"),
            ("french", "fr"),
            ("french", "fr"),
            ("german", "de"),
            ("spanish", "es"),
            ("japanese", "ja"),
            ("korean", "ko"),
            ("russian", "ru"),
            ("arabic", "ar"),
            ("portuguese", "pt"),
            ("italian", "it"),
            ("dutch", "nl"),
            ("polish", "pl"),
            ("turkish", "tr"),
            ("vietnamese", "vi"),
            ("thai", "th"),
            ("hindi", "hi"),
            ("bengali", "bn"),
        ];

        // Try to extract "to LANG" or "into LANG"
        let to_separators = [" to ", " into "];
        for sep in &to_separators {
            if let Some((text_part, lang_part)) = body.split_once(sep) {
                let lang_code = lang_map
                    .iter()
                    .find(|(name, _)| lang_part.starts_with(name))
                    .map(|(_, code)| *code)
                    .unwrap_or("en");
                return DetectedIntent::Translate(
                    input
                        .strip_prefix("translate ")
                        .map(|s| s.trim_end_matches(lang_part).trim_end_matches(sep.trim()))
                        .unwrap_or(text_part)
                        .to_string(),
                    lang_code.to_string(),
                );
            }
        }

        // No target language specified — assume English
        DetectedIntent::Translate(body.to_string(), "en".to_string())
    }

    pub fn execute(&mut self, intent: &DetectedIntent) -> (String, bool) {
        let start = std::time::Instant::now();

        let result: (String, bool) = match intent {
            DetectedIntent::WebSearch(query) => self.executor.web_search(query),
            DetectedIntent::WebFetch(url) => self.executor.web_fetch(url),
            DetectedIntent::FileRead(path) => self.executor.file_read(path),
            DetectedIntent::FileWrite(path, content) => self.executor.file_write(path, content),
            DetectedIntent::FileEdit(path, old, new) => self.executor.file_edit(path, old, new),
            DetectedIntent::Bash(cmd) => self.executor.bash(cmd),
            DetectedIntent::Glob(pattern) => self.executor.glob(pattern),
            DetectedIntent::Grep(pattern, path) => self.executor.grep(pattern, path),
            DetectedIntent::Reasoning(_)
            | DetectedIntent::Greeting
            | DetectedIntent::Status
            | DetectedIntent::Translate(_, _)
            | DetectedIntent::Unknown(_) => {
                // No tool needed (Translate handled at caller level)
                return (String::new(), true);
            }
        };

        let elapsed = start.elapsed();

        let record = format!(
            "{}:{}ms:success={}",
            intent.label(),
            elapsed.as_millis(),
            result.1
        );
        self.execution_history.push_back(record);
        if self.execution_history.len() > self.max_history {
            self.execution_history.pop_front();
        }

        result
    }

    pub fn format_tool_result(
        &self,
        intent: &DetectedIntent,
        output: &str,
        success: bool,
    ) -> String {
        if !success {
            return format!("Error executing {}: {}", intent.label(), output);
        }
        match intent {
            DetectedIntent::WebSearch(query) => {
                if output.is_empty() || output.starts_with("Search results") {
                    output.to_string()
                } else {
                    format!("Search results for \"{}\":\n{}", query, output)
                }
            }
            DetectedIntent::WebFetch(url) => {
                format!("Fetched {}:\n{}", url, output)
            }
            DetectedIntent::FileRead(path) => {
                format!("File '{}':\n{}", path, output)
            }
            DetectedIntent::FileWrite(path, _) => {
                format!("Written to {}: {}", path, output)
            }
            DetectedIntent::FileEdit(path, _, _) => {
                format!("Edited {}: {}", path, output)
            }
            DetectedIntent::Bash(cmd) => {
                format!("$ {}\n{}", cmd, output)
            }
            DetectedIntent::Glob(pattern) => {
                format!("Glob '{}':\n{}", pattern, output)
            }
            DetectedIntent::Grep(pattern, path) => {
                format!("Grep '{}' in {}:\n{}", pattern, path, output)
            }
            _ => output.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_traits::ToolExecutor;

    struct MockExecutor;
    impl ToolExecutor for MockExecutor {
        fn web_search(&self, _q: &str) -> (String, bool) {
            (String::new(), true)
        }
        fn web_fetch(&self, _u: &str) -> (String, bool) {
            (String::new(), true)
        }
        fn file_read(&self, _p: &str) -> (String, bool) {
            (String::new(), true)
        }
        fn file_write(&self, _p: &str, _c: &str) -> (String, bool) {
            (String::new(), true)
        }
        fn file_edit(&self, _p: &str, _o: &str, _n: &str) -> (String, bool) {
            (String::new(), true)
        }
        fn bash(&self, _c: &str) -> (String, bool) {
            (String::new(), true)
        }
        fn glob(&self, _p: &str) -> (String, bool) {
            (String::new(), true)
        }
        fn grep(&self, _p: &str, _pa: &str) -> (String, bool) {
            (String::new(), true)
        }
    }

    fn mock_orch() -> ToolOrchestrator {
        ToolOrchestrator::new(Box::new(MockExecutor))
    }

    #[test]
    fn test_detect_greeting() {
        let orch = mock_orch();
        assert_eq!(orch.detect_intent("hello"), DetectedIntent::Greeting);
        assert_eq!(orch.detect_intent("Hi there"), DetectedIntent::Greeting);
        assert_eq!(orch.detect_intent("hey"), DetectedIntent::Greeting);
    }

    #[test]
    fn test_detect_search() {
        let orch = mock_orch();
        match orch.detect_intent("search quantum computing") {
            DetectedIntent::WebSearch(q) => assert_eq!(q, "quantum computing"),
            _ => panic!("expected WebSearch"),
        }
        match orch.detect_intent("find Rust tutorials") {
            DetectedIntent::WebSearch(q) => assert_eq!(q, "Rust tutorials"),
            _ => panic!("expected WebSearch"),
        }
    }

    #[test]
    fn test_detect_read() {
        let orch = mock_orch();
        match orch.detect_intent("read /tmp/test.txt") {
            DetectedIntent::FileRead(p) => assert_eq!(p, "/tmp/test.txt"),
            _ => panic!("expected FileRead"),
        }
    }

    #[test]
    fn test_detect_reasoning() {
        let orch = mock_orch();
        match orch.detect_intent("what is the meaning of life?") {
            DetectedIntent::Reasoning(q) => assert!(!q.is_empty()),
            other => panic!("expected Reasoning, got {:?}", other),
        }
    }

    #[test]
    fn test_detect_status() {
        let orch = mock_orch();
        assert_eq!(orch.detect_intent("stats"), DetectedIntent::Status);
        assert_eq!(orch.detect_intent("status"), DetectedIntent::Status);
    }

    #[test]
    fn test_detect_bash() {
        let orch = mock_orch();
        match orch.detect_intent("run ls -la") {
            DetectedIntent::Bash(cmd) => assert_eq!(cmd, "ls -la"),
            _ => panic!("expected Bash"),
        }
    }

    #[test]
    fn test_format_search_result() {
        let orch = mock_orch();
        let intent = DetectedIntent::WebSearch("test".to_string());
        let formatted = orch.format_tool_result(&intent, "1. Result - snippet", true);
        assert!(formatted.contains("test"));
        assert!(formatted.contains("Result"));
    }

    #[test]
    fn test_format_error() {
        let orch = mock_orch();
        let intent = DetectedIntent::FileRead("/nonexistent".to_string());
        let formatted = orch.format_tool_result(&intent, "No such file", false);
        assert!(formatted.contains("Error"));
    }
}
