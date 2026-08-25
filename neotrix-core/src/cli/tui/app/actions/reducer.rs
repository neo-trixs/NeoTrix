//! Reducer - Pure state transitions (Redux pattern)

pub use self::reducers::*;

use super::*;

/// 纯函数 Reducer: (State, Action) -> State
pub fn reduce(state: AppState, action: Action) -> AppState {
    use crate::cli::tui::app::actions::Action;
    match action {
        // === Input Actions ===
        Action::InputInsert(c) => reducers::reduce_input_insert(state, c),
        Action::InputDeleteBackward => reducers::reduce_input_delete_backward(state),
        Action::InputDeleteForward => reducers::reduce_input_delete_forward(state),
        Action::InputMoveCursorLeft => reducers::reduce_input_move_cursor_left(state),
        Action::InputMoveCursorRight => reducers::reduce_input_move_cursor_right(state),
        Action::InputMoveCursorHome => reducers::reduce_input_move_cursor_home(state),
        Action::InputMoveCursorEnd => reducers::reduce_input_move_cursor_end(state),
        Action::InputMoveWordForward => reducers::reduce_input_move_word_forward(state),
        Action::InputMoveWordBackward => reducers::reduce_input_move_word_backward(state),
        Action::InputClear => reducers::reduce_input_clear(state),
        Action::InputSetText(text) => reducers::reduce_input_set_text(state, text),
        Action::InputToggleMultiLine => reducers::reduce_input_toggle_multiline(state),
        Action::InputToggleSearch => reducers::reduce_input_toggle_search(state),
        Action::InputSearchQuery(q) => reducers::reduce_input_search_query(state, q),
        Action::InputSearchNext => reducers::reduce_input_search_next(state),
        Action::InputSearchPrev => reducers::reduce_input_search_prev(state),
        Action::InputHistoryPrev => reducers::reduce_input_history_prev(state),
        Action::InputHistoryNext => reducers::reduce_input_history_next(state),
        Action::InputSubmit => reducers::reduce_input_submit(state),
        Action::InputCancel => reducers::reduce_input_cancel(state),

        // === Session Actions ===
        Action::SessionNew(name) => reducers::reduce_session_new(state, name),
        Action::SessionSwitch(id) => reducers::reduce_session_switch(state, id),
        Action::SessionDelete(id) => reducers::reduce_session_delete(state, id),
        Action::SessionRename(id, name) => reducers::reduce_session_rename(state, id, name),
        Action::SessionClear => reducers::reduce_session_clear(state),
        Action::SessionCompact(keep) => reducers::reduce_session_compact(state, keep),
        Action::SessionNewTab => reducers::reduce_session_new_tab(state),
        Action::SessionClose(id) => {
            reducers::reduce_session_close(state, id)
        }
        Action::SessionNext => reducers::reduce_session_next(state),
        Action::SessionPrev => reducers::reduce_session_prev(state),
        Action::SessionRenameCurrent(name) => {
            reducers::reduce_session_rename_current(state, name)
        }
        Action::SessionDuplicate => reducers::reduce_session_duplicate(state),

        // === Layout Actions ===
        Action::LayoutToggleSessions | Action::UIToggleSessionsPanel | Action::UIToggleSessions => {
            reducers::reduce_layout_toggle_sessions(state)
        }
        Action::LayoutToggleThinking | Action::UIToggleThinking | Action::UIToggleThinkingBlocks => {
            reducers::reduce_layout_toggle_thinking(state)
        }
        Action::LayoutToggleToolCalls
        | Action::UIToggleToolCalls
        | Action::UIToggleToolCallsPanel => reducers::reduce_layout_toggle_tool_calls(state),
        Action::LayoutToggleLineNumbers | Action::UIToggleLineNumbers => {
            reducers::reduce_layout_toggle_line_numbers(state)
        }
        Action::LayoutToggleWordWrap | Action::UIToggleWordWrap => {
            reducers::reduce_layout_toggle_word_wrap(state)
        }
        Action::LayoutToggleAutoScroll | Action::UIToggleAutoScroll => {
            reducers::reduce_layout_toggle_auto_scroll(state)
        }
        Action::UIToggleTools => reducers::reduce_layout_toggle_tools(state),

        // === Config ===
        Action::ConfigReset => reducers::reduce_config_reset(state),

        // === Theme ===
        Action::ThemeCycle | Action::ThemePresetCycle => reducers::reduce_theme_cycle(state),
        Action::ThemePresetSet(p) => reducers::reduce_theme_preset_set(state, p),

        // === Vim ===
        Action::VimModeEnter => reducers::reduce_vim_mode_enter(state),
        Action::VimModeExit => reducers::reduce_vim_mode_exit(state),

        // === System / No-op passthroughs ===
        _ => state,
    }
}

