//! 事件前置条件解析与校验（纯逻辑）.
// StS 事件运行时 — data/sts_events.ron（66 事件）的轻量结算层
//
// 职责：前置数值门校验（金币/生命）+ 选项关键词结算（生命上限/回血/扣血/金币/遗物）
// 数据契约：super::data::StsEventDef（id/name/act/description/preconditions/options）
// 注意：本文件只新增，不修改已有文件；接入需在 main.rs 加 `mod events_sts;`
//      并在 world.rs 的 GameWorld 追加 `event_rt` 字段（见文件末尾注释）。


// ═══════════════════════════════════════════════════════════════════

/// 前置条件种类
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreconditionId { Gold, Hp, Relic, None }

/// 单条前置条件。Hp 为负数时表示上限门（`amount = -n` 即要求 hp ≤ n，
/// 用于 "Requires ≤70% HP" 这类写法；正数表示下限门，要求 hp ≥ n）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Precondition {
    pub id: PreconditionId,
    pub amount: i32,
}

impl Precondition {
    pub fn none() -> Self {
        Self { id: PreconditionId::None, amount: 0 }
    }

    /// 纯检查：gold=持有金币，hp=当前生命，relics=持有遗物数。
    pub fn check(&self, gold: u32, hp: f32, relics: usize) -> bool {
        match self.id {
            PreconditionId::None => true,
            PreconditionId::Gold => {
                if self.amount <= 0 { true } else { gold >= self.amount as u32 }
            }
            PreconditionId::Hp => {
                if self.amount >= 0 { hp >= self.amount as f32 }
                else { hp <= self.amount.saturating_neg() as f32 }
            }
            PreconditionId::Relic => {
                let need = if self.amount > 0 { self.amount as usize } else { 1 };
                relics >= need
            }
        }
    }
}

/// 解析英文前置条件（大小写不敏感），如 "need 50 gold" / "hp above 20" / "relic X"。
/// 输入可能是 RON 列表编码（如 `["Requires 100+ gold", "Act 2 only"]`），按分隔符拆子句；
/// 无法识别的子句回 `None`（检查恒过：牌组/幕数等本层不消费的门直接放行）。
pub fn parse_precondition(pre: &str) -> Vec<Precondition> {
    let low = pre.to_lowercase();
    let stripped: String = low.chars()
        .filter(|c| !matches!(c, '[' | ']' | '"' | '\''))
        .collect();
    // "A or B" 拍平成 AND 安全：B 若不可识别则为 None（恒过），A 的数值门仍生效
    let normalized = stripped.replace(" or ", ",");
    let mut out = Vec::new();
    for seg in normalized.split(|c| matches!(c, ',' | ';' | '/' | '&' | '|' | '\n')) {
        let t = seg.trim();
        if t.is_empty() { continue; }
        out.push(parse_single_clause(t));
    }
    out
}

fn parse_single_clause(seg: &str) -> Precondition {
    let n = first_number(seg).unwrap_or(0);
    if seg.contains("gold") {
        Precondition { id: PreconditionId::Gold, amount: n }
    } else if seg.contains("hp") || seg.contains("health") || seg.contains("hit point") {
        let cap = seg.contains('≤') || seg.contains("<=") || seg.contains("below")
            || seg.contains("under") || seg.contains("less") || seg.contains("at most");
        if cap && n > 0 {
            Precondition { id: PreconditionId::Hp, amount: n.saturating_neg() }
        } else {
            Precondition { id: PreconditionId::Hp, amount: n }
        }
    } else if seg.contains("relic") {
        Precondition { id: PreconditionId::Relic, amount: if n > 0 { n } else { 1 } }
    } else {
        Precondition::none()
    }
}

