use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use serde::{Deserialize, Serialize};

fn now_secs() -> i64 {
    crate::core::nt_core_time::unix_now_secs() as i64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionGuideline {
    pub id: String,
    pub pattern: String,
    pub priority: u8,
    pub max_tokens: usize,
    pub preserve_keys: Vec<String>,
    pub compress_action: String,
    pub created_at: i64,
    pub success_count: u64,
    pub failure_count: u64,
}

impl CompressionGuideline {
    pub fn is_effective(&self) -> bool {
        let total = self.success_count + self.failure_count;
        total > 0 && self.success_count as f64 / total as f64 > 0.6
    }

    pub fn confidence(&self) -> f64 {
        let total = (self.success_count + self.failure_count).max(1);
        self.success_count as f64 / total as f64
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedEntry {
    pub summary: String,
    pub original_count: usize,
    pub timestamp: i64,
    pub guideline_id: String,
    pub compression_ratio: f64,
}

pub struct CognitiveContextCompressor {
    pub guidelines: Vec<CompressionGuideline>,
    pub compressed: Vec<CompressedEntry>,
    pub max_compressed: usize,
    pub total_original: usize,
    pub total_compressed: usize,
    pub enabled: bool,
    /// VSA similarity threshold for clustering (0.0-1.0).
    /// Entries with similarity above this are merged into a single compressed entry.
    pub vsa_threshold: f64,
    /// Total VSA-based cluster merges performed.
    pub vsa_cluster_count: u64,
}

impl CognitiveContextCompressor {
    pub fn new() -> Self {
        Self {
            guidelines: Self::default_guidelines(),
            compressed: Vec::new(),
            max_compressed: 50,
            total_original: 0,
            total_compressed: 0,
            enabled: true,
            vsa_threshold: 0.85,
            vsa_cluster_count: 0,
        }
    }

    fn default_guidelines() -> Vec<CompressionGuideline> {
        vec![
            CompressionGuideline {
                id: "gl_high_freq".into(),
                pattern: "high_frequency_handler".into(),
                priority: 1,
                max_tokens: 50,
                preserve_keys: vec!["handler".into(), "result".into(), "latency".into()],
                compress_action: "summarize".into(),
                created_at: now_secs(),
                success_count: 0,
                failure_count: 0,
            },
            CompressionGuideline {
                id: "gl_low_impact".into(),
                pattern: "low_impact_event".into(),
                priority: 2,
                max_tokens: 20,
                preserve_keys: vec!["cycle".into(), "outcome".into()],
                compress_action: "drop".into(),
                created_at: now_secs(),
                success_count: 0,
                failure_count: 0,
            },
            CompressionGuideline {
                id: "gl_thought_trace".into(),
                pattern: "thought".into(),
                priority: 0,
                max_tokens: 100,
                preserve_keys: vec!["content".into(), "score".into()],
                compress_action: "truncate".into(),
                created_at: now_secs(),
                success_count: 0,
                failure_count: 0,
            },
        ]
    }

    pub fn compress_thought_history(
        &mut self,
        history: &[(String, Vec<u8>, f64)],
    ) -> Vec<(String, Vec<u8>, f64)> {
        if !self.enabled || history.len() < 10 {
            return history.to_vec();
        }

        self.total_original += history.len();
        let mut result: Vec<(String, Vec<u8>, f64)> = Vec::new();

        let recent_cutoff = 5;
        let (recent, older) = history.split_at(history.len().saturating_sub(recent_cutoff));

        result.extend_from_slice(recent);

        // === Step 1: VSA-based clustering ===
        // Group entries with VSA similarity above threshold.
        let mut used: Vec<bool> = vec![false; older.len()];
        let mut cluster_entries: Vec<CompressedEntry> = Vec::new();

        for i in 0..older.len() {
            if used[i] {
                continue;
            }
            let mut cluster: Vec<usize> = vec![i];
            used[i] = true;
            for j in (i + 1)..older.len() {
                if used[j] {
                    continue;
                }
                let sim = QuantizedVSA::similarity(&older[i].1, &older[j].1);
                if sim >= self.vsa_threshold {
                    cluster.push(j);
                    used[j] = true;
                }
            }

            if cluster.len() > 1 {
                // Merge cluster into a single bundled entry
                self.vsa_cluster_count += 1;
                let vsa_refs: Vec<&[u8]> =
                    cluster.iter().map(|&idx| older[idx].1.as_slice()).collect();
                let bundled = QuantizedVSA::bundle(&vsa_refs);
                let combined_text: String = cluster
                    .iter()
                    .map(|&idx| older[idx].0.as_str())
                    .collect::<Vec<&str>>()
                    .join(" | ");
                let avg_score: f64 =
                    cluster.iter().map(|&idx| older[idx].2).sum::<f64>() / cluster.len() as f64;

                let summary = if combined_text.len() > 100 {
                    format!(
                        "[vsa_cluster:{}] {}...",
                        cluster.len(),
                        &combined_text[..100]
                    )
                } else {
                    format!("[vsa_cluster:{}] {}", cluster.len(), combined_text)
                };

                result.push((summary, bundled, avg_score));

                cluster_entries.push(CompressedEntry {
                    summary: format!(
                        "vsa_cluster:{}_events|sim>{:.2}",
                        cluster.len(),
                        self.vsa_threshold
                    ),
                    original_count: cluster.len(),
                    timestamp: now_secs(),
                    guideline_id: "gl_vsa_cluster".into(),
                    compression_ratio: 1.0 / cluster.len() as f64,
                });
            }
        }

        // Record VSA cluster entries in compressed log
        for entry in &cluster_entries {
            self.compressed.push(entry.clone());
            if self.compressed.len() > self.max_compressed {
                self.compressed.remove(0);
            }
        }

        // === Step 2: VSA-based batching for unclustered entries ===
        let mut batch_count = 0usize;
        let mut batch_summary = String::new();
        let mut batch_score_sum = 0.0f64;
        let mut batch_anchor: Option<Vec<u8>> = None;

        for (i, (text, vsa, score)) in older.iter().enumerate() {
            if used[i] {
                continue;
            }
            let should_batch = match &batch_anchor {
                None => true,
                Some(anchor) => QuantizedVSA::similarity(vsa, anchor) >= 0.7,
            };
            if should_batch {
                if batch_anchor.is_none() {
                    batch_anchor = Some(vsa.clone());
                }
                batch_count += 1;
                batch_summary.push_str(&text[..50.min(text.len())]);
                batch_summary.push('|');
                batch_score_sum += score;
            } else {
                let guideline = self.select_guideline(text);
                match guideline.compress_action.as_str() {
                    "drop" => {
                        self.total_compressed += 1;
                    }
                    "summarize" => {
                        let key = guideline.preserve_keys.first().cloned().unwrap_or_default();
                        let summary = format!("[{}:{}]", key, &text[..40.min(text.len())]);
                        result.push((summary, vsa.clone(), *score));
                        self.total_compressed += 1;
                    }
                    _ => {
                        result.push((text.clone(), vsa.clone(), *score));
                    }
                }
            }
        }

        if batch_count > 1 {
            let entry = CompressedEntry {
                summary: format!(
                    "{} events: {}",
                    batch_count,
                    batch_summary.trim_end_matches('|')
                ),
                original_count: batch_count,
                timestamp: now_secs(),
                guideline_id: "gl_batch".into(),
                compression_ratio: 1.0 / batch_count as f64,
            };
            self.compressed.push(entry);
            if self.compressed.len() > self.max_compressed {
                self.compressed.remove(0);
            }
        }

        result
    }

    fn select_guideline(&self, text: &str) -> &CompressionGuideline {
        for gl in &self.guidelines {
            if gl.priority <= 2 || text.contains(&gl.pattern) {
                return gl;
            }
        }
        &self.guidelines[2]
    }

    pub fn report_failure(&mut self, guideline_id: &str) {
        if let Some(gl) = self.guidelines.iter_mut().find(|g| g.id == guideline_id) {
            gl.failure_count += 1;
        }
    }

    pub fn report_success(&mut self, guideline_id: &str) {
        if let Some(gl) = self.guidelines.iter_mut().find(|g| g.id == guideline_id) {
            gl.success_count += 1;
        }
    }

    pub fn add_guideline(&mut self, guideline: CompressionGuideline) {
        self.guidelines.push(guideline);
    }

    pub fn update_guideline_from_failure(
        &mut self,
        guideline_id: &str,
        failure_analysis: &str,
    ) -> Option<String> {
        let gl = self.guidelines.iter_mut().find(|g| g.id == guideline_id)?;
        gl.failure_count += 1;

        if failure_analysis.contains("too_aggressive") && gl.max_tokens < 500 {
            gl.max_tokens = (gl.max_tokens as f64 * 1.5) as usize;
            return Some(format!(
                "guideline:relaxed|{}|max_tokens->{}",
                guideline_id, gl.max_tokens
            ));
        }
        if failure_analysis.contains("missing_key") && gl.compress_action != "summarize" {
            gl.compress_action = "summarize".into();
            return Some(format!(
                "guideline:adjusted|{}|action->summarize",
                guideline_id
            ));
        }
        None
    }

    pub fn stats(&self) -> String {
        let ratio = if self.total_original > 0 {
            self.total_compressed as f64 / self.total_original as f64
        } else {
            0.0
        };
        let effective = self.guidelines.iter().filter(|g| g.is_effective()).count();
        format!(
            "compressor:{}_guidelines|{}_effective|{}_compressed|{:.2}_ratio|vsa_clusters:{}",
            self.guidelines.len(),
            effective,
            self.total_compressed,
            ratio,
            self.vsa_cluster_count
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_compress_small_history() {
        let mut compressor = CognitiveContextCompressor::new();
        let history: Vec<(String, Vec<u8>, f64)> = (0..5)
            .map(|i| (format!("event {}", i), vec![0u8; 64], 0.5))
            .collect();
        let result = compressor.compress_thought_history(&history);
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_compress_large_history() {
        let mut compressor = CognitiveContextCompressor::new();
        let history: Vec<(String, Vec<u8>, f64)> = (0..20)
            .map(|i| {
                (
                    format!("long event content that should trigger compression {}", i),
                    vec![0u8; 64],
                    0.5,
                )
            })
            .collect();
        let result = compressor.compress_thought_history(&history);
        assert!(result.len() <= 20);
        assert!(result.len() >= 5);
    }

    #[test]
    fn test_vsa_clustering_merges_similar() {
        let mut compressor = CognitiveContextCompressor::new();
        // Create entries with identical VSA vectors → should be merged
        let vsa_vec = vec![42u8; 64];
        let history: Vec<(String, Vec<u8>, f64)> = (0..15)
            .map(|i| (format!("similar thought {}", i), vsa_vec.clone(), 0.5))
            .collect();
        let result = compressor.compress_thought_history(&history);
        // At least some should be VSA-merged
        assert!(result.len() < 15);
        // VSA cluster count should be > 0
        assert!(compressor.vsa_cluster_count > 0);
        // Verify bundled entries exist
        let has_vsa_cluster = result.iter().any(|(t, _, _)| t.contains("vsa_cluster"));
        assert!(has_vsa_cluster);
    }

    #[test]
    fn test_vsa_clustering_dissimilar_not_merged() {
        let mut compressor = CognitiveContextCompressor::new();
        // Create entries with dissimilar VSA vectors (different random seeds)
        let history: Vec<(String, Vec<u8>, f64)> = (0..12)
            .map(|i| {
                let seed = (i as u64).wrapping_mul(2654435761);
                let vsa = QuantizedVSA::seeded_random(seed, 64);
                (format!("distinct thought {}", i), vsa, 0.5)
            })
            .collect();
        let result = compressor.compress_thought_history(&history);
        // With different vectors, most should NOT be clustered
        // The VSA threshold is 0.85 and random vectors should be far apart
        let vsa_merged_count = result
            .iter()
            .filter(|(t, _, _)| t.contains("vsa_cluster"))
            .count();
        assert!(
            vsa_merged_count < 3,
            "should have few VSA merges for dissimilar vectors"
        );
    }

    #[test]
    fn test_guideline_update_from_failure() {
        let mut compressor = CognitiveContextCompressor::new();
        let r = compressor
            .update_guideline_from_failure("gl_high_freq", "too_aggressive lost important context");
        assert!(r.is_some());
        assert!(r.unwrap().contains("relaxed"));
    }

    #[test]
    fn test_stats() {
        let compressor = CognitiveContextCompressor::new();
        let s = compressor.stats();
        assert!(s.contains("compressor:"));
        assert!(s.contains("guidelines"));
        assert!(s.contains("vsa_clusters:"));
    }
}
