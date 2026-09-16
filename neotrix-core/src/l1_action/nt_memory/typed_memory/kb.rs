#![forbid(unsafe_code)]

use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use super::entry::{ConflictOutcome, ConflictStrategy, TypedMemoryEntry};
use super::estate::MemoryEstate;
use super::forgetting::{ForgetAction, ForgetEvent, PolicyDrivenForgetting};
use super::multitier::{ColdStore, MemoryMultitier, Tier};

/// SQLite-backed typed memory store that coexists with existing KB.
#[derive(Debug)]
pub struct TypedMemoryStore {
    conn: Connection,
    cold: ColdStore,
    multitier: MemoryMultitier,
    forgetting: PolicyDrivenForgetting,
}

const TYPED_MEMORY_SCHEMA: &str = "
    CREATE TABLE IF NOT EXISTS typed_memory (
        id TEXT PRIMARY KEY,
        estate TEXT NOT NULL,
        content TEXT NOT NULL,
        confidence REAL NOT NULL DEFAULT 0.5,
        timestamp INTEGER NOT NULL,
        ttl INTEGER DEFAULT NULL,
        access_count INTEGER NOT NULL DEFAULT 0,
        metadata TEXT DEFAULT '{}',
        created_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
    );
    CREATE INDEX IF NOT EXISTS idx_typed_memory_estate ON typed_memory(estate);
    CREATE INDEX IF NOT EXISTS idx_typed_memory_confidence ON typed_memory(confidence);
    CREATE INDEX IF NOT EXISTS idx_typed_memory_timestamp ON typed_memory(timestamp);
    CREATE TABLE IF NOT EXISTS typed_memory_conflicts (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        entry_id TEXT NOT NULL,
        conflicting_id TEXT NOT NULL,
        strategy TEXT NOT NULL,
        outcome TEXT NOT NULL,
        resolved_at INTEGER NOT NULL,
        FOREIGN KEY (entry_id) REFERENCES typed_memory(id)
    );
    CREATE TABLE IF NOT EXISTS typed_memory_cold_archive (
        id TEXT PRIMARY KEY, estate TEXT NOT NULL, content TEXT NOT NULL,
        confidence REAL NOT NULL, timestamp INTEGER NOT NULL,
        ttl INTEGER DEFAULT NULL, access_count INTEGER NOT NULL,
        compressed_summary TEXT DEFAULT NULL,
        archived_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
    );
";

impl TypedMemoryStore {
    pub fn new(conn: Connection) -> Result<Self, String> {
        conn.execute_batch(TYPED_MEMORY_SCHEMA)
            .map_err(|e| e.to_string())?;
        let cold = ColdStore::new(None).map_err(|e| e.to_string())?;
        let multitier = MemoryMultitier::new(50, 500).map_err(|e| e.to_string())?;
        let forgetting = PolicyDrivenForgetting::new();
        Ok(Self {
            conn,
            cold,
            multitier,
            forgetting,
        })
    }

