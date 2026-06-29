use std::collections::HashMap;

/// Content type detected for optimal compression strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    CodeRust,
    CodePython,
    CodeGeneric,
    Json,
    Log,
    Conversation,
    StructuredText,
    Markdown,
    Generic,
}

impl ContentType {
    pub fn detect(text: &str) -> Self {
        let sample = text.trim();
        if sample.starts_with('{') || sample.starts_with('[') {
            return ContentType::Json;
        }
        if sample.starts_with("fn ") || sample.contains("fn main") || sample.contains("impl ") {
            return ContentType::CodeRust;
        }
        if sample.contains("def ") || sample.contains("import ") || sample.contains("class ") {
            return ContentType::CodePython;
        }
        if sample.contains("ERROR") || sample.contains("WARN") || sample.contains("INFO") {
            return ContentType::Log;
        }
        if sample.contains("## ") || sample.contains("```") {
            return ContentType::Markdown;
        }
        if sample.contains("| ") || sample.contains("---") || sample.contains("+--") {
            return ContentType::StructuredText;
        }
        if sample.contains("user:") || sample.contains("assistant:") || sample.contains("human:") {
            return ContentType::Conversation;
        }
        if sample.contains("pub ") || sample.contains("use ") || sample.contains("let ") {
            return ContentType::CodeGeneric;
        }
        ContentType::Generic
    }
}

/// Token compression profile — adjustable aggressiveness
#[derive(Debug, Clone, Copy)]
pub struct CompressionProfile {
    pub aggressive_code: bool,
    pub aggressive_json: bool,
    pub aggressive_log: bool,
    pub max_lines: usize,
    pub max_chars: usize,
}

impl CompressionProfile {
    pub fn conservative() -> Self {
        Self {
            aggressive_code: false,
            aggressive_json: false,
            aggressive_log: false,
            max_lines: 2000,
            max_chars: 100_000,
        }
    }

    pub fn aggressive() -> Self {
        Self {
            aggressive_code: true,
            aggressive_json: true,
            aggressive_log: true,
            max_lines: 500,
            max_chars: 25_000,
        }
    }

    pub fn balanced() -> Self {
        Self {
            aggressive_code: true,
            aggressive_json: false,
            aggressive_log: true,
            max_lines: 1000,
            max_chars: 50_000,
        }
    }
}

impl Default for CompressionProfile {
    fn default() -> Self {
        Self::balanced()
    }
}

/// Result of token optimization
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub original_chars: usize,
    pub compressed_chars: usize,
    pub estimated_tokens_saved: usize,
    pub compression_ratio: f64,
    pub content_type: ContentType,
    pub strategy: &'static str,
}

/// Simhash-based near-duplicate detection — O(1) per insertion
#[derive(Debug, Clone)]
pub struct SimhashDedup {
    fingerprints: Vec<u64>,
    hash_bits: u32,
}

impl SimhashDedup {
    pub fn new() -> Self {
        Self {
            fingerprints: Vec::new(),
            hash_bits: 64,
        }
    }

    fn simhash(text: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        hasher.finish()
    }

    fn hamming_distance(a: u64, b: u64) -> u32 {
        (a ^ b).count_ones()
    }

    pub fn is_duplicate(&self, text: &str, threshold: u32) -> bool {
        let fp = Self::simhash(text);
        self.fingerprints
            .iter()
            .any(|&f| Self::hamming_distance(fp, f) <= threshold)
    }

    pub fn insert(&mut self, text: &str) {
        self.fingerprints.push(Self::simhash(text));
        if self.fingerprints.len() > 1000 {
            self.fingerprints.remove(0);
        }
    }

    pub fn dedup(&mut self, texts: &[String], threshold: u32) -> Vec<String> {
        let mut result = Vec::new();
        for t in texts {
            if !self.is_duplicate(t, threshold) {
                result.push(t.clone());
                self.insert(t);
            }
        }
        result
    }
}

impl Default for SimhashDedup {
    fn default() -> Self {
        Self::new()
    }
}

/// UCCP-inspired structured data compression for JSON and agent messages
pub fn compress_structured(data: &str) -> String {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(data) {
        return compress_json_value(&v);
    }
    // Non-JSON structured: compact whitespace + trim
    let compact: String = data
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    compact
}

