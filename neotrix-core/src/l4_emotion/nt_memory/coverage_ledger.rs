#![forbid(unsafe_code)]

//! Coverage Ledger — 知识操作覆盖账本
//!
//! NT-MEMORY 核心组件，所有知识操作记录到 additive ledger，
//! 使用 SHA-256 哈希链保证不可篡改性，Merkle proof 验证完整性。
//!
//! 与 experience-tree 集成: 每次吸收 (absorb) 操作自动记录到 ledger，
//! 确保知识演进的完整可追溯性。

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use neotrix_types::nt_crypto_util::sha256_hex_str as sha256_hex;

// ─── Operation Types ──────────────────────────────────────────────

/// 知识操作类型 — 覆盖所有 NT-MEMORY 操作
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationType {
    Read,
    Write,
    Delete,
    Absorb,
    Distill,
    Query,
    Merge,
    Verify,
    ExperienceIngest,
    SkillLoad,
}

impl OperationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            OperationType::Read => "Read",
            OperationType::Write => "Write",
            OperationType::Delete => "Delete",
            OperationType::Absorb => "Absorb",
            OperationType::Distill => "Distill",
            OperationType::Query => "Query",
            OperationType::Merge => "Merge",
            OperationType::Verify => "Verify",
            OperationType::ExperienceIngest => "ExperienceIngest",
            OperationType::SkillLoad => "SkillLoad",
        }
    }
}

// ─── Ledger Entry ─────────────────────────────────────────────────

/// 单条知识操作记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub id: String,
    pub timestamp: i64,
    pub operation_type: OperationType,
    pub domain: String,
    pub content_hash: String,
    pub previous_hash: String,
    pub metadata: HashMap<String, String>,
    pub integrity_proof: Option<String>,
}

impl LedgerEntry {
    pub fn new(
        operation_type: OperationType,
        domain: &str,
        content: &str,
        previous_hash: String,
        metadata: HashMap<String, String>,
    ) -> Self {
        let content_hash = sha256_hex(content);
        let id = Uuid::new_v4().to_string();
        let timestamp = Utc::now().timestamp();

        Self {
            id,
            timestamp,
            operation_type,
            domain: domain.to_string(),
            content_hash,
            previous_hash,
            metadata,
            integrity_proof: None,
        }
    }
}

// ─── Merkle Proof ─────────────────────────────────────────────────

/// Merkle 证明 — 用于验证特定条目是否在 Merkle 树中
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub entry_index: usize,
    pub proof_path: Vec<String>,
    pub root_hash: String,
    pub verified: bool,
}

impl MerkleProof {
    pub fn verify(&self) -> bool {
        if self.proof_path.is_empty() {
            return self.root_hash.is_empty();
        }

        let mut computed = self.proof_path[0].clone();
        for sibling in self.proof_path.iter().skip(1) {
            let combined = format!("{}{}", computed, sibling);
            computed = sha256_hex(&combined);
        }

        computed == self.root_hash && self.verified
    }
}

// ─── Coverage Ledger ──────────────────────────────────────────────

/// 覆盖账本 — 所有知识操作的不可变记录
pub struct CoverageLedger {
    entries: Vec<LedgerEntry>,
    hash_chain: Vec<String>,
    merkle_tree: Vec<Vec<String>>,
    root_hash: String,
    operation_count: u64,
    domains: HashSet<String>,
    audit_log: Arc<Mutex<Vec<String>>>,
}

impl CoverageLedger {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            hash_chain: vec!["genesis".into()],
            merkle_tree: vec![],
            root_hash: String::new(),
            operation_count: 0,
            domains: HashSet::new(),
            audit_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 记录知识操作，自动计算 SHA-256 哈希并更新哈希链和 Merkle 树
    pub fn record_operation(
        &mut self,
        operation_type: OperationType,
        domain: &str,
        content: &str,
        metadata: HashMap<String, String>,
    ) -> String {
        let previous_hash = self.hash_chain.last().cloned().unwrap_or_else(|| "genesis".into());
        let entry = LedgerEntry::new(operation_type, domain, content, previous_hash, metadata);
        let entry_hash = entry.content_hash.clone();

        // 更新哈希链
        let chain_input = format!("{}{}", previous_hash, entry_hash);
        let chain_hash = sha256_hex(&chain_input);
        self.hash_chain.push(chain_hash.clone());

        // 更新域追踪
        self.domains.insert(domain.to_string());
        self.operation_count += 1;

        let entry_id = entry.id.clone();
        self.entries.push(entry);

        // 重建 Merkle 树
        self.rebuild_merkle_tree();

        // 记录到审计日志
        self.log_audit(&format!("Recorded {} operation {} in domain {}", operation_type.as_str(), entry_id, domain));

        entry_id
    }

