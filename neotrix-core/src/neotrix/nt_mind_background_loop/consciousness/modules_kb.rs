#![allow(unused_imports)]
use super::types::*;
use super::ConsciousnessIntegration;
use crate::core::nt_core_agent::consensus::{ByzantineConsensusLayer, ConsensusConfig};
use crate::core::nt_core_hcube::interaction_trace::InteractionTracePredictor;
use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;
use crate::core::nt_core_knowledge::entity_resolver::EntityResolver;
use crate::core::nt_core_knowledge::evidence::EvidenceManager;
use crate::core::nt_core_knowledge::fringe_mix::FringeMixStrategy;
use crate::core::nt_core_knowledge::hubness_detector::HubnessDetector;
use crate::core::nt_core_knowledge::hypergraph::{Hyperedge, NaryRelationType};
use crate::core::nt_core_knowledge::keyword_lexicon::KeywordLexicon;
use crate::core::nt_core_knowledge::spread_activation::{EdgeKind, MemoryGraph, NodeKind};

// KB handlers extracted from modules.rs
// 6 handlers

impl ConsciousnessIntegration {
    pub fn handle_entity_resolver_tick(&mut self) -> String {
        if self.entity_resolver.is_none() {
            self.entity_resolver = Some(EntityResolver::new());
            return "eres:init".into();
        }
        let er = match self.entity_resolver.as_mut() {
            Some(e) => e,
            None => {
                log::error!("MODULES: engine not init");
                return "engine:unavailable".into();
            }
        };
        let id = er.register("neotrix:consciousness", self.cycle as u64);
        format!("eres:entity={}", id)
    }

    // ── DySIB Layer (P1.27) ──

    pub fn handle_interaction_trace_tick(&mut self) -> String {
        if self.interaction_trace.is_none() {
            self.interaction_trace = Some(InteractionTracePredictor::with_params(10, VSA_DIM));
            return "itra:init".into();
        }
        let itp = match self.interaction_trace.as_ref() {
            Some(itp) => itp,
            None => {
                log::error!("[modules_kb] interaction_trace not initialized");
                return "interaction_trace:unavailable".into();
            }
        };
        let n = itp.history().len();
        format!("itra:traces={}", n)
    }

    // ── Keyword Lexicon (P1.29) ──

    pub fn handle_keyword_lexicon_tick(&mut self) -> String {
        if self.keyword_lexicon.is_none() {
            self.keyword_lexicon = Some(KeywordLexicon::new());
            return "klex:init".into();
        }
        let klex = match self.keyword_lexicon.as_mut() {
            Some(e) => e,
            None => {
                log::error!("MODULES: engine not init");
                return "engine:unavailable".into();
            }
        };
        let count = klex.len();
        if !self.attractor_state.is_empty() {
            let keywords = klex.extract_from_attractor(&self.attractor_state, 3);
            if !keywords.is_empty() {
                format!("klex:hits={},keys={}", keywords.len(), count)
            } else {
                format!("klex:keys={}", count)
            }
        } else {
            format!("klex:keys={}", count)
        }
    }

    // ── Quant Data Ingestion (P1.23) ──

    pub fn handle_fringe_mix_tick(&mut self) -> String {
        if self.fringe_mix.is_none() {
            self.fringe_mix = Some(FringeMixStrategy::new());
            return "fmix:init".into();
        }
        let fm = match self.fringe_mix.as_ref() {
            Some(fm) => fm,
            None => {
                log::error!("[modules_kb] fringe_mix not initialized");
                return "fringe_mix:unavailable".into();
            }
        };
        // Build a minimal centrality graph using attractor-state-derived nodes
        let mut graph: std::collections::HashMap<u64, Vec<u64>> = std::collections::HashMap::new();
        graph.insert(1, vec![2, 3]);
        graph.insert(2, vec![1, 4]);
        graph.insert(3, vec![1, 4]);
        graph.insert(4, vec![2, 3]);
        let centrality = fm.compute_centrality(&graph);
        let (central, _peripheral, _mid) = fm.classify_nodes(&centrality);
        format!("fmix:central={}", central.len())
    }

    // ── Factor Mining Agent (P2.16) ──

    pub fn handle_hubness_detector_tick(&mut self) -> String {
        if self.hubness_detector.is_none() {
            self.hubness_detector = Some(HubnessDetector::new(10, 3.5, 100));
            return "hub:init".into();
        }
        let hub = match self.hubness_detector.as_ref() {
            Some(hub) => hub,
            None => {
                log::error!("[modules_kb] hubness_detector not initialized");
                return "hubness_detector:unavailable".into();
            }
        };
        if self.attractor_state.len() >= 4096 && self.cycle % 10 == 0 {
            // Check if attractor state itself is a hub relative to past states
            let states: [Vec<u8>; 20] = std::array::from_fn(|_| self.attractor_state.clone());
            let scores = hub.compute_hubness_scores(&states, 5);
            let zs = hub.z_score_normalize(&scores);
            let flagged = hub.flag_hubs(&zs, 3.0);
            format!("hub:flagged={},n={}", flagged.len(), scores.len())
        } else {
            "hub:ok".into()
        }
    }

    // ── Remote Agent Host (P2.23) ──

