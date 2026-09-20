#![deny(clippy::unwrap_used)]

use std::path::Path;

use serde::{Deserialize, Serialize};

/// A single evaluation dataset entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetEntry {
    pub input: String,
    pub expected_output: String,
    pub tags: Vec<String>,
}

/// A collection of evaluation entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    pub name: String,
    pub entries: Vec<DatasetEntry>,
}

impl Dataset {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            entries: Vec::new(),
        }
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn add_entry(&mut self, entry: DatasetEntry) {
        self.entries.push(entry);
    }

    /// Filter entries by tag (exact match), returning a new owned Dataset.
    pub fn filter_by_tag(&self, tag: &str) -> Dataset {
        Dataset {
            name: self.name.clone(),
            entries: self
                .entries
                .iter()
                .filter(|e| e.tags.iter().any(|t| t == tag))
                .cloned()
                .collect(),
        }
    }
}

/// Save a dataset to a JSON file.
pub fn save_json(dataset: &Dataset, path: &Path) -> Result<(), String> {
    let json = serde_json::to_string_pretty(dataset)
        .map_err(|e| format!("serialize error: {e}"))?;
    std::fs::write(path, json).map_err(|e| format!("io error: {e}"))?;
    Ok(())
}

/// Load a dataset from a JSON file.
pub fn load_json(path: &Path) -> Result<Dataset, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("io error: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("deserialize error: {e}"))
}

/// Filter a dataset by tag, returning a new owned Dataset.
pub fn filter_by_tag(dataset: &Dataset, tag: &str) -> Dataset {
    dataset.filter_by_tag(tag)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry(input: &str, output: &str, tags: &[&str]) -> DatasetEntry {
        DatasetEntry {
            input: input.into(),
            expected_output: output.into(),
            tags: tags.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn test_dataset_new() {
        let ds = Dataset::new("test");
        assert_eq!(ds.name, "test");
        assert_eq!(ds.entry_count(), 0);
    }

    #[test]
    fn test_dataset_add_entry() {
        let mut ds = Dataset::new("test");
        ds.add_entry(sample_entry("q1", "a1", &["math"]));
        ds.add_entry(sample_entry("q2", "a2", &["reading"]));
        assert_eq!(ds.entry_count(), 2);
    }

    #[test]
    fn test_filter_by_tag_method() {
        let mut ds = Dataset::new("test");
        ds.add_entry(sample_entry("q1", "a1", &["math"]));
        ds.add_entry(sample_entry("q2", "a2", &["reading"]));
        ds.add_entry(sample_entry("q3", "a3", &["math", "hard"]));

        let math = ds.filter_by_tag("math");
        assert_eq!(math.entry_count(), 2);
        assert!(math.entries.iter().all(|e| e.tags.contains(&"math".to_string())));
    }

    #[test]
    fn test_filter_by_tag_no_match() {
        let mut ds = Dataset::new("test");
        ds.add_entry(sample_entry("q1", "a1", &["math"]));
        let filtered = ds.filter_by_tag("physics");
        assert_eq!(filtered.entry_count(), 0);
    }

    #[test]
    fn test_filter_by_tag_standalone_fn() {
        let mut ds = Dataset::new("test");
        ds.add_entry(sample_entry("q1", "a1", &["math"]));
        ds.add_entry(sample_entry("q2", "a2", &["reading"]));
        let math = filter_by_tag(&ds, "math");
        assert_eq!(math.entry_count(), 1);
    }

    #[test]
    fn test_save_json_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.json");

        let mut ds = Dataset::new("my-dataset");
        ds.add_entry(sample_entry("What is 2+2?", "4", &["math", "easy"]));
        ds.add_entry(sample_entry("What is Rust?", "A language", &["programming"]));

        save_json(&ds, &path).unwrap();
        let loaded = load_json(&path).unwrap();

        assert_eq!(loaded.name, "my-dataset");
        assert_eq!(loaded.entry_count(), 2);
        assert_eq!(loaded.entries[0].input, "What is 2+2?");
        assert_eq!(loaded.entries[0].expected_output, "4");
        assert_eq!(loaded.entries[0].tags, vec!["math", "easy"]);
    }

    #[test]
    fn test_save_json_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.json");

        let ds = Dataset::new("empty");
        save_json(&ds, &path).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn test_load_json_nonexistent() {
        let result = load_json(Path::new("/nonexistent/path.json"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("io error"));
    }

    #[test]
    fn test_load_json_invalid() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.json");
        std::fs::write(&path, "not json").unwrap();

        let result = load_json(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("deserialize error"));
    }

    #[test]
    fn test_dataset_entry_serialization() {
        let entry = sample_entry("q", "a", &["tag1"]);
        let json = serde_json::to_string(&entry).unwrap();
        let deser: DatasetEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.input, "q");
        assert_eq!(deser.tags, vec!["tag1"]);
    }

    #[test]
    fn test_dataset_serialization() {
        let mut ds = Dataset::new("test");
        ds.add_entry(sample_entry("q1", "a1", &[]));
        let json = serde_json::to_string(&ds).unwrap();
        let deser: Dataset = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.name, "test");
        assert_eq!(deser.entry_count(), 1);
    }

    #[test]
    fn test_filter_preserves_name() {
        let mut ds = Dataset::new("my-ds");
        ds.add_entry(sample_entry("q1", "a1", &["tag-a"]));
        ds.add_entry(sample_entry("q2", "a2", &["tag-b"]));
        let filtered = ds.filter_by_tag("tag-a");
        assert_eq!(filtered.name, "my-ds");
    }

    #[test]
    fn test_empty_dataset_filter() {
        let ds = Dataset::new("empty");
        let filtered = ds.filter_by_tag("anything");
        assert_eq!(filtered.entry_count(), 0);
    }
}
