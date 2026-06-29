#![allow(unused_imports)]
use super::types::*;
use super::ConsciousnessIntegration;
use crate::core::nt_core_experience::{OutputQualityRecord, Slide, SlideContent, SlideLayout};

// VISION handlers extracted from modules.rs
// 4 handlers

impl ConsciousnessIntegration {
    pub fn handle_html_presentation_tick(&mut self) -> String {
        if self.attractor_state.len() < 16 {
            return "html:no_attractor".to_string();
        }
        let coherence = self.specious_present.average_coherence();
        let arousal = self.neuromodulator.arousal_contribution();

        // ── Active inference modulation ──
        // High EFE → uncertainty → exploratory decode (more items, broader coverage)
        // Low EFE → confidence → exploitative decode (fewer, focused items)
        let efe_mod = if self.last_efe_energy > 0.0 {
            // Normalize EFE to [0.5, 2.0] modulation factor
            // Typical EFE range: 0.0–2.0
            let raw = (self.last_efe_energy * 0.75).clamp(0.5, 2.0);
            raw
        } else {
            1.0
        };
        let original_diversity = self.vsa_decoder.policy.item_diversity;
        self.vsa_decoder.policy.item_diversity = (original_diversity * efe_mod).clamp(2.0, 8.0);

        let decoded = self.vsa_decoder.decode(
            &self.attractor_state,
            "presentation",
            self.cycle,
            coherence,
            arousal,
        );

        // Restore policy (long-term learning preserved)
        self.vsa_decoder.policy.item_diversity = original_diversity;

        // Build slides from decoded cognitive sections
        let mut slides = Vec::new();
        slides.push(Slide::new(
            0,
            SlideLayout::Cover,
            &decoded.title,
            SlideContent::Text(String::new()),
        ));
        for section in &decoded.sections {
            slides.push(Slide::new(
                slides.len(),
                SlideLayout::Bullets,
                &section.label,
                SlideContent::BulletList(section.items.clone()),
            ));
        }
        slides.push(Slide::new(
            slides.len(),
            SlideLayout::Thanks,
            "Thank You",
            SlideContent::Text(String::new()),
        ));

        self.html_presentation.slides = slides;
        self.html_presentation.title = decoded.title.clone();

        // Track quality feedback
        self.vsa_decoder.record_quality(
            OutputQualityRecord {
                cycle: self.cycle,
                format: "presentation".to_string(),
                quality_score: decoded.quality_score,
                section_count: decoded.sections.len(),
            },
            Some(&self.attractor_state),
        );

        format!(
            "html:generated_{}_slides_q={:.2}",
            decoded.sections.len(),
            decoded.quality_score
        )
    }

    pub fn handle_motion_synthesizer_tick(&mut self) -> String {
        if self.cycle % 30 != 0 {
            return "motion:skipped".to_string();
        }
        if self.attractor_state.len() < 16 {
            return "motion:no_attractor".to_string();
        }
        let sig = self.visual_signature();
        let preset = if sig.geometry.complexity > 0.7 {
            "orbit"
        } else if sig.palette.warmth > 0.6 {
            "warmth"
        } else if sig.overall_satisfaction > 0.5 {
            "pulse"
        } else {
            "bounce"
        };
        let json = self.generate_lottie(preset);
        let len = json.len();

        self.response_buffer
            .push_back(format!("[lottie:{}:{}b]", preset, len));

        format!("motion:generated_{}_{}b", preset, len)
    }

    pub fn handle_visual_planner_tick(&mut self) -> String {
        let result = self.visual_planner.tick(None);
        log::debug!("MODULES: {}", result);
        result
    }

    // ── Research writer ──

    pub fn handle_transcript_analysis_tick(&mut self) -> String {
        if self.cycle % 50 != 0 {
            return "transcript:skip".to_string();
        }
        let history: Vec<(String, Vec<u8>, f64)> = self
            .thought_history
            .iter()
            .rev()
            .take(20)
            .map(|(t, v, ts)| (t.clone(), v.clone(), *ts))
            .collect();
        if history.len() < 3 {
            return "transcript:too_few".to_string();
        }
        let threshold = 0.78;
        let mut clusters: Vec<Vec<usize>> = Vec::new();
        let mut assigned = vec![false; history.len()];
        for i in 0..history.len() {
            if assigned[i] {
                continue;
            }
            let mut cluster = vec![i];
            assigned[i] = true;
            for j in (i + 1)..history.len() {
                if assigned[j] {
                    continue;
                }
                let sim = crate::core::nt_core_hcube::QuantizedVSA::similarity(
                    &history[i].1,
                    &history[j].1,
                );
                if sim >= threshold {
                    cluster.push(j);
                    assigned[j] = true;
                }
            }
            if cluster.len() >= 2 {
                clusters.push(cluster);
            }
        }
        let n_patterns = clusters.len();
        let top_desc = if n_patterns > 0 {
            clusters.sort_by(|a, b| b.len().cmp(&a.len()));
            let top_text = &history[clusters[0][0]].0;
            if top_text.len() > 40 {
                format!("{}...", &top_text[..37])
            } else {
                top_text.clone()
            }
        } else {
            "none".to_string()
        };
        if n_patterns > 0 {
            let feed_patterns: Vec<(Vec<u8>, f64)> = clusters
                .iter()
                .map(|c| {
                    let rep_vsa = history[c[0]].1.clone();
                    let effectiveness = c.len() as f64 / history.len() as f64;
                    (rep_vsa, effectiveness)
                })
                .collect();
            self.dream_consolidator.feed("transcript", &feed_patterns);
        }
        format!("transcript:patterns={}_top={}", n_patterns, top_desc)
    }

    // ── Skill trend exposure (P2.5: every 150 cycles) ──

    // ── Vision Integration Tick ──

    pub fn handle_vision_integrate_tick(&mut self) -> String {
        // Lazily initialize the vision pipeline on first tick
        if self.vision.is_none() {
            self.init_image_pipeline();
        }
        match &self.vision {
            Some(pipeline) => {
                if pipeline.is_available() {
                    log::info!("vision_integrate_tick: pipeline available");
                    "vision_integrate:ok".to_string()
                } else {
                    log::warn!("vision_integrate_tick: pipeline not available");
                    "vision_integrate:unavailable".to_string()
                }
            }
            None => "vision_integrate:unwired".to_string(),
        }
    }
}
