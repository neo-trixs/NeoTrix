use crate::core::nt_core_error::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRoom {
    pub id: usize,
    pub name: String,
    pub description: String,
    pub center_vsa: Vec<u8>,
    pub radius: f64,
    pub created_at: u64,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PalaceEntry {
    pub id: u64,
    pub room_id: Option<usize>,
    pub content: String,
    pub vsa_hash: Vec<u8>,
    pub timestamp: u64,
    pub access_count: u64,
    pub last_accessed: u64,
    pub significance: f64,
    pub source_layer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PalaceSnapshot {
    pub rooms: Vec<MemoryRoom>,
    pub entries: Vec<PalaceEntry>,
    pub current_room_id: Option<usize>,
    pub next_id: u64,
    pub cycle: u64,
}

impl PalaceSnapshot {
    /// Compute a SHA-256 integrity hash over the snapshot state.
    pub fn compute_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&self.rooms.len().to_le_bytes());
        hasher.update(&self.entries.len().to_le_bytes());
        hasher.update(&self.current_room_id.unwrap_or(usize::MAX).to_le_bytes());
        hasher.update(&self.next_id.to_le_bytes());
        hasher.update(&self.cycle.to_le_bytes());
        for entry in &self.entries {
            hasher.update(&entry.vsa_hash);
        }
        let result = hasher.finalize();
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&result);
        arr
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPalace {
    pub rooms: Vec<MemoryRoom>,
    pub entries: Vec<PalaceEntry>,
    pub current_room_id: Option<usize>,
    pub next_id: u64,
    pub cycle: u64,

    #[serde(skip)]
    room_name_index: HashMap<String, usize>,
}

impl Default for MemoryPalace {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryPalace {
    pub fn new() -> Self {
        let mut palace = Self {
            rooms: Vec::new(),
            entries: Vec::new(),
            current_room_id: None,
            next_id: 1,
            cycle: 0,
            room_name_index: HashMap::new(),
        };
        palace.init_default_rooms();
        palace
    }

    fn init_default_rooms(&mut self) {
        let defaults = [
            ("curiosity", "好奇心 — 探索未知的驱动空间", "#a855f7"),
            ("epistemic", "认知 — 知识与信念的存储穹顶", "#3b82f6"),
            ("social", "社会 — 交互与关系的映射回廊", "#22c55e"),
            ("self", "自我 — 身份与叙事的核心殿堂", "#f59e0b"),
            ("reasoning", "推理 — 因果与逻辑的演算大厅", "#ef4444"),
            ("dream", "梦境 — 无意识整合的潜意识室", "#6366f1"),
            ("wisdom", "反思 — 蒸馏洞察与长期智慧的存储圣殿", "#d946ef"),
            (
                "patterns",
                "模式 — 循环模式与知识结构检测的归档厅",
                "#06b6d4",
            ),
        ];
        for (name, desc, color) in &defaults {
            use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            name.hash(&mut hasher);
            let seed = hasher.finish();
            let center = QuantizedVSA::seeded_random(seed, 4096);
            self.rooms.push(MemoryRoom {
                id: self.rooms.len(),
                name: name.to_string(),
                description: desc.to_string(),
                center_vsa: center,
                radius: 0.35,
                created_at: 0,
                color: color.to_string(),
            });
            self.room_name_index
                .insert(name.to_string(), self.rooms.len() - 1);
        }
    }

    pub fn create_room(
        &mut self,
        name: &str,
        description: &str,
        center_vsa: Vec<u8>,
        color: &str,
    ) -> usize {
        if self.room_name_index.contains_key(name) {
            return self.room_name_index[name];
        }
        let id = self.rooms.len();
        self.rooms.push(MemoryRoom {
            id,
            name: name.to_string(),
            description: description.to_string(),
            center_vsa,
            radius: 0.35,
            created_at: self.cycle,
            color: color.to_string(),
        });
        self.room_name_index.insert(name.to_string(), id);
        id
    }

    pub fn walk_to(&mut self, room_id: usize) -> bool {
        if room_id < self.rooms.len() {
            self.current_room_id = Some(room_id);
            true
        } else {
            false
        }
    }

    pub fn walk_to_by_name(&mut self, name: &str) -> bool {
        if let Some(&id) = self.room_name_index.get(name) {
            self.current_room_id = Some(id);
            true
        } else {
            false
        }
    }

    pub fn current_room(&self) -> Option<&MemoryRoom> {
        self.current_room_id.and_then(|id| self.rooms.get(id))
    }

    pub fn enter(&mut self, content: String, vsa_hash: Vec<u8>, source_layer: &str) -> u64 {
        let entry = PalaceEntry {
            id: self.next_id,
            room_id: self.current_room_id,
            content,
            vsa_hash,
            timestamp: self.cycle,
            access_count: 0,
            last_accessed: self.cycle,
            significance: 0.5,
            source_layer: source_layer.to_string(),
        };
        self.next_id += 1;
        self.entries.push(entry);
        self.next_id - 1
    }

    pub fn find_room_for(&self, vsa_hash: &[u8]) -> Option<usize> {
        use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
        let mut best_sim = 0.0f64;
        let mut best_id = None;
        for room in &self.rooms {
            let sim = QuantizedVSA::similarity(vsa_hash, &room.center_vsa);
            if sim > room.radius && sim > best_sim {
                best_sim = sim;
                best_id = Some(room.id);
            }
        }
        best_id
    }

    pub fn find_by_content(&self, query: &str) -> Vec<&PalaceEntry> {
        let q = query.to_lowercase();
        let mut results: Vec<&PalaceEntry> = self
            .entries
            .iter()
            .filter(|e| e.content.to_lowercase().contains(&q))
            .collect();
        results.sort_by(|a, b| {
            b.significance
                .partial_cmp(&a.significance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(20);
        results
    }

    pub fn find_by_vsa(&self, vsa_hash: &[u8], k: usize) -> Vec<&PalaceEntry> {
        use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
        let mut scored: Vec<(&PalaceEntry, f64)> = self
            .entries
            .iter()
            .map(|e| (e, QuantizedVSA::similarity(vsa_hash, &e.vsa_hash)))
            .filter(|(_, s)| *s > 0.4)
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(k);
        scored.into_iter().map(|(e, _)| e).collect()
    }

    pub fn time_travel(&self, start_cycle: u64, end_cycle: u64) -> Vec<&PalaceEntry> {
        let mut results: Vec<&PalaceEntry> = self
            .entries
            .iter()
            .filter(|e| e.timestamp >= start_cycle && e.timestamp <= end_cycle)
            .collect();
        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        results.truncate(50);
        results
    }

    pub fn nearest_rooms(&self, vsa_hash: &[u8], k: usize) -> Vec<&MemoryRoom> {
        use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
        let mut scored: Vec<(&MemoryRoom, f64)> = self
            .rooms
            .iter()
            .map(|r| (r, QuantizedVSA::similarity(vsa_hash, &r.center_vsa)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(k);
        scored.into_iter().map(|(r, _)| r).collect()
    }

    pub fn rooms_nearby(&self, k: usize) -> Vec<&MemoryRoom> {
        match self.current_room_id {
            Some(id) => {
                if let Some(current) = self.rooms.get(id) {
                    self.nearest_rooms(&current.center_vsa, k + 1)
                        .into_iter()
                        .filter(|r| r.id != id)
                        .take(k)
                        .collect()
                } else {
                    Vec::new()
                }
            }
            None => self.rooms.iter().take(k).collect(),
        }
    }

    pub fn access(&mut self, entry_id: u64) -> bool {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.id == entry_id) {
            entry.access_count += 1;
            entry.last_accessed = self.cycle;
            entry.significance = (entry.significance + 0.05).min(1.0);
            true
        } else {
            false
        }
    }

    pub fn decay(&mut self, factor: f64) {
        for entry in self.entries.iter_mut() {
            entry.significance *= factor;
        }
    }

    pub fn tick(&mut self) -> String {
        self.cycle += 1;
        let mut events = Vec::new();
        if self.cycle % 10 == 0 {
            self.decay(0.995);
            let p = self.prune_stale();
            if p > 0 {
                events.push(format!("prune:{}", p));
            }
        }
        if self.cycle % 50 == 0 {
            let diag = self.diagnostic();
            events.push(diag);
        }
        events.join("|")
    }

    fn prune_stale(&mut self) -> usize {
        let before = self.entries.len();
        self.entries.retain(|e| {
            let age = self.cycle.saturating_sub(e.last_accessed);
            age < 500 || e.significance > 0.2 || e.access_count > 2
        });
        before - self.entries.len()
    }

    pub fn recent_entries(&self, n: usize) -> Vec<&PalaceEntry> {
        let n = n.min(self.entries.len());
        let mut sorted: Vec<&PalaceEntry> = self.entries.iter().collect();
        sorted.sort_by(|a, b| b.id.cmp(&a.id));
        sorted.truncate(n);
        sorted
    }

    pub fn item_count(&self) -> usize {
        self.entries.len()
    }

    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }

    pub fn palace_map(&self) -> String {
        let mut lines = Vec::new();
        lines.push("┌─ Memory Palace ─────────────────┐".to_string());
        lines.push(format!(
            "│ Rooms: {}  Entries: {}           │",
            self.rooms.len(),
            self.entries.len()
        ));
        lines.push(format!(
            "│ Cycle: {}  Current: {}              │",
            self.cycle,
            self.current_room()
                .map(|r| &r.name)
                .unwrap_or(&"none".to_string())
                .to_string()
        ));
        lines.push("├─ Rooms ─────────────────────────┤".to_string());
        for room in &self.rooms {
            let entries = self
                .entries
                .iter()
                .filter(|e| e.room_id == Some(room.id))
                .count();
            let marker = if self.current_room_id == Some(room.id) {
                "◉"
            } else {
                "○"
            };
            lines.push(format!(
                "│ {} {:<12} ({}) {}   │",
                marker, room.name, entries, room.color
            ));
        }
        lines.push("└──────────────────────────────────┘".to_string());
        lines.join("\n")
    }

    pub fn diagnostic(&self) -> String {
        format!(
            "palace:rms={}|ents={}|cur={:?}",
            self.rooms.len(),
            self.entries.len(),
            self.current_room_id
        )
    }

    pub fn snapshot(&self) -> PalaceSnapshot {
        PalaceSnapshot {
            rooms: self.rooms.clone(),
            entries: self.entries.clone(),
            current_room_id: self.current_room_id,
            next_id: self.next_id,
            cycle: self.cycle,
        }
    }

    pub fn load_snapshot(&mut self, snap: PalaceSnapshot) {
        self.rooms = snap.rooms;
        self.entries = snap.entries;
        self.current_room_id = snap.current_room_id;
        self.next_id = snap.next_id;
        self.cycle = snap.cycle;
        self.room_name_index = self.rooms.iter().map(|r| (r.name.clone(), r.id)).collect();
    }

    pub fn save_to_json(&self, path: &std::path::Path) -> CoreResult<()> {
        let snap = self.snapshot();
        let json = serde_json::to_string_pretty(&snap)
            .map_err(|e| CoreError::Serde(format!("serialize: {}", e)))?;
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, &json).map_err(|e| CoreError::Io(std::sync::Arc::new(e)))?;
        std::fs::rename(&tmp, path).map_err(|e| CoreError::Io(std::sync::Arc::new(e)))?;
        Ok(())
    }

    pub fn load_from_json(path: &std::path::Path) -> CoreResult<Self> {
        let json =
            std::fs::read_to_string(path).map_err(|e| CoreError::Io(std::sync::Arc::new(e)))?;
        let snap: PalaceSnapshot = serde_json::from_str(&json)
            .map_err(|e| CoreError::Serde(format!("deserialize: {}", e)))?;
        let mut palace = Self::new();
        palace.load_snapshot(snap);
        Ok(palace)
    }

    /// Save with integrity hash: writes a sidecar `.sha256` file.
    pub fn save_with_integrity(&self, path: &std::path::Path) -> CoreResult<String> {
        let snap = self.snapshot();
        let hash_hex = hex::encode(snap.compute_hash());
        let json = serde_json::to_string_pretty(&snap)
            .map_err(|e| CoreError::Serde(format!("serialize: {}", e)))?;
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, &json).map_err(|e| CoreError::Io(std::sync::Arc::new(e)))?;
        std::fs::rename(&tmp, path).map_err(|e| CoreError::Io(std::sync::Arc::new(e)))?;
        let hash_path = path.with_extension("json.sha256");
        let hash_tmp = std::path::PathBuf::from(format!("{}.tmp", hash_path.display()));
        std::fs::write(&hash_tmp, &hash_hex).map_err(|e| CoreError::Io(std::sync::Arc::new(e)))?;
        std::fs::rename(&hash_tmp, hash_path).map_err(|e| CoreError::Io(std::sync::Arc::new(e)))?;
        Ok(hash_hex)
    }

    /// Load and verify integrity hash.
    pub fn load_with_integrity(path: &std::path::Path) -> CoreResult<(Self, bool)> {
        let json =
            std::fs::read_to_string(path).map_err(|e| CoreError::Io(std::sync::Arc::new(e)))?;
        let snap: PalaceSnapshot = serde_json::from_str(&json)
            .map_err(|e| CoreError::Serde(format!("deserialize: {}", e)))?;
        let computed_hash = snap.compute_hash();
        let hash_path = path.with_extension("json.sha256");
        let stored_hash = std::fs::read_to_string(&hash_path).unwrap_or_default();
        let valid = hex::encode(computed_hash) == stored_hash.trim();
        if !valid {
            log::warn!("MemoryPalace integrity check FAILED for {}", path.display());
        }
        let mut palace = Self::new();
        palace.load_snapshot(snap);
        Ok((palace, valid))
    }

    /// Verify integrity of an existing JSON file without loading its contents.
    pub fn verify_integrity(path: &std::path::Path) -> CoreResult<bool> {
        let json =
            std::fs::read_to_string(path).map_err(|e| CoreError::Io(std::sync::Arc::new(e)))?;
        let snap: PalaceSnapshot = serde_json::from_str(&json)
            .map_err(|e| CoreError::Serde(format!("deserialize: {}", e)))?;
        let computed_hash = snap.compute_hash();
        let hash_path = path.with_extension("json.sha256");
        let stored_hash = std::fs::read_to_string(&hash_path).unwrap_or_default();
        let valid = hex::encode(computed_hash) == stored_hash.trim();
        if !valid {
            log::warn!("MemoryPalace integrity check FAILED for {}", path.display());
        }
        Ok(valid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vsa(seed: u64) -> Vec<u8> {
        crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA::seeded_random(seed, 4096)
    }

    #[test]
    fn test_new_palace_has_default_rooms() {
        let palace = MemoryPalace::new();
        assert_eq!(palace.rooms.len(), 8);
        assert!(palace.room_name_index.contains_key("curiosity"));
        assert!(palace.room_name_index.contains_key("wisdom"));
        assert!(palace.room_name_index.contains_key("patterns"));
    }

    #[test]
    fn test_create_custom_room() {
        let mut palace = MemoryPalace::new();
        let vsa = make_vsa(42);
        let id = palace.create_room("test", "test room", vsa, "#ff0000");
        assert_eq!(id, 8);
        assert_eq!(palace.rooms.len(), 9);
    }

    #[test]
    fn test_walk_to_room() {
        let mut palace = MemoryPalace::new();
        assert!(palace.walk_to_by_name("curiosity"));
        assert_eq!(palace.current_room().unwrap().name, "curiosity");
        assert!(!palace.walk_to(999));
    }

    #[test]
    fn test_enter_and_find() {
        let mut palace = MemoryPalace::new();
        palace.walk_to_by_name("self");
        let vsa = make_vsa(100);
        let id = palace.enter("I am NeoTrix".to_string(), vsa.clone(), "episodic");
        assert!(id > 0);
        let results = palace.find_by_content("NeoTrix");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "I am NeoTrix");
        assert_eq!(results[0].room_id, palace.current_room_id);
    }

    #[test]
    fn test_find_room_for_vsa() {
        let palace = MemoryPalace::new();
        let curiosity_vsa = make_vsa({
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            "curiosity".hash(&mut h);
            h.finish()
        });
        let result = palace.find_room_for(&curiosity_vsa);
        assert_eq!(result, Some(0));
    }

    #[test]
    fn test_time_travel() {
        let mut palace = MemoryPalace::new();
        for i in 0..10 {
            let vsa = make_vsa(i);
            palace.enter(format!("entry {}", i), vsa, "episodic");
        }
        let results = palace.time_travel(5, 9);
        assert_eq!(results.len(), 5);
        assert!(results[0].content.contains("9"));
    }

    #[test]
    fn test_access_increments_count() {
        let mut palace = MemoryPalace::new();
        let vsa = make_vsa(42);
        palace.enter("important".to_string(), vsa, "episodic");
        assert!(palace.access(1));
        assert!(palace.access(1));
        let results = palace.find_by_content("important");
        assert_eq!(results[0].access_count, 2);
    }

    #[test]
    fn test_prune_removes_stale() {
        let mut palace = MemoryPalace::new();
        for i in 0..20 {
            let vsa = make_vsa(i as u64);
            palace.enter(format!("stale {}", i), vsa, "episodic");
        }
        palace.cycle = 600;
        for entry in palace.entries.iter_mut() {
            entry.last_accessed = 100;
            entry.significance = 0.1;
            entry.access_count = 0;
        }
        let pruned = palace.prune_stale();
        assert!(pruned > 0);
    }

    #[test]
    fn test_palace_map_format() {
        let palace = MemoryPalace::new();
        let map = palace.palace_map();
        assert!(map.contains("Memory Palace"));
        assert!(map.contains("curiosity"));
    }

    #[test]
    fn test_diagnostic_format() {
        let palace = MemoryPalace::new();
        let d = palace.diagnostic();
        assert!(d.starts_with("palace:"));
        assert!(d.contains("rms=8"));
    }

    #[test]
    fn test_save_load_roundtrip() {
        let mut palace = MemoryPalace::new();
        if palace.walk_to_by_name("self") {
            let vsa = make_vsa(42);
            palace.enter("persistent memory".to_string(), vsa, "episodic");
        }
        let tmp = std::env::temp_dir().join("test_palace.json");
        palace.save_to_json(&tmp).unwrap();
        let loaded = MemoryPalace::load_from_json(&tmp).unwrap();
        assert_eq!(loaded.rooms.len(), palace.rooms.len());
        assert_eq!(loaded.entries.len(), palace.entries.len());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_nearest_rooms() {
        let palace = MemoryPalace::new();
        let vsa = make_vsa({
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            "curiosity".hash(&mut h);
            h.finish()
        });
        let nearest = palace.nearest_rooms(&vsa, 3);
        assert_eq!(nearest.len(), 3);
        assert_eq!(nearest[0].name, "curiosity");
    }

    #[test]
    fn test_rooms_nearby() {
        let mut palace = MemoryPalace::new();
        palace.walk_to_by_name("curiosity");
        let nearby = palace.rooms_nearby(2);
        assert!(nearby.len() <= 2);
        for r in &nearby {
            assert_ne!(r.name, "curiosity");
        }
    }

    #[test]
    fn test_create_duplicate_room_returns_existing() {
        let mut palace = MemoryPalace::new();
        let vsa = make_vsa(42);
        let id1 = palace.create_room("curiosity", "dup", vsa.clone(), "#000");
        let id2 = palace.create_room("curiosity", "dup2", vsa, "#000");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_find_by_vsa() {
        let mut palace = MemoryPalace::new();
        let vsa = make_vsa(42);
        palace.enter("target memory".to_string(), vsa.clone(), "episodic");
        let results = palace.find_by_vsa(&vsa, 5);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_recall_bench() {
        let mut palace = MemoryPalace::new();
        let n = 100usize;
        // Fill with distracting entries
        for i in 0..n {
            let vsa = make_vsa((i + 100) as u64);
            palace.enter(format!("distractor memory {}", i), vsa, "episodic");
        }
        // Insert target at known position
        let query_vsa = make_vsa(42);
        palace.enter("target memory".to_string(), query_vsa.clone(), "episodic");

        // Benchmark find_by_vsa recall
        let start = std::time::Instant::now();
        let results = palace.find_by_vsa(&query_vsa, 5);
        let elapsed = start.elapsed();
        let found = results.iter().any(|e| e.content == "target memory");
        assert!(
            found,
            "VSA recall should find target among {} distractors",
            n
        );
        assert!(
            elapsed.as_micros() < 50_000,
            "VSA recall too slow: {}µs",
            elapsed.as_micros()
        );

        // Benchmark find_by_content recall
        let start2 = std::time::Instant::now();
        let results2 = palace.find_by_content("target");
        let elapsed2 = start2.elapsed();
        let found2 = results2.iter().any(|e| e.content == "target memory");
        assert!(found2, "Content recall should find target");
        assert!(
            elapsed2.as_micros() < 10_000,
            "Content recall too slow: {}µs",
            elapsed2.as_micros()
        );

        // Benchmark room assignment
        let room_id = palace.find_room_for(&query_vsa);
        assert!(room_id.is_some(), "Target VSA should map to a room");
    }

    #[test]
    fn test_significance_decay() {
        let mut palace = MemoryPalace::new();
        let vsa = make_vsa(1);
        palace.enter("decay test".to_string(), vsa, "episodic");
        assert!((palace.entries[0].significance - 0.5).abs() < 1e-6);

        // Decay 100 times (1000 cycles worth at 10-cycle intervals)
        for _ in 0..100 {
            palace.decay(0.995);
        }
        assert!(
            palace.entries[0].significance < 0.5,
            "significance should decay: {}",
            palace.entries[0].significance
        );
        assert!(
            palace.entries[0].significance > 0.3,
            "significance should stay above 0.3 after 100 decays: {}",
            palace.entries[0].significance
        );

        // Access reinforces significance back up
        palace.access(0);
        assert!(
            (palace.entries[0].significance - (0.5f64 * 0.995f64.powi(100) + 0.05)).abs() < 0.001,
            "access should reinforce significance: {}",
            palace.entries[0].significance
        );
    }
}
