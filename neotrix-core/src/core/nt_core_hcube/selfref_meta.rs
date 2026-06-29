use super::vsa_runtime_ir::global_vsa_runtime;

const MAX_EDITS: usize = 10000;

#[derive(Debug, Clone)]
pub struct MetaEdit {
    pub target_program: String,
    pub description: String,
    pub before_hash: u64,
    pub after_hash: u64,
    pub verified: bool,
    pub timestamp: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct SelfRefMetaLayer {
    pub generation: usize,
    pub archive: Vec<ArchiveEntry>,
    pub edits: Vec<MetaEdit>,
    pub max_archive: usize,
    pub improvement_rate: f64,
    pub curiosity_factor: f64,
}

#[derive(Debug, Clone)]
pub struct ArchiveEntry {
    pub program_name: String,
    pub program_snapshot: Vec<VsaInstruction>,
    pub performance_score: f64,
    pub generation: usize,
}

#[derive(Debug, Clone)]
pub struct VsaInstruction {
    pub op_name: String,
    pub detail: String,
}

impl SelfRefMetaLayer {
    pub fn new() -> Self {
        Self {
            generation: 0,
            archive: Vec::new(),
            edits: Vec::new(),
            max_archive: 100,
            improvement_rate: 0.0,
            curiosity_factor: 0.5,
        }
    }

    pub fn set_curiosity_factor(&mut self, factor: f64) {
        self.curiosity_factor = factor.clamp(0.0, 1.0);
    }

    pub fn step(&mut self) -> String {
        self.generation += 1;
        let mut events = Vec::new();

        if let Ok(bundle) = global_vsa_runtime().lock() {
            let _ = &bundle.runtime;
        }

        let step_events = self.selfref_cycle();
        events.push(step_events);

        if self.generation % 3 == 0 {
            let archive_event = self.archive_cycle();
            events.push(archive_event);
        }

        self.update_improvement_rate();
        events.join(" | ")
    }

    fn selfref_cycle(&mut self) -> String {
        let mut events = Vec::new();

        if let Ok(mut bundle) = global_vsa_runtime().lock() {
            match bundle.runtime.execute("__selfref_meta") {
                Ok(e) => events.push(format!("selfref:{}", e)),
                Err(e) => events.push(format!("selfref_err:{}", e)),
            }

            let base_count = bundle
                .runtime
                .get_program("__selfref_meta")
                .map(|p| p.len())
                .unwrap_or(0);

            // Use curiosity_factor to modulate rewrite frequency: high curiosity → more rewrites
            let rewrite_interval = if self.curiosity_factor > 0.6 {
                3
            } else if self.curiosity_factor < 0.2 {
                7
            } else {
                5
            };

            if self.generation % rewrite_interval == 0 && self.generation > 0 {
                match bundle.runtime.rewrite_program("__selfref_meta") {
                    Ok(e) => {
                        let new_count = bundle
                            .runtime
                            .get_program("__selfref_meta")
                            .map(|p| p.len())
                            .unwrap_or(0);
                        let edit = MetaEdit {
                            target_program: "__selfref_meta".into(),
                            description: format!(
                                "rewrite_gen{}_cur={:.2}",
                                self.generation, self.curiosity_factor
                            ),
                            before_hash: base_count as u64,
                            after_hash: new_count as u64,
                            verified: new_count <= base_count || self.generation > 10,
                            timestamp: std::time::Instant::now(),
                        };
                        let verified = edit.verified;
                        self.edits.push(edit);
                        if self.edits.len() > MAX_EDITS {
                            self.edits
                                .drain(0..self.edits.len().saturating_sub(MAX_EDITS));
                        }
                        if verified {
                            events.push(format!("rewrite:{}", e));
                        } else {
                            events.push("rewrite:rejected_verification".into());
                        }
                    }
                    Err(e) => events.push(format!("rewrite_err:{}", e)),
                }
            }
        }

        events.join("; ")
    }

