use super::broadcast::downtree_broadcast;
use super::chunk::{ExternalScores, ScoreWeights};
use super::competition::uptree_competition;
use super::evolver::ProcessorEvolver;
use super::graph::ProcessorGraph;
use super::nash_competition::{CompetitionMode, NashCompetitionEngine, NashConfig};
use super::processor::LtmProcessor;

pub struct CtmConfig {
    pub max_iterations: usize,
    pub temperature: f64,
    pub output_threshold: f64,
    pub link_threshold: f64,
    pub competition_mode: CompetitionMode,
    pub nash_config: NashConfig,
}

impl Default for CtmConfig {
    fn default() -> Self {
        Self {
            max_iterations: 3,
            temperature: 0.1,
            output_threshold: 0.8,
            link_threshold: 0.7,
            competition_mode: CompetitionMode::Nash,
            nash_config: NashConfig::default(),
        }
    }
}

pub struct CtmResult {
    pub gist: Vec<u8>,
    pub weight: f64,
    pub iterations_used: usize,
    pub winner_name: String,
    pub link_count: usize,
}

pub fn ctm_inference(
    processors: &mut [Box<dyn LtmProcessor>],
    graph: &mut ProcessorGraph,
    query: &[u8],
    config: &CtmConfig,
    score_weights: &ScoreWeights,
    external: Option<&ExternalScores>,
    mut evolver: Option<&mut ProcessorEvolver>,
    mut nash_engine: Option<&mut NashCompetitionEngine>,
) -> CtmResult {
    for t in 0..config.max_iterations {
        let chunks = graph.ask_all_parallel(processors, query, t, false, external);
        let winner = match config.competition_mode {
            CompetitionMode::Nash => {
                if let Some(ne) = nash_engine.as_deref_mut() {
                    ne.nash_competition(&chunks, score_weights)
                } else {
                    uptree_competition(&chunks, score_weights, config.temperature)
                }
            }
            CompetitionMode::Softmax => {
                uptree_competition(&chunks, score_weights, config.temperature)
            }
        };
        let Some(winner) = winner else {
            continue;
        };
        let w = winner.weight(score_weights);

        if let Some(ev) = evolver.as_deref_mut() {
            ev.record_outcomes(&chunks, &winner.processor_name);
        }

        if w >= config.output_threshold || t == config.max_iterations - 1 {
            let link_count = graph.link_count();
            return CtmResult {
                gist: winner.gist.clone(),
                weight: w,
                iterations_used: t + 1,
                winner_name: winner.processor_name.clone(),
                link_count,
            };
        }

        downtree_broadcast(processors, winner);
        graph.link_from_broadcast(winner, &chunks, config.link_threshold);
    }

    CtmResult {
        gist: query.to_vec(),
        weight: 0.0,
        iterations_used: config.max_iterations,
        winner_name: "none".to_string(),
        link_count: graph.link_count(),
    }
}

pub struct CtmEngine {
    pub processors: Vec<Box<dyn LtmProcessor>>,
    pub graph: ProcessorGraph,
    pub config: CtmConfig,
    pub score_weights: ScoreWeights,
    pub total_inferences: u64,
    pub evolver: ProcessorEvolver,
    pub nash_engine: NashCompetitionEngine,
}

impl CtmEngine {
    pub fn new(processors: Vec<Box<dyn LtmProcessor>>) -> Self {
        let graph = ProcessorGraph::new(&processors);
        Self {
            processors,
            graph,
            config: CtmConfig::default(),
            score_weights: ScoreWeights::default(),
            total_inferences: 0,
            evolver: ProcessorEvolver::default(),
            nash_engine: NashCompetitionEngine::new(),
        }
    }

    pub fn infer(&mut self, query: &[u8]) -> CtmResult {
        self.infer_with_external(query, None)
    }

    pub fn infer_with_external(
        &mut self,
        query: &[u8],
        external: Option<&ExternalScores>,
    ) -> CtmResult {
        self.total_inferences += 1;
        ctm_inference(
            &mut self.processors,
            &mut self.graph,
            query,
            &self.config,
            &self.score_weights,
            external,
            Some(&mut self.evolver),
            Some(&mut self.nash_engine),
        )
    }

    pub fn stats(&self) -> CtmStats {
        let es = self.evolver.stats();
        CtmStats {
            total_inferences: self.total_inferences,
            processor_count: self.processors.len(),
            link_count: self.graph.link_count(),
            evolver_avg_win_rate: es.avg_win_rate,
            evolver_stagnating_count: es.stagnating_count,
            competition_mode: self.config.competition_mode,
            nash_iterations: self.nash_engine.stats.iterations_used as u64,
            nash_equilibrium_found: self.nash_engine.stats.nash_equilibrium_found,
        }
    }
}

pub struct CtmStats {
    pub total_inferences: u64,
    pub processor_count: usize,
    pub link_count: usize,
    pub evolver_avg_win_rate: f64,
    pub evolver_stagnating_count: usize,
    pub competition_mode: CompetitionMode,
    pub nash_iterations: u64,
    pub nash_equilibrium_found: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctm_engine_creation() {
        use crate::core::nt_core_ctm::processor::default_processors;
        let engine = CtmEngine::new(default_processors());
        assert_eq!(engine.processors.len(), 4);
        assert_eq!(engine.config.max_iterations, 3);
    }

    #[test]
    fn test_ctm_inference_runs() {
        use crate::core::nt_core_ctm::processor::default_processors;
        let mut engine = CtmEngine::new(default_processors());
        let query = vec![0u8; 64];
        let result = engine.infer(&query);
        assert!(result.iterations_used >= 1);
        assert!(result.iterations_used <= 3);
        assert!(!result.winner_name.is_empty());
        assert!(result.weight >= 0.0);
    }

    #[test]
    fn test_ctm_stats() {
        use crate::core::nt_core_ctm::processor::default_processors;
        let mut engine = CtmEngine::new(default_processors());
        let q = vec![0u8; 64];
        engine.infer(&q);
        engine.infer(&q);
        let stats = engine.stats();
        assert_eq!(stats.total_inferences, 2);
        assert_eq!(stats.processor_count, 4);
    }
}
