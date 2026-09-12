use crate::engine::input::KeyCode;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputAction {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Tool1,
    Tool2,
    Tool3,
    Tool4,
    Tool5,
    Interact,
    Inventory,
    Crafting,
    Menu,
    Confirm,
    Cancel,
    Pause,
    Attack,
    Defend,
    UseItem,
    Sprint,
    ZoomIn,
    ZoomOut,
}

#[derive(Debug, Clone)]
pub struct InputBinding {
    pub keys: Vec<KeyCode>,
    pub gamepad_button: Option<u32>,
    pub mouse_button: Option<u32>,
}

pub struct InputMap {
    pub bindings: HashMap<InputAction, Vec<InputBinding>>,
    pub just_pressed: Vec<InputAction>,
    pub just_released: Vec<InputAction>,
    pub held: Vec<InputAction>,
}

impl InputMap {
    pub fn new() -> Self {
        let mut bindings = HashMap::new();

        bindings.insert(
            InputAction::MoveUp,
            vec![InputBinding {
                keys: vec![KeyCode::W, KeyCode::Up],
                gamepad_button: None,
                mouse_button: None,
            }],
        );
        bindings.insert(
            InputAction::MoveDown,
            vec![InputBinding {
                keys: vec![KeyCode::S, KeyCode::Down],
                gamepad_button: None,
                mouse_button: None,
            }],
        );
        bindings.insert(
            InputAction::MoveLeft,
            vec![InputBinding {
                keys: vec![KeyCode::A, KeyCode::Left],
                gamepad_button: None,
                mouse_button: None,
            }],
        );
        bindings.insert(
            InputAction::MoveRight,
            vec![InputBinding {
                keys: vec![KeyCode::D, KeyCode::Right],
                gamepad_button: None,
                mouse_button: None,
            }],
        );

        bindings.insert(
            InputAction::Tool1,
            vec![InputBinding {
                keys: vec![KeyCode::Num1],
                gamepad_button: None,
                mouse_button: None,
            }],
        );
        bindings.insert(
            InputAction::Tool2,
            vec![InputBinding {
                keys: vec![KeyCode::Num2],
                gamepad_button: None,
                mouse_button: None,
            }],
        );
        bindings.insert(
            InputAction::Tool3,
            vec![InputBinding {
                keys: vec![KeyCode::Num3],
                gamepad_button: None,
                mouse_button: None,
            }],
        );
        bindings.insert(
            InputAction::Tool4,
            vec![InputBinding {
                keys: vec![KeyCode::Num4],
                gamepad_button: None,
                mouse_button: None,
            }],
        );
        bindings.insert(
            InputAction::Tool5,
            vec![InputBinding {
                keys: vec![KeyCode::Num5],
                gamepad_button: None,
                mouse_button: None,
            }],
        );

        bindings.insert(
            InputAction::Interact,
            vec![InputBinding {
                keys: vec![KeyCode::E],
                gamepad_button: None,
                mouse_button: None,
            }],
        );
        bindings.insert(
            InputAction::Inventory,
            vec![InputBinding {
                keys: vec![KeyCode::I],
                gamepad_button: None,
                mouse_button: None,
            }],
        );
        bindings.insert(
            InputAction::Crafting,
            vec![InputBinding {
                keys: vec![KeyCode::C],
                gamepad_button: None,
                mouse_button: None,
            }],
        );
        bindings.insert(
            InputAction::Menu,
            vec![InputBinding {
                keys: vec![KeyCode::Escape],
                gamepad_button: None,
                mouse_button: None,
            }],
        );
        bindings.insert(
            InputAction::Confirm,
            vec![InputBinding {
                keys: vec![KeyCode::Enter, KeyCode::Space],
                gamepad_button: None,
                mouse_button: None,
            }],
        );
        bindings.insert(
            InputAction::Cancel,
            vec![InputBinding {
                keys: vec![KeyCode::Escape, KeyCode::Backspace],
                gamepad_button: None,
                mouse_button: None,
            }],
        );
        bindings.insert(
            InputAction::Pause,
            vec![InputBinding {
                keys: vec![KeyCode::P],
                gamepad_button: None,
                mouse_button: None,
            }],
        );
        bindings.insert(
            InputAction::Attack,
            vec![InputBinding {
                keys: vec![],
                gamepad_button: None,
                mouse_button: Some(0),
            }],
        );
        bindings.insert(
            InputAction::Defend,
            vec![InputBinding {
                keys: vec![KeyCode::Shift],
                gamepad_button: None,
                mouse_button: None,
            }],
        );
        bindings.insert(
            InputAction::UseItem,
            vec![InputBinding {
                keys: vec![KeyCode::F],
                gamepad_button: None,
                mouse_button: None,
            }],
        );
        bindings.insert(
            InputAction::Sprint,
            vec![InputBinding {
                keys: vec![KeyCode::Shift],
                gamepad_button: None,
                mouse_button: None,
            }],
        );
        bindings.insert(
            InputAction::ZoomIn,
            vec![InputBinding {
                keys: vec![KeyCode::Plus, KeyCode::Equals],
                gamepad_button: None,
                mouse_button: None,
            }],
        );
        bindings.insert(
            InputAction::ZoomOut,
            vec![InputBinding {
                keys: vec![KeyCode::Minus],
                gamepad_button: None,
                mouse_button: None,
            }],
        );

        Self {
            bindings,
            just_pressed: Vec::new(),
            just_released: Vec::new(),
            held: Vec::new(),
        }
    }

