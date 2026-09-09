use super::evolution_metrics::EvolutionMetrics;

pub enum EvolutionAction {
    DisableSource(String),
    EnableSource(String),
    AdjustQuality { source: String, quality: String },
    EnableCache,
    DisableCache,
    Alert { message: String },
}

pub fn decide_evolution(metrics: &EvolutionMetrics) -> Vec<EvolutionAction> {
    let mut actions = vec![];
    if metrics.search_success_rate < 0.8 {
        actions.push(EvolutionAction::Alert { message: "Low search success rate".into() });
    }
    for (source, health) in &metrics.source_health {
        if *health < 0.3 {
            actions.push(EvolutionAction::DisableSource(source.clone()));
        }
    }
    if metrics.cache_hit_rate < 0.3 && metrics.avg_latency_ms > 3000.0 {
        actions.push(EvolutionAction::EnableCache);
    }
    actions
}