/// 取文本中首个整数（"Gain 2 Max HP" → 2；"Requires 100-149 gold" → 100）。
pub fn first_number(text: &str) -> Option<i32> {
    let mut buf = String::new();
    let mut in_run = false;
    for ch in text.chars().chain(std::iter::once(' ')) {
        if ch.is_ascii_digit() {
            buf.push(ch);
            in_run = true;
        } else if in_run {
            break;
        }
    }
    if buf.is_empty() { Option::None } else { buf.parse::<i32>().ok() }
}

/// 取文本中全部整数（按出现顺序），供结算取第二个数等场景用。
pub fn all_numbers(text: &str) -> Vec<i32> {
    let mut out = Vec::new();
    let mut buf = String::new();
    for ch in text.chars().chain(std::iter::once(' ')) {
        if ch.is_ascii_digit() {
            buf.push(ch);
        } else if !buf.is_empty() {
            if let Ok(n) = buf.parse::<i32>() {
                out.push(n);
            }
            buf.clear();
        }
    }
    out
}

/// 取关键词之后首个整数（"Take 3 damage" + "take" → 3；找不到回 None）。
pub fn number_after(text: &str, keyword: &str) -> Option<i32> {
    let low = text.to_lowercase();
    let key = keyword.to_lowercase();
    low.find(key.as_str()).and_then(|pos| {
        let tail = low.get(pos + key.len()..).unwrap_or("");
        first_number(tail)
    })
}

// ═══════════════════════════════════════════════════════════════════
// GameWorld 运行时方法
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_gold_gate() {
        let v = parse_precondition("need 50 gold");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].id, PreconditionId::Gold);
        assert_eq!(v[0].amount, 50);
        assert!(v[0].check(60, 100.0, 0));
        assert!(!v[0].check(40, 100.0, 0));
    }

    #[test]
    fn parse_hp_gate() {
        let v = parse_precondition("HP above 20");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].id, PreconditionId::Hp);
        assert_eq!(v[0].amount, 20);
        assert!(v[0].check(0, 30.0, 0));
        assert!(!v[0].check(0, 10.0, 0));
    }

    #[test]
    fn parse_relic_gate() {
        let v = parse_precondition("Relic Blood Vial");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].id, PreconditionId::Relic);
        assert!(v[0].check(0, 100.0, 1));
        assert!(!v[0].check(0, 100.0, 0));
    }

    #[test]
    fn parse_invalid_falls_back_to_none() {
        let v = parse_precondition("Requires 2+ Strikes in deck");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].id, PreconditionId::None);
        // None 恒过：本层不消费的门直接放行
        assert!(v[0].check(0, 1.0, 0));
    }

    #[test]
    fn parse_ron_list_encoding() {
        let v = parse_precondition("[\"Requires 100+ gold\", \"Act 2 only\"]");
        assert_eq!(v.len(), 2);
        assert_eq!(v[0], Precondition { id: PreconditionId::Gold, amount: 100 });
        assert_eq!(v[1].id, PreconditionId::None);
    }

    #[test]
    fn number_helpers() {
        assert_eq!(first_number("Gain 2 Max HP. Take 3 damage."), Some(2));
        assert_eq!(number_after("Take 3 damage", "take"), Some(3));
        assert_eq!(number_after("Heal 10 HP.", "heal"), Some(10));
        assert_eq!(first_number("Leave"), Option::None);
        assert_eq!(all_numbers("Requires 100-149 gold"), vec![100, 149]);
    }

    #[test]
    fn hp_cap_gate() {
        // "Requires ≤70% HP"：上限门，要求 hp ≤ 70
        let v = parse_precondition("Requires ≤70% HP");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].id, PreconditionId::Hp);
        assert!(v[0].amount < 0);
        assert!(v[0].check(0, 50.0, 0));
        assert!(!v[0].check(0, 90.0, 0));
    }
}

// ── world.rs 追加字段（请手动加到 GameWorld 结构体，gold 附近即可）──
// pub(crate) event_rt: Option<super::events_sts::StsEventRuntime>,
// 构造处（GameWorld::new 的 Self { ... } 内）加：event_rt: None,