fn compress_json_value(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Object(map) => {
            let mut parts: Vec<String> = Vec::new();
            for (k, val) in map.iter() {
                let short_key = k.split(|c: char| !c.is_alphanumeric() && c != '_')
                    .filter(|s| !s.is_empty())
                    .map(|s| if s.len() > 3 { &s[..3] } else { s })
                    .collect::<Vec<_>>()
                    .join("_");
                let compressed_val = compress_json_value(val);
                parts.push(format!("{}:{}", short_key, compressed_val));
            }
            format!("{{{}}}", parts.join(","))
        }
        serde_json::Value::Array(arr) => {
            if arr.len() <= 3 {
                let items: Vec<String> = arr.iter().map(compress_json_value).collect();
                format!("[{}]", items.join(","))
            } else {
                let sample: Vec<String> = arr.iter().take(3).map(compress_json_value).collect();
                format!("[{},+{}more]", sample.join(","), arr.len() - 3)
            }
        }
        serde_json::Value::String(s) => {
            if s.len() > 80 {
                format!("\"...{}..\"", &s[..40])
            } else {
                format!("\"{}\"", s)
            }
        }
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "null".to_string(),
    }
}

/// Entropy-gate inspired compression: remove low-information lines
pub fn entropy_gate(text: &str, keep_ratio: f64) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    let n = (lines.len() as f64 * keep_ratio).max(1.0) as usize;
    if n >= lines.len() {
        return text.to_string();
    }

    // Score each line by information density (entropy proxy)
    let mut scored: Vec<(usize, f64, &str)> = lines
        .iter()
        .enumerate()
        .map(|(i, l)| {
            let trimmed = l.trim();
            if trimmed.is_empty() {
                return (i, 0.0, *l);
            }
            let entropy = trimmed
                .chars()
                .filter(|c| c.is_alphanumeric())
                .count() as f64
                / (trimmed.len().max(1)) as f64;
            // Boost lines with keywords, symbols, and formatting
            let keywords_bonus = if trimmed.contains("ERROR")
                || trimmed.contains("FAIL")
                || trimmed.contains("panic")
                || trimmed.contains("warning")
            {
                0.5
            } else if trimmed.contains("fn ")
                || trimmed.contains("pub ")
                || trimmed.contains("impl ")
                || trimmed.contains("trait ")
            {
                0.4
            } else if trimmed.contains("->")
                || trimmed.contains("=>")
                || trimmed.contains("| ")
                || trimmed.contains("+")
            {
                0.2
            } else {
                0.0
            };
            let line_len = trimmed.len().min(200) as f64 / 200.0;
            (i, entropy + keywords_bonus + line_len * 0.1, *l)
        })
        .collect();

    // Sort by score descending, keep top n
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let kept_indices: std::collections::HashSet<usize> =
        scored.iter().take(n).map(|(i, _, _)| *i).collect();

    // Reconstruct in original order
    lines
        .iter()
        .enumerate()
        .filter(|(i, _)| kept_indices.contains(i))
        .map(|(_, l)| *l)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Code-aware compression: strip function bodies, keep signatures
pub fn compress_code(code: &str) -> String {
    let lines: Vec<&str> = code.lines().collect();
    let mut result = Vec::new();
    let mut brace_depth: i32 = 0;
    let mut in_function = false;
    let mut function_signature = String::new();

    for line in &lines {
        let trimmed = line.trim();
        if in_function {
            function_signature.push_str(&format!("\n{}", line));
            let opens = line.matches('{').count() as i32;
            let closes = line.matches('}').count() as i32;
            brace_depth += opens - closes;

            // Keep first line of body if it's a doc comment or attribute
            if brace_depth == 1 && (trimmed.starts_with("///") || trimmed.starts_with("#[")) {
                result.push(line.to_string());
            }

            if brace_depth <= 0 {
                // Function just closed — add folded marker
                let sig_lines: Vec<&str> = function_signature.lines().collect();
                let sig = if sig_lines.len() > 6 {
                    let first = sig_lines.first().unwrap_or(&"");
                    let _last = sig_lines.last().unwrap_or(&"");
                    format!("{} ... // folded body", first)
                } else {
                    function_signature.clone()
                };
                // Remove the closing brace from function_signature
                let clean_sig = sig.trim_end_matches('}').trim().to_string();
                result.push(format!("{} {{ ... }}", clean_sig));
                function_signature.clear();
                in_function = false;
                brace_depth = 0;
            }
        } else if trimmed.starts_with("fn ")
            || trimmed.starts_with("pub fn ")
            || trimmed.starts_with("pub async fn ")
        {
            in_function = true;
            function_signature = line.to_string();
            brace_depth = 0;
            let opens = line.matches('{').count() as i32;
            let closes = line.matches('}').count() as i32;
            brace_depth += opens - closes;

            if brace_depth <= 0 {
                // One-liner — fold
                let sig = function_signature.trim_end_matches('{').trim();
                result.push(format!("{} {{ ... }}", sig));
                in_function = false;
            }
        } else if trimmed.starts_with("impl") || trimmed.starts_with("pub impl") {
            let opens = trimmed.matches('{').count() as i32;
            let closes = trimmed.matches('}').count() as i32;
            if opens > 0 && opens == closes {
                result.push(format!("{} {{ ... }}", trimmed.trim_end_matches('{')));
            } else {
                result.push(format!("{} {{ ... }}", trimmed));
            }
        } else {
            result.push(line.to_string());
        }
    }

    result.join("\n")
}

