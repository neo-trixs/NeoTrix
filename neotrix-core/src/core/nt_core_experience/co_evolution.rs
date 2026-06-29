use std::path::PathBuf;

use super::self_evolution_loop::{MutationOp, SelfEvolutionArchive, SelfEvolutionStep};

/// A single trajectory record capturing one SEAL mutation step
/// in a format suitable for RL / GRPO training pipelines.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrajectoryRecord {
    pub generation: u32,
    pub mutation_type: String,
    pub mutation_target: String,
    pub score_before: f64,
    pub score_after: f64,
    pub delta: f64,
    pub accepted: bool,
    pub compile_success: bool,
    pub timestamp: u64,
    pub cmp_score: Option<f64>,
}

/// Bridge that converts SelfEvolutionLoop steps into RL-ready trajectory records.
///
/// Buffers `TrajectoryRecord`s from ingested `SelfEvolutionStep`s and can export
/// them as JSONL files for downstream GRPO / RL training.
pub struct CoEvolutionBridge {
    pub min_trajectory_length: usize,
    trajectory_buffer: Vec<TrajectoryRecord>,
    pub output_dir: PathBuf,
}

impl CoEvolutionBridge {
    pub fn new(output_dir: PathBuf) -> Self {
        Self {
            min_trajectory_length: 1,
            trajectory_buffer: Vec::new(),
            output_dir,
        }
    }

    /// Extract mutation-type label and target string from a step.
    fn extract_target(step: &SelfEvolutionStep) -> (String, String) {
        match &step.mutation {
            MutationOp::TuneParam { target, delta: _ } => ("TuneParam".to_string(), target.clone()),
            MutationOp::AddHandler { position, code: _ } => {
                ("AddHandler".to_string(), position.clone())
            }
            MutationOp::RewriteHandler { name, code: _ } => {
                ("RewriteHandler".to_string(), name.clone())
            }
            MutationOp::SwapPolicy { gates } => ("SwapPolicy".to_string(), gates.join(",")),
            MutationOp::RewritePrimitive { name, impl_: _ } => {
                ("RewritePrimitive".to_string(), name.clone())
            }
            MutationOp::RewriteMeta { strategy } => {
                ("RewriteMeta".to_string(), format!("v{}", strategy.version))
            }
            MutationOp::SelfModifyProposal {
                target,
                target_type,
                source_code: _,
            } => (
                format!("SelfModifyProposal:{}", target_type),
                target.clone(),
            ),
        }
    }

    /// Ingest a `SelfEvolutionStep` and append a `TrajectoryRecord` to the buffer.
    pub fn ingest_step(&mut self, step: &SelfEvolutionStep, _archive: &SelfEvolutionArchive) {
        let score_after = step.score_after.unwrap_or(step.score_before);
        let delta = score_after - step.score_before;
        let (mutation_type, mutation_target) = Self::extract_target(step);

        self.trajectory_buffer.push(TrajectoryRecord {
            generation: step.generation,
            mutation_type,
            mutation_target,
            score_before: step.score_before,
            score_after,
            delta,
            accepted: step.accepted,
            compile_success: step.compiles,
            timestamp: step.timestamp,
            cmp_score: step.cmp_score,
        });
    }

    /// Export all buffered records to a JSONL file in `output_dir`.
    /// File name: `trajectory_{unix_ts}.jsonl`.
    /// Returns the number of records written.
    pub fn export_jsonl(&self) -> Result<usize, String> {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let path = self.output_dir.join(format!("trajectory_{}.jsonl", ts));
        let mut content = String::new();
        for record in &self.trajectory_buffer {
            let line = serde_json::to_string(record).map_err(|e| format!("JSON error: {}", e))?;
            content.push_str(&line);
            content.push('\n');
        }
        std::fs::write(&path, &content)
            .map_err(|e| format!("Write error {}: {}", path.display(), e))?;
        Ok(self.trajectory_buffer.len())
    }

