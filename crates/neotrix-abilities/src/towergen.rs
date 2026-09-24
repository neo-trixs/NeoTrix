//! 爬塔节点图生成与行进状态机（确定性 LCG，纯逻辑）.

// 登塔（Tower）运行时 — StS 式爬塔节点图（纯逻辑层）
//
// - 不碰渲染/输入：只做节点图生成与行进状态机，渲染层经
//   `option_ids()` 拉取候选节点自行绘制。
// - 状态独立存于 `TowerMap`，调用方持有；与线性关卡无关。
// - 确定性：`generate_act` 为纯函数，同样 `(act, encounters, seed)`
//   必得同样节点图；随机只用自带 LCG，不引外部依赖。
// - 硬规则：生产代码无 unwrap/expect/panic（`?`/`if let`/`map_or`/`unwrap_or`），
//   中文 UI 字符串，跨模块引用走 `super::x`。

use super::schema::{RoomTypeDef, StsEncounterDef};

/// 塔顶层数（1..=15）。
pub const TOWER_TOP_FLOOR: u32 = 15;
/// 精英层（中段 / 末段守门）。
pub const TOWER_ELITE_FLOORS: [u32; 2] = [7, 14];
/// 宝箱层 / 休整层（Boss 前喘息）。
pub const TOWER_TREASURE_FLOOR: u32 = 8;
pub const TOWER_REST_FLOOR: u32 = 9;

/// 爬塔节点类型。
///
/// 与 `RoomTypeDef` 对应：`Normal→Combat`，`Elite→Elite`，`Boss→Boss`；
/// `Event/Shop/Treasure/Rest` 为行进节点（无战斗遭遇）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TowerNodeKind {
    Combat,
    Elite,
    Boss,
    Event,
    Shop,
    Treasure,
    Rest,
}

impl TowerNodeKind {
    /// 中文名（直接供渲染层显示）。
    pub fn name(&self) -> &'static str {
        match self {
            TowerNodeKind::Combat => "战斗",
            TowerNodeKind::Elite => "精英",
            TowerNodeKind::Boss => "首领",
            TowerNodeKind::Event => "事件",
            TowerNodeKind::Shop => "商店",
            TowerNodeKind::Treasure => "宝箱",
            TowerNodeKind::Rest => "休整",
        }
    }

    /// 遭遇房间类型 → 爬塔节点类型。
    pub fn from_room(room: RoomTypeDef) -> Self {
        match room {
            RoomTypeDef::Normal => TowerNodeKind::Combat,
            RoomTypeDef::Elite => TowerNodeKind::Elite,
            RoomTypeDef::Boss => TowerNodeKind::Boss,
        }
    }

    /// 是否为战斗节点（需要分配遭遇 id）。
    pub fn is_fight(&self) -> bool {
        matches!(
            self,
            TowerNodeKind::Combat | TowerNodeKind::Elite | TowerNodeKind::Boss
        )
    }
}

/// 爬塔节点：`id` 恒等于其在 `TowerMap::nodes` 中的下标。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TowerNode {
    pub id: usize,
    pub floor: u32,
    pub kind: TowerNodeKind,
    pub encounter_id: Option<String>,
    pub next: Vec<usize>,
}

/// 爬塔整幕状态机。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TowerMap {
    pub act: u32,
    pub nodes: Vec<TowerNode>,
    /// 当前立足节点 id。
    pub current: usize,
    /// 已完成（含立足起点）节点 id 表。
    pub completed: Vec<usize>,
}

// ── 自带确定性随机（LCG，不依赖外部 rand）──

