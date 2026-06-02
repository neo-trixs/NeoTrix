use std::time::Instant;
use serde::{Serialize, Deserialize};

/// Skill 来源（ECC 兼容扩展）
#[derive(Debug, Clone, PartialEq)]
pub enum SkillSource {
    LocalDir(String),
    GitHub { owner: String, repo: String, path: String, branch: Option<String> },
    Npm(String),
    Pip(String),
    Registry(String),
    Url(String),
    /// ECC 社区注册表
    EccCommunity { skill_id: String, version: String },
}

/// Skill 元数据（ECC 兼容 Frontmatter 格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMeta {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: Option<String>,
    pub origin: Option<String>,
    pub triggers: Vec<String>,
    pub condition: Option<String>,
    pub requires_tools: Vec<String>,
    pub requires_capabilities: Vec<String>,
    /// ATT&CK 技术 ID (Decepticon 风格): T1595, T1046 等
    pub mitre_attack_ids: Vec<String>,
}

/// Skill 执行统计
#[derive(Debug, Clone)]
pub struct SkillStats {
    /// 使用次数
    pub use_count: u64,
    /// 成功次数
    pub success_count: u64,
    /// 置信度评分 (0.0 - 1.0)
    pub confidence: f64,
    /// 平均执行耗时
    pub avg_execution_ms: f64,
    /// 最后使用时间
    pub last_used: Option<Instant>,
    /// ECC 风格：技能版本演化历史
    pub evolution_history: Vec<String>,
}

/// Skill 定义
#[derive(Debug, Clone)]
pub struct Skill {
    pub meta: SkillMeta,
    pub source: SkillSource,
    /// Skill 内容（prompt 模板、代码等）
    pub content: String,
    /// 注入到 system prompt 的文本
    pub system_prompt: String,
    /// ECC 风格：执行统计与置信度
    pub stats: SkillStats,
}

impl Skill {
    pub fn new(meta: SkillMeta, source: SkillSource, content: String, system_prompt: String) -> Self {
        Self {
            stats: SkillStats {
                use_count: 0,
                success_count: 0,
                confidence: if content.is_empty() || system_prompt.is_empty() { 0.0 } else { 0.5 },
                avg_execution_ms: 0.0,
                last_used: None,
                evolution_history: Vec::new(),
            },
            meta, source, content, system_prompt,
        }
    }

    /// 更新置信度（ECC 持续学习模式）
    pub fn update_confidence(&mut self, success: bool, execution_ms: u64) {
        self.stats.use_count += 1;
        if success { self.stats.success_count += 1; }
        self.stats.last_used = Some(Instant::now());
        self.stats.avg_execution_ms = (self.stats.avg_execution_ms * (self.stats.use_count as f64 - 1.0) + execution_ms as f64) / self.stats.use_count as f64;
        let alpha = 0.3;
        let reward = if success { 1.0 } else { -0.5 };
        self.stats.confidence = (self.stats.confidence * (1.0 - alpha) + reward * alpha).clamp(0.0, 1.0);
    }
}

/// Skill 执行结果（含 ECC 风格的置信度反馈）
#[derive(Debug, Clone)]
pub struct SkillOutput {
    pub skill_name: String,
    pub success: bool,
    pub output: String,
    pub execution_time_ms: u64,
    /// 执行后的置信度调整量（ECC 持续学习）
    pub confidence_delta: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_new_empty_content_zero_confidence() {
        let meta = SkillMeta {
            name: "test".into(), description: "desc".into(), version: "1.0".into(),
            author: None, origin: None, triggers: vec![], condition: None,
            requires_tools: vec![], requires_capabilities: vec![], mitre_attack_ids: vec![],
        };
        let skill = Skill::new(meta, SkillSource::LocalDir("/tmp".into()), "".into(), "".into());
        assert!((skill.stats.confidence - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_skill_new_with_content_half_confidence() {
        let meta = SkillMeta {
            name: "test".into(), description: "desc".into(), version: "1.0".into(),
            author: None, origin: None, triggers: vec![], condition: None,
            requires_tools: vec![], requires_capabilities: vec![], mitre_attack_ids: vec![],
        };
        let skill = Skill::new(meta, SkillSource::GitHub {
            owner: "user".into(), repo: "repo".into(), path: "/".into(), branch: None,
        }, "content".into(), "prompt".into());
        assert!((skill.stats.confidence - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_skill_update_confidence_success() {
        let meta = SkillMeta {
            name: "t".into(), description: "d".into(), version: "1".into(),
            author: None, origin: None, triggers: vec![], condition: None,
            requires_tools: vec![], requires_capabilities: vec![], mitre_attack_ids: vec![],
        };
        let mut skill = Skill::new(meta, SkillSource::Registry("reg".into()), "c".into(), "p".into());
        assert_eq!(skill.stats.use_count, 0);
        skill.update_confidence(true, 100);
        assert_eq!(skill.stats.use_count, 1);
        assert_eq!(skill.stats.success_count, 1);
    }

    #[test]
    fn test_skill_update_confidence_failure() {
        let meta = SkillMeta {
            name: "t".into(), description: "d".into(), version: "1".into(),
            author: None, origin: None, triggers: vec![], condition: None,
            requires_tools: vec![], requires_capabilities: vec![], mitre_attack_ids: vec![],
        };
        let mut skill = Skill::new(meta, SkillSource::Npm("pkg".into()), "c".into(), "p".into());
        skill.update_confidence(false, 200);
        assert_eq!(skill.stats.use_count, 1);
        assert_eq!(skill.stats.success_count, 0);
    }

    #[test]
    fn test_skill_confidence_clamped() {
        let meta = SkillMeta {
            name: "t".into(), description: "d".into(), version: "1".into(),
            author: None, origin: None, triggers: vec![], condition: None,
            requires_tools: vec![], requires_capabilities: vec![], mitre_attack_ids: vec![],
        };
        let mut skill = Skill::new(meta, SkillSource::Url("https://x.com".into()), "c".into(), "p".into());
        for _ in 0..10 {
            skill.update_confidence(false, 50);
        }
        assert!(skill.stats.confidence >= 0.0);
        for _ in 0..20 {
            skill.update_confidence(true, 50);
        }
        assert!(skill.stats.confidence <= 1.0);
    }

    #[test]
    fn test_skill_stats_avg_execution_ms() {
        let meta = SkillMeta {
            name: "t".into(), description: "d".into(), version: "1".into(),
            author: None, origin: None, triggers: vec![], condition: None,
            requires_tools: vec![], requires_capabilities: vec![], mitre_attack_ids: vec![],
        };
        let mut skill = Skill::new(meta, SkillSource::EccCommunity {
            skill_id: "s1".into(), version: "1.0".into(),
        }, "c".into(), "p".into());
        skill.update_confidence(true, 100);
        skill.update_confidence(true, 300);
        assert!((skill.stats.avg_execution_ms - 200.0).abs() < 1e-6);
    }

    #[test]
    fn test_skill_source_partial_eq() {
        let a = SkillSource::LocalDir("/a".into());
        let b = SkillSource::LocalDir("/b".into());
        assert_ne!(a, b);
        assert_eq!(a, a);
    }
}