    /// Export steps directly as JSONL without touching the buffer.
    pub fn export_batch(steps: &[SelfEvolutionStep], output_path: &str) -> Result<usize, String> {
        let mut content = String::new();
        for step in steps {
            let score_after = step.score_after.unwrap_or(step.score_before);
            let delta = score_after - step.score_before;
            let (mutation_type, mutation_target) = Self::extract_target(step);
            let record = TrajectoryRecord {
                generation: step.generation,
                mutation_type,
                mutation_target,
                score_before: step.score_before,
                score_after,
                delta,
                accepted: step.accepted,
                compile_success: step.compiles,
                timestamp: step.timestamp,
                cmp_score: step.cmp_score,
            };
            let line = serde_json::to_string(&record).map_err(|e| format!("JSON error: {}", e))?;
            content.push_str(&line);
            content.push('\n');
        }
        std::fs::write(output_path, &content)
            .map_err(|e| format!("Write error {}: {}", output_path, e))?;
        Ok(steps.len())
    }

    /// Generate a GRPO-compatible training configuration as JSON.
    pub fn generate_training_config(&self) -> serde_json::Value {
        serde_json::json!({
            "algorithm": "grpo",
            "clip_epsilon": 0.2,
            "clip_higher": true,
            "dynamic_sampling": true,
            "group_size": 8,
            "learning_rate": 0.01,
            "beta_kl": 0.001,
            "source": "co_evolution_bridge",
            "min_trajectory_length": self.min_trajectory_length,
            "total_records": self.trajectory_buffer.len(),
            "output_dir": self.output_dir.to_string_lossy(),
        })
    }

    pub fn trajectory_buffer(&self) -> &[TrajectoryRecord] {
        &self.trajectory_buffer
    }

    pub fn clear_buffer(&mut self) {
        self.trajectory_buffer.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_experience::self_evolution_loop::MetaStrategy;

    fn make_step(
        id: u64,
        gen: u32,
        before: f64,
        after: Option<f64>,
        accepted: bool,
        compiles: bool,
        cmp: Option<f64>,
    ) -> SelfEvolutionStep {
        SelfEvolutionStep {
            id,
            mutation: MutationOp::TuneParam {
                target: "cognitive_load.thinking_budget".to_string(),
                delta: 0.12,
            },
            parent_id: 0,
            score_before: before,
            score_after: after,
            compiles,
            accepted,
            timestamp: 1000 + id,
            generation: gen,
            cmp_score: cmp,
        }
    }

    fn make_archive(steps: Vec<SelfEvolutionStep>) -> SelfEvolutionArchive {
        let mut archive = SelfEvolutionArchive::new();
        for s in steps {
            archive.add(s);
        }
        archive
    }

    #[test]
    fn test_bridge_new() {
        let dir = std::env::temp_dir();
        let bridge = CoEvolutionBridge::new(dir.join("coev_test"));
        assert_eq!(bridge.min_trajectory_length, 1);
        assert!(bridge.trajectory_buffer().is_empty());
    }

    #[test]
    fn test_ingest_step_creates_record() {
        let dir = std::env::temp_dir();
        let mut bridge = CoEvolutionBridge::new(dir.join("coev_test"));
        let step = make_step(1, 0, 0.5, Some(0.62), true, true, Some(0.31));
        let archive = make_archive(vec![step.clone()]);
        bridge.ingest_step(&step, &archive);

        assert_eq!(bridge.trajectory_buffer().len(), 1);
        let rec = &bridge.trajectory_buffer()[0];
        assert_eq!(rec.generation, 0);
        assert_eq!(rec.mutation_type, "TuneParam");
        assert_eq!(rec.mutation_target, "cognitive_load.thinking_budget");
        assert!((rec.score_before - 0.5).abs() < 1e-10);
        assert!((rec.score_after - 0.62).abs() < 1e-10);
        assert!((rec.delta - 0.12).abs() < 1e-10);
        assert!(rec.accepted);
        assert!(rec.compile_success);
        assert!((rec.cmp_score.unwrap() - 0.31).abs() < 1e-10);
    }

    #[test]
    fn test_ingest_step_without_score_after() {
        let dir = std::env::temp_dir();
        let mut bridge = CoEvolutionBridge::new(dir.join("coev_test"));
        let step = make_step(2, 1, 0.4, None, false, false, None);
        let archive = make_archive(vec![step.clone()]);
        bridge.ingest_step(&step, &archive);

        assert_eq!(bridge.trajectory_buffer().len(), 1);
        let rec = &bridge.trajectory_buffer()[0];
        assert!((rec.score_after - 0.4).abs() < 1e-10);
        assert!((rec.delta - 0.0).abs() < 1e-10);
        assert!(!rec.accepted);
        assert!(!rec.compile_success);
        assert!(rec.cmp_score.is_none());
    }

    #[test]
    fn test_export_jsonl_writes_file() {
        let dir = tempfile::tempdir().unwrap();
        let mut bridge = CoEvolutionBridge::new(dir.path().to_path_buf());
        let step = make_step(3, 0, 0.5, Some(0.62), true, true, Some(0.31));
        let archive = make_archive(vec![step.clone()]);
        bridge.ingest_step(&step, &archive);

        let count = bridge.export_jsonl().unwrap();
        assert_eq!(count, 1);

        let entries: Vec<_> = std::fs::read_dir(dir.path()).unwrap().collect();
        assert_eq!(entries.len(), 1);
        let path = entries.into_iter().next().unwrap().unwrap().path();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.starts_with("{\""));
        assert!(content.ends_with('\n'));
        let parsed: serde_json::Value = serde_json::from_str(content.trim()).unwrap();
        assert_eq!(parsed["generation"], 0);
        assert_eq!(parsed["mutation_type"], "TuneParam");
    }

