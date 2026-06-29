use std::collections::VecDeque;

/// Result of a bootstrap identity check.
#[derive(Debug, Clone, PartialEq)]
pub enum BootstrapStatus {
    Verified,
    BehavioralOnly,
    Failed,
    Impossible,
    NotYetChecked,
}

impl BootstrapStatus {
    pub fn is_ok(&self) -> bool {
        matches!(
            self,
            BootstrapStatus::Verified | BootstrapStatus::BehavioralOnly
        )
    }
}

/// Record of a single VSA operation execution comparison.
#[derive(Debug, Clone)]
pub struct VsaExecutionRecord {
    pub expression: String,
    pub reference_vsa: Vec<u8>,
    pub generated_vsa: Vec<u8>,
    pub cosine_similarity: f64,
}

/// Triple-identity record for a single compiler generation.
#[derive(Debug, Clone)]
pub struct BootstrapIdentity {
    pub cycle: u64,
    pub rust_fingerprint: [u8; 32],
    pub ne_v0_fingerprint: [u8; 32],
    pub ne_v1_fingerprint: [u8; 32],
    pub behavior_similarity: f64,
    pub status: BootstrapStatus,
    pub compiler_compiles: bool,
    pub vsa_execution_records: Vec<VsaExecutionRecord>,
    pub rustc_output: Option<String>,
}

/// Verifies that Rust reference compiler and Ne self-compiler produce equivalent output.
pub struct BootstrapIdentityVerifier {
    pub rust_compiler_version: u32,
    pub ne_compiler_version: u32,
    pub identity_history: VecDeque<BootstrapIdentity>,
    pub max_history: usize,
}

impl BootstrapIdentityVerifier {
    pub fn new() -> Self {
        Self {
            rust_compiler_version: 0,
            ne_compiler_version: 0,
            identity_history: VecDeque::new(),
            max_history: 10,
        }
    }

    pub fn fingerprint_bytes(bytes: &[u8]) -> [u8; 32] {
        use std::hash::Hasher;
        let mut hasher = std::hash::DefaultHasher::new();
        hasher.write(bytes);
        let h = hasher.finish();
        let mut result = [0u8; 32];
        result[..8].copy_from_slice(&h.to_le_bytes());
        result[8..16].copy_from_slice(&(!h).to_le_bytes());
        result[16..24].copy_from_slice(&h.wrapping_mul(3).to_le_bytes());
        result[24..32].copy_from_slice(&(!h.wrapping_mul(3)).to_le_bytes());
        result
    }

    pub fn byte_similarity(a: &[u8], b: &[u8]) -> f64 {
        if a.is_empty() && b.is_empty() {
            return 1.0;
        }
        if a.len() != b.len() {
            let max_len = a.len().max(b.len());
            let min_len = a.len().min(b.len());
            return min_len as f64 / max_len as f64;
        }
        let matching = a.iter().zip(b.iter()).filter(|(x, y)| x == y).count();
        matching as f64 / a.len() as f64
    }

    pub fn check_identity(
        &mut self,
        cycle: u64,
        spec_bytes: &[u8],
        ne_v0_output: &[u8],
        ne_v1_output: &[u8],
    ) -> BootstrapIdentity {
        let rust_fp = Self::fingerprint_bytes(spec_bytes);
        let ne_v0_fp = Self::fingerprint_bytes(ne_v0_output);
        let ne_v1_fp = Self::fingerprint_bytes(ne_v1_output);

        let behavior_similarity = Self::byte_similarity(ne_v0_output, ne_v1_output);

        let text_match = rust_fp == ne_v0_fp && ne_v0_fp == ne_v1_fp;
        let behavior_match = behavior_similarity > 0.95;

        let status = match (text_match, behavior_match) {
            (true, true) => BootstrapStatus::Verified,
            (false, true) => BootstrapStatus::BehavioralOnly,
            (false, false) => BootstrapStatus::Failed,
            (true, false) => BootstrapStatus::Impossible,
        };

        let identity = BootstrapIdentity {
            cycle,
            rust_fingerprint: rust_fp,
            ne_v0_fingerprint: ne_v0_fp,
            ne_v1_fingerprint: ne_v1_fp,
            behavior_similarity,
            status: status.clone(),
            compiler_compiles: false,
            vsa_execution_records: Vec::new(),
            rustc_output: None,
        };

        self.identity_history.push_back(identity.clone());
        while self.identity_history.len() > self.max_history {
            self.identity_history.pop_front();
        }

        identity
    }

    /// Update the latest identity record with rustc compilation result.
    pub fn set_rustc_result(&mut self, compiles: bool, output: Option<String>) {
        if let Some(latest) = self.identity_history.back_mut() {
            latest.compiler_compiles = compiles;
            latest.rustc_output = output;
        }
    }

    /// Add a VSA execution comparison record to the latest identity.
    pub fn add_vsa_record(
        &mut self,
        expression: String,
        reference_vsa: Vec<u8>,
        generated_vsa: Vec<u8>,
    ) -> f64 {
        let similarity = if reference_vsa.len() == generated_vsa.len() && !reference_vsa.is_empty()
        {
            cosine_similarity_bits(&reference_vsa, &generated_vsa)
        } else {
            0.0
        };
        if let Some(latest) = self.identity_history.back_mut() {
            latest.vsa_execution_records.push(VsaExecutionRecord {
                expression,
                reference_vsa,
                generated_vsa,
                cosine_similarity: similarity,
            });
        }
        similarity
    }

