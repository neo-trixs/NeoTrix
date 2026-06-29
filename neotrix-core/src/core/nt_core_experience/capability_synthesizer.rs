use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};
use log;
use neotrix_mind::distillation::cross_model_distiller::DistillationReport;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CapabilityType {
    Primitive,
    Composite,
    Generated,
    Pipeline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub cap_type: CapabilityType,
    pub sub_ids: Vec<u64>,
    pub vsa_vector: Vec<u8>,
    pub invocation_count: u64,
    pub success_rate: f64,
}

#[derive(Debug, Clone)]
pub struct PipelineStage {
    pub name: String,
    pub description: String,
    pub input_type: String,
    pub output_type: String,
    pub handler: String,
    pub vsa_vector: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct PipelineCapability {
    pub id: u64,
    pub name: String,
    pub stages: Vec<PipelineStage>,
    pub input_type: String,
    pub output_type: String,
    pub vsa_vector: Vec<u8>,
}

pub struct PipelineExecutor {
    pipelines: Vec<PipelineCapability>,
    stage_outputs: HashMap<String, String>,
    next_id: u64,
}

impl PipelineExecutor {
    pub fn new() -> Self {
        Self {
            pipelines: Vec::new(),
            stage_outputs: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn register_pipeline(&mut self, pipeline: PipelineCapability) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let mut p = pipeline;
        p.id = id;
        self.pipelines.push(p);
        id
    }

    pub fn execute(&mut self, pipeline_id: u64, input: &str) -> Result<String, String> {
        let pipeline = self
            .pipelines
            .iter()
            .find(|p| p.id == pipeline_id)
            .ok_or_else(|| format!("pipeline {} not found", pipeline_id))?;

        self.stage_outputs.clear();

        let mut current_input = input.to_string();
        let mut current_type = pipeline.input_type.clone();

        for (i, stage) in pipeline.stages.iter().enumerate() {
            if stage.input_type != current_type {
                return Err(format!(
                    "stage {} '{}': expected input type '{}', got '{}'",
                    i, stage.name, stage.input_type, current_type
                ));
            }

            // In a real implementation, this would look up the handler in CapabilitySynthesizer
            // and invoke it. Here we simulate with a simple transform.
            let output = format!(
                "[{} processed by {}]: {}",
                stage.name, stage.handler, current_input
            );
            self.stage_outputs
                .insert(stage.name.clone(), output.clone());
            current_input = output;
            current_type = stage.output_type.clone();
        }

        Ok(current_input)
    }

    pub fn discover_pipelines(
        &self,
        synthesizer: &CapabilitySynthesizer,
        input: &str,
    ) -> Vec<&PipelineCapability> {
        let query = CapabilitySynthesizer::encode(input);
        self.pipelines
            .iter()
            .filter(|p| {
                CapabilitySynthesizer::similarity(&query, &p.vsa_vector)
                    >= synthesizer.min_match_threshold
            })
            .collect()
    }

    pub fn pipeline_count(&self) -> usize {
        self.pipelines.len()
    }

    pub fn get_pipeline(&self, id: u64) -> Option<&PipelineCapability> {
        self.pipelines.iter().find(|p| p.id == id)
    }
}

impl Default for PipelineExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum SynthesisOutcome {
    DirectMatch(u64),
    CompositeCreated(u64),
    NeedsHuman(String),
}

#[derive(Debug, Clone)]
pub struct CapabilityStats {
    pub total_capabilities: usize,
    pub primitives: usize,
    pub composites: usize,
    pub generated: usize,
    pub pipelines: usize,
    pub synthesized_count: u64,
    pub avg_success_rate: f64,
}

#[derive(Clone)]
pub struct CapabilitySynthesizer {
    pub(crate) capabilities: Vec<Capability>,
    next_id: u64,
    synthesized_count: u64,
    min_composition_threshold: f64,
    pub(crate) min_match_threshold: f64,
    max_capabilities: usize,
}

impl CapabilitySynthesizer {
    pub fn new() -> Self {
        Self {
            capabilities: Vec::new(),
            next_id: 1,
            synthesized_count: 0,
            min_composition_threshold: 0.45,
            min_match_threshold: 0.55,
            max_capabilities: 200,
        }
    }

