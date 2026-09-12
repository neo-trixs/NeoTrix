use std::collections::HashMap;

/// 5 色符文插槽
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum RuneColor {
    /// Crimson(数据) — 数据摄取
    Crimson,
    /// Indigo(变换) — 变换处理
    Indigo,
    /// Obsidian(缓存) — 缓存加速
    Obsidian,
    /// Golden(错误) — 错误恢复
    Golden,
    /// Alabaster(监控) — 监控可观测
    Alabaster,
}

impl RuneColor {
    pub fn label(&self) -> &str {
        match self {
            Self::Crimson => "Crimson (数据)",
            Self::Indigo => "Indigo (变换)",
            Self::Obsidian => "Obsidian (缓存)",
            Self::Golden => "Golden (错误)",
            Self::Alabaster => "Alabaster (监控)",
        }
    }

    pub fn all() -> Vec<RuneColor> {
        vec![
            Self::Crimson,
            Self::Indigo,
            Self::Obsidian,
            Self::Golden,
            Self::Alabaster,
        ]
    }
}

/// 符文插槽 — 每个模块有固定的 rune 槽位
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RuneSlot {
    pub color: RuneColor,
    pub rune_id: Option<String>,
    /// 槽位是否已插入符文
    pub occupied: bool,
}

impl RuneSlot {
    pub fn new(color: RuneColor) -> Self {
        Self {
            color,
            rune_id: None,
            occupied: false,
        }
    }

    pub fn insert(&mut self, rune_id: impl Into<String>) {
        self.rune_id = Some(rune_id.into());
        self.occupied = true;
    }

    pub fn remove(&mut self) {
        self.rune_id = None;
        self.occupied = false;
    }
}

/// 符文 — 插入插槽的单一能力单元
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Rune {
    pub id: String,
    pub name: String,
    pub color: RuneColor,
    pub power: f64,
    pub tags: Vec<String>,
}

impl Rune {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        color: RuneColor,
        power: f64,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            color,
            power: power.clamp(0.0, 1.0),
            tags: Vec::new(),
        }
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

/// Runeword 组合效应 — 特定符文组合触发额外效果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Runeword {
    pub name: String,
    pub required_colors: Vec<RuneColor>,
    pub bonus_power: f64,
    pub description: String,
    /// 组合是否激活
    pub active: bool,
}

impl Runeword {
    pub fn new(
        name: impl Into<String>,
        required_colors: Vec<RuneColor>,
        bonus_power: f64,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            required_colors,
            bonus_power: bonus_power.clamp(0.0, 2.0),
            description: description.into(),
            active: false,
        }
    }

    /// 检查是否满足组合条件
    pub(crate) fn _check_active(&self, slots: &[RuneSlot]) -> bool {
        let inserted_colors: Vec<RuneColor> = slots
            .iter()
            .filter(|s| s.occupied)
            .map(|s| s.color)
            .collect();
        for color in &self.required_colors {
            if !inserted_colors.contains(color) {
                return false;
            }
        }
        true
    }
}

/// 模块符文配置 — 每个模块拥有一组 RuneSlot
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModuleRunes {
    pub module_id: String,
    pub slots: Vec<RuneSlot>,
    pub runewords: Vec<Runeword>,
}

impl ModuleRunes {
    pub fn new(module_id: impl Into<String>, slot_colors: Vec<RuneColor>) -> Self {
        Self {
            module_id: module_id.into(),
            slots: slot_colors.iter().map(|c| RuneSlot::new(*c)).collect(),
            runewords: Vec::new(),
        }
    }

    pub(crate) fn _add_runeword(&mut self, rw: Runeword) {
        self.runewords.push(rw);
    }

    /// 插入符文到指定颜色槽位
    pub fn insert_rune(&mut self, color: RuneColor, rune: &Rune) -> Result<(), String> {
        if rune.color != color {
            return Err(format!("Rune color {:?} does not match slot color {:?}", rune.color, color));
        }
        if let Some(slot) = self.slots.iter_mut().find(|s| s.color == color) {
            slot.insert(rune.id.clone());
            return Ok(());
        }
        Err(format!("No slot for {:?}", color))
    }

    /// 移除符文
    pub fn _remove_rune(&mut self, color: &RuneColor) {
        if let Some(slot) = self.slots.iter_mut().find(|s| s.color == *color) {
            slot.remove();
        }
    }

    /// 检查所有 Runeword 并更新激活状态
    pub(crate) fn _check_runewords(&mut self) {
        for rw in &mut self.runewords {
            rw.active = rw._check_active(&self.slots);
        }
    }

    /// 获取激活的 Runeword 总加成
    pub fn total_bonus(&self) -> f64 {
        self.runewords
            .iter()
            .filter(|rw| rw.active)
            .map(|rw| rw.bonus_power)
            .sum()
    }

    /// 获取激活的符文颜色列表
    pub(crate) fn _active_colors(&self) -> Vec<RuneColor> {
        self.slots
            .iter()
            .filter(|s| s.occupied)
            .map(|s| s.color)
            .collect()
    }

