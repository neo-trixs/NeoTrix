//! Individual memory — 记忆连续性.
//!
//! 个体性 = 能力 × 记忆连续性。本模块只存指针（KB experience 的
//! session/cycle 引用 + skill 引用），不存正文；正文唯一出处是 KB hub。
//!
//! 对应 Python 侧经验协议：`~/.neotrix/pending-absorb.json` → 后台吸收。

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// 记忆链接（一只个体的全部跨会话记忆 = 指针集合）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryLink {
    /// KB experience 的 session_id 列表（按时间增序）
    pub sessions: Vec<String>,
    /// 引用的 skill 名
    pub skills: Vec<String>,
    /// 引用的 prompt/规则版本
    pub policy_refs: Vec<String>,
    /// 累计吸收的分支数
    pub branches_absorbed: u64,
}

impl MemoryLink {
    pub fn new() -> Self {
        Self::default()
    }

    /// 吸收一次会话的经验指针（幂等：同 session_id 不重复记）
    pub fn absorb_session(&mut self, session_id: &str, branches: u64) {
        if !self.sessions.iter().any(|s| s == session_id) {
            self.sessions.push(session_id.to_string());
            self.branches_absorbed += branches;
        }
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absorb_is_idempotent() {
        let mut m = MemoryLink::new();
        m.absorb_session("sess_a", 13);
        m.absorb_session("sess_a", 13);
        m.absorb_session("sess_b", 21);
        assert_eq!(m.session_count(), 2);
        assert_eq!(m.branches_absorbed, 34);
    }
}