    pub fn with_max_capabilities(mut self, max: usize) -> Self {
        self.max_capabilities = max;
        self
    }

    pub fn register_primitive(&mut self, name: &str, description: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let desc_text = format!("{}: {}", name, description);
        let vsa_vector = Self::encode(&desc_text);
        self.capabilities.push(Capability {
            id,
            name: name.to_string(),
            description: description.to_string(),
            cap_type: CapabilityType::Primitive,
            sub_ids: Vec::new(),
            vsa_vector,
            invocation_count: 0,
            success_rate: 1.0,
        });
        id
    }

    pub fn synthesize(&mut self, request: &str) -> SynthesisOutcome {
        let request_vsa = Self::encode(request);

        let direct = self.find_best_match(&request_vsa, self.min_match_threshold);
        if let Some(id) = direct {
            self.record_hit(id);
            return SynthesisOutcome::DirectMatch(id);
        }

        let composite = self.decompose_and_compose(request, &request_vsa);
        match composite {
            Some(id) => {
                self.synthesized_count += 1;
                SynthesisOutcome::CompositeCreated(id)
            }
            None => SynthesisOutcome::NeedsHuman(
                "no capability matches this request, and not enough primitives to compose one"
                    .to_string(),
            ),
        }
    }

    pub fn find_matches(&self, text: &str, threshold: f64) -> Vec<&Capability> {
        let query = Self::encode(text);
        self.capabilities
            .iter()
            .filter(|c| Self::similarity(&query, &c.vsa_vector) >= threshold)
            .collect()
    }

    pub fn capability(&self, id: u64) -> Option<&Capability> {
        self.capabilities.iter().find(|c| c.id == id)
    }

    pub fn stats(&self) -> CapabilityStats {
        let total = self.capabilities.len();
        let primitives = self
            .capabilities
            .iter()
            .filter(|c| c.cap_type == CapabilityType::Primitive)
            .count();
        let composites = self
            .capabilities
            .iter()
            .filter(|c| c.cap_type == CapabilityType::Composite)
            .count();
        let generated = self
            .capabilities
            .iter()
            .filter(|c| c.cap_type == CapabilityType::Generated)
            .count();
        let pipelines = self
            .capabilities
            .iter()
            .filter(|c| c.cap_type == CapabilityType::Pipeline)
            .count();
        let avg_sr = self
            .capabilities
            .iter()
            .map(|c| c.success_rate)
            .sum::<f64>()
            / total.max(1) as f64;
        CapabilityStats {
            total_capabilities: total,
            primitives,
            composites,
            generated,
            pipelines,
            synthesized_count: self.synthesized_count,
            avg_success_rate: avg_sr,
        }
    }

    pub fn record_invocation(&mut self, id: u64, success: bool) {
        if let Some(cap) = self.capabilities.iter_mut().find(|c| c.id == id) {
            cap.invocation_count += 1;
            let n = cap.invocation_count as f64;
            cap.success_rate = ((n - 1.0) * cap.success_rate + if success { 1.0 } else { 0.0 }) / n;
        }
    }

    /// Prune least-used composite capabilities when over capacity.
    /// Preserves all primitives — only evicts composites with lowest invocation count.
    /// Returns number of capabilities removed.
    pub fn prune(&mut self) -> usize {
        if self.capabilities.len() <= self.max_capabilities {
            return 0;
        }
        let excess = self.capabilities.len() - self.max_capabilities;
        let mut composite_ids: Vec<(u64, u64)> = self
            .capabilities
            .iter()
            .filter(|c| c.cap_type != CapabilityType::Primitive)
            .map(|c| (c.id, c.invocation_count))
            .collect();
        composite_ids.sort_by_key(|&(_, count)| count);
        let remove_ids: std::collections::HashSet<u64> = composite_ids
            .iter()
            .take(excess.min(composite_ids.len()))
            .map(|&(id, _)| id)
            .collect();
        let before = self.capabilities.len();
        self.capabilities.retain(|c| !remove_ids.contains(&c.id));
        before - self.capabilities.len()
    }

