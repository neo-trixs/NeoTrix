//! `nt_select` — 确定性选文件.
//!
//! 对标 `internal/agent/selection.go` (纯函数): 优先级即顺序 —
//! secret > user-exclude > user-include > allowlist-ext > default-exclude >
//! size-ceiling. `--preview` 与实跑共用同一函数 (OCR #782 漂移教训).

use serde::{Deserialize, Serialize};

/// 排除原因 (OCR `whyExcluded` 同语义).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExclusionReason {
    Secret,
    UserRule,
    Extension,
    DefaultPath,
    TooLarge,
}

impl ExclusionReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Secret => "secret",
            Self::UserRule => "user_rule",
            Self::Extension => "extension",
            Self::DefaultPath => "default_path",
            Self::TooLarge => "too_large",
        }
    }
}

/// 选文件配置.
#[derive(Debug, Clone)]
pub struct SelectConfig {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    /// 允许的扩展名 (小写, 无点; 空 = 全允许).
    pub allowed_exts: Vec<String>,
    /// 单文件上限字节.
    pub max_file_bytes: u64,
}

impl Default for SelectConfig {
    fn default() -> Self {
        Self {
            include: Vec::new(),
            exclude: Vec::new(),
            allowed_exts: vec![
                "rs", "go", "py", "ts", "tsx", "js", "jsx", "md", "toml", "json", "yaml",
                "yml", "sh",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
            max_file_bytes: 256 * 1024,
        }
    }
}

/// 单文件判决.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOutcome {
    pub path: String,
    pub selected: bool,
    pub reason: Option<ExclusionReason>,
}

/// 绝密路径片段 (OCR `default_secret_patterns` 核心子集).
fn is_secret_path(path: &str) -> bool {
    let lowered = path.to_ascii_lowercase();
    const MARKERS: [&str; 6] = [".env", ".pem", ".key", "secret", "credential", "token"];
    // `.env.example/sample/template` 放行 (OCR 同源语义).
    if lowered.contains(".env.example")
        || lowered.contains(".env.sample")
        || lowered.contains(".env.template")
    {
        return false;
    }
    MARKERS.iter().any(|marker| lowered.contains(marker))
}

/// 默认排除目录片段.
fn is_default_excluded(path: &str) -> bool {
    const SEGMENTS: [&str; 7] = [
        "target/",
        "node_modules/",
        ".git/",
        "dist/",
        "build/",
        "__pycache__/",
        ".venv/",
    ];
    SEGMENTS.iter().any(|seg| path.contains(seg))
}

fn glob_any(patterns: &[String], path: &str) -> bool {
    patterns.iter().any(|pattern| {
        glob::Pattern::new(pattern)
            .map(|matcher| matcher.matches(path))
            .unwrap_or(false)
    })
}

fn file_ext(path: &str) -> Option<String> {
    path.rsplit('.')
        .next()
        .filter(|ext| !ext.contains('/'))
        .map(|ext| ext.to_ascii_lowercase())
}

/// 确定性选择 — 同输入同输出, 顺序即优先级.
pub fn select_files(
    files: &[(String, u64)],
    config: &SelectConfig,
) -> Vec<SelectOutcome> {
    files
        .iter()
        .map(|(path, size)| {
            if is_secret_path(path) {
                return SelectOutcome {
                    path: path.clone(),
                    selected: false,
                    reason: Some(ExclusionReason::Secret),
                };
            }
            if glob_any(&config.exclude, path) {
                return SelectOutcome {
                    path: path.clone(),
                    selected: false,
                    reason: Some(ExclusionReason::UserRule),
                };
            }
            if !config.include.is_empty() && !glob_any(&config.include, path) {
                return SelectOutcome {
                    path: path.clone(),
                    selected: false,
                    reason: Some(ExclusionReason::UserRule),
                };
            }
            if !config.allowed_exts.is_empty() {
                let allowed = file_ext(path)
                    .is_some_and(|ext| config.allowed_exts.iter().any(|keep| keep == &ext));
                if !allowed {
                    return SelectOutcome {
                        path: path.clone(),
                        selected: false,
                        reason: Some(ExclusionReason::Extension),
                    };
                }
            }
            if is_default_excluded(path) {
                return SelectOutcome {
                    path: path.clone(),
                    selected: false,
                    reason: Some(ExclusionReason::DefaultPath),
                };
            }
            if *size > config.max_file_bytes {
                return SelectOutcome {
                    path: path.clone(),
                    selected: false,
                    reason: Some(ExclusionReason::TooLarge),
                };
            }
            SelectOutcome {
                path: path.clone(),
                selected: true,
                reason: None,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{ExclusionReason, SelectConfig, select_files};

    fn files() -> Vec<(String, u64)> {
        vec![
            ("src/a.rs".to_owned(), 100),
            (".env".to_owned(), 10),
            (".env.example".to_owned(), 10),
            ("target/x.rs".to_owned(), 100),
            ("big.rs".to_owned(), 10 * 1024 * 1024),
            ("img.png".to_owned(), 100),
        ]
    }

    #[test]
    fn priority_order() {
        let outcomes = select_files(&files(), &SelectConfig::default());
        let get = |path: &str| {
            outcomes
                .iter()
                .find(|outcome| outcome.path == path)
                .expect("present")
        };
        assert!(get("src/a.rs").selected);
        assert_eq!(get(".env").reason, Some(ExclusionReason::Secret));
        // `.env.example` 不算 secret (扩展名仍可另判, 但绝不进 secret 桶).
        assert_ne!(get(".env.example").reason, Some(ExclusionReason::Secret));
        assert_eq!(get("target/x.rs").reason, Some(ExclusionReason::DefaultPath));
        assert_eq!(get("big.rs").reason, Some(ExclusionReason::TooLarge));
        assert_eq!(get("img.png").reason, Some(ExclusionReason::Extension));
    }

    #[test]
    fn user_exclude_beats_include() {
        let config = SelectConfig {
            include: vec!["src/**".to_owned()],
            exclude: vec!["src/secret.rs".to_owned()],
            ..SelectConfig::default()
        };
        let outcomes = select_files(
            &[("src/a.rs".to_owned(), 10), ("src/secret.rs".to_owned(), 10)],
            &config,
        );
        assert!(outcomes[0].selected);
        assert!(!outcomes[1].selected);
    }
}
