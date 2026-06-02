use std::collections::HashMap;
use std::path::PathBuf;
use chrono::Utc;
use super::search::LiteratureSearcher;
use super::types::{
    KnowledgeEngineStats, KnowledgeEntry, KnowledgeRelation, RelationType,
};
use crate::neotrix::error::{NeoTrixError, NeoTrixResult};
use crate::neotrix::nt_mind::embedding::TextEmbedder;

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
    #[allow(dead_code)]
    compact_threshold: usize,
    pub literature_searcher: Option<LiteratureSearcher>,
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
            compact_threshold: 500,
            literature_searcher: None,
        }
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
            let min_id = self.entries.iter()
                .min_by(|a, b| a.1.importance.partial_cmp(&b.1.importance).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(k, _)| k.clone());
            if let Some(old_id) = min_id {
                self.remove_entry(&old_id);
            }
        }

        self.title_index.insert(entry.title.clone(), id.clone());
        for tag in &entry.tags {
            self.tag_index.entry(tag.clone()).or_default().push(id.clone());
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
        } else { false }
    }

    pub fn add_relation(&mut self, from_id: &str, to_id: &str, rtype: RelationType, weight: f64, desc: &str) {
        if !self.entries.contains_key(from_id) || !self.entries.contains_key(to_id) { return; }
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
        let jpath = self.journal_path.as_ref()
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
            std::fs::write(jpath, &buf)?;
        }
        Ok(())
    }

    pub fn reset_dirty(&mut self) {
        self.dirty_since_compact = 0;
    }

    pub fn compact(&self) -> NeoTrixResult<()> {
        let path = self.persist_path.as_ref()
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
                            .inspect_err(|e| log::warn!("[knowledge] parse entry failed: {}", e)) {
                            let id = entry.id.clone();
                            engine.title_index.insert(entry.title.clone(), id.clone());
                            for tag in &entry.tags {
                                engine.tag_index.entry(tag.clone()).or_default().push(id.clone());
                            }
                            engine.entries.insert(id, entry);
                        }
                    }
                }
                if let Some(relations) = json["relations"].as_array() {
                    for rel_val in relations {
                        if let Ok(rel) = serde_json::from_value::<KnowledgeRelation>(rel_val.clone()) {
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
                    if line.trim().is_empty() { continue; }
                    if let Ok(entry) = serde_json::from_str::<KnowledgeEntry>(line) {
                        let id = entry.id.clone();
                        if !engine.entries.contains_key(&id) {
                            engine.title_index.insert(entry.title.clone(), id.clone());
                            for tag in &entry.tags {
                                engine.tag_index.entry(tag.clone()).or_default().push(id.clone());
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
        top.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap_or(std::cmp::Ordering::Equal));
        r.push_str("\n  最重要条目 (top 5):\n");
        for e in top.iter().take(5) {
            r.push_str(&format!("    [{:.2}] {} (来源: {})\n", e.importance, e.title, e.source.name()));
        }
        r
    }

    pub fn export_graph(&self) -> serde_json::Value {
        let nodes: Vec<serde_json::Value> = self.entries.values().map(|e| {
            serde_json::json!({
                "id": e.id, "title": e.title, "source": e.source.name(),
                "importance": e.importance, "tags": e.tags, "dimensions": e.dimensions,
            })
        }).collect();
        let edges: Vec<serde_json::Value> = self.relations.iter().map(|r| {
            serde_json::json!({
                "from": r.from_id, "to": r.to_id, "type": format!("{:?}", r.relation_type),
                "weight": r.weight, "description": r.description,
            })
        }).collect();
        serde_json::json!({ "nodes": nodes, "edges": edges })
    }
}
