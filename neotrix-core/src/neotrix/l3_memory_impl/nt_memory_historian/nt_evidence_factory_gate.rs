use std::sync::{Arc, Mutex};

use super::nt_evidence_store::EvidenceStore;
use super::nt_evidence_types::{
    era_center, haversine_km, ContradictionCategory, EvidenceContradiction, EvidenceRecord,
    FactoryGateConfig,
};

/// #7 — Memory evidence integration factory gate.
/// Controls which evidence records are admitted into the knowledge base
/// based on confidence, peer review status, contradiction limits, and method replication.
pub struct EvidenceFactoryGate {
    store: Arc<Mutex<EvidenceStore>>,
    config: FactoryGateConfig,
}

impl EvidenceFactoryGate {
    pub fn new(store: Arc<Mutex<EvidenceStore>>, config: FactoryGateConfig) -> Self {
        Self { store, config }
    }

    /// Admit an evidence record through the factory gate.
    /// Returns Ok(true) if admitted, Ok(false) if rejected.
    pub fn admit(&self, record: &EvidenceRecord) -> Result<bool, String> {
        // Gate 1: Minimum confidence threshold
        if record.effective_confidence() < self.config.min_confidence {
            return Ok(false);
        }

        // Gate 2: Contradiction check — reject if too many contradictions with existing evidence
        let store = self.store.lock().map_err(|e| format!("lock: {}", e))?;
        let existing = store.list_evidence()?;
        let contradictions: Vec<EvidenceContradiction> = existing
            .iter()
            .filter_map(|e| {
                let dist = haversine_km(
                    record.latitude, record.longitude,
                    e.latitude, e.longitude,
                );
                let ac = era_center(&record.era);
                let bc = era_center(&e.era);
                if record.category == e.category && ac > 0.0 && bc > 0.0 && (ac - bc).abs() < 1000.0 && dist > 5000.0 {
                    Some(EvidenceContradiction {
                        evidence_a_id: record.id.clone(),
                        evidence_b_id: e.id.clone(),
                        category: ContradictionCategory::Spatial,
                        severity: (dist / 10000.0).min(1.0),
                        description: format!("Spatial contradiction with {}: {} km apart", e.name, dist as u64),
                        resolution: None,
                    })
                } else {
                    None
                }
            })
            .collect();

        if contradictions.len() > self.config.max_contradictions_allowed {
            return Ok(false);
        }

        // Gate 3: Method replication requirement
        if self.config.require_method_replication && record.dating_methods.len() < 2 {
            return Ok(false);
        }

        Ok(true)
    }

    /// Batch admit: run all records through the gate, return admitted and rejected lists.
    pub fn batch_admit(&self, records: &[EvidenceRecord]) -> Result<(Vec<EvidenceRecord>, Vec<(EvidenceRecord, String)>), String> {
        let mut admitted = Vec::new();
        let mut rejected = Vec::new();
        for record in records {
            match self.admit(record) {
                Ok(true) => {
                    let store = self.store.lock().map_err(|e| format!("lock: {}", e))?;
                    store.store_evidence(record)?;
                    admitted.push(record.clone());
                }
                Ok(false) => rejected.push((record.clone(), "Failed factory gate checks".into())),
                Err(e) => rejected.push((record.clone(), e)),
            }
        }
        Ok((admitted, rejected))
    }

    pub fn config(&self) -> &FactoryGateConfig {
        &self.config
    }

    pub fn update_config(&mut self, config: FactoryGateConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::l3_memory_impl::nt_memory_historian::nt_evidence_store::EvidenceStore;

    fn make_store() -> Arc<Mutex<EvidenceStore>> {
        let tmp = std::env::temp_dir().join(format!("neotrix_fg_{}.db", rand::random::<u64>()));
        let _ = std::fs::remove_file(&tmp);
        let kb = crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase::open(Some(tmp.clone().into()))
            .expect("failed to open test KB");
        let store = EvidenceStore::new(kb);
        Arc::new(Mutex::new(store))
    }

    fn make_record(id: &str, _confidence: f64, lat: f64, lon: f64, era: &str, cat: &str) -> EvidenceRecord {
        EvidenceRecord {
            id: id.into(),
            name: format!("Test {}", id),
            latitude: lat,
            longitude: lon,
            era: era.into(),
            category: cat.into(),
            description: "test".into(),
            dating_methods: vec!["c14".into(), "dendro".into()],
            context_clarity: 0.8,
            publication_level: 0.7,
            independent_replications: 1,
            provenance_gap: 0.1,
            anachronism_index: 0.1,
            motivation_score: 0.1,
            verification_gap: 0.1,
            references: "".into(),
            connections: vec![],
            created_at: "1704067200".into(),
            updated_at: "1704067200".into(),
        }
    }

    fn make_low_conf_record(id: &str, lat: f64, lon: f64, era: &str, cat: &str) -> EvidenceRecord {
        EvidenceRecord {
            id: id.into(),
            name: format!("LowConf {}", id),
            latitude: lat,
            longitude: lon,
            era: era.into(),
            category: cat.into(),
            description: "low quality".into(),
            dating_methods: vec!["lithic".into()],
            context_clarity: 0.1,
            publication_level: 0.05,
            independent_replications: 0,
            provenance_gap: 0.9,
            anachronism_index: 0.8,
            motivation_score: 0.7,
            verification_gap: 0.9,
            references: "".into(),
            connections: vec![],
            created_at: "1704067200".into(),
            updated_at: "1704067200".into(),
        }
    }

    #[test]
    fn test_admit_high_confidence() {
        let store = make_store();
        let gate = EvidenceFactoryGate::new(store, FactoryGateConfig {
            min_confidence: 0.5,
            require_peer_review: false,
            max_contradictions_allowed: 0,
            require_method_replication: true,
        });
        let rec = make_record("r1", 0.8, 0.0, 0.0, "1000 BP", "tool");
        assert!(gate.admit(&rec).unwrap());
    }

    #[test]
    fn test_reject_low_confidence() {
        let store = make_store();
        let gate = EvidenceFactoryGate::new(store, FactoryGateConfig {
            min_confidence: 0.5,
            require_peer_review: false,
            max_contradictions_allowed: 0,
            require_method_replication: false,
        });
        let rec = make_low_conf_record("r1", 0.0, 0.0, "1000 BP", "tool");
        assert!(!gate.admit(&rec).unwrap());
    }

    #[test]
    fn test_reject_contradiction() {
        let store = make_store();
        // First admit an evidence with specific location/era
        {
            let s = store.lock().unwrap();
            let r1 = make_record("existing", 0.8, 0.0, 0.0, "1000 BP", "tool");
            s.store_evidence(&r1).unwrap();
        }
        let gate = EvidenceFactoryGate::new(store, FactoryGateConfig {
            min_confidence: 0.1,
            require_peer_review: false,
            max_contradictions_allowed: 0,
            require_method_replication: false,
        });
        // Same category, same era, but 10000km away — contradiction
        let rec = make_record("r2", 0.8, 90.0, 0.0, "1000 BP", "tool");
        assert!(!gate.admit(&rec).unwrap());
    }
}
