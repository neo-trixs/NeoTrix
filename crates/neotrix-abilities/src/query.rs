//! 百科查询层（codex query）：StS 图鉴数据的纯函数筛选器。
//!
//! - 学 spire-codex 各 router 的筛选语义：精确匹配走枚举规范名（大小写不敏感），
//!   关键词走 `keywords` 包含，自由文本走 `name + description + id` 子串包含。
//! - 全部纯函数：只读切片参数、无副作用、不做 IO、不加载 RON。
//! - 返回原 `Vec` 的下标（`Vec<usize>`），调用方用下标回查原数组即可。
//! - 空过滤（`None` 或空白字符串）一律视为"不过滤"。

use super::schema::{
    CardDef, CardRarityDef, CardTypeDef, KeywordDef, MonsterKindDef, PotionDef, PowerDef,
    RelicDef, RelicRarityDef, StsEncounterDef, StsEventDef, StsMonsterDef,
};

// ── 内部小工具 ──

/// 枚举规范名：卡牌稀有度。
fn card_rarity_label(rarity: CardRarityDef) -> &'static str {
    match rarity {
        CardRarityDef::Basic => "basic",
        CardRarityDef::Common => "common",
        CardRarityDef::Uncommon => "uncommon",
        CardRarityDef::Rare => "rare",
        CardRarityDef::Special => "special",
    }
}

/// 枚举规范名：卡牌类型。
fn card_type_label(card_type: CardTypeDef) -> &'static str {
    match card_type {
        CardTypeDef::Attack => "attack",
        CardTypeDef::Skill => "skill",
        CardTypeDef::Power => "power",
        CardTypeDef::Curse => "curse",
        CardTypeDef::Status => "status",
    }
}

/// 枚举规范名：遗物稀有度。
fn relic_rarity_label(rarity: RelicRarityDef) -> &'static str {
    match rarity {
        RelicRarityDef::Starter => "starter",
        RelicRarityDef::Common => "common",
        RelicRarityDef::Uncommon => "uncommon",
        RelicRarityDef::Rare => "rare",
        RelicRarityDef::Boss => "boss",
        RelicRarityDef::Shop => "shop",
        RelicRarityDef::Event => "event",
        RelicRarityDef::Ancient => "ancient",
    }
}

/// 枚举规范名：怪物种类。
fn monster_kind_label(kind: MonsterKindDef) -> &'static str {
    match kind {
        MonsterKindDef::Normal => "normal",
        MonsterKindDef::Elite => "elite",
        MonsterKindDef::Boss => "boss",
    }
}

/// 精确匹配（大小写不敏感，两端去空白）；`None`/空白 = 不过滤。
fn matches_exact(value: &str, wanted: Option<&str>) -> bool {
    match wanted {
        None => true,
        Some(query) => {
            let query = query.trim();
            query.is_empty() || value.trim().eq_ignore_ascii_case(query)
        }
    }
}

/// 任一字段包含查询串（大小写不敏感）；`None`/空白 = 不过滤。
fn matches_search(fields: &[&str], search: Option<&str>) -> bool {
    match search {
        None => true,
        Some(query) => {
            let needle = query.trim().to_lowercase();
            if needle.is_empty() {
                return true;
            }
            fields
                .iter()
                .any(|field| field.to_lowercase().contains(needle.as_str()))
        }
    }
}

/// 关键词匹配：任一条 keyword 包含查询串（大小写不敏感）；`None`/空白 = 不过滤。
fn matches_keyword(keywords: &[String], keyword: Option<&str>) -> bool {
    match keyword {
        None => true,
        Some(query) => {
            let needle = query.trim().to_lowercase();
            if needle.is_empty() {
                return true;
            }
            keywords
                .iter()
                .any(|key| key.to_lowercase().contains(needle.as_str()))
        }
    }
}

// ── 卡牌 ──

