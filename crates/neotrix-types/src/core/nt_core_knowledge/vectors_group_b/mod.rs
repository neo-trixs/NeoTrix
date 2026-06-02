mod cli_agent_tools;
mod memory_systems;
mod infrastructure_cosmology;
mod tool_ecosystem;
mod skills_and_misc;

use crate::core::CapabilityVector;
use crate::core::nt_core_knowledge::KnowledgeSource;

pub(super) fn capability_vector_group_b(s: &KnowledgeSource) -> CapabilityVector {
    if let Some(cv) = cli_agent_tools::handle_cli_agent_tools(s) {
        return cv;
    }
    if let Some(cv) = memory_systems::handle_memory_systems(s) {
        return cv;
    }
    if let Some(cv) = infrastructure_cosmology::handle_infrastructure_cosmology(s) {
        return cv;
    }
    if let Some(cv) = tool_ecosystem::handle_tool_ecosystem(s) {
        return cv;
    }
    skills_and_misc::handle_skills_and_misc(s)
}
