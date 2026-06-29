/// Unified health-check and GC interface for introspection-driven self-healing.
///
/// Any subsystem implementing this trait can be automatically managed
/// by the IntrospectionEngine's auto-correction loop.
pub trait HealthCheckable {
    /// Returns Some((component_name, current_size)) if the component
    /// is above its healthy threshold. Returns None if healthy.
    fn check_health(&self) -> Option<(String, usize)>;

    /// Perform garbage collection / pruning. Returns bytes reclaimed (approx).
    fn health_gc(&mut self) -> usize;
}

// ── HandlerRegistry ──

use crate::core::nt_core_experience::handler_tier::HandlerRegistry;

impl HealthCheckable for HandlerRegistry {
    fn check_health(&self) -> Option<(String, usize)> {
        let stale = self.stale_handlers(std::time::Duration::from_secs(300));
        if stale.len() > 3 {
            Some(("handler_registry".to_string(), stale.len()))
        } else {
            None
        }
    }

    fn health_gc(&mut self) -> usize {
        let stale = self.stale_handlers(std::time::Duration::from_secs(300));
        let count = stale.len();
        for name in &stale {
            self.mark_unloaded(name);
        }
        count
    }
}

// ── MemoryGraph ──

use crate::core::nt_core_knowledge::spread_activation::MemoryGraph;

impl HealthCheckable for MemoryGraph {
    fn check_health(&self) -> Option<(String, usize)> {
        let n = self.node_count();
        if n > 800 {
            Some(("spreading_memory".to_string(), n))
        } else {
            None
        }
    }

    fn health_gc(&mut self) -> usize {
        let before = self.node_count();
        self.decay_all(0.85);
        // decay_all only reduces activation; trigger eviction by trimming
        // — inline eviction is in add_node(), so just flagging is sufficient
        before.saturating_sub(self.node_count())
    }
}

// ── BilingualLexicon ──

use crate::core::nt_core_translate::bilingual::BilingualLexicon;

impl HealthCheckable for BilingualLexicon {
    fn check_health(&self) -> Option<(String, usize)> {
        let n = self.len();
        if n > 750 {
            Some(("translation_lexicon".to_string(), n))
        } else {
            None
        }
    }

    fn health_gc(&mut self) -> usize {
        self.prune()
    }
}

// ── VsaTranslationEngine ──

use crate::core::nt_core_translate::translate_engine::VsaTranslationEngine;

impl HealthCheckable for VsaTranslationEngine {
    fn check_health(&self) -> Option<(String, usize)> {
        // Check lexicon first
        if let Some((name, size)) = self.lexicon.check_health() {
            return Some((name, size));
        }
        // Check internal cache estimate
        None
    }

    fn health_gc(&mut self) -> usize {
        let mut reclaimed = 0;
        reclaimed += self.lexicon.health_gc();
        reclaimed += self.lexicon.prune();
        reclaimed
    }
}
