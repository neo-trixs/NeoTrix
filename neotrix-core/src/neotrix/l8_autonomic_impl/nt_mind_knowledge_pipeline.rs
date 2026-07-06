//! NeoTrix 知识吸收管道 — GitHub/外部代码 → KB 蒸馏 → 去重更新

use std::collections::HashMap;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::neotrix::nt_memory_kb::KnowledgeBase;

// ============================================================
// 源类型 (使用唯一名避免冲突)
// ============================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KbSourceType {
    GitHub,
    ArXiv,
    Wikipedia,
    WebArticle,
    Paper,
    Documentation,
    CodeRepository,
    Blog,
}

impl std::fmt::Display for KbSourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KbSourceType::GitHub => write!(f, "GitHub"),
            KbSourceType::ArXiv => write!(f, "ArXiv"),
            KbSourceType::Wikipedia => write!(f, "Wikipedia"),
            KbSourceType::WebArticle => write!(f, "WebArticle"),
            KbSourceType::Paper => write!(f, "Paper"),
            KbSourceType::Documentation => write!(f, "Documentation"),
            KbSourceType::CodeRepository => write!(f, "CodeRepository"),
            KbSourceType::Blog => write!(f, "Blog"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceEntry {
    pub url: String,
    pub source_type: KbSourceType,
    pub title: String,
    pub kb_node_id: Option<String>,
    pub last_absorbed: i64,
    pub sha_hash: Option<String>,
    pub distill_summary: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsorbState {
    pub source_map: HashMap<String, SourceEntry>,
    pub total_absorbed: usize,
    pub last_panorama_update: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillResult {
    pub title: String,
    pub summary: String,
    pub core_concepts: Vec<String>,
    pub architecture_insights: Vec<String>,
    pub key_algorithms: Vec<String>,
    pub dependencies: Vec<String>,
    pub code_patterns: Vec<String>,
    pub confidence: f64,
}

// ============================================================
// 知识吸收管道
// ============================================================

pub struct KnowledgeAbsorptionPipeline {
    pub kb: Option<KnowledgeBase>,
    state: AbsorbState,
}

impl Default for KnowledgeAbsorptionPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeAbsorptionPipeline {
    pub fn new() -> Self {
        Self {
            kb: None,
            state: AbsorbState {
                source_map: HashMap::new(),
                total_absorbed: 0,
                last_panorama_update: 0,
            },
        }
    }

    pub fn attach_kb(&mut self, kb: KnowledgeBase) {
        self.kb = Some(kb);
    }

    pub fn absorb_url(&mut self, url: &str) -> Result<AbsorptionReport, String> {
        let url_key = url.to_string();
        if let Some(entry) = self.state.source_map.get(&url_key) {
            let age = Utc::now().timestamp() - entry.last_absorbed;
            if age < 86400 {
                return Ok(AbsorptionReport {
                    url: url.into(), source_type: KbSourceType::WebArticle,
                    action: "cached".into(), nodes_created: 0, edges_created: 0,
                    distil_summary: None,
                });
            }
        }
        if let Some(ref kb) = self.kb {
            if let Ok(Some(_existing)) = kb.find_node_by_url(url) {
                return Ok(AbsorptionReport {
                    url: url.into(), source_type: KbSourceType::WebArticle,
                    action: "cached".into(), nodes_created: 0, edges_created: 0,
                    distil_summary: None,
                });
            }
        }

        self.state.source_map.insert(url.to_string(), SourceEntry {
            url: url.into(), source_type: KbSourceType::WebArticle,
            title: url.into(), kb_node_id: None,
            last_absorbed: Utc::now().timestamp(),
            sha_hash: None, distill_summary: None, tags: vec![],
        });
        self.state.total_absorbed += 1;

        Ok(AbsorptionReport {
            url: url.into(), source_type: KbSourceType::WebArticle,
            action: "absorbed".into(), nodes_created: 1, edges_created: 0,
            distil_summary: None,
        })
    }

    pub fn absorb_github(&mut self, url: &str) -> Result<AbsorptionReport, String> {
        let parts: Vec<&str> = url.trim_end_matches('/').split('/').collect();
        let repo = parts.last().ok_or("无法解析仓库名")?.to_string();
        let owner = if parts.len() >= 2 { parts[parts.len()-2].to_string() } else { String::new() };

        let url_key = format!("github:{}/{}", owner, repo);

        // 去重检查
        if let Some(entry) = self.state.source_map.get(&url_key) {
            if Utc::now().timestamp() - entry.last_absorbed < 3600 {
                return Ok(AbsorptionReport {
                    url: url.into(), source_type: KbSourceType::GitHub,
                    action: "skipped".into(), nodes_created: 0, edges_created: 0,
                    distil_summary: None,
                });
            }
        }

        // 蒸馏
        let distill = DistillResult {
            title: repo.clone(),
            summary: format!("GitHub 仓库 {}/{}", owner, repo),
            core_concepts: vec![format!("{}/{}", owner, repo)],
            architecture_insights: vec![],
            key_algorithms: vec![],
            dependencies: vec![],
            code_patterns: vec![],
            confidence: 0.6,
        };

        self.state.source_map.insert(url_key, SourceEntry {
            url: url.into(), source_type: KbSourceType::GitHub,
            title: repo, kb_node_id: None,
            last_absorbed: Utc::now().timestamp(),
            sha_hash: None,
            distill_summary: Some(distill.summary.clone()),
            tags: vec![],
        });
        self.state.total_absorbed += 1;

        Ok(AbsorptionReport {
            url: url.into(), source_type: KbSourceType::GitHub,
            action: "absorbed".into(), nodes_created: 1, edges_created: 0,
            distil_summary: Some(distill.summary),
        })
    }

    /// 更新全景索引
    pub fn update_panorama(&mut self) -> Result<PanoramaReport, String> {
        let now = Utc::now().timestamp();
        let mut by_type: HashMap<String, usize> = HashMap::new();
        for entry in self.state.source_map.values() {
            *by_type.entry(format!("{}", entry.source_type)).or_insert(0) += 1;
        }
        self.state.last_panorama_update = now;
        Ok(PanoramaReport {
            total_sources: self.state.source_map.len(),
            by_type,
            updated_at: now,
        })
    }

    pub fn recent_sources(&self, n: usize) -> Vec<&SourceEntry> {
        let mut entries: Vec<_> = self.state.source_map.values().collect();
        entries.sort_by_key(|e| std::cmp::Reverse(e.last_absorbed));
        entries.into_iter().take(n).collect()
    }

    pub fn stats(&self) -> KbPipelineStats {
        KbPipelineStats {
            total_sources: self.state.source_map.len(),
            total_absorbed: self.state.total_absorbed,
        }
    }
}

// ============================================================
// 报告类型
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsorptionReport {
    pub url: String,
    pub source_type: KbSourceType,
    pub action: String,
    pub nodes_created: usize,
    pub edges_created: usize,
    pub distil_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanoramaReport {
    pub total_sources: usize,
    pub by_type: HashMap<String, usize>,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbPipelineStats {
    pub total_sources: usize,
    pub total_absorbed: usize,
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_new() {
        let pipe = KnowledgeAbsorptionPipeline::new();
        assert_eq!(pipe.stats().total_sources, 0);
    }

    #[test]
    fn test_absorb_url_dedup() {
        let mut pipe = KnowledgeAbsorptionPipeline::new();
        let r1 = pipe.absorb_url("https://example.com/a").expect("first");
        assert_eq!(r1.action, "absorbed");
        let r2 = pipe.absorb_url("https://example.com/a").expect("second");
        assert_eq!(r2.action, "cached");
    }

    #[test]
    fn test_absorb_github() {
        let mut pipe = KnowledgeAbsorptionPipeline::new();
        let r = pipe.absorb_github("https://github.com/rust-lang/rust").expect("github");
        assert_eq!(r.source_type, KbSourceType::GitHub);
    }

    #[test]
    fn test_panorama() {
        let mut pipe = KnowledgeAbsorptionPipeline::new();
        let _ = pipe.absorb_url("https://a.com").unwrap();
        let _ = pipe.absorb_github("https://github.com/x/y").unwrap();
        let pan = pipe.update_panorama().expect("panorama");
        assert_eq!(pan.total_sources, 2);
    }

    #[test]
    fn test_source_type_display() {
        assert_eq!(KbSourceType::GitHub.to_string(), "GitHub");
        assert_eq!(KbSourceType::ArXiv.to_string(), "ArXiv");
    }
}
