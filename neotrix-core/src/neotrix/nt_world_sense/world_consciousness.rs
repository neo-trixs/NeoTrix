use crate::core::nt_core_sense::*;
use crate::neotrix::nt_world_sense::nt_world_sense_hub::SensoryIntegrationHub;
use crate::neotrix::nt_world_sense::omniscient_view::OmniscientView;

pub struct WorldConsciousness {
    pub nt_world_sense: SensoryIntegrationHub,
    pub conversation_observer: ConversationObserver,
    pub omniscient_view: OmniscientView,
    pub active: bool,
}

impl WorldConsciousness {
    pub fn new() -> Self {
        Self {
            nt_world_sense: SensoryIntegrationHub::new(),
            conversation_observer: ConversationObserver::new(100),
            omniscient_view: OmniscientView::new(),
            active: false,
        }
    }

    pub fn record_conversation_turn(
        &mut self,
        user_message: &str,
        system_response: &str,
        tools_used: Vec<String>,
        duration_ms: u64,
    ) -> usize {
        let id = self.conversation_observer.turns.len() + 1;
        let turn = ConversationTurn {
            turn_number: id,
            user_message: user_message.to_string(),
            system_response: system_response.to_string(),
            intent_label: None,
            user_satisfaction: None,
            duration_ms,
            tools_used,
        };
        self.conversation_observer.record_turn(turn);
        self.omniscient_view = self.omniscient_view.from_observer(&self.conversation_observer);
        id
    }

    pub fn god_view(&self) -> GodViewReport {
        self.conversation_observer.god_view_report()
    }

    pub fn omniscient_status(&self) -> &OmniscientView {
        &self.omniscient_view
    }

    pub fn refresh_self_awareness(&mut self) {
        self.omniscient_view = self.omniscient_view.from_observer(&self.conversation_observer);
    }

    pub fn consciousness_status(&self) -> String {
        let nt_world_sense_narrative = self.nt_world_sense.current_perception_narrative();
        let turn_count = self.conversation_observer.turns.len();
        let last_turn = self.conversation_observer.turns.last();
        let o = &self.omniscient_view;

        let self_awareness = format!(
            "Self-awareness: {} | scores [{:.2}, {:.2}, {:.2}] | user_model: {}",
            o.self_narrative, o.consciousness_scores[0], o.consciousness_scores[1],
            o.consciousness_scores[2], o.user_model,
        );
        let world_awareness = format!("World-awareness: {}", nt_world_sense_narrative);
        let observer_awareness = match last_turn {
            Some(t) => format!("Observer-awareness: {} turns, last user msg: \"{}\"", turn_count, t.user_message),
            None => format!("Observer-awareness: {} turns, awaiting first interaction", turn_count),
        };

        format!(
            "=== Consciousness Status ===\n\
             1. {}\n\
             2. {}\n\
             3. {}\n\
             === Dialogue Arc ({} phases) ===\n{:?}\n\
             === Sentiment Trend === {:.2} | Repetition: {}\n\
             === God's Eye View ===\n{}\n\
             === Capability Summary ===\n{}\n\
             === Conversation Arc ===\n{}\n\
             === Omniscient View ===\n{}",
            self_awareness, world_awareness, observer_awareness,
            o.dialogue_phases.len(), o.dialogue_phases,
            o.sentiment_trend,
            if o.repetition_detected { format!("yes: {:?}", o.repeated_topics) } else { "none".to_string() },
            o.god_eye_narrative, o.capability_summary,
            o.conversation_arc.join("\n"), o.self_narrative,
        )
    }
}

impl Default for WorldConsciousness {
    fn default() -> Self { Self::new() }
}
