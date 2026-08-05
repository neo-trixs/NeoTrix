use std::sync::{Arc, Mutex};

use super::nt_evidence_types::{
    era_center, haversine_km, BayesianLink, CalibrationResult, ConfidenceTier, ConflictResolution,
    ContradictionCategory, EvidenceCluster, EvidenceContradiction, EvidenceRecord, EvidenceStats,
    EvidenceTableSnapshot,
};
use super::nt_evidence_hypothesis::HypothesisNetwork;
use crate::neotrix::l3_memory_impl::nt_memory_kb::{KnowledgeBase, KnowledgeEdge, KnowledgeNode, NodeType, RelationType};
use crate::core::nt_core_consciousness_tree::EvidenceChain;

const EWHR_DOMAIN: &str = "nt_memory_historian";

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

pub struct EvidenceStore {
    kb: Arc<Mutex<KnowledgeBase>>,
    pub hypothesis_network: Option<Arc<Mutex<HypothesisNetwork>>>,
}

impl EvidenceStore {
    pub fn new(kb: KnowledgeBase) -> Self {
        Self { kb: Arc::new(Mutex::new(kb)), hypothesis_network: None }
    }

    pub fn try_open_default() -> Option<Self> {
        KnowledgeBase::open(None).ok().map(|kb| Self { kb: Arc::new(Mutex::new(kb)), hypothesis_network: None })
    }

    pub fn with_hypothesis_network(mut self, net: Arc<Mutex<HypothesisNetwork>>) -> Self {
        self.hypothesis_network = Some(net);
        self
    }

    pub fn store_evidence(&self, record: &EvidenceRecord) -> Result<(), String> {
        let json = serde_json::to_string(record).map_err(|e| format!("serialize: {}", e))?;
        let ts = now_ts();
        let node = KnowledgeNode {
            id: format!("ewhr-{}", record.id),
            node_type: NodeType::Concept,
            title: record.name.clone(),
            summary: Some(record.description.chars().take(200).collect()),
            content: Some(json),
            url: None,
            domain: Some(EWHR_DOMAIN.into()),
            language: "en".into(),
            confidence: record.effective_confidence(),
            importance: record.effective_confidence(),
            created_at: ts,
            updated_at: ts,
            access_count: 0,
            metadata: None,
            temporal: None,
            supersedes: None,
            source_episode: None,
        };
        let kb = self.kb.lock().map_err(|e| format!("lock: {}", e))?;
        kb.insert_node(&node).map_err(|e| format!("insert: {}", e))?;
        Ok(())
    }

    /// 证据链统一入弧 — 将 ConsciousnessTree 的 EvidenceChain (WARC/sha256/run_id
    /// 溯源三件套) 桥接为 historian 考古证据记录,落入同一 KB 命名空间,实现
    /// 证据三套合一 (archaeology record / meta-cognition chain / KB node 同源)。
    /// 来源: MetaClaw 溯源模式 + Claude-OSINT 可复现性要求 (P1-1 接线)。
    pub fn store_chain_evidence(&self, chain: &EvidenceChain) -> Result<String, String> {
        let ts = chain.timestamp.max(1).to_string();
        let id = chain
            .sha256
            .clone()
            .unwrap_or_else(|| format!("chain-{}", chain.timestamp));
        let description = format!(
            "warc={:?} run_id={:?} tools={:?}",
            chain.warc_path, chain.run_id, chain.tool_versions
        );
        let record = EvidenceRecord {
            id: id.clone(),
            name: format!("chain-{}", id.chars().take(12).collect::<String>()),
            latitude: 0.0,
            longitude: 0.0,
            era: "evidence-chain".into(),
            category: "chain".into(),
            description,
            dating_methods: chain.tool_versions.clone(),
            context_clarity: 1.0,
            publication_level: 0.9,
            independent_replications: 1,
            provenance_gap: 0.0,
            anachronism_index: 0.0,
            motivation_score: 0.0,
            verification_gap: 0.0,
            references: chain.warc_path.clone().unwrap_or_default(),
            connections: vec![],
            created_at: ts.clone(),
            updated_at: ts,
        };
        self.store_evidence(&record)?;
        Ok(id)
    }

