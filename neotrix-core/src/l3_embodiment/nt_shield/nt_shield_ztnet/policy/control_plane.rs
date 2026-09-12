//! C5: Control Plane — 控制平面
//!
//! 管理策略决策和会话建立

use std::collections::HashMap;

/// 控制平面消息
#[derive(Debug, Clone)]
pub enum _ControlMessage {
    /// 创建会话
    CreateSession {
        session_id: String,
        subject: String,
        resource: String,
    },
    /// 销毁会话
    DestroySession {
        session_id: String,
    },
    /// 更新策略
    UpdatePolicy {
        policy_id: String,
        rules: Vec<String>,
    },
    /// 心跳
    Heartbeat {
        timestamp: u64,
    },
}

/// 控制平面
pub struct _ControlPlane {
    sessions: HashMap<String, SessionInfo>,
}

/// 会话信息
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub session_id: String,
    pub subject: String,
    pub resource: String,
    pub active: bool,
}

impl _ControlPlane {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub fn _handle_message(&mut self, msg: _ControlMessage) -> Option<_ControlMessage> {
        match msg {
            _ControlMessage::CreateSession { session_id, subject, resource } => {
                self.sessions.insert(session_id.clone(), SessionInfo {
                    session_id: session_id.clone(),
                    subject,
                    resource,
                    active: true,
                });
                None
            }
            _ControlMessage::DestroySession { session_id } => {
                if let Some(session) = self.sessions.get_mut(&session_id) {
                    session.active = false;
                }
                None
            }
            _ => None,
        }
    }

    pub fn sessions(&self) -> &HashMap<String, SessionInfo> {
        &self.sessions
    }
}

impl Default for _ControlPlane {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_session() {
        let mut cp = _ControlPlane::new();
        let msg = _ControlMessage::CreateSession {
            session_id: "s1".into(),
            subject: "user1".into(),
            resource: "tunnel/test".into(),
        };

        cp._handle_message(msg);
        assert_eq!(cp.sessions().len(), 1);
    }
}
