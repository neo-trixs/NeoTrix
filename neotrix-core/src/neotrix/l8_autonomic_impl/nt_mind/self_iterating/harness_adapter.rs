//! HarnessAdapter 适配器 — 类型定义在 core/nt_core_harness 中（防 L5→L8 反向依赖），
//! KB 持久化方法通过 HarnessKbExt trait 扩展至此层。

pub use crate::core::nt_core_harness::{HarnessAdapter, HarnessProfile};

use crate::neotrix::nt_memory_kb::{KnowledgeBase, KnowledgeNode, NodeType};
use std::collections::HashMap;

/// HarnessAdapter 的 KB 持久化扩展。
/// save_to_kb / load_from_kb 需要 KnowledgeBase 依赖，故留在 neotrix 层。
pub trait HarnessKbExt {
    fn save_to_kb(&self, kb: &KnowledgeBase) -> Result<usize, String>;
    fn load_from_kb(kb: &KnowledgeBase) -> Result<HarnessAdapter, String>;
}

impl HarnessKbExt for HarnessAdapter {
    fn save_to_kb(&self, kb: &KnowledgeBase) -> Result<usize, String> {
        let mut count = 0;
        for (env, profile) in &self.profiles {
            let json =
                serde_json::to_value(profile).map_err(|e| format!("Serialize profile: {}", e))?;
            let title = format!("HarnessProfile: {}", env);
            let summary = format!(
                "Harness profile for environment '{}' from model '{}' with {} contracts, {} skills, delta={}",
                env,
                profile.source_model,
                profile.environment_contracts.len(),
                profile.procedural_skills.len(),
                profile.performance_delta,
            );
            let node = KnowledgeNode {
                id: format!("harness-profile-{}", env),
                node_type: NodeType::HarnessProfile,
                title,
                summary: Some(summary),
                content: None,
                url: Some(format!("harness://profile/{}", env)),
                domain: Some("harness".to_string()),
                language: "en".to_string(),
                confidence: 0.9,
                importance: 0.6,
                created_at: 0,
                updated_at: 0,
                access_count: 0,
                metadata: Some(json),
                temporal: None,
                supersedes: None,
                source_episode: None,
            };
            kb.insert_node(&node)?;
            count += 1;
        }
        Ok(count)
    }

    fn load_from_kb(kb: &KnowledgeBase) -> Result<HarnessAdapter, String> {
        let nodes = kb.search_by_type(&NodeType::HarnessProfile, 100)?;
        let mut profiles = HashMap::new();
        for node in &nodes {
            if let Some(ref meta) = node.metadata {
                if let Ok(profile) = serde_json::from_value::<HarnessProfile>(meta.clone()) {
                    let env = node
                        .title
                        .strip_prefix("HarnessProfile: ")
                        .unwrap_or(&node.title)
                        .to_string();
                    profiles.insert(env, profile);
                }
            }
        }
        Ok(HarnessAdapter {
            profiles,
            active: None,
            transfer_history: Vec::new(),
        })
    }
}
