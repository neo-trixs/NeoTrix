use super::{KnowledgeSource, SourceAccessTracker, TaskType};
use crate::core::CapabilityVector;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActivationPolicy {
    Threshold(f64),
    FrequencyBoost {
        base_threshold: f64,
        boost: f64,
    },
    Hybrid {
        threshold: f64,
        boost: f64,
        decay_half_life_secs: f64,
    },
}

impl Default for ActivationPolicy {
    fn default() -> Self {
        ActivationPolicy::Threshold(0.7)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KsLifecycle {
    Young,
    Mature,
    Stale,
    Archived,
}

#[derive(Debug, Clone)]
pub struct RegisteredSource {
    pub source: KnowledgeSource,
    pub lifecycle: KsLifecycle,
    pub policy: ActivationPolicy,
    pub hit_count: u64,
}

#[derive(Debug, Clone)]
pub struct CascadeSelector {
    pub top_k: usize,
    pub fallback_sources: Vec<KnowledgeSource>,
}

impl Default for CascadeSelector {
    fn default() -> Self {
        Self {
            top_k: 3,
            fallback_sources: vec![KnowledgeSource::MemOS, KnowledgeSource::BaseUI],
        }
    }
}

pub struct KSActivationEngine {
    registry: HashMap<KnowledgeSource, RegisteredSource>,
    selector: CascadeSelector,
    tracker: SourceAccessTracker,
}

impl Default for KSActivationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl KSActivationEngine {
    pub fn new() -> Self {
        let all_sources = KnowledgeSource::all();
        let mut registry = HashMap::new();
        for source in all_sources {
            registry.insert(
                source,
                RegisteredSource {
                    source,
                    lifecycle: KsLifecycle::Young,
                    policy: ActivationPolicy::default(),
                    hit_count: 0,
                },
            );
        }
        Self {
            registry,
            selector: CascadeSelector::default(),
            tracker: SourceAccessTracker::new(3),
        }
    }

    pub fn register(&mut self, source: KnowledgeSource, policy: ActivationPolicy) {
        self.registry.insert(
            source,
            RegisteredSource {
                source,
                lifecycle: KsLifecycle::Young,
                policy,
                hit_count: 0,
            },
        );
    }

    pub fn select(&self, _task_type: TaskType, query: &CapabilityVector) -> Vec<KnowledgeSource> {
        let mut scored: Vec<(f64, KnowledgeSource)> = self
            .registry
            .values()
            .filter(|r| r.lifecycle != KsLifecycle::Archived)
            .filter_map(|r| {
                let cv = r.source.capability_vector();
                let sim = cv.similarity(query);
                let (threshold, boost) = match r.policy {
                    ActivationPolicy::Threshold(t) => (t, 0.0),
                    ActivationPolicy::FrequencyBoost {
                        base_threshold,
                        boost,
                    } => (base_threshold, boost),
                    ActivationPolicy::Hybrid {
                        threshold, boost, ..
                    } => (threshold, boost),
                };
                let freq_boost = if r.hit_count > 0 {
                    boost * (r.hit_count as f64).ln()
                } else {
                    0.0
                };
                let effective = sim + freq_boost;
                if effective >= threshold {
                    Some((effective, r.source))
                } else {
                    None
                }
            })
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        let top: Vec<KnowledgeSource> = scored
            .into_iter()
            .take(self.selector.top_k)
            .map(|(_, s)| s)
            .collect();
        if top.is_empty() {
            self.selector.fallback_sources.clone()
        } else {
            top
        }
    }

    pub fn record_hit(&mut self, source: KnowledgeSource) {
        self.tracker.record_access(&source);
        if let Some(registered) = self.registry.get_mut(&source) {
            registered.hit_count += 1;
            if registered.hit_count >= 3 {
                registered.lifecycle = KsLifecycle::Mature;
            }
        }
    }

    pub fn lifecycle_report(&self) -> Vec<(KnowledgeSource, KsLifecycle, u64)> {
        let mut report: Vec<_> = self
            .registry
            .values()
            .map(|r| (r.source, r.lifecycle, r.hit_count))
            .collect();
        report.sort_by_key(|b| std::cmp::Reverse(b.2));
        report
    }

    pub fn engine_size(&self) -> usize {
        self.registry.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_initialization_loads_all_ks() {
        let engine = KSActivationEngine::new();
        assert!(engine.engine_size() > 40);
    }

    #[test]
    fn test_register_new_source() {
        let mut engine = KSActivationEngine::new();
        engine.register(KnowledgeSource::BaseUI, ActivationPolicy::Threshold(0.5));
        assert!(engine.engine_size() > 40);
    }

    #[test]
    fn test_select_returns_sources() {
        let engine = KSActivationEngine::new();
        let cv = CapabilityVector::from_array(&[0.5; 23]).expect("value should be ok in test");
        let result = engine.select(TaskType::General, &cv);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_record_hit_advances_lifecycle() {
        let mut engine = KSActivationEngine::new();
        let source = KnowledgeSource::HeroUI;
        engine.record_hit(source);
        engine.record_hit(source);
        engine.record_hit(source);
        let report = engine.lifecycle_report();
        let (_, lifecycle, count) = report
            .iter()
            .find(|(s, _, _)| *s == source)
            .expect("value should be ok in test");
        assert_eq!(*lifecycle, KsLifecycle::Mature);
        assert_eq!(*count, 3);
    }

    #[test]
    fn test_engine_size_is_stable() {
        let engine = KSActivationEngine::new();
        let size = engine.engine_size();
        let engine2 = KSActivationEngine::new();
        assert_eq!(size, engine2.engine_size());
    }
}