    pub fn get_evidence(&self, id: &str) -> Result<Option<EvidenceRecord>, String> {
        let kb = self.kb.lock().map_err(|e| format!("lock: {}", e))?;
        let node_id = format!("ewhr-{}", id);
        match kb.get_node(&node_id).map_err(|e| format!("get: {}", e))? {
            Some(n) => {
                let content = n.content.ok_or("no content".to_string())?;
                let rec: EvidenceRecord = serde_json::from_str(&content)
                    .map_err(|e| format!("deserialize: {}", e))?;
                Ok(Some(rec))
            }
            None => Ok(None),
        }
    }

    pub fn list_evidence(&self) -> Result<Vec<EvidenceRecord>, String> {
        let kb = self.kb.lock().map_err(|e| format!("lock: {}", e))?;
        let all = kb.search_by_type(&NodeType::Concept, 1000)
            .map_err(|e| format!("search_by_type: {}", e))?;
        let ev: Vec<EvidenceRecord> = all
            .into_iter()
            .filter(|n| n.domain.as_deref() == Some(EWHR_DOMAIN))
            .filter_map(|n| {
                let content = n.content.as_deref()?;
                serde_json::from_str::<EvidenceRecord>(content).ok()
            })
            .collect();
        Ok(ev)
    }

    pub fn delete_evidence(&self, id: &str) -> Result<(), String> {
        let kb = self.kb.lock().map_err(|e| format!("lock: {}", e))?;
        kb.delete_node(&format!("ewhr-{}", id)).map_err(|e| format!("delete: {}", e))?;
        Ok(())
    }

    pub fn stats(&self) -> Result<EvidenceStats, String> {
        let records = self.list_evidence()?;
        let total = records.len();
        let mut t1 = 0; let mut t2 = 0; let mut t3 = 0; let mut t4 = 0; let mut t5 = 0;
        for r in &records {
            match r.tier() {
                ConfidenceTier::T1 => t1 += 1, ConfidenceTier::T2 => t2 += 1,
                ConfidenceTier::T3 => t3 += 1, ConfidenceTier::T4 => t4 += 1,
                ConfidenceTier::T5 => t5 += 1,
            }
        }
        let links = self.compute_links(&records).len();
        let clusters = self.compute_clusters(&records).len();
        Ok(EvidenceStats { total, t1_count: t1, t2_count: t2, t3_count: t3, t4_count: t4, t5_count: t5, links, clusters })
    }

    pub fn compute_links(&self, records: &[EvidenceRecord]) -> Vec<BayesianLink> {
        let mut links = Vec::new();
        for i in 0..records.len() {
            for j in (i + 1)..records.len() {
                let a = &records[i]; let b = &records[j];
                let dist = haversine_km(a.latitude, a.longitude, b.latitude, b.longitude);
                let spatial = (-(dist * dist) / (2.0 * 3000.0 * 3000.0)).exp();
                if spatial < 0.05 { continue; }
                let ac = era_center(&a.era);
                let bc = era_center(&b.era);
                let temp = if ac > 0.0 && bc > 0.0 { 1.0 - (ac - bc).abs() / 100_000.0 } else { 0.1 };
                let cat = if a.category == b.category { 2.0 } else { 1.0 };
                let shared = a.dating_methods.iter().filter(|m| b.dating_methods.contains(m)).count();
                let prob = (0.08 * spatial * temp.max(0.0) * cat * (1.0 + shared as f64 * 0.15)
                    * (0.5 + (a.effective_confidence() + b.effective_confidence()) / 2.0)).min(0.99);
                if prob > 0.12 {
                    links.push(BayesianLink {
                        from: a.id.clone(), to: b.id.clone(), probability: prob,
                        distance_km: dist, temporal_overlap: temp,
                        shared_dating_methods: shared, same_category: a.category == b.category,
                    });
                }
            }
        }
        links.sort_by(|a, b| b.probability.partial_cmp(&a.probability).unwrap_or(std::cmp::Ordering::Equal));
        links
    }

