#![allow(dead_code)]
// modules.rs — decomposed into 8 domain sub-files
// See: modules_core.rs, modules_agent.rs, modules_kb.rs, modules_e8.rs,
//      modules_jepa.rs, modules_storage.rs, modules_vision.rs, modules_a2a.rs

use crate::core::nt_core_consciousness::cognitive_load::ThinkingMode;

pub(crate) fn tm_to_str(tm: ThinkingMode) -> String {
    match tm {
        ThinkingMode::Fast => "cognitive_load_tick:Fast".to_string(),
        ThinkingMode::Balanced => "cognitive_load_tick:Balanced".to_string(),
        ThinkingMode::Deep => "cognitive_load_tick:Deep".to_string(),
    }
}
