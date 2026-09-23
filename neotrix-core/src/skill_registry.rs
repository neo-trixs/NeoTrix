//! SkillRegistry — Layer-3 路由门面（T27a，E3 新建）。
//!
//! 委托 `skill_loader::SkillLoader`；路由场景不抛错（空 Vec 代替）。
//! 纯发现层：评分/进化归 `SkillCandidate`（见 skill_evolution），本模块不碰。

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::l1_action::nt_infra_agent_card::AgentCardRegistry;
use crate::skill_loader::{ResolvedSkill, SkillFilter, SkillLoader};

/// Skill 注册门面：发现＋触发匹配。
pub struct SkillRegistry {
    loader: SkillLoader,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            loader: SkillLoader::new(),
        }
    }

    /// 指定目录构造（测试/隔离用）。
    pub fn with_dirs(dirs: Vec<PathBuf>) -> Self {
        Self {
            loader: SkillLoader::with_dirs(dirs),
        }
    }

    /// 触发匹配（E3 Layer-3 入口）：子串语义与 loader 一致；空 keyword 返空。
    pub fn match_trigger(&mut self, keyword: &str) -> Vec<ResolvedSkill> {
        if keyword.trim().is_empty() {
            return Vec::new();
        }
        let filter = SkillFilter {
            triggers: vec![keyword.to_string()],
            ..Default::default()
        };
        self.loader.search_skills(&filter).map_or_else(
            |_| Vec::new(),
            |results| results.into_iter().map(|r| r.skill).collect(),
        )
    }

    /// 全量发现（透传）。
    pub fn discover(&mut self) -> Vec<ResolvedSkill> {
        self.loader.list_skills().unwrap_or_default()
    }

    /// 按名加载（透传）。
    pub fn get_skill(&mut self, name: &str) -> Option<ResolvedSkill> {
        self.loader.load_skill(name).ok()
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ─── E3 三层路由决议（T27c，由死目录 dispatch.rs 迁入活文件） ───────────────
// 死目录 `l5_cognition/consciousness_core/` 未声明编译，原实现永不链接；
// 本实现只依赖活模块（SkillLoader＋AgentCardRegistry），语义等价、字段兼容。

/// 路由决议：Skill＞Agent＞Static＞DirectLlm（优先级降序）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityRouteDecision {
    Skill {
        skill_id: String,
        agent_id: Option<String>,
    },
    Agent {
        agent_id: String,
    },
    /// 静态表兜底位：静态表本体在死目录，复活前由调用方（orchestrator）填充；
    /// 本函数暂不产生此变体（直落 DirectLlm），见注记。
    Static {
        capability: String,
        layer: String,
        role: String,
    },
    DirectLlm,
}

/// 单次执行上下文（E3）：工具只存名，避免跨层类型。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub workspace_id: String,
    pub agent_id: String,
    pub task_id: Option<String>,
    pub skill_id: Option<String>,
    #[serde(default)]
    pub shared_memory: serde_json::Value,
    #[serde(default)]
    pub available_tools: Vec<String>,
}

/// 三层路由（活路径版）：Layer-3 Skill 触发 → Layer-2 Agent 能力 → DirectLlm。
/// Layer-1 静态兜底暂由 orchestrator 侧处理（静态表在死目录，待模块归属裁决）。
pub fn route_entity_aware(
    input: &str,
    workspace_id: &str,
    skill_registry: &mut SkillRegistry,
    agent_registry: &AgentCardRegistry,
) -> EntityRouteDecision {
    let query = input.trim();
    if query.is_empty() {
        return EntityRouteDecision::DirectLlm;
    }
    // Layer-3：首个触发命中的 skill；agent 绑定取其 tags＋name 作能力愿望单。
    if let Some(skill) = skill_registry.match_trigger(query).into_iter().next() {
        let mut wanted = skill.tags.clone();
        wanted.push(skill.name.clone());
        let agent_id = agent_registry
            .find_best_agent_for(&wanted)
            .map(|c| c.id.clone());
        return EntityRouteDecision::Skill {
            skill_id: skill.name,
            agent_id,
        };
    }
    // Layer-2：全文作能力查询。
    if let Some(agent) = agent_registry.find_best_agent_for(&[query.to_string()]) {
        return EntityRouteDecision::Agent {
            agent_id: agent.id.clone(),
        };
    }
    // Layer-0：直接 LLM（Layer-1 静态表待死目录裁决后接入）。
    let _ = workspace_id;
    EntityRouteDecision::DirectLlm
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_trigger_empty() {
        let mut reg = SkillRegistry::with_dirs(vec![PathBuf::from("/nonexistent-nt-dir")]);
        assert!(reg.match_trigger("").is_empty());
        assert!(reg.match_trigger("   ").is_empty());
    }

    #[test]
    fn test_discover_error_path_no_panic() {
        let mut reg = SkillRegistry::with_dirs(vec![PathBuf::from("/nonexistent-nt-dir")]);
        assert!(reg.discover().is_empty());
        assert!(reg.get_skill("ghost").is_none());
    }

    #[test]
    fn test_route_empty_input_goes_direct() {
        let mut skills = SkillRegistry::with_dirs(vec![PathBuf::from("/nonexistent-nt-dir")]);
        let agents = AgentCardRegistry::new();
        assert_eq!(
            route_entity_aware("", "ws-1", &mut skills, &agents),
            EntityRouteDecision::DirectLlm
        );
        // 空注册表＋未知输入 → DirectLlm（Layer-1 静态表待接入）
        assert_eq!(
            route_entity_aware("zxqw kjrblp vapour", "ws-1", &mut skills, &agents),
            EntityRouteDecision::DirectLlm
        );
    }

    #[test]
    fn test_execution_context_roundtrip() {
        let ctx = ExecutionContext {
            workspace_id: "ws-1".into(),
            agent_id: "a-1".into(),
            task_id: Some("t-1".into()),
            skill_id: None,
            shared_memory: serde_json::json!({"k": 1}),
            available_tools: vec!["tool-a".into()],
        };
        let back: ExecutionContext =
            serde_json::from_value(serde_json::to_value(&ctx).expect("ser")).expect("de");
        assert_eq!(back.workspace_id, "ws-1");
        assert_eq!(back.available_tools, vec!["tool-a".to_string()]);
    }
}
