use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KnowledgeCategory {
    Domain,
    Signal,
    Lesson,
    SearchPattern,
}

impl KnowledgeCategory {
    pub fn name(&self) -> &'static str {
        match self {
            KnowledgeCategory::Domain => "domains",
            KnowledgeCategory::Signal => "signals",
            KnowledgeCategory::Lesson => "lessons",
            KnowledgeCategory::SearchPattern => "search_patterns",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    pub title: String,
    pub content: String,
    pub category: KnowledgeCategory,
    pub tags: Vec<String>,
    pub confidence: f64,
    pub created_at: u64,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompoundKnowledgeBase {
    pub entries: Vec<KnowledgeEntry>,
    pub base_path: PathBuf,
    pub max_entries: usize,
}

impl CompoundKnowledgeBase {
    pub fn new(base_path: PathBuf) -> Self {
        let mut ckb = Self {
            entries: Vec::new(),
            base_path,
            max_entries: 500,
        };
        ckb.load_from_disk();
        ckb
    }

    pub fn register(&mut self, entry: KnowledgeEntry) {
        if self.entries.len() >= self.max_entries {
            self.entries.sort_by(|a, b| {
                a.confidence
                    .partial_cmp(&b.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            if entry.confidence > self.entries[0].confidence {
                self.entries.remove(0);
            } else {
                return;
            }
        }
        self.entries.push(entry);
    }

    pub fn find(&self, query: &str, category: Option<KnowledgeCategory>) -> Vec<&KnowledgeEntry> {
        let q = query.to_lowercase();
        self.entries
            .iter()
            .filter(|e| {
                if let Some(cat) = category {
                    if e.category != cat {
                        return false;
                    }
                }
                e.title.to_lowercase().contains(&q)
                    || e.content.to_lowercase().contains(&q)
                    || e.tags.iter().any(|t| t.to_lowercase().contains(&q))
            })
            .take(10)
            .collect()
    }

    pub fn save_to_disk(&self) -> Result<usize, String> {
        fs::create_dir_all(&self.base_path).map_err(|e| e.to_string())?;
        for cat in &[
            KnowledgeCategory::Domain,
            KnowledgeCategory::Signal,
            KnowledgeCategory::Lesson,
            KnowledgeCategory::SearchPattern,
        ] {
            let cat_dir = self.base_path.join(cat.name());
            fs::create_dir_all(&cat_dir).map_err(|e| e.to_string())?;
            let entries: Vec<&KnowledgeEntry> =
                self.entries.iter().filter(|e| e.category == *cat).collect();
            for entry in &entries {
                let slug: String = entry
                    .title
                    .to_lowercase()
                    .chars()
                    .map(|c| {
                        if c.is_alphanumeric() || c == '-' {
                            c
                        } else {
                            '_'
                        }
                    })
                    .collect();
                let file_path = cat_dir.join(format!("{}.md", slug));
                if file_path.exists() {
                    continue;
                }
                let md = format!(
                    "# {}\n\n**source**: {} | **confidence**: {:.2} | **tags**: {}\n\n{}",
                    entry.title,
                    entry.source,
                    entry.confidence,
                    entry.tags.join(", "),
                    entry.content,
                );
                fs::write(&file_path, md).map_err(|e| e.to_string())?;
            }
        }
        let index_path = self.base_path.join("INDEX.md");
        let mut index = String::from("# Compounding Knowledge Index\n\n");
        for cat in &[
            KnowledgeCategory::Domain,
            KnowledgeCategory::Signal,
            KnowledgeCategory::Lesson,
            KnowledgeCategory::SearchPattern,
        ] {
            let cat_dir = self.base_path.join(cat.name());
            index.push_str(&format!("## {}\n\n", cat.name()));
            if let Ok(entries) = fs::read_dir(&cat_dir) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        index.push_str(&format!("- [{}]({}/{})\n", name, cat.name(), name));
                    }
                }
            }
            index.push('\n');
        }
        fs::write(&index_path, index).map_err(|e| e.to_string())?;
        Ok(self.entries.len())
    }

    pub fn load_from_disk(&mut self) {
        let _ = fs::create_dir_all(&self.base_path);
        for cat in &[
            KnowledgeCategory::Domain,
            KnowledgeCategory::Signal,
            KnowledgeCategory::Lesson,
            KnowledgeCategory::SearchPattern,
        ] {
            let cat_dir = self.base_path.join(cat.name());
            let _ = fs::create_dir_all(&cat_dir);
        }
    }

    pub fn category_count(&self) -> HashMap<KnowledgeCategory, usize> {
        let mut counts = HashMap::new();
        for entry in &self.entries {
            *counts.entry(entry.category).or_insert(0) += 1;
        }
        counts
    }

    pub fn diagnostic(&self) -> String {
        let counts = self.category_count();
        let parts: Vec<String> = counts
            .iter()
            .map(|(k, v)| format!("{}={}", k.name(), v))
            .collect();
        format!(
            "compound_kb:total={}|{}",
            self.entries.len(),
            parts.join("|")
        )
    }
}
