//! Community runtime: `from_runtime_jsonl` loader + matrix seeding entry.
//! Pure move from `nt_core_community_ingester.rs`; behavior unchanged.

use super::nt_community_data::{CommunityDataIngester, CommunityDataset};
use crate::l2_perception::nt_core_e8::E8TransitionMatrix;

impl CommunityDataIngester {
    /// `scripts/absorb-fable-2m.py`: per-task-type lists of {from,to,count},
    /// plus a `_meta` object) and build a `CommunityDataIngester` from it.
    ///
    /// This replaces the hardcoded `default_datasets()` priors with real
    /// 2M-trace data at runtime (the previously-missing `RuntimeCommunityLoader`).
    /// Unknown task types fall back to "General". Returns `None` if the file
    /// cannot be parsed.
    pub fn from_runtime_jsonl(
        path: &std::path::Path,
        base_source_url: &str,
        base_weight: f64,
    ) -> Option<Self> {
        let raw = std::fs::read_to_string(path).ok()?;
        let root: serde_json::Value = serde_json::from_str(&raw).ok()?;
        let mut datasets = Vec::new();
        for (task_type, val) in root.as_object()? {
            if task_type == "_meta" {
                continue;
            }
            let arr = val.as_array()?;
            let mut transitions: Vec<(u8, u8, u64)> = Vec::new();
            for item in arr {
                let obj = item.as_object()?;
                let from = obj.get("from")?.as_u64()? as u8;
                let to = obj.get("to")?.as_u64()? as u8;
                let count = obj.get("count")?.as_u64()?;
                transitions.push((from, to, count));
            }
            let name = format!("runtime_{task_type}");
            datasets.push(CommunityDataset {
                name: name.clone(),
                source_url: format!("{base_source_url}/{}", task_type.to_lowercase()),
                weight: base_weight,
                transitions,
                description: format!(
                    "Runtime-loaded {task_type} transitions from {base_source_url}"
                ),
            });
        }
        Some(Self { datasets })
    }
}

/// Apply all community dataset priors directly to the observer's transition matrix.
/// This is the primary entry point called from engine initialization.
/// The fused matrix is merged into the engine's observer TM via `merge()`,
/// so locally-observed transitions take priority (community patterns are priors,
/// not ground truth).
pub fn seed_transition_matrix_with_community(tm: &mut E8TransitionMatrix) {
    // 优先加载运行时 ETL 产出 (scripts/absorb-fable-2m.py → data/community_transitions.json),
    // 文件缺失时回退硬编码 default 数据集。R-P79: 确保 Python ETL 输出有生产消费者。
    let runtime_path = std::path::Path::new("data/community_transitions.json");
    let ingester = match CommunityDataIngester::from_runtime_jsonl(
        runtime_path,
        "https://huggingface.co/datasets/Glint-Research/Complete-FABLE.5-traces-2M",
        0.15,
    ) {
        Some(ing) => {
            log::info!(
                "[E8-COMMUNITY] loaded runtime transitions from {}",
                runtime_path.display()
            );
            ing
        }
        None => {
            log::info!("[E8-COMMUNITY] runtime transitions file missing, using hardcoded defaults");
            CommunityDataIngester::default()
        }
    };
    let community = ingester.fuse_all();
    tm.merge(&community);
    log::info!(
        "[E8-COMMUNITY] seeded transition matrix with {} virtual community observations",
        ingester.total_virtual_observations()
    );
}

#[cfg(test)]
mod tests {
    use crate::l2_perception::nt_core_e8::nt_core_community_ingester::CommunityDataIngester;

    #[test]
    fn test_from_runtime_jsonl_parses_fable2m_shape() {
        use std::io::Write;
        let mut tmp = std::env::temp_dir();
        tmp.push(format!("nt_fable2m_test_{}", std::process::id()));
        let json = r#"{
            "Reasoning": [{"from": 56, "to": 48, "count": 100}, {"from": 48, "to": 40, "count": 80}],
            "Coding": [{"from": 56, "to": 42, "count": 40}],
            "_meta": {"source": "x", "total_rows": 10, "valid_traces": 5, "task_type_distribution": {}, "total_transition_pairs": 220}
        }"#;
        {
            let mut f = std::fs::File::create(&tmp).unwrap();
            f.write_all(json.as_bytes()).unwrap();
        }
        let ingester = CommunityDataIngester::from_runtime_jsonl(
            &tmp,
            "https://huggingface.co/datasets/Complete-FABLE.5-traces-2M",
            0.15,
        );
        let _ = std::fs::remove_file(&tmp);
        let ingester = ingester.expect("runtime loader should parse");
        assert_eq!(ingester.datasets.len(), 2);
        assert!(ingester
            .datasets
            .iter()
            .any(|d| d.name == "runtime_Reasoning"));
        assert!(ingester.datasets.iter().any(|d| d.name == "runtime_Coding"));
        let reasoning = ingester
            .datasets
            .iter()
            .find(|d| d.name == "runtime_Reasoning")
            .unwrap();
        assert_eq!(reasoning.transitions, vec![(56, 48, 100), (48, 40, 80)]);
        assert!((reasoning.weight - 0.15).abs() < 1e-9);
    }
}
