//! # NT-SHIELD: Shield Capability Module
//!
//! 渗透测试与漏洞扫描能力模块。
//! 吸收 GitHub 开源项目: nuclei, fscan, pentestgpt, ghidra (via FFI)
//!
//! Architecture Position:
//! - Layer: L1 Body (Physical Security & Execution)
//! - Domain: NT-SHIELD
//! - Integration: GWT attention routing + E8 reasoning + VSA embedding
//!
//! Capability Tree Nodes:
//! - Small Passive: `nt_shield_vuln_scanner` (nuclei integration)
//! - Notable Passive: `nt_shield_pentest_agent` (AI-driven exploitation)
//! - Keystone: `nt_shield_internal_scan` (fscan internal network module)

use std::collections::HashMap;
use std::path::PathBuf;
use crate::core::nt_core_gwt::GWTContext;
use crate::core::nt_core_e8::E8;

/// Nuclei vulnerability scanner integration
pub mod nt_shield_vuln_scanner;

/// AI-driven pentest agent (PentestGPT adapter)
pub mod nt_shield_pentest_agent;

/// Internal network scanning (fscan integration)
pub mod nt_shield_internal_scan;

/// Reverse engineering capabilities (Ghidra/radare2 bridge)
pub mod nt_shield_reverse_engineer;

/// Mobile security analysis (Objection adapter)
pub mod nt_shield_mobile_analyzer;

/// Web security scanning (w3af/arachni integration)
pub mod nt_shield_web_scanner;

/// AI security testing (ART toolbox integration)
pub mod nt_shield_ai_security;

/// Local model inference optimization (quantization, KV cache, FlashAttention)
pub mod nt_shield_local_inference;

/// PentestCode swarm agent system (13 agents)
pub mod nt_shield_pentest_swarm;

/// Swarm Intelligence coordination
pub mod nt_shield_swarm;

pub use nt_shield_vuln_scanner::NucleiEngine;
pub use nt_shield_pentest_agent::PentestGPTAdapter;
pub use nt_shield_internal_scan::FscanModule;
pub use nt_shield_reverse_engineer::GhidraAnalyzer;
pub use nt_shield_mobile_analyzer::ObjectionAdapter;
pub use nt_shield_web_scanner::W3afEngine;
pub use nt_shield_ai_security::ARTToolbox;
pub use nt_shield_local_inference::{LocalInferenceEngine, QuantizationEngine, KVCacheOptimizer};
pub use nt_shield_local_inference::quantization_engine::{QuantizationConfig, QuantFormatConfig, QualityBenchmark, GGUFModel, QuantLevel, ModelScore, DynamicQuantSelection, EvoPressResult, LayerQuantConfig, GptqGgufConfig, EvoPressConfig, GptqConfig, ImportanceMatrix, IMMethod};
pub use nt_shield_local_inference::kv_cache_optimizer::{KVCacheLayout, MemorySavingsReport, ContextCapacity, ContinuousBatchingConfig, KVCACHEType};
pub use nt_shield_local_inference::inference_runtime::{InferenceRuntime, BackendEngine, RuntimeConfig, ThroughputBenchmark};
pub use nt_shield_local_inference::model_selector::{ModelSelector, ModelRecommendation, ModelInfo, HardwareProfile};
pub use nt_shield_local_inference::apple_silicon::{AppleSiliconOptimizer, AppleChip, MLXPerfData, RuntimeSelection};
pub use nt_shield_local_inference::speculative_decoding::{SpeculativeDecoder, SpeculativeResult, AcceptanceStats};
pub use nt_shield_local_inference::{OptimalServerCmd, PerformanceExpectation, OptimalLlamaArgs};

/// Shield capability orchestrator
#[derive(Debug)]
pub struct ShieldCapability {
    /// Vulnerability scanning engine (nuclei)
    pub scanner: NucleiEngine,
    
    /// AI penetration testing agent
    pub pentest_agent: PentestGPTAdapter,
    
    /// Internal network discovery module
    pub internal_scan: FscanModule,
    
    /// Reverse engineering capabilities
    pub reverse_engineer: GhidraAnalyzer,
    
    /// Mobile security analyzer
    pub mobile_analyzer: ObjectionAdapter,
    
    /// Web security scanner
    pub web_scanner: W3afEngine,
    
    /// AI security testing toolkit
    pub ai_security: ARTToolbox,
    
    /// Local model inference optimization
    pub local_inference: LocalInferenceEngine,
    
    /// State management
    pub findings: HashMap<String, Vec<String>>,
    pub attack_graph: HashMap<String, Vec<String>>,
}

impl Default for ShieldCapability {
    fn default() -> Self {
        Self {
            scanner: NucleiEngine::new(),
            pentest_agent: PentestGPTAdapter::new(),
            internal_scan: FscanModule::new(),
            reverse_engineer: GhidraAnalyzer::new(),
            mobile_analyzer: ObjectionAdapter::new(),
            web_scanner: W3afEngine::new(),
            ai_security: ARTToolbox::new(),
            local_inference: LocalInferenceEngine::new(),
            findings: HashMap::new(),
            attack_graph: HashMap::new(),
        }
    }
}

impl ShieldCapability {
    /// Initialize Shield capability with custom paths
    pub fn with_paths(
        nuclei_templates: PathBuf,
        ghidra_path: PathBuf,
    ) -> Result<Self, String> {
        Ok(Self {
            scanner: NucleiEngine::with_templates_dir(nuclei_templates)?,
            pentest_agent: PentestGPTAdapter::new(),
            internal_scan: FscanModule::new(),
            reverse_engineer: GhidraAnalyzer::with_path(ghidra_path)?,
            mobile_analyzer: ObjectionAdapter::new(),
            web_scanner: W3afEngine::new(),
            ai_security: ARTToolbox::new(),
            local_inference: LocalInferenceEngine::new(),
            findings: HashMap::new(),
            attack_graph: HashMap::new(),
        })
    }
    
    /// Run comprehensive security assessment
    pub async fn run_assessment(
        &mut self,
        target: &str,
        gwt_context: &GWTContext,
    ) -> Vec<String> {
        let mut results = Vec::new();
        
        // Phase 1: Reconnaissance via Nuclei
        let recon = self.scanner.reconnaissance(target).await;
        for finding in &recon.findings {
            self.findings.entry(target.to_string()).or_default().push(finding.clone());
        }
        
        // Phase 2: AI-driven vulnerability detection (GWT routing)
        if let Some(vulns) = self.pentest_agent.detect_vulnerabilities(target, gwt_context).await {
            for vuln in vulns {
                results.push(format!("{}: {}", target, vuln));
            }
        }
        
        // Phase 3: Internal network scanning (if applicable)
        if let Some(internal_hosts) = self.internal_scan.discover_internal_hosts().await {
            for host in internal_hosts {
                self.attack_graph
                    .entry(format!("{} -> {}", target, host))
                    .or_default();
            }
        }
        
        results
    }
    
    /// Extract VSA embedding from vulnerability findings
    use crate::core::nt_core_hcube::{FhrrHyperCube, FhrrVector};
    
    pub fn extract_finding_embedding(&self, finding: &str) -> FhrrVector {
        let vec = FhrrHyperCube::random_vector(1024);
        // In production: encode finding text into VSA using word2vec/BERT
        vec
    }
}
