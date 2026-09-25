// 输入系统 — 键盘映射 / 按下 / 单次触发
// 键位: WASD移动 E互动 空格攻击 B背包 C制作 X商店 G装备 Q任务 Tab切换 F5存档 F9读档 F1音量 F10暂停 I调试 M世界图 J百科 [/]百科翻页 V立体世界 P剧情实景

use std::collections::HashMap;
use macroquad::prelude::{KeyCode, is_key_down, is_mouse_button_pressed, mouse_position, MouseButton};

/// 键位绑定表（§10.3 可重绑；input-systems：action 命名 + 绑定即数据 + 冲突检测 + 持久化）
#[derive(Debug, Clone)]
pub struct BindingTable {
    map: HashMap<String, Vec<KeyCode>>,
}

/// 默认绑定（单源：引擎中性动作；游戏动作由各产品自建表或重绑接入）
const DEFAULT_BINDINGS: &[(&str, &[KeyCode])] = &[
    ("up", &[KeyCode::W, KeyCode::Up]),
    ("down", &[KeyCode::S, KeyCode::Down]),
    ("left", &[KeyCode::A, KeyCode::Left]),
    ("right", &[KeyCode::D, KeyCode::Right]),
    ("attack", &[KeyCode::Space]),
    ("interact", &[KeyCode::E]),
    ("pause", &[KeyCode::F10]),
    ("debug", &[KeyCode::I]),
];

/// 键名（持久化用；Debug 形如 "Space"，此处显式映射防上游改名）
/// （生产经 export 消费；重绑 UI 落子前按 tween.rs 惯例保留）
#[allow(dead_code)]
fn key_name(k: KeyCode) -> Option<&'static str> {
    Some(match k {
        KeyCode::W => "W", KeyCode::A => "A", KeyCode::S => "S", KeyCode::D => "D",
        KeyCode::E => "E", KeyCode::I => "I", KeyCode::Q => "Q", KeyCode::F => "F",
        KeyCode::B => "B", KeyCode::C => "C", KeyCode::X => "X", KeyCode::G => "G",
        KeyCode::H => "H", KeyCode::J => "J", KeyCode::K => "K", KeyCode::L => "L",
        KeyCode::M => "M", KeyCode::N => "N", KeyCode::O => "O", KeyCode::P => "P",
        KeyCode::R => "R", KeyCode::T => "T", KeyCode::U => "U", KeyCode::V => "V",
        KeyCode::Y => "Y", KeyCode::Z => "Z",
        KeyCode::Key1 => "1", KeyCode::Key2 => "2", KeyCode::Key3 => "3",
        KeyCode::Key4 => "4", KeyCode::Key5 => "5", KeyCode::Key6 => "6",
        KeyCode::Key7 => "7", KeyCode::Key8 => "8", KeyCode::Key9 => "9",
        KeyCode::Key0 => "0",
        KeyCode::Up => "Up", KeyCode::Down => "Down", KeyCode::Left => "Left",
        KeyCode::Right => "Right", KeyCode::Space => "Space", KeyCode::Tab => "Tab",
        KeyCode::F1 => "F1", KeyCode::F5 => "F5", KeyCode::F9 => "F9", KeyCode::F10 => "F10",
        KeyCode::LeftBracket => "LeftBracket", KeyCode::RightBracket => "RightBracket",
        _ => return None,
    })
}

