//! StS 卡牌游戏数据 schema — 卡牌/遗物/能力/药水/遭遇/事件/关键词 + 复刻库.
//!
//! 数据文件位于 `../data/*.ron`（手写原创 6 表 + spire-codex 导入 8 表）。
//! 纯逻辑 crate，无渲染依赖。

use serde::Deserialize;

/// 天气类型（worldmap 天气倾向 / weather 倾向票）
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum WeatherKind { Clear, Rain, Snow, Fog }

impl Default for WeatherKind {
    fn default() -> Self { WeatherKind::Clear }
}


/// 元素类型 — 五行系统 (伤害流水线用)
/// 金→木→土→水→火→金（循环相克）
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "PascalCase")]
pub enum ElementDef { Metal, Wood, Water, Fire, Earth }

impl ElementDef {
    /// 五行相克：攻击方 → 被克制方
    pub fn beats(self) -> ElementDef {
        match self {
            ElementDef::Metal => ElementDef::Wood,
            ElementDef::Wood => ElementDef::Earth,
            ElementDef::Earth => ElementDef::Water,
            ElementDef::Water => ElementDef::Fire,
            ElementDef::Fire => ElementDef::Metal,
        }
    }

    /// 五行被克：攻击方 → 克制方
    pub fn weakest_against(self) -> ElementDef {
        match self {
            ElementDef::Metal => ElementDef::Fire,
            ElementDef::Wood => ElementDef::Metal,
            ElementDef::Earth => ElementDef::Wood,
            ElementDef::Water => ElementDef::Earth,
            ElementDef::Fire => ElementDef::Water,
        }
    }

    /// 伤害倍率：攻击方元素 vs 防御方元素
    pub fn damage_multiplier(attacker: ElementDef, defender: ElementDef) -> f32 {
        if attacker.beats() == defender { 1.3 }   // 克制 +30%
        else if attacker.weakest_against() == defender { 0.7 }  // 被克 -30%
        else { 1.0 }  // 中立
    }
}
// ── 卡牌 (cards.ron, StS schema) ──

/// 卡牌类型
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum CardTypeDef { Attack, Skill, Power, Curse, Status }

/// 卡牌稀有度
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum CardRarityDef { Basic, Common, Uncommon, Rare, Special }

/// 卡牌目标
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum CardTargetDef { Enemy, AllEnemies, Own }

/// 卡牌效果
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum CardEffectDef {
    Damage(f32),
    DamageAll(f32),
    DamageHits(f32, u32),
    Block(f32),
    ApplyPower(String, i32),
    DrawCards(u32),
    GainEnergy(u32),
    Heal(f32),
}

/// 卡牌定义
#[derive(Debug, Clone, Deserialize)]
pub struct CardDef {
    pub id: String,
    pub name: String,
    pub cost: u32,
    pub card_type: CardTypeDef,
    pub rarity: CardRarityDef,
    pub target: CardTargetDef,
    #[serde(default)]
    pub element: Option<ElementDef>,
    #[serde(default)]
    pub effects: Vec<CardEffectDef>,
    #[serde(default)]
    pub upgrade_effects: Vec<CardEffectDef>,
    #[serde(default)]
    pub upgrade_note: String,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub color: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub upgraded_description: String,
}

pub fn load_cards() -> Vec<CardDef> {
    ron::from_str(include_str!("../data/cards.ron"))
        .unwrap_or_else(|e| { log::warn!("cards.ron 加载失败: {}", e); Vec::new() })
}

// ── 遗物 (relics.ron, StS schema) ──

/// 遗物稀有度
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum RelicRarityDef { Starter, Common, Uncommon, Rare, Boss, Shop, Event, Ancient }

/// 遗物效果
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum RelicEffectDef {
    HealAfterCombat(f32),
    StartEnergy(u32),
    FireDamageMult(f32),
    MetalDamageMult(f32),
    Thorns(f32),
    DrawExtra(u32),
    MaxHp(u32),
    EnergyMax(u32),
}

/// 遗物定义
#[derive(Debug, Clone, Deserialize)]
pub struct RelicDef {
    pub id: String,
    pub name: String,
    pub rarity: RelicRarityDef,
    #[serde(default)]
    pub pool: String,
    pub effect: RelicEffectDef,
    #[serde(default)]
    pub merchant_price: Option<u32>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub flavor: String,
}

pub fn load_relics() -> Vec<RelicDef> {
    ron::from_str(include_str!("../data/relics.ron"))
        .unwrap_or_else(|e| { log::warn!("relics.ron 加载失败: {}", e); Vec::new() })
}

// ── 能力 (powers.ron, StS schema) ──

