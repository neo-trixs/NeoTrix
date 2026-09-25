//! 三层实体路由（route_entity_aware）（由 `dispatch.rs` 纯搬移拆分，行为零变更）

use super::nt_dispatch_routes::decompose_instruction;
use super::nt_dispatch_types::EntityRouteDecision;

// ─── 三层路由＋执行上下文（T27c＋T29，蓝图 V3 §5 E3）──────────────────────────

/// 三层路由：L3 skill 触发 → L2 agent 能力匹配 → L1 静态表兜底 → DirectLlm。
///
/// L1 复用既有 `decompose_instruction` 入口，不动 `CAPABILITY_ROUTES` 内容；
/// 跨层引用只向下（L5 → L0/L1），`workspace_id` 用于 workspace 内 agent 优先。
pub fn route_entity_aware(
    input: &str,
    workspace_id: &str,
    skill_registry: &mut crate::skill_registry::SkillRegistry,
    agent_registry: &crate::l1_action::nt_infra_agent_card::AgentCardRegistry,
) -> EntityRouteDecision {
    let query = input.trim();
    if query.is_empty() {
        return EntityRouteDecision::DirectLlm;
    }
    // Layer-3：skill 触发命中（新门面 SkillRegistry::match_trigger）。
    if let Some(skill) = skill_registry.match_trigger(query).into_iter().next() {
        return bind_skill_to_agent(&skill.name, &skill.triggers, agent_registry);
    }
    // Layer-2：agent 能力匹配（wanted 取自既有静态分解的能力标签）。
    let tasks = decompose_instruction(query);
    let mut wanted: Vec<String> = Vec::new();
    for task in &tasks {
        if !wanted.iter().any(|w| w == &task.capability_tag) {
            wanted.push(task.capability_tag.clone());
        }
    }
    // workspace 内优先（只读既有 find_by_workspace 公共入口，同 match_capabilities 交集规则）。
    let workspace_best = agent_registry
        .find_by_workspace(workspace_id)
        .into_iter()
        .map(|card| {
            let overlap = card
                .capabilities
                .iter()
                .filter(|cap| wanted.iter().any(|w| w == &cap.name))
                .count();
            (card.id.clone(), overlap)
        })
        .filter(|(_, overlap)| *overlap > 0)
        .max_by_key(|(_, overlap)| *overlap)
        .map(|(id, _)| id);
    if let Some(agent_id) = workspace_best {
        return EntityRouteDecision::Agent { agent_id };
    }
    if let Some(card) = agent_registry.find_best_agent_for(&wanted) {
        return EntityRouteDecision::Agent {
            agent_id: card.id.clone(),
        };
    }
    // Layer-1：静态表兜底（首个分解任务；纯 orchestration 视为无路由）。
    if let Some(task) = tasks.into_iter().next() {
        if task.capability_tag != "orchestration" {
            return EntityRouteDecision::Static {
                capability: task.capability_tag,
                layer: task.domain,
                role: task.specialist,
            };
        }
    }
    EntityRouteDecision::DirectLlm
}

/// Skill 分支决议：agent 绑定可选（仅已声明该 skill/同名能力的 agent 才绑定）。
fn bind_skill_to_agent(
    skill_id: &str,
    skill_triggers: &[String],
    agent_registry: &crate::l1_action::nt_infra_agent_card::AgentCardRegistry,
) -> EntityRouteDecision {
    let mut wanted: Vec<String> = vec![skill_id.to_string()];
    wanted.extend(skill_triggers.iter().cloned());
    let agent_id = agent_registry
        .find_best_agent_for(&wanted)
        .map(|card| card.id.clone());
    EntityRouteDecision::Skill {
        skill_id: skill_id.to_string(),
        agent_id,
    }
}

#[cfg(test)]
mod entity_routing_tests {
    use super::*;
    use super::super::nt_dispatch_types::EntityRouteDecision;
    use std::path::PathBuf;

    fn empty_registries() -> (
        crate::skill_registry::SkillRegistry,
        crate::l1_action::nt_infra_agent_card::AgentCardRegistry,
    ) {
        (
            crate::skill_registry::SkillRegistry::with_dirs(vec![PathBuf::from(
                "/nonexistent-nt-dir",
            )]),
            crate::l1_action::nt_infra_agent_card::AgentCardRegistry::new(),
        )
    }

    #[test]
    fn test_route_skill_branch_binds_optional_agent() {
        // 注：Skill 真命中需 SkillLoader 读盘（fixture 在 skills/ 下），单测不碰文件系统；
        // 此处锁定 Skill 分支的纯决议逻辑：skill_id 必携带，无声明该能力的 agent 时绑定为 None。
        let agent_registry = crate::l1_action::nt_infra_agent_card::AgentCardRegistry::new();
        let decision = bind_skill_to_agent("tdd", &["测试".to_string()], &agent_registry);
        assert!(
            matches!(decision, EntityRouteDecision::Skill { ref skill_id, agent_id: None } if skill_id == "tdd"),
            "期望 Skill(tdd, None)，实际 {decision:?}"
        );
    }

    #[test]
    fn test_route_all_miss_falls_back_to_static_or_direct() {
        // 空 registry（不存在目录 → 空命中，不碰文件系统）＋空 agent 注册表。
        let (mut skills, agents) = empty_registries();
        let static_hit = route_entity_aware("请做漏洞扫描", "ws-test", &mut skills, &agents);
        assert!(
            matches!(static_hit, EntityRouteDecision::Static { ref capability, .. } if capability == "agentic_scan"),
            "静态兜底应命中 agentic_scan，实际 {static_hit:?}"
        );
        let direct = route_entity_aware("zxqw kjrblp vapour", "ws-test", &mut skills, &agents);
        assert!(
            matches!(direct, EntityRouteDecision::DirectLlm),
            "无意义输入应走 DirectLlm，实际 {direct:?}"
        );
        let empty = route_entity_aware("   ", "ws-test", &mut skills, &agents);
        assert!(
            matches!(empty, EntityRouteDecision::DirectLlm),
            "空输入应走 DirectLlm，实际 {empty:?}"
        );
    }
}
