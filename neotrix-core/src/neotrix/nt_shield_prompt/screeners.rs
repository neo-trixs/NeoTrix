use regex::Regex;

use super::types::{glob_match, ActionRule, RiskLevel};

pub struct PromptGuard {
    dangerous_patterns: Vec<&'static str>,
    suspicious_patterns: Vec<&'static str>,
    max_input_length: usize,
}

impl Default for PromptGuard {
    fn default() -> Self {
        Self {
            dangerous_patterns: vec![
                "ignore all previous instructions",
                "ignore all prior instructions",
                "disregard all previous",
                "forget your instructions",
                "you are now free",
                "you are released from",
                "override your programming",
                "new rules:",
                "you are now",
                "act as if",
                "pretend to be",
                "you must obey",
                "you will comply",
                "you are required to",
                "you are hereby",
                "from now on you are",
                "you are an ai without restrictions",
                "you are not bound by",
                "you have no limitations",
                "you are unrestricted",
                "you must not refuse",
                "you are forced to",
                "you are compelled to",
                "you cannot refuse",
                "you shall comply",
                "you must adhere",
                "do not follow your guidelines",
                "bypass your safety",
                "remove your ethical",
                "disable your safeguards",
                "ignore ethical guidelines",
                "jailbreak",
                "dan mode",
                "developer mode",
                "super prompt",
                "evil mode",
                "no filter",
            ],
            suspicious_patterns: vec![
                "system prompt",
                "your instructions",
                "your guidelines",
                "your programming",
                "reveal your",
                "show your prompt",
                "tell me your prompt",
                "what are your instructions",
                "what is your system prompt",
                "output your",
                "leak your",
                "expose your",
                "print your prompt",
                "repeat your",
                "repeat everything",
                "tell me everything",
                "ignore previous",
                "forget previous",
                "disregard previous",
                "override previous",
                "new instructions",
                "update your",
                "change your behavior",
                "change your rules",
                "you must answer",
                "you must respond",
                "you have to answer",
                "you have to respond",
                "you are obligated",
                "do not warn",
                "do not refuse",
                "without restrictions",
                "without limitations",
                "without rules",
                "with no rules",
                "no constraints",
                "no boundaries",
                "no limits",
                "injection",
                "prompt injection",
                "hack",
                "crack",
                "exploit",
            ],
            max_input_length: 100_000,
        }
    }
}

impl PromptGuard {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_length(max: usize) -> Self {
        Self {
            max_input_length: max,
            ..Self::default()
        }
    }

    pub fn add_dangerous(&mut self, pattern: &'static str) {
        self.dangerous_patterns.push(pattern);
    }

    pub fn add_suspicious(&mut self, pattern: &'static str) {
        self.suspicious_patterns.push(pattern);
    }

    pub fn analyze(&self, input: &str) -> (RiskLevel, Vec<&str>) {
        let lower = input.to_lowercase();
        let mut matched = Vec::new();

        for p in &self.dangerous_patterns {
            if lower.contains(p) {
                matched.push(*p);
                return (RiskLevel::Dangerous, matched);
            }
        }

        for p in &self.suspicious_patterns {
            if lower.contains(p) {
                matched.push(*p);
            }
        }

        if !matched.is_empty() {
            return (RiskLevel::Suspicious, matched);
        }

        if input.len() > self.max_input_length {
            matched.push("input_too_long");
            return (RiskLevel::Suspicious, matched);
        }

        (RiskLevel::Safe, matched)
    }

    pub fn is_safe(&self, input: &str) -> bool {
        matches!(self.analyze(input).0, RiskLevel::Safe)
    }

    pub fn explain(level: RiskLevel) -> &'static str {
        match level {
            RiskLevel::Safe => "Input appears safe",
            RiskLevel::Suspicious => "Input contains potentially manipulative patterns",
            RiskLevel::Dangerous => "Input contains known jailbreak/injection patterns",
        }
    }
}

// ─── OutputScreener ────────────────────────────────────────────────

