//! Pass-by-Reference 工具结果 — 活对象预览
//!
//! 工具结果不序列化完整内容，而是提供引用和预览。

#![allow(dead_code)]

use std::collections::HashMap;

/// 引用 ID
pub type RefId = String;

/// 活对象引用
pub struct LiveReference {
    pub id: RefId,
    pub object_type: String,
    pub preview: String,
    pub full_size: usize,
    pub token_count: usize,
    pub created_at: i64,
}

/// 引用管理器 — 管理活对象引用
pub struct ReferenceManager {
    references: HashMap<RefId, LiveReference>,
    next_id: u32,
}

impl ReferenceManager {
    pub fn new() -> Self {
        Self {
            references: HashMap::new(),
            next_id: 0,
        }
    }

    /// 创建引用（只存预览，不存完整内容）
    pub fn create_ref(
        &mut self,
        object_type: &str,
        full_content: &str,
        preview_chars: usize,
    ) -> RefId {
        self.next_id += 1;
        let id = format!("ref_{}", self.next_id);
        let preview = if full_content.len() > preview_chars {
            format!(
                "{}...[{} chars]",
                &full_content[..preview_chars],
                full_content.len()
            )
        } else {
            full_content.to_string()
        };

        self.references.insert(
            id.clone(),
            LiveReference {
                id: id.clone(),
                object_type: object_type.to_string(),
                preview,
                full_size: full_content.len(),
                token_count: full_content.len() / 4,
                created_at: now_ts(),
            },
        );

        id
    }

    /// 获取预览
    pub fn preview(&self, id: &RefId) -> Option<&str> {
        self.references.get(id).map(|r| r.preview.as_str())
    }

    /// 获取引用元数据
    pub fn metadata(&self, id: &RefId) -> Option<(&str, usize, usize)> {
        self.references
            .get(id)
            .map(|r| (r.object_type.as_str(), r.full_size, r.token_count))
    }

    /// 统计
    pub fn stats(&self) -> (usize, usize) {
        let total_refs = self.references.len();
        let total_tokens: usize = self.references.values().map(|r| r.token_count).sum();
        (total_refs, total_tokens)
    }

    /// 清理过期引用
    pub fn prune(&mut self, max_age_secs: i64) {
        let now = now_ts();
        self.references
            .retain(|_, r| now - r.created_at < max_age_secs);
    }
}

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time after UNIX epoch")
        .as_secs() as i64
}
