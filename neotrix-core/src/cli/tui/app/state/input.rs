use super::*;
#[allow(unused_imports)]
use std::collections::VecDeque;

/// 输入状态
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InputState {
    pub buffer: String,
    pub cursor: usize,
    pub multi_line: bool,
    pub history: Vec<String>,
    pub history_index: Option<usize>,
    pub search_mode: bool,
    pub search_query: String,
    pub search_results: Vec<usize>,
    pub search_index: usize,
}

impl InputState {
    pub fn insert_char(&mut self, c: char) {
        self.buffer.insert(self.cursor, c);
        self.cursor += 1;
    }

    pub fn delete_char(&mut self) -> bool {
        if self.cursor > 0 {
            self.buffer.remove(self.cursor - 1);
            self.cursor -= 1;
            true
        } else {
            false
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor < self.buffer.len() {
            self.cursor += 1;
        }
    }

    pub fn set_text(&mut self, text: String) {
        self.buffer = text;
        self.cursor = self.buffer.len();
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
    }
}