/// 筛选卡牌，返回命中的下标。
///
/// - `rarity`：basic/common/uncommon/rare/special 精确匹配（大小写不敏感）。
/// - `type_`：attack/skill/power/curse/status 精确匹配（大小写不敏感）。
/// - `color`：颜色名精确匹配（大小写不敏感）。
/// - `keyword`：`keywords` 中任一条包含即可（大小写不敏感）。
/// - `search`：`name + description + id` 任一包含即可（大小写不敏感）。
pub fn filter_cards(
    cards: &[CardDef],
    rarity: Option<&str>,
    type_: Option<&str>,
    color: Option<&str>,
    keyword: Option<&str>,
    search: Option<&str>,
) -> Vec<usize> {
    cards
        .iter()
        .enumerate()
        .filter(|(_, card)| {
            matches_exact(card_rarity_label(card.rarity), rarity)
                && matches_exact(card_type_label(card.card_type), type_)
                && matches_exact(card.color.as_str(), color)
                && matches_keyword(&card.keywords, keyword)
                && matches_search(
                    &[
                        card.name.as_str(),
                        card.description.as_str(),
                        card.id.as_str(),
                    ],
                    search,
                )
        })
        .map(|(index, _)| index)
        .collect()
}

// ── 遗物 ──

/// 筛选遗物，返回命中的下标。
///
/// - `rarity`：starter/common/uncommon/rare/boss/shop/event/ancient 精确匹配。
/// - `pool`：掉落池精确匹配（大小写不敏感）。
/// - `search`：`name + description + id` 任一包含即可。
pub fn filter_relics(
    relics: &[RelicDef],
    rarity: Option<&str>,
    pool: Option<&str>,
    search: Option<&str>,
) -> Vec<usize> {
    relics
        .iter()
        .enumerate()
        .filter(|(_, relic)| {
            matches_exact(relic_rarity_label(relic.rarity), rarity)
                && matches_exact(relic.pool.as_str(), pool)
                && matches_search(
                    &[
                        relic.name.as_str(),
                        relic.description.as_str(),
                        relic.id.as_str(),
                    ],
                    search,
                )
        })
        .map(|(index, _)| index)
        .collect()
}

// ── 怪物（含遭遇交叉） ──

/// 筛选怪物，返回命中的下标。
///
/// - `kind`：normal/elite/boss 精确匹配（大小写不敏感）。
/// - `act`：幕次交叉——怪物 id 被任一 `act_num` 相等的遭遇的 `monsters` 引用即命中
///  （引用比对大小写不敏感；`None` = 不过滤）。
/// - `search`：`name + id` 任一包含即可（怪物无描述字段，故只查名字与 id）。
pub fn filter_monsters(
    monsters: &[StsMonsterDef],
    encounters: &[StsEncounterDef],
    kind: Option<&str>,
    act: Option<u32>,
    search: Option<&str>,
) -> Vec<usize> {
    monsters
        .iter()
        .enumerate()
        .filter(|(_, monster)| {
            if !matches_exact(monster_kind_label(monster.kind), kind) {
                return false;
            }
            if let Some(act_num) = act {
                let referenced = encounters.iter().any(|encounter| {
                    encounter.act_num == act_num
                        && encounter
                            .monsters
                            .iter()
                            .any(|id| id.eq_ignore_ascii_case(monster.id.as_str()))
                });
                if !referenced {
                    return false;
                }
            }
            matches_search(&[monster.name.as_str(), monster.id.as_str()], search)
        })
        .map(|(index, _)| index)
        .collect()
}

// ── 能力 ──

/// 筛选能力，返回命中的下标；`search` 匹配 `name + description + id`。
pub fn filter_powers(powers: &[PowerDef], search: Option<&str>) -> Vec<usize> {
    powers
        .iter()
        .enumerate()
        .filter(|(_, power)| {
            matches_search(
                &[
                    power.name.as_str(),
                    power.description.as_str(),
                    power.id.as_str(),
                ],
                search,
            )
        })
        .map(|(index, _)| index)
        .collect()
}

// ── 药水 ──

/// 筛选药水，返回命中的下标；`search` 匹配 `name + description + id`。
pub fn filter_potions(potions: &[PotionDef], search: Option<&str>) -> Vec<usize> {
    potions
        .iter()
        .enumerate()
        .filter(|(_, potion)| {
            matches_search(
                &[
                    potion.name.as_str(),
                    potion.description.as_str(),
                    potion.id.as_str(),
                ],
                search,
            )
        })
        .map(|(index, _)| index)
        .collect()
}

