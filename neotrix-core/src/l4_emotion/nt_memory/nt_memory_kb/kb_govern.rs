//! kb_govern — 从 `nt_memory_kb/mod.rs` 拆分 (行为零变更).
//! `impl KnowledgeBase` 按域搬运, 原文逐行, 仅补可见性/导入。


use super::KnowledgeBase;
use super::nt_memory_proficiency;
use super::nt_memory_svaf_gate;
use super::shared_utils;

impl KnowledgeBase {
    pub fn gate_knowledge(
        &self,
        title: &str,
        content: &str,
        source_type: &str,
    ) -> nt_memory_svaf_gate::SvafEvaluation {
        let gate = self.svaf_gate.read().map_err(|e| format!("Lock: {}", e));
        match gate {
            Ok(g) => g.evaluate(self, title, content, source_type),
            Err(_) => nt_memory_svaf_gate::SvafEvaluation {
                decision: nt_memory_svaf_gate::SvafDecision::Accept,
                novelty: 0.5,
                coherence: 0.5,
                relevance: 0.5,
                authority: 0.5,
                reason: "lock error, default accept".into(),
            },
        }
    }

    pub fn gate_content_only(
        &self,
        content: &str,
        source_type: &str,
    ) -> nt_memory_svaf_gate::SvafEvaluation {
        self.svaf_gate
            .read()
            .map(|g| g.evaluate_content_only(content, source_type))
            .unwrap_or(nt_memory_svaf_gate::SvafEvaluation {
                decision: nt_memory_svaf_gate::SvafDecision::Accept,
                novelty: 0.5,
                coherence: 0.5,
                relevance: 0.5,
                authority: 0.5,
                reason: "lock error".into(),
            })
    }

    pub fn save_svaf_gate(&self) -> Result<(), String> {
        let gate = self.svaf_gate.read().map_err(|e| format!("Lock: {}", e))?;
        shared_utils::save_kv_state(self, "svaf_gate", "config", &*gate)
    }

    pub fn load_svaf_gate(&self) -> Result<(), String> {
        let loaded: Option<nt_memory_svaf_gate::SvafGate> =
            shared_utils::load_kv_state(self, "svaf_gate", "config")?;
        if let Some(data) = loaded {
            let mut gate = self.svaf_gate.write().map_err(|e| format!("Lock: {}", e))?;
            *gate = data;
        }
        Ok(())
    }

    // ── Proficiency ──

    pub fn proficiency(&self) -> nt_memory_proficiency::MemoryProficiency {
        self.proficiency
            .read()
            .map(|p| p.clone())
            .unwrap_or_default()
    }

    pub fn record_memory_action(&self, record: nt_memory_proficiency::MemoryActionRecord) {
        if let Ok(mut p) = self.proficiency.write() {
            p.record_action(record);
        }
    }

    pub fn proficiency_report(&self) -> nt_memory_proficiency::MemoryProficiencyReport {
        self.proficiency.read().map(|p| p.report()).unwrap_or(
            nt_memory_proficiency::MemoryProficiencyReport {
                total_actions: 0,
                revision_count: 0,
                action_breakdown: Vec::new(),
                context_preferences: std::collections::HashMap::new(),
                overall_efficiency: 0.0,
            },
        )
    }

    pub fn proficiency_recommend(
        &self,
        context_key: &str,
    ) -> (nt_memory_proficiency::MemoryAction, f64) {
        self.proficiency
            .read()
            .map(|p| p.recommend_action(context_key))
            .unwrap_or((nt_memory_proficiency::MemoryAction::SearchFts, 0.0))
    }

    pub fn outer_loop_revision(&self) -> usize {
        self.proficiency
            .write()
            .map(|mut p| p.outer_loop_revision())
            .unwrap_or(0)
    }

    pub fn save_proficiency(&self) -> Result<(), String> {
        let p = self
            .proficiency
            .read()
            .map_err(|e| format!("Lock: {}", e))?;
        shared_utils::save_kv_state(self, "proficiency", "state", &*p)
    }

    pub fn load_proficiency(&self) -> Result<(), String> {
        let loaded: Option<nt_memory_proficiency::MemoryProficiency> =
            shared_utils::load_kv_state(self, "proficiency", "state")?;
        if let Some(data) = loaded {
            let mut p = self
                .proficiency
                .write()
                .map_err(|e| format!("Lock: {}", e))?;
            *p = data;
        }
        Ok(())
    }

    // ── GraphRAG ──
}