/// 能力类型
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum PowerTypeDef { Buff, Debuff }

/// 堆叠方式
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum PowerStackDef { Counter, Single }

/// 能力定义
#[derive(Debug, Clone, Deserialize)]
pub struct PowerDef {
    pub id: String,
    pub name: String,
    pub power_type: PowerTypeDef,
    pub stack: PowerStackDef,
    #[serde(default)]
    pub description: String,
}

pub fn load_powers() -> Vec<PowerDef> {
    ron::from_str(include_str!("../data/powers.ron"))
        .unwrap_or_else(|e| { log::warn!("powers.ron 加载失败: {}", e); Vec::new() })
}

// ── 药水 (potions.ron, StS schema) ──

/// 药水稀有度
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum PotionRarityDef { Common, Uncommon, Rare }

/// 药水目标
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum PotionTargetDef { Enemy, Own }

impl Default for PotionTargetDef {
    fn default() -> Self { PotionTargetDef::Own }
}

/// 药水效果
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum PotionEffectDef {
    HealPercent(f32),
    DoubleNextElement,
    Stealth(u32),
    Strength(u32),
    Block(f32),
    Cleanse,
}

/// 药水定义
#[derive(Debug, Clone, Deserialize)]
pub struct PotionDef {
    pub id: String,
    pub name: String,
    pub rarity: PotionRarityDef,
    #[serde(default)]
    pub target: PotionTargetDef,
    #[serde(default)]
    pub effect: Option<PotionEffectDef>,
    #[serde(default)]
    pub pool: String,
    #[serde(default)]
    pub description: String,
}

pub fn load_potions() -> Vec<PotionDef> {
    ron::from_str(include_str!("../data/potions.ron"))
        .unwrap_or_else(|e| { log::warn!("potions.ron 加载失败: {}", e); Vec::new() })
}

// ── 遭遇 (encounters.ron, StS schema) ──

/// 房间类型
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum RoomTypeDef { Normal, Elite, Boss }

/// 遭遇定义
#[derive(Debug, Clone, Deserialize)]
pub struct EncounterDef {
    pub id: String,
    pub name: String,
    pub act: u32,
    pub room: RoomTypeDef,
    #[serde(default)]
    pub enemies: Vec<(String, f32, f32)>,
    #[serde(default)]
    pub weight: u32,
}

pub fn load_encounters() -> Vec<EncounterDef> {
    ron::from_str(include_str!("../data/encounters.ron"))
        .unwrap_or_else(|e| { log::warn!("encounters.ron 加载失败: {}", e); Vec::new() })
}

// ── 关键词 (keywords.ron, StS schema) ──

/// 关键词定义
#[derive(Debug, Clone, Deserialize)]
pub struct KeywordDef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
}

pub fn load_keywords() -> Vec<KeywordDef> {
    ron::from_str(include_str!("../data/keywords.ron"))
        .unwrap_or_else(|e| { log::warn!("keywords.ron 加载失败: {}", e); Vec::new() })
}

// ── StS 复刻库 (sts_*.ron，由 tools/import_codex.py 从 spire-codex 生成) ──

/// StS 怪物种类
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum MonsterKindDef { Normal, Elite, Boss }

/// StS 招式（含意图/双难伤害/多段/格挡）
#[derive(Debug, Clone, Deserialize)]
pub struct MoveDef {
    pub id: String,
    #[serde(default)]
    pub name: String,
    pub intent: IntentKindDef,
    #[serde(default)]
    pub damage: i32,
    #[serde(default)]
    pub damage_asc: i32,
    #[serde(default)]
    pub hit_count: u32,
    #[serde(default)]
    pub block: i32,
}

/// StS 意图（复用三色标：Attack 红 / Defend 蓝 / Buff 紫）
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum IntentKindDef { Attack, Defend, Buff }

/// StS 怪物定义
#[derive(Debug, Clone, Deserialize)]
pub struct StsMonsterDef {
    pub id: String,
    pub name: String,
    pub kind: MonsterKindDef,
    #[serde(default)]
    pub hp_min: i32,
    #[serde(default)]
    pub hp_max: i32,
    #[serde(default)]
    pub hp_min_asc: i32,
    #[serde(default)]
    pub hp_max_asc: i32,
    #[serde(default)]
    pub moves: Vec<MoveDef>,
    #[serde(default)]
    pub pattern: Vec<String>,
    #[serde(default)]
    pub innate: Vec<(String, i32)>,
}

pub fn load_sts_monsters() -> Vec<StsMonsterDef> {
    ron::from_str(include_str!("../data/sts_monsters.ron"))
        .unwrap_or_else(|e| { log::warn!("sts_monsters.ron 加载失败: {}", e); Vec::new() })
}

