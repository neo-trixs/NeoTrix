//! Actions - All user intents as enum (Redux pattern)

#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use std::collections::HashMap;
#[allow(unused_imports)]
use uuid::Uuid;
#[allow(unused_imports)]
use chrono::{DateTime, Utc};

#[allow(unused_imports)]
use crate::cli::tui::app::state::*;
#[allow(unused_imports)]
use crate::cli::tui::app::state::session::Session;
#[allow(unused_imports)]
use crate::cli::tui::app::state::chat::{ChatMessage, ToolCall, ThinkingBlock};
#[allow(unused_imports)]
use crate::cli::tui::app::state::ui::{Panel, Focus, VimMode, VimModeState};
#[allow(unused_imports)]
use crate::cli::tui::app::state::config::{ThemePreset, KeyMap};
#[allow(unused_imports)]
use crate::cli::tui::app::state::input::InputState;

// Reducer module
pub mod reducer;
pub use reducer::reduce;

/// 所有用户意图作为 Action 枚举 (Redux pattern)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    // === Input Actions ===
    InputInsert(char),
    InputDeleteBackward,
    InputDeleteForward,
    InputMoveCursorLeft,
    InputMoveCursorRight,
    InputMoveCursorHome,
    InputMoveCursorEnd,
    InputMoveWordForward,
    InputMoveWordBackward,
    InputClear,
    InputSetText(String),
    InputToggleMultiLine,
    InputToggleSearch,
    InputSearchQuery(String),
    InputSearchNext,
    InputSearchPrev,
    InputHistoryPrev,
    InputHistoryNext,
    InputSubmit,
    InputCancel,

    // === Session Actions ===
    SessionNew(String),
    SessionSwitch(String),
    SessionDelete(String),
    SessionRename(String, String),
    SessionClear,
    SessionCompact(usize),
    SessionExport(String),
    SessionImport(String),
    SessionList,
    SessionRestore(String),
    SessionNewTab,
    SessionClose(String),
    SessionNext,
    SessionPrev,
    SessionRenameCurrent(String),
    SessionDuplicate,
    SessionSave,
    SessionLoad(String),
    SessionListOpen,
    SessionPickerOpen,
    SessionPickerClose,
    SessionPickerSelect(String),
    SessionNewNamed(String),
    SessionCloseCurrent,
    SessionCloseOther,
    SessionCloseAll,
    SessionMoveLeft(usize),
    SessionMoveRight(usize),
    SessionPin(usize),
    SessionUnpin(usize),

    // === Chat Actions ===
    ChatSendMessage(String, Option<String>),
    ChatRetryLast,
    ChatRegenerateLast,
    ChatDeleteMessage(usize),
    ChatEditMessage(usize, String),
    ChatCopyMessage(usize),
    ChatToggleThinking(usize),
    ChatToggleToolCall(usize),
    ChatCopyCodeBlock(usize, usize),
    ChatCopyToolCall(usize),
    ChatClearHistory,
    ChatCompact(usize),

    // === Streaming Actions ===
    StreamStart(String, Option<String>),
    StreamChunk(String),
    StreamToolCallStart(String, String),
    StreamToolCallChunk(String, String),
    StreamToolCallEnd(String, String, bool),
    StreamEnd,
    StreamError(String),

    // === Tool Actions ===
    ToolCall(String, serde_json::Value),
    ToolResult(String, ToolResult),
    ToolApprovalAllow,
    ToolApprovalDeny,
    ToolExecute(String, serde_json::Value),
    ToolApprovePending,
    ToolDenyPending,

    // Search/History
    HistorySearchOpen,
    HistorySearchClose,
    HistorySearchQuery(String),
    HistorySearchNext,
    HistorySearchPrev,
    HistorySearchSelect(usize),

    // Theme/UI
    ThemeCycle,
    ThemeSet(String),
    ThemePresetCycle,
    ThemePresetSet(ThemePreset),
    FontSizeIncrease,
    FontSizeDecrease,
    FontReset,

    // Vim Mode
    VimModeEnter,
    VimModeExit,
    VimOperator(char),
    VimCount(usize),

    // Window/Layout - UI Toggles
    UIToggleVimMode,
    UIToggleSessionsPanel,
    UIToggleLineNumbers,
    UIToggleTools,
    UIToggleThinking,
    UIToggleToolCalls,
    UIToggleWordWrap,
    UIToggleAutoScroll,
    UIToggleThinkingBlocks,
    UIToggleToolCallsPanel,
    UIToggleTheme,
    UISetStatusMessage(String),
    UISetErrorMessage(String),
    UISetTheme(String),
    UIFocusInput,
    UIFocusChat,
    UIFocusSessions,
    UIToggleSessions,
    // Window/Layout
    LayoutToggleSessions,
    LayoutToggleTools,
    LayoutToggleThinking,
    LayoutToggleToolCalls,
    LayoutToggleLineNumbers,
    LayoutToggleWordWrap,
    LayoutToggleAutoScroll,
    LayoutSplitHorizontal,
    LayoutSplitVertical,
    LayoutClosePane,
    LayoutFocusNext,
    LayoutFocusPrev,
    LayoutFocusChat,
    LayoutFocusInput,
    LayoutFocusSessions,
    LayoutFocusTools,

    // Config
    ConfigReload,
    ConfigSave,
    ConfigReset,
    ConfigSet(String, String),

    // System
    Quit,
    Restart,
    Reset,
    Refresh,

    // Tick/NoOp
    Tick,
    NoOp,
}



