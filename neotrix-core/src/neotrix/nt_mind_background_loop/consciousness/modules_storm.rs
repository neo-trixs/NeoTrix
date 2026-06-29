// STORM/Co-STORM research pipeline — 4-phase multi-perspective handler
//
// Phase 1: PerspectiveDiscovery — generate 6-8 diverse viewpoints
// Phase 2: ConversationSimulation — simulated dialogue + contradiction mapping
// Phase 3: Synthesis — outline-driven report
// Phase 4: SelfCritique — peer-review polish loop

use super::ConsciousnessIntegration;
use crate::core::nt_core_experience::storm_engine::{
    default_perspective_names, StormEngine, StormPhase,
};

impl ConsciousnessIntegration {
    /// Phase 1: Discover 6-8 perspectives for the current topic.
    /// Transitions from Idle -> PerspectiveDiscovery, or advances
    /// an already-started research pipeline.
    pub fn handle_storm_perspective_tick(&mut self) -> String {
        if self.storm.is_none() {
            self.storm = Some(StormEngine::new("consciousness:auto"));
            return "storm:init".into();
        }
        let engine = match self.storm.as_mut() {
            Some(e) => e,
            None => {
                log::error!("MODULES: engine not init");
                return "engine:unavailable".into();
            }
        };

        if !engine.config.enabled {
            return "storm:disabled".into();
        }

        // Initialize perspectives if empty
        if engine.perspectives.is_empty() {
            let names = default_perspective_names();
            for name in names {
                engine.perspectives.push(
                    crate::core::nt_core_experience::storm_engine::PerspectiveDefinition {
                        name: name.to_string(),
                        assumptions: Vec::new(),
                        insights: Vec::new(),
                        blind_spots: Vec::new(),
                        questions: Vec::new(),
                        vsa_perspective: None,
                    },
                );
            }
            engine.phase = StormPhase::PerspectiveDiscovery;
            engine.cycle += 1;
            format!("storm:perspectives_init={}", engine.perspectives.len())
        } else {
            // Advance to conversation phase
            engine.advance_phase();
            engine.cycle += 1;
            format!("storm:advance->{}", engine.phase.label())
        }
    }

    /// Phase 2: Simulate conversation between perspectives and build ContradictionMap.
    pub fn handle_storm_conversation_tick(&mut self) -> String {
        if self.storm.is_none() {
            self.storm = Some(StormEngine::new("consciousness:auto"));
            return "storm:init".into();
        }
        let engine = match self.storm.as_mut() {
            Some(e) => e,
            None => {
                log::error!("MODULES: engine not init");
                return "engine:unavailable".into();
            }
        };

        if !engine.config.enabled || engine.perspectives.is_empty() {
            return "storm:skip".into();
        }

        // If contradiction map doesn't exist yet, build it from perspective pairs
        if engine.contradiction_map.is_none() {
            let n_perspectives = engine.perspectives.len();
            let n_pairs = n_perspectives * (n_perspectives - 1) / 2;
            let map = crate::core::nt_core_experience::storm_engine::ContradictionMap {
                agreements: vec!["pending".to_string()],
                contradictions: Vec::new(),
                knowledge_gaps: Vec::new(),
                reliability_ratings: Vec::new(),
            };
            engine.contradiction_map = Some(map);
            engine.phase = StormPhase::ConversationSimulation;
            engine.cycle += 1;
            format!("storm:map_initialized_pairs={}", n_pairs)
        } else {
            engine.advance_phase();
            engine.cycle += 1;
            format!("storm:advance->{}", engine.phase.label())
        }
    }

    /// Phase 3: Synthesize outline-driven report.
    pub fn handle_storm_synthesis_tick(&mut self) -> String {
        if self.storm.is_none() {
            self.storm = Some(StormEngine::new("consciousness:auto"));
            return "storm:init".into();
        }
        let engine = match self.storm.as_mut() {
            Some(e) => e,
            None => {
                log::error!("MODULES: engine not init");
                return "engine:unavailable".into();
            }
        };

        if !engine.config.enabled {
            return "storm:skip".into();
        }

        if engine.report.is_none() && engine.contradiction_map.is_some() {
            let cm = match engine.contradiction_map.as_ref() {
                Some(cm) => cm,
                None => {
                    log::error!("[modules_storm] contradiction_map not initialized");
                    return "storm:contradiction_map_unavailable".into();
                }
            };
            let report = crate::core::nt_core_experience::storm_engine::StormReport {
                title: engine.topic.clone(),
                introduction: format!(
                    "Multi-perspective analysis of '{}' covering {} perspectives.",
                    engine.topic,
                    engine.perspectives.len()
                ),
                sections: Vec::new(),
                contradiction_map: cm.clone(),
                recommendations: Vec::new(),
                reliability_assessment: "pending".to_string(),
            };
            engine.report = Some(report);
            engine.phase = StormPhase::Synthesis;
            engine.cycle += 1;
            "storm:report_initialized".into()
        } else {
            engine.advance_phase();
            engine.cycle += 1;
            format!("storm:advance->{}", engine.phase.label())
        }
    }

    /// Phase 4: Self-critique and polish.
    pub fn handle_storm_critique_tick(&mut self) -> String {
        if self.storm.is_none() {
            self.storm = Some(StormEngine::new("consciousness:auto"));
            return "storm:init".into();
        }
        let engine = match self.storm.as_mut() {
            Some(e) => e,
            None => {
                log::error!("MODULES: engine not init");
                return "engine:unavailable".into();
            }
        };

        if !engine.config.enabled {
            return "storm:skip".into();
        }

        if engine.critique.is_none() && engine.report.is_some() {
            let critique = crate::core::nt_core_experience::storm_engine::StormCritique {
                overall_score: 7.0,
                strengths: vec!["Multi-perspective coverage".to_string()],
                weaknesses: vec!["Pending full grounding integration".to_string()],
                biases: Vec::new(),
                oversimplifications: Vec::new(),
                missing_angles: Vec::new(),
                improvements: vec!["Integrate with KnowledgeBase evidence".to_string()],
                final_version: String::new(),
            };
            engine.critique = Some(critique);
            engine.phase = StormPhase::SelfCritique;
            engine.cycle += 1;
            "storm:critique_initialized".into()
        } else {
            engine.advance_phase();
            engine.cycle += 1;
            format!("storm:advance->{}", engine.phase.label())
        }
    }

    /// STORM status tick — returns current pipeline state.
    pub fn handle_storm_status_tick(&mut self) -> String {
        if self.storm.is_none() {
            self.storm = Some(StormEngine::new("consciousness:auto"));
            return "storm:init".into();
        }
        match &self.storm {
            Some(engine) => engine.stats(),
            None => "storm:uninitialized".into(),
        }
    }

    /// STORM start — set a new topic and begin research.
    pub fn handle_storm_start_tick(&mut self, topic: &str) -> String {
        self.storm = Some(StormEngine::new(topic));
        if let Some(ref mut engine) = self.storm {
            engine.start_research(topic);
            format!("storm:started_topic=\"{}\"", topic)
        } else {
            "storm:start_failed".into()
        }
    }
}
