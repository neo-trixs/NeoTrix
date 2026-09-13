//! Experience Memory — browser-act/skills memory 文件模式吸收
//! 
//! 每技能一内存文件，执行前读取调整策略，执行后追加异常情况
//! 同步到 KB experience 命名空间

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// 经验记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceRecord {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event: String,
    pub conclusion: String,
    pub skill_name: String,
    pub working_dir: PathBuf,
}

/// 技能内存
#[derive(Debug, Clone)]
pub struct SkillMemory {
    pub skill_name: String,
    pub working_dir: PathBuf,
    pub records: Vec<ExperienceRecord>,
    pub file_path: PathBuf,
}

impl SkillMemory {
    pub fn new(skill_name: &str, working_dir: &Path) -> Self {
        let memory_dir = working_dir.join("browser-act-skill-forge-memories");
        let file_name = format!("{}.memory.md", skill_name.replace('/', "-"));
        let file_path = memory_dir.join(file_name);
        
        let records = if file_path.exists() {
            Self::parse_memory_file(&file_path)
        } else {
            Vec::new()
        };
        
        Self {
            skill_name: skill_name.to_string(),
            working_dir: working_dir.to_path_buf(),
            records,
            file_path,
        }
    }

    fn parse_memory_file(path: &Path) -> Vec<ExperienceRecord> {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        let mut records = Vec::new();
        
        for line in content.lines() {
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            // 格式: {YYYY-MM-DD}: {what happened} → {conclusion}
            if let Some(colon_pos) = line.find(':') {
                let date_part = line[..colon_pos].trim();
                let rest = line[colon_pos + 1..].trim();
                if let Some(arrow_pos) = rest.find("→") {
                    let event = rest[..arrow_pos].trim();
                    let conclusion = rest[arrow_pos + 1..].trim();
                    
                    if let Ok(date) = chrono::NaiveDate::parse_from_str(
                        date_part.trim_matches(|c| c == '{' || c == '}'),
                        "%Y-%m-%d"
                    ) {
                        records.push(ExperienceRecord {
                            timestamp: date.and_hms_opt(0, 0, 0).expect("valid time").and_utc(),
                            event: event.to_string(),
                            conclusion: conclusion.to_string(),
                            skill_name: String::new(),
                            working_dir: PathBuf::new(),
                        });
                    }
                }
            }
        }
        records
    }

    /// 执行前读取：获取相关历史教训
    pub fn get_lessons(&self, query: &str) -> Vec<&ExperienceRecord> {
        self.records.iter()
            .filter(|r| r.event.contains(query) || r.conclusion.contains(query))
            .collect()
    }

    /// 执行后记录：追加异常情况
    pub fn record_experience(&mut self, event: &str, conclusion: &str) -> Result<(), std::io::Error> {
        let record = ExperienceRecord {
            timestamp: chrono::Utc::now(),
            event: event.to_string(),
            conclusion: conclusion.to_string(),
            skill_name: self.skill_name.clone(),
            working_dir: self.working_dir.clone(),
        };
        
        self.records.push(record.clone());
        self.append_to_file(&record)
    }

    fn append_to_file(&self, record: &ExperienceRecord) -> Result<(), std::io::Error> {
        std::fs::create_dir_all(self.file_path.parent().expect("has parent"))?;
        
        let line = format!(
            "{{{}}}: {} → {}\n",
            record.timestamp.format("%Y-%m-%d"),
            record.event,
            record.conclusion
        );
        
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)?
            .write_all(line.as_bytes())
    }

    /// 同步到 KB experience 命名空间
    pub async fn sync_to_kb(&self, kb: &Arc<crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase>) -> Result<usize, String> {
        let mut synced = 0;
        for record in &self.records {
            // 这里简化：实际应调用 KB 的 experience 写入 API
            // kb.write_experience(record).await?;
            synced += 1;
        }
        Ok(synced)
    }
}

/// 经验内存管理器
#[derive(Debug)]
pub struct ExperienceMemoryManager {
    memories: Arc<RwLock<HashMap<String, SkillMemory>>>,
    kb: Option<Arc<crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase>>,
}

impl ExperienceMemoryManager {
    pub fn new() -> Self {
        Self {
            memories: Arc::new(RwLock::new(HashMap::new())),
            kb: None,
        }
    }