/// Reducer implementations module
mod reducers {
    use super::*;
    use crate::cli::tui::app::state::*;

    // === Input Reducers ===
    pub fn reduce_input_insert(mut state: AppState, c: char) -> AppState {
        state.input.insert_char(c);
        state
    }

    pub fn reduce_input_delete_backward(mut state: AppState) -> AppState {
        state.input.delete_char();
        state
    }

    pub fn reduce_input_delete_forward(mut state: AppState) -> AppState {
        let len = state.input.buffer.len();
        if state.input.cursor < len {
            state.input.buffer.remove(state.input.cursor);
        }
        state
    }

    pub fn reduce_input_move_cursor_left(mut state: AppState) -> AppState {
        state.input.move_cursor_left();
        state
    }

    pub fn reduce_input_move_cursor_right(mut state: AppState) -> AppState {
        state.input.move_cursor_right();
        state
    }

    pub fn reduce_input_move_cursor_home(mut state: AppState) -> AppState {
        state.input.cursor = 0;
        state
    }

    pub fn reduce_input_move_cursor_end(mut state: AppState) -> AppState {
        state.input.cursor = state.input.buffer.len();
        state
    }

    pub fn reduce_input_move_word_forward(mut state: AppState) -> AppState {
        let buf_len = state.input.buffer.len();
        let s = &state.input.buffer[state.input.cursor..];
        let mut advanced = 0;
        let mut in_word = false;
        for ch in s.chars() {
            if ch.is_whitespace() {
                if in_word {
                    break;
                }
            } else {
                in_word = true;
                advanced += ch.len_utf8();
            }
        }
        state.input.cursor = (state.input.cursor + advanced).min(buf_len);
        state
    }

    pub fn reduce_input_move_word_backward(mut state: AppState) -> AppState {
        let s = &state.input.buffer[..state.input.cursor];
        let mut pos = state.input.cursor;
        let mut in_word = false;
        for ch in s.chars().rev() {
            if ch.is_whitespace() {
                if in_word {
                    break;
                }
            } else {
                in_word = true;
                pos = pos.saturating_sub(ch.len_utf8());
            }
        }
        state.input.cursor = pos;
        state
    }

    pub fn reduce_input_clear(mut state: AppState) -> AppState {
        state.input.clear();
        state
    }

    pub fn reduce_input_set_text(mut state: AppState, text: String) -> AppState {
        state.input.set_text(text);
        state
    }

    pub fn reduce_input_toggle_multiline(mut state: AppState) -> AppState {
        state.input.multi_line = !state.input.multi_line;
        state
    }

    pub fn reduce_input_toggle_search(mut state: AppState) -> AppState {
        state.input.search_mode = !state.input.search_mode;
        state
    }

    pub fn reduce_input_search_query(mut state: AppState, query: String) -> AppState {
        // 搜索历史：从最近往前找匹配项
        let matches: Vec<usize> = state
            .input
            .history
            .iter()
            .enumerate()
            .rev()
            .filter(|(_, item)| item.contains(&query))
            .map(|(i, _)| i)
            .collect();
        state.input.search_index = matches.first().copied().unwrap_or(0);
        if let Some(&idx) = matches.first() {
            if let Some(item) = state.input.history.get(idx) {
                state.input.set_text(item.clone());
            }
        }
        state
    }

    pub fn reduce_input_search_next(mut state: AppState) -> AppState {
        if !state.input.history.is_empty() {
            state.input.search_index =
                (state.input.search_index + 1) % state.input.history.len();
            if let Some(item) = state.input.history.get(state.input.search_index) {
                let text = item.clone();
                state.input.set_text(text);
            }
        }
        state
    }

