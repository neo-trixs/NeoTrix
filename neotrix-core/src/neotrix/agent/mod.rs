use std::collections::HashMap;
use crate::neotrix::kernel_core::ReasoningKernel;
use crate::neotrix::nt_world_browse::BrowserCircuit;

pub mod plugin;

pub use plugin::*;

pub struct AutonomousAgent {
    pub kernel: ReasoningKernel,
    pub nt_world_browse: BrowserCircuit,
}

impl AutonomousAgent {
    pub fn new(stage: usize) -> Self {
        Self { kernel: ReasoningKernel::new(stage.min(18)), nt_world_browse: BrowserCircuit::new() }
    }

    pub fn research(&mut self, query: &str) -> Result<String, String> {
        let query_vec = crate::neotrix::standalone::text_to_vector(query, self.kernel.state.len());
        let mut findings = Vec::new();
        let mut sources = Vec::new();

        let search_url = format!("https://lite.duckduckgo.com/lite/?q={}", url_encode(query));
        if let Ok(text) = self.nt_world_browse.browse(&search_url) {
            findings.push(text.chars().take(2000).collect::<String>());
            sources.push(search_url);
        }

        let first = findings.first().cloned().unwrap_or_default();
        for line in first.lines().take(5) {
            let url = line.split_whitespace().find(|w| w.starts_with("http")).unwrap_or("");
            if !url.is_empty() && !sources.iter().any(|s| s == url) {
                if let Ok(text) = self.nt_world_browse.browse(url) {
                    findings.push(text.chars().take(3000).collect::<String>());
                    sources.push(url.to_string());
                }
            }
        }

        let mut ctx = HashMap::new();
        for (i, f) in findings.iter().enumerate() {
            ctx.insert(format!("src_{}", i), crate::neotrix::standalone::text_to_vector(f, self.kernel.state.len()));
        }
        let out = self.kernel.reason(&query_vec, Some(ctx));
        let e: f64 = out.state_delta.iter().map(|x| x.abs()).sum::<f64>() / out.state_delta.len().max(1) as f64;
        let _s = self.kernel.stats();
        let report = format!(
            "═══ Research: {} ═══\nSources: {} | Conf: {:.0}%\n\n{}",
            query, sources.len(), out.confidence * 100.0,
            crate::agent::decoder::decode_state(&out.state_delta, out.confidence, e.min(1.0))
        );
        Ok(report)
    }
}

fn url_encode(s: &str) -> String {
    s.chars().map(|c| match c { 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(), ' ' => "+".to_string(), _ => format!("%{:02X}", c as u8) }).collect()
}
