use std::collections::VecDeque;

use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

/// Proof-Carrying Output: binds AI output, formal proof, validator version, and timestamp
/// into a non-repudiable commitment chain.
///
/// Reference: IACR 2026/994 — Proof-Carrying Output for AI
#[derive(Debug, Clone)]
pub struct ProofRecord {
    pub output_hash: [u8; 32],
    pub proof_hash: [u8; 32],
    pub validator_version: u32,
    pub timestamp_ns: u64,
    pub prev_proof_hash: [u8; 32],
    pub signature: Vec<u8>,
}

/// A chain of `ProofRecord` entries with HMAC-SHA256 signing and chain linking.
///
/// Each record is signed with HMAC-SHA256(key, output_hash || proof_hash ||
/// validator_version || timestamp_ns || prev_proof_hash) and linked to its
/// predecessor via `prev_proof_hash = SHA-256(prev_record)`.
pub struct PcoChain {
    records: VecDeque<ProofRecord>,
    max_len: usize,
    hmac_key: [u8; 32],
}

impl PcoChain {
    pub fn new(hmac_key: [u8; 32]) -> Self {
        Self {
            records: VecDeque::new(),
            max_len: 1000,
            hmac_key,
        }
    }

    pub fn with_max_len(mut self, max: usize) -> Self {
        self.max_len = if max < 1 { 1 } else { max };
        self
    }

    pub fn commit(&mut self, output: &[u8], proof: &[u8]) -> ProofRecord {
        let output_hash = Sha256::digest(output).into();
        let proof_hash = Sha256::digest(proof).into();
        let timestamp_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        let prev_proof_hash = self
            .records
            .back()
            .map(|r| hash_record(r))
            .unwrap_or([0u8; 32]);
        let unsig = ProofRecord {
            output_hash,
            proof_hash,
            validator_version: 1,
            timestamp_ns,
            prev_proof_hash,
            signature: Vec::new(),
        };
        let signature = sign_record(&self.hmac_key, &unsig);
        let record = ProofRecord { signature, ..unsig };
        if self.records.len() >= self.max_len {
            self.records.pop_front();
        }
        self.records.push_back(record.clone());
        record
    }

    pub fn verify_chain(&self) -> bool {
        if self.records.is_empty() {
            return true;
        }
        let mut prev_hash = [0u8; 32];
        for record in &self.records {
            if record.prev_proof_hash != prev_hash {
                return false;
            }
            if !verify_signature(&self.hmac_key, record) {
                return false;
            }
            prev_hash = hash_record(record);
        }
        true
    }

    pub fn verify_record(&self, record: &ProofRecord) -> bool {
        verify_signature(&self.hmac_key, record)
    }

    pub fn verify_record_with_output(&self, record: &ProofRecord, output: &[u8]) -> bool {
        let expected = Sha256::digest(output);
        if record.output_hash != expected[..] {
            return false;
        }
        verify_signature(&self.hmac_key, record)
    }

    pub fn last(&self) -> Option<&ProofRecord> {
        self.records.back()
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn prune(&mut self, keep: usize) {
        let keep = if keep < 1 { 1 } else { keep };
        while self.records.len() > keep {
            self.records.pop_front();
        }
    }
}

fn hash_record(record: &ProofRecord) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(&record.output_hash);
    h.update(&record.proof_hash);
    h.update(&record.validator_version.to_le_bytes());
    h.update(&record.timestamp_ns.to_le_bytes());
    h.update(&record.prev_proof_hash);
    h.update(&record.signature);
    h.finalize().into()
}

fn sign_record(key: &[u8; 32], record: &ProofRecord) -> Vec<u8> {
    let mut mac = match HmacSha256::new_from_slice(key) {
        Ok(m) => m,
        Err(e) => {
            log::error!("HMAC-SHA256 key error: {}", e);
            return Vec::new();
        }
    };
    mac.update(&record.output_hash);
    mac.update(&record.proof_hash);
    mac.update(&record.validator_version.to_le_bytes());
    mac.update(&record.timestamp_ns.to_le_bytes());
    mac.update(&record.prev_proof_hash);
    mac.finalize().into_bytes().to_vec()
}

