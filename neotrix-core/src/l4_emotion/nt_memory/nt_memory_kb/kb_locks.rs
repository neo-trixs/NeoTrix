//! kb_locks — 从 `nt_memory_kb/mod.rs` 拆分 (行为零变更).
//! `impl KnowledgeBase` 按域搬运, 原文逐行, 仅补可见性/导入。


use super::KnowledgeBase;
use super::nt_memory_commitment;
use super::shared_utils;
use fs2::FileExt;
use std::sync::atomic::Ordering;

impl KnowledgeBase {
    /// Acquire an exclusive file lock before a write operation.
    /// 使用 AtomicBool 跟踪锁状态，避免 macOS flock() 不可重入死锁。
    pub(crate) fn lock_before_write(&self) -> std::io::Result<()> {
        if self.file_lock_held.load(Ordering::Relaxed) {
            // 锁已持有，跳过 (flock 不可重入)
            return Ok(());
        }
        if let Some(ref f) = self.db_file {
            f.lock_exclusive()?;
            self.file_lock_held.store(true, Ordering::Relaxed);
        }
        Ok(())
    }

    /// Release the file lock after a write operation.
    pub(crate) fn unlock_after_write(&self) -> std::io::Result<()> {
        if !self.file_lock_held.load(Ordering::Relaxed) {
            return Ok(());
        }
        if let Some(ref f) = self.db_file {
            f.unlock()?;
            self.file_lock_held.store(false, Ordering::Relaxed);
        }
        Ok(())
    }

    // ── Embedding Commitment ──

    pub fn store_commitment(
        &self,
        node_id: String,
        vector: &[f32],
        model_name: String,
    ) -> Result<nt_memory_commitment::EmbeddingCommitment, String> {
        let mut store = self
            .commitment_store
            .write()
            .map_err(|e| format!("Lock: {}", e))?;
        store.commit_vector(node_id, vector, model_name, "neotrix".to_string())
    }

    pub fn verify_commitment(
        &self,
        node_id: &str,
        vector: &[f32],
    ) -> Result<nt_memory_commitment::CommitmentProof, String> {
        let store = self
            .commitment_store
            .read()
            .map_err(|e| format!("Lock: {}", e))?;
        store.verify_commitment(node_id, vector)
    }

    pub fn persist_commitments(&self) -> Result<(), String> {
        let store = self
            .commitment_store
            .read()
            .map_err(|e| format!("Lock: {}", e))?;
        shared_utils::save_kv_state(self, "commitment", "store", &*store)
    }

    pub fn load_commitments(&self) -> Result<(), String> {
        let loaded: Option<nt_memory_commitment::EmbeddingCommitmentStore> =
            shared_utils::load_kv_state(self, "commitment", "store")?;
        if let Some(data) = loaded {
            let mut store = self
                .commitment_store
                .write()
                .map_err(|e| format!("Lock: {}", e))?;
            *store = data;
        }
        Ok(())
    }

    // ── Confidence Store ──
}