    pub fn compute_clusters(&self, records: &[EvidenceRecord]) -> Vec<EvidenceCluster> {
        let links = self.compute_links(records);
        let strong: Vec<&BayesianLink> = links.iter().filter(|l| l.probability > 0.2).collect();
        let mut adj: std::collections::HashMap<&str, Vec<&str>> = std::collections::HashMap::new();
        for r in records { adj.entry(&r.id).or_default(); }
        for l in &strong {
            adj.entry(&l.from).or_default().push(&l.to);
            adj.entry(&l.to).or_default().push(&l.from);
        }
        let mut visited = std::collections::HashSet::new();
        let mut clusters = Vec::new();
        for r in records {
            if visited.contains(r.id.as_str()) { continue; }
            let mut comp = Vec::new();
            let mut queue = vec![r.id.as_str()];
            visited.insert(r.id.as_str());
            while let Some(cur) = queue.pop() {
                comp.push(cur);
                if let Some(neighbors) = adj.get(cur) {
                    for n in neighbors { if visited.insert(n) { queue.push(n); } }
                }
            }
            if comp.len() >= 3 {
                let mut confs = Vec::new();
                let mut topics_set = std::collections::HashSet::new();
                let mut internal = 0;
                for m_id in &comp {
                    if let Some(rec) = records.iter().find(|r| r.id == *m_id) {
                        confs.push(rec.effective_confidence());
                        topics_set.insert(rec.category.clone());
                    }
                }
                for l in &strong {
                    if comp.contains(&l.from.as_str()) && comp.contains(&l.to.as_str()) { internal += 1; }
                }
                let avg = if confs.is_empty() { 0.0 } else { confs.iter().sum::<f64>() / confs.len() as f64 };
                let members: Vec<String> = comp.into_iter().map(|s| s.to_string()).collect();
                let mc = members.len();
                let topics: Vec<String> = topics_set.into_iter().collect();
                clusters.push(EvidenceCluster {
                    id: format!("C{}", clusters.len()), members, member_count: mc,
                    avg_confidence: avg, internal_links: internal, topics,
                });
            }
        }
        clusters
    }

    /// Persist computed BayesianLinks as KnowledgeEdge entries in the KB.
    /// Supports edges for probability > 0.5, Related edges for > 0.12.
    /// Idempotent: overwrites existing edges with same source/target/type.
    pub fn persist_links_to_kb(&self) -> Result<usize, String> {
        let records = self.list_evidence()?;
        let links = self.compute_links(&records);
        let mut written = 0;
        let ts = now_ts();
        for link in &links {
            let from_id = format!("ewhr-{}", link.from);
            let to_id = format!("ewhr-{}", link.to);
            let rel = if link.probability > 0.5 {
                RelationType::Supports
            } else {
                RelationType::Related
            };
            let edge = KnowledgeEdge {
                id: format!("ewhr-link-{}-{}", link.from, link.to),
                source_id: from_id,
                target_id: to_id,
                relation_type: rel,
                weight: link.probability,
                description: Some(format!(
                    "EWHR Bayesian link: dist={:.0}km temp={:.3} shared_dating={} same_cat={}",
                    link.distance_km, link.temporal_overlap, link.shared_dating_methods, link.same_category
                )),
                created_at: ts,
                metadata: Some(serde_json::json!({
                    "domain": EWHR_DOMAIN,
                    "distance_km": link.distance_km,
                    "temporal_overlap": link.temporal_overlap,
                    "shared_dating_methods": link.shared_dating_methods,
                    "same_category": link.same_category,
                    "from_evidence_id": link.from,
                    "to_evidence_id": link.to,
                })),
            };
            let kb = self.kb.lock().map_err(|e| format!("lock: {}", e))?;
            let _ = kb.delete_edge(&edge.id);
            kb.insert_edge(&edge).map_err(|e| format!("insert edge: {}", e))?;
            written += 1;
        }
        Ok(written)
    }

    /// #4 — Serialize the full evidence table to a checkpoint stored in KB.
    pub fn serialize_evidence_table(&self) -> Result<EvidenceTableSnapshot, String> {
        let records = self.list_evidence()?;
        let links = self.compute_links(&records);
        let clusters = self.compute_clusters(&records);
        Ok(EvidenceTableSnapshot {
            version: 2,
            timestamp: now_ts(),
            records,
            links,
            clusters,
        })
    }

