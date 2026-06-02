//! 安全审计模块 — 对标 Grippy 确定性规则引擎 + 供应链扫描
//!
//! 特性:
//!   - secrets-in-diff: 检测代码中的密钥泄露
//!   - path-traversal: 检测路径遍历风险
//!   - command-injection: 检测 shell 注入模式
//!   - unsafe-code: 审计 unsafe 块
//!   - weak-crypto: 检测弱加密算法引用
//!   - supply-chain: cargo-audit 集成 + 依赖版本审计 (S-CR-15)
//!
//! OWASP Top 10:2025 覆盖:
//!   A04 Cryptographic Failures — 弱加密检测
//!   A05 Injection — 命令/路径注入检测
//!   A03 Software Supply Chain — 依赖审计
//!   X02 Memory Management Failures — unsafe 审计

use std::path::{Path, PathBuf};
use std::process::Command;
use crate::neotrix::error::{NeoTrixResult, NeoTrixError};

/// 安全审计发现
#[derive(Debug, Clone)]
pub struct SecurityFinding {
    pub file: PathBuf,
    pub line: usize,
    pub severity: String,
    pub rule: String,
    pub description: String,
    pub fix: String,
}

/// 供应链漏洞 (cargo-audit 结果)
#[derive(Debug, Clone)]
pub struct SupplyChainVuln {
    pub package: String,
    pub version: String,
    pub severity: String,
    pub advisory_id: String,
    pub description: String,
    pub fix_version: Option<String>,
}

/// 安全审计引擎（参照 ECC AgentShield 102 规则 + OWASP Top 10:2025）
pub struct SecurityAudit {
    pub rules: Vec<AuditRule>,
}

/// 审计规则
#[derive(Debug, Clone)]
pub struct AuditRule {
    pub name: &'static str,
    pub severity: &'static str,
    pub pattern: &'static str,
    pub description: &'static str,
    pub fix: &'static str,
    pub owasp: Option<&'static str>,
}

impl SecurityAudit {
    pub fn new() -> Self {
        Self {
            rules: Self::default_rules(),
        }
    }