    fn find_best_match(&self, query: &[u8], threshold: f64) -> Option<u64> {
        self.capabilities
            .iter()
            .filter(|c| Self::similarity(query, &c.vsa_vector) >= threshold)
            .max_by(|a, b| {
                Self::similarity(query, &a.vsa_vector)
                    .partial_cmp(&Self::similarity(query, &b.vsa_vector))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|c| c.id)
    }

    fn decompose_and_compose(&mut self, request: &str, request_vsa: &[u8]) -> Option<u64> {
        let terms = Self::extract_terms(request);
        if terms.len() < 2 {
            return None;
        }

        let mut matched: Vec<u64> = Vec::new();
        for term in &terms {
            let term_vsa = Self::encode(term);
            let best = self.find_best_match(&term_vsa, self.min_composition_threshold);
            if let Some(id) = best {
                if !matched.contains(&id) {
                    matched.push(id);
                }
            }
        }

        if matched.len() < 2 {
            return None;
        }

        let composite_vsa = self.compose_vsa_from_ids(&matched, request_vsa);
        let is_duplicate = self
            .capabilities
            .iter()
            .any(|c| Self::similarity(&c.vsa_vector, &composite_vsa) >= 0.90);
        if is_duplicate {
            return self.find_best_match(request_vsa, self.min_match_threshold);
        }

        let id = self.next_id;
        self.next_id += 1;
        let name = format!("composite_{}", id);
        let description = format!(
            "composite capability composed from {} primitives to handle: {}",
            matched.len(),
            request.chars().take(80).collect::<String>(),
        );
        self.capabilities.push(Capability {
            id,
            name,
            description,
            cap_type: CapabilityType::Composite,
            sub_ids: matched.clone(),
            vsa_vector: composite_vsa,
            invocation_count: 0,
            success_rate: 0.5,
        });
        Some(id)
    }

    fn compose_vsa_from_ids(&self, ids: &[u64], request_vsa: &[u8]) -> Vec<u8> {
        let mut vectors: Vec<&[u8]> = ids
            .iter()
            .filter_map(|id| self.capability(*id))
            .map(|c| c.vsa_vector.as_slice())
            .collect();
        vectors.push(request_vsa);
        QuantizedVSA::bundle(&vectors)
    }

    pub(crate) fn encode(text: &str) -> Vec<u8> {
        let seed: u64 = text
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        QuantizedVSA::seeded_random(seed, VSA_DIM)
    }

    pub(crate) fn similarity(a: &[u8], b: &[u8]) -> f64 {
        QuantizedVSA::cosine(a, b)
    }

    fn extract_terms(text: &str) -> Vec<String> {
        let stop_words = [
            "the", "a", "an", "this", "that", "to", "for", "of", "in", "on", "with", "and", "or",
            "is", "are", "was", "were", "be", "been", "being", "have", "has", "had", "do", "does",
            "did", "will", "would", "could", "should", "may", "might", "i", "you", "he", "she",
            "it", "we", "they", "me", "my", "your", "his", "her", "its", "our", "their", "please",
            "can", "need", "want", "help", "make", "do", "get", "use", "create", "build", "find",
            "tell", "show", "give", "take",
        ];
        let mut terms: Vec<String> = Vec::new();
        let lower = text.to_lowercase();
        for word in lower.split_whitespace() {
            let cleaned: String = word.chars().filter(|c| c.is_alphanumeric()).collect();
            if cleaned.len() >= 3 && !stop_words.contains(&cleaned.as_str()) {
                terms.push(cleaned);
            }
        }
        terms.sort();
        terms.dedup();
        terms
    }

    fn record_hit(&mut self, id: u64) {
        if let Some(cap) = self.capabilities.iter_mut().find(|c| c.id == id) {
            cap.invocation_count += 1;
        }
    }

    pub fn synthesize_pipeline(
        &mut self,
        input: &str,
        pipeline: &PipelineCapability,
    ) -> SynthesisOutcome {
        let request_vsa = Self::encode(input);
        let pipe_sim = Self::similarity(&request_vsa, &pipeline.vsa_vector);

        if pipe_sim >= self.min_match_threshold {
            // Pipeline matches directly — register it as a pipeline-type capability
            let id = self.next_id;
            self.next_id += 1;
            self.capabilities.push(Capability {
                id,
                name: pipeline.name.clone(),
                description: format!(
                    "pipeline capability with {} stages: {}",
                    pipeline.stages.len(),
                    pipeline
                        .stages
                        .iter()
                        .map(|s| s.name.clone())
                        .collect::<Vec<_>>()
                        .join(" → "),
                ),
                cap_type: CapabilityType::Pipeline,
                sub_ids: Vec::new(),
                vsa_vector: pipeline.vsa_vector.clone(),
                invocation_count: 0,
                success_rate: 1.0,
            });
            self.synthesized_count += 1;
            return SynthesisOutcome::CompositeCreated(id);
        }

        // Fall through to normal synthesis
        self.synthesize(input)
    }

    pub fn register_pipeline_as_composite(&mut self, pipeline: &PipelineCapability) -> u64 {
        let sub_ids: Vec<u64> = pipeline
            .stages
            .iter()
            .filter_map(|stage| {
                let stage_vsa = Self::encode(&stage.handler);
                self.find_best_match(&stage_vsa, self.min_composition_threshold)
            })
            .collect();

        if sub_ids.is_empty() {
            let id = self.next_id;
            self.next_id += 1;
            self.capabilities.push(Capability {
                id,
                name: pipeline.name.clone(),
                description: format!(
                    "pipeline composite: {} (no stage primitives matched)",
                    pipeline.name
                ),
                cap_type: CapabilityType::Composite,
                sub_ids: Vec::new(),
                vsa_vector: pipeline.vsa_vector.clone(),
                invocation_count: 0,
                success_rate: 0.5,
            });
            return id;
        }

        let composite_vsa = QuantizedVSA::bundle(
            &sub_ids
                .iter()
                .filter_map(|id| self.capability(*id))
                .map(|c| c.vsa_vector.as_slice())
                .collect::<Vec<_>>(),
        );

        let id = self.next_id;
        self.next_id += 1;
        self.capabilities.push(Capability {
            id,
            name: format!("pipe_composite_{}", id),
            description: format!(
                "composite capability from pipeline '{}' with {} matched stages",
                pipeline.name,
                sub_ids.len(),
            ),
            cap_type: CapabilityType::Composite,
            sub_ids,
            vsa_vector: composite_vsa,
            invocation_count: 0,
            success_rate: 0.5,
        });
        id
    }

    /// Export all capabilities as SKILL.md format string
    pub fn export_skills_markdown(&self) -> String {
        let mut out = String::new();
        for cap in &self.capabilities {
            out.push_str(&Self::format_skill_md(cap));
            out.push('\n');
        }
        out
    }

    /// Export skills to a directory as individual SKILL.md files (atomic write)
    pub fn export_skills_to_dir(&self, dir: &std::path::Path) -> std::io::Result<usize> {
        std::fs::create_dir_all(dir)?;
        let mut count = 0;
        for cap in &self.capabilities {
            let content = Self::format_skill_md(cap);
            let file_name = format!("skill_{}.md", cap.id);
            let tmp_name = format!("skill_{}.md.tmp", cap.id);
            std::fs::write(dir.join(&tmp_name), &content)?;
            std::fs::rename(dir.join(&tmp_name), dir.join(&file_name))?;
            count += 1;
        }
        Ok(count)
    }

    /// Format a single capability as SKILL.md
    fn format_skill_md(cap: &Capability) -> String {
        let vsa_preview = cap
            .vsa_vector
            .iter()
            .take(16)
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(" ");
        format!(
            "# {}\n\n## Description\n{}\n\n## Skill Type\n{:?}\n\n## Capability Vector\n`{}`\n\n## Invocation Count\n{}\n\n## Success Rate\n{:.2}\n",
            cap.name, cap.description, cap.cap_type, vsa_preview, cap.invocation_count, cap.success_rate
        )
    }

    /// Register multiple primitives at once, returns count of successful registrations
    pub fn register_primitives_batch(&mut self, primitives: &[(&str, &str)]) -> usize {
        let mut count = 0;
        for (name, desc) in primitives {
            self.register_primitive(name, desc);
            count += 1;
        }
        count
    }

    /// Mines thought history traces for recurring patterns, registers composite capabilities.
    /// Returns (n_clusters, n_registered) — clusters found and new capabilities registered.
    pub fn mine_traces(
        &mut self,
        thought_history: &[(String, Vec<u8>, f64)],
        min_cluster_size: usize,
    ) -> (usize, usize) {
        if thought_history.len() < min_cluster_size {
            return (0, 0);
        }

        let threshold = 0.75;
        let duplicate_threshold = 0.90;

        // Sort by the f64 value descending (most recent first) for seeding priority
        let mut sorted: Vec<&(String, Vec<u8>, f64)> = thought_history.iter().collect();
        sorted.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        let mut clustered: Vec<bool> = vec![false; sorted.len()];
        let mut n_clusters = 0usize;
        let mut n_registered = 0usize;

        for i in 0..sorted.len() {
            if clustered[i] {
                continue;
            }

            // Seed a new cluster with the highest-confidence (most recent) unclustered item
            let mut cluster: Vec<usize> = vec![i];
            clustered[i] = true;

            // Greedy: absorb all neighbors within similarity threshold
            for j in (i + 1)..sorted.len() {
                if clustered[j] {
                    continue;
                }
                let sim = Self::similarity(&sorted[i].1, &sorted[j].1);
                if sim >= threshold {
                    cluster.push(j);
                    clustered[j] = true;
                }
            }

            if cluster.len() < min_cluster_size {
                continue;
            }
            n_clusters += 1;

            // Compute centroid by bundling all cluster member VSA vectors
            let centroid_vsas: Vec<&[u8]> = cluster
                .iter()
                .map(|&idx| sorted[idx].1.as_slice())
                .collect();
            let centroid = QuantizedVSA::bundle(&centroid_vsas);

            // Skip if centroid is too similar to an existing capability (duplicate)
            let is_duplicate = self
                .capabilities
                .iter()
                .any(|c| Self::similarity(&c.vsa_vector, &centroid) >= duplicate_threshold);
            if is_duplicate {
                continue;
            }

            // Extract pattern name from most common keywords across cluster members
            let mut word_freq: HashMap<String, usize> = HashMap::new();
            for &idx in &cluster {
                let terms = Self::extract_terms(&sorted[idx].0);
                for term in terms {
                    *word_freq.entry(term).or_insert(0) += 1;
                }
            }
            let mut freq: Vec<(String, usize)> = word_freq.into_iter().collect();
            freq.sort_by(|a, b| b.1.cmp(&a.1));
            let pattern_name = freq
                .iter()
                .take(3)
                .map(|(w, _)| w.clone())
                .collect::<Vec<_>>()
                .join("_");
            let name = if pattern_name.is_empty() {
                format!("mined_cluster_{}", n_clusters)
            } else {
                format!("mined_{}", pattern_name)
            };

            // Link sub-IDs from best-matching primitives for each cluster member text
            let sub_ids: Vec<u64> = cluster
                .iter()
                .filter_map(|&idx| {
                    let text_vsa = Self::encode(&sorted[idx].0);
                    self.find_best_match(&text_vsa, self.min_composition_threshold)
                        .and_then(|id| {
                            if self
                                .capabilities
                                .iter()
                                .any(|c| c.id == id && c.cap_type == CapabilityType::Primitive)
                            {
                                Some(id)
                            } else {
                                None
                            }
                        })
                })
                .collect::<Vec<_>>();
            let mut sub_ids = sub_ids;
            sub_ids.sort();
            sub_ids.dedup();

            let id = self.next_id;
            self.next_id += 1;
            let desc = format!(
                "trace-mined composite from {} thoughts: cluster of {}",
                cluster.len(),
                name
            );
            self.capabilities.push(Capability {
                id,
                name,
                description: desc,
                cap_type: CapabilityType::Composite,
                sub_ids,
                vsa_vector: centroid,
                invocation_count: 1,
                success_rate: 0.5,
            });
            n_registered += 1;

            // Prune if over capacity
            if self.capabilities.len() > self.max_capabilities {
                self.prune();
            }
        }

        (n_clusters, n_registered)
    }

    /// Record execution feedback for a capability and optionally trigger trace
    /// mining from thought history. This bridges CapabilitySynthesizer ↔ MemoryLattice.
    pub fn record_execution_feedback(
        &mut self,
        cap_id: u64,
        success: bool,
        thought_history: &[(String, Vec<u8>, f64)],
        min_trace_cluster: usize,
    ) -> (usize, usize) {
        if let Some(cap) = self.capabilities.iter_mut().find(|c| c.id == cap_id) {
            cap.invocation_count += 1;
            let n = cap.invocation_count as f64;
            cap.success_rate = ((n - 1.0) * cap.success_rate + if success { 1.0 } else { 0.0 }) / n;
        }
        if thought_history.len() >= min_trace_cluster {
            self.mine_traces(thought_history, min_trace_cluster)
        } else {
            (0, 0)
        }
    }

    /// Absorb insights from cross-model distillation into capability registry.
    /// Converts high-confidence demonstrated capabilities and behavioral patterns
    /// into registered primitives. Returns count of new capabilities registered.
    pub fn absorb_distillation_report(&mut self, report: &DistillationReport) -> usize {
        let mut count = 0usize;

        // Register demonstrated capabilities with proficiency > 0.55
        for cap in &report.capabilities {
            if cap.proficiency > 0.55 {
                let desc_text = format!("{}: {}", cap.name, cap.description);
                let vsa = Self::encode(&desc_text);
                let already = self.find_best_match(&vsa, 0.55).is_some();
                if !already {
                    let id = self.next_id;
                    self.next_id += 1;
                    self.capabilities.push(Capability {
                        id,
                        name: cap.name.clone(),
                        description: cap.description.clone(),
                        cap_type: CapabilityType::Generated,
                        sub_ids: Vec::new(),
                        vsa_vector: vsa,
                        invocation_count: cap.observation_count as u64,
                        success_rate: cap.proficiency,
                    });
                    count += 1;
                }
            }
        }

        // Register behavioral patterns with avg_outcome > 0.7
        for pat in &report.behavioral_patterns {
            if pat.avg_outcome > 0.7 && pat.confidence > 0.6 {
                let pat_name = format!("pattern:{}", pat.topic);
                let desc_text = format!("{}: {}", pat.topic, pat.description);
                let vsa = Self::encode(&desc_text);
                let already = self.find_best_match(&vsa, 0.55).is_some();
                if !already {
                    let id = self.next_id;
                    self.next_id += 1;
                    self.capabilities.push(Capability {
                        id,
                        name: pat_name,
                        description: pat.description.clone(),
                        cap_type: CapabilityType::Generated,
                        sub_ids: Vec::new(),
                        vsa_vector: vsa,
                        invocation_count: pat.observation_count as u64,
                        success_rate: pat.avg_outcome,
                    });
                    count += 1;
                }
            }
        }

        if count > 0 {
            log::debug!(
                "absorb_distillation: registered {} new capabilities from distillation",
                count
            );
        }

        count
    }
}

impl Default for CapabilitySynthesizer {
    fn default() -> Self {
        Self::new()
    }
}

// ── Leap 4: KnowledgePackage / cross-instance merge ──

/// Serializable snapshot of a MemoryLattice for knowledge transfer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatticeSnapshot {
    pub skills: Vec<(String, Vec<u8>, f64)>,
    pub meta_rules: Vec<(String, Vec<u8>, f64)>,
}

/// A knowledge package for cross-instance capability & lattice transfer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgePackage {
    pub version: u8,
    pub instance_id: String,
    pub domain: String,
    pub capabilities: Vec<Capability>,
    pub lattice_snapshot: LatticeSnapshot,
}

/// Merge a remote KnowledgePackage into a local CapabilitySynthesizer.
/// Uses three strategies: TrustSource (direct import if absent), VsaBundle (fuse
/// if both have high success rate), ArenaResolve (keep local if better).
/// Returns (imported, replaced, bundled) counts.
pub fn merge_knowledge_package(
    local_caps: &mut CapabilitySynthesizer,
    remote: &KnowledgePackage,
) -> (usize, usize, usize) {
    let mut imported = 0usize;
    let mut replaced = 0usize;
    let mut bundled = 0usize;

    for rc in &remote.capabilities {
        let best = local_caps.find_best_match(&rc.vsa_vector, 0.55);
        match best {
            None => {
                local_caps.next_id += 1;
                local_caps.capabilities.push(rc.clone());
                imported += 1;
            }
            Some(local_id) => {
                let local_idx = local_caps
                    .capabilities
                    .iter()
                    .position(|c| c.id == local_id);
                if let Some(idx) = local_idx {
                    // Clone before mutation to avoid borrow conflict
                    let lc_sr = local_caps.capabilities[idx].success_rate;
                    let lc_vsa = local_caps.capabilities[idx].vsa_vector.clone();
                    if rc.success_rate > lc_sr * 1.2 {
                        let loc = &mut local_caps.capabilities[idx];
                        loc.success_rate = rc.success_rate;
                        loc.invocation_count = rc.invocation_count.max(loc.invocation_count);
                        replaced += 1;
                    } else if rc.success_rate > 0.8 && lc_sr > 0.8 {
                        let bundled_vsa =
                            crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA::bundle(&[
                                lc_vsa.as_slice(),
                                rc.vsa_vector.as_slice(),
                            ]);
                        let loc = &mut local_caps.capabilities[idx];
                        loc.vsa_vector = bundled_vsa;
                        loc.success_rate = (lc_sr + rc.success_rate) / 2.0;
                        bundled += 1;
                    }
                }
            }
        }
    }

    // ── Merge LatticeSnapshot skills/meta_rules into local synthesizer as primitives ──
    let mut skill_imported = 0usize;
    let mut meta_imported = 0usize;
    for (name, vsa, conf) in &remote.lattice_snapshot.skills {
        if *conf > 0.6 {
            let already = local_caps.find_best_match(vsa, 0.55).is_some();
            if !already {
                let desc = format!("imported_skill:{}", name);
                let id = local_caps.next_id;
                local_caps.next_id += 1;
                local_caps.capabilities.push(Capability {
                    id,
                    name: name.clone(),
                    description: desc,
                    cap_type: CapabilityType::Primitive,
                    sub_ids: Vec::new(),
                    vsa_vector: vsa.clone(),
                    invocation_count: 1,
                    success_rate: *conf,
                });
                skill_imported += 1;
            }
        }
    }
    for (name, vsa, conf) in &remote.lattice_snapshot.meta_rules {
        if *conf > 0.7 {
            let already = local_caps.find_best_match(vsa, 0.55).is_some();
            if !already {
                let desc = format!("imported_meta:{}", name);
                let id = local_caps.next_id;
                local_caps.next_id += 1;
                local_caps.capabilities.push(Capability {
                    id,
                    name: name.clone(),
                    description: desc,
                    cap_type: CapabilityType::Generated,
                    sub_ids: Vec::new(),
                    vsa_vector: vsa.clone(),
                    invocation_count: 1,
                    success_rate: *conf,
                });
                meta_imported += 1;
            }
        }
    }
    if skill_imported > 0 || meta_imported > 0 {
        log::info!(
            "merge_knowledge_package: imported {} skills + {} meta-rules from lattice snapshot",
            skill_imported,
            meta_imported,
        );
    }

    (imported, replaced, bundled)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── trace mining tests ──

    #[test]
    fn test_mine_traces_empty() {
        let mut cs = CapabilitySynthesizer::new();
        cs.register_primitive("search", "search capability");
        let history: Vec<(String, Vec<u8>, f64)> = vec![];
        let (n_clusters, n_registered) = cs.mine_traces(&history, 3);
        assert_eq!(n_clusters, 0);
        assert_eq!(n_registered, 0);
    }

    #[test]
    fn test_mine_traces_single_cluster() {
        let mut cs = CapabilitySynthesizer::new();
        cs.register_primitive("search", "search capability");
        cs.register_primitive("web", "web search primitive");

        // 3 identical vectors form a cluster (same seed = identical VSA)
        let v1 = QuantizedVSA::seeded_random(42, VSA_DIM);
        let v2 = QuantizedVSA::seeded_random(42, VSA_DIM);
        let v3 = QuantizedVSA::seeded_random(42, VSA_DIM);

        let history = vec![
            ("search the web".to_string(), v1, 100.0),
            ("search internet".to_string(), v2, 101.0),
            ("search online".to_string(), v3, 102.0),
        ];

        let (n_clusters, n_registered) = cs.mine_traces(&history, 3);
        assert_eq!(
            n_clusters, 1,
            "should find 1 cluster of 3 identical vectors"
        );
        assert_eq!(n_registered, 1, "should register 1 composite capability");

        let stats = cs.stats();
        assert!(stats.composites >= 1, "composites should increase");
    }

    #[test]
    fn test_mine_traces_duplicate_skipped() {
        let mut cs = CapabilitySynthesizer::new();
        cs.register_primitive("search", "search capability");

        // Pre-register a capability whose VSA matches the cluster centroid
        let v_dup = QuantizedVSA::seeded_random(42, VSA_DIM);
        cs.capabilities.push(Capability {
            id: 999,
            name: "existing_search".to_string(),
            description: "preexisting search composite".to_string(),
            cap_type: CapabilityType::Composite,
            sub_ids: vec![],
            vsa_vector: v_dup.clone(),
            invocation_count: 0,
            success_rate: 0.5,
        });
        cs.next_id = 1000;

        let v2 = QuantizedVSA::seeded_random(42, VSA_DIM);
        let v3 = QuantizedVSA::seeded_random(42, VSA_DIM);

        let history = vec![
            ("search the web".to_string(), v_dup, 100.0),
            ("search the web".to_string(), v2, 101.0),
            ("search the web".to_string(), v3, 102.0),
        ];

        let (n_clusters, n_registered) = cs.mine_traces(&history, 3);
        assert_eq!(
            n_clusters, 0,
            "should not count clusters when centroid is duplicate"
        );
        assert_eq!(n_registered, 0, "should not register duplicate capability");
    }

    #[test]
    fn test_mine_traces_below_threshold() {
        let mut cs = CapabilitySynthesizer::new();
        cs.register_primitive("search", "search capability");

        // Only 2 items, below min_cluster_size=3
        let v1 = QuantizedVSA::seeded_random(42, VSA_DIM);
        let v2 = QuantizedVSA::seeded_random(42, VSA_DIM);

        let history = vec![
            ("search the web".to_string(), v1, 100.0),
            ("search internet".to_string(), v2, 101.0),
        ];

        let (n_clusters, n_registered) = cs.mine_traces(&history, 3);
        assert_eq!(n_clusters, 0, "2 items < min_cluster_size=3");
        assert_eq!(n_registered, 0);
    }

    // ── export / SKILL.md tests ──

    #[test]
    fn test_export_skills_markdown_non_empty() {
        let mut cs = CapabilitySynthesizer::new();
        cs.register_primitive("test_cap", "a test capability");
        let md = cs.export_skills_markdown();
        assert!(
            !md.is_empty(),
            "export should produce non-empty output with registered primitives"
        );
        assert!(md.contains("test_cap"));
        assert!(md.contains("a test capability"));
    }

    #[test]
    fn test_format_skill_md_contains_fields() {
        let cap = Capability {
            id: 42,
            name: "search".to_string(),
            description: "search the web".to_string(),
            cap_type: CapabilityType::Primitive,
            sub_ids: Vec::new(),
            vsa_vector: vec![0xab; 64],
            invocation_count: 7,
            success_rate: 0.85,
        };
        let md = CapabilitySynthesizer::format_skill_md(&cap);
        assert!(md.contains("search"));
        assert!(md.contains("search the web"));
        assert!(md.contains("Primitive"));
        assert!(md.contains("ab")); // hex preview
        assert!(md.contains("7")); // invocation count
        assert!(md.contains("0.85")); // success rate
    }

    #[test]
    fn test_register_primitives_batch_count() {
        let mut cs = CapabilitySynthesizer::new();
        let primitives = [("a", "cap a"), ("b", "cap b"), ("c", "cap c")];
        let count = cs.register_primitives_batch(&primitives);
        assert_eq!(count, 3);
        assert_eq!(cs.capabilities.len(), 3);
    }

    #[test]
    fn test_export_skills_to_dir_creates_files() {
        let mut cs = CapabilitySynthesizer::new();
        cs.register_primitive("alpha", "first capability");
        cs.register_primitive("beta", "second capability");
        let dir = std::env::temp_dir()
            .join("neotrix_skill_test")
            .join("skills");
        let _ = std::fs::remove_dir_all(&dir);
        let count = cs.export_skills_to_dir(&dir).unwrap();
        assert_eq!(count, 2);
        assert!(dir.join("skill_1.md").exists());
        assert!(dir.join("skill_2.md").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_register_primitives_batch_empty() {
        let mut cs = CapabilitySynthesizer::new();
        let count = cs.register_primitives_batch(&[]);
        assert_eq!(count, 0);
    }
}