    /// 获取指定颜色的槽位引用
    pub fn slot(&self, color: &RuneColor) -> Option<&RuneSlot> {
        self.slots.iter().find(|s| s.color == *color)
    }

    pub(crate) fn _slot_mut(&mut self, color: &RuneColor) -> Option<&mut RuneSlot> {
        self.slots.iter_mut().find(|s| s.color == *color)
    }
}

/// 符文系统 — 管理所有模块的符文配置
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RuneSystem {
    pub modules: HashMap<String, ModuleRunes>,
    pub global_runewords: Vec<Runeword>,
}

impl RuneSystem {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            global_runewords: Vec::new(),
        }
    }

    pub fn register_module(&mut self, module_id: impl Into<String>, slot_colors: Vec<RuneColor>) {
        let id = module_id.into();
        self.modules.insert(id.clone(), ModuleRunes::new(id, slot_colors));
    }

    pub(crate) fn _add_global_runeword(&mut self, rw: Runeword) {
        self.global_runewords.push(rw);
    }

    pub fn module(&self, module_id: &str) -> Option<&ModuleRunes> {
        self.modules.get(module_id)
    }

    pub(crate) fn _module_mut(&mut self, module_id: &str) -> Option<&mut ModuleRunes> {
        self.modules.get_mut(module_id)
    }

    /// 插入符文到指定模块
    pub fn insert_rune(
        &mut self,
        module_id: &str,
        color: RuneColor,
        rune: &Rune,
    ) -> Result<(), String> {
        if let Some(mod_runes) = self.modules.get_mut(module_id) {
            mod_runes.insert_rune(color, rune)?;
            mod_runes._check_runewords();
            return Ok(());
        }
        Err(format!("Module {} not found", module_id))
    }

    /// 检查所有模块的 Runeword
    pub fn check_all_runewords(&mut self) {
        for mod_runes in self.modules.values_mut() {
            mod_runes._check_runewords();
        }
        for rw in &mut self.global_runewords {
            rw.active = rw._check_active(&Vec::new());
        }
    }

    /// 获取所有激活的 Runeword 总加成
    pub fn total_bonus(&self) -> f64 {
        let module_bonus: f64 = self
            .modules
            .values()
            .map(|m| m.total_bonus())
            .sum();
        let global_bonus: f64 = self
            .global_runewords
            .iter()
            .filter(|rw| rw.active)
            .map(|rw| rw.bonus_power)
            .sum();
        module_bonus + global_bonus
    }

    /// 移除符文
    pub(crate) fn _remove_rune(&mut self, module_id: &str, color: &RuneColor) {
        if let Some(mod_runes) = self.modules.get_mut(module_id) {
            mod_runes._remove_rune(color);
            mod_runes._check_runewords();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rune_color_labels() {
        assert_eq!(RuneColor::Crimson.label(), "Crimson (数据)");
        assert_eq!(RuneColor::Indigo.label(), "Indigo (变换)");
        assert_eq!(RuneColor::Obsidian.label(), "Obsidian (缓存)");
        assert_eq!(RuneColor::Golden.label(), "Golden (错误)");
        assert_eq!(RuneColor::Alabaster.label(), "Alabaster (监控)");
    }

    #[test]
    fn test_rune_slot_insert_remove() {
        let mut slot = RuneSlot::new(RuneColor::Crimson);
        assert!(!slot.occupied);
        slot.insert("rune-1");
        assert!(slot.occupied);
        assert_eq!(slot.rune_id, Some("rune-1".to_string()));
        slot.remove();
        assert!(!slot.occupied);
    }

    #[test]
    fn test_rune_creation() {
        let rune = Rune::new("r1", "Fire", RuneColor::Crimson, 0.8);
        assert_eq!(rune.id, "r1");
        assert_eq!(rune.power, 0.8);
        assert!(rune.tags.is_empty());
    }

    #[test]
    fn test_rune_with_tag() {
        let rune = Rune::new("r1", "Fire", RuneColor::Crimson, 0.5)
            .with_tag("burn")
            .with_tag("damage");
        assert_eq!(rune.tags.len(), 2);
        assert!(rune.tags.contains(&"burn".to_string()));
    }

    #[test]
    fn test_module_runes_new() {
        let mr = ModuleRunes::new("nt-core", vec![RuneColor::Crimson, RuneColor::Indigo]);
        assert_eq!(mr.slots.len(), 2);
        assert!(mr.slots[0].color == RuneColor::Crimson);
        assert!(mr.slots[1].color == RuneColor::Indigo);
    }

    #[test]
    fn test_module_runes_insert_rune() {
        let mut mr = ModuleRunes::new("nt-core", vec![RuneColor::Crimson, RuneColor::Indigo]);
        let rune = Rune::new("r1", "Fire", RuneColor::Crimson, 0.8);
        mr.insert_rune(RuneColor::Crimson, &rune).unwrap();
        assert!(mr.slot(&RuneColor::Crimson).unwrap().occupied);
    }

    #[test]
    fn test_module_runes_insert_rune_wrong_color() {
        let mut mr = ModuleRunes::new("nt-core", vec![RuneColor::Crimson]);
        let rune = Rune::new("r1", "Fire", RuneColor::Indigo, 0.8);
        assert!(mr.insert_rune(RuneColor::Crimson, &rune).is_err());
    }

    #[test]
    fn test_runeword_check_active() {
        let rw = Runeword::new(
            "Crimson-Indigo Combo",
            vec![RuneColor::Crimson, RuneColor::Indigo],
            0.5,
            "Combo effect",
        );
        let mut mr = ModuleRunes::new("nt-core", vec![RuneColor::Crimson, RuneColor::Indigo]);
        let rune1 = Rune::new("r1", "Fire", RuneColor::Crimson, 0.8);
        let rune2 = Rune::new("r2", "Shift", RuneColor::Indigo, 0.7);
        mr.insert_rune(RuneColor::Crimson, &rune1).unwrap();
        mr.insert_rune(RuneColor::Indigo, &rune2).unwrap();
        mr._add_runeword(rw);
        mr._check_runewords();
        assert!(mr.runewords[0].active);
        assert_eq!(mr.total_bonus(), 0.5);
    }

    #[test]
    fn test_runeword_not_active_without_all_colors() {
        let rw = Runeword::new(
            "Crimson-Indigo Combo",
            vec![RuneColor::Crimson, RuneColor::Indigo],
            0.5,
            "Combo effect",
        );
        let mut mr = ModuleRunes::new("nt-core", vec![RuneColor::Crimson, RuneColor::Indigo]);
        let rune1 = Rune::new("r1", "Fire", RuneColor::Crimson, 0.8);
        mr.insert_rune(RuneColor::Crimson, &rune1).unwrap();
        mr._add_runeword(rw);
        mr._check_runewords();
        assert!(!mr.runewords[0].active);
        assert_eq!(mr.total_bonus(), 0.0);
    }

    #[test]
    fn test_rune_system_register_and_insert() {
        let mut rs = RuneSystem::new();
        rs.register_module(
            "nt-core",
            vec![RuneColor::Crimson, RuneColor::Indigo, RuneColor::Obsidian],
        );
        let rune = Rune::new("r1", "Fire", RuneColor::Crimson, 0.8);
        rs.insert_rune("nt-core", RuneColor::Crimson, &rune).unwrap();
        assert!(rs.module("nt-core").unwrap().slot(&RuneColor::Crimson).unwrap().occupied);
    }

    #[test]
    fn test_rune_system_total_bonus() {
        let mut rs = RuneSystem::new();
        rs.register_module(
            "nt-core",
            vec![RuneColor::Crimson, RuneColor::Indigo],
        );
        let rw = Runeword::new(
            "Test Combo",
            vec![RuneColor::Crimson, RuneColor::Indigo],
            0.5,
            "Test",
        );
        if let Some(mod_runes) = rs._module_mut("nt-core") {
            mod_runes._add_runeword(rw);
        }
        let rune1 = Rune::new("r1", "Fire", RuneColor::Crimson, 0.8);
        let rune2 = Rune::new("r2", "Shift", RuneColor::Indigo, 0.7);
        rs.insert_rune("nt-core", RuneColor::Crimson, &rune1).unwrap();
        rs.insert_rune("nt-core", RuneColor::Indigo, &rune2).unwrap();
        rs.check_all_runewords();
        assert_eq!(rs.total_bonus(), 0.5);
    }

    #[test]
    fn test_remove_rune() {
        let mut rs = RuneSystem::new();
        rs.register_module("nt-core", vec![RuneColor::Crimson]);
        let rune = Rune::new("r1", "Fire", RuneColor::Crimson, 0.8);
        rs.insert_rune("nt-core", RuneColor::Crimson, &rune).unwrap();
        assert!(rs.module("nt-core").unwrap().slot(&RuneColor::Crimson).unwrap().occupied);
        rs._remove_rune("nt-core", &RuneColor::Crimson);
        assert!(!rs.module("nt-core").unwrap().slot(&RuneColor::Crimson).unwrap().occupied);
    }

    #[test]
    fn test_active_colors() {
        let mut mr = ModuleRunes::new("nt-core", vec![RuneColor::Crimson, RuneColor::Indigo]);
        let rune1 = Rune::new("r1", "Fire", RuneColor::Crimson, 0.8);
        mr.insert_rune(RuneColor::Crimson, &rune1).unwrap();
        let colors = mr._active_colors();
        assert_eq!(colors.len(), 1);
        assert!(colors.contains(&RuneColor::Crimson));
    }

    #[test]
    fn test_all_rune_colors() {
        let colors = RuneColor::all();
        assert_eq!(colors.len(), 5);
    }
}