fn verify_signature(key: &[u8; 32], record: &ProofRecord) -> bool {
    let mut mac = match HmacSha256::new_from_slice(key) {
        Ok(m) => m,
        Err(e) => {
            log::error!("HMAC-SHA256 key error: {}", e);
            return false;
        }
    };
    mac.update(&record.output_hash);
    mac.update(&record.proof_hash);
    mac.update(&record.validator_version.to_le_bytes());
    mac.update(&record.timestamp_ns.to_le_bytes());
    mac.update(&record.prev_proof_hash);
    mac.verify_slice(&record.signature).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_chain() -> PcoChain {
        PcoChain::new([42u8; 32])
    }

    #[test]
    fn test_commit_and_verify_chain() {
        let mut chain = test_chain();
        chain.commit(b"hello world", b"proof_data");
        assert!(chain.verify_chain());
        assert_eq!(chain.len(), 1);
    }

    #[test]
    fn test_verify_record() {
        let mut chain = test_chain();
        let record = chain.commit(b"output", b"proof");
        assert!(chain.verify_record(&record));
    }

    #[test]
    fn test_output_mismatch() {
        let mut chain = test_chain();
        let record = chain.commit(b"original output", b"proof");
        assert!(chain.verify_record_with_output(&record, b"original output"));
        assert!(!chain.verify_record_with_output(&record, b"tampered output"));
    }

    #[test]
    fn test_proof_mismatch() {
        let mut chain = PcoChain::new([42u8; 32]);
        let record = chain.commit(b"output", b"original proof");
        let mut tampered = record.clone();
        tampered.proof_hash = Sha256::digest(b"tampered proof").into();
        assert!(!chain.verify_record(&tampered));
    }

    #[test]
    fn test_chain_continuity() {
        let mut chain = PcoChain::new([99u8; 32]);
        chain.commit(b"first", b"p1");
        chain.commit(b"second", b"p2");
        chain.commit(b"third", b"p3");
        assert_eq!(chain.len(), 3);
        assert!(chain.verify_chain());
        let records: Vec<&ProofRecord> = chain.records.iter().collect();
        let expected_prev = hash_record(records[0]);
        assert_eq!(records[1].prev_proof_hash, expected_prev);
    }

    #[test]
    fn test_broken_chain_detected() {
        let mut chain = PcoChain::new([42u8; 32]);
        chain.commit(b"a", b"p1");
        chain.commit(b"b", b"p2");
        if let Some(second) = chain.records.get_mut(1) {
            second.prev_proof_hash = [0u8; 32];
        }
        assert!(!chain.verify_chain());
    }

    #[test]
    fn test_multiple_commits() {
        let mut chain = PcoChain::new([42u8; 32]);
        for i in 0..100 {
            let out = format!("output {}", i);
            let prf = format!("proof {}", i);
            chain.commit(out.as_bytes(), prf.as_bytes());
        }
        assert_eq!(chain.len(), 100);
        assert!(chain.verify_chain());
    }

    #[test]
    fn test_prune() {
        let mut chain = PcoChain::new([42u8; 32]);
        for i in 0..10 {
            let out = format!("out {}", i);
            let prf = format!("prf {}", i);
            chain.commit(out.as_bytes(), prf.as_bytes());
        }
        assert_eq!(chain.len(), 10);
        chain.prune(3);
        assert_eq!(chain.len(), 3);
        assert!(chain.verify_chain());
    }

    #[test]
    fn test_prune_keeps_at_least_one() {
        let mut chain = PcoChain::new([42u8; 32]);
        chain.commit(b"only", b"record");
        chain.prune(0);
        assert_eq!(chain.len(), 1);
    }

    #[test]
    fn test_empty_chain_verifies() {
        let chain = PcoChain::new([42u8; 32]);
        assert!(chain.verify_chain());
        assert!(chain.last().is_none());
        assert_eq!(chain.len(), 0);
    }

    #[test]
    fn test_max_len_enforced() {
        let mut chain = PcoChain::new([42u8; 32]).with_max_len(3);
        for i in 0..10 {
            let out = format!("o {}", i);
            let prf = format!("p {}", i);
            chain.commit(out.as_bytes(), prf.as_bytes());
        }
        assert_eq!(chain.len(), 3);
        assert!(chain.verify_chain());
    }

    #[test]
    fn test_different_keys_produce_different_signatures() {
        let mut chain_a = PcoChain::new([1u8; 32]);
        let mut chain_b = PcoChain::new([2u8; 32]);
        let rec_a = chain_a.commit(b"same", b"same");
        let rec_b = chain_b.commit(b"same", b"same");
        assert_ne!(rec_a.signature, rec_b.signature);
        assert!(chain_a.verify_chain());
        assert!(chain_b.verify_chain());
        assert!(!chain_a.verify_record(&rec_b));
        assert!(!chain_b.verify_record(&rec_a));
    }

    #[test]
    fn test_last_returns_newest() {
        let mut chain = PcoChain::new([42u8; 32]);
        assert!(chain.last().is_none());
        chain.commit(b"first", b"p1");
        chain.commit(b"second", b"p2");
        let last = chain.last().expect("should have a record");
        let expected: [u8; 32] = Sha256::digest(b"second").into();
        assert_eq!(last.output_hash, expected);
    }

    #[test]
    fn test_record_fields_are_populated() {
        let mut chain = PcoChain::new([42u8; 32]);
        let record = chain.commit(b"test output", b"test proof");
        assert_ne!(record.output_hash, [0u8; 32]);
        assert_ne!(record.proof_hash, [0u8; 32]);
        assert_eq!(record.validator_version, 1);
        assert!(record.timestamp_ns > 0);
        assert!(!record.signature.is_empty());
    }

    #[test]
    fn test_signature_tampered_detected() {
        let mut chain = PcoChain::new([42u8; 32]);
        let mut record = chain.commit(b"output", b"proof");
        record.signature[0] ^= 0x01;
        assert!(!chain.verify_record(&record));
    }
}
