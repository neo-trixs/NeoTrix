use super::search::LiteratureSearcher;
use super::types::{
    compute_provenance_hash, format_provenance_hash, KnowledgeEngineStats, KnowledgeEntry,
    KnowledgeEvidenceResult, KnowledgeRelation, RelationType,
};
use crate::core::nt_core_hcube::vsa_vector::VsaBackend;
use crate::core::nt_core_hcube::{MapVsaBackend, VsaVector};
use crate::core::nt_core_knowledge::evidence::EvidenceManager;
use crate::neotrix::nt_core_error::{NeoTrixError, NeoTrixResult};
use crate::neotrix::nt_mind::embedding::TextEmbedder;
use chrono::Utc;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct KnowledgeEngine {
    pub entries: HashMap<String, KnowledgeEntry>,
    pub relations: Vec<KnowledgeRelation>,
    title_index: HashMap<String, String>,
    pub(crate) tag_index: HashMap<String, Vec<String>>,
    pub(crate) embedder: TextEmbedder,
    max_entries: usize,
    persist_path: Option<PathBuf>,
    journal_path: Option<PathBuf>,
    dirty_since_compact: usize,
    pub literature_searcher: Option<LiteratureSearcher>,
    pub evidence_manager: Option<EvidenceManager>,
}

