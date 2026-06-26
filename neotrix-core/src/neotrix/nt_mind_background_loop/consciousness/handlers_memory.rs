#![allow(unused_imports)]
use super::types::*;
use super::ConsciousnessIntegration;
use crate::core::nt_core_util;
use crate::neotrix::nt_world_infer::MemoryPalace;

// MEMORY handlers extracted from modules_core.rs
// 10 handlers

impl ConsciousnessIntegration {
    // ── Memory Palace (spatial-temporal memory navigation) ──

    pub fn handle_memory_palace_tick(&mut self) -> String {
        if self.cycle == 0 {
            if let Some(path) = self.memory_palace_path() {
                if path.exists() {
                    match MemoryPalace::load_from_json(&path) {
                        Ok(palace) => {
                            self.memory_palace = palace;
                            log::info!("MODULES: memory_palace loaded from disk");
                        }
                        Err(e) => log::error!("MODULES: memory_palace load error: {}", e),
                    }
                }
            }
        }
        let result = self.memory_palace.tick();
        let diag = self.memory_palace.diagnostic();
        if self.cycle > 0 && self.cycle % 100 == 0 {
            if let Some(path) = self.memory_palace_path() {
                if let Err(e) = self.memory_palace.save_to_json(&path) {
                    log::error!("MODULES: memory_palace save error: {}", e);
                }
            }
        }
        log::debug!("MODULES: memory_palace_tick result={} {}", result, diag);
        format!("palace:{}|{}", result, diag)
    }

    pub fn handle_memory_palace_store(
        &mut self,
        content: String,
        vsa_hash: Vec<u8>,
        source_layer: &str,
    ) -> String {
        self.memory_palace.enter(content, vsa_hash, source_layer);
        format!("palace_store:ok|total={}", self.memory_palace.item_count())
    }

    pub fn handle_memory_palace_map(&self) -> String {
        self.memory_palace.palace_map()
    }

