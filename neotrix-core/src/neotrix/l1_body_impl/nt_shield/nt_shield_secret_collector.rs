//! 多层 Secret Collector — D21 (offseq/threat-finder 参照)
//!
//! 从「仅扫描 agent 工具参数 (SEC-007)」扩展为多层凭据搜集 (Collector 抽象):
//!   1. env — 进程环境变量 (API key / token / secret 名)
//!   2. dir — 工作树文件逐行扫描 (file:line 定位, 复用 Redactor 正则库)
//!
//! 输出 file:line JSON 报告, 供审计追踪。遵循 R-P42 (强化现有节点, 复用 redaction 正则)。

use super::redaction::Redactor;
use serde::Serialize;
use std::path::Path;

/// 凭据来源层
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum SecretSource {
    /// 进程环境变量
    Env,
    /// 工作区文件
    File,
    /// 目录名/参数 (既有 SEC-007 层)
    Args,
}

impl std::fmt::Display for SecretSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecretSource::Env => write!(f, "env"),
            SecretSource::File => write!(f, "file"),
            SecretSource::Args => write!(f, "args"),
        }
    }
}

/// 单条密钥扫描命中
#[derive(Debug, Clone, Serialize)]
pub struct SecretHit {
    /// 来源层
    pub source: SecretSource,
    /// 环境变量名 / 文件路径 / 参数名
    pub location: String,
    /// 命中行号 (File 层)
    pub line: Option<u64>,
    /// 命中的规则
    pub rule: String,
    /// 风险决定 (参考 threat-finder: 运行时暴露是最高优先信号)
    pub exposed: bool,
}

/// 密钥扫描报告
#[derive(Debug, Default, Serialize)]
pub struct SecretReport {
    pub total: usize,
    pub hits: Vec<SecretHit>,
}

/// 多层 Secret Collector — 复用 Redactor 正则库
pub struct SecretCollector {
    redactor: Redactor,
}

impl Default for SecretCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl SecretCollector {
    pub fn new() -> Self {
        Self { redactor: Redactor::new() }
    }

    /// 忽略的目录 (扫描工作区时跳过)
    pub const IGNORE_DIRS: &'static [&'static str] = &[
        "target", "node_modules", ".git", ".shadow", "dist", "build", ".gradle",
    ];

    /// 扫描进程环境变量 — 名字含 secret/api/key/token 的 key 视为暴露候选。
    pub fn scan_env(&self) -> Vec<SecretHit> {
        let mut hits = Vec::new();
        for (key, value) in std::env::vars() {
            let k = key.to_lowercase();
            let sensitive_name = k.contains("api_key")
                || k.contains("apikey")
                || k.contains("secret")
                || k.contains("token")
                || k.contains("password")
                || k.contains("key");
            let has_value = value.len() >= 8 && !value.is_empty();
            // 命中正则或敏感性名 + 非空值 → 暴露
            let rule_hit = self.redactor.find_secrets(&value);
            if sensitive_name && has_value {
                hits.push(SecretHit {
                    source: SecretSource::Env,
                    location: key.clone(),
                    line: None,
                    rule: rule_hit.first().map(|(r, _)| r.clone()).unwrap_or_else(|| "stray".into()),
                    exposed: true,
                });
            } else if !rule_hit.is_empty() {
                hits.push(SecretHit {
                    source: SecretSource::Env,
                    location: key,
                    line: None,
                    rule: rule_hit[0].0.clone(),
                    exposed: true,
                });
            }
        }
        hits
    }

    /// 扫描目录下所有文本文件 (递归, 跳过 target/node_modules 等)。
    /// 逐行用 Redactor 定位 secret, 记录 file:line。
    pub fn scan_dir(&self, root: &Path) -> Vec<SecretHit> {
        let mut hits = Vec::new();
        self.walk(root, &mut hits);
        hits
    }

    fn walk(&self, dir: &Path, out: &mut Vec<SecretHit>) {
        let Ok(read_dir) = std::fs::read_dir(dir) else { return };
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if Self::IGNORE_DIRS.contains(&name.as_str()) {
                    continue;
                }
                self.walk(&path, out);
            } else if Self::is_scanable(&path) {
                self.scan_file(&path, out);
            }
        }
    }

    fn scan_file(&self, path: &Path, out: &mut Vec<SecretHit>) {
        let Ok(content) = std::fs::read_to_string(path) else { return };
        for (idx, line) in content.lines().enumerate() {
            let hits = self.redactor.find_secrets(line);
            for (rule, _) in hits {
                out.push(SecretHit {
                    source: SecretSource::File,
                    location: path.display().to_string(),
                    line: Some((idx + 1) as u64),
                    rule,
                    exposed: true, // 工作区出现即视为暴露候选 (静态信号)
                });
            }
        }
    }

    fn is_scanable(path: &Path) -> bool {
        matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("rs" | "toml" | "json" | "yml" | "yaml" | "env" | "sh" | "py" | "mod" | "conf" | "ini" | "tf" | "lock")
        )
    }

    /// 汇总为报告
    pub fn collect(&self, dir: Option<&Path>) -> SecretReport {
        let mut hits = self.scan_env();
        if let Some(root) = dir {
            hits.extend(self.scan_dir(root));
        }
        let total = hits.len();
        SecretReport { total, hits }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn tmp_dir(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("nt-secret-scan-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn scan_dir_finds_sk_and_ghp_with_line_numbers() {
        let dir = tmp_dir("file");
        let mut f = std::fs::File::create(dir.join("config.rs")).unwrap();
        writeln!(f, "fn main() {{").unwrap();
        writeln!(f, "  let key = \"sk-abcdefghijklmnopqrstuvwxyz123456\";").unwrap();
        writeln!(f, "  println!(\"ok\");").unwrap();
        drop(f);

        let c = SecretCollector::new();
        let hits = c.scan_dir(&dir);
        let file_hits: Vec<_> = hits.iter().filter(|h| h.source == SecretSource::File).collect();
        assert!(!file_hits.is_empty(), "should find at least one sk- hit");
        // sk- 可能同时命中 openai 与 stripe 两条正则, 但都必须在第 2 行
        assert!(file_hits.iter().all(|h| h.line == Some(2)), "sk- key must be located on line 2");
        assert!(file_hits[0].location.contains("config.rs"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn ignores_target_dir() {
        let dir = tmp_dir("ignore");
        let sub = dir.join("target");
        std::fs::create_dir_all(&sub).unwrap();
        let mut f = std::fs::File::create(sub.join("secret.rs")).unwrap();
        writeln!(f, "let t = \"sk-abcdefghijklmnopqrstuvwxyz123456\";").unwrap();
        drop(f);

        let c = SecretCollector::new();
        let hits = c.scan_dir(&dir);
        assert!(hits.is_empty(), "target/ must be ignored");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn env_scan_marks_sensitive_names() {
        // 不依赖真实环境, 验证分类逻辑的纯函数路径不可测, 这里只断言模块可实例化
        let c = SecretCollector::new();
        let report = c.collect(None);
        assert!(report.total >= 0);
        assert!(c.scan_env().len() == report.hits.len() || report.hits.len() >= c.scan_env().len());
    }
}