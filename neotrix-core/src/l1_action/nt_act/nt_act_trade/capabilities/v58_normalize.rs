//! V58 七维标准化引擎（Layer 3 输入归一化，仅用于匹配指纹构建）.
//!
//! 来源：VPOE_MASTER_V58.0-询价版 Layer 3（型号/材质/压力/口径/驱动/连接/认证）。
//! 规则：未知输入返回原文（不编造）；英制口径按附录 E 换算并保留原值备注。

#![forbid(unsafe_code)]

/// 3.2 型号标准化：去多余空格→大写
pub fn norm_model(model: &str) -> String {
    let m: String = model.split_whitespace().collect();
    if m == "-" || m.is_empty() {
        return "-".to_string();
    }
    m.to_uppercase()
}

/// 3.3 材质标准化（别名→标准）
pub fn norm_material(text: &str) -> String {
    let t = text.trim().to_lowercase();
    if t == "-" || t.is_empty() {
        return "-".to_string();
    }
    match t.as_str() {
        "wcb" | "a216 wcb" | "a216 gr.wcb" => "WCB".to_string(),
        "qt450" | "qt450-10" => "球墨铸铁 QT450-10".to_string(),
        "ss304" | "304" | "0cr18ni9" => "304".to_string(),
        "cf8" => "CF8".to_string(),
        "ss316" | "316" | "0cr17ni12mo2" => "316".to_string(),
        "cf8m" => "CF8M".to_string(),
        "ss316l" | "316l" | "cf3m" => "316L".to_string(),
        "球铁" | "qt" | "ggg" | "球墨铸铁" | "ductile iron" | "di" => "球墨铸铁".to_string(),
        "epdm" | "三元乙丙" | "三元乙丙橡胶" => "EPDM".to_string(),
        "ptfe" | "聚四氟乙烯" => "PTFE".to_string(),
        "stl" | "司太立" | "stellite" => "STL".to_string(),
        "碳钢" | "carbon steel" => "碳钢".to_string(),
        "wcc" => "WCC".to_string(),
        "不锈钢" | "ss" => "不锈钢".to_string(),
        "黄铜" | "brass" => "黄铜".to_string(),
        "bronze" | "铝青铜" => "铝青铜".to_string(),
        "灰铁" | "ht200" => "灰铁".to_string(),
        _ => text.trim().to_string(),
    }
}

/// 3.4 压力标准化（含 MPa→PN、LB→Class）
pub fn norm_pressure(text: &str) -> String {
    let t: String = text
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_lowercase();
    if t == "-" || t.is_empty() {
        return "-".to_string();
    }
    match t.as_str() {
        "pn10" | "1.0mpa" => "PN10".to_string(),
        "pn16" | "1.6mpa" | "16公斤级" => "PN16".to_string(),
        "pn25" | "2.5mpa" | "25公斤级" => "PN25".to_string(),
        "pn40" | "4.0mpa" => "PN40".to_string(),
        "class150" | "150lb" | "150#" | "150lbs" => "Class150".to_string(),
        "class300" | "300lb" => "Class300".to_string(),
        "300psi" => "300PSI".to_string(),
        "awwac515" => "AWWA C515".to_string(),
        _ => text.trim().to_string(),
    }
}

/// 附录 E 英制→DN（返回 None 表示非英制输入）
pub fn inch_to_dn(inch: &str) -> Option<&'static str> {
    match inch.trim() {
        "1/2" => Some("DN15"),
        "3/4" => Some("DN20"),
        "1" => Some("DN25"),
        "1-1/4" => Some("DN32"),
        "1-1/2" => Some("DN40"),
        "2" => Some("DN50"),
        "2-1/2" => Some("DN65"),
        "3" => Some("DN80"),
        "4" => Some("DN100"),
        "5" => Some("DN125"),
        "6" => Some("DN150"),
        "8" => Some("DN200"),
        "10" => Some("DN250"),
        "12" => Some("DN300"),
        "14" => Some("DN350"),
        "16" => Some("DN400"),
        "18" => Some("DN450"),
        "20" => Some("DN500"),
        "24" => Some("DN600"),
        "32" => Some("DN800"),
        _ => None,
    }
}

/// 3.5 口径标准化 → (DN, 原值备注)
pub fn norm_size(text: &str) -> (String, String) {
    let s = text.trim();
    if s.is_empty() || s == "-" {
        return ("-".to_string(), String::new());
    }
    let up = s.to_uppercase();
    if let Some(rest) = up.strip_prefix("DN") {
        if rest.trim().chars().all(|c| c.is_ascii_digit()) {
            return (format!("DN{}", rest.trim()), String::new());
        }
    }
    // 英制：数字 + " / inch
    let core = s
        .trim_end_matches(['"', '’', '”'])
        .trim_end_matches(|c: char| c.is_alphabetic())
        .trim();
    let core = core
        .strip_suffix("inch")
        .or_else(|| core.strip_suffix("INCH"))
        .unwrap_or(core);
    let core = core.trim().replace(' ', "");
    if !core.is_empty()
        && core
            .chars()
            .all(|c| c.is_ascii_digit() || c == '/' || c == '-' || c == '.')
    {
        if let Some(dn) = inch_to_dn(&core) {
            return (dn.to_string(), format!("（原值 {}\\\"）", core));
        }
        if let Ok(v) = core.replace('-', ".").parse::<f64>() {
            if v > 0.0 && v < 100.0 {
                return (
                    format!("DN{}", (v * 25.4).round() as u32),
                    format!("（原值 {}\\\"）", core),
                );
            }
        }
    }
    if let Some(mm) = s.strip_suffix("mm").or_else(|| s.strip_suffix("MM")) {
        if mm.trim().chars().all(|c| c.is_ascii_digit()) {
            return (format!("DN{}", mm.trim()), String::new());
        }
    }
    (s.to_string(), String::new())
}

