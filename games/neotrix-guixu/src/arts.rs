//! 兵器谱 — 以武器为纲（流星出招表 + 真实武功图谱）.
//!
//! 公理：同手不同器即不同艺（输入语法共享，招式/数值/性格归兵器）；
//! 招式 = 方向序列＋A（空中/持握态门禁）＋氣耗＋词条（倍率/击退）；
//! 招名取真实谱系（青萍剑/胡家刀），数值取太吾式模型（耗/伤/范围/附加）。
//! 流星语法要点：左右镜像（←折叠为→，对称公平）；A 落子即结算（中否皆清缓冲）。

/// 输入笔划（方向 + 攻击；跳跃键不入序列，空中由门判定）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stroke {
    Up,
    Down,
    Right,
    Atk,
}

/// 出招门禁（二元：空中专属 vs 通用；地面招即通用招）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtGate {
    Any,
    Air,
}

/// 单招
#[derive(Debug, Clone, Copy)]
pub struct MoveDef {
    /// 真名（青萍剑/胡家刀谱）
    pub name: &'static str,
    /// 输入序列（后缀匹配；左右已镜像，只写 Right）
    pub seq: &'static [Stroke],
    /// 氣耗
    pub qi: f32,
    /// 伤害倍率（相对基础刀光）
    pub dmg_mult: f32,
    /// 击退初速（0 = 无）
    pub knockback: f32,
    pub gate: ArtGate,
}

/// 兵器
#[derive(Debug, Clone, Copy)]
pub struct WeaponDef {
    pub name: &'static str,
    /// 攻击距离乘区
    pub reach: f32,
    /// 挥砍间隔（s，刀沉则慢）
    pub swing_cd: f32,
    pub moves: &'static [MoveDef],
}

use Stroke::*;
use ArtGate::*;

// ── 劍 · 青萍剑谱（轻灵迅捷，虚实相应） ──
const JIAN_MOVES: &[MoveDef] = &[
    MoveDef { name: "白蛇吐信", seq: &[Atk], qi: 0.0, dmg_mult: 1.0, knockback: 0.0, gate: Any },
    MoveDef { name: "白鹤亮翅", seq: &[Up, Atk], qi: 0.0, dmg_mult: 1.1, knockback: 60.0, gate: Any },
    MoveDef { name: "迎风挥扇", seq: &[Down, Atk], qi: 5.0, dmg_mult: 1.3, knockback: 120.0, gate: Any },
    MoveDef { name: "苍龙探爪", seq: &[Right, Atk], qi: 0.0, dmg_mult: 1.0, knockback: 40.0, gate: Any },
    MoveDef { name: "拨云瞻日", seq: &[Down, Up, Atk], qi: 10.0, dmg_mult: 1.6, knockback: 150.0, gate: Any },
    MoveDef { name: "凤凰点头", seq: &[Atk], qi: 0.0, dmg_mult: 1.2, knockback: 80.0, gate: Air },
];

// ── 刀 · 胡家刀法（大开大合，招数精奇，以力破巧反为下） ──
const DAO_MOVES: &[MoveDef] = &[
    MoveDef { name: "穿手藏刀", seq: &[Atk], qi: 0.0, dmg_mult: 1.4, knockback: 100.0, gate: Any },
    MoveDef { name: "怀中抱月", seq: &[Down, Atk], qi: 5.0, dmg_mult: 1.6, knockback: 160.0, gate: Any },
    MoveDef { name: "鹞子翻身刀", seq: &[Up, Atk], qi: 0.0, dmg_mult: 1.5, knockback: 140.0, gate: Any },
    MoveDef { name: "关平献印", seq: &[Right, Atk], qi: 10.0, dmg_mult: 1.8, knockback: 220.0, gate: Any },
    MoveDef { name: "夜叉探海", seq: &[Down, Up, Atk], qi: 15.0, dmg_mult: 2.2, knockback: 300.0, gate: Any },
    MoveDef { name: "浪子回头", seq: &[Atk], qi: 0.0, dmg_mult: 1.6, knockback: 180.0, gate: Air },
];

const JIAN: WeaponDef =
    WeaponDef { name: "劍", reach: 1.0, swing_cd: 0.35, moves: JIAN_MOVES };
const DAO: WeaponDef =
    WeaponDef { name: "刀", reach: 1.1, swing_cd: 0.5, moves: DAO_MOVES };

pub const WEAPONS: &[WeaponDef] = &[JIAN, DAO];

/// 击退衰减（指数近似，按帧率无关；纯函数可测）
pub fn kb_decay(kb: f32, dt: f32) -> f32 {
    kb * (1.0 - 6.0 * dt).max(0.0)
}

/// 招式匹配：门过滤 + 最长后缀；等长取门更精确者（空中 J 命中凤凰点头而非白蛇吐信）。
/// 无匹配回 None（调用方按普通挥砍处理，
/// 但本表六式全覆盖——素 J 必中 base，各武器必有出招）。
pub fn match_art(weapon: &WeaponDef, seq: &[Stroke], air: bool) -> Option<&'static MoveDef> {
    let spec = |m: &MoveDef| match m.gate {
        Any => 0,
        _ => 1,
    };
    let mut best: Option<&'static MoveDef> = None;
    for m in weapon.moves {
        let gate_ok = match m.gate {
            Any => true,
            Air => air,
        };
        if !gate_ok || seq.len() < m.seq.len() {
            continue;
        }
        let tail = &seq[seq.len() - m.seq.len()..];
        if tail == m.seq {
            let key = (m.seq.len(), spec(m));
            let keep = match best {
                Some(b) => (b.seq.len(), spec(b)) >= key,
                None => false,
            };
            if !keep {
                best = Some(m);
            }
        }
    }
    best
}

