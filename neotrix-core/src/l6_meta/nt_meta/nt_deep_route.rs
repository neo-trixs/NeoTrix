//! NT-DEEP-ROUTE — 深特征路由（D1，deep_route.py 的 Rust 镜像）。
//!
//! 按深层特征（可逆性/确定性/延迟预算/校准需求/域）选执行臂，
//! 而非字面关键词。域成绩由调用方从 router_table.json 喂入（无 IO）。
//! 无 unwrap / expect / panic。

/// 深层特征（vs 字面关键词）
#[derive(Debug, Clone, Default)]
pub struct NtDeepFeatures {
    pub domain: String,
    pub judgment: bool,
    pub reversible: bool,
    pub fast: bool,
    pub need_calib: bool,
}

/// 按域成绩行（router_table.json 同构子集）
#[derive(Debug, Clone)]
pub struct NtDomainScore {
    pub domain: String,
    pub laya_acc: f64,
    pub jev_acc: f64,
    pub n: usize,
}

/// 路由结果
#[derive(Debug, Clone)]
pub struct NtArmPick {
    /// "agentjev" | "laya"
    pub arm: &'static str,
    pub reasons: Vec<String>,
}

/// 特征抽取：纯规则（重型语义分类后补）。
pub fn extract_features(
    text: &str,
    domain: &str,
    fast: bool,
    need_calib: bool,
    irreversible: bool,
) -> NtDeepFeatures {
    let judgment_markers = [
        "?",
        "吗",
        "是否",
        "是不是",
        "可靠",
        "reliable",
        "which",
        "哪个",
    ];
    NtDeepFeatures {
        domain: domain.to_string(),
        judgment: judgment_markers.iter().any(|w| text.contains(w)),
        reversible: !irreversible,
        fast,
        need_calib,
    }
}

/// 选臂：不可逆→保守；域成绩（n≥20 才信）；快→Laya；要校准→brier 最优（Laya+tempfit）；默认门。
pub fn route(features: &NtDeepFeatures, table: &[NtDomainScore]) -> NtArmPick {
    let mut reasons = Vec::new();
    if !features.reversible {
        reasons.push("irreversible→conservative".to_string());
        return NtArmPick {
            arm: "agentjev",
            reasons,
        };
    }
    if let Some(d) = table.iter().find(|d| d.domain == features.domain) {
        if d.n >= 20 {
            if d.laya_acc - d.jev_acc > 0.02 {
                reasons.push(format!(
                    "domain {} laya+{:.2}",
                    d.domain,
                    d.laya_acc - d.jev_acc
                ));
                return NtArmPick {
                    arm: "laya",
                    reasons,
                };
            }
            if d.jev_acc - d.laya_acc > 0.02 {
                reasons.push(format!(
                    "domain {} jev+{:.2}",
                    d.domain,
                    d.jev_acc - d.laya_acc
                ));
                return NtArmPick {
                    arm: "agentjev",
                    reasons,
                };
            }
        }
    }
    if features.fast {
        reasons.push("latency_budget→laya 0.09s".to_string());
        return NtArmPick {
            arm: "laya",
            reasons,
        };
    }
    if features.need_calib {
        reasons.push("calib_need→brier best".to_string());
        return NtArmPick {
            arm: "laya",
            reasons,
        };
    }
    reasons.push("default gate".to_string());
    NtArmPick {
        arm: "agentjev",
        reasons,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_irreversible_goes_conservative() {
        let f = extract_features("删库吗？", "Finance", false, false, true);
        assert!(!f.reversible);
        let r = route(&f, &[]);
        assert_eq!(r.arm, "agentjev");
    }

    #[test]
    fn test_fast_and_calib_pick_laya() {
        let f = extract_features("这个快吗", "", true, false, false);
        assert_eq!(route(&f, &[]).arm, "laya");
        let f = extract_features("概率准吗", "", false, true, false);
        assert_eq!(route(&f, &[]).arm, "laya");
    }

    #[test]
    fn test_domain_scores_respected_with_n_floor() {
        let table = vec![NtDomainScore {
            domain: "X".to_string(),
            laya_acc: 0.9,
            jev_acc: 0.5,
            n: 100,
        }];
        let f = extract_features("x", "X", false, false, false);
        assert_eq!(route(&f, &table).arm, "laya");
        let thin = vec![NtDomainScore {
            domain: "Y".to_string(),
            laya_acc: 1.0,
            jev_acc: 0.0,
            n: 2,
        }];
        let f = extract_features("y", "Y", false, false, false);
        assert_eq!(route(&f, &thin).arm, "agentjev", "n<20 不信域成绩");
    }

    #[test]
    fn test_default_gate() {
        let f = extract_features("一般判断", "", false, false, false);
        let r = route(&f, &[]);
        assert_eq!(r.arm, "agentjev");
        assert!(!r.reasons.is_empty());
    }
}