// ── 关键词 ──

/// 筛选关键词，返回命中的下标；`search` 匹配 `name + description + id`。
pub fn filter_keywords(keywords: &[KeywordDef], search: Option<&str>) -> Vec<usize> {
    keywords
        .iter()
        .enumerate()
        .filter(|(_, keyword)| {
            matches_search(
                &[
                    keyword.name.as_str(),
                    keyword.description.as_str(),
                    keyword.id.as_str(),
                ],
                search,
            )
        })
        .map(|(index, _)| index)
        .collect()
}

// ── 事件 ──

/// 筛选事件，返回命中的下标；`search` 匹配 `name + description + id`。
pub fn filter_events(events: &[StsEventDef], search: Option<&str>) -> Vec<usize> {
    events
        .iter()
        .enumerate()
        .filter(|(_, event)| {
            matches_search(
                &[
                    event.name.as_str(),
                    event.description.as_str(),
                    event.id.as_str(),
                ],
                search,
            )
        })
        .map(|(index, _)| index)
        .collect()
}

// ── 单元测试：手写迷你数据，不加载真实 RON ──

#[cfg(test)]
mod tests {
    use super::super::schema::{
        CardRarityDef, CardTargetDef, CardTypeDef, KeywordDef, MonsterKindDef, PotionDef,
        PotionRarityDef, PotionTargetDef, PowerDef, PowerStackDef, PowerTypeDef, RelicDef,
        RelicEffectDef, RelicRarityDef, RoomTypeDef, StsEncounterDef, StsEventDef,
    };
    use super::{
        filter_cards, filter_events, filter_keywords, filter_monsters, filter_potions,
        filter_powers, filter_relics,
    };
    use super::super::schema::{CardDef, StsMonsterDef};

    /// 迷你卡牌构造器。
    fn mini_card(
        id: &str,
        name: &str,
        rarity: CardRarityDef,
        card_type: CardTypeDef,
        color: &str,
        keywords: &[&str],
        description: &str,
    ) -> CardDef {
        CardDef {
            id: id.to_string(),
            name: name.to_string(),
            cost: 1,
            card_type,
            rarity,
            target: CardTargetDef::Enemy,
            element: None,
            effects: Vec::new(),
            upgrade_effects: Vec::new(),
            upgrade_note: String::new(),
            keywords: keywords.iter().map(|k| k.to_string()).collect(),
            color: color.to_string(),
            description: description.to_string(),
            upgraded_description: String::new(),
        }
    }

    /// 三张迷你卡牌：打击（红/普通/攻击）/ 碎裂（红/罕见/攻击， exhaustion 味）/ 冥想（蓝/普通/技能）。
    fn mini_cards() -> Vec<CardDef> {
        vec![
            mini_card(
                "strike",
                "Strike 打击",
                CardRarityDef::Common,
                CardTypeDef::Attack,
                "Red",
                &[],
                "造成 6 点伤害",
            ),
            mini_card(
                "sever",
                "Sever 碎裂",
                CardRarityDef::Rare,
                CardTypeDef::Attack,
                "Red",
                &["Exhaust"],
                "造成 16 点伤害，消耗",
            ),
            mini_card(
                "meditate",
                "Meditate 冥想",
                CardRarityDef::Common,
                CardTypeDef::Skill,
                "Blue",
                &[],
                "获得 2 点格挡",
            ),
        ]
    }

    #[test]
    fn cards_filter_by_rarity_exact() {
        let cards = mini_cards();
        // 罕见只有碎裂。
        assert_eq!(
            filter_cards(&cards, Some("rare"), None, None, None, None),
            vec![1]
        );
    }

    #[test]
    fn cards_filter_intersection_of_all_dimensions() {
        let cards = mini_cards();
        // 红 + 攻击 + 普通 = 只有打击。
        assert_eq!(
            filter_cards(
                &cards,
                Some("common"),
                Some("attack"),
                Some("red"),
                None,
                None
            ),
            vec![0]
        );
    }

