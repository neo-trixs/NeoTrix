use std::collections::HashMap;

use crate::core::nt_core_storage::{Record, SegmentReader, SegmentType, SegmentWriter, VsaTag};
use crate::core::nt_core_translate::bilingual::BilingualLexicon;
use crate::core::nt_core_translate::translate_engine::translate_types::{
    EngineSaveData, TranslationStrategy, TRANSLATION_ENGINE_STORAGE_FILE,
};
use crate::core::nt_core_translate::translate_engine::VsaTranslationEngine;

impl VsaTranslationEngine {
    /// Persist lexicon entries + engine stats to `~/.neotrix/translation_engine.json`
    pub fn save(&mut self) -> Result<(), String> {
        let strat_key = |s: &TranslationStrategy| -> String {
            match s {
                TranslationStrategy::DirectLookup => "direct".into(),
                TranslationStrategy::Compositional => "compo".into(),
                TranslationStrategy::Analogical => "analog".into(),
                TranslationStrategy::Refinement => "refine".into(),
            }
        };
        let mut stats_map: HashMap<String, (u64, u64)> = HashMap::new();
        for (strat, stats) in &self.strategy_stats {
            stats_map.insert(strat_key(strat), *stats);
        }
        let data = EngineSaveData {
            entries: self.lexicon.entries.clone(),
            next_id: self.lexicon.next_id,
            max_entries: self.lexicon.max_entries,
            strategy_stats: stats_map,
            total_translations: self.total_translations,
        };
        let home = dirs::home_dir().ok_or("Cannot find home directory")?;
        let path = home.join(".neotrix").join(TRANSLATION_ENGINE_STORAGE_FILE);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {}", e))?;
        }
        let json =
            serde_json::to_string_pretty(&data).map_err(|e| format!("Serialize error: {}", e))?;
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, json).map_err(|e| format!("Write error: {}", e))?;
        std::fs::rename(&tmp, &path).map_err(|e| format!("Rename error: {}", e))?;
        if self.persist_with_nts {
            let _ = self.save_nts();
        }
        self.sync_to_hypergraph();
        Ok(())
    }

    /// Load persisted engine state from `~/.neotrix/translation_engine.json`
    pub fn load() -> Self {
        let home = match dirs::home_dir() {
            Some(h) => h,
            None => return Self::new(),
        };
        let path = home.join(".neotrix").join(TRANSLATION_ENGINE_STORAGE_FILE);
        match std::fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<EngineSaveData>(&content) {
                Ok(data) => {
                    let mut eng = Self::new();
                    eng.lexicon =
                        BilingualLexicon::load_custom(data.entries, data.next_id, data.max_entries);
                    eng.total_translations = data.total_translations;
                    for (key, stats) in data.strategy_stats {
                        let strat = match key.as_str() {
                            "direct" => TranslationStrategy::DirectLookup,
                            "compo" => TranslationStrategy::Compositional,
                            "analog" => TranslationStrategy::Analogical,
                            "refine" => TranslationStrategy::Refinement,
                            _ => continue,
                        };
                        eng.strategy_stats.insert(strat, stats);
                    }
                    eng
                }
                Err(_) => Self::new(),
            },
            Err(_) => Self::new(),
        }
    }

    pub fn save_nts(&self) -> Result<String, String> {
        let home = dirs::home_dir().ok_or("Cannot find home directory")?;
        let dir = home.join(".neotrix");
        std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create dir: {}", e))?;
        let path = dir.join("translation_segment.nts");
        let mut writer = SegmentWriter::create(&path, SegmentType::Data)
            .map_err(|e| format!("Failed to create segment: {}", e))?;
        for entry in &self.lexicon.entries {
            let key = format!(
                "{}:{}:{}",
                entry.id,
                entry.source_lang.code(),
                entry.target_lang.code()
            );
            let data = serde_json::to_vec(entry).map_err(|e| format!("Serialize error: {}", e))?;
            let record = Record::new(VsaTag::SelfMemory, 0x03, &key, data);
            writer
                .append(&record)
                .map_err(|e| format!("Append error: {}", e))?;
        }
        writer
            .finalize()
            .map_err(|e| format!("Finalize error: {}", e))?;
        Ok(path.to_string_lossy().to_string())
    }

    pub fn load_nts(path: &str) -> Self {
        let reader = match SegmentReader::open(path) {
            Ok(r) => r,
            Err(_) => return Self::new(),
        };
        let records = reader.records();
        let mut entries: Vec<crate::core::nt_core_translate::bilingual::BilingualEntry> =
            Vec::new();
        for rec in records {
            if rec.record_type == 0x03 && !rec.tombstone {
                if let Ok(entry) = serde_json::from_slice::<
                    crate::core::nt_core_translate::bilingual::BilingualEntry,
                >(&rec.data)
                {
                    entries.push(entry);
                }
            }
        }
        let mut eng = Self::new();
        if !entries.is_empty() {
            let max_id = entries.iter().map(|e| e.id).max().unwrap_or(0) + 1;
            let max_entries = entries.len().max(1000);
            eng.lexicon = BilingualLexicon::load_custom(entries, max_id, max_entries);
        }
        eng
    }
}