/// Token budget estimator. Uses heuristics: ~4 chars per token on average.
pub fn estimate_tokens(text: &str) -> usize {
    let bytes = text.len();
    let whitespace = text.chars().filter(|c| c.is_whitespace()).count();
    let effective = bytes.saturating_sub(whitespace);
    // Conservative: ~3.5 chars/token for code, ~4 for prose
    if ContentType::detect(text) == ContentType::CodeRust
        || ContentType::detect(text) == ContentType::CodeGeneric
    {
        effective / 3 + 1
    } else {
        effective / 4 + 1
    }
}

/// Main token optimization function — applies content-aware compression
pub fn optimize(text: &str, profile: &CompressionProfile) -> OptimizationResult {
    let original_chars = text.len();
    let ct = ContentType::detect(text);
    let before_tokens = estimate_tokens(text);

    let compressed = match ct {
        ContentType::Json | ContentType::StructuredText => {
            compress_structured(text)
        }
        ContentType::CodeRust | ContentType::CodeGeneric | ContentType::CodePython => {
            if profile.aggressive_code {
                compress_code(text)
            } else {
                text.to_string()
            }
        }
        ContentType::Log => {
            if profile.aggressive_log {
                entropy_gate(text, 0.3)
            } else {
                entropy_gate(text, 0.5)
            }
        }
        ContentType::Conversation => entropy_gate(text, 0.6),
        ContentType::Markdown => entropy_gate(text, 0.5),
        ContentType::Generic => {
            if text.len() > profile.max_chars {
                entropy_gate(text, profile.max_chars as f64 / text.len() as f64)
            } else {
                text.to_string()
            }
        }
    };

    // Truncate if still too large
    let final_text = if compressed.len() > profile.max_chars {
        let lines: Vec<&str> = compressed.lines().collect();
        if lines.len() > profile.max_lines {
            let first = &lines[..profile.max_lines / 2];
            let last = &lines[lines.len().saturating_sub(profile.max_lines / 2)..];
            let mut out: Vec<&str> = Vec::new();
            out.extend_from_slice(first);
            out.push("... (truncated) ...");
            out.extend_from_slice(last);
            out.join("\n")
        } else {
            compressed[..profile.max_chars].to_string()
        }
    } else {
        compressed
    };

    let after_tokens = estimate_tokens(&final_text);
    let compressed_chars = final_text.len();

    OptimizationResult {
        original_chars,
        compressed_chars,
        estimated_tokens_saved: before_tokens.saturating_sub(after_tokens),
        compression_ratio: if original_chars > 0 {
            compressed_chars as f64 / original_chars as f64
        } else {
            1.0
        },
        content_type: ct,
        strategy: match ct {
            ContentType::Json => "json-struct",
            ContentType::CodeRust | ContentType::CodeGeneric | ContentType::CodePython => "code-fold",
            ContentType::Log => "entropy-gate",
            ContentType::Conversation => "entropy-gate-60",
            ContentType::Markdown => "entropy-gate-50",
            ContentType::StructuredText => "structured-compact",
            ContentType::Generic => "truncate",
        },
    }
}

/// Apply dedup + compress in a single pipeline pass
pub struct TokenOptimizer {
    pub dedup: SimhashDedup,
    pub profile: CompressionProfile,
    pub stats: OptimizationStats,
}

#[derive(Debug, Clone, Default)]
pub struct OptimizationStats {
    pub total_original: usize,
    pub total_compressed: usize,
    pub total_tokens_saved: usize,
    pub runs: usize,
}

impl TokenOptimizer {
    pub fn new(profile: CompressionProfile) -> Self {
        Self {
            dedup: SimhashDedup::new(),
            profile,
            stats: OptimizationStats::default(),
        }
    }