    pub fn reduce_input_search_prev(mut state: AppState) -> AppState {
        if !state.input.history.is_empty() {
            state.input.search_index = if state.input.search_index == 0 {
                state.input.history.len() - 1
            } else {
                state.input.search_index - 1
            };
            if let Some(item) = state.input.history.get(state.input.search_index) {
                let text = item.clone();
                state.input.set_text(text);
            }
        }
        state
    }

    pub fn reduce_input_history_prev(mut state: AppState) -> AppState {
        if state.input.history.is_empty() {
            return state;
        }
        let idx = state
            .input
            .history_index
            .map_or(state.input.history.len() - 1, |i| i.saturating_sub(1));
        if let Some(item) = state.input.history.get(idx) {
            let text = item.clone();
            state.input.set_text(text);
        }
        state.input.history_index = Some(idx);
        state
    }

    pub fn reduce_input_history_next(mut state: AppState) -> AppState {
        if state.input.history.is_empty() {
            return state;
        }
        let idx = state
            .input
            .history_index
            .map_or(0, |i| (i + 1).min(state.input.history.len() - 1));
        if let Some(item) = state.input.history.get(idx) {
            let text = item.clone();
            state.input.set_text(text);
        }
        state.input.history_index = Some(idx);
        state
    }

    pub fn reduce_input_submit(mut state: AppState) -> AppState {
        if !state.input.buffer.trim().is_empty() {
            let text = state.input.buffer.clone();
            state.input.history.push(text);
            state.input.clear();
        }
        state.input.history_index = None;
        state
    }

    pub fn reduce_input_cancel(mut state: AppState) -> AppState {
        state.input.clear();
        state.input.history_index = None;
        state
    }

    // === Session Reducers ===
    pub fn reduce_session_new(mut state: AppState, name: String) -> AppState {
        let session = Session::new(name);
        let id = session.id.clone();
        state.sessions.sessions.insert(id.clone(), session);
        state.sessions.active_id = Some(id.clone());
        state.sessions.order.push(id);
        state
    }

    pub fn reduce_session_switch(mut state: AppState, id: String) -> AppState {
        if state.sessions.sessions.contains_key(&id) {
            state.sessions.active_id = Some(id);
        }
        state
    }

    pub fn reduce_session_delete(mut state: AppState, id: String) -> AppState {
        state.sessions.sessions.remove(&id);
        state.sessions.order.retain(|x| x != &id);
        if state.sessions.active_id.as_ref() == Some(&id) {
            state.sessions.active_id = state.sessions.order.first().cloned();
        }
        state
    }

    pub fn reduce_session_rename(mut state: AppState, id: String, name: String) -> AppState {
        if let Some(session) = state.sessions.sessions.get_mut(&id) {
            session.name = name;
        }
        state
    }

    pub fn reduce_session_clear(mut state: AppState) -> AppState {
        if let Some(id) = state.sessions.active_id.clone() {
            if let Some(session) = state.sessions.sessions.get_mut(&id) {
                session.messages.clear();
            }
        }
        state
    }

    pub fn reduce_session_compact(mut state: AppState, keep: usize) -> AppState {
        if let Some(id) = state.sessions.active_id.clone() {
            if let Some(session) = state.sessions.sessions.get_mut(&id) {
                while session.messages.len() > keep {
                    session.messages.pop_front();
                }
            }
        }
        state
    }

    pub fn reduce_session_new_tab(mut state: AppState) -> AppState {
        let session = Session::new(format!("Session {}", state.sessions.sessions.len() + 1));
        let id = session.id.clone();
        state.sessions.sessions.insert(id.clone(), session);
        state.sessions.order.push(id);
        state
    }

    pub fn reduce_session_close(state: AppState, id: String) -> AppState {
        super::reducers::reduce_session_delete(state, id)
    }

    pub fn reduce_session_next(mut state: AppState) -> AppState {
        if let Some(current) = state.sessions.active_id.clone() {
            if let Some(pos) = state.sessions.order.iter().position(|x| x == &current) {
                if pos + 1 < state.sessions.order.len() {
                    state.sessions.active_id = Some(state.sessions.order[pos + 1].clone());
                }
            }
        }
        state
    }

