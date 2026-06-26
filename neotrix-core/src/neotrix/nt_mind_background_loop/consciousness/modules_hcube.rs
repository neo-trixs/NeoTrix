use super::ConsciousnessIntegration;

impl ConsciousnessIntegration {
    /// Lazy-init multi-modal aligner, prune weak alignments periodically,
    /// report alignment statistics.
    pub fn handle_multi_modal_tick(&mut self) -> String {
        let engine = self.multi_modal_aligner.get_or_insert_with(|| {
            crate::core::nt_core_hcube::multi_modal_aligner::MultiModalAligner::new()
        });

        let pruned = engine.prune(0.15);
        let stats = engine.stats();

        format!(
            "multi_modal:alignments={}_pruned={}_{}",
            engine.alignments.len(),
            pruned,
            stats,
        )
    }
}