    fn memory_palace_path(&self) -> Option<std::path::PathBuf> {
        let base = std::env::var("NEOTRIX_SOUL_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| nt_core_util::home_dir().join(".neotrix"));
        std::fs::create_dir_all(&base).ok()?;
        Some(base.join("memory_palace.json"))
    }

    // ── Memory lattice (multi-layer memory consolidation) ──

    pub fn handle_memory_lattice_tick(&mut self) -> String {
        let result = self.memory_lattice.tick();
        let diag = self.memory_lattice.diagnostic();
        log::debug!("MODULES: memory_lattice_tick result={} {}", result, diag);
        format!("memory:{}|{}", result, diag)
    }

    pub fn handle_memory_store(&mut self, content: Vec<u8>, vsa_hash: Vec<u8>) -> String {
        let text = String::from_utf8_lossy(&content).to_string();
        let layer = crate::core::nt_core_consciousness::LatticeLayer::Episodic;
        self.memory_lattice.store(text, vsa_hash, layer);
        let total = self.memory_lattice.episodic.len()
            + self.memory_lattice.facts.len()
            + self.memory_lattice.skills.len()
            + self.memory_lattice.meta_rules.len()
            + self.memory_lattice.identity.len();
        format!("memory_store:stored_{}_total={}", content.len(), total)
    }

    pub fn handle_memory_consolidate(&mut self) -> String {
        let n = self.memory_lattice.consolidate();
        if n > 0 {
            log::debug!("MODULES: memory_consolidate {} entries promoted", n);
        }
        format!("memory_consolidate:{}", n)
    }

    pub fn handle_memory_sync_tick(&mut self) -> String {
        let mut p2l = 0usize;
        let mut l2p = 0usize;

        // Direction 1: Palace → Lattice (top 5 by significance)
        {
            let mut top_palace: Vec<_> = self.memory_palace.entries.iter().collect();
            top_palace.sort_by(|a, b| {
                b.significance
                    .partial_cmp(&a.significance)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let to_sync: Vec<_> = top_palace
                .iter()
                .take(5)
                .map(|e| {
                    (
                        e.content.clone(),
                        e.vsa_hash.clone(),
                        e.source_layer.clone(),
                    )
                })
                .collect();
            for (content, vsa_hash, source_layer) in &to_sync {
                let already = self.memory_lattice.find(content);
                if already.is_empty() {
                    use crate::core::nt_core_consciousness::LatticeLayer;
                    let layer = if source_layer == "episodic" {
                        LatticeLayer::Episodic
                    } else {
                        LatticeLayer::Facts
                    };
                    self.memory_lattice
                        .store(content.clone(), vsa_hash.clone(), layer);
                    p2l += 1;
                }
            }
        }

        // Direction 2: Lattice → Palace
        // Facts (top 3 by confidence)
        {
            let top: Vec<_> = self.memory_lattice.facts.iter().collect();
            let to_sync: Vec<_> = top
                .iter()
                .take(3)
                .map(|e| (e.content.clone(), e.vsa_hash.clone()))
                .collect();
            for (content, vsa_hash) in &to_sync {
                let existing = self.memory_palace.find_by_content(content);
                if existing.is_empty() {
                    if let Some(rid) = self.memory_palace.find_room_for(vsa_hash) {
                        self.memory_palace.walk_to(rid);
                    }
                    self.memory_palace
                        .enter(content.clone(), vsa_hash.clone(), "facts");
                    l2p += 1;
                }
            }
        }
        // Skills (top 1 by confidence)
        {
            let top: Vec<_> = self.memory_lattice.skills.iter().collect();
            if let Some(entry) = top.iter().max_by(|a, b| {
                a.confidence
                    .partial_cmp(&b.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }) {
                let existing = self.memory_palace.find_by_content(&entry.content);
                if existing.is_empty() {
                    if let Some(rid) = self.memory_palace.find_room_for(&entry.vsa_hash) {
                        self.memory_palace.walk_to(rid);
                    }
                    self.memory_palace.enter(
                        entry.content.clone(),
                        entry.vsa_hash.clone(),
                        "skills",
                    );
                    l2p += 1;
                }
            }
        }
        // MetaRules (top 1 by confidence)
        {
            let top: Vec<_> = self.memory_lattice.meta_rules.iter().collect();
            if let Some(entry) = top.iter().max_by(|a, b| {
                a.confidence
                    .partial_cmp(&b.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }) {
                let existing = self.memory_palace.find_by_content(&entry.content);
                if existing.is_empty() {
                    if let Some(rid) = self.memory_palace.find_room_for(&entry.vsa_hash) {
                        self.memory_palace.walk_to(rid);
                    }
                    self.memory_palace.enter(
                        entry.content.clone(),
                        entry.vsa_hash.clone(),
                        "meta_rules",
                    );
                    l2p += 1;
                }
            }
        }

        if p2l > 0 || l2p > 0 {
            log::debug!("MODULES: memory_sync p2l={} l2p={}", p2l, l2p);
        }
        format!("mem_sync:p2l={}|l2p={}", p2l, l2p)
    }

    // ── Memory reflector (Stanford-style reflection pillar) ──

    pub fn handle_memory_reflector_tick(&mut self) -> String {
        self.memory_reflector.tick();
        if self.cycle % 10 != 0 {
            return format!("reflect:{}", self.memory_reflector.diagnostic());
        }

        // Gather entries from palace and lattice for reflection
        let palace_entries: Vec<_> = self.memory_palace.entries.iter().collect();
        let lattice_layers = vec![
            self.memory_lattice.facts.iter().collect::<Vec<_>>(),
            self.memory_lattice.skills.iter().collect::<Vec<_>>(),
            self.memory_lattice.meta_rules.iter().collect::<Vec<_>>(),
        ];

        let palace_slice: Vec<_> = palace_entries.iter().map(|e| (*e).clone()).collect();
        let lattice_refs: Vec<Vec<_>> = lattice_layers
            .into_iter()
            .map(|v| v.into_iter().cloned().collect())
            .collect();

        let insights = self.memory_reflector.reflect(&palace_slice, &lattice_refs);

        // Consolidate: promote recurring insights to MetaInsights
        let consolidated = self.memory_reflector.consolidate();
        let all_insights: Vec<
            &crate::core::nt_core_consciousness::memory_reflector::ReflectionInsight,
        > = insights.iter().chain(consolidated.iter()).collect();

        // Store insights back into memory
        let mut stored = 0usize;
        let mut stored_vsas: Vec<Vec<u8>> = Vec::new();
        for insight in all_insights {
            // Confidence gate: skip low-quality noise
            if insight.confidence < 0.5 {
                continue;
            }
            use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            insight.content.hash(&mut hasher);
            let seed = hasher.finish();
            let vsa = QuantizedVSA::seeded_random(seed, 4096);

            // Dedup: skip if very similar to already-stored insight in this batch
            let too_similar = stored_vsas.iter().any(|existing: &Vec<u8>| {
                existing
                    .iter()
                    .zip(vsa.iter())
                    .filter(|(a, b)| a == b)
                    .count()
                    > vsa.len() / 2
            });
            if too_similar {
                continue;
            }
            stored_vsas.push(vsa.clone());

            // Store in both memory systems
            self.memory_lattice.store(
                insight.content.clone(),
                vsa.clone(),
                crate::core::nt_core_consciousness::LatticeLayer::Facts,
            );
            if insight.insight_type
                == crate::core::nt_core_consciousness::memory_reflector::InsightType::MetaInsight
            {
                self.memory_palace.walk_to_by_name("wisdom");
            } else if let Some(rid) = self.memory_palace.find_room_for(&vsa) {
                self.memory_palace.walk_to(rid);
            }
            self.memory_palace
                .enter(insight.content.clone(), vsa, "reflection");
            stored += 1;
        }

        if stored > 0 {
            log::debug!("MODULES: memory_reflector stored {} insights", stored);
        }
        let diag = self.memory_reflector.diagnostic();
        format!("reflect:{}|stored={}", diag, stored)
    }

    // ── Working memory ──

    pub fn handle_working_memory_tick(&mut self) -> String {
        let count = self.working_memory.item_count();
        log::debug!("MODULES: working_memory_tick items={}", count);
        format!("working_memory_tick:{}_items", count)
    }
}