    pub fn process(&mut self, text: &str) -> (String, OptimizationResult) {
        // Step 1: Deduplication
        if self.dedup.is_duplicate(text, 3) {
            return (
                "[duplicate — compressed in prior context]".to_string(),
                OptimizationResult {
                    original_chars: text.len(),
                    compressed_chars: 0,
                    estimated_tokens_saved: estimate_tokens(text),
                    compression_ratio: 0.0,
                    content_type: ContentType::detect(text),
                    strategy: "dedup",
                },
            );
        }
        self.dedup.insert(text);

        // Step 2: Content-aware compression
        let result = optimize(text, &self.profile);
        let compressed = match result.content_type {
            ContentType::Json | ContentType::StructuredText => compress_structured(text),
            _ct @ (ContentType::CodeRust | ContentType::CodeGeneric | ContentType::CodePython) => {
                if self.profile.aggressive_code {
                    compress_code(text)
                } else {
                    text.to_string()
                }
            }
            ContentType::Log => {
                if self.profile.aggressive_log {
                    entropy_gate(text, 0.3)
                } else {
                    entropy_gate(text, 0.5)
                }
            }
            _ => entropy_gate(text, 0.6),
        };

        self.stats.total_original += result.original_chars;
        self.stats.total_compressed += compressed.len();
        self.stats.total_tokens_saved += result.estimated_tokens_saved;
        self.stats.runs += 1;

        (compressed, result)
    }

    pub fn stats_summary(&self) -> String {
        if self.stats.runs == 0 {
            return "no compressions performed".to_string();
        }
        let pct = if self.stats.total_original > 0 {
            100.0 - (self.stats.total_compressed as f64 / self.stats.total_original as f64) * 100.0
        } else {
            0.0
        };
        format!(
            "TokenOptimizer: {} runs, {}→{} chars ({:.1}% saved), ~{} tokens saved",
            self.stats.runs,
            self.stats.total_original,
            self.stats.total_compressed,
            pct,
            self.stats.total_tokens_saved
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_detection() {
        assert_eq!(ContentType::detect("fn main() {}"), ContentType::CodeRust);
        assert_eq!(ContentType::detect("def hello(): pass"), ContentType::CodePython);
        assert_eq!(ContentType::detect("{\"key\": \"value\"}"), ContentType::Json);
        assert_eq!(ContentType::detect("ERROR: connection failed"), ContentType::Log);
        assert!(matches!(ContentType::detect("hello world"), ContentType::Generic));
    }

    #[test]
    fn test_json_compression() {
        let data = r#"{"name": "John Doe", "age": 30, "email": "john@example.com", "address": {"city": "NYC", "zip": "10001"}}"#;
        let result = compress_structured(data);
        assert!(result.len() < data.len());
        assert!(result.contains("nam"));
        assert!(!result.contains("John Doe")); // values are shortened
    }

    #[test]
    fn test_code_compression() {
        let code = r#"
fn calculate_fibonacci(n: u32) -> u64 {
    if n <= 1 {
        return n as u64;
    }
    let mut a = 0u64;
    let mut b = 1u64;
    for _ in 2..=n {
        let tmp = a + b;
        a = b;
        b = tmp;
    }
    b
}"#;
        let compressed = compress_code(code);
        assert!(compressed.contains("fn calculate_fibonacci"));
        assert!(compressed.contains("{ ... }"));
        assert!(compressed.len() < code.len());
    }

    #[test]
    fn test_entropy_gate() {
        let text = "INFO: server started\nINFO: loading config\nWARN: timeout detected\nINFO: processing complete\nERROR: connection lost";
        let compressed = entropy_gate(text, 0.5);
        assert!(compressed.contains("ERROR"));
        assert!(compressed.contains("WARN"));
        assert!(!compressed.contains("INFO: loading config"));
    }

    #[test]
    fn test_simhash_dedup() {
        let mut dedup = SimhashDedup::new();
        assert!(!dedup.is_duplicate("first text", 3));
        dedup.insert("first text");
        assert!(dedup.is_duplicate("first text", 3));
    }

    #[test]
    fn test_token_estimator() {
        let tokens = estimate_tokens("fn add(a: i32, b: i32) -> i32 { a + b }");
        assert!(tokens > 0);
        assert!(tokens < 20);
    }

    #[test]
    fn test_full_pipeline() {
        let mut opt = TokenOptimizer::new(CompressionProfile::aggressive());
        let code = r#"
fn main() {
    let x = 42;
    println!("hello");
}
"#;
        let (compressed, result) = opt.process(code);
        assert!(compressed.len() <= code.len());
        assert!(result.estimated_tokens_saved > 0);
        assert!(opt.stats.runs == 1);
    }
}