    /// #4 — Write a checkpoint snapshot as a KB node for persistence.
    pub fn checkpoint_evidence_table(&self) -> Result<(), String> {
        let snapshot = self.serialize_evidence_table()?;
        let json = serde_json::to_string(&snapshot).map_err(|e| format!("serialize snapshot: {}", e))?;
        let ts = now_ts();
        let node = KnowledgeNode {
            id: "ewhr-checkpoint-latest".into(),
            node_type: NodeType::Concept,
            title: "EWHR Evidence Table Checkpoint".into(),
            summary: Some(format!("evidence_count={} links={} clusters={}", snapshot.records.len(), snapshot.links.len(), snapshot.clusters.len())),
            content: Some(json),
            url: None,
            domain: Some(EWHR_DOMAIN.into()),
            language: "en".into(),
            confidence: 1.0,
            importance: 0.5,
            created_at: ts,
            updated_at: ts,
            access_count: 0,
            metadata: None,
            temporal: None,
            supersedes: None,
            source_episode: None,
        };
        let kb = self.kb.lock().map_err(|e| format!("lock: {}", e))?;
        kb.insert_node(&node).map_err(|e| format!("insert checkpoint: {}", e))?;
        Ok(())
    }

    /// #4 — Restore evidence table from the latest checkpoint in KB.
    pub fn restore_from_checkpoint(&self) -> Result<Option<EvidenceTableSnapshot>, String> {
        let kb = self.kb.lock().map_err(|e| format!("lock: {}", e))?;
        match kb.get_node("ewhr-checkpoint-latest").map_err(|e| format!("get checkpoint: {}", e))? {
            Some(n) => {
                let content = n.content.ok_or("no checkpoint content".to_string())?;
                let snapshot: EvidenceTableSnapshot = serde_json::from_str(&content)
                    .map_err(|e| format!("deserialize checkpoint: {}", e))?;
                Ok(Some(snapshot))
            }
            None => Ok(None),
        }
    }

    /// #5 — Cross-examine all evidence pairs and detect contradictions.
    pub fn cross_examine(&self) -> Result<Vec<EvidenceContradiction>, String> {
        let records = self.list_evidence()?;
        let mut contradictions = Vec::new();
        for i in 0..records.len() {
            for j in (i + 1)..records.len() {
                let a = &records[i];
                let b = &records[j];
                let dist = haversine_km(a.latitude, a.longitude, b.latitude, b.longitude);
                let ac = era_center(&a.era);
                let bc = era_center(&b.era);
                // Spatial contradiction: same era but very far apart when category matches
                if a.category == b.category && ac > 0.0 && bc > 0.0 && (ac - bc).abs() < 1000.0 && dist > 5000.0 {
                    contradictions.push(EvidenceContradiction {
                        evidence_a_id: a.id.clone(),
                        evidence_b_id: b.id.clone(),
                        category: ContradictionCategory::Spatial,
                        severity: (dist / 10000.0).min(1.0),
                        description: format!(
                            "Same category '{}' and era ({}, {}) but {} km apart",
                            a.category, a.era, b.era, dist as u64
                        ),
                        resolution: None,
                    });
                }
                // Temporal contradiction: same location but very different eras
                if dist < 10.0 && ac > 0.0 && bc > 0.0 && (ac - bc).abs() > 10000.0 {
                    contradictions.push(EvidenceContradiction {
                        evidence_a_id: a.id.clone(),
                        evidence_b_id: b.id.clone(),
                        category: ContradictionCategory::Temporal,
                        severity: ((ac - bc).abs() / 100000.0).min(1.0),
                        description: format!(
                            "Same location but era difference of {:.0} years",
                            (ac - bc).abs()
                        ),
                        resolution: None,
                    });
                }
            }
        }
        // Sort by severity descending
        contradictions.sort_by(|a, b| b.severity.partial_cmp(&a.severity).unwrap_or(std::cmp::Ordering::Equal));

        // Auto-propose hypotheses for high-severity contradictions
        if let Some(ref net_lock) = self.hypothesis_network {
            if let Ok(mut net) = net_lock.lock() {
                for c in &contradictions {
                    if c.severity > 0.5 {
                        let hyp_id = format!("contra-{}-{}",
                            &c.evidence_a_id.chars().take(8).collect::<String>(),
                            &c.evidence_b_id.chars().take(8).collect::<String>());
                        if !net.hypotheses.iter().any(|h| h.id == hyp_id) {
                            net.propose_hypothesis(
                                &hyp_id,
                                &format!("Contradiction: {:?} evidence conflict", c.category),
                                &c.description,
                                0.3,
                            );
                        }
                    }
                }
            }
        }

        Ok(contradictions)
    }