    #[test]
    fn cards_filter_case_insensitive() {
        let cards = mini_cards();
        // 大小写混写仍应命中。
        assert_eq!(
            filter_cards(&cards, Some("COMMON"), Some("ATTACK"), Some("rEd"), None, None),
            vec![0]
        );
        // 关键词大小写不敏感。
        assert_eq!(
            filter_cards(&cards, None, None, None, Some("exhaust"), None),
            vec![1]
        );
    }

    #[test]
    fn cards_filter_search_and_none_passthrough() {
        let cards = mini_cards();
        // 搜中文描述命中冥想。
        assert_eq!(
            filter_cards(&cards, None, None, None, None, Some("格挡")),
            vec![2]
        );
        // 搜 id 命中。
        assert_eq!(
            filter_cards(&cards, None, None, None, None, Some("SEVER")),
            vec![1]
        );
        // 全 None = 全部返回。
        assert_eq!(
            filter_cards(&cards, None, None, None, None, None),
            vec![0, 1, 2]
        );
    }

    #[test]
    fn cards_filter_empty_result() {
        let cards = mini_cards();
        // 蓝色攻击牌不存在。
        assert!(
            filter_cards(&cards, None, Some("attack"), Some("blue"), None, None).is_empty()
        );
        // 查不到的关键词。
        assert!(filter_cards(&cards, None, None, None, Some("虚无"), None).is_empty());
    }

    /// 迷你遗物构造器。
    fn mini_relic(
        id: &str,
        name: &str,
        rarity: RelicRarityDef,
        pool: &str,
        description: &str,
    ) -> RelicDef {
        RelicDef {
            id: id.to_string(),
            name: name.to_string(),
            rarity,
            pool: pool.to_string(),
            effect: RelicEffectDef::MaxHp(10),
            merchant_price: None,
            description: description.to_string(),
            flavor: String::new(),
        }
    }

    /// 三件迷你遗物。
    fn mini_relics() -> Vec<RelicDef> {
        vec![
            mini_relic(
                "anchor",
                "Anchor 船锚",
                RelicRarityDef::Common,
                "shared",
                "战斗开始时获得 8 点格挡",
            ),
            mini_relic(
                "fossil",
                "Fossil 化石",
                RelicRarityDef::Uncommon,
                "red",
                "受到攻击伤害时获得 1 点金",
            ),
            mini_relic(
                "candle",
                "Candle 蜡烛",
                RelicRarityDef::Uncommon,
                "shared",
                "不可移除的诅咒被打出时获得 1 点能量",
            ),
        ]
    }

    #[test]
    fn relics_filter_by_rarity() {
        let relics = mini_relics();
        assert_eq!(filter_relics(&relics, Some("uncommon"), None, None), vec![1, 2]);
    }

    #[test]
    fn relics_filter_pool_case_insensitive() {
        let relics = mini_relics();
        // 池名大小写不敏感。
        assert_eq!(filter_relics(&relics, None, Some("SHARED"), None), vec![0, 2]);
    }

    #[test]
    fn relics_filter_search_hits_description() {
        let relics = mini_relics();
        // 描述含"格挡"的只有船锚。
        assert_eq!(filter_relics(&relics, None, None, Some("格挡")), vec![0]);
    }

    #[test]
    fn relics_filter_intersection_empty() {
        let relics = mini_relics();
        // 罕见 shared 池遗物不存在。
        assert!(filter_relics(&relics, Some("rare"), Some("shared"), None).is_empty());
    }

    /// 迷你怪物构造器。
    fn mini_monster(id: &str, name: &str, kind: MonsterKindDef) -> StsMonsterDef {
        StsMonsterDef {
            id: id.to_string(),
            name: name.to_string(),
            kind,
            hp_min: 10,
            hp_max: 20,
            hp_min_asc: 12,
            hp_max_asc: 22,
            moves: Vec::new(),
            pattern: Vec::new(),
            innate: Vec::new(),
        }
    }

