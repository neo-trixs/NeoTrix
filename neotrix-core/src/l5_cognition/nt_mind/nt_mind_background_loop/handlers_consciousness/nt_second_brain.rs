use super::super::*;

impl BackgroundLoopHandle {
    pub(crate) async fn handle_second_brain_tick(&mut self) {
        if let Some(ref mut sb) = self.second_brain {
            if let Some(ref mut cr) = self.consciousness_runtime {
                let report = cr.tick_emotion();
                if let Some(ref mut tree) = self.consciousness_tree {
                    tree.apply_emotion_report(report);
                }
                let note = format!(
                    "consciousness_tick iteration={} quality={:.3}",
                    self.brain.try_read().map(|b| b.iteration).unwrap_or(0),
                    cr.last_quality().unwrap_or(0.0),
                );
                sb.tick(Some(cr.emotion_engine()), Some(&note));
                // Persist emotion state to KB every tick (600s)
                let engine = cr.emotion_engine();
                sb.save_emotion_raw(engine);
                // Persist human affective interface (user emotion + relationship) for
                // cross-session continuity — restored by the 5s startup handler.
                sb.save_affective(cr.affective());
            } else {
                sb.tick(None, None);
            }
        }
    }
}
