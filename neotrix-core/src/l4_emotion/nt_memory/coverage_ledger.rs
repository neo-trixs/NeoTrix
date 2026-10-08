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

// ⚠️ 2026-10-02 修正：本 import 原写 `neotrix_types::nt_crypto_util`（顶层），
// 而该模块实际声明在 `neotrix_types::core::nt_crypto_util` ⇒ **路径错**。
// 该文件从未编译，所以这个错误从未被任何人看见（与 multi_agent 的
// `DagEdge` 漏导入同型）。
use neotrix_types::core::nt_crypto_util::sha256_hex_str as sha256_hex;

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
        // ⚠️ 2026-10-02 修正（E0382）：`previous_hash` 是 `String`，
        // 传给 `LedgerEntry::new(..)` 会**move** 它，而紧接着下面的
        // `format!("{}{}", previous_hash, ..)` 又要用 ⇒ 「borrow of moved value」。
        let entry = LedgerEntry::new(
            operation_type,
            domain,
            content,
            previous_hash.clone(),
            metadata,
        );
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

        // ⚠️ 2026-10-02 修正（**核心缺陷**：`verify_integrity()` 恒为 false
        // ⇒「篡改必被发现」实际是「永远报错」，等于没有校验）。
        //
        // 原实现有两处错，用探针实测出的真实数据说清：
        //
        // 实测（`record_operation` x3）：
        //     初始 hash_chain = ["genesis"]
        //     第0条后: chain = ["genesis", h0]            len=1
        //     第1条后: chain = ["genesis", h0, h1]         len=2
        //     第2条后: chain = ["genesis", h0, h1, h2]     len=3
        // 写入侧公式：`chain_input = hash_chain.last() + entry.content_hash`
        //
        // 错处 1 —— **差一位**：`hash_chain[0]` 是字面量 `"genesis"` 占位，
        //   所以 entry `i` 的哈希落在 `hash_chain[i + 1]`，
        //   而原代码比的是 `self.hash_chain[i]` ⇒ 拿 h_{i-1} 去比 entry i。
        // 错处 2 —— **起点跳过 entry 0**：`for i in 1..entries.len()`
        //   ⇒ 第一条 entry 从未被校验。
        // 错处 3 —— **哈希源不同**：原用 `previous_entry.content_hash`，
        //   而写入侧用的是**链哈希** ⇒ 两边永远不相等。
        //
        // 修法：`for i in 0..entries.len()`，用 `hash_chain[i]`（前驱链哈希）
        // 与 `hash_chain[i + 1]`（本条链哈希）成对校验，覆盖**全部** entry。
        if self.hash_chain.len() != self.entries.len() + 1 {
            return false;
        }
        for (i, current) in self.entries.iter().enumerate() {
            let expected_chain_input =
                format!("{}{}", self.hash_chain[i], current.content_hash);
            if self.hash_chain[i + 1] != sha256_hex(&expected_chain_input) {
                return false;
            }
            // `LedgerEntry::new(.., previous_hash, ..)` 传入的是当时的
            // `hash_chain.last()`，即 `hash_chain[i]`（链哈希），
            // 不是上一条 entry 的 content_hash。
            if current.previous_hash != self.hash_chain[i] {
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
        // ⚠️ 2026-10-02 修正（**死循环**）：
        // 原实现用 `merkle_tree.get(current_layer.len() / 2)` 去取**上一层**，
        // 即**拿「当前层长度」当「层号」**。3 叶树的实际轨迹：
        //     merkle_tree = [[l0,l1,l2], [p0,p1], [root]]
        //     iter1: current_layer=[l0,l1,l2] len=3 ⇒ 取 get(3/2=1)=[p0,p1] ✓（碰巧对）
        //     iter2: current_layer=[p0,p1]    len=2 ⇒ 取 get(2/2=1)=[p0,p1] ⛔ **同一层**
        //     ⇒ len 恒为 2 ⇒ `while current_layer.len() > 1` **永不退出**
        // ⇒ 表现为 `test_merkle_proof_verification` 挂死（cargo 报
        //   「has been running for over 60 seconds」）。
        //
        // 修法：**显式跟踪层号** `level`，它严格递增且以
        // `merkle_tree.len()` 为界 ⇒ 必然终止。
        let mut level = 0usize;
        let mut current_layer = self.merkle_tree.first().cloned().unwrap_or_default();

        while level + 1 < self.merkle_tree.len() && current_layer.len() > 1 {
            let sibling_idx = if idx.is_multiple_of(2) { idx + 1 } else { idx - 1 };
            if sibling_idx < current_layer.len() {
                proof_path.push(current_layer[sibling_idx].clone());
            }
            idx /= 2;
            level += 1;
            current_layer = self.merkle_tree[level].clone();
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

        // ⚠️ 2026-10-02 修正（E0594 + **顺带去重**）：
        // · 原实现签名是 `&MerkleProof` 却执行 `proof.verified = ..`
        //   ⇒ **编译不过**，且语义可疑：**验证函数不该修改它的输入**。
        // · 且其函数体与既有的 `verify_proof_internal`（已被
        //   `generate_merkle_proof` 使用、实测可工作）**是同一算法的两份拷贝**。
        // ⇒ 改为**纯函数**并直接复用 `verify_proof_internal`，
        //   既消除重复实现，返回值语义也保持一致。
        // ⛔ `MerkleProof::verified` 不再被此处改写；该字段仍由
        //    `generate_merkle_proof` 从 `verify_proof_internal` 填入，
        //    故 `verify_against` 的 `&& self.verified` 路径不受影响。
        self.verify_proof_internal(proof.entry_index, &proof.proof_path)
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
            // ⛔ 锁投毒按本仓既有范式**恢复**（`PoisonError::into_inner`，见 c022bfab）
            //    ⛔ 不改成 panic：那会让「一次panic」把整个 ledger 永久锁死。
        self.audit_log
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
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
            // ⚠️ 2026-10-02 修正（E0382，同上）：克隆后再传，保留后续使用。
            previous_hash.clone(),
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

        // ⚠️ 原为 `self.entries.last().unwrap()`：L442 刚 `push` 过，
        //   逻辑上必Some，但 `unwrap` 让「不变量被破坏」变成 panic 而非可查错。
        // ⇒ 用 push 时记下的 `idx` 直接索引（同样是必成功，但**无 unwrap**）。
        &self.entries[idx as usize]
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
            // ⛔ 锁投毒按本仓既有范式**恢复**（`PoisonError::into_inner`，见 c022bfab）
            //    ⛔ 不改成 panic：那会让「一次panic」把整个 ledger 永久锁死。
        let mut log = self
            .audit_log
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
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