    /// #5 — Resolve a specific contradiction by deciding which evidence to favor.
    pub fn resolve_conflict(&self, contradiction: &EvidenceContradiction) -> Result<ConflictResolution, String> {
        let records = self.list_evidence()?;
        let a = records.iter().find(|r| r.id == contradiction.evidence_a_id)
            .ok_or_else(|| format!("evidence {} not found", contradiction.evidence_a_id))?;
        let b = records.iter().find(|r| r.id == contradiction.evidence_b_id)
            .ok_or_else(|| format!("evidence {} not found", contradiction.evidence_b_id))?;
        // Favor the one with higher effective confidence, more dating methods, more replications
        let a_score = a.effective_confidence() * (1.0 + a.dating_methods.len() as f64 * 0.1)
            * (1.0 + a.independent_replications as f64 * 0.05);
        let b_score = b.effective_confidence() * (1.0 + b.dating_methods.len() as f64 * 0.1)
            * (1.0 + b.independent_replications as f64 * 0.05);
        if a_score >= b_score {
            Ok(ConflictResolution {
                favored_id: a.id.clone(),
                reason: format!(
                    "Higher composite score ({:.3} vs {:.3}): confidence={:.2} methods={} replications={}",
                    a_score, b_score, a.effective_confidence(), a.dating_methods.len(), a.independent_replications
                ),
                new_score: a_score.min(1.0),
            })
        } else {
            Ok(ConflictResolution {
                favored_id: b.id.clone(),
                reason: format!(
                    "Higher composite score ({:.3} vs {:.3}): confidence={:.2} methods={} replications={}",
                    b_score, a_score, b.effective_confidence(), b.dating_methods.len(), b.independent_replications
                ),
                new_score: b_score.min(1.0),
            })
        }
    }

    /// #8 — Redistribute confidence: apply temporal decay based on staleness,
    /// then renormalize the distribution across all evidence.
    pub fn redistribute_decay(&self, half_life_days: f64) -> Result<EvidenceStats, String> {
        let records = self.list_evidence()?;
        let now = now_ts();
        let half_life_secs = half_life_days * 86400.0;
        for rec in &records {
            let created = rec.created_at.parse::<i64>().unwrap_or(now);
            let age_secs = (now - created).max(0) as f64;
            let decay_factor = (-age_secs / half_life_secs).exp();
            let original = rec.effective_confidence();
            // Apply decay, but floor at 0.05 to avoid losing evidence entirely
            let new_conf = (original * decay_factor).max(0.05);
            if (new_conf - original).abs() > 0.01 {
                let mut updated_rec = rec.clone();
                updated_rec.provenance_gap = (rec.provenance_gap + (1.0 - decay_factor) * 0.1).min(1.0);
                updated_rec.updated_at = now.to_string();
                let json = serde_json::to_string(&updated_rec).map_err(|e| format!("serialize: {}", e))?;
                let ts = now;
                let node = KnowledgeNode {
                    id: format!("ewhr-{}", updated_rec.id),
                    node_type: NodeType::Concept,
                    title: updated_rec.name.clone(),
                    summary: Some(updated_rec.description.chars().take(200).collect()),
                    content: Some(json),
                    url: None,
                    domain: Some(EWHR_DOMAIN.into()),
                    language: "en".into(),
                    confidence: new_conf,
                    importance: new_conf,
                    created_at: ts,
                    updated_at: ts,
                    access_count: 0,
                    metadata: None,
                    temporal: None,
                    supersedes: None,
                    source_episode: None,
                };
                let kb = self.kb.lock().map_err(|e| format!("lock: {}", e))?;
                kb.insert_node(&node).map_err(|e| format!("insert decayed: {}", e))?;
            }
        }
        let stats = self.stats()?;
        Ok(stats)
    }

