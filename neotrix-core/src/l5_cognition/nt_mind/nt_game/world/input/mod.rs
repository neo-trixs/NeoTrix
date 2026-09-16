use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputAction {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Attack,
    Defend,
    Interact,
    Inventory,
    Menu,
    Confirm,
    Cancel,
    Skill1,
    Skill2,
    Skill3,
    Skill4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyBind {
    Key(u32),
    Mouse(u32),
    Gamepad(u32),
}

pub struct InputMap {
    bindings: HashMap<InputAction, Vec<KeyBind>>,
    just_pressed: Vec<InputAction>,
    held: Vec<InputAction>,
    just_released: Vec<InputAction>,
}

impl InputMap {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            just_pressed: Vec::new(),
            held: Vec::new(),
            just_released: Vec::new(),
        }
    }

    pub fn bind(&mut self, action: InputAction, key: KeyBind) {
        self.bindings.entry(action).or_default().push(key);
    }

    pub fn is_action_pressed(&self, action: InputAction) -> bool {
        self.just_pressed.contains(&action)
    }

    pub fn is_action_held(&self, action: InputAction) -> bool {
        self.held.contains(&action)
    }

    pub fn is_action_released(&self, action: InputAction) -> bool {
        self.just_released.contains(&action)
    }

    pub fn clear_frame(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    pub fn simulate_press(&mut self, action: InputAction) {
        self.just_pressed.push(action);
        self.held.push(action);
    }

    pub fn simulate_release(&mut self, action: InputAction) {
        self.held.retain(|a| *a != action);
        self.just_released.push(action);
    }

    pub fn default_keyboard() -> Self {
        let mut map = Self::new();
        map.bind(InputAction::MoveUp, KeyBind::Key(87));
        map.bind(InputAction::MoveUp, KeyBind::Key(38));
        map.bind(InputAction::MoveDown, KeyBind::Key(83));
        map.bind(InputAction::MoveDown, KeyBind::Key(40));
        map.bind(InputAction::MoveLeft, KeyBind::Key(65));
        map.bind(InputAction::MoveLeft, KeyBind::Key(37));
        map.bind(InputAction::MoveRight, KeyBind::Key(68));
        map.bind(InputAction::MoveRight, KeyBind::Key(39));
        map.bind(InputAction::Attack, KeyBind::Key(32));
        map.bind(InputAction::Interact, KeyBind::Key(69));
        map.bind(InputAction::Inventory, KeyBind::Key(73));
        map.bind(InputAction::Menu, KeyBind::Key(27));
        map
    }
}

impl Default for InputMap {
    fn default() -> Self {
        Self::new()
    }
}