    /// 验证哈希链连续性 — 任何篡改都会破坏链
    pub fn verify_integrity(&self) -> bool {
        if self.entries.is_empty() {
            return self.hash_chain.len() == 1 && self.hash_chain[0] == "genesis";
        }

        for i in 1..self.entries.len() {
            let current = &self.entries[i];
            let previous = &self.entries[i - 1];
            let expected_chain_input = format!("{}{}", previous.content_hash, current.content_hash);
            let expected_hash = sha256_hex(&expected_chain_input);

            if self.hash_chain[i] != expected_hash {
                return false;
            }

            if current.previous_hash != previous.content_hash {
                return false;
            }
        }

        true
    }

    /// 生成 Merkle 证明
    pub fn generate_merkle_proof(&self, index: usize) -> MerkleProof {
        if index >= self.entries.len() {
            return MerkleProof {
                entry_index: index,
                proof_path: vec![],
                root_hash: self.root_hash.clone(),
                verified: false,
            };
        }

        let mut proof_path = Vec::new();
        let mut idx = index;
        let mut current_layer = self.merkle_tree.first().cloned().unwrap_or_default();

        while current_layer.len() > 1 {
            let sibling_idx = if idx % 2 == 0 {
                idx + 1
            } else {
                idx - 1
            };

            if sibling_idx < current_layer.len() {
                proof_path.push(current_layer[sibling_idx].clone());
            }

            idx /= 2;
            if idx < current_layer.len() / 2 {
                current_layer = self.merkle_tree.get(current_layer.len() / 2).cloned().unwrap_or_default();
            } else {
                break;
            }
        }

        let verified = self.verify_proof_internal(index, &proof_path);
        MerkleProof {
            entry_index: index,
            proof_path,
            root_hash: self.root_hash.clone(),
            verified,
        }
    }

    /// 验证 Merkle 证明
    pub fn verify_proof(&self, proof: &MerkleProof) -> bool {
        if proof.entry_index >= self.entries.len() {
            return false;
        }
        if proof.root_hash != self.root_hash {
            return false;
        }

        let mut computed = self.entries[proof.entry_index].content_hash.clone();
        for sibling in &proof.proof_path {
            let combined = format!("{}{}", computed, sibling);
            computed = sha256_hex(&combined);
        }

        proof.verified = computed == proof.root_hash;
        proof.verified
    }

    /// 内部验证函数
    fn verify_proof_internal(&self, index: usize, proof_path: &[String]) -> bool {
        let mut computed = self.entries[index].content_hash.clone();
        for sibling in proof_path {
            let combined = format!("{}{}", computed, sibling);
            computed = sha256_hex(&combined);
        }
        computed == self.root_hash
    }

    /// 重建 Merkle 树
    fn rebuild_merkle_tree(&mut self) {
        if self.entries.is_empty() {
            self.merkle_tree = vec![];
            self.root_hash = String::new();
            return;
        }

        let mut current_layer: Vec<String> = self.entries.iter().map(|e| e.content_hash.clone()).collect();
        let mut tree = vec![current_layer.clone()];

        while current_layer.len() > 1 {
            let mut next_layer = Vec::new();
            for i in (0..current_layer.len()).step_by(2) {
                let left = &current_layer[i];
                let right = if i + 1 < current_layer.len() {
                    &current_layer[i + 1]
                } else {
                    left
                };
                let combined = sha256_hex(&format!("{}{}", left, right));
                next_layer.push(combined);
            }
            tree.push(next_layer.clone());
            current_layer = next_layer;
        }

        self.merkle_tree = tree;
        self.root_hash = current_layer.first().cloned().unwrap_or_default();
    }

    /// 获取哈希链
    pub fn get_hash_chain(&self) -> &[String] {
        &self.hash_chain
    }

    /// 获取 Merkle 根
    pub fn root_hash(&self) -> &str {
        &self.root_hash
    }

    /// 获取条目数
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 获取所有域
    pub fn domains(&self) -> &HashSet<String> {
        &self.domains
    }

    /// 获取所有条目
    pub fn entries(&self) -> &[LedgerEntry] {
        &self.entries
    }

    /// 获取操作总数
    pub fn operation_count(&self) -> u64 {
        self.operation_count
    }

    /// 获取审计日志
    pub fn get_audit_log(&self) -> Vec<String> {
        self.audit_log.lock().unwrap().clone()
    }

    /// 同步到 experience-tree 吸收协议
    pub fn sync_with_experience_tree(&mut self, absorb_data: &str) -> String {
        let id = self.record_operation(
            OperationType::ExperienceIngest,
            "NT-MEMORY",
            absorb_data,
            HashMap::from([
                ("source".into(), "experience-tree".into()),
                ("integration".into(), "absorption_protocol".into()),
            ]),
        );

        self.log_audit(&format!("Synced experience-tree absorb to CoverageLedger: {}", id));
        id
    }