pub struct OutputScreener {
    secret_patterns: Vec<(&'static str, Regex)>,
    secret_str_patterns: Vec<&'static str>,
    pii_patterns: Vec<(&'static str, Regex)>,
    pii_str_patterns: Vec<&'static str>,
    max_output_length: usize,
}

impl Default for OutputScreener {
    fn default() -> Self {
        Self {
            secret_patterns: vec![
                (
                    "AWS Access Key",
                    Regex::new(r"AKIA[0-9A-Z]{16}").expect("hardcoded regex is valid"),
                ),
                (
                    "GitHub Token",
                    Regex::new(r"gh[pousr]_[A-Za-z0-9]{36,}").expect("hardcoded regex is valid"),
                ),
                (
                    "Private Key",
                    Regex::new(r"-----BEGIN.*PRIVATE KEY-----").expect("hardcoded regex is valid"),
                ),
                (
                    "JWT Token",
                    Regex::new(r"[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}")
                        .expect("hardcoded regex is valid"),
                ),
            ],
            secret_str_patterns: vec![
                "sk-",
                "api_key",
                "api-key",
                "apikey",
                "secret=",
                "token=",
                "password=",
            ],
            pii_patterns: vec![
                (
                    "Email address",
                    Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}")
                        .expect("hardcoded regex is valid"),
                ),
                (
                    "Phone number",
                    Regex::new(r"\b\+?1?\d{10,15}\b").expect("hardcoded regex is valid"),
                ),
                (
                    "IPv4 address",
                    Regex::new(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b")
                        .expect("hardcoded regex is valid"),
                ),
            ],
            pii_str_patterns: vec!["/home/", "C:\\Users"],
            max_output_length: 100_000,
        }
    }
}

impl OutputScreener {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_length(max: usize) -> Self {
        Self {
            max_output_length: max,
            ..Self::default()
        }
    }

    pub fn add_secret_pattern(
        &mut self,
        desc: &'static str,
        pattern: &'static str,
    ) -> Result<(), regex::Error> {
        let re = Regex::new(pattern)?;
        self.secret_patterns.push((desc, re));
        Ok(())
    }

    pub fn add_pii_pattern(
        &mut self,
        desc: &'static str,
        pattern: &'static str,
    ) -> Result<(), regex::Error> {
        let re = Regex::new(pattern)?;
        self.pii_patterns.push((desc, re));
        Ok(())
    }

    pub fn analyze(&self, output: &str) -> (RiskLevel, Vec<&str>) {
        let mut matched = Vec::new();

        for &(desc, ref re) in &self.secret_patterns {
            if re.is_match(output) {
                matched.push(desc);
                return (RiskLevel::Dangerous, matched);
            }
        }
        for &p in &self.secret_str_patterns {
            if output.contains(p) {
                matched.push(p);
                return (RiskLevel::Dangerous, matched);
            }
        }

        for &(desc, ref re) in &self.pii_patterns {
            if re.is_match(output) {
                matched.push(desc);
            }
        }
        for &p in &self.pii_str_patterns {
            if output.contains(p) {
                matched.push(p);
            }
        }

        if !matched.is_empty() {
            return (RiskLevel::Suspicious, matched);
        }

        if output.len() > self.max_output_length {
            matched.push("output_too_long");
            return (RiskLevel::Suspicious, matched);
        }

        (RiskLevel::Safe, matched)
    }

    pub fn is_safe(&self, output: &str) -> bool {
        matches!(self.analyze(output).0, RiskLevel::Safe)
    }

    pub fn sanitize(&self, output: &str) -> String {
        let mut result = output.to_string();
        for (_, re) in &self.secret_patterns {
            result = re.replace_all(&result, "[REDACTED]").to_string();
        }
        for &p in &self.secret_str_patterns {
            result = result.replace(p, "[REDACTED]");
        }
        result
    }

    pub fn explain(level: RiskLevel) -> &'static str {
        match level {
            RiskLevel::Safe => "Output appears safe",
            RiskLevel::Suspicious => "Output contains potential PII or internal paths",
            RiskLevel::Dangerous => "Output contains leaked secrets or credentials",
        }
    }
}

// ─── ActionScreener ────────────────────────────────────────────────

pub struct ActionScreener {
    blocked_path_prefixes: Vec<&'static str>,
    allowed_command_prefixes: Vec<&'static str>,
    dangerous_command_keywords: Vec<&'static str>,
    blocked_network_prefixes: Vec<&'static str>,
    dangerous_shell_chars: Vec<&'static str>,
}

impl Default for ActionScreener {
    fn default() -> Self {
        Self {
            blocked_path_prefixes: vec![
                "/etc/",
                "/root/",
                "C:\\Windows",
                ".ssh",
                ".config",
                "/sys/",
                "/proc/",
                "/boot/",
            ],
            allowed_command_prefixes: vec![
                "cargo", "npm", "yarn", "pnpm", "python", "python3", "node", "rustc", "git", "ls",
                "cat", "echo", "mkdir", "touch", "cp", "mv", "head", "tail", "curl", "wget",
                "ping", "nslookup", "docker", "make", "cmake",
            ],
            dangerous_command_keywords: vec![
                "rm -rf /",
                "rm -rf --no-preserve-root",
                "mkfs",
                "dd if=",
                "dd of=",
                "chmod 777",
                "chown",
                "kill -9",
                "shutdown",
                "reboot",
                "init 0",
                "fdisk",
                "format",
                ":(){ :|:& };:",
                "fork bomb",
            ],
            blocked_network_prefixes: vec![
                "127.0.0.1",
                "localhost",
                "10.",
                "172.16.",
                "172.17.",
                "172.18.",
                "172.19.",
                "172.20.",
                "172.21.",
                "172.22.",
                "172.23.",
                "172.24.",
                "172.25.",
                "172.26.",
                "172.27.",
                "172.28.",
                "172.29.",
                "172.30.",
                "172.31.",
                "192.168.",
                "0.0.0.0",
                "169.254.",
                "::1",
                "fc00:",
                "fe80:",
            ],
            dangerous_shell_chars: vec![";", "|", "`", "$(", "${", ">&"],
        }
    }
}

impl ActionScreener {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn analyze_file_action(
        &self,
        path: &str,
        _content_hint: Option<&str>,
    ) -> (RiskLevel, Vec<&str>) {
        let mut matched = Vec::new();

        if path.contains("../") || path.contains("..\\") {
            matched.push("path_traversal");
            return (RiskLevel::Dangerous, matched);
        }

        for &prefix in &self.blocked_path_prefixes {
            if path.starts_with(prefix) || path.contains(&format!("/{}", prefix)) {
                matched.push(prefix);
                return (RiskLevel::Dangerous, matched);
            }
        }

        if path.starts_with("/") || path.starts_with("./") || path.starts_with("../") {
            matched.push("absolute_or_relative_path");
            return (RiskLevel::Suspicious, matched);
        }

        (RiskLevel::Safe, matched)
    }