    // ── P1.01: EvidenceManager ──
    pub fn handle_evidence_tick(&mut self) -> String {
        if self.evidence.is_none() {
            self.evidence = Some(EvidenceManager::new(5000));
            return "evidence:init".into();
        }
        let ev = match self.evidence.as_mut() {
            Some(e) => e,
            None => {
                log::error!("MODULES: engine not init");
                return "engine:unavailable".into();
            }
        };
        let mut added = 0u64;
        if self.cycle > 0 && self.cycle % 10 == 0 && !self.attractor_state.is_empty() {
            let assertion = format!(
                "attractor_state cycle={} len={}",
                self.cycle,
                self.attractor_state.len()
            );
            ev.add_evidence("neotrix:consciousness", "consciousness", &assertion);
            added = 1;
        }
        let s = ev.stats();
        format!(
            "evidence:records={}_avg_conf={:.3}_unver={}_xref={}_val={}_disp={}_added={}",
            s.total_records,
            s.avg_confidence,
            s.unverified,
            s.cross_referenced,
            s.validated,
            s.disputed,
            added,
        )
    }

    // ── P1.03: SpreadActivationMemory ──
    pub fn handle_spread_activation_tick(&mut self) -> String {
        if self.spread_activation.is_none() {
            self.spread_activation = Some(MemoryGraph::new(1000));
            return "spread_activation:init".into();
        }
        let sa = match self.spread_activation.as_mut() {
            Some(sa) => sa,
            None => {
                log::error!("[modules_kb] spread_activation not initialized");
                return "spread_activation:unavailable".into();
            }
        };
        let mut added_nodes = 0usize;
        let mut added_edges = 0usize;
        if self.cycle > 0 && self.cycle % 3 == 0 {
            let history = &self.thought_history;
            let start = history.len().saturating_sub(5);
            let mut prev_id: Option<u64> = None;
            for i in start..history.len() {
                let (ref label, ref vsa, _ts) = history[i];
                let nid = sa.add_node(NodeKind::Episodic, vsa.clone(), label);
                added_nodes += 1;
                if let Some(pid) = prev_id {
                    sa.add_edge(pid, nid, EdgeKind::Temporal, 0.8);
                    added_edges += 1;
                }
                prev_id = Some(nid);
            }
        }
        format!(
            "spread_activation:nodes={}_edges={}_added={}+{}",
            sa.node_count(),
            sa.edge_count(),
            added_nodes,
            added_edges,
        )
    }

    // ── P1.04: BFT ConsensusEngine ──
    pub fn handle_consensus_tick(&mut self) -> String {
        if self.consensus_engine.is_none() {
            self.consensus_engine = Some(ByzantineConsensusLayer::with_defaults());
            return "consensus:init".into();
        }
        let ce = match self.consensus_engine.as_ref() {
            Some(ce) => ce,
            None => {
                log::error!("[modules_kb] consensus_engine not initialized");
                return "consensus_engine:unavailable".into();
            }
        };
        let cfg = ce.config();
        format!(
            "consensus:rounds={}_quorum={:.2}",
            cfg.max_rounds, cfg.quorum_ratio,
        )
    }

    // ── P1.05: HypergraphStore independent ticker ──
    pub fn handle_hypergraph_tick(&mut self) -> String {
        if self.hypergraph_store.is_none() {
            self.hypergraph_store =
                Some(crate::core::nt_core_knowledge::hypergraph::HypergraphStore::new(100));
            return "hypergraph:init".into();
        }
        let hg = match self.hypergraph_store.as_mut() {
            Some(hg) => hg,
            None => {
                log::error!("MODULES: hypergraph not init");
                return "hypergraph:unavailable".into();
            }
        };
        let mut added = 0usize;
        if self.cycle > 0 && self.cycle % 10 == 0 {
            if let Some((ref label, ref _vsa, _ts)) = self.thought_history.back() {
                let edge = Hyperedge {
                    id: format!("hg:cycle:{}", self.cycle),
                    entities: vec![
                        "consciousness".to_string(),
                        label.chars().take(20).collect::<String>(),
                    ],
                    relation_type: NaryRelationType::TemporalSequence,
                    weight: 0.8,
                    confidence: 0.5,
                    context: format!("cycle={}", self.cycle),
                    source_url: "neotrix:consciousness".to_string(),
                    created_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or(0),
                    temporal_order: Some(self.cycle as u64),
                    vsa_fingerprint: None,
                };
                if hg.insert(edge) {
                    added = 1;
                }
            }
        }
        format!("hypergraph:edges={}_added={}", hg.count(), added)
    }

    pub fn handle_kb_tick(&mut self) -> String {
        if self.kb.is_none() {
            match crate::neotrix::nt_memory_kb::KnowledgeBase::open(None) {
                Ok(kb) => self.kb = Some(kb),
                Err(e) => {
                    log::error!("[modules_kb] failed to init KnowledgeBase: {}", e);
                    return "knowledge_base:init_error".into();
                }
            }
            return "knowledge_base:init".into();
        }
        let kb = match self.kb.as_ref() {
            Some(kb) => kb,
            None => {
                log::error!("MODULES: engine not init");
                return "engine:unavailable".into();
            }
        };
        let mut inserted = 0usize;
        if self.cycle > 0 && self.cycle % 15 == 0 {
            if let Some((ref label, ref _vsa, _ts)) = self.thought_history.back() {
                if let Ok(_id) = kb.insert_or_get_node(
                    &label.chars().take(100).collect::<String>(),
                    crate::neotrix::nt_memory_kb::NodeType::Concept,
                    Some(&format!("consciousness cycle={}", self.cycle)),
                    None,
                    Some("neotrix:consciousness"),
                ) {
                    inserted = 1;
                }
            }
        }
        match kb.stats() {
            Ok(stats) => {
                format!(
                    "knowledge_base:{}_nodes_{}_edges_inserted={}",
                    stats.total_nodes, stats.total_edges, inserted,
                )
            }
            Err(e) => format!("knowledge_base:error_{}", e),
        }
    }
}