/// 3.6 驱动标准化
pub fn norm_drive(text: &str) -> String {
    match text.trim().to_lowercase().as_str() {
        "手轮" | "手动" | "handwheel" => "手动".to_string(),
        "蜗轮" | "齿轮" | "齿轮箱" | "gear" | "gearbox" => "蜗轮".to_string(),
        "电动" | "electric" => "电动".to_string(),
        "气动" | "pneumatic" => "气动".to_string(),
        "液动" | "hydraulic" => "液动".to_string(),
        "自动" => "自动".to_string(),
        "-" | "" => "-".to_string(),
        _ => text.trim().to_string(),
    }
}

/// 3.7 连接标准化
pub fn norm_conn(text: &str) -> String {
    match text.trim().to_lowercase().as_str() {
        "法兰" | "flange" | "rf" => "法兰".to_string(),
        "对夹" | "wafer" => "对夹式".to_string(),
        "对焊" | "bw" | "butt weld" => "对焊".to_string(),
        "承插" | "sw" | "socket weld" => "承插焊".to_string(),
        "螺纹" | "丝扣" | "thread" | "npt" | "bsp" => "螺纹".to_string(),
        "沟槽" | "groove" => "沟槽".to_string(),
        "-" | "" => "-".to_string(),
        _ => text.trim().to_string(),
    }
}

/// 3.8 认证标准化
pub fn norm_cert(text: &str) -> String {
    match text.trim().to_lowercase().as_str() {
        "fm" | "ul" | "fm/ul" => "FM/UL".to_string(),
        "wras" => "WRAS".to_string(),
        "api600" | "api594" | "api623" => "API标准".to_string(),
        "gb/t" | "gb" => "国标".to_string(),
        "din" | "德标" => "德标".to_string(),
        "bs" | "英标" => "英标".to_string(),
        "awwa" | "美标" => "美标".to_string(),
        "-" | "" => "-".to_string(),
        _ => text.trim().to_string(),
    }
}

/// F04 七维指纹：标准化型号+大类+小类+口径+压力+阀体+标准+连接+驱动+认证
#[allow(clippy::too_many_arguments)]
pub fn fingerprint(
    model: &str,
    cat1: &str,
    cat2: &str,
    size: &str,
    pressure: &str,
    body: &str,
    std: &str,
    conn: &str,
    drive: &str,
    cert: &str,
) -> String {
    let (dn, _) = norm_size(size);
    [
        norm_model(model),
        cat1.to_string(),
        cat2.to_string(),
        dn,
        norm_pressure(pressure),
        norm_material(body),
        std.to_string(),
        norm_conn(conn),
        norm_drive(drive),
        norm_cert(cert),
    ]
    .into_iter()
    .map(|p| {
        if p == "-" || p.is_empty() {
            "?".to_string()
        } else {
            p
        }
    })
    .collect::<Vec<_>>()
    .join("+")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_upper_and_compact() {
        assert_eq!(norm_model("Z41 H - 16 C"), "Z41H-16C");
        assert_eq!(norm_model("z41h-16c"), "Z41H-16C");
    }

    #[test]
    fn material_aliases() {
        assert_eq!(norm_material("A216 WCB"), "WCB");
        assert_eq!(norm_material("qt450"), "球墨铸铁 QT450-10");
        assert_eq!(norm_material("SS316"), "316");
        assert_eq!(norm_material("三元乙丙橡胶"), "EPDM");
    }

    #[test]
    fn pressure_aliases() {
        assert_eq!(norm_pressure("150LB"), "Class150");
        assert_eq!(norm_pressure("1.6MPa"), "PN16");
    }

    #[test]
    fn inch_to_dn_table() {
        assert_eq!(norm_size("4\"").0, "DN100");
        assert_eq!(norm_size("2 inch").0, "DN50");
        assert_eq!(norm_size("DN50").0, "DN50");
        assert!(norm_size("4\"").1.contains("原值"));
    }

    #[test]
    fn drive_conn_cert() {
        assert_eq!(norm_drive("gear"), "蜗轮");
        assert_eq!(norm_conn("BW"), "对焊");
        assert_eq!(norm_cert("FM"), "FM/UL");
    }

    #[test]
    fn fingerprint_seven_dims() {
        let fp = fingerprint(
            "Z41H-16C", "?", "闸阀", "DN50", "PN16", "WCB", "?", "?", "?", "?",
        );
        assert!(fp.starts_with("Z41H-16C+?+闸阀+DN50+PN16+WCB"));
    }
}