/// StS 遭遇定义
#[derive(Debug, Clone, Deserialize)]
pub struct StsEncounterDef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub act: String,
    #[serde(default)]
    pub act_num: u32,
    pub room: RoomTypeDef,
    #[serde(default)]
    pub monsters: Vec<String>,
}

pub fn load_sts_encounters() -> Vec<StsEncounterDef> {
    ron::from_str(include_str!("../data/sts_encounters.ron"))
        .unwrap_or_else(|e| { log::warn!("sts_encounters.ron 加载失败: {}", e); Vec::new() })
}

/// StS 事件选项
#[derive(Debug, Clone, Deserialize)]
pub struct StsEventOption {
    pub id: String,
    #[serde(default)]
    pub title: String,
    /// 中文标题（zhs 官方译名，空则回退英文）
    #[serde(default)]
    pub title_cn: String,
    #[serde(default)]
    pub description: String,
}

/// StS 事件定义
#[derive(Debug, Clone, Deserialize)]
pub struct StsEventDef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub act: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub preconditions: Option<String>,
    #[serde(default)]
    pub options: Vec<StsEventOption>,
}

pub fn load_sts_events() -> Vec<StsEventDef> {
    ron::from_str(include_str!("../data/sts_events.ron"))
        .unwrap_or_else(|e| { log::warn!("sts_events.ron 加载失败: {}", e); Vec::new() })
}

pub fn load_sts_cards() -> Vec<CardDef> {
    ron::from_str(include_str!("../data/sts_cards.ron"))
        .unwrap_or_else(|e| { log::warn!("sts_cards.ron 加载失败: {}", e); Vec::new() })
}

pub fn load_sts_relics() -> Vec<RelicDef> {
    ron::from_str(include_str!("../data/sts_relics.ron"))
        .unwrap_or_else(|e| { log::warn!("sts_relics.ron 加载失败: {}", e); Vec::new() })
}

pub fn load_sts_powers() -> Vec<PowerDef> {
    ron::from_str(include_str!("../data/sts_powers.ron"))
        .unwrap_or_else(|e| { log::warn!("sts_powers.ron 加载失败: {}", e); Vec::new() })
}

pub fn load_sts_potions() -> Vec<PotionDef> {
    ron::from_str(include_str!("../data/sts_potions.ron"))
        .unwrap_or_else(|e| { log::warn!("sts_potions.ron 加载失败: {}", e); Vec::new() })
}

pub fn load_sts_keywords() -> Vec<KeywordDef> {
    ron::from_str(include_str!("../data/sts_keywords.ron"))
        .unwrap_or_else(|e| { log::warn!("sts_keywords.ron 加载失败: {}", e); Vec::new() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sts_tables_parse() {
        assert_eq!(load_cards().len(), 10);
        assert_eq!(load_relics().len(), 8);
        assert_eq!(load_powers().len(), 8);
        assert_eq!(load_potions().len(), 6);
        assert_eq!(load_encounters().len(), 8);
        assert_eq!(load_keywords().len(), 10);
    }

    #[test]
    fn sts_card_effects_reference_known_powers() {
        let powers = load_powers();
        let pids: Vec<&str> = powers.iter().map(|p| p.id.as_str()).collect();
        for c in load_cards() {
            for e in c.effects.iter().chain(c.upgrade_effects.iter()) {
                if let CardEffectDef::ApplyPower(pid, _) = e {
                    assert!(pids.contains(&pid.as_str()), "卡 {} 引用未知能力 {}", c.id, pid);
                }
            }
        }
    }

    #[test]
    fn sts_codex_parses() {
        assert_eq!(load_sts_cards().len(), 577);
        assert_eq!(load_sts_relics().len(), 296);
        assert_eq!(load_sts_monsters().len(), 115);
        assert_eq!(load_sts_powers().len(), 257);
        assert_eq!(load_sts_potions().len(), 63);
        assert_eq!(load_sts_encounters().len(), 87);
        assert_eq!(load_sts_events().len(), 66);
        assert_eq!(load_sts_keywords().len(), 7);
        let cards = load_sts_cards();
        assert!(cards.iter().any(|c| c.id == "seven_stars" && c.name == "七星"));
        let mons = load_sts_monsters();
        assert!(mons.iter().any(|m| m.kind == MonsterKindDef::Boss && !m.moves.is_empty()));
        let encs = load_sts_encounters();
        assert!(encs.iter().any(|e| e.room == RoomTypeDef::Boss && !e.monsters.is_empty()));
        let evts = load_sts_events();
        assert!(evts.iter().any(|e| !e.options.is_empty()));
    }
}