    fn default_rules() -> Vec<AuditRule> {
        vec![
            // ========== Critical (7) ==========
            AuditRule {
                name: "secrets-in-diff",
                severity: "critical",
                pattern: "api_key|apiKey|GITHUB_TOKEN|NEOTRIX_API_KEY|sk-[a-zA-Z0-9]{20,}",
                description: "检测到可能硬编码的密钥/令牌",
                fix: "移动密钥到环境变量或加密 vault; 检查 git history 是否泄露",
                owasp: Some("A04:2025"),
            },
            AuditRule {
                name: "command-injection",
                severity: "critical",
                pattern: r#"Command::new\("sh"\)|Command::new\("bash"\)|\.args\(\["-c"|cmd\.exe"#,
                description: "检测到 shell 命令注入风险",
                fix: "使用 Command::arg() 直接传参; 不要拼接 shell 字符串",
                owasp: Some("A05:2025"),
            },
            AuditRule {
                name: "ssrf",
                severity: "critical",
                pattern: r"reqwest::get\(.*input|reqwest::Client::new\(\)|\.get\(.*params|open\(.*url",
                description: "检测到 SSRF 风险 — 用户输入直接用于发起网络请求",
                fix: "使用 allowlist 限制目标 URL; 禁用内网地址; 设置超时",
                owasp: Some("A05:2025"),
            },
            AuditRule {
                name: "insecure-deserialization",
                severity: "critical",
                pattern: r"serde_json::from_str<.*>\(.*input|bincode::deserialize|pickle\.loads|JSON\.parse\(user|eval\(",
                description: "检测到不安全的反序列化模式",
                fix: "验证输入 schema; 限制反序列化深度; 避免反序列化不可信数据",
                owasp: Some("A08:2025"),
            },
            AuditRule {
                name: "os-command-exec",
                severity: "critical",
                pattern: r"std::process::Command|Process::Start|subprocess\.run|exec\s+|system\(|popen\(",
                description: "检测到 OS 命令执行模式",
                fix: "使用安全 API 替代; 验证命令参数; 避免 shell 拼接",
                owasp: Some("A05:2025"),
            },
            AuditRule {
                name: "sql-injection",
                severity: "critical",
                pattern: r"format!\(.*SELECT.*from|format!\(.*INSERT INTO|format!\(.*DELETE FROM|sqlx::query\(&format|\.prepare\(.*\+.*user|execute\(.*\+.*params",
                description: "检测到 SQL 注入风险",
                fix: "使用参数化查询(sqlx::query!); 不要拼接 SQL 字符串",
                owasp: Some("A03:2025"),
            },
            AuditRule {
                name: "hardcoded-jwt",
                severity: "critical",
                pattern: r#"jwt_secret|JWT_SECRET|signing_key.*=.*" |signing_secret|HS256.*=.*""#,
                description: "检测到硬编码 JWT 密钥",
                fix: "从环境变量或 secrets manager 加载密钥; 轮换签名密钥",
                owasp: Some("A04:2025"),
            },
            // ========== High (10) ==========
            AuditRule {
                name: "path-traversal",
                severity: "high",
                pattern: r#"\.\./\.\./|PathBuf::from\(.*user|read_to_string\(.*input"#,
                description: "检测到路径遍历风险 — 用户输入直接用于文件操作",
                fix: "规范化路径; 使用 allowlist; 限制访问范围",
                owasp: Some("A01:2025"),
            },
            AuditRule {
                name: "unsafe-block",
                severity: "high",
                pattern: r"unsafe\s*\{",
                description: "unsafe 块无安全注释说明",
                fix: "添加 Safety: 注释说明为什么 unsafe 是安全的; 考虑安全抽象替代",
                owasp: Some("X02:2025"),
            },
            AuditRule {
                name: "weak-crypto",
                severity: "high",
                pattern: r"md5|sha1|des\b|rc4\b|ecb\b|aes-128-ecb|RSA/ECB",
                description: "检测到弱加密算法引用",
                fix: "使用 SHA-256/384, AES-GCM, ChaCha20-Poly1305 替代",
                owasp: Some("A04:2025"),
            },
            AuditRule {
                name: "template-injection",
                severity: "high",
                pattern: r"Template::render\(.*input|\.render\(.*&user|Handlebars::new\(\)|Tera::one_off\(.*user",
                description: "检测到服务器端模板注入风险",
                fix: "预编译模板; 不要将用户输入作为模板; 使用自动转义引擎",
                owasp: Some("A05:2025"),
            },
            AuditRule {
                name: "open-redirect",
                severity: "high",
                pattern: r"redirect\(.*params|Redirect::to\(.*input|\.redirect\(.*query|Location:.*params",
                description: "检测到开放重定向风险",
                fix: "使用 allowlist 验证重定向目标; 不要直接使用用户输入构造 URL",
                owasp: Some("A01:2025"),
            },
            AuditRule {
                name: "insecure-direct-object-ref",
                severity: "high",
                pattern: r"\.find\(.*params\[|\.get\(.*&id|DELETE FROM.*WHERE id =.*input|UPDATE.*SET.*WHERE id =.*user",
                description: "检测到不安全的直接对象引用",
                fix: "验证用户是否有权限访问该资源; 使用间接引用或权限检查",
                owasp: Some("A01:2025"),
            },
            AuditRule {
                name: "xxe",
                severity: "high",
                pattern: r"XMLParser|xml::parse|quick_xml::Reader|serde_xml|xml2json|loadXML|DOMParser",
                description: "检测到 XML 外部实体处理",
                fix: "禁用外部实体解析; 使用 JSON 替代 XML; 配置 XML 解析器禁用 DTD",
                owasp: Some("A05:2025"),
            },
            AuditRule {
                name: "race-condition",
                severity: "high",
                pattern: r"Arc<Mutex<|Arc<RwLock<|tokio::sync::Mutex|AtomicU|std::sync::atomic|\.store\(|\.load\(",
                description: "检测到共享状态并发访问",
                fix: "确保正确加锁; 使用事务; 使用原子操作; 避免 TOCTOU 模式",
                owasp: Some("A01:2025"),
            },
            AuditRule {
                name: "weak-key-gen",
                severity: "high",
                pattern: r"rand::thread_rng|rng\.gen_range|rand::random|Random\.Next|SecureRandom\(\)",
                description: "检测到可能弱的随机数生成",
                fix: "对密码学用途使用 OsRng / getrandom; 不要用 thread_rng 生成密钥",
                owasp: Some("A04:2025"),
            },
            // ========== Medium (8) ==========
            AuditRule {
                name: "unwrap-usage",
                severity: "medium",
                pattern: r"\.unwrap\(\)",
                description: "使用 .unwrap() 可能导致 panic",
                fix: "替换为 .expect(\"msg\") 或 ? 操作符 + 统一 NeoTrixError",
                owasp: Some("A08:2025"),
            },
            AuditRule {
                name: "panic-usage",
                severity: "medium",
                pattern: r"panic!\(",
                description: "使用 panic!() 导致不可恢复崩溃",
                fix: "返回 Result 类型代替 panic!",
                owasp: Some("A08:2025"),
            },
            AuditRule {
                name: "xss",
                severity: "medium",
                pattern: r"inner_html|innerHTML|\.html\(.*input|\.append\(.*user|dangerouslySetInnerHTML|v-html|raw\(.*input",
                description: "检测到跨站脚本风险",
                fix: "使用 text() 替代 html(); 为 HTML 上下文使用自动转义模板",
                owasp: Some("A07:2025"),
            },
            AuditRule {
                name: "info-leak",
                severity: "medium",
                pattern: r"println!\(.*error|eprintln!\(.*secret|log::info!\(.*password|console\.log\(.*secret|debug!\(.*key",
                description: "检测到信息泄露风险 — 日志中可能包含敏感数据",
                fix: "在日志中脱敏; 使用结构化日志过滤敏感字段; 避免打印密钥",
                owasp: Some("A01:2025"),
            },
            AuditRule {
                name: "debug-endpoint",
                severity: "medium",
                pattern: r#""/debug"|"/_debug"|"/status"|"/health"|GET.*/admin|"/api-docs|swagger|"/actuator"#,
                description: "检测到调试/管理端点暴露",
                fix: "生产环境禁用调试端点; 添加认证; 使用内网访问限制",
                owasp: Some("A05:2025"),
            },
            AuditRule {
                name: "none-cipher",
                severity: "medium",
                pattern: r"AES\.GCM\.NoPadding|Cipher\.None|ssl_version.*=.*0|no_tls|tls_version.*tls1\.[01]",
                description: "检测到空加密或不安全 TLS 版本",
                fix: "启用 TLS 1.2+; 使用 AEAD 模式(GCM/ChaCha20); 禁用空密码套件",
                owasp: Some("A04:2025"),
            },
            AuditRule {
                name: "cors-wildcard",
                severity: "medium",
                pattern: r#"Access-Control-Allow-Origin: \*|allow_origins\(Any|\.set\("\*"\)|Access-Control-Allow-Origin.*\*"#,
                description: "检测到通配符 CORS 配置",
                fix: "限制 Allow-Origin 为具体域名; 不要在 production 使用 *",
                owasp: Some("A01:2025"),
            },
            AuditRule {
                name: "deprecated-http",
                severity: "medium",
                pattern: r"http://[a-z]|HttpConnector|http\.get\(|reqwest::get\(http://",
                description: "检测到明文 HTTP 连接",
                fix: "使用 HTTPS(TLS); 禁用 HTTP 回退; 设置 HSTS 头",
                owasp: Some("A04:2025"),
            },
        ]
    }