    pub fn is_action_active(&self, action: InputAction) -> bool {
        self.held.contains(&action) || self.just_pressed.contains(&action)
    }

    pub fn is_action_just_pressed(&self, action: InputAction) -> bool {
        self.just_pressed.contains(&action)
    }

    pub fn is_action_just_released(&self, action: InputAction) -> bool {
        self.just_released.contains(&action)
    }

    pub fn get_movement(&self) -> (f32, f32) {
        let mut dx = 0.0;
        let mut dy = 0.0;
        if self.is_action_active(InputAction::MoveUp) {
            dy -= 1.0;
        }
        if self.is_action_active(InputAction::MoveDown) {
            dy += 1.0;
        }
        if self.is_action_active(InputAction::MoveLeft) {
            dx -= 1.0;
        }
        if self.is_action_active(InputAction::MoveRight) {
            dx += 1.0;
        }
        if dx != 0.0 && dy != 0.0 {
            let len = (dx * dx + dy * dy).sqrt();
            dx /= len;
            dy /= len;
        }
        (dx, dy)
    }

    pub fn clear_frame(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    pub fn bind_action(&mut self, action: InputAction, binding: InputBinding) {
        self.bindings
            .entry(action)
            .or_default()
            .push(binding);
    }

    pub fn unbind_action(&mut self, action: InputAction) {
        self.bindings.remove(&action);
    }

    pub fn action_count(&self) -> usize {
        self.bindings.len()
    }

    pub fn has_binding(&self, action: InputAction) -> bool {
        self.bindings.contains_key(&action)
    }
}

impl Default for InputMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_bindings() {
        let map = InputMap::new();
        assert!(map.has_binding(InputAction::MoveUp));
        assert!(map.has_binding(InputAction::Attack));
        assert!(map.has_binding(InputAction::Interact));
        assert_eq!(map.action_count(), 22);
    }

    #[test]
    fn test_movement_normalized() {
        let mut map = InputMap::new();
        map.held.push(InputAction::MoveRight);
        map.held.push(InputAction::MoveDown);
        let (dx, dy) = map.get_movement();
        let len = (dx * dx + dy * dy).sqrt();
        assert!((len - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_movement_single_axis() {
        let mut map = InputMap::new();
        map.held.push(InputAction::MoveUp);
        let (dx, dy) = map.get_movement();
        assert_eq!(dx, 0.0);
        assert_eq!(dy, -1.0);
    }

    #[test]
    fn test_just_pressed() {
        let mut map = InputMap::new();
        assert!(!map.is_action_just_pressed(InputAction::Confirm));
        map.just_pressed.push(InputAction::Confirm);
        assert!(map.is_action_just_pressed(InputAction::Confirm));
        map.clear_frame();
        assert!(!map.is_action_just_pressed(InputAction::Confirm));
    }

    #[test]
    fn test_bind.unbind() {
        let mut map = InputMap::new();
        map.bind_action(
            InputAction::Tool1,
            InputBinding {
                keys: vec![KeyCode::Z],
                gamepad_button: None,
                mouse_button: None,
            },
        );
        assert_eq!(map.bindings.get(&InputAction::Tool1).unwrap().len(), 2);
        map.unbind_action(InputAction::Tool1);
        assert!(!map.has_binding(InputAction::Tool1));
    }
}