    pub fn insert_entry(&mut self, entry: &TypedMemoryEntry) -> Result<(), String> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| e.to_string())?;
        tx.execute("INSERT OR REPLACE INTO typed_memory (id, estate, content, confidence, timestamp, ttl, access_count, metadata) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)", params![entry.id, entry.estate.as_str(), entry.content, entry.confidence, entry.timestamp, entry.ttl, entry.access_count, serde_json::to_string(&entry.conflicts).unwrap_or_default()]).map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        let tier = self.multitier.insert(entry.clone());
        if matches!(tier, Tier::Cold) {
            let _ = self.cold.insert(entry);
        }
        Ok(())
    }

    pub fn get_entry(&mut self, id: &str) -> Option<TypedMemoryEntry> {
        if let Some(entry) = self.multitier.get(id) {
            return Some(entry.clone());
        }
        let result = self.conn.query_row("SELECT id, estate, content, confidence, timestamp, ttl, access_count FROM typed_memory WHERE id=?1", params![id], |row| {
            let estate_str: String = row.get(1)?;
            Ok(TypedMemoryEntry { id: row.get(0)?, estate: MemoryEstate::from_str(&estate_str), content: row.get(2)?, confidence: row.get(3)?, timestamp: row.get(4)?, ttl: row.get::<_, Option<i64>>(5)?, conflicts: Vec::new(), access_count: row.get(6)? })
        }).ok();
        result
    }

    pub fn get_by_estate(&self, estate: MemoryEstate) -> Vec<TypedMemoryEntry> {
        let mut stmt = self.conn.prepare(&format!("SELECT id, estate, content, confidence, timestamp, ttl, access_count FROM typed_memory WHERE estate='{}'", estate.as_str())).unwrap();
        let rows = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let estate_str: String = row.get(1)?;
                let content: String = row.get(2)?;
                let confidence: f64 = row.get(3)?;
                let timestamp: i64 = row.get(4)?;
                let ttl: Option<i64> = row.get(5)?;
                let access_count: u64 = row.get(6)?;
                Ok(TypedMemoryEntry {
                    id,
                    estate: MemoryEstate::from_str(&estate_str),
                    content,
                    confidence,
                    timestamp,
                    ttl,
                    conflicts: Vec::new(),
                    access_count,
                })
            })
            .unwrap();
        rows.filter_map(|r| r.ok()).collect()
    }

    pub fn record_conflict(
        &self,
        entry_id: &str,
        conflicting_id: &str,
        strategy: ConflictStrategy,
        outcome: ConflictOutcome,
    ) -> Result<(), String> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| e.to_string())?;
        tx.execute("INSERT INTO typed_memory_conflicts (entry_id, conflicting_id, strategy, outcome, resolved_at) VALUES (?1, ?2, ?3, ?4, ?5)", params![entry_id, conflicting_id, strategy.as_str(), match outcome { ConflictOutcome::Resolved => "resolved", ConflictOutcome::Superseded => "superseded", ConflictOutcome::Merged => "merged", ConflictOutcome::Pending => "pending" }, SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64]).map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn resolve_conflicts(
        &mut self,
        entries: &mut Vec<TypedMemoryEntry>,
        strategy: ConflictStrategy,
    ) -> Vec<String> {
        use super::conflict::ConflictResolver;
        let mut resolver = ConflictResolver::new(strategy);
        let resolutions = resolver.resolve_all(entries);
        let mut resolved_ids = Vec::new();
        for resolution in &resolutions {
            resolved_ids.push(resolution.winner_id.clone());
            if let Some(loser_id) = &resolution.loser_id {
                if self
                    .record_conflict(
                        &resolution.winner_id,
                        loser_id,
                        strategy,
                        ConflictOutcome::Resolved,
                    )
                    .is_ok()
                {
                    let _ = self.delete_entry(loser_id);
                }
            }
        }
        resolved_ids
    }

    pub fn delete_entry(&self, id: &str) -> Result<bool, String> {
        let count = self
            .conn
            .execute("DELETE FROM typed_memory WHERE id=?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(count > 0)
    }

    pub fn apply_forgetting(&mut self) -> Vec<ForgetEvent> {
        let entries = self.get_all_entries();
        let mut events = Vec::new();
        for mut entry in entries {
            let actions = self.forgetting.apply_all(&mut entry);
            if actions.contains(&ForgetAction::Forgiven) || actions.contains(&ForgetAction::Decayed)
            {
                if self.delete_entry(&entry.id).is_ok() {
                    events.push(ForgetEvent {
                        entry_id: entry.id,
                        action: ForgetAction::Forgiven,
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64,
                        reason: "Forgetting policy removed entry".to_string(),
                    });
                }
            } else if actions.contains(&ForgetAction::Compressed) {
                events.push(ForgetEvent {
                    entry_id: entry.id,
                    action: ForgetAction::Compressed,
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs() as i64,
                    reason: "Compressed entry".to_string(),
                });
            }
        }
        events
    }

    pub fn get_all_entries(&self) -> Vec<TypedMemoryEntry> {
        let mut stmt = self.conn.prepare("SELECT id, estate, content, confidence, timestamp, ttl, access_count FROM typed_memory").unwrap();
        let rows = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let estate_str: String = row.get(1)?;
                let content: String = row.get(2)?;
                let confidence: f64 = row.get(3)?;
                let timestamp: i64 = row.get(4)?;
                let ttl: Option<i64> = row.get(5)?;
                let access_count: u64 = row.get(6)?;
                Ok(TypedMemoryEntry {
                    id,
                    estate: MemoryEstate::from_str(&estate_str),
                    content,
                    confidence,
                    timestamp,
                    ttl,
                    conflicts: Vec::new(),
                    access_count,
                })
            })
            .unwrap();
        rows.filter_map(|r| r.ok()).collect()
    }

    pub fn stats(&self) -> TypedMemoryStats {
        let total = self
            .conn
            .query_row("SELECT COUNT(*) FROM typed_memory", [], |r| r.get(0))
            .unwrap_or(0);
        let mut by_estate = self
            .conn
            .prepare("SELECT estate, COUNT(*) FROM typed_memory GROUP BY estate")
            .unwrap();
        let estate_counts: Vec<(String, i64)> = by_estate
            .query_map([], |r| {
                Ok((r.get(0).unwrap_or_default(), r.get(1).unwrap_or(0)))
            })
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        let avg = self
            .conn
            .query_row("SELECT AVG(confidence) FROM typed_memory", [], |r| r.get(0))
            .unwrap_or(0.0);
        TypedMemoryStats {
            total_entries: total,
            by_estate: estate_counts,
            average_confidence: avg,
        }
    }

    pub fn migrate_from_kb(&mut self) -> Result<usize, String> {
        let total: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM procedural_memory", [], |r| r.get(0))
            .unwrap_or(0);
        if total == 0 {
            return Ok(0);
        }
        let mut rows = self.conn.prepare("SELECT id, skill_id, name, description, success_rate, execution_count, created_at FROM procedural_memory").map_err(|e| e.to_string())?;
        let mapped_rows = rows
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0).unwrap_or_default(),
                    r.get::<_, String>(1).unwrap_or_default(),
                    r.get::<_, String>(2).unwrap_or_default(),
                    r.get::<_, String>(3).unwrap_or_default(),
                    r.get::<_, f64>(4).unwrap_or(0.0),
                    r.get::<_, i64>(5).unwrap_or(0),
                    r.get::<_, String>(6).unwrap_or_default(),
                ))
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect::<Vec<_>>();
        drop(rows);
        let mut migrated = 0;
        for (id, _skill_id, name, description, success_rate, _exec, _created) in mapped_rows {
            let entry = TypedMemoryEntry::new(
                id,
                MemoryEstate::Procedural,
                format!("{}: {}", name, description),
                success_rate.min(1.0).max(0.0),
            );
            if self.insert_entry(&entry).is_ok() {
                migrated += 1;
            }
        }
        Ok(migrated)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TypedMemoryStats {
    pub total_entries: i64,
    pub by_estate: Vec<(String, i64)>,
    pub average_confidence: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    fn make_store() -> TypedMemoryStore {
        let dir = tempdir().unwrap();
        let conn = Connection::open(dir.path().join("test.db")).unwrap();
        TypedMemoryStore::new(conn).unwrap()
    }
    fn make_entry(estate: MemoryEstate) -> TypedMemoryEntry {
        TypedMemoryEntry::new(
            format!("test-{}", estate.as_str()),
            estate,
            format!("content for {}", estate.as_str()),
            0.85,
        )
    }
    #[test]
    fn test_store_create_insert() {
        let mut s = make_store();
        let e = make_entry(MemoryEstate::Semantic);
        s.insert_entry(&e).unwrap();
        assert!(s.get_entry(&e.id).is_some());
    }
    #[test]
    fn test_store_get_by_estate() {
        let mut s = make_store();
        s.insert_entry(&make_entry(MemoryEstate::Procedural))
            .unwrap();
        s.insert_entry(&make_entry(MemoryEstate::Episodic)).unwrap();
        assert_eq!(s.get_by_estate(MemoryEstate::Procedural).len(), 1);
    }
    #[test]
    fn test_store_delete() {
        let mut s = make_store();
        let e = make_entry(MemoryEstate::Working);
        s.insert_entry(&e).unwrap();
        assert!(s.delete_entry(&e.id).unwrap());
        assert!(s.get_entry(&e.id).is_none());
    }
    #[test]
    fn test_store_record_conflict() {
        let mut s = make_store();
        let e = make_entry(MemoryEstate::Semantic);
        s.insert_entry(&e).unwrap();
        assert!(s
            .record_conflict(
                &e.id,
                "other",
                ConflictStrategy::LatestWins,
                ConflictOutcome::Resolved
            )
            .is_ok());
    }
    #[test]
    fn test_store_stats() {
        let mut s = make_store();
        s.insert_entry(&make_entry(MemoryEstate::Semantic)).unwrap();
        s.insert_entry(&make_entry(MemoryEstate::Working)).unwrap();
        let stats = s.stats();
        assert_eq!(stats.total_entries, 2);
    }
    #[test]
    fn test_store_get_all() {
        let mut s = make_store();
        s.insert_entry(&make_entry(MemoryEstate::Procedural))
            .unwrap();
        s.insert_entry(&make_entry(MemoryEstate::Episodic)).unwrap();
        assert_eq!(s.get_all_entries().len(), 2);
    }
    #[test]
    fn test_migrate_from_kb() {
        let mut s = make_store();
        let result = s.migrate_from_kb();
        assert!(result.is_ok());
    }
}
