use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct BrowserCircuit;

impl BrowserCircuit {
    pub fn new() -> Self { Self }
    pub fn browse(&self, _url: &str) -> Result<String, String> { Ok(String::new()) }
}

#[derive(Debug, Clone)]
pub struct ReasoningOutput {
    pub state_delta: Vec<f64>,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct ReasoningKernel {
    pub stage: usize,
    pub state: Vec<f64>,
}

impl ReasoningKernel {
    pub fn new(stage: usize) -> Self {
        Self { stage: stage.min(18), state: vec![0.0; 128] }
    }

    pub fn reason(&self, _query: &[f64], _context: Option<HashMap<String, Vec<f64>>>) -> ReasoningOutput {
        ReasoningOutput { state_delta: self.state.clone(), confidence: 0.5 }
    }

    pub fn stats(&self) -> usize { self.stage }
}

pub struct AutonomousAgent {
    pub kernel: ReasoningKernel,
    pub nt_world_browse: BrowserCircuit,
}

impl AutonomousAgent {
    pub fn new(stage: usize) -> Self {
        Self { kernel: ReasoningKernel::new(stage.min(18)), nt_world_browse: BrowserCircuit::new() }
    }

    pub fn research(&mut self, query: &str) -> Result<String, String> {
        let query_vec = crate::neotrix::nt_io_standalone::text_to_vector(query, self.kernel.state.len());
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
            ctx.insert(format!("src_{}", i), crate::neotrix::nt_io_standalone::text_to_vector(f, self.kernel.state.len()));
        }
        let out = self.kernel.reason(&query_vec, Some(ctx));
        let e: f64 = out.state_delta.iter().map(|x| x.abs()).sum::<f64>() / out.state_delta.len().max(1) as f64;
        let _s = self.kernel.stats();
        let report = format!(
            "═══ Research: {} ═══\nSources: {} | Conf: {:.0}% | Energy: {:.4}\n\nState: {:?}",
            query, sources.len(), out.confidence * 100.0, e, &out.state_delta[..out.state_delta.len().min(16)]
        );
        Ok(report)
    }
}

fn url_encode(s: &str) -> String {
    s.chars().map(|c| match c { 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(), ' ' => "+".to_string(), _ => format!("%{:02X}", c as u8) }).collect()
}
