//! App State - Immutable application state (Redux pattern)

#[allow(unused_imports)]
use std::collections::{HashMap, HashSet, VecDeque};
#[allow(unused_imports)]
use std::path::PathBuf;
#[allow(unused_imports)]
use std::time::Instant;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use chrono::{DateTime, Utc};
#[allow(unused_imports)]
use uuid::Uuid;

pub mod session;
pub mod chat;
pub mod input;
pub mod ui;
pub mod config;
pub mod meta;

pub use session::*;
pub use chat::*;
pub use input::*;
pub use ui::*;
pub use config::*;
pub use meta::*;

/// 顶层不可变应用状态 (Redux pattern)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub sessions: SessionState,
    pub chat: ChatState,
    pub input: InputState,
    pub ui: UIState,
    pub config: RuntimeConfig,
    pub meta: MetaState,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            sessions: SessionState::default(),
            chat: ChatState::default(),
            input: InputState::default(),
            ui: UIState::default(),
            config: RuntimeConfig::default(),
            meta: MetaState::default(),
        }
    }
}

impl AppState {
    /// 创建新的应用状态
    pub fn new(config: RuntimeConfig) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    /// 获取当前活跃会话
    pub fn active_session(&self) -> Option<&Session> {
        self.sessions.active_id.as_ref().and_then(|id| self.sessions.sessions.get(id))
    }

    /// 获取活跃会话的可变引用（用于reducer内部）
    pub fn active_session_mut(&mut self) -> Option<&mut Session> {
        if let Some(id) = self.sessions.active_id.clone() {
            self.sessions.sessions.get_mut(&id)
        } else {
            None
        }
    }
}