    /// 扫描单个文件的安全风险（使用正则匹配）
    pub fn scan_file(&self, path: &Path, content: &str) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        for rule in &self.rules {
            let re = match regex::Regex::new(rule.pattern) {
                Ok(r) => r,
                Err(_) => continue,
            };

            for (i, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("//") || trimmed.starts_with("#[") {
                    continue;
                }

                if re.find(line).is_some() {
                    findings.push(SecurityFinding {
                        file: path.to_path_buf(),
                        line: i + 1,
                        severity: rule.severity.to_string(),
                        rule: rule.name.to_string(),
                        description: format!("[OWASP {}] {}: {}", rule.owasp.unwrap_or("N/A"), rule.description, trimmed),
                        fix: rule.fix.to_string(),
                    });
                    break;
                }
            }
        }

        findings
    }

    /// 扫描整个目录的安全风险
    pub fn scan_directory(&self, root: &str) -> Vec<SecurityFinding> {
        let mut all_findings = Vec::new();
        let root_path = Path::new(root);

        if !root_path.exists() || !root_path.is_dir() {
            return all_findings;
        }

        if let Ok(entries) = std::fs::read_dir(root_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "rs") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let findings = self.scan_file(&path, &content);
                        all_findings.extend(findings);
                    }
                }
            }
        }

        all_findings
    }

    // ========== 供应链漏洞扫描 (S-CR-15) ==========

    /// 运行 cargo-audit 扫描供应链漏洞
    /// 对标 nitpik secret scanning + 200+ gitleaks rules
    /// S-CR-17: 返回 NeoTrixResult 统一错误类型
    pub fn cargo_audit(&self, project_path: &str) -> NeoTrixResult<Vec<SupplyChainVuln>> {
        let mut vulns = Vec::new();

        let output = Command::new("cargo")
            .args(["audit", "--json"])
            .current_dir(project_path)
            .output()
            .map_err(|e| NeoTrixError::Command {
                cmd: "cargo audit --json".to_string(),
                exit_code: None,
                stderr: e.to_string(),
            })?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            vulns = Self::parse_cargo_audit_json(&stdout);
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::warn!("cargo audit failed (maybe not installed): {}", stderr);
        }

        if vulns.is_empty() {
            // fallback: 静态检查 Cargo.toml 中已知风险模式
            let cargo_path = Path::new(project_path).join("Cargo.toml");
            let content = std::fs::read_to_string(&cargo_path)
                .map_err(|e| NeoTrixError::Path {
                    path: cargo_path.clone(),
                    detail: e.to_string(),
                })?;
            vulns = self.static_supply_chain_check(&content);
        }

        Ok(vulns)
    }

    /// 解析 cargo-audit JSON 输出
    fn parse_cargo_audit_json(json_str: &str) -> Vec<SupplyChainVuln> {
        let mut vulns = Vec::new();

        // 尝试结构化解析
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
            if let Some(vulnerabilities) = val.get("vulnerabilities").and_then(|v| v.as_object()) {
                for (_pkg_key, vuln_list) in vulnerabilities {
                    if let Some(arr) = vuln_list.as_array() {
                        for vuln in arr {
                            let package = vuln.get("package").and_then(|v| v.as_str())
                                .unwrap_or("unknown").to_string();
                            let version = vuln.get("version").and_then(|v| v.as_str())
                                .unwrap_or("unknown").to_string();
                            let severity = vuln.get("severity").and_then(|v| v.as_str())
                                .unwrap_or("unknown").to_string();
                            let advisory_id = vuln.get("advisory").and_then(|v| v.as_str())
                                .or_else(|| vuln.get("id").and_then(|v| v.as_str()))
                                .unwrap_or("unknown").to_string();
                            let description = vuln.get("title").and_then(|v| v.as_str())
                                .or_else(|| vuln.get("description").and_then(|v| v.as_str()))
                                .unwrap_or("no description").to_string();
                            let fix_version = vuln.get("patched_version").and_then(|v| v.as_str())
                                .or_else(|| vuln.get("patched_versions").and_then(|v| v.as_str()))
                                .map(|s| s.to_string());

                            vulns.push(SupplyChainVuln {
                                package, version, severity, advisory_id,
                                description, fix_version,
                            });
                        }
                    }
                }
            }
        }

        vulns
    }

    /// 静态供应链安全检查（cargo-audit 不可用时的 fallback）
    fn static_supply_chain_check(&self, cargo_toml: &str) -> Vec<SupplyChainVuln> {
        let mut vulns = Vec::new();

        for (i, line) in cargo_toml.lines().enumerate() {
            let trimmed = line.trim();

            // 通配符依赖检测
            if trimmed.contains('=') && trimmed.contains('"') && !trimmed.starts_with('[')
                && (trimmed.contains('*') || trimmed.contains("\"*\"")) {
                    vulns.push(SupplyChainVuln {
                        package: trimmed.split('=').next().unwrap_or("unknown").trim().to_string(),
                        version: "*".to_string(),
                        severity: "medium".to_string(),
                        advisory_id: "static-check".to_string(),
                        description: format!("通配符依赖版本 `*` (line {}) — 可能导致供应链攻击", i + 1),
                        fix_version: Some("锁定到精确版本或 semver 范围".to_string()),
                    });
                }

            // Git 依赖未锁定 rev
            if trimmed.contains("git = \"") && !trimmed.contains("rev = \"") {
                let lines: Vec<&str> = cargo_toml.lines().take(i).collect();
                let pkg_name = lines.iter().rev()
                    .find(|l| l.contains("[dependencies") || l.contains(']'))
                    .and_then(|l| {
                        if l.contains(']') {
                            l.trim().trim_start_matches('[').trim_end_matches(']').split('/').next_back().map(|s| s.to_string())
                        } else { None }
                    })
                    .unwrap_or_default();
                if !pkg_name.is_empty() {
                    vulns.push(SupplyChainVuln {
                        package: pkg_name,
                        version: "git-unpinned".to_string(),
                        severity: "medium".to_string(),
                        advisory_id: "static-check".to_string(),
                        description: format!("Git 依赖未锁定 rev 哈希 (line {})", i + 1),
                        fix_version: Some("添加 rev = \"<commit-hash>\"".to_string()),
                    });
                }
            }
        }

        vulns
    }

    /// 综合供应链安全评分 (0-100)
    pub fn supply_chain_score(&self, vulns: &[SupplyChainVuln]) -> u32 {
        if vulns.is_empty() {
            return 100;
        }

        let critical = vulns.iter().filter(|v| v.severity == "critical").count() as u32 * 25;
        let high = vulns.iter().filter(|v| v.severity == "high" || v.severity == "medium").count() as u32 * 10;
        let penalty = critical + high;

        100u32.saturating_sub(penalty.min(100))
    }
}

