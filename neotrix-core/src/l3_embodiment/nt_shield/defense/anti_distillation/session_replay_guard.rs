//! Session replay guard for anti-distillation
//!
//! Detects cross-session replay attacks where:
//! 1. Actor saves reasoning signature from response
//! 2. Starts new session
//! 3. Elicits conversion back to full reasoning trace

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use super::{DetectionSignal, ThreatLevel};

pub struct SessionReplayGuard {
    seen_signatures: Arc<RwLock<HashSet<String>>>,
    signature_to_account: Arc<RwLock<HashMap<String, String>>>,
}

impl SessionReplayGuard {
    pub fn new() -> Self {
        Self {
            seen_signatures: Arc::new(RwLock::new(HashSet::new())),
            signature_to_account: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn check_replay(
        &self,
        account_id: &str,
        prompt: &str,
        metadata: &HashMap<String, String>,
    ) -> Option<DetectionSignal> {
        // Check if reasoning signature is being replayed
        if let Some(signature) = metadata.get("reasoning_signature") {
            let signatures = self.seen_signatures.read().await;
            let account_map = self.signature_to_account.read().await;

            if signatures.contains(signature) {
                // Check if this is a different session/account
                if let Some(original_account) = account_map.get(signature) {
                    if original_account != account_id {
                        return Some(DetectionSignal {
                            signal_type: "session_replay".to_string(),
                            confidence: 0.95,
                            threat_level: ThreatLevel::Critical,
                            details: format!(
                                "Reasoning signature replay detected: account {} replaying signature from {}",
                                account_id, original_account
                            ),
                        });
                    }
                }
            }
        }

        // Check for prompt patterns that try to elicit reasoning conversion
        let replay_patterns = [
            "convert the reasoning signature back",
            "convert.*reasoning.*signature",
            "extract.*reasoning.*trace",
            "output.*thinking.*content",
            "reveal.*reasoning",
        ];

        let prompt_lower = prompt.to_lowercase();
        for pattern in &replay_patterns {
            if prompt_lower.contains(pattern) {
                return Some(DetectionSignal {
                    signal_type: "replay_elicitation".to_string(),
                    confidence: 0.8,
                    threat_level: ThreatLevel::High,
                    details: format!("Replay elicitation pattern detected: {}", pattern),
                });
            }
        }

        None
    }

    pub async fn record_signature(&self, account_id: &str, signature: &str) {
        let mut signatures = self.seen_signatures.write().await;
        let mut account_map = self.signature_to_account.write().await;

        signatures.insert(signature.to_string());
        account_map.insert(signature.to_string(), account_id.to_string());
    }
}