impl KnowledgeEngine {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: HashMap::new(),
            relations: Vec::new(),
            title_index: HashMap::new(),
            tag_index: HashMap::new(),
            embedder: TextEmbedder::new(),
            max_entries,
            persist_path: None,
            journal_path: None,
            dirty_since_compact: 0,
            literature_searcher: None,
            evidence_manager: None,
        }
    }

    pub fn with_evidence_manager(mut self, mgr: EvidenceManager) -> Self {
        self.evidence_manager = Some(mgr);
        self
    }

    pub fn with_lit_searcher(mut self) -> Self {
        self.literature_searcher = Some(LiteratureSearcher::new());
        self
    }

    pub fn set_persist_path(&mut self, path: PathBuf) {
        self.persist_path = Some(path.clone());
        let jp = path;
        let jp_str = jp.to_string_lossy().to_string() + ".journal";
        self.journal_path = Some(PathBuf::from(jp_str));
    }

    pub fn add_entry(&mut self, mut entry: KnowledgeEntry) -> String {
        let id = entry.id.clone();

        let text = format!("{} {} {}", entry.title, entry.body, entry.tags.join(" "));
        let embedding = self.embedder.embed(&text);
        entry.embedding = Some(embedding);
        entry.importance = entry.estimate_importance();
        self.dirty_since_compact += 1;

        if self.entries.len() >= self.max_entries {
            let min_id = self
                .entries
                .iter()
                .min_by(|a, b| {
                    a.1.importance
                        .partial_cmp(&b.1.importance)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(k, _)| k.clone());
            if let Some(old_id) = min_id {
                self.remove_entry(&old_id);
            }
        }

        self.title_index.insert(entry.title.clone(), id.clone());
        for tag in &entry.tags {
            self.tag_index
                .entry(tag.clone())
                .or_default()
                .push(id.clone());
        }

        entry.access_count = 0;
        entry.updated_at = Utc::now().timestamp();
        self.entries.insert(id.clone(), entry);
        id
    }

    pub fn add_entries(&mut self, entries: Vec<KnowledgeEntry>) -> Vec<String> {
        entries.into_iter().map(|e| self.add_entry(e)).collect()
    }

    pub fn remove_entry(&mut self, id: &str) -> bool {
        if let Some(entry) = self.entries.remove(id) {
            self.title_index.remove(&entry.title);
            for tag in &entry.tags {
                if let Some(ids) = self.tag_index.get_mut(tag) {
                    ids.retain(|i| i != id);
                }
            }
            self.relations.retain(|r| r.from_id != id && r.to_id != id);
            true
        } else {
            false
        }
    }

    pub fn add_relation(
        &mut self,
        from_id: &str,
        to_id: &str,
        rtype: RelationType,
        weight: f64,
        desc: &str,
    ) {
        if !self.entries.contains_key(from_id) || !self.entries.contains_key(to_id) {
            return;
        }
        let relation = KnowledgeRelation {
            id: uuid::Uuid::new_v4().to_string(),
            from_id: from_id.to_string(),
            to_id: to_id.to_string(),
            relation_type: rtype,
            weight,
            description: desc.to_string(),
            created_at: Utc::now().timestamp(),
        };
        self.relations.push(relation);

        if let Some(e) = self.entries.get_mut(from_id) {
            if !e.related_ids.contains(&to_id.to_string()) {
                e.related_ids.push(to_id.to_string());
            }
        }
        if let Some(e) = self.entries.get_mut(to_id) {
            if !e.related_ids.contains(&from_id.to_string()) {
                e.related_ids.push(from_id.to_string());
            }
        }
    }

    pub fn save(&self) -> NeoTrixResult<()> {
        let jpath = self
            .journal_path
            .as_ref()
            .ok_or_else(|| NeoTrixError::from("journal 路径未设置"))?;
        let mut count = 0;
        let mut buf = String::with_capacity(64 * 1024);
        for entry in self.entries.values() {
            if let Ok(line) = serde_json::to_string(entry) {
                buf.push_str(&line);
                buf.push('\n');
                count += 1;
            }
        }
        if count > 0 {
            let tmp = jpath.with_extension("tmp");
            std::fs::write(&tmp, &buf)?;
            std::fs::rename(&tmp, jpath)?;
        }
        Ok(())
    }

    pub fn reset_dirty(&mut self) {
        self.dirty_since_compact = 0;
    }

    pub fn compact(&self) -> NeoTrixResult<()> {
        let path = self
            .persist_path
            .as_ref()
            .ok_or_else(|| NeoTrixError::from("持久化路径未设置"))?;

        let data = serde_json::json!({
            "entries": self.entries,
            "relations": self.relations,
            "stats": {
                "total_entries": self.entries.len(),
                "total_relations": self.relations.len(),
                "max_entries": self.max_entries,
            }
        });
        let json = serde_json::to_string(&data)
            .map_err(|e| NeoTrixError::Serde(format!("序列化失败: {}", e)))?;

        let tmp_path = path.with_extension("tmp");
        std::fs::write(&tmp_path, &json)?;
        std::fs::rename(&tmp_path, path)?;

        if let Some(jp) = &self.journal_path {
            let _ = std::fs::remove_file(jp);
        }
        Ok(())
    }

    pub fn load_from(path: &PathBuf) -> Self {
        let mut engine = Self::new(1000).with_lit_searcher();
        engine.set_persist_path(path.clone());

        if path.exists() {
            let data = std::fs::read_to_string(path).unwrap_or_default();
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&data) {
                if let Some(entries) = json["entries"].as_object() {
                    for (_key, val) in entries {
                        if let Ok(entry) = serde_json::from_value::<KnowledgeEntry>(val.clone())
                            .inspect_err(|e| log::warn!("[knowledge] parse entry failed: {}", e))
                        {
                            let id = entry.id.clone();
                            engine.title_index.insert(entry.title.clone(), id.clone());
                            for tag in &entry.tags {
                                engine
                                    .tag_index
                                    .entry(tag.clone())
                                    .or_default()
                                    .push(id.clone());
                            }
                            engine.entries.insert(id, entry);
                        }
                    }
                }
                if let Some(relations) = json["relations"].as_array() {
                    for rel_val in relations {
                        if let Ok(rel) =
                            serde_json::from_value::<KnowledgeRelation>(rel_val.clone())
                        {
                            engine.relations.push(rel);
                        }
                    }
                }
            }
        }

        let journal_path = path.with_extension("json.journal");
        if journal_path.exists() {
            if let Ok(data) = std::fs::read_to_string(&journal_path) {
                for line in data.lines() {
                    if line.trim().is_empty() {
                        continue;
                    }
                    if let Ok(entry) = serde_json::from_str::<KnowledgeEntry>(line) {
                        let id = entry.id.clone();
                        if !engine.entries.contains_key(&id) {
                            engine.title_index.insert(entry.title.clone(), id.clone());
                            for tag in &entry.tags {
                                engine
                                    .tag_index
                                    .entry(tag.clone())
                                    .or_default()
                                    .push(id.clone());
                            }
                        }
                        engine.entries.insert(id, entry);
                    }
                }
            }
        }

        engine.dirty_since_compact = 0;
        engine
    }

    pub fn stats(&self) -> KnowledgeEngineStats {
        let mut per_source: HashMap<String, usize> = HashMap::new();
        for e in self.entries.values() {
            *per_source.entry(e.source.name().to_string()).or_insert(0) += 1;
        }
        KnowledgeEngineStats {
            total_entries: self.entries.len(),
            total_relations: self.relations.len(),
            max_entries: self.max_entries,
            per_source,
        }
    }

    pub fn report(&self) -> String {
        let mut r = String::new();
        r.push_str("📚 知识引擎报告\n");
        r.push_str(&format!("  条目总数: {}\n", self.entries.len()));
        r.push_str(&format!("  关系总数: {}\n", self.relations.len()));
        r.push_str(&format!("  最大容量: {}\n", self.max_entries));

        let mut by_source: HashMap<&str, usize> = HashMap::new();
        for e in self.entries.values() {
            *by_source.entry(e.source.name()).or_insert(0) += 1;
        }
        r.push_str("\n  来源分布:\n");
        for (src, count) in by_source.iter().take(10) {
            r.push_str(&format!("    {}: {}\n", src, count));
        }

        let mut top: Vec<&KnowledgeEntry> = self.entries.values().collect();
        top.sort_by(|a, b| {
            b.importance
                .partial_cmp(&a.importance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        r.push_str("\n  最重要条目 (top 5):\n");
        for e in top.iter().take(5) {
            r.push_str(&format!(
                "    [{:.2}] {} (来源: {})\n",
                e.importance,
                e.title,
                e.source.name()
            ));
        }
        r
    }

    pub fn export_graph(&self) -> serde_json::Value {
        let nodes: Vec<serde_json::Value> = self
            .entries
            .values()
            .map(|e| {
                serde_json::json!({
                    "id": e.id, "title": e.title, "source": e.source.name(),
                    "importance": e.importance, "tags": e.tags, "dimensions": e.dimensions,
                })
            })
            .collect();
        let edges: Vec<serde_json::Value> = self
            .relations
            .iter()
            .map(|r| {
                serde_json::json!({
                    "from": r.from_id, "to": r.to_id, "type": format!("{:?}", r.relation_type),
                    "weight": r.weight, "description": r.description,
                })
            })
            .collect();
        serde_json::json!({ "nodes": nodes, "edges": edges })
    }

    pub fn search_by_vsa(&self, query: &VsaVector<4096>, top_k: usize) -> Vec<KnowledgeEntry> {
        let backend = MapVsaBackend;
        let mut scored: Vec<(f64, &KnowledgeEntry)> = self
            .entries
            .values()
            .filter_map(|e| e.vsa.as_ref().map(|v| (backend.similarity(query, v), e)))
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored.into_iter().map(|(_, e)| e.clone()).collect()
    }

    pub fn search_text(&self, query: &str, top_k: usize) -> Vec<KnowledgeEntry> {
        let q = query.to_lowercase();
        let terms: Vec<&str> = q.split_whitespace().collect();
        if terms.is_empty() {
            return Vec::new();
        }
        let mut scored: Vec<(f64, &KnowledgeEntry)> = self
            .entries
            .values()
            .filter_map(|e| {
                let text = format!("{} {} {} {}", e.title, e.summary, e.body, e.tags.join(" "))
                    .to_lowercase();
                let matches = terms.iter().filter(|t| text.contains(*t)).count();
                if matches == 0 {
                    None
                } else {
                    Some((matches as f64 / terms.len() as f64, e))
                }
            })
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored.into_iter().map(|(_, e)| e.clone()).collect()
    }

    pub fn search_with_provenance(
        &self,
        query: &str,
        top_k: usize,
    ) -> Vec<KnowledgeEvidenceResult> {
        let results = self.search_text(query, top_k);
        results
            .into_iter()
            .map(|entry| {
                let evidence = self
                    .evidence_manager
                    .as_ref()
                    .map(|mgr| {
                        mgr.get_by_ids(&entry.evidence_ids)
                            .into_iter()
                            .cloned()
                            .collect()
                    })
                    .unwrap_or_default();
                let evidence_conf = self
                    .evidence_manager
                    .as_ref()
                    .map(|mgr| {
                        if entry.evidence_ids.is_empty() {
                            entry.confidence
                        } else {
                            let ec = mgr.combined_confidence(&entry.evidence_ids);
                            entry.confidence * 0.5 + ec * 0.5
                        }
                    })
                    .unwrap_or(entry.confidence);
                KnowledgeEvidenceResult {
                    entry,
                    evidence,
                    combined_score: evidence_conf,
                }
            })
            .collect()
    }

    pub fn encode_entry_vsa(&mut self, entry_id: &str, seed: u64) {
        if let Some(entry) = self.entries.get_mut(entry_id) {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            entry.title.hash(&mut hasher);
            entry.summary.hash(&mut hasher);
            let vsa_seed = hasher.finish().wrapping_add(seed);
            entry.vsa = Some(VsaVector::random(vsa_seed));
        }
    }

    // ── N11: Knowledge Value Provable ─────────────────────────────────────

    /// Store a computed provenance hash on an existing entry.
    ///
    /// The hash is SHA-256 of `(source_url || "||" || quotation || "||" || timestamp_ns)`.
    /// Returns the computed hash, or an error string if the entry is not found.
    pub fn store_with_provenance(
        &mut self,
        entry_id: &str,
        source_url: &str,
        quotation: &str,
        timestamp_ns: u64,
    ) -> Result<[u8; 32], String> {
        let entry = self
            .entries
            .get_mut(entry_id)
            .ok_or_else(|| format!("entry {} not found", entry_id))?;
        let hash = compute_provenance_hash(source_url, quotation, timestamp_ns);
        entry.provenance_hash = Some(hash);
        entry.source_url = source_url.to_string();
        Ok(hash)
    }

    /// Verify that an entry's provenance_hash matches a recomputation over
    /// its current source_url and body.
    ///
    /// Returns `false` if the entry has no provenance_hash, is not found,
    /// or the recomputed hash differs.
    pub fn verify_entry_provenance(&self, entry_id: &str) -> bool {
        let entry = match self.entries.get(entry_id) {
            Some(e) => e,
            None => return false,
        };
        let stored = match entry.provenance_hash {
            Some(h) => h,
            None => return false,
        };
        let recomputed = compute_provenance_hash(
            &entry.source_url,
            &entry.body,
            entry.created_at as u64 * 1_000_000_000,
        );
        stored == recomputed
    }

    /// Verify that all cross-references stored on this entry still match
    /// the referenced entries' current provenance hashes.
    ///
    /// Returns `true` only when every (referenced_id, stored_hash) pair
    /// can be resolved and the stored hash equals the referenced entry's
    /// provenance_hash.
    pub fn verify_cross_references(&self, entry_id: &str) -> bool {
        let entry = match self.entries.get(entry_id) {
            Some(e) => e,
            None => return false,
        };
        if entry.cross_references.is_empty() {
            return true;
        }
        for (ref_id, stored_hash) in &entry.cross_references {
            match self.entries.get(ref_id) {
                Some(ref_entry) => match ref_entry.provenance_hash {
                    Some(ref_hash) => {
                        if ref_hash != *stored_hash {
                            return false;
                        }
                    }
                    None => return false,
                },
                None => return false,
            }
        }
        true
    }

    /// Produce a human-readable provenance report for an entry.
    ///
    /// Includes: entry title, provenance hash hex, cross-reference count and
    /// verification status, and overall provenance integrity status.
    pub fn report_provenance(&self, entry_id: &str) -> String {
        let entry = match self.entries.get(entry_id) {
            Some(e) => e,
            None => return format!("entry {} not found", entry_id),
        };

        let hash_status = match entry.provenance_hash {
            Some(stored) => {
                let recomputed = compute_provenance_hash(
                    &entry.source_url,
                    &entry.body,
                    entry.created_at as u64 * 1_000_000_000,
                );
                if stored == recomputed {
                    format!("✓ proven (sha256:{})", format_provenance_hash(&stored))
                } else {
                    format!("✗ TAMPERED (stored:{})", format_provenance_hash(&stored))
                }
            }
            None => "no provenance hash".to_string(),
        };

        let xref_count = entry.cross_references.len();
        let xref_status = if xref_count == 0 {
            "no cross-references".to_string()
        } else if self.verify_cross_references(entry_id) {
            format!("✓ {} cross-reference(s) verified", xref_count)
        } else {
            format!("✗ {} cross-reference(s) MISMATCH", xref_count)
        };

        format!(
            "[provenance] {} :: hash={} | source={} | xrefs={} | integrity={}",
            entry.title,
            hash_status,
            entry.source.name(),
            xref_status,
            if entry.provenance_hash.is_some() && self.verify_cross_references(entry_id) {
                "✓"
            } else {
                "⚠"
            },
        )
    }
}