    /// 迷你遭遇构造器。
    fn mini_encounter(id: &str, act_num: u32, monsters: &[&str]) -> StsEncounterDef {
        StsEncounterDef {
            id: id.to_string(),
            name: id.to_string(),
            act: String::new(),
            act_num,
            room: RoomTypeDef::Normal,
            monsters: monsters.iter().map(|m| m.to_string()).collect(),
        }
    }

    /// 三只怪物 + 两场遭遇（第一幕引用前两只，第二幕引用 Boss）。
    fn mini_monsters_and_encounters() -> (Vec<StsMonsterDef>, Vec<StsEncounterDef>) {
        let monsters = vec![
            mini_monster("jaw", "Jaw 大颚虫", MonsterKindDef::Normal),
            mini_monster("looter", "Looter 掠夺者", MonsterKindDef::Normal),
            mini_monster("hexaghost", "Hexaghost 六火灵", MonsterKindDef::Boss),
        ];
        let encounters = vec![
            mini_encounter("enc_act1", 1, &["jaw", "looter"]),
            mini_encounter("enc_act3", 3, &["hexaghost"]),
        ];
        (monsters, encounters)
    }

    #[test]
    fn monsters_filter_by_kind() {
        let (monsters, encounters) = mini_monsters_and_encounters();
        assert_eq!(
            filter_monsters(&monsters, &encounters, Some("BOSS"), None, None),
            vec![2]
        );
    }

    #[test]
    fn monsters_filter_by_act_cross_reference() {
        let (monsters, encounters) = mini_monsters_and_encounters();
        // 第一幕引用大颚虫与掠夺者。
        assert_eq!(
            filter_monsters(&monsters, &encounters, None, Some(1), None),
            vec![0, 1]
        );
        // 第二幕没有任何遭遇引用。
        assert!(
            filter_monsters(&monsters, &encounters, None, Some(2), None).is_empty()
        );
    }

    #[test]
    fn monsters_filter_kind_and_act_intersection() {
        let (monsters, encounters) = mini_monsters_and_encounters();
        // 普通怪 ∩ 第一幕 = 前两只；Boss ∩ 第一幕 = 空。
        assert_eq!(
            filter_monsters(&monsters, &encounters, Some("normal"), Some(1), None),
            vec![0, 1]
        );
        assert!(
            filter_monsters(&monsters, &encounters, Some("boss"), Some(1), None).is_empty()
        );
    }

    #[test]
    fn monsters_filter_search_case_insensitive() {
        let (monsters, encounters) = mini_monsters_and_encounters();
        // id 大小写不敏感命中。
        assert_eq!(
            filter_monsters(&monsters, &encounters, None, None, Some("HEXAGHOST")),
            vec![2]
        );
        // 中文名命中。
        assert_eq!(
            filter_monsters(&monsters, &encounters, None, None, Some("掠夺")),
            vec![1]
        );
    }

    /// 迷你能力构造器。
    fn mini_power(id: &str, name: &str, description: &str) -> PowerDef {
        PowerDef {
            id: id.to_string(),
            name: name.to_string(),
            power_type: PowerTypeDef::Buff,
            stack: PowerStackDef::Counter,
            description: description.to_string(),
        }
    }

    #[test]
    fn powers_filter_search_hits_each_field() {
        let powers = vec![
            mini_power("strength", "Strength 力量", "攻击伤害提高"),
            mini_power("vuln", "Vulnerable 易伤", "受到的攻击伤害提高一半"),
            mini_power("regen", "Regen 再生", "回合结束时回复生命"),
        ];
        // 名命中。
        assert_eq!(filter_powers(&powers, Some("易伤")), vec![1]);
        // 描述命中（两条都含"攻击伤害"）。
        assert_eq!(filter_powers(&powers, Some("攻击伤害")), vec![0, 1]);
        // id 大小写不敏感命中。
        assert_eq!(filter_powers(&powers, Some("REGEN")), vec![2]);
    }

    #[test]
    fn powers_filter_none_returns_all() {
        let powers = vec![mini_power("a", "甲", "一"), mini_power("b", "乙", "二")];
        assert_eq!(filter_powers(&powers, None), vec![0, 1]);
    }

