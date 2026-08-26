//! # G2 门控感知 — BubbleWall: 泡壁语义显式化 (灵境协议 4/7.2/9 工程转译)
//!
//! 吸收来源: Agent Infra 钱学森灵境引擎参赛方案 (2026-08-26)。R-P42 接线:
//! 强化 nt_core_consciousness 现有感知回路, 不建平行注意力模块。
//!
//! - **清晰区** (泡壁内): 与任务光锥相交的知识域 → 高精度注入意识流
//! - **迷雾区** (泡壁外): 仅保留 domain hub 统计量, 细节不展开 (协议 7.2 可见性门控)
//! - **面积律记账**: 认知成本 ∝ 清晰区表面积 (命中域×条目), 与语料总体积无关 —— 把
//! O(L²)/O(L³) 从物理叙事变成可测的认知成本定律 (协议 7.2 面积律)

use serde::{Deserialize, Serialize};

/// token 估算: ~4 字符/token (占位模型, 后续可换 tiktoken 级计数器)。
pub fn estimate_tokens(chars: usize) -> usize {
    chars.div_ceil(4)
}

/// 一次感知的面积律账单。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenBill {
    /// 清晰区注入内容的估算 token (表面成本)
    pub clear_tokens: usize,
    /// 清晰区注入条目数
    pub clear_entries: usize,
    /// 迷雾域数量 (仅统计量参与, 不展开细节)
    pub fog_domains: usize,
    /// 语料总体积节点数 (不进入成本, 只作对照)
    pub total_volume_nodes: i64,
}

/// 任务光锥投影出的泡壁。
#[derive(Debug, Clone, PartialEq)]
pub struct BubbleWall {
    /// 任务词元 (小写)
    pub center_terms: Vec<String>,
    /// 清晰区知识域 (已排序)
    pub clear_domains: Vec<String>,
    /// 迷雾域及其粗粒统计量 (node 数) —— 协议 9: 迷雾不消失, 只是降维
    pub fog_domains: Vec<(String, i64)>,
}

const TERM_SEPARATOR_CHARS: &[char] = &['_', '-', '/', ' ', '.', ':'];

fn tokenize(s: &str) -> Vec<String> {
    s.to_lowercase()
        .split(|c| TERM_SEPARATOR_CHARS.contains(&c) || !c.is_ascii_alphanumeric())
        .filter(|t| t.chars().count() >= 2)
        .map(|t| t.to_string())
        .collect()
}

impl BubbleWall {
    /// 任务 × 域分布 → 光锥投影。min_score ∈ [0,1]: 词元重合率阈值。
    /// 同参数同分布 ⇒ 同一泡壁 (确定性, 协议 6 兼容)。
    pub fn project(task: &str, by_domain: &[(String, i64)], min_score: f64) -> Self {
        let center_terms = tokenize(task);
        let mut clear = Vec::new();
        let mut fog = Vec::new();
        for (domain, nodes) in by_domain {
            let dparts = tokenize(domain);
            let overlap = center_terms.iter().filter(|t| dparts.contains(t)).count();
            let denom = center_terms.len().max(dparts.len()).max(1);
            let score = overlap as f64 / denom as f64;
            if score >= min_score {
                clear.push(domain.clone());
            } else {
                fog.push((domain.clone(), *nodes));
            }
        }
        clear.sort();
        Self {
            center_terms,
            clear_domains: clear,
            fog_domains: fog,
        }
    }

    /// 单条知识的可见性判定。无域标签的条目保守放行 (不因门控丢失未归类知识)。
    pub fn allows(&self, domain: Option<&str>) -> bool {
        match domain {
            None => true,
            Some(d) => self.clear_domains.iter().any(|c| c == d),
        }
    }

    /// 面积律账单: 成本只由清晰区内容决定, 迷雾只贡献域数 (O(表面积), 非 O(体积))。
    pub fn bill(&self, injected_title_chars: &[usize], total_volume_nodes: i64) -> TokenBill {
        let clear_chars: usize = injected_title_chars.iter().sum();
        TokenBill {
            clear_tokens: estimate_tokens(clear_chars),
            clear_entries: injected_title_chars.len(),
            fog_domains: self.fog_domains.len(),
            total_volume_nodes,
        }
    }
}