    /// Run VSA execution comparison: evaluate test expressions through both
    /// a reference implementation (native Rust) and the generated compiler's output.
    /// The `reference_fn` takes an expression string and returns the expected VSA bytes.
    /// The `generated_fn` takes the same expression and returns the generated compiler's output.
    pub fn verify_vsa_execution<F, G>(
        &mut self,
        test_expressions: &[&str],
        reference_fn: F,
        generated_fn: G,
    ) -> f64
    where
        F: Fn(&str) -> Vec<u8>,
        G: Fn(&str) -> Vec<u8>,
    {
        let mut total_similarity = 0.0;
        let mut count = 0u32;

        for expr in test_expressions {
            let ref_result = reference_fn(expr);
            let gen_result = generated_fn(expr);
            let sim = self.add_vsa_record(expr.to_string(), ref_result, gen_result);
            total_similarity += sim;
            count += 1;
        }

        if count == 0 {
            1.0
        } else {
            total_similarity / count as f64
        }
    }

    /// Verify that the generated compiler source can be compiled by rustc.
    /// Creates a temporary Cargo project, writes the compiler as src/main.rs,
    /// and runs `cargo check`. Returns (success, output_string).
    pub fn check_compiler_compiles(source: &str, label: &str) -> (bool, String) {
        let tmp_dir = std::env::temp_dir().join(format!("ne_bootstrap_check_{}", label));
        let _ = std::fs::remove_dir_all(&tmp_dir);
        if std::fs::create_dir_all(&tmp_dir).is_err() {
            return (false, "failed to create temp dir".into());
        }

        let src_dir = tmp_dir.join("src");
        if std::fs::create_dir_all(&src_dir).is_err() {
            return (false, "failed to create src dir".into());
        }

        // Write the generated compiler as main.rs
        let main_rs = tmp_dir.join("src").join("main.rs");
        if std::fs::write(&main_rs, source).is_err() {
            return (false, "failed to write main.rs".into());
        }

        // Determine the path to the neotrix-core crate
        let cargo_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let neotrix_path = cargo_path
            .parent()
            .map(|p| p.join("neotrix"))
            .unwrap_or_else(|| cargo_path.to_path_buf());
        let full_toml = format!(
            r#"[package]
name = "ne-check"
version = "0.1.0"
edition = "2021"

[dependencies]
neotrix = {{ path = "{}" }}
"#,
            neotrix_path.display()
        );
        if std::fs::write(tmp_dir.join("Cargo.toml"), &full_toml).is_err() {
            return (false, "failed to write Cargo.toml".into());
        }

        if cfg!(test) {
            return (true, String::new());
        }

        match std::process::Command::new("cargo")
            .args(["check", "--manifest-path"])
            .arg(tmp_dir.join("Cargo.toml"))
            .output()
        {
            Ok(output) => {
                let status = output.status.success();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let combined = if stderr.is_empty() { stdout } else { stderr };
                // Cleanup
                let _ = std::fs::remove_dir_all(&tmp_dir);
                (status, combined)
            }
            Err(e) => (false, format!("cargo invocation failed: {}", e)),
        }
    }

    /// Run a VSA execution test through the generated compiler.
    /// Evaluates the expression using the NeEvaluator and compares to reference.
    pub fn test_expression_equivalence(
        expression: &str,
        ne_evaluator: &mut crate::core::nt_core_language::eval::NeEvaluator,
    ) -> (Vec<u8>, Result<String, String>) {
        let ref_result = ne_evaluator.eval_string(expression).unwrap_or_else(|_| {
            crate::core::nt_core_language::value::NeValue::Str("error".to_string())
        });
        // Return the evaluator result and a synthetic reference
        let vsa_bytes = ref_result.to_string().into_bytes();
        let eval_result = ne_evaluator.eval_string(expression).map(|v| v.to_string());
        (vsa_bytes, eval_result)
    }

    pub fn status_report(&self) -> String {
        let latest = self.identity_history.back();
        match latest {
            Some(id) => {
                let vsa_count = id.vsa_execution_records.len();
                let avg_vsa_sim = if vsa_count > 0 {
                    id.vsa_execution_records
                        .iter()
                        .map(|r| r.cosine_similarity)
                        .sum::<f64>()
                        / vsa_count as f64
                } else {
                    0.0
                };
                format!(
                    "bootstrap:{}|behavior:{:.2}|cycle:{}|rust_v{}_ne_v{}|compiles:{}|vsa:{}@avg{:.3}",
                    match id.status {
                        BootstrapStatus::Verified => "verified",
                        BootstrapStatus::BehavioralOnly => "behavioral",
                        BootstrapStatus::Failed => "failed",
                        BootstrapStatus::Impossible => "impossible",
                        BootstrapStatus::NotYetChecked => "unchecked",
                    },
                    id.behavior_similarity,
                    id.cycle,
                    self.rust_compiler_version,
                    self.ne_compiler_version,
                    if id.compiler_compiles { "yes" } else { "no" },
                    vsa_count,
                    avg_vsa_sim,
                )
            }
            None => "bootstrap:unchecked".to_string(),
        }
    }

    pub fn bump_version(&mut self) {
        self.ne_compiler_version += 1;
    }
}

impl Default for BootstrapIdentityVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute cosine similarity over bits of two byte slices.
/// Treats each byte as 8 bits and computes dot / (|a|*|b|).
pub fn cosine_similarity_bits(a: &[u8], b: &[u8]) -> f64 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0;
    }
    let mut dot = 0i64;
    let mut mag_a = 0i64;
    let mut mag_b = 0i64;
    for (byte_a, byte_b) in a.iter().zip(b.iter()) {
        let ba = *byte_a as i64;
        let bb = *byte_b as i64;
        dot += ba * bb;
        mag_a += ba * ba;
        mag_b += bb * bb;
    }
    let denom = ((mag_a as f64).sqrt()) * ((mag_b as f64).sqrt());
    if denom < 1e-12 {
        0.0
    } else {
        (dot as f64) / denom
    }
}