impl Default for SecurityAudit {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_scans_rs_file() {
        let audit = SecurityAudit::new();
        let content = "let token = \"sk-test123\";\nfn main() { unsafe { *p = 1; } }";
        let findings = audit.scan_file(Path::new("test.rs"), content);
        assert!(!findings.is_empty());
    }

    #[test]
    fn test_cargo_audit_fallback_static_check() {
        let audit = SecurityAudit::new();
        let cargo = r#"
[package]
name = "test"

[dependencies]
danger-dep = "*"
unstable-dep = { git = "https://github.com/evil/repo" }
"#;
        let vulns = audit.static_supply_chain_check(cargo);
        assert!(!vulns.is_empty());
        assert!(vulns.iter().any(|v| v.description.contains("通配符")));
    }

    #[test]
    fn test_supply_chain_score() {
        let audit = SecurityAudit::new();
        let vulns = vec![
            SupplyChainVuln {
                package: "bad".into(), version: "1.0".into(),
                severity: "critical".into(), advisory_id: "CVE-2024".into(),
                description: "RCE vuln".into(), fix_version: Some("2.0".into()),
            },
        ];
        let score = audit.supply_chain_score(&vulns);
        assert!(score < 100);
        assert_eq!(score, 75);
    }

    #[test]
    fn test_clean_supply_chain_score() {
        let audit = SecurityAudit::new();
        let score = audit.supply_chain_score(&[]);
        assert_eq!(score, 100);
    }
}
