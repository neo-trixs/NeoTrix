//! 江湖志 — 真实资料数据层（考据来源见各条注，游戏语区统一简体）.
//!
//! 公理：小世界细节必须有现实锚点——兵器有尺寸出典，门派有十字诀，
//! 章节有名有姓，切口可对答。纯数据＋纯函数；调用方按章取加成、按波取事件。

/// 兵器实测（考古＋武志交叉；spec/doctrine 供 M3 图鉴屏，数值已服务 arts）
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct WeaponLore {
    /// 名
    pub name: &'static str,
    /// 形制（实测）
    pub spec: &'static str,
    /// 打法纲领
    pub doctrine: &'static str,
}

pub const WEAPON_LORE: &[WeaponLore] = &[
    WeaponLore {
        name: "劍",
        spec: "双刃短兵，百兵之君；秦剑80至90厘米，现代以反手直臂剑尖达耳垂为准",
        doctrine: "虚实相应，攻防交替；青萍剑：先发夺人，后发制人",
    },
    WeaponLore {
        name: "刀",
        spec: "单锋；苗刀刀身三尺三寸八分、靶一尺二寸，双手大刀圈",
        doctrine: "胡家刀：招数精奇，不在以力碰力",
    },
    WeaponLore {
        name: "槍",
        spec: "杨家梨花枪：虚实奇正，进锐退速，二十四势图；回马枪",
        doctrine: "不动如山，动如雷霆；中平枪，枪中王",
    },
    WeaponLore {
        name: "棍",
        spec: "少林棍，长丈二，梢把并用；齐眉棍长短在身高上下",
        doctrine: "棍打一大片，一扫一劈皆是群战法",
    },
];

/// 按名取兵器志（暂停页兵器卡用；名与兵器表一致：劍/刀/槍/棍）
pub fn weapon_lore(name: &str) -> Option<&'static WeaponLore> {
    WEAPON_LORE.iter().find(|w| w.name == name)
}

/// 门派（十字诀＋镇派，江湖墨社＋峨眉基地＋金庸志交叉）
#[derive(Debug, Clone, Copy)]
pub struct Sect {
    /// 名
    pub name: &'static str,
    /// 十字诀
    pub creed: &'static str,
    /// 镇派（M3 图鉴屏消费；mults 已服务波次）
    #[allow(dead_code)]
    pub signature: &'static str,
    /// 波次加成（hp, dmg, speed）
    pub mults: (f32, f32, f32),
}

pub const SECTS: &[Sect] = &[
    Sect { name: "无门无派", creed: "独行江湖", signature: "王八拳", mults: (1.0, 1.0, 1.0) },
    Sect { name: "青城", creed: "狠准捷变", signature: "天遁剑法", mults: (0.9, 1.0, 1.25) },
    Sect { name: "少林", creed: "禅武合一", signature: "金刚不坏", mults: (1.5, 1.0, 0.9) },
    Sect { name: "武当", creed: "以柔克刚", signature: "太极绵掌", mults: (1.1, 1.2, 1.0) },
    Sect { name: "丐帮", creed: "侠义干云", signature: "降龙十八掌", mults: (1.0, 1.3, 1.1) },
    Sect { name: "华山", creed: "奇险峻拔", signature: "独孤九剑", mults: (0.9, 1.4, 1.15) },
    Sect { name: "唐门", creed: "暴雨梨花", signature: "毒蒺藜", mults: (0.8, 1.2, 1.3) },
    Sect { name: "明教", creed: "圣火光明", signature: "乾坤挪移", mults: (1.3, 1.1, 0.95) },
];

/// 章节 → 门派（3 波一章，8 门派轮转；首章无门无派）
/// 章节长度规则 single-source 于 waves::chapter_of（3 波一章）
pub fn chapter_sect(wave: u32) -> &'static Sect {
    let ch = super::waves::chapter_of(wave);
    &SECTS[(ch as usize - 1) % SECTS.len()]
}

/// 章节名（横幅用）
pub fn chapter_name(wave: u32) -> String {
    let ch = super::waves::chapter_of(wave);
    format!("第{}章·{}", ch, chapter_sect(wave).name)
}

/// 切口对（盘道玩法：起句→接句；采信史：天王盖地虎/春点表交叉）
pub const SLANG: &[(&str, &str, &str)] = &[
    ("天王盖地虎", "宝塔镇河妖", "东北胡子盘道"),
    ("风紧", "扯呼", "情况不妙快撤"),
    ("合吾", "同道", "自报同道身份"),
    ("盘道", "套来历", "问对方师承来头"),
    ("万儿", "姓名来头", "报字号"),
    ("肘山", "酒", "讨酒喝"),
    ("并肩子", "朋友", "自己人"),
    ("瓢把子", "头领", "话事人"),
    ("踩盘子", "踩点", "事先侦察"),
    ("倒阳切密", "东南西北", "方位暗语"),
];

/// 切口应答校验（盘道事件用）
pub fn slang_reply(call: &str) -> Option<&'static str> {
    SLANG.iter().find(|(c, _, _)| *c == call).map(|(_, r, _)| *r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn tables_sane_and_unique() {
        assert_eq!(WEAPON_LORE.len(), 4);
        assert_eq!(SECTS.len(), 8);
        assert_eq!(SLANG.len(), 10);
        let names: HashSet<&str> = SECTS.iter().map(|s| s.name).collect();
        assert_eq!(names.len(), 8);
        for s in SECTS {
            assert!(s.mults.0 > 0.0 && s.mults.1 > 0.0 && s.mults.2 > 0.0);
        }
    }

    #[test]
    fn chapters_cycle_sects() {
        assert_eq!(chapter_sect(1).name, "无门无派");
        assert_eq!(chapter_sect(4).name, "青城");
        assert_eq!(chapter_sect(7).name, "少林");
        assert_eq!(chapter_sect(25).name, "无门无派"); // 8 门派轮转归位
        assert!(chapter_name(4).contains("青城"));
    }

    #[test]
    fn slang_classic_pair() {
        assert_eq!(slang_reply("天王盖地虎"), Some("宝塔镇河妖"));
        assert_eq!(slang_reply("风紧"), Some("扯呼"));
        assert_eq!(slang_reply("你好"), None);
    }

    #[test]
    fn weapon_lore_lookup() {
        let w = weapon_lore("劍").unwrap();
        assert!(w.spec.contains("双刃"));
        assert!(weapon_lore("槍").is_some());
        assert!(weapon_lore("斧").is_none());
    }
}