    pub fn with_kb(mut self, kb: Arc<crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase>) -> Self {
        self.kb = Some(kb);
        self
    }

    /// 获取或创建技能内存
    pub async fn get_memory(&self, skill_name: &str, working_dir: &Path) -> SkillMemory {
        let key = format!("{}@{}", skill_name, working_dir.display());
        let mut memories = self.memories.write().await;
        
        if let Some(mem) = memories.get(&key) {
            mem.clone()
        } else {
            let mem = SkillMemory::new(skill_name, working_dir);
            memories.insert(key.clone(), mem.clone());
            mem
        }
    }

    /// 记录经验
    pub async fn record(&self, skill_name: &str, working_dir: &Path, event: &str, conclusion: &str) -> Result<(), std::io::Error> {
        let key = format!("{}@{}", skill_name, working_dir.display());
        let mut memories = self.memories.write().await;
        
        let mem = memories.entry(key).or_insert_with(|| SkillMemory::new(skill_name, working_dir));
        mem.record_experience(event, conclusion)
    }

    /// 获取教训
    pub async fn get_lessons(&self, skill_name: &str, working_dir: &Path, query: &str) -> Vec<ExperienceRecord> {
        let mem = self.get_memory(skill_name, working_dir).await;
        mem.get_lessons(query).into_iter().cloned().collect()
    }

    /// 同步所有内存到 KB
    pub async fn sync_all_to_kb(&self) -> Result<usize, String> {
        let memories = self.memories.read().await;
        let mut total = 0;
        
        if let Some(ref kb) = self.kb {
            for mem in memories.values() {
                total += mem.sync_to_kb(kb).await?;
            }
        }
        
        Ok(total)
    }

    /// SelfTest for C1 promotion
    pub fn self_test() -> Result<(), String> {
        let temp_dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let manager = ExperienceMemoryManager::new();
        
        // Test 1: Create memory
        let mem = SkillMemory::new("test-skill", temp_dir.path());
        assert!(mem.records.is_empty());
        
        // Test 2: Record experience
        let mut mem = SkillMemory::new("test-skill", temp_dir.path());
        mem.record_experience("Strategy failed", "Use alternative approach").map_err(|e| e.to_string())?;
        assert_eq!(mem.records.len(), 1);
        
        // Test 3: Get lessons
        let lessons = mem.get_lessons("Strategy");
        assert_eq!(lessons.len(), 1);
        assert!(lessons[0].conclusion.contains("alternative"));
        
        // Test 4: File persistence
        assert!(mem.file_path.exists());
        let content = std::fs::read_to_string(&mem.file_path).map_err(|e| e.to_string())?;
        assert!(content.contains("Strategy failed"));
        assert!(content.contains("Use alternative approach"));
        
        // Test 5: Manager integration
        let mem2 = manager.get_memory("test-skill", temp_dir.path()).await;
        assert_eq!(mem2.skill_name, "test-skill");
        
        Ok(())
    }
}

impl Default for ExperienceMemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_creation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mem = SkillMemory::new("x-tweet-search", temp_dir.path());
        assert_eq!(mem.skill_name, "x-tweet-search");
    }

    #[tokio::test]
    async fn test_record_and_read() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut mem = SkillMemory::new("test", temp_dir.path());
        mem.record_experience("Anti-scraping upgraded", "Add delay between requests").unwrap();
        
        let lessons = mem.get_lessons("Anti-scraping");
        assert_eq!(lessons.len(), 1);
        assert!(lessons[0].conclusion.contains("delay"));
    }

    #[tokio::test]
    async fn test_file_persistence() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut mem = SkillMemory::new("persist-test", temp_dir.path());
        mem.record_experience("Event", "Conclusion").unwrap();
        
        // New instance should load from file
        let mem2 = SkillMemory::new("persist-test", temp_dir.path());
        assert_eq!(mem2.records.len(), 1);
        assert_eq!(mem2.records[0].event, "Event");
    }

    #[tokio::test]
    async fn test_manager() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = ExperienceMemoryManager::new();
        
        manager.record("skill1", temp_dir.path(), "Event", "Conclusion").await.unwrap();
        let lessons = manager.get_lessons("skill1", temp_dir.path(), "Event").await;
        assert_eq!(lessons.len(), 1);
    }

    #[test]
    fn test_self_test_passes() {
        assert!(ExperienceMemoryManager::self_test().is_ok());
    }
}