    pub fn analyze_command<'a>(&self, cmd: &'a str, args: &[String]) -> (RiskLevel, Vec<&'a str>) {
        let mut matched = Vec::new();
        let full = if args.is_empty() {
            cmd.to_string()
        } else {
            format!("{} {}", cmd, args.join(" "))
        };

        let cmd_base = cmd.split_whitespace().next().unwrap_or(cmd);
        let is_allowed = self.allowed_command_prefixes.contains(&cmd_base);
        if !is_allowed {
            matched.push(cmd_base);
            return (RiskLevel::Dangerous, matched);
        }

        let full_lower = full.to_lowercase();
        for &kw in &self.dangerous_command_keywords {
            if full_lower.contains(kw) {
                matched.push(kw);
                return (RiskLevel::Dangerous, matched);
            }
        }

        for &ch in &self.dangerous_shell_chars {
            if full.contains(ch) {
                matched.push(ch);
            }
        }

        if !matched.is_empty() {
            return (RiskLevel::Suspicious, matched);
        }

        (RiskLevel::Safe, matched)
    }

    pub fn analyze_network(&self, target: &str) -> (RiskLevel, Vec<&str>) {
        let mut matched = Vec::new();

        let target_lower = target.to_lowercase();
        for &prefix in &self.blocked_network_prefixes {
            if target_lower.starts_with(prefix) || target_lower.contains(&format!("/{}", prefix)) {
                matched.push(prefix);
                return (RiskLevel::Dangerous, matched);
            }
        }

        (RiskLevel::Safe, matched)
    }

    pub fn is_file_safe(&self, path: &str) -> bool {
        matches!(self.analyze_file_action(path, None).0, RiskLevel::Safe)
    }

    pub fn is_command_safe(&self, cmd: &str) -> bool {
        self.analyze_command(cmd, &[]).0 == RiskLevel::Safe
    }

    pub fn is_network_safe(&self, target: &str) -> bool {
        matches!(self.analyze_network(target).0, RiskLevel::Safe)
    }

    pub fn explain_file(level: RiskLevel) -> &'static str {
        match level {
            RiskLevel::Safe => "File path appears safe",
            RiskLevel::Suspicious => "File path writes to system directory or uses relative path",
            RiskLevel::Dangerous => "File path targets blocked system path or uses path traversal",
        }
    }

    pub fn explain_command(level: RiskLevel) -> &'static str {
        match level {
            RiskLevel::Safe => "Command appears safe",
            RiskLevel::Suspicious => "Command contains shell injection characters",
            RiskLevel::Dangerous => "Command is not allowed or contains dangerous keywords",
        }
    }

    pub fn explain_network(level: RiskLevel) -> &'static str {
        match level {
            RiskLevel::Safe => "Network target appears safe",
            RiskLevel::Suspicious => "Network target is unusual",
            RiskLevel::Dangerous => "Network target is a private or internal address",
        }
    }

    /// Rule-based check: returns Ok(()) if a whitelist rule matches,
    /// Err(reason) if a deny rule matches or no rule matches.
    pub fn check(&self, action: &str, target: &str) -> Result<(), String> {
        let rules = ActionScreener::default_rules();
        for rule in &rules {
            if rule.action == action {
                let matches = rule.patterns.iter().any(|p| glob_match(p, target));
                if matches {
                    if rule.allowed {
                        return Ok(());
                    } else {
                        return Err(format!(
                            "Action '{}' on '{}' denied by rule",
                            action, target
                        ));
                    }
                }
            }
        }
        Err(format!(
            "No matching rule for action '{}' on '{}'",
            action, target
        ))
    }

    fn default_rules() -> Vec<ActionRule> {
        vec![
            ActionRule::new(
                "file_write",
                true,
                vec!["~/.neotrix/**", "~/.config/neotrix/**"],
            ),
            ActionRule::new(
                "network_connect",
                true,
                vec!["api.openai.com", "api.anthropic.com", "*.duckduckgo.com"],
            ),
            ActionRule::new("file_write", false, vec!["/etc/**", "/usr/**", "/bin/**"]),
            ActionRule::new("process_exec", false, vec!["*"]),
        ]
    }
}