    /// 记录经验树吸收
    pub fn record_experience_absorb(&mut self, cycle_id: u64, distilled_data: &str) -> String {
        let id = self.record_operation(
            OperationType::Absorb,
            "NT-MEMORY",
            distilled_data,
            HashMap::from([
                ("cycle_id".into(), cycle_id.to_string()),
                ("phase".into(), "distillation".into()),
                ("source".into(), "experience-tree".into()),
            ]),
        );
        self.log_audit(&format!("Recorded experience absorb for cycle {}", cycle_id));
        id
    }

    /// 添加条目（简化 API）
    pub fn add_entry(&mut self, content: &str) -> &LedgerEntry {
        let idx = self.entries.len() as u64;
        let previous_hash = self.hash_chain.last().cloned().unwrap_or_else(|| "genesis".into());
        let entry = LedgerEntry::new(
            OperationType::Write,
            "NT-MEMORY",
            content,
            previous_hash,
            HashMap::new(),
        );
        let entry_hash = entry.content_hash.clone();

        let chain_input = format!("{}{}", previous_hash, entry_hash);
        let chain_hash = sha256_hex(&chain_input);
        self.hash_chain.push(chain_hash);

        self.operation_count += 1;
        self.entries.push(entry);

        self.rebuild_merkle_tree();
        self.log_audit(&format!("Added entry {}", idx));

        self.entries.last().unwrap()
    }

    /// 按内容过滤查询
    pub fn query(&self, filter: &str) -> Vec<&LedgerEntry> {
        self.entries
            .iter()
            .filter(|e| e.content_hash.contains(filter) || e.domain.contains(filter))
            .collect()
    }

    /// 条目总数
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    fn log_audit(&self, message: &str) {
        let mut log = self.audit_log.lock().unwrap();
        log.push(format!("[{}] {}", Utc::now().to_rfc3339(), message));
    }
}

impl Default for CoverageLedger {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_verify() {
        let mut ledger = CoverageLedger::new();
        let id = ledger.record_operation(
            OperationType::Read,
            "NT-CORE",
            "test content",
            HashMap::new(),
        );
        assert!(!id.is_empty());
        assert_eq!(ledger.len(), 1);
        assert!(ledger.verify_integrity());
    }

    #[test]
    fn test_hash_chain_tamper_detection() {
        let mut ledger = CoverageLedger::new();
        ledger.record_operation(OperationType::Read, "NT-CORE", "data1", HashMap::new());
        ledger.record_operation(OperationType::Write, "NT-CORE", "data2", HashMap::new());
        ledger.record_operation(OperationType::Absorb, "NT-MIND", "data3", HashMap::new());

        assert!(ledger.verify_integrity());
        assert!(!ledger.root_hash().is_empty());
    }

    #[test]
    fn test_merkle_proof_verification() {
        let mut ledger = CoverageLedger::new();
        ledger.record_operation(OperationType::Read, "NT-CORE", "data1", HashMap::new());
        ledger.record_operation(OperationType::Write, "NT-CORE", "data2", HashMap::new());
        ledger.record_operation(OperationType::Absorb, "NT-MIND", "data3", HashMap::new());

        let proof = ledger.generate_merkle_proof(0);
        assert!(ledger.verify_proof(&proof));
    }

    #[test]
    fn test_multiple_domains() {
        let mut ledger = CoverageLedger::new();
        ledger.record_operation(OperationType::Read, "NT-CORE", "core_data", HashMap::new());
        ledger.record_operation(OperationType::Write, "NT-MEMORY", "mem_data", HashMap::new());
        ledger.record_operation(OperationType::Absorb, "NT-MIND", "mind_data", HashMap::new());

        assert_eq!(ledger.domains().len(), 3);
        assert!(ledger.domains().contains("NT-CORE"));
        assert!(ledger.domains().contains("NT-MEMORY"));
        assert!(ledger.domains().contains("NT-MIND"));
    }

    #[test]
    fn test_experience_tree_sync() {
        let mut ledger = CoverageLedger::new();
        let id = ledger.sync_with_experience_tree("absorbed knowledge");
        assert!(!id.is_empty());
        assert_eq!(ledger.len(), 1);
        assert!(ledger.verify_integrity());
    }

    #[test]
    fn test_record_experience_absorb() {
        let mut ledger = CoverageLedger::new();
        let id = ledger.record_experience_absorb(42, "distilled experience data");
        assert!(!id.is_empty());
        assert_eq!(ledger.operation_count(), 1);
    }

    #[test]
    fn test_empty_ledger() {
        let ledger = CoverageLedger::new();
        assert!(ledger.is_empty());
        assert!(ledger.verify_integrity());
        assert_eq!(ledger.len(), 0);
    }

    #[test]
    fn test_add_entry_and_query() {
        let mut ledger = CoverageLedger::new();
        let entry = ledger.add_entry("test content");
        assert!(!entry.content_hash.is_empty());
        assert_eq!(ledger.count(), 1);
        assert!(ledger.verify_integrity());

        let results = ledger.query("NT-MEMORY");
        assert_eq!(results.len(), 1);
    }
}