fn parse_key(s: &str) -> Option<KeyCode> {
    Some(match s {
        "W" => KeyCode::W, "A" => KeyCode::A, "S" => KeyCode::S, "D" => KeyCode::D,
        "E" => KeyCode::E, "I" => KeyCode::I, "Q" => KeyCode::Q, "F" => KeyCode::F,
        "B" => KeyCode::B, "C" => KeyCode::C, "X" => KeyCode::X, "G" => KeyCode::G,
        "H" => KeyCode::H, "J" => KeyCode::J, "K" => KeyCode::K, "L" => KeyCode::L,
        "M" => KeyCode::M, "N" => KeyCode::N, "O" => KeyCode::O, "P" => KeyCode::P,
        "R" => KeyCode::R, "T" => KeyCode::T, "U" => KeyCode::U, "V" => KeyCode::V,
        "Y" => KeyCode::Y, "Z" => KeyCode::Z,
        "1" => KeyCode::Key1, "2" => KeyCode::Key2, "3" => KeyCode::Key3,
        "4" => KeyCode::Key4, "5" => KeyCode::Key5, "6" => KeyCode::Key6,
        "7" => KeyCode::Key7, "8" => KeyCode::Key8, "9" => KeyCode::Key9,
        "0" => KeyCode::Key0,
        "Up" => KeyCode::Up, "Down" => KeyCode::Down, "Left" => KeyCode::Left,
        "Right" => KeyCode::Right, "Space" => KeyCode::Space, "Tab" => KeyCode::Tab,
        "F1" => KeyCode::F1, "F5" => KeyCode::F5, "F9" => KeyCode::F9, "F10" => KeyCode::F10,
        "LeftBracket" => KeyCode::LeftBracket, "RightBracket" => KeyCode::RightBracket,
        _ => return None,
    })
}

/// （defaults/import 已生产消费；keys/rebind/export 为重绑 UI 的 authoring API，
/// 单测锁定，按 tween.rs 惯例保留待 UI 落子）
#[allow(dead_code)]
impl BindingTable {
    pub fn defaults() -> Self {
        Self {
            map: DEFAULT_BINDINGS
                .iter()
                .map(|(a, ks)| (a.to_string(), ks.to_vec()))
                .collect(),
        }
    }

    pub fn keys(&self, action: &str) -> &[KeyCode] {
        self.map.get(action).map_or(&[], Vec::as_slice)
    }

    /// 重绑：未知动作/空键/跨动作键冲突 → Err（同动作多键如 W/Up 不算冲突）
    pub fn rebind(&mut self, action: &str, keys: Vec<KeyCode>) -> Result<(), String> {
        if !self.map.contains_key(action) {
            return Err(format!("未知动作: {}", action));
        }
        if keys.is_empty() {
            return Err("至少绑定一个键".to_string());
        }
        for (other, ks) in &self.map {
            if other != action && ks.iter().any(|k| keys.contains(k)) {
                return Err(format!("键已被动作 '{}' 占用", other));
            }
        }
        self.map.insert(action.to_string(), keys);
        Ok(())
    }

    /// 导出可持久化形态（按动作排序，文件稳定）
    pub fn export(&self) -> Vec<(String, Vec<String>)> {
        let mut rows: Vec<(String, Vec<String>)> = self
            .map
            .iter()
            .map(|(a, ks)| {
                (a.clone(), ks.iter().filter_map(|k| key_name(*k).map(str::to_string)).collect())
            })
            .collect();
        rows.sort_by(|a, b| a.0.cmp(&b.0));
        rows
    }

    /// 导入：未知键名/空绑定/跨动作冲突 → Err
    pub fn import(rows: &[(String, Vec<String>)]) -> Result<Self, String> {
        let mut t = Self { map: HashMap::new() };
        for (a, names) in rows {
            let mut keys = Vec::new();
            for n in names {
                match parse_key(n) {
                    Some(k) => keys.push(k),
                    None => return Err(format!("未知键名: {}", n)),
                }
            }
            if keys.is_empty() {
                return Err(format!("动作 '{}' 无绑定键", a));
            }
            t.map.insert(a.clone(), keys);
        }
        let actions: Vec<String> = t.map.keys().cloned().collect();
        for i in 0..actions.len() {
            for j in (i + 1)..actions.len() {
                let (a, b) = (&actions[i], &actions[j]);
                if t.map[a].iter().any(|k| t.map[b].contains(k)) {
                    return Err(format!("键冲突: '{}' 与 '{}'", a, b));
                }
            }
        }
        Ok(t)
    }
}

#[derive(Debug, Clone)]
pub struct InputState {
    pressed: HashMap<String, bool>,
    just_pressed: HashMap<String, bool>,
    /// 鼠标：本帧左键点击位置（UI 点击交互用；键盘只留对话框）
    pub clicked: Option<(f32, f32)>,
    pub mouse: (f32, f32),
    /// 键位绑定（§10.3；默认表与旧硬编码一致，运行时可 rebind）
    pub bindings: BindingTable,
}

