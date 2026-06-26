use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::core::nt_core_util;

const MAX_ENTRIES: usize = 500;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHistory {
    pub entries: Vec<String>,
    #[serde(skip)]
    pub position: Option<usize>,
    #[serde(skip)]
    pub search_query: String,
    #[serde(skip)]
    pub search_results: Vec<usize>,
    #[serde(skip)]
    pub search_active: bool,
    #[serde(skip)]
    pub search_selection: usize,
    max_entries: usize,
}

impl CommandHistory {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            position: None,
            search_query: String::new(),
            search_results: Vec::new(),
            search_active: false,
            search_selection: 0,
            max_entries: MAX_ENTRIES,
        }
    }

    pub fn load_or_new() -> Self {
        let mut this = Self::new();
        this.load();
        this
    }

    pub fn push(&mut self, cmd: String) {
        if cmd.trim().is_empty() {
            return;
        }
        if self.entries.last() == Some(&cmd) {
            return;
        }
        self.entries.push(cmd);
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
        self.position = None;
        self.save();
    }

    pub fn navigate_up(&mut self) -> Option<String> {
        if self.entries.is_empty() {
            return None;
        }
        let pos = self.position.get_or_insert(self.entries.len());
        if *pos > 0 {
            *pos -= 1;
            Some(self.entries[*pos].clone())
        } else {
            None
        }
    }

    pub fn navigate_down(&mut self) -> Option<String> {
        if let Some(pos) = self.position {
            if pos + 1 < self.entries.len() {
                self.position = Some(pos + 1);
                Some(self.entries[pos + 1].clone())
            } else {
                self.position = None;
                Some(String::new())
            }
        } else {
            None
        }
    }

    pub fn start_search(&mut self) {
        self.search_active = true;
        self.search_query.clear();
        self.search_results.clear();
        self.search_selection = 0;
        self.update_search_results();
    }

    pub fn update_search_results(&mut self) {
        let q = self.search_query.to_lowercase();
        self.search_results = self
            .entries
            .iter()
            .enumerate()
            .rev()
            .filter(|(_, e)| e.to_lowercase().contains(&q))
            .map(|(i, _)| i)
            .collect();
        if self.search_selection >= self.search_results.len() {
            self.search_selection = 0;
        }
    }

    pub fn cycle_search(&mut self) {
        if self.search_results.is_empty() {
            return;
        }
        self.search_selection = (self.search_selection + 1) % self.search_results.len();
    }

    pub fn select_search(&mut self) -> Option<String> {
        let result = self
            .search_results
            .get(self.search_selection)
            .map(|&i| self.entries[i].clone());
        self.cancel_search();
        result
    }

    pub fn cancel_search(&mut self) {
        self.search_active = false;
        self.search_query.clear();
        self.search_results.clear();
        self.search_selection = 0;
    }

    fn path() -> PathBuf {
        let home = nt_core_util::home_dir().to_string_lossy().to_string();
        PathBuf::from(home).join(".neotrix").join("history.json")
    }

    pub fn save(&self) {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string(&self.entries) {
            let tmp = path.with_extension("tmp");
            if std::fs::write(&tmp, &json).is_ok() {
                let _ = std::fs::rename(&tmp, &path);
            }
        }
    }

    fn load(&mut self) {
        let path = Self::path();
        if path.exists() {
            if let Ok(json) = std::fs::read_to_string(&path) {
                if let Ok(entries) = serde_json::from_str::<Vec<String>>(&json) {
                    self.entries = entries;
                    if self.entries.len() > self.max_entries {
                        let excess = self.entries.len() - self.max_entries;
                        self.entries.drain(..excess);
                    }
                }
            }
        }
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new()
    }
}
