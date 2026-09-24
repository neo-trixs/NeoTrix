//! kb_evolution — 从 `nt_memory_kb/mod.rs` 拆分 (行为零变更).
//! `impl KnowledgeBase` 按域搬运, 原文逐行, 仅补可见性/导入。


use super::KnowledgeBase;
use super::nt_memory_integration;
use super::{ConversationRecord, EvolutionPatternType, EvolutionRecord, UserMemory};

impl KnowledgeBase {
    pub fn save_user_memory(&self, um: &UserMemory) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        um.save(&conn)
    }

    pub fn load_user_memory(&self, user_id: &str) -> Option<UserMemory> {
        let conn = self.conn.lock().ok()?;
        UserMemory::load(&conn, user_id)
    }

    // ── Integration (WebMiner persist) ──

    pub fn persist_mined(
        &self,
        title: &str,
        summary: &str,
        url: &str,
        source_type: &str,
        confidence: f64,
        edits: &[(String, f64)],
        insights: &[String],
    ) -> Result<String, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_integration::persist_mined_knowledge(
            &conn,
            title,
            summary,
            url,
            source_type,
            confidence,
            edits,
            insights,
        )
    }

    // ─── GWT queries (wired in nt_memory_gwtq.rs) ───
    // query_by_e8_state, query_by_specialist, record_consciousness_snapshot,
    // recommend_for_e8_mode, query_broadcast_context are in nt_memory_gwtq.rs

    // ── Evolution / Conversation ──

    pub fn get_evolution_history(&self, limit: usize) -> Result<Vec<ConversationRecord>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, task_description, user_intent, strategy_used, e8_mode,
                    specialist_winner, actions_taken, obstacles_encountered, fix_patterns,
                    outcome, effectiveness, reasoning_iterations, error_count, timestamp
             FROM conversation_records
             ORDER BY timestamp DESC
             LIMIT ?1",
            )
            .map_err(|e| format!("prepare: {}", e))?;
        let rows = stmt
            .query_map([limit as i64], |row| {
                Ok(ConversationRecord {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    task_description: row.get(2)?,
                    user_intent: row.get(3)?,
                    strategy_used: row.get(4)?,
                    e8_mode: row.get(5)?,
                    specialist_winner: row.get(6)?,
                    actions_taken: serde_json::from_str(&row.get::<_, String>(7)?)
                        .unwrap_or_default(),
                    obstacles_encountered: serde_json::from_str(&row.get::<_, String>(8)?)
                        .unwrap_or_default(),
                    fix_patterns: serde_json::from_str(&row.get::<_, String>(9)?)
                        .unwrap_or_default(),
                    outcome: row.get(10)?,
                    effectiveness: row.get(11)?,
                    reasoning_iterations: row.get(12)?,
                    error_count: row.get(13)?,
                    timestamp: row.get(14)?,
                })
            })
            .map_err(|e| format!("query_map: {}", e))?;
        let mut records = Vec::new();
        for row in rows {
            records.push(row.map_err(|e| format!("row: {}", e))?);
        }
        Ok(records)
    }

    pub fn store_evolution_record(&self, record: &EvolutionRecord) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        conn.execute(
            "INSERT INTO evolution_records
             (id, source_conversation_id, pattern_type, description, before_behavior,
              after_behavior, effectiveness_gain, applied_to, verified, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                record.id,
                record.source_conversation_id,
                format!("{:?}", record.pattern_type),
                record.description,
                record.before_behavior,
                record.after_behavior,
                record.effectiveness_gain,
                serde_json::to_string(&record.applied_to).unwrap_or_default(),
                record.verified,
                record.timestamp,
            ],
        )
        .map_err(|e| format!("store_evolution_record: {}", e))?;
        Ok(())
    }

    pub fn store_conversation_record(&self, record: &ConversationRecord) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        conn.execute(
            "INSERT INTO conversation_records
             (id, session_id, task_description, user_intent, strategy_used, e8_mode,
              specialist_winner, actions_taken, obstacles_encountered, fix_patterns,
              outcome, effectiveness, reasoning_iterations, error_count, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            rusqlite::params![
                record.id,
                record.session_id,
                record.task_description,
                record.user_intent,
                record.strategy_used,
                record.e8_mode,
                record.specialist_winner,
                serde_json::to_string(&record.actions_taken).unwrap_or_default(),
                serde_json::to_string(&record.obstacles_encountered).unwrap_or_default(),
                serde_json::to_string(&record.fix_patterns).unwrap_or_default(),
                record.outcome,
                record.effectiveness,
                record.reasoning_iterations,
                record.error_count,
                record.timestamp,
            ],
        )
        .map_err(|e| format!("store_conversation_record: {}", e))?;
        Ok(())
    }

    pub fn get_evolution_patterns(&self, limit: usize) -> Result<Vec<EvolutionRecord>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, source_conversation_id, pattern_type, description, before_behavior,
                    after_behavior, effectiveness_gain, applied_to, verified, timestamp
             FROM evolution_records
             ORDER BY timestamp DESC
             LIMIT ?1",
            )
            .map_err(|e| format!("prepare: {}", e))?;
        let rows = stmt
            .query_map([limit as i64], |row| {
                Ok(EvolutionRecord {
                    id: row.get(0)?,
                    source_conversation_id: row.get(1)?,
                    pattern_type: EvolutionPatternType::from_str(&row.get::<_, String>(2)?),
                    description: row.get(3)?,
                    before_behavior: row.get(4)?,
                    after_behavior: row.get(5)?,
                    effectiveness_gain: row.get(6)?,
                    applied_to: serde_json::from_str(&row.get::<_, String>(7)?).unwrap_or_default(),
                    verified: row.get(8)?,
                    timestamp: row.get(9)?,
                })
            })
            .map_err(|e| format!("query_map: {}", e))?;
        let mut records = Vec::new();
        for row in rows {
            records.push(row.map_err(|e| format!("row: {}", e))?);
        }
        Ok(records)
    }

    // ── Trace data (anti-distillation persistence) ──

    pub fn store_trace_data(&self, data: &serde_json::Value) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        conn.execute(
            "INSERT INTO trace_data (data_json, created_at) VALUES (?1, ?2)",
            rusqlite::params![
                data.to_string(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0),
            ],
        )
        .map_err(|e| format!("store_trace_data: {}", e))?;
        Ok(())
    }

    pub fn get_trace_data(&self, limit: usize) -> Result<Vec<serde_json::Value>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let mut stmt = conn
            .prepare("SELECT data_json FROM trace_data ORDER BY created_at DESC LIMIT ?1")
            .map_err(|e| format!("prepare: {}", e))?;
        let rows = stmt
            .query_map([limit as i64], |row| {
                let s: String = row.get(0)?;
                Ok(serde_json::from_str(&s).unwrap_or(serde_json::Value::Null))
            })
            .map_err(|e| format!("query_map: {}", e))?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| format!("row: {}", e))?);
        }
        Ok(results)
    }

    // ── Learning report ──

    pub fn store_learning_report(&self, report: &serde_json::Value) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        conn.execute(
            "INSERT INTO learning_reports (report_json, created_at) VALUES (?1, ?2)",
            rusqlite::params![
                report.to_string(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0),
            ],
        )
        .map_err(|e| format!("store_learning_report: {}", e))?;
        Ok(())
    }
}