/// 招式解锁：表内序号 idx 需兵器等级 idx/2+1（0/1 招 L1，2/3 招 L2，4/5 招 L3）
pub fn req_level(idx: usize) -> u32 {
    (idx / 2 + 1) as u32
}

/// 已解锁招式（保持表序）
pub fn unlocked<'a>(weapon: &'a WeaponDef, wlevel: u32) -> Vec<&'a MoveDef> {
    weapon
        .moves
        .iter()
        .enumerate()
        .filter(|(i, _)| wlevel >= req_level(*i))
        .map(|(_, m)| m)
        .collect()
}

/// 自动优选（全自动战斗的大脑）：已解锁＋门合＋负担得起中伤害最高；并列取表序第一
pub fn choose_art<'a>(
    weapon: &'a WeaponDef,
    wlevel: u32,
    qi: f32,
    air: bool,
) -> Option<&'a MoveDef> {
    let mut best: Option<&'a MoveDef> = None;
    for (i, m) in weapon.moves.iter().enumerate() {
        if wlevel < req_level(i) {
            continue;
        }
        let gate_ok = match m.gate {
            ArtGate::Any => true,
            ArtGate::Air => air,
        };
        if !gate_ok || m.qi > qi {
            continue;
        }
        let better = match best {
            Some(b) => m.dmg_mult > b.dmg_mult,
            None => true,
        };
        if better {
            best = Some(m);
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_always_matches() {
        // 素 J 必中 base（各武器必有出招）
        assert_eq!(match_art(&JIAN, &[Atk], false).map(|m| m.name), Some("白蛇吐信"));
        assert_eq!(match_art(&DAO, &[Atk], false).map(|m| m.name), Some("穿手藏刀"));
    }

    #[test]
    fn longest_suffix_wins() {
        // [Down,Up,Atk] 同时以后缀 [Up,Atk] 候选 → 取最长的拨云瞻日
        assert_eq!(
            match_art(&JIAN, &[Down, Up, Atk], false).map(|m| m.name),
            Some("拨云瞻日")
        );
        assert_eq!(
            match_art(&DAO, &[Down, Up, Atk], false).map(|m| m.name),
            Some("夜叉探海")
        );
    }

    #[test]
    fn gates_filter() {
        // 空中 J → 凤凰点头/浪子回头；地面 J → 白蛇吐信/穿手藏刀
        assert_eq!(match_art(&JIAN, &[Atk], true).map(|m| m.name), Some("凤凰点头"));
        assert_eq!(match_art(&DAO, &[Atk], true).map(|m| m.name), Some("浪子回头"));
        assert_eq!(match_art(&JIAN, &[Atk], false).map(|m| m.name), Some("白蛇吐信"));
    }

    #[test]
    fn kb_decay_slows() {
        assert!((kb_decay(300.0, 0.016) - 300.0 * (1.0 - 6.0 * 0.016)).abs() < 1e-3);
        assert_eq!(kb_decay(100.0, 10.0), 0.0); // 大步钳制不反向
        assert_eq!(kb_decay(-50.0, 0.016), -50.0 * (1.0 - 6.0 * 0.016));
    }

    #[test]
    fn weapon_tables_sane() {
        assert_eq!(WEAPONS.len(), 2);
        for w in WEAPONS {
            assert_eq!(w.moves.len(), 6);
            assert!(w.moves.iter().any(|m| m.seq == [Atk] && m.gate == Any));
            assert!(w.moves.iter().any(|m| m.gate == Air));
            for m in w.moves {
                assert!(m.dmg_mult >= 1.0 && m.qi >= 0.0 && m.knockback >= 0.0);
            }
        }
        // 刀沉：更慢更痛
        assert!(DAO.swing_cd > JIAN.swing_cd);
        assert!(DAO.moves[0].dmg_mult > JIAN.moves[0].dmg_mult);
    }

    #[test]
    fn unlock_gates_by_level() {
        assert_eq!(req_level(0), 1);
        assert_eq!(req_level(1), 1);
        assert_eq!(req_level(4), 3);
        assert_eq!(unlocked(&JIAN, 1).len(), 2);
        assert_eq!(unlocked(&JIAN, 3).len(), 6);
    }

    #[test]
    fn auto_picks_strongest_affordable() {
        // 地面无气：白鹤亮翅（qi0 中最强 1.1）
        assert_eq!(choose_art(&JIAN, 3, 0.0, false).map(|m| m.name), Some("白鹤亮翅"));
        // 氣足：拨云瞻日 1.6
        assert_eq!(choose_art(&JIAN, 3, 50.0, false).map(|m| m.name), Some("拨云瞻日"));
        // L1：只有前两招
        assert_eq!(choose_art(&JIAN, 1, 50.0, false).map(|m| m.name), Some("白鹤亮翅"));
        // 空中：凤凰点头
        assert_eq!(choose_art(&JIAN, 3, 0.0, true).map(|m| m.name), Some("凤凰点头"));
    }
}