    pub fn calibrate(&self) -> Result<CalibrationResult, String> {
        let records = self.list_evidence()?;
        let links = self.compute_links(&records);
        let clusters = self.compute_clusters(&records);
        let mut tier_changes = Vec::new();
        let ts = now_ts();
        for record in &records {
            let old_tier = record.tier();
            let rec_links: Vec<&BayesianLink> = links.iter().filter(|l| l.from == record.id || l.to == record.id).collect();
            let calibrated_score = if rec_links.is_empty() {
                record.effective_confidence()
            } else {
                let mut num = 0.0; let mut den = 0.0;
                for l in &rec_links {
                    let nid = if l.from == record.id { &l.to } else { &l.from };
                    if let Some(neighbor) = records.iter().find(|r| &r.id == nid) {
                        let w = l.probability * if l.probability > 0.3 { 1.5 } else { 1.0 };
                        num += w * neighbor.effective_confidence();
                        den += w;
                    }
                }
                let navg = if den > 0.0 { num / den } else { 0.5 };
                let pull = 0.15_f64.min(rec_links.len() as f64 * 0.02);
                let adj = record.effective_confidence() + pull * (navg - record.effective_confidence());
                let boost = 0.08_f64.min(rec_links.len() as f64 * 0.01);
                (adj + boost).max(0.01).min(1.0)
            };
            let new_tier = ConfidenceTier::from_score(calibrated_score, record.forgery_risk().total());
            if old_tier != new_tier {
                tier_changes.push(format!("{}: {:?}→{:?}", record.name, old_tier, new_tier));
            }
            // Persist updated confidence back to KB
            let mut updated_rec = record.clone();
            updated_rec.provenance_gap = (record.provenance_gap + (1.0 - calibrated_score) * 0.05).min(1.0);
            updated_rec.updated_at = ts.to_string();
            let json = serde_json::to_string(&updated_rec).map_err(|e| format!("serialize: {}", e))?;
            let node = KnowledgeNode {
                id: format!("ewhr-{}", updated_rec.id),
                node_type: NodeType::Concept,
                title: updated_rec.name.clone(),
                summary: Some(updated_rec.description.chars().take(200).collect()),
                content: Some(json),
                url: None,
                domain: Some(EWHR_DOMAIN.into()),
                language: "en".into(),
                confidence: calibrated_score,
                importance: calibrated_score,
                created_at: ts,
                updated_at: ts,
                access_count: 0,
                metadata: None,
                temporal: None,
                supersedes: None,
                source_episode: None,
            };
            let kb = self.kb.lock().map_err(|e| format!("lock: {}", e))?;
            kb.insert_node(&node).map_err(|e| format!("insert calibrated: {}", e))?;
        }
        Ok(CalibrationResult {
            evidence_count: records.len(), links_found: links.len(),
            clusters_found: clusters.len(), tier_changes,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }
}

#[cfg(test)]
mod tests {
    fn new_store() -> super::EvidenceStore {
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_evstore_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase::open(Some(tmp))
            .expect("open kb");
        super::EvidenceStore::new(kb)
    }

    #[test]
    fn test_placeholder() {
        assert!(true);
    }

    #[test]
    fn test_store_chain_evidence_roundtrip() {
        // P1-1: EvidenceChain (WARC/sha256/run_id) 桥接为考古证据记录并落库
        use crate::core::nt_core_consciousness_tree::EvidenceChain;
        let store = new_store();
        let chain = EvidenceChain {
            warc_path: Some("/tmp/chain.warc.gz".into()),
            sha256: Some("deadbeef".into()),
            run_id: Some("run-42".into()),
            timestamp: 1700000000,
            tool_versions: vec!["crawl-0.1".into(), "parse-0.2".into()],
        };
        let id = store.store_chain_evidence(&chain).expect("store chain");
        assert_eq!(id, "deadbeef");

        let rec = store.get_evidence(&id).expect("get").expect("record");
        assert_eq!(rec.category, "chain");
        assert!(rec.description.contains("run-42"), "run_id 应入 description");
        assert_eq!(rec.dating_methods.len(), 2, "tool_versions 桥接为 dating_methods");
    }
}