impl Default for Action {
    fn default() -> Self {
        Action::NoOp
    }
}

impl Action {
    /// 是否为输入相关动作
    pub fn is_input_action(&self) -> bool {
        matches!(self,
            Action::InputInsert(_) |
            Action::InputDeleteBackward |
            Action::InputDeleteForward |
            Action::InputMoveCursorLeft |
            Action::InputMoveCursorRight |
            Action::InputMoveCursorHome |
            Action::InputMoveCursorEnd |
            Action::InputMoveWordForward |
            Action::InputMoveWordBackward |
            Action::InputClear |
            Action::InputSetText(_) |
            Action::InputToggleMultiLine |
            Action::InputToggleSearch |
            Action::InputSearchQuery(_) |
            Action::InputSearchNext |
            Action::InputSearchPrev |
            Action::InputHistoryPrev |
            Action::InputHistoryNext |
            Action::InputSubmit |
            Action::InputCancel
        )
    }

    /// 是否为会话相关动作
    pub fn is_session_action(&self) -> bool {
        matches!(self,
            Action::SessionNew(_) |
            Action::SessionSwitch(_) |
            Action::SessionDelete(_) |
            Action::SessionRename(_, _) |
            Action::SessionClear |
            Action::SessionCompact(_) |
            Action::SessionExport(_) |
            Action::SessionImport(_) |
            Action::SessionList |
            Action::SessionRestore(_) |
            Action::SessionNewTab |
            Action::SessionClose(_) |
            Action::SessionNext |
            Action::SessionPrev |
            Action::SessionRenameCurrent(_) |
            Action::SessionDuplicate |
            Action::SessionSave |
            Action::SessionLoad(_) |
            Action::SessionListOpen |
            Action::SessionPickerOpen |
            Action::SessionPickerClose |
            Action::SessionPickerSelect(_) |
            Action::SessionNewNamed(_) |
            Action::SessionCloseCurrent |
            Action::SessionCloseOther |
            Action::SessionCloseAll
        )
    }

    /// 是否为 UI 相关动作
    pub fn is_ui_action(&self) -> bool {
        matches!(self,
            Action::UIToggleSessionsPanel |
            Action::UIToggleThinking |
            Action::UIToggleToolCalls |
            Action::UIToggleLineNumbers |
            Action::UIToggleWordWrap |
            Action::UIToggleAutoScroll |
            Action::UIToggleVimMode |
            Action::UIToggleTheme |
            Action::UIToggleThinkingBlocks |
            Action::UIToggleToolCallsPanel |
            Action::UISetTheme(_) |
            Action::UISetStatusMessage(_) |
            Action::UISetErrorMessage(_) |
            Action::UIToggleVimMode |
            Action::UIToggleSessionsPanel |
            Action::UIToggleThinkingBlocks |
            Action::UIToggleToolCallsPanel |
            Action::UIToggleLineNumbers |
            Action::UISetStatusMessage(_) |
            Action::UISetErrorMessage(_)
        )
    }

    /// 是否为导航动作
    pub fn is_navigation(&self) -> bool {
        matches!(self,
            Action::InputMoveCursorLeft |
            Action::InputMoveCursorRight |
            Action::InputMoveCursorHome |
            Action::InputMoveCursorEnd |
            Action::InputMoveWordForward |
            Action::InputMoveWordBackward |
        )
    }

    /// 是否为编辑动作
    pub fn is_edit(&self) -> bool {
        matches!(self,
            Action::InputInsert(_) |
            Action::InputDeleteBackward |
            Action::InputDeleteForward |
            Action::InputClear |
            Action::InputSetText(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_categories() {
        assert!(Action::InputInsert('a').is_input_action());
        assert!(Action::SessionNew("test".into()).is_session_action());
        assert!(Action::UIToggleTheme.is_ui_action());
        assert!(Action::InputMoveCursorLeft.is_navigation());
        assert!(Action::InputInsert('a').is_edit());
    }
}