    pub fn reduce_session_prev(mut state: AppState) -> AppState {
        if let Some(current) = state.sessions.active_id.clone() {
            if let Some(pos) = state.sessions.order.iter().position(|x| x == &current) {
                if pos > 0 {
                    state.sessions.active_id = Some(state.sessions.order[pos - 1].clone());
                }
            }
        }
        state
    }

    pub fn reduce_session_rename_current(mut state: AppState, name: String) -> AppState {
        if let Some(id) = state.sessions.active_id.clone() {
            if let Some(session) = state.sessions.sessions.get_mut(&id) {
                session.name = name;
            }
        }
        state
    }

    pub fn reduce_session_duplicate(mut state: AppState) -> AppState {
        if let Some(id) = state.sessions.active_id.clone() {
            if let Some(session) = state.sessions.sessions.get(&id).cloned() {
                let new_session = Session::new(format!("{} (copy)", session.name));
                let new_id = new_session.id.clone();
                state.sessions.sessions.insert(new_id.clone(), new_session);
                state.sessions.order.push(new_id);
            }
        }
        state
    }

    // === Layout Reducers ===
    pub fn reduce_layout_toggle_sessions(mut state: AppState) -> AppState {
        state.ui.show_sessions = !state.ui.show_sessions;
        state
    }

    pub fn reduce_layout_toggle_tools(mut state: AppState) -> AppState {
        state.ui.show_tools = !state.ui.show_tools;
        state
    }

    pub fn reduce_layout_toggle_thinking(mut state: AppState) -> AppState {
        state.ui.show_thinking = !state.ui.show_thinking;
        state
    }

    pub fn reduce_layout_toggle_tool_calls(mut state: AppState) -> AppState {
        state.ui.show_tool_calls = !state.ui.show_tool_calls;
        state
    }

    pub fn reduce_layout_toggle_line_numbers(mut state: AppState) -> AppState {
        state.config.show_line_numbers = !state.config.show_line_numbers;
        state
    }

    pub fn reduce_layout_toggle_word_wrap(mut state: AppState) -> AppState {
        state.config.word_wrap = !state.config.word_wrap;
        state
    }

    pub fn reduce_layout_toggle_auto_scroll(mut state: AppState) -> AppState {
        state.config.auto_scroll = !state.config.auto_scroll;
        state
    }

    // === Config Reducers ===
    pub fn reduce_config_reset(mut state: AppState) -> AppState {
        state.config = RuntimeConfig::default();
        state
    }

    // === Theme Reducers ===
    pub fn reduce_theme_cycle(mut state: AppState) -> AppState {
                state.config.theme_preset = match state.config.theme_preset {
            ThemePreset::Dark => ThemePreset::Light,
            ThemePreset::Light => ThemePreset::Gruvbox,
            ThemePreset::Gruvbox => ThemePreset::Nord,
            ThemePreset::Nord => ThemePreset::Dracula,
            ThemePreset::Dracula => ThemePreset::Solarized,
            ThemePreset::Solarized => ThemePreset::Dark,
        };
        state
    }

    pub fn reduce_theme_set(mut state: AppState, name: String) -> AppState {
                state.config.theme_preset = match name.as_str() {
            "light" => ThemePreset::Light,
            "gruvbox" => ThemePreset::Gruvbox,
            "nord" => ThemePreset::Nord,
            "dracula" => ThemePreset::Dracula,
            "solarized" => ThemePreset::Solarized,
            _ => ThemePreset::Dark,
        };
        state
    }

    pub fn reduce_theme_preset_cycle(state: AppState) -> AppState {
        super::reducers::reduce_theme_cycle(state)
    }

    pub fn reduce_theme_preset_set(
        mut state: AppState,
        preset: crate::cli::tui::app::state::config::ThemePreset,
    ) -> AppState {
        state.config.theme_preset = preset;
        state
    }

    // === Vim Mode Reducers ===
    pub fn reduce_vim_mode_enter(mut state: AppState) -> AppState {
        state.ui.vim_mode = true;
        state
    }

    pub fn reduce_vim_mode_exit(mut state: AppState) -> AppState {
        state.ui.vim_mode = false;
        state
    }

    #[allow(dead_code)]
    pub fn reduce_vim_operator(mut state: AppState, _op: char) -> AppState {
        state
    }

    #[allow(dead_code)]
    pub fn reduce_vim_count(mut state: AppState, _count: usize) -> AppState {
        state
    }
}
