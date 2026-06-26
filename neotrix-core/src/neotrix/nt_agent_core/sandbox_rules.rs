/// Sandbox Rule DSL — execpolicy-style execution policy engine.
///
/// Inspired by Codex's ~/.codex/rules/*.rules DSL and Claude Code's
/// Bubblewrap/Mac Seatbelt sandbox profiles. Provides glob-based
/// allow/deny lists, danger pattern matching, and shell restrictions.
///
/// # Rule file format (`.rules` or `.toml`)
/// ```toml
/// [shell]
/// allow = ["ls", "echo", "cat", "git *"]
/// deny = ["rm -rf /*", "dd *", "mkfs *", "sudo *"]
///
/// [network]
/// allow = ["api.github.com", "*.rust-lang.org"]
/// deny = ["*.malicious.com"]
///
/// [filesystem]
/// read_allow = ["src/**", "tests/**", "Cargo.*"]
/// write_allow = ["src/**", "tests/**"]
/// deny = [".env", "**/credentials*", "**/secrets*"]
///
/// [danger]
/// patterns = ["rm -rf /", "dd if=", ":(){ :|:& };:", "> /dev/sda"]
///
/// [sandbox]
/// level = "isolated"  # none | readonly | isolated | strict
/// timeout_secs = 300
/// max_output_bytes = 1048576
/// ```
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellRules {
    pub allow: Vec<String>,
    pub deny: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRules {
    pub allow: Vec<String>,
    pub deny: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemRules {
    pub read_allow: Vec<String>,
    pub write_allow: Vec<String>,
    pub deny: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DangerPatterns {
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxLevelConfig {
    pub level: String,
    pub timeout_secs: u64,
    pub max_output_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecPolicy {
    pub shell: ShellRules,
    pub network: NetworkRules,
    pub filesystem: FilesystemRules,
    pub danger: DangerPatterns,
    pub sandbox: SandboxLevelConfig,
    /// Optional: named overrides for specific tools
    #[serde(default)]
    pub tool_overrides: HashMap<String, ToolOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOverride {
    pub allowed_hosts: Vec<String>,
    pub allowed_paths: Vec<String>,
    pub max_args: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum RuleDecision {
    Allow,
    Deny(String),
    Escalate,
}

impl ExecPolicy {
    pub fn default_dev() -> Self {
        Self {
            shell: ShellRules {
                allow: vec![
                    "ls".into(),
                    "echo".into(),
                    "cat".into(),
                    "head".into(),
                    "tail".into(),
                    "wc".into(),
                    "sort".into(),
                    "uniq".into(),
                    "grep".into(),
                    "find".into(),
                    "mkdir".into(),
                    "touch".into(),
                    "cp".into(),
                    "mv".into(),
                    "rm".into(),
                    "git *".into(),
                    "cargo *".into(),
                    "rustc *".into(),
                    "python3 *".into(),
                    "node *".into(),
                    "npm *".into(),
                    "deno *".into(),
                ],
                deny: vec![
                    "sudo *".into(),
                    "su *".into(),
                    "chmod 777 *".into(),
                    "dd *".into(),
                    "mkfs *".into(),
                    "fdisk *".into(),
                    "shutdown".into(),
                    "reboot".into(),
                    "halt".into(),
                    "iptables *".into(),
                    "ufw *".into(),
                ],
            },
            network: NetworkRules {
                allow: vec![
                    "api.github.com".into(),
                    "*.rust-lang.org".into(),
                    "crates.io".into(),
                    "pypi.org".into(),
                    "registry.npmjs.org".into(),
                    "*.googleapis.com".into(),
                    "api.openai.com".into(),
                ],
                deny: vec![
                    "*.malicious.com".into(),
                    "10.*".into(),
                    "172.16.*".into(),
                    "192.168.*".into(),
                ],
            },
            filesystem: FilesystemRules {
                read_allow: vec![
                    "**/*.rs".into(),
                    "**/Cargo.*".into(),
                    "**/*.toml".into(),
                    "src/**".into(),
                    "tests/**".into(),
                    "docs/**".into(),
                    "*.md".into(),
                    "*.json".into(),
                    "*.yaml".into(),
                    "*.yml".into(),
                ],
                write_allow: vec![
                    "src/**".into(),
                    "tests/**".into(),
                    "docs/**".into(),
                    "*.md".into(),
                    "*.toml".into(),
                    "*.json".into(),
                ],
                deny: vec![
                    ".env".into(),
                    "**/credentials*".into(),
                    "**/secrets*".into(),
                    "**/*.pem".into(),
                    "**/*.key".into(),
                    "**/.ssh/**".into(),
                    "**/.aws/**".into(),
                    "**/.config/**".into(),
                ],
            },
            danger: DangerPatterns {
                patterns: vec![
                    "rm -rf /".into(),
                    "rm -rf /*".into(),
                    "dd if=".into(),
                    ":(){ :|:& };:".into(),
                    "> /dev/sda".into(),
                    "mkfs".into(),
                    "chmod -R 777 /".into(),
                    "mv /* /dev/null".into(),
                ],
            },
            sandbox: SandboxLevelConfig {
                level: "isolated".into(),
                timeout_secs: 300,
                max_output_bytes: 1_048_576,
            },
            tool_overrides: HashMap::new(),
        }
    }

    pub fn default_strict() -> Self {
        let mut dev = Self::default_dev();
        dev.sandbox.level = "strict".into();
        dev.sandbox.timeout_secs = 60;
        dev.shell
            .allow
            .retain(|c| ["ls", "echo", "cat", "grep", "head", "tail"].contains(&c.as_str()));
        dev.tool_overrides.insert(
            "file_write".into(),
            ToolOverride {
                allowed_hosts: vec![],
                allowed_paths: vec![],
                max_args: Some(1),
            },
        );
        dev
    }

    pub fn check_shell(&self, cmd: &str) -> RuleDecision {
        for pattern in &self.danger.patterns {
            if self.glob_match(cmd, pattern) {
                return RuleDecision::Deny(format!("danger pattern matched: {}", pattern));
            }
        }
        for pattern in &self.shell.deny {
            if self.glob_match(cmd, pattern) {
                return RuleDecision::Deny(format!("shell deny pattern matched: {}", pattern));
            }
        }
        for pattern in &self.shell.allow {
            if self.glob_match(cmd, pattern) {
                return RuleDecision::Allow;
            }
        }
        RuleDecision::Escalate
    }

    pub fn check_network(&self, host: &str) -> RuleDecision {
        for pattern in &self.network.deny {
            if self.glob_match(host, pattern) {
                return RuleDecision::Deny(format!("network deny pattern matched: {}", pattern));
            }
        }
        for pattern in &self.network.allow {
            if self.glob_match(host, pattern) {
                return RuleDecision::Allow;
            }
        }
        RuleDecision::Escalate
    }

    pub fn check_filesystem_read(&self, path: &str) -> RuleDecision {
        for pattern in &self.filesystem.deny {
            if self.glob_match(path, pattern) {
                return RuleDecision::Deny(format!("fs deny pattern matched: {}", pattern));
            }
        }
        for pattern in &self.filesystem.read_allow {
            if self.glob_match(path, pattern) {
                return RuleDecision::Allow;
            }
        }
        RuleDecision::Escalate
    }

    pub fn check_filesystem_write(&self, path: &str) -> RuleDecision {
        for pattern in &self.filesystem.deny {
            if self.glob_match(path, pattern) {
                return RuleDecision::Deny(format!("fs deny pattern matched: {}", pattern));
            }
        }
        for pattern in &self.filesystem.write_allow {
            if self.glob_match(path, pattern) {
                return RuleDecision::Allow;
            }
        }
        RuleDecision::Escalate
    }

    fn glob_match(&self, input: &str, pattern: &str) -> bool {
        if let Ok(re) = self.glob_to_regex(pattern) {
            re.is_match(input)
        } else {
            input == pattern
        }
    }

    fn glob_to_regex(&self, pattern: &str) -> Result<regex::Regex, regex::Error> {
        let mut re_str = String::from("^");
        for ch in pattern.chars() {
            match ch {
                '*' => re_str.push_str(".*"),
                '?' => re_str.push_str("."),
                '.' => re_str.push_str("\\."),
                '/' => re_str.push('/'),
                c => re_str.push(c),
            }
        }
        re_str.push('$');
        regex::Regex::new(&re_str)
    }
}

impl Default for ExecPolicy {
    fn default() -> Self {
        Self::default_dev()
    }
}

/// Load policy from TOML string
pub fn parse_exec_policy(toml_str: &str) -> Result<ExecPolicy, String> {
    toml::from_str(toml_str).map_err(|e| format!("failed to parse exec policy: {}", e))
}

/// SandboxRuleEngine — manages multiple named execution policies
pub struct SandboxRuleEngine {
    policies: HashMap<String, ExecPolicy>,
    active_policy: String,
    default_policy: ExecPolicy,
}

impl SandboxRuleEngine {
    pub fn new() -> Self {
        let mut policies = HashMap::new();
        policies.insert("dev".into(), ExecPolicy::default_dev());
        policies.insert("strict".into(), ExecPolicy::default_strict());
        Self {
            policies,
            active_policy: "dev".into(),
            default_policy: ExecPolicy::default_dev(),
        }
    }

    pub fn with_policy(mut self, name: &str, policy: ExecPolicy) -> Self {
        self.policies.insert(name.into(), policy);
        self
    }

    pub fn set_active(&mut self, name: &str) -> Result<(), String> {
        if self.policies.contains_key(name) {
            self.active_policy = name.into();
            Ok(())
        } else {
            Err(format!("policy '{}' not found", name))
        }
    }

    pub fn active(&self) -> &str {
        &self.active_policy
    }

    pub fn active_policy(&self) -> &ExecPolicy {
        self.policies
            .get(&self.active_policy)
            .unwrap_or(&self.default_policy)
    }

    pub fn check_shell(&self, cmd: &str) -> RuleDecision {
        self.active_policy().check_shell(cmd)
    }

    pub fn check_network(&self, host: &str) -> RuleDecision {
        self.active_policy().check_network(host)
    }

    pub fn check_filesystem_read(&self, path: &str) -> RuleDecision {
        self.active_policy().check_filesystem_read(path)
    }

    pub fn check_filesystem_write(&self, path: &str) -> RuleDecision {
        self.active_policy().check_filesystem_write(path)
    }

    pub fn summary(&self) -> String {
        format!(
            "SandboxRuleEngine: {} policies, active='{}'",
            self.policies.len(),
            self.active_policy
        )
    }
}

impl Default for SandboxRuleEngine {
    fn default() -> Self {
        Self::new()
    }
}