/// T5 判据工具: 对 (表面积, 成本) 样本做最小二乘拟合 cost=a·surface+b, 返回 R²。
/// 面积律成立 ⇔ R²→1 且成本随表面积而非体积增长。
pub fn area_law_r2(points: &[(f64, f64)]) -> f64 {
    let n = points.len() as f64;
    if n < 2.0 {
        return 0.0;
    }
    let sx: f64 = points.iter().map(|p| p.0).sum();
    let sy: f64 = points.iter().map(|p| p.1).sum();
    let sxx: f64 = points.iter().map(|p| p.0 * p.0).sum();
    let sxy: f64 = points.iter().map(|p| p.0 * p.1).sum();
    let denom = n * sxx - sx * sx;
    if denom.abs() < 1e-12 {
        return 0.0;
    }
    let slope = (n * sxy - sx * sy) / denom;
    let intercept = (sy - slope * sx) / n;
    let mean_y = sy / n;
    let ss_tot: f64 = points.iter().map(|p| (p.1 - mean_y).powi(2)).sum();
    if ss_tot.abs() < 1e-12 {
        return 1.0;
    }
    let ss_res: f64 = points
        .iter()
        .map(|p| (p.1 - (slope * p.0 + intercept)).powi(2))
        .sum();
    1.0 - ss_res / ss_tot
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_stats() -> Vec<(String, i64)> {
        vec![
            ("rust_lang".to_string(), 120),
            ("k8s_ops".to_string(), 80),
            ("ml_papers".to_string(), 9000),
        ]
    }

    #[test]
    fn test_project_deterministic_partition() {
        let w1 = BubbleWall::project("debug rust lang borrow checker", &sample_stats(), 0.34);
        let w2 = BubbleWall::project("debug rust lang borrow checker", &sample_stats(), 0.34);
        assert_eq!(w1, w2, "同参数同分布 ⇒ 同一泡壁");
        assert_eq!(w1.clear_domains, vec!["rust_lang".to_string()]);
        assert_eq!(w1.fog_domains.len(), 2);
    }

    #[test]
    fn test_allows_untagged_pass() {
        let w = BubbleWall::project("rust lang", &sample_stats(), 0.34);
        assert!(w.allows(Some("rust_lang")));
        assert!(!w.allows(Some("ml_papers")));
        assert!(w.allows(None), "未打域标签的知识保守放行");
    }

    #[test]
    fn test_area_law_cost_independent_of_volume() {
        // 核心断言: 清晰区相同、迷雾体积差 100 倍 → 账单成本必须相同 (O(表面积))
        let task = "rust lang ownership";
        let small = vec![
            ("rust_lang".to_string(), 100),
            ("ml_papers".to_string(), 500),
        ];
        let large = vec![
            ("rust_lang".to_string(), 100),
            ("ml_papers".to_string(), 50_000),
        ];
        let w_small = BubbleWall::project(task, &small, 0.34);
        let w_large = BubbleWall::project(task, &large, 0.34);

        let b_small = w_small.bill(&[24, 32], 600);
        let b_large = w_large.bill(&[24, 32], 50_100);
        assert_eq!(b_small.clear_tokens, b_large.clear_tokens);
        assert_eq!(b_small.clear_entries, b_large.clear_entries);
        assert_ne!(
            b_small.total_volume_nodes, b_large.total_volume_nodes,
            "体积不同但成本相同的对照必须成立"
        );
    }

    #[test]
    fn test_area_law_r2_perfect_fit() {
        let pts: Vec<(f64, f64)> = (1..=6).map(|r| ((r * r) as f64, 3.0 * (r * r) as f64 + 1.0)).collect();
        assert!(area_law_r2(&pts) > 0.999, "cost∝R² 完美拟合 R²≈1");
        assert_eq!(area_law_r2(&[(1.0, 2.0)]), 0.0, "样本不足返回 0");
    }
}