impl InputState {
    pub fn new() -> Self { Self { pressed: HashMap::new(), just_pressed: HashMap::new(), clicked: None, mouse: (0.0, 0.0), bindings: BindingTable::defaults() } }

    pub fn update(&mut self) {
        self.just_pressed.clear();
        // 绑定表驱动（默认表与旧硬编码逐键一致；同动作多键任一按下即 down）
        for (a, keys) in &self.bindings.map {
            let mut down = false;
            for k in keys {
                if is_key_down(*k) {
                    down = true;
                    break;
                }
            }
            let was = self.pressed.get(a).copied().unwrap_or(false);
            self.pressed.insert(a.clone(), down);
            if down && !was { self.just_pressed.insert(a.clone(), true); }
        }
        // 鼠标（每帧采样，供 UiButton 点击）
        self.mouse = mouse_position();
        self.clicked = if is_mouse_button_pressed(MouseButton::Left) { Some(self.mouse) } else { None };
    }

    /// 本帧左键是否点中矩形（x,y,w,h 屏幕坐标）
    pub fn clicked_in(&self, x: f32, y: f32, w: f32, h: f32) -> bool {
        match self.clicked {
            Some((mx, my)) => mx >= x && mx <= x + w && my >= y && my <= y + h,
            None => false,
        }
    }

    pub fn pressed(&self, a: &str) -> bool { self.pressed.get(a).copied().unwrap_or(false) }
    pub fn just_pressed(&self, a: &str) -> bool { self.just_pressed.get(a).copied().unwrap_or(false) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extended_vocabulary_roundtrips() {
        // 数字/补齐字母可持久化往返（游戏键位表依赖）
        for name in ["1", "2", "3", "0", "T", "H", "R", "L", "N", "P", "U", "Y", "Z"] {
            let k = parse_key(name).expect("可解析");
            assert_eq!(key_name(k), Some(name));
        }
        let rows = vec![("skill1".to_string(), vec!["1".to_string()])];
        let t = BindingTable::import(&rows).unwrap();
        assert_eq!(t.keys("skill1"), &[KeyCode::Key1]);
    }

    #[test]
    fn defaults_match_legacy_hardcoded() {
        let t = BindingTable::defaults();
        assert_eq!(t.keys("attack"), &[KeyCode::Space]);
        assert_eq!(t.keys("up"), &[KeyCode::W, KeyCode::Up]);
        assert_eq!(t.keys("pause"), &[KeyCode::F10]);
        assert!(t.keys("nope").is_empty());
        // 默认表自身无跨动作冲突
        let rows = t.export();
        assert!(BindingTable::import(&rows).is_ok());
    }

    #[test]
    fn rebind_ok_conflict_unknown() {
        let mut t = BindingTable::defaults();
        assert!(t.rebind("attack", vec![KeyCode::F]).is_ok());
        assert_eq!(t.keys("attack"), &[KeyCode::F]);
        // Space 已空出，interact 可接
        assert!(t.rebind("interact", vec![KeyCode::Space]).is_ok());
        // 冲突：attack 占着 F
        let err = t.rebind("debug", vec![KeyCode::F]).unwrap_err();
        assert!(err.contains("attack"), "意外: {}", err);
        assert!(t.rebind("void_action", vec![KeyCode::Z]).is_err());
        assert!(t.rebind("attack", vec![]).is_err());
    }

    #[test]
    fn export_import_roundtrip_and_bad_input() {
        let t = BindingTable::defaults();
        let rows = t.export();
        // 按动作排序，文件稳定
        let names: Vec<&str> = rows.iter().map(|(a, _)| a.as_str()).collect();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        assert_eq!(names, sorted);
        let t2 = BindingTable::import(&rows).unwrap();
        assert_eq!(t2.export(), rows);
        // 未知键名
        assert!(BindingTable::import(&[("attack".into(), vec!["Nope".into()])]).is_err());
        // 导入期冲突
        assert!(BindingTable::import(&[
            ("attack".into(), vec!["Space".into()]),
            ("interact".into(), vec!["Space".into()]),
        ])
        .is_err());
        assert!(parse_key("Bogus").is_none());
    }
}
