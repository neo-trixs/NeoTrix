//! 全量数据服务层 — 对标 spire-codex backend/app/services/.
//!
//! 一次加载（启动时），多处共享：避免各调用点重复解析 RON。
//! 手写原创 6 表 + spire-codex 导入 8 表，共 14 表。

use super::schema;

/// 全库句柄（GameWorld.codex 持有）。
#[derive(Debug, Clone, Default)]
pub struct CodexStore {
    // 手写原创表
    pub cards: Vec<schema::CardDef>,
    pub relics: Vec<schema::RelicDef>,
    pub powers: Vec<schema::PowerDef>,
    pub potions: Vec<schema::PotionDef>,
    pub encounters: Vec<schema::EncounterDef>,
    pub keywords: Vec<schema::KeywordDef>,
    // spire-codex 导入表
    pub sts_cards: Vec<schema::CardDef>,
    pub sts_relics: Vec<schema::RelicDef>,
    pub sts_monsters: Vec<schema::StsMonsterDef>,
    pub sts_powers: Vec<schema::PowerDef>,
    pub sts_potions: Vec<schema::PotionDef>,
    pub sts_encounters: Vec<schema::StsEncounterDef>,
    pub sts_events: Vec<schema::StsEventDef>,
    pub sts_keywords: Vec<schema::KeywordDef>,
}

impl CodexStore {
    /// 全量加载（启动时一次）。
    pub fn load() -> Self {
        Self {
            cards: schema::load_cards(),
            relics: schema::load_relics(),
            powers: schema::load_powers(),
            potions: schema::load_potions(),
            encounters: schema::load_encounters(),
            keywords: schema::load_keywords(),
            sts_cards: schema::load_sts_cards(),
            sts_relics: schema::load_sts_relics(),
            sts_monsters: schema::load_sts_monsters(),
            sts_powers: schema::load_sts_powers(),
            sts_potions: schema::load_sts_potions(),
            sts_encounters: schema::load_sts_encounters(),
            sts_events: schema::load_sts_events(),
            sts_keywords: schema::load_sts_keywords(),
        }
    }

    /// 各表条目数（诊断/关于页用）。
    pub fn counts(&self) -> Vec<(&'static str, usize)> {
        vec![
            ("cards", self.cards.len()),
            ("relics", self.relics.len()),
            ("powers", self.powers.len()),
            ("potions", self.potions.len()),
            ("encounters", self.encounters.len()),
            ("keywords", self.keywords.len()),
            ("sts_cards", self.sts_cards.len()),
            ("sts_relics", self.sts_relics.len()),
            ("sts_monsters", self.sts_monsters.len()),
            ("sts_powers", self.sts_powers.len()),
            ("sts_potions", self.sts_potions.len()),
            ("sts_encounters", self.sts_encounters.len()),
            ("sts_events", self.sts_events.len()),
            ("sts_keywords", self.sts_keywords.len()),
        ]
    }

    /// 总条目数。
    pub fn total(&self) -> usize {
        self.counts().iter().map(|(_, n)| n).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_loads_all_tables() {
        let s = CodexStore::load();
        assert_eq!(s.cards.len(), 10);
        assert_eq!(s.sts_cards.len(), 577);
        assert_eq!(s.sts_monsters.len(), 115);
        assert_eq!(s.sts_events.len(), 66);
        assert!(s.total() > 1300);
    }

    #[test]
    fn counts_cover_14_tables() {
        let s = CodexStore::load();
        assert_eq!(s.counts().len(), 14);
        assert!(s.counts().iter().all(|(_, n)| *n > 0));
    }
}
