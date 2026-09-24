//! kb_kv — 从 `nt_memory_kb/mod.rs` 拆分 (行为零变更).
//! `impl KnowledgeBase` 按域搬运, 原文逐行, 仅补可见性/导入。


use super::KnowledgeBase;
use super::nt_field_ledger;
use super::nt_memory_unify;

impl KnowledgeBase {
    pub fn kv_get(&self, namespace: &str, key: &str) -> Result<Option<String>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_unify::kv_get(&conn, namespace, key)
    }

    pub fn kv_set(&self, namespace: &str, key: &str, value: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_unify::kv_set(&conn, namespace, key, value)
    }

    pub fn kv_delete(&self, namespace: &str, key: &str) -> Result<bool, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_unify::kv_delete(&conn, namespace, key)
    }

    pub fn kv_list(&self, namespace: &str) -> Result<Vec<(String, String)>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_unify::kv_list(&conn, namespace)
    }

    /// 读取 experience-tree 吸收的经验条目, 供门控校准 (CalibrationSet::from_kb_experience)。
    pub fn experience_entries(&self) -> Result<Vec<(String, String)>, String> {
        self.kv_list("experience")
    }

    // ── G1 场化地基: 版本化场写入 (灵境协议6 转译, 2026-08-26 吸收, R-P79) ──

    /// 投递一条 ΔJ 到场暂存区 (不立即改变正式状态)。
    pub fn field_stage(
        &self,
        ns: &str,
        key: &str,
        value: &str,
        writer: &str,
    ) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_field_ledger::field_stage(&conn, ns, key, value, writer)
    }

    /// 统一求解下一版本 (任何写者可调用; 空集幂等返回 None)。
    pub fn field_tick(&self) -> Result<Option<nt_field_ledger::FieldReceipt>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_field_ledger::field_tick(&conn)
    }

    /// 当前场版本号。
    pub fn field_version(&self) -> Result<u64, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_field_ledger::field_version(&conn)
    }

    /// 整链回放校验 (审计: 任何篡改返回 false)。
    pub fn field_verify_chain(&self) -> Result<bool, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_field_ledger::field_verify_chain(&conn)
    }

    /// G4: 各写者已参与求解的最高版本游标 (未入链的写者不出现)。
    pub fn field_writer_cursors(&self) -> Result<Vec<(String, u64)>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_field_ledger::field_writer_cursors(&conn)
    }

    /// G4: 多锚点共识帧 (head / quorum / 各写者滞后量)。
    pub fn field_consensus_frame(&self) -> Result<nt_field_ledger::ConsensusFrame, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_field_ledger::consensus_frame(&conn)
    }
}