    #[test]
    fn test_export_batch_writes_file() {
        let dir = tempfile::tempdir().unwrap();
        let steps = vec![
            make_step(10, 0, 0.5, Some(0.62), true, true, Some(0.31)),
            make_step(11, 1, 0.3, Some(0.45), true, false, None),
        ];
        let output_path = dir.path().join("batch.jsonl");
        let count = CoEvolutionBridge::export_batch(&steps, output_path.to_str().unwrap()).unwrap();
        assert_eq!(count, 2);

        let content = std::fs::read_to_string(&output_path).unwrap();
        let lines: Vec<&str> = content.trim().lines().collect();
        assert_eq!(lines.len(), 2);
        let parsed: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(parsed["generation"], 0);
        assert_eq!(parsed["mutation_type"], "TuneParam");
    }

    #[test]
    fn test_generate_training_config() {
        let dir = std::env::temp_dir();
        let mut bridge = CoEvolutionBridge::new(dir.join("coev_test"));
        bridge.min_trajectory_length = 3;

        let step = make_step(20, 5, 0.7, Some(0.85), true, true, None);
        let archive = make_archive(vec![step.clone()]);
        bridge.ingest_step(&step, &archive);

        let config = bridge.generate_training_config();
        assert_eq!(config["algorithm"], "grpo");
        assert_eq!(config["clip_epsilon"], 0.2);
        assert_eq!(config["min_trajectory_length"], 3);
        assert_eq!(config["total_records"], 1);
        assert_eq!(config["source"], "co_evolution_bridge");
        let _serialized = serde_json::to_string_pretty(&config).unwrap();
    }

    #[test]
    fn test_extract_target_variants() {
        let check = |op: MutationOp, expected_type: &str, expected_target: &str| {
            let step = SelfEvolutionStep {
                id: 99,
                mutation: op,
                parent_id: 0,
                score_before: 0.5,
                score_after: Some(0.6),
                compiles: true,
                accepted: true,
                timestamp: 2000,
                generation: 2,
                cmp_score: None,
            };
            let (t, target) = CoEvolutionBridge::extract_target(&step);
            assert_eq!(t, expected_type);
            assert_eq!(target, expected_target);
        };

        check(
            MutationOp::TuneParam {
                target: "rate".into(),
                delta: 0.1,
            },
            "TuneParam",
            "rate",
        );
        check(
            MutationOp::AddHandler {
                position: "pos_a".into(),
                code: "code".into(),
            },
            "AddHandler",
            "pos_a",
        );
        check(
            MutationOp::RewriteHandler {
                name: "handler_x".into(),
                code: "code".into(),
            },
            "RewriteHandler",
            "handler_x",
        );
        check(
            MutationOp::SwapPolicy {
                gates: vec!["g1".into(), "g2".into()],
            },
            "SwapPolicy",
            "g1,g2",
        );
        check(
            MutationOp::RewritePrimitive {
                name: "prim_vsa".into(),
                impl_: "impl".into(),
            },
            "RewritePrimitive",
            "prim_vsa",
        );
        check(
            MutationOp::RewriteMeta {
                strategy: MetaStrategy::default_v1(),
            },
            "RewriteMeta",
            "v1",
        );
        check(
            MutationOp::SelfModifyProposal {
                target: "handler_y".into(),
                target_type: "handler".into(),
                source_code: "code".into(),
            },
            "SelfModifyProposal:handler",
            "handler_y",
        );
    }
}
