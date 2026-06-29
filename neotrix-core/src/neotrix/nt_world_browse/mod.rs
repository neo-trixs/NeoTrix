pub mod circuits_types;
pub mod humanize;
pub mod session;
pub mod stealth;

use self::circuits_types::{
    ReasoningCircuit, ReasoningInput, ReasoningMethod, ReasoningOutput, ReasoningTrace,
};
use std::sync::Mutex;

pub struct BrowserCircuit {
    pub session: Mutex<session::BrowserSession>,
}

impl Default for BrowserCircuit {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserCircuit {
    pub fn new() -> Self {
        Self {
            session: Mutex::new(session::BrowserSession::new()),
        }
    }
    pub fn browse(&self, url: &str) -> Result<String, String> {
        self.session
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .fetch(url)
    }
    pub fn login(&self, url: &str) -> Result<(), String> {
        self.session
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .login(url)
    }
}

impl ReasoningCircuit for BrowserCircuit {
    fn method(&self) -> ReasoningMethod {
        ReasoningMethod::SystemIntegration
    }
    fn complexity_ceiling(&self) -> f64 {
        0.5
    }
    fn process(&self, input: &ReasoningInput) -> ReasoningOutput {
        let session = self.session.lock().unwrap_or_else(|e| e.into_inner());
        let q: String = input
            .query
            .iter()
            .take(12)
            .map(|&x| ((x.abs() * 25.0) as u8).min(25) as char)
            .collect();
        let result = session
            .fetch_http(&format!(
                "https://lite.duckduckgo.com/lite/?q={}",
                url_encode(&q)
            ))
            .unwrap_or_default();
        let bytes: Vec<u8> = result.bytes().collect();
        let mut state = input.state.clone();
        for (i, val) in state.iter_mut().enumerate() {
            if let Some(&b) = bytes.get(i % bytes.len().max(1)) {
                *val = *val * 0.6 + (b as f64 / 255.0) * 0.4;
            }
        }
        let norm = state.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-8);
        for v in state.iter_mut() {
            *v /= norm;
        }
        ReasoningOutput {
            state_delta: state,
            confidence: (result.len() as f64 / 500.0).clamp(0.05, 1.0),
            trace: ReasoningTrace {
                method: ReasoningMethod::SystemIntegration,
                steps: result.lines().count(),
                intermediate_states: vec![],
                convergence: (result.len() as f64 / 2000.0).min(1.0),
            },
        }
    }
    fn is_applicable(&self, _c: f64) -> bool {
        true
    }
}

fn url_encode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "+".to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nt_world_browse_circuit_new_default() {
        let circuit = BrowserCircuit::new();
        assert_eq!(circuit.method(), ReasoningMethod::SystemIntegration);
    }

    #[test]
    fn test_nt_world_browse_circuit_default() {
        let circuit = BrowserCircuit::default();
        assert_eq!(circuit.complexity_ceiling(), 0.5);
    }

    #[test]
    fn test_url_encode_plain_text() {
        assert_eq!(url_encode("hello"), "hello");
    }

    #[test]
    fn test_url_encode_space_becomes_plus() {
        assert_eq!(url_encode("hello world"), "hello+world");
    }

    #[test]
    fn test_url_encode_special_chars() {
        assert_eq!(url_encode("a&b=c"), "a%26b%3Dc");
    }

    #[test]
    fn test_url_encode_empty_string() {
        assert_eq!(url_encode(""), "");
    }

    #[test]
    fn test_is_applicable_always_true() {
        let circuit = BrowserCircuit::new();
        assert!(circuit.is_applicable(0.0));
        assert!(circuit.is_applicable(100.0));
    }

    #[test]
    fn test_nt_world_browse_circuit_is_reasoning_circuit() {
        fn use_trait(c: &dyn ReasoningCircuit) {
            assert!(c.complexity_ceiling() > 0.0);
        }
        use_trait(&BrowserCircuit::new());
    }
}
