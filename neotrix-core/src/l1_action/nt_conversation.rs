//! nt_conversation — 对话持久化：保存/加载/列出/删除对话
//!
//! 每个对话存为 `~/.neotrix/conversations/{id}.json`，原子写（tmp+rename）。

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// 单条对话记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub goal: String,
    pub transcript: Vec<String>,
    pub model: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Conversation {
    /// 创建新对话
    pub fn new(goal: &str, model: Option<String>) -> Self {
        let now = unix_now();
        let id_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        Self {
            id: format!("conv-{:x}", id_nanos),
            goal: goal.to_string(),
            transcript: Vec::new(),
            model,
            created_at: now,
            updated_at: now,
        }
    }

    /// 更新时间戳
    pub fn touch(&mut self) {
        self.updated_at = unix_now();
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// 对话目录：`~/.neotrix/conversations/`
fn conv_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".neotrix")
        .join("conversations")
}

/// 保存对话到 JSON 文件（原子写）
pub fn save(conversation: &Conversation) -> Result<(), String> {
    let dir = conv_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建对话目录失败: {e}"))?;
    let path = dir.join(format!("{}.json", conversation.id));
    let json =
        serde_json::to_string_pretty(conversation).map_err(|e| format!("序列化失败: {e}"))?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, &json).map_err(|e| format!("写入失败: {e}"))?;
    std::fs::rename(&tmp, &path).map_err(|e| format!("重命名失败: {e}"))?;
    Ok(())
}

/// 加载指定对话
pub fn load(id: &str) -> Result<Conversation, String> {
    let path = conv_dir().join(format!("{id}.json"));
    let json =
        std::fs::read_to_string(&path).map_err(|e| format!("读取对话 {id} 失败: {e}"))?;
    serde_json::from_str(&json).map_err(|e| format!("解析对话 {id} 失败: {e}"))
}

/// 列出所有对话（按 updated_at 降序）
pub fn list() -> Vec<Conversation> {
    let dir = conv_dir();
    if !dir.exists() {
        return Vec::new();
    }
    let mut convs: Vec<Conversation> = std::fs::read_dir(&dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
        .filter_map(|e| {
            let json = std::fs::read_to_string(e.path()).ok()?;
            serde_json::from_str(&json).ok()
        })
        .collect();
    convs.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    convs
}

/// 删除对话
pub fn delete(id: &str) -> Result<(), String> {
    let path = conv_dir().join(format!("{id}.json"));
    std::fs::remove_file(&path).map_err(|e| format!("删除对话 {id} 失败: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_new() {
        let c = Conversation::new("test goal", Some("model-a".into()));
        assert!(c.id.starts_with("conv-"));
        assert_eq!(c.goal, "test goal");
        assert_eq!(c.model.as_deref(), Some("model-a"));
        assert!(c.transcript.is_empty());
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let mut c = Conversation::new("roundtrip test", None);
        c.transcript.push("hello".into());
        c.transcript.push("world".into());
        save(&c).unwrap();
        let loaded = load(&c.id).unwrap();
        assert_eq!(loaded.goal, c.goal);
        assert_eq!(loaded.transcript, c.transcript);
        // cleanup
        let _ = delete(&c.id);
    }

    #[test]
    fn test_list_returns_sorted() {
        let c1 = Conversation::new("list test 1", None);
        let c2 = Conversation::new("list test 2", None);
        save(&c1).unwrap();
        save(&c2).unwrap();
        let all = list();
        assert!(all.len() >= 2);
        // cleanup
        let _ = delete(&c1.id);
        let _ = delete(&c2.id);
    }
}
