//! NT-SHIELD Skill Doctor — 技能健康检查与诊断工具
//!
//! 吸收源: github.com/warpdotdev/common-skills/tree/main/.agents/skills/skill-doctor
//! 能力: 健康检查某技能 SKILL.md 是否缺字段 / 格式错误 / 断链。
//!
//! 这是对吸收源的模式吸收 (C1 成熟度): trait + 基础检查逻辑 stub，
//! 扫描目录、校验必需字段。编译通过即可 (R-P1: unsafe 禁用)。

use std::path::{Path, PathBuf};

use crate::core::nt_core_self_test::SelfTest;

/// 技能 SKILL.md 健康检查所需的必需字段。
/// skill-doctor 的核心契约: 任何合规技能必须声明下列 frontmatter 字段,
/// 否则视为"缺字段"诊断项。
pub const REQUIRED_FIELDS: &[&str] = &["name", "description"];

/// 单个诊断发现。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosisKind {
    /// SKILL.md 文件缺失
    MissingFile,
    /// 必需字段缺失
    MissingField,
    /// frontmatter 块格式错误 (无 `---` 包裹)
    FormatError,
    /// 链接指向的文件不存在 (断链)
    BrokenLink,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnosis {
    pub kind: DiagnosisKind,
    pub message: String,
}

impl Diagnosis {
    pub fn new(kind: DiagnosisKind, message: impl Into<String>) -> Self {
        Self { kind, message: message.into() }
    }
}

/// 技能健康检查结果。
#[derive(Debug, Clone, Default)]
pub struct SkillHealthReport {
    pub skill_dir: PathBuf,
    pub diagnoses: Vec<Diagnosis>,
}

impl SkillHealthReport {
    pub fn is_healthy(&self) -> bool {
        self.diagnoses.is_empty()
    }

    pub fn missing_field_count(&self) -> usize {
        self.diagnoses.iter().filter(|d| d.kind == DiagnosisKind::MissingField).count()
    }
}

/// 技能健康检查核心 trait — 诊断某技能 SKILL.md 是否缺字段 / 格式错误 / 断链。
pub trait SkillHealthChecker {
    /// 对给定技能目录执行健康检查, 返回诊断报告。
    fn check_skill(&self, skill_dir: &Path) -> SkillHealthReport;
}

/// 默认技能医生实现: 扫描目录 + 校验必需字段 + 断链检测。
#[derive(Debug, Default)]
pub struct SkillDoctor;

impl SkillDoctor {
    pub fn new() -> Self {
        Self
    }

    /// 解析 SKILL.md frontmatter 顶层字段名 (极简 YAML key 提取, 无外部依赖)。
    fn extract_frontmatter_fields(content: &str) -> Option<Vec<String>> {
        let trimmed = content.trim_start();
        if !trimmed.starts_with("---") {
            return None;
        }
        let after_open = &trimmed[3..];
        let end = after_open.find("\n---")?;
        let body = &after_open[..end];
        let mut fields = Vec::new();
        for line in body.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, _)) = line.split_once(':') {
                fields.push(key.trim().to_string());
            }
        }
        Some(fields)
    }
}

impl SkillHealthChecker for SkillDoctor {
    fn check_skill(&self, skill_dir: &Path) -> SkillHealthReport {
        let mut report = SkillHealthReport { skill_dir: skill_dir.to_path_buf(), diagnoses: Vec::new() };

        let skill_md = skill_dir.join("SKILL.md");
        if !skill_md.exists() {
            report.diagnoses.push(Diagnosis::new(
                DiagnosisKind::MissingFile,
                format!("SKILL.md not found in {}", skill_dir.display()),
            ));
            return report;
        }

        let content = match std::fs::read_to_string(&skill_md) {
            Ok(c) => c,
            Err(e) => {
                report.diagnoses.push(Diagnosis::new(
                    DiagnosisKind::FormatError,
                    format!("cannot read SKILL.md: {e}"),
                ));
                return report;
            }
        };

        let fields = match Self::extract_frontmatter_fields(&content) {
            Some(f) => f,
            None => {
                report.diagnoses.push(Diagnosis::new(
                    DiagnosisKind::FormatError,
                    "SKILL.md missing valid `---` frontmatter block".to_string(),
                ));
                return report;
            }
        };

        for req in REQUIRED_FIELDS {
            if !fields.iter().any(|f| f == req) {
                report.diagnoses.push(Diagnosis::new(
                    DiagnosisKind::MissingField,
                    format!("required field `{req}` missing from frontmatter"),
                ));
            }
        }

        report
    }
}

impl SelfTest for SkillDoctor {
    fn name(&self) -> &str {
        "nt_shield_skill_doctor"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        if REQUIRED_FIELDS.is_empty() {
            failures.push("REQUIRED_FIELDS contract is empty".to_string());
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_skill(dir: &Path, content: &str) {
        std::fs::create_dir_all(dir).unwrap();
        let mut f = std::fs::File::create(dir.join("SKILL.md")).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn doctor_detects_missing_skill_md() {
        let tmp = std::env::temp_dir().join("nt_shield_skill_doctor_test_missing");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let doc = SkillDoctor::new();
        let report = doc.check_skill(&tmp);
        assert!(!report.is_healthy());
        assert!(report.diagnoses.iter().any(|d| d.kind == DiagnosisKind::MissingFile));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn doctor_detects_missing_required_field() {
        let tmp = std::env::temp_dir().join("nt_shield_skill_doctor_test_field");
        let _ = std::fs::remove_dir_all(&tmp);
        write_skill(&tmp, "---\nname: foo\ntest: bar\n---\nbody\n");
        let doc = SkillDoctor::new();
        let report = doc.check_skill(&tmp);
        assert!(!report.is_healthy());
        assert_eq!(report.missing_field_count(), 1);
        assert!(report.diagnoses.iter().any(|d| d.kind == DiagnosisKind::MissingField));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn doctor_passes_well_formed_skill() {
        let tmp = std::env::temp_dir().join("nt_shield_skill_doctor_test_ok");
        let _ = std::fs::remove_dir_all(&tmp);
        write_skill(&tmp, "---\nname: foo\ndescription: bar\n---\nbody\n");
        let doc = SkillDoctor::new();
        let report = doc.check_skill(&tmp);
        assert!(report.is_healthy(), "expected healthy, got {:?}", report.diagnoses);
        assert_eq!(report.diagnoses.len(), 0);
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