/// LCG 步进（Knuth MMIX 常数），返回新状态。
fn lcg_next(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

/// Fisher-Yates 洗牌（LCG 驱动，同样 seed 必同样顺序）。
fn shuffle_lcg<T>(items: &mut [T], seed: u64) {
    if items.len() < 2 {
        return;
    }
    let mut rng = seed;
    let mut i = items.len();
    while i > 1 {
        i -= 1;
        let j = (lcg_next(&mut rng) % (i as u64 + 1)) as usize;
        if i < items.len() && j < items.len() {
            items.swap(i, j);
        }
    }
}

/// 固定层模板：1 战斗 / 7·14 精英 / 8 宝箱 / 9 休整 / 15 首领；其余 None 表交错。
fn fixed_floor_kind(floor: u32) -> Option<TowerNodeKind> {
    if floor == 1 {
        Some(TowerNodeKind::Combat)
    } else if TOWER_ELITE_FLOORS.contains(&floor) {
        Some(TowerNodeKind::Elite)
    } else if floor == TOWER_TREASURE_FLOOR {
        Some(TowerNodeKind::Treasure)
    } else if floor == TOWER_REST_FLOOR {
        Some(TowerNodeKind::Rest)
    } else if floor == TOWER_TOP_FLOOR {
        Some(TowerNodeKind::Boss)
    } else {
        None
    }
}

/// 行进层抽取：战斗 3/5，事件 1/5，商店 1/5。
fn draw_travel_kind(rng: &mut u64) -> TowerNodeKind {
    match lcg_next(rng) % 5 {
        2 => TowerNodeKind::Event,
        3 => TowerNodeKind::Shop,
        _ => TowerNodeKind::Combat,
    }
}

/// 生成整幕爬塔图（纯函数，无 IO、无全局状态）。
///
/// - 15 层：1 战斗 / 7·14 精英 / 8 宝箱 / 9 休整 / 15 首领，其余战斗·事件·商店交错；
/// - 每层 2-3 节点（LCG 决定），同层节点互不连边；
/// - 相邻层按索引邻近连边（`i→i-1,i,i+1` 下标越界 clamp），下层孤儿节点兜底接最近上层；
/// - 遭遇按房间类型轮询分配（同类循环取模；对应池为空则该节点 `encounter_id=None`，
///   行进节点恒为 `None`）；遭遇池优先取 `act_num == act` 者，无则回退全量；
/// - `seed` 经 LCG 驱动节点数 / 行进类型 / 池洗牌，同样输入必同样输出。
pub fn generate_act(act: u32, encounters: &[StsEncounterDef], seed: u64) -> TowerMap {
    let mut rng = seed;

    // ── 遭遇按幕过滤（本幕无遭遇则回退全幕），再按房间类型分池 ──
    let has_act = encounters.iter().any(|e| e.act_num == act);
    let mut normals: Vec<String> = Vec::new();
    let mut elites: Vec<String> = Vec::new();
    let mut bosses: Vec<String> = Vec::new();
    for e in encounters {
        if has_act && e.act_num != act {
            continue;
        }
        match e.room {
            RoomTypeDef::Normal => normals.push(e.id.clone()),
            RoomTypeDef::Elite => elites.push(e.id.clone()),
            RoomTypeDef::Boss => bosses.push(e.id.clone()),
        }
    }
    // 确定性洗牌：同类轮询时同样 seed 必同样顺序。
    shuffle_lcg(&mut normals, seed.wrapping_add(0x9E3779B97F4A7C15));
    shuffle_lcg(&mut elites, seed.wrapping_add(0xBF58476D1CE4E5B9));
    shuffle_lcg(&mut bosses, seed.wrapping_add(0x94D049BB133111EB));

    // ── 15 层 × 每层 2-3 节点 ──
    let mut nodes: Vec<TowerNode> = Vec::new();
    let mut floors: Vec<Vec<usize>> = Vec::new();
    let mut floor = 1u32;
    while floor <= TOWER_TOP_FLOOR {
        let count = 2usize + (lcg_next(&mut rng) % 2) as usize;
        let mut kinds: Vec<TowerNodeKind> = Vec::new();
        if let Some(fixed) = fixed_floor_kind(floor) {
            let mut k = 0usize;
            while k < count {
                kinds.push(fixed);
                k += 1;
            }
        } else {
            let mut k = 0usize;
            while k < count {
                kinds.push(draw_travel_kind(&mut rng));
                k += 1;
            }
            let salt = lcg_next(&mut rng);
            shuffle_lcg(&mut kinds, salt);
        }
        let mut ids: Vec<usize> = Vec::new();
        for kind in kinds {
            let id = nodes.len();
            nodes.push(TowerNode {
                id,
                floor,
                kind,
                encounter_id: None,
                next: Vec::new(),
            });
            ids.push(id);
        }
        floors.push(ids);
        floor += 1;
    }

    // ── 相邻层按索引邻近连边（同层永不连边）──
    let mut f = 0usize;
    while f + 1 < floors.len() {
        let upper_len = floors.get(f).map_or(0, |v| v.len());
        let lower_len = floors.get(f + 1).map_or(0, |v| v.len());
        if upper_len > 0 && lower_len > 0 {
            // 上层每个节点向邻近索引连边。
            let mut i = 0usize;
            while i < upper_len {
                let mut js: Vec<usize> = Vec::new();
                if i > 0 {
                    js.push(i - 1);
                }
                js.push(i);
                js.push(i + 1);
                for j in js {
                    let jj = if j >= lower_len { lower_len - 1 } else { j };
                    let uid = floors.get(f).and_then(|v| v.get(i)).copied();
                    let vid = floors.get(f + 1).and_then(|v| v.get(jj)).copied();
                    if let (Some(u), Some(v)) = (uid, vid) {
                        if let Some(node) = nodes.get_mut(u) {
                            if !node.next.contains(&v) {
                                node.next.push(v);
                            }
                        }
                    }
                }
                i += 1;
            }
            // 兜底：下层孤儿节点（无入边）接最近上层节点。
            let mut j = 0usize;
            while j < lower_len {
                if let Some(v) = floors.get(f + 1).and_then(|v| v.get(j)).copied() {
                    let mut has_in = false;
                    let mut k = 0usize;
                    while k < upper_len {
                        let has = floors
                            .get(f)
                            .and_then(|uv| uv.get(k))
                            .copied()
                            .and_then(|u| nodes.get(u))
                            .map_or(false, |n| n.next.contains(&v));
                        if has {
                            has_in = true;
                            break;
                        }
                        k += 1;
                    }
                    if !has_in {
                        let ni = if j >= upper_len { upper_len - 1 } else { j };
                        if let Some(u) = floors.get(f).and_then(|uv| uv.get(ni)).copied() {
                            if let Some(node) = nodes.get_mut(u) {
                                if !node.next.contains(&v) {
                                    node.next.push(v);
                                }
                            }
                        }
                    }
                }
                j += 1;
            }
        }
        f += 1;
    }

    // ── 遭遇按房间类型轮询分配（同类循环取模；池空则 None）──
    let (mut ci, mut ei, mut bi) = (0usize, 0usize, 0usize);
    for node in nodes.iter_mut() {
        if node.kind == TowerNodeKind::Combat && !normals.is_empty() {
            if let Some(eid) = normals.get(ci % normals.len()) {
                node.encounter_id = Some(eid.clone());
                ci += 1;
            }
        } else if node.kind == TowerNodeKind::Elite && !elites.is_empty() {
            if let Some(eid) = elites.get(ei % elites.len()) {
                node.encounter_id = Some(eid.clone());
                ei += 1;
            }
        } else if node.kind == TowerNodeKind::Boss && !bosses.is_empty() {
            if let Some(eid) = bosses.get(bi % bosses.len()) {
                node.encounter_id = Some(eid.clone());
                bi += 1;
            }
        }
    }

    let start = floors.first().and_then(|v| v.first()).copied().unwrap_or(0);
    TowerMap {
        act,
        nodes,
        current: start,
        completed: Vec::new(),
    }
}

impl TowerMap {
    /// 按 id 取节点（id 恒为下标；错位时回退线性查找）。
    pub fn node(&self, id: usize) -> Option<&TowerNode> {
        if let Some(n) = self.nodes.get(id) {
            if n.id == id {
                return Some(n);
            }
        }
        self.nodes.iter().find(|n| n.id == id)
    }

    /// 某层全部节点 id（渲染塔图用）。
    pub fn floor_nodes(&self, floor: u32) -> Vec<usize> {
        self.nodes
            .iter()
            .filter(|n| n.floor == floor)
            .map(|n| n.id)
            .collect()
    }

    /// 当前节点的可行候选 id 表。
    pub fn option_ids(&self) -> Vec<usize> {
        self.node(self.current)
            .map_or(Vec::new(), |c| c.next.clone())
    }

    /// 纯行进校验：目标须在 `current.next` 内，且 `completed` 含当前节点；
    /// 成功则移动 `current` 并记录目标完成。非法移动返回 false。
    pub fn try_advance(&mut self, target: usize) -> bool {
        let cur = self.current;
        let linked = self.node(cur).map_or(false, |c| c.next.contains(&target));
        if !linked {
            return false;
        }
        if !self.completed.contains(&cur) {
            return false;
        }
        if self.node(target).is_none() {
            return false;
        }
        self.current = target;
        if !self.completed.contains(&target) {
            self.completed.push(target);
        }
        true
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn sample_encounters() -> Vec<StsEncounterDef> {
        vec![
            StsEncounterDef {
                id: "n_cultist".to_string(),
                name: "邪教徒".to_string(),
                act: "第一幕".to_string(),
                act_num: 1,
                room: RoomTypeDef::Normal,
                monsters: vec!["cultist".to_string()],
            },
            StsEncounterDef {
                id: "n_jaw".to_string(),
                name: "大颚虫".to_string(),
                act: "第一幕".to_string(),
                act_num: 1,
                room: RoomTypeDef::Normal,
                monsters: vec!["jaw_worm".to_string()],
            },
            StsEncounterDef {
                id: "e_nob".to_string(),
                name: "小鬼头目".to_string(),
                act: "第一幕".to_string(),
                act_num: 1,
                room: RoomTypeDef::Elite,
                monsters: vec!["gremlin_nob".to_string()],
            },
            StsEncounterDef {
                id: "b_slime".to_string(),
                name: "史莱姆之王".to_string(),
                act: "第一幕".to_string(),
                act_num: 1,
                room: RoomTypeDef::Boss,
                monsters: vec!["slime_boss".to_string()],
            },
        ]
    }

    #[test]
    fn floors_complete_and_boss_only_on_top() {
        let map = generate_act(1, &sample_encounters(), 42);
        // 15 层都有节点
        let mut floor = 1u32;
        while floor <= TOWER_TOP_FLOOR {
            assert!(
                map.nodes.iter().any(|n| n.floor == floor),
                "第{}层无节点",
                floor
            );
            floor += 1;
        }
        // Boss 只在 15 层，且 15 层确有 Boss
        assert!(map
            .nodes
            .iter()
            .any(|n| n.kind == TowerNodeKind::Boss && n.floor == TOWER_TOP_FLOOR));
        for n in &map.nodes {
            if n.kind == TowerNodeKind::Boss {
                assert_eq!(n.floor, TOWER_TOP_FLOOR, "Boss 不在顶层");
            }
        }
        // 固定层类型
        for n in map.nodes.iter().filter(|n| n.floor == 1) {
            assert_eq!(n.kind, TowerNodeKind::Combat);
        }
        for n in map.nodes.iter().filter(|n| n.floor == TOWER_TREASURE_FLOOR) {
            assert_eq!(n.kind, TowerNodeKind::Treasure);
        }
        for n in map.nodes.iter().filter(|n| n.floor == TOWER_REST_FLOOR) {
            assert_eq!(n.kind, TowerNodeKind::Rest);
        }
        for n in map.nodes.iter().filter(|n| n.floor == 7 || n.floor == 14) {
            assert_eq!(n.kind, TowerNodeKind::Elite);
        }
        // 遭遇轮询：战斗节点有遭遇（取自同类池），行进节点无遭遇
        for n in map.nodes.iter().filter(|n| n.kind.is_fight()) {
            assert!(n.encounter_id.is_some(), "战斗节点缺遭遇：{}", n.id);
        }
        for n in map.nodes.iter().filter(|n| !n.kind.is_fight()) {
            assert!(n.encounter_id.is_none(), "行进节点不应有遭遇：{}", n.id);
        }
    }

    #[test]
    fn edges_adjacent_and_same_seed_stable() {
        let a = generate_act(1, &sample_encounters(), 7);
        let b = generate_act(1, &sample_encounters(), 7);
        // 同样 seed 两次结果一致
        assert_eq!(a, b);
        // 边只连相邻层，同层永不连边
        for n in &a.nodes {
            for t in n.next.iter() {
                if let Some(m) = a.node(*t) {
                    assert_eq!(m.floor, n.floor + 1, "边跨层非法");
                } else {
                    assert!(false, "边指向未知节点");
                }
            }
        }
        // 空遭遇池不断言崩溃：全 None 但图结构完整
        let empty = generate_act(1, &[], 5);
        assert!(empty.nodes.iter().all(|n| n.encounter_id.is_none()));
        let mut floor = 1u32;
        while floor <= TOWER_TOP_FLOOR {
            assert!(empty.nodes.iter().any(|n| n.floor == floor));
            floor += 1;
        }
    }

    #[test]
    fn illegal_advance_rejected() {
        let mut map = generate_act(1, &sample_encounters(), 99);
        let first_next = map.node(map.current).and_then(|c| c.next.first()).copied();
        // 未立足（completed 为空）：任何移动都拒绝
        if let Some(t) = first_next {
            assert!(!map.try_advance(t));
        }
        // 立足后：跳层目标拒绝
        map.completed.push(map.current);
        let far = map
            .nodes
            .iter()
            .find(|n| n.floor > map.node(map.current).map_or(0, |c| c.floor + 1))
            .map(|n| n.id);
        if let Some(f) = far {
            let in_next = map.node(map.current).map_or(false, |c| c.next.contains(&f));
            if !in_next {
                assert!(!map.try_advance(f));
            }
        }
        // 不存在的目标拒绝
        assert!(!map.try_advance(map.nodes.len() + 100));
        // 合法移动成功并记录完成
        if let Some(t) = first_next {
            assert!(map.try_advance(t));
            assert_eq!(map.current, t);
            assert!(map.completed.contains(&t));
        }
    }
}