    #[test]
    fn powers_filter_empty_result() {
        let powers = vec![mini_power("strength", "Strength 力量", "攻击伤害提高")];
        assert!(filter_powers(&powers, Some("查无此物")).is_empty());
    }

    /// 迷你药水构造器。
    fn mini_potion(id: &str, name: &str, description: &str) -> PotionDef {
        PotionDef {
            id: id.to_string(),
            name: name.to_string(),
            rarity: PotionRarityDef::Common,
            target: PotionTargetDef::Own,
            effect: None,
            pool: String::new(),
            description: description.to_string(),
        }
    }

    #[test]
    fn potions_filter_search_hits_name() {
        let potions = vec![
            mini_potion("blood", "Blood 血瓶", "回复 20% 生命"),
            mini_potion("smoke", "Smoke 烟雾", "获得 2 回合隐匿"),
        ];
        assert_eq!(filter_potions(&potions, Some("烟雾")), vec![1]);
    }

    #[test]
    fn potions_filter_case_insensitive_id() {
        let potions = vec![mini_potion("blood", "Blood 血瓶", "回复生命")];
        assert_eq!(filter_potions(&potions, Some("BLOOD")), vec![0]);
    }

    #[test]
    fn potions_filter_empty_and_passthrough() {
        let potions = vec![mini_potion("blood", "Blood 血瓶", "回复生命")];
        assert!(filter_potions(&potions, Some("不存在")).is_empty());
        assert_eq!(filter_potions(&potions, None), vec![0]);
    }

    /// 迷你关键词构造器。
    fn mini_keyword(id: &str, name: &str, description: &str) -> KeywordDef {
        KeywordDef {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
        }
    }

    #[test]
    fn keywords_filter_search_hits_description() {
        let keywords = vec![
            mini_keyword("exhaust", "Exhaust 消耗", "打出后从牌组中移除"),
            mini_keyword("ethereal", "Ethereal 虚无", "回合结束时未打出则消耗"),
        ];
        // 两条描述都含"消耗"。
        assert_eq!(filter_keywords(&keywords, Some("消耗")), vec![0, 1]);
    }

    #[test]
    fn keywords_filter_case_insensitive() {
        let keywords = vec![mini_keyword("exhaust", "Exhaust 消耗", "移除")];
        assert_eq!(filter_keywords(&keywords, Some("EXHAUST")), vec![0]);
    }

    #[test]
    fn keywords_filter_empty_and_passthrough() {
        let keywords = vec![mini_keyword("exhaust", "Exhaust 消耗", "移除")];
        assert!(filter_keywords(&keywords, Some("查无")).is_empty());
        assert_eq!(filter_keywords(&keywords, None), vec![0]);
    }

    /// 迷你事件构造器。
    fn mini_event(id: &str, name: &str, description: &str) -> StsEventDef {
        StsEventDef {
            id: id.to_string(),
            name: name.to_string(),
            act: String::new(),
            description: description.to_string(),
            preconditions: None,
            options: Vec::new(),
        }
    }

    #[test]
    fn events_filter_search_hits_name() {
        let events = vec![
            mini_event("shrine", "Shrine 神龛", "献上生命换取力量"),
            mini_event("lotus", "Lotus 莲花", "采摘获得遗物"),
        ];
        assert_eq!(filter_events(&events, Some("神龛")), vec![0]);
    }

    #[test]
    fn events_filter_search_hits_description_and_id() {
        let events = vec![
            mini_event("shrine", "Shrine 神龛", "献上生命换取力量"),
            mini_event("lotus", "Lotus 莲花", "采摘获得遗物"),
        ];
        assert_eq!(filter_events(&events, Some("遗物")), vec![1]);
        assert_eq!(filter_events(&events, Some("LOTUS")), vec![1]);
    }

    #[test]
    fn events_filter_empty_and_passthrough() {
        let events = vec![mini_event("shrine", "Shrine 神龛", "献上生命")];
        assert!(filter_events(&events, Some("查无")).is_empty());
        assert_eq!(filter_events(&events, None), vec![0]);
    }
}