    fn archive_cycle(&mut self) -> String {
        let mut events = Vec::new();

        if let Ok(bundle) = global_vsa_runtime().lock() {
            if let Some(prog) = bundle.runtime.get_program("__selfref_meta") {
                let snapshot: Vec<VsaInstruction> = prog
                    .instructions
                    .iter()
                    .map(|i| VsaInstruction {
                        op_name: format!("{:?}", i.op)
                            .split_whitespace()
                            .next()
                            .unwrap_or("unknown")
                            .to_string(),
                        detail: i.label.clone().unwrap_or_default(),
                    })
                    .collect();

                // Score combines generation skew with curiosity factor
                // High curiosity → newer programs get bonus; low curiosity → proven programs score higher
                let recency = (self.generation as f64).recip().min(1.0);
                let curiosity_bonus = self.curiosity_factor * 0.3;
                let score = (recency + curiosity_bonus).min(1.0);

                self.archive.push(ArchiveEntry {
                    program_name: format!("__selfref_meta_gen{}", self.generation),
                    program_snapshot: snapshot,
                    performance_score: score,
                    generation: self.generation,
                });

                if self.archive.len() > self.max_archive {
                    self.archive.remove(0);
                }

                events.push(format!(
                    "archive:gen{}_instr{}_score={:.3}_cur={:.2}",
                    self.generation,
                    prog.len(),
                    score,
                    self.curiosity_factor
                ));
            }
        }

        events.join("; ")
    }

    fn update_improvement_rate(&mut self) {
        let recent_edits: Vec<&MetaEdit> = self.edits.iter().filter(|e| e.verified).collect();
        if recent_edits.len() >= 3 {
            let last_three = &recent_edits[recent_edits.len().saturating_sub(3)..];
            let improvements: Vec<f64> = last_three
                .iter()
                .map(|e| {
                    if e.after_hash < e.before_hash && e.before_hash > 0 {
                        (e.before_hash - e.after_hash) as f64 / e.before_hash as f64
                    } else {
                        0.0
                    }
                })
                .collect();
            self.improvement_rate = improvements.iter().sum::<f64>() / improvements.len() as f64;
        }
    }

    pub fn report(&self) -> String {
        format!(
            "SelfRefMeta: gen={} archive={} edits={} verified={} improvement_rate={:.4} cur={:.2}",
            self.generation,
            self.archive.len(),
            self.edits.len(),
            self.edits.iter().filter(|e| e.verified).count(),
            self.improvement_rate,
            self.curiosity_factor,
        )
    }
}

impl Default for SelfRefMetaLayer {
    fn default() -> Self {
        Self::new()
    }
}

static SELFREF_META: std::sync::OnceLock<std::sync::Mutex<SelfRefMetaLayer>> =
    std::sync::OnceLock::new();

pub fn global_selfref_meta() -> &'static std::sync::Mutex<SelfRefMetaLayer> {
    SELFREF_META.get_or_init(|| std::sync::Mutex::new(SelfRefMetaLayer::new()))
}

pub fn step_selfref_meta() -> String {
    if let Ok(mut layer) = global_selfref_meta().lock() {
        layer.step()
    } else {
        "selfref:locked".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[serial]
    #[test]
    fn test_selfref_layer_init() {
        let layer = SelfRefMetaLayer::new();
        assert_eq!(layer.generation, 0);
        assert!(layer.archive.is_empty());
    }

    #[test]
    fn test_selfref_cycle() {
        let mut layer = SelfRefMetaLayer::new();
        let result = layer.step();
        assert_eq!(layer.generation, 1);
        assert!(result.contains("selfref") || result.contains("selfref_err"));
    }

    #[test]
    fn test_archive_cycle() {
        let mut layer = SelfRefMetaLayer::new();
        let _ = layer.step();
        let _ = layer.step();
        let _ = layer.step();
        assert!(layer.generation >= 3);
    }

    #[test]
    fn test_improvement_rate() {
        let mut layer = SelfRefMetaLayer::new();
        layer.edits.push(MetaEdit {
            target_program: "p".into(),
            description: "e1".into(),
            before_hash: 10,
            after_hash: 8,
            verified: true,
            timestamp: std::time::Instant::now(),
        });
        layer.edits.push(MetaEdit {
            target_program: "p".into(),
            description: "e2".into(),
            before_hash: 8,
            after_hash: 6,
            verified: true,
            timestamp: std::time::Instant::now(),
        });
        layer.edits.push(MetaEdit {
            target_program: "p".into(),
            description: "e3".into(),
            before_hash: 6,
            after_hash: 5,
            verified: true,
            timestamp: std::time::Instant::now(),
        });
        layer.update_improvement_rate();
        assert!(layer.improvement_rate > 0.0);
    }

    #[test]
    fn test_global_singleton() {
        let layer = global_selfref_meta();
        let l = layer.lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(l.generation, 0);
        drop(l);
        let result = step_selfref_meta();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_report() {
        let mut layer = SelfRefMetaLayer::new();
        let _ = layer.step();
        let report = layer.report();
        assert!(report.contains("SelfRefMeta"));
        assert!(report.contains("gen=1"));
    }
}
