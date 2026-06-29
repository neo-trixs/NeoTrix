use std::collections::HashMap;

// --- Helper: deterministic VSA encoding ---
fn text_to_vsa(text: &str) -> Vec<u8> {
    let b = text.as_bytes();
    (0..4096)
        .map(|i| {
            let idx = i % b.len().max(1);
            b[idx]
                .wrapping_add((i as u32).wrapping_mul(0x9E3779B9) as u8)
                .wrapping_mul(0x3D)
                .wrapping_add(0x17)
        })
        .collect()
}

fn vsa_cos(a: &[u8], b: &[u8]) -> f64 {
    let len = a.len().min(b.len());
    let dot: u64 = a[..len]
        .iter()
        .zip(b[..len].iter())
        .map(|(x, y)| (*x as u64) * (*y as u64))
        .sum();
    let na = (a[..len].iter().map(|x| (*x as f64).powi(2)).sum::<f64>()).sqrt();
    let nb = (b[..len].iter().map(|x| (*x as f64).powi(2)).sum::<f64>()).sqrt();
    if na < 1e-10 || nb < 1e-10 {
        0.0
    } else {
        dot as f64 / (na * nb)
    }
}

// --- Structs ---
#[derive(Debug, Clone)]
pub struct ToolCapability {
    pub name: String,
    pub description: String,
    pub semantic_vector: Vec<u8>,
    pub domain: String,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct DiscoveryRequest {
    pub id: u64,
    pub gap_description: String,
    pub semantic_vector: Vec<u8>,
    pub urgency: f64,
}

#[derive(Debug, Clone)]
pub struct DiscoveryAttempt {
    pub request_id: u64,
    pub discovered: Vec<ToolCapability>,
    pub success: bool,
}

#[derive(Debug)]
pub struct SemanticToolRouter {
    pub known_tools: Vec<ToolCapability>,
    pub domain_centroids: HashMap<String, Vec<u8>>,
}

#[derive(Debug)]
pub struct ToolDiscoveryEngine {
    pub router: SemanticToolRouter,
    pub pending: Vec<DiscoveryRequest>,
    pub completed: Vec<DiscoveryAttempt>,
    pub gap_counter: u64,
    pub threshold: f64,
}

pub struct DiscoveryStats {
    pub total_requests: u64,
    pub completed_count: u64,
    pub pending_count: usize,
    pub tools_known: usize,
}

// --- Implementations ---
impl SemanticToolRouter {
    pub fn new() -> Self {
        Self {
            known_tools: vec![],
            domain_centroids: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &str, desc: &str, domain: &str) {
        let vec = text_to_vsa(&format!("{}:{}:{}", name, desc, domain));
        self.known_tools.push(ToolCapability {
            name: name.into(),
            description: desc.into(),
            semantic_vector: vec.clone(),
            domain: domain.into(),
            confidence: 1.0,
        });
        let centroid = self
            .domain_centroids
            .entry(domain.into())
            .or_insert_with(|| vec![0u8; 4096]);
        for i in 0..4096 {
            centroid[i] = centroid[i].wrapping_add(vec[i]);
        }
    }

    pub fn route(&self, query: &[u8]) -> Vec<(ToolCapability, f64)> {
        let mut scored: Vec<_> = self
            .known_tools
            .iter()
            .map(|t| (t.clone(), vsa_cos(query, &t.semantic_vector)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored
    }

    pub fn classify_domain(&self, query: &[u8]) -> (String, f64) {
        self.domain_centroids
            .iter()
            .map(|(d, c)| (d.clone(), vsa_cos(query, c)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(("unknown".into(), 0.0))
    }
}

impl ToolDiscoveryEngine {
    pub fn new(threshold: f64) -> Self {
        Self {
            router: SemanticToolRouter::new(),
            pending: vec![],
            completed: vec![],
            gap_counter: 0,
            threshold,
        }
    }

    pub fn detect_gap(&mut self, task_desc: &str, context: &str) -> Option<DiscoveryRequest> {
        let vec = text_to_vsa(&format!("{}:{}", task_desc, context));
        let matches = self.router.route(&vec);
        let max_sim = matches.first().map(|(_, s)| *s).unwrap_or(0.0);
        if max_sim < self.threshold {
            self.gap_counter += 1;
            Some(DiscoveryRequest {
                id: self.gap_counter,
                gap_description: task_desc.into(),
                semantic_vector: vec,
                urgency: 1.0 - max_sim,
            })
        } else {
            None
        }
    }

    pub fn submit_request(&mut self, req: DiscoveryRequest) {
        self.pending.push(req);
    }

    pub fn attempt_discovery(&mut self) -> Option<DiscoveryAttempt> {
        let req = self.pending.pop()?;
        let matches = self.router.route(&req.semantic_vector);
        let (discovered, success) = if matches.is_empty() || matches[0].1 < self.threshold {
            (vec![], false)
        } else {
            (matches.into_iter().take(3).map(|(t, _)| t).collect(), true)
        };
        let attempt = DiscoveryAttempt {
            request_id: req.id,
            discovered,
            success,
        };
        self.completed.push(attempt.clone());
        Some(attempt)
    }

    pub fn stats(&self) -> DiscoveryStats {
        DiscoveryStats {
            total_requests: self.gap_counter,
            completed_count: self.completed.len() as u64,
            pending_count: self.pending.len(),
            tools_known: self.router.known_tools.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_to_vsa_deterministic() {
        let a = text_to_vsa("hello world");
        let b = text_to_vsa("hello world");
        assert_eq!(a, b);
    }

    #[test]
    fn test_text_to_vsa_different() {
        let a = text_to_vsa("hello");
        let b = text_to_vsa("world");
        assert_ne!(a, b);
    }

    #[test]
    fn test_vsa_cos_identity() {
        let v = text_to_vsa("test");
        assert!((vsa_cos(&v, &v) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_register_and_route() {
        let mut router = SemanticToolRouter::new();
        router.register("read", "read file contents", "file");
        let q = text_to_vsa("read file");
        let results = router.route(&q);
        assert!(!results.is_empty());
        assert_eq!(results[0].0.name, "read");
    }

    #[test]
    fn test_detect_gap_creates_gap() {
        let mut engine = ToolDiscoveryEngine::new(0.6);
        let gap = engine.detect_gap("some completely unknown capability", "need this tool");
        assert!(gap.is_some());
        assert_eq!(engine.stats().total_requests, 1);
    }

    #[test]
    fn test_detect_gap_no_gap() {
        let mut engine = ToolDiscoveryEngine::new(0.6);
        engine
            .router
            .register("search_files", "search file contents", "file");
        let gap = engine.detect_gap("search file contents", "grep for pattern");
        assert!(gap.is_none());
    }

    #[test]
    fn test_discovery_attempt_fails_with_no_tools() {
        let mut engine = ToolDiscoveryEngine::new(0.6);
        let req = engine.detect_gap("unknown tool", "need it").unwrap();
        engine.submit_request(req);
        let attempt = engine.attempt_discovery().unwrap();
        assert!(!attempt.success);
        assert!(attempt.discovered.is_empty());
    }

    #[test]
    fn test_classify_domain() {
        let mut router = SemanticToolRouter::new();
        router.register("read_file", "read", "file");
        router.register("web_fetch", "fetch", "network");
        let (domain, sim) = router.classify_domain(&text_to_vsa("read stuff"));
        assert_eq!(domain, "file");
        assert!(sim > 0.0);
    }
}
