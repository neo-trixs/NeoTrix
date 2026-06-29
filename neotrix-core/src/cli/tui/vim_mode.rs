use crossterm::event::{KeyCode, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VimMode {
    Normal,
    Insert,
    Command,
    Visual,
}

impl VimMode {
    pub fn is_normal(&self) -> bool {
        *self == VimMode::Normal
    }

    pub fn is_insert(&self) -> bool {
        *self == VimMode::Insert
    }

    pub fn label(&self) -> &str {
        match self {
            VimMode::Normal => "NORMAL",
            VimMode::Insert => "INSERT",
            VimMode::Command => "COMMAND",
            VimMode::Visual => "VISUAL",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VimAction {
    PassThrough,
    InsertChar(char),
    DeleteChar,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    MoveWordForward,
    MoveWordBack,
    MoveLineStart,
    MoveLineEnd,
    MovePageUp,
    MovePageDown,
    EnterInsertMode,
    EnterNormalMode,
    EnterVisualMode,
    Yank,
    Paste,
    Undo,
    Search(String),
    SwitchSession(usize),
    Quit,
    None,
}

pub struct VimModeManager {
    pub mode: VimMode,
    pub command_buffer: String,
    pub last_insert: String,
    pending_key: Option<KeyCode>,
    enabled: bool,
}

impl VimModeManager {
    pub fn new() -> Self {
        Self {
            mode: VimMode::Normal,
            command_buffer: String::new(),
            last_insert: String::new(),
            pending_key: None,
            enabled: false,
        }
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
        if !self.enabled {
            self.mode = VimMode::Normal;
            self.command_buffer.clear();
            self.pending_key = None;
        }
    }

    pub fn set_enabled(&mut self, val: bool) {
        self.enabled = val;
        if !val {
            self.mode = VimMode::Normal;
            self.command_buffer.clear();
            self.pending_key = None;
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> VimAction {
        match self.mode {
            VimMode::Command => self.handle_command_key(code),
            VimMode::Insert => self.handle_insert_key(code),
            VimMode::Normal => self.handle_normal_key(code, modifiers),
            VimMode::Visual => self.handle_visual_key(code),
        }
    }

    fn handle_command_key(&mut self, code: KeyCode) -> VimAction {
        match code {
            KeyCode::Enter => {
                let cmd = self.command_buffer.clone();
                self.command_buffer.clear();
                self.mode = VimMode::Normal;
                VimAction::Search(cmd)
            }
            KeyCode::Esc => {
                self.command_buffer.clear();
                self.mode = VimMode::Normal;
                VimAction::EnterNormalMode
            }
            KeyCode::Backspace => {
                self.command_buffer.pop();
                VimAction::DeleteChar
            }
            KeyCode::Char(c) => {
                self.command_buffer.push(c);
                VimAction::None
            }
            _ => VimAction::None,
        }
    }

    fn handle_insert_key(&mut self, code: KeyCode) -> VimAction {
        match code {
            KeyCode::Esc => {
                self.last_insert.clear();
                self.mode = VimMode::Normal;
                VimAction::EnterNormalMode
            }
            _ => VimAction::PassThrough,
        }
    }

    fn handle_normal_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> VimAction {
        if let Some(pending) = self.pending_key.take() {
            match pending {
                KeyCode::Char('g') if code == KeyCode::Char('g') => {
                    return VimAction::MoveLineStart;
                }
                KeyCode::Char('y') if code == KeyCode::Char('y') => {
                    return VimAction::Yank;
                }
                _ => {}
            }
        }

        match code {
            KeyCode::Char('j') if modifiers == KeyModifiers::NONE => {
                return VimAction::MoveDown;
            }
            KeyCode::Char('k') if modifiers == KeyModifiers::NONE => {
                return VimAction::MoveUp;
            }
            KeyCode::Char('h') if modifiers == KeyModifiers::NONE => {
                return VimAction::MoveLeft;
            }
            KeyCode::Char('l') if modifiers == KeyModifiers::NONE => {
                return VimAction::MoveRight;
            }
            KeyCode::Char('d') if modifiers == KeyModifiers::CONTROL => {
                return VimAction::MovePageDown;
            }
            KeyCode::Char('u') if modifiers == KeyModifiers::CONTROL => {
                return VimAction::MovePageUp;
            }
            KeyCode::Char('i') => {
                self.mode = VimMode::Insert;
                return VimAction::EnterInsertMode;
            }
            KeyCode::Char('a') => {
                self.mode = VimMode::Insert;
                return VimAction::EnterInsertMode;
            }
            KeyCode::Char(':') => {
                self.mode = VimMode::Command;
                self.command_buffer = String::from(":");
                return VimAction::None;
            }
            KeyCode::Char('/') => {
                self.mode = VimMode::Command;
                self.command_buffer = String::from("/");
                return VimAction::None;
            }
            KeyCode::Char('g') => {
                self.pending_key = Some(KeyCode::Char('g'));
                return VimAction::None;
            }
            KeyCode::Char('G') => {
                return VimAction::MoveLineEnd;
            }
            KeyCode::Char('n') => {
                return VimAction::None;
            }
            KeyCode::Char('N') => {
                return VimAction::None;
            }
            KeyCode::Char('y') => {
                self.pending_key = Some(KeyCode::Char('y'));
                return VimAction::None;
            }
            KeyCode::Char('p') => {
                return VimAction::Paste;
            }
            KeyCode::Char('w') => {
                return VimAction::MoveWordForward;
            }
            KeyCode::Char('b') => {
                return VimAction::MoveWordBack;
            }
            KeyCode::Char('0') => {
                return VimAction::MoveLineStart;
            }
            KeyCode::Char('$') => {
                return VimAction::MoveLineEnd;
            }
            KeyCode::Esc => {
                return VimAction::EnterNormalMode;
            }
            KeyCode::Char('q') => {
                return VimAction::Quit;
            }
            KeyCode::Tab => {
                return VimAction::SwitchSession(1);
            }
            KeyCode::BackTab => {
                return VimAction::SwitchSession(usize::MAX);
            }
            KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
                let n = c.to_digit(10).expect("char is ascii digit") as usize;
                return VimAction::SwitchSession(n.wrapping_sub(1));
            }
            KeyCode::Up => {
                return VimAction::MoveUp;
            }
            KeyCode::Down => {
                return VimAction::MoveDown;
            }
            KeyCode::Left => {
                return VimAction::MoveLeft;
            }
            KeyCode::Right => {
                return VimAction::MoveRight;
            }
            KeyCode::PageUp => {
                return VimAction::MovePageUp;
            }
            KeyCode::PageDown => {
                return VimAction::MovePageDown;
            }
            _ => {}
        }
        VimAction::PassThrough
    }

    fn handle_visual_key(&mut self, code: KeyCode) -> VimAction {
        match code {
            KeyCode::Esc | KeyCode::Char('v') => {
                self.mode = VimMode::Normal;
                VimAction::EnterNormalMode
            }
            KeyCode::Char('j') | KeyCode::Down => VimAction::MoveDown,
            KeyCode::Char('k') | KeyCode::Up => VimAction::MoveUp,
            _ => VimAction::None,
        }
    }
}

impl Default for VimModeManager {
    fn default() -> Self {
        Self::new()
    }
}
