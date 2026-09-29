//! CJK 字符判定 —— **两种语义各有唯一实现**（2026-09-29 立）
//!
//! ## 为什么要分两种，而不是统一成一个
//!
//! 此前全仓有 **8 个 `is_cjk` 副本、4 种口径**，看起来是纯重复。实测后发现
//! **它们是两种不同语义**，各自都对：
//!
//! | 语义 | 函数 | 覆盖 | 用途 | 若用错会怎样 |
//! |---|---|---|---|---|
//! | **分词** | [`is_cjk_han`] | 仅基本汉字 `4E00–9FFF` | bigram 切词、关键词提取 | ⛔ 纳入标点会产出垃圾 bigram（见下） |
//! | **计量** | [`is_cjk_wide`] | 汉字/假名/谚文/CJK标点/全角 | token 估算（1 token/char）、格式判断 | ⛔ 窄口径会把假名/谚文/全角错判为「英文 1/4 char」 |
//!
//! ### 为什么分词必须用窄口径（实测）
//!
//! ```text
//! "支付，网关"
//!   窄（仅汉字）→ run=支付网关  bigrams=[支付, 付网, 网关]      ✅ 无垃圾
//!   宽（含标点）→ run=支付，网关 bigrams=[支付, 付，, ，网, 网关] ⛔ 产垃圾
//! ```
//!
//! 假名/谚文/全角同理：把它们塞进 bigram 会得到跨文字系统的无意义二字组。
//!
//! ## 环��依赖已解除
//!
//! `context_strategy.rs` 的 `is_cjk` 曾注释「与 `neotrix-core::context_budget::is_cjk`
//! 口径一致」，而 **`context_budget` 模块根本不存在**；`nt_core_llm/mod.rs` 又反向
//! 指向 `context_strategy` ⇒ 两个「单一事实源」互指，且指针一端是虚的。
//! 现在两个事实源都在本文件，环消除。

/// **分词口径**：仅基本汉字（`4E00–9FFF`）。
///
/// ⛔ **不要**用它做 token 估算 —— 见 [`is_cjk_wide`]。
#[inline]
#[must_use]
pub fn is_cjk_han(c: char) -> bool {
    ('\u{4e00}'..='\u{9fff}').contains(&c)
}

/// **计量口径**：CJK 书写系统全集 —— 汉字 / 假名 / 谚文 / CJK 标点 / 全角。
///
/// 用于「这段文本按多少 token 算」这类**计量**判断：这些字符在主流 tokenizer
/// 里都接近 1 token/char，而 ASCII 约 1/4。
///
/// ⛔ **不要**用它做分词 —— 会把标点/全角塞进 bigram，见 [`is_cjk_han`]。
#[inline]
#[must_use]
pub fn is_cjk_wide(c: char) -> bool {
    matches!(
        c,
        '\u{3000}'..='\u{303F}'   // CJK 标点
        | '\u{3040}'..='\u{30FF}' // 平假名 / 片假名
        | '\u{3400}'..='\u{4DBF}' // CJK 扩展 A
        | '\u{4E00}'..='\u{9FFF}' // CJK 统一表意
        | '\u{AC00}'..='\u{D7AF}' // 谚文音节
        | '\u{FF00}'..='\u{FFEF}' // 全角 / 半角
    )
}

#[cfg(test)]
mod tests {
    use super::{is_cjk_han, is_cjk_wide};

    #[test]
    fn 分词口径_只认汉字_排除标点() {
        assert!(is_cjk_han('支') && is_cjk_han('付'));
        assert!(!is_cjk_han('，'), "标点必须被排除，否则 bigram 产垃圾");
        assert!(!is_cjk_han('あ'), "假名不属于汉字分词口径");
        assert!(!is_cjk_han('Ａ'), "全角字母不属于汉字分词口径");
    }

    #[test]
    fn 计量口径_覆盖全部_cjk_书写系统() {
        for c in ['支', 'あ', 'ア', '한', '，', 'Ａ', '　'] {
            assert!(is_cjk_wide(c), "计量口径应覆盖 {c:?}");
        }
        assert!(!is_cjk_wide('a') && !is_cjk_wide('1'));
    }

    /// ⛔ 这条锁住「两种口径不可混用」这个判断本身。
    /// 若有人后来把 `is_cjk_han` 改成宽口径，本测试会红。
    #[test]
    fn 窄口径是宽口径的真子集() {
        // 汉字两者都认
        assert!(is_cjk_wide('支'));
        // 标点/假名只有宽口径认 —— 这正是分词必须用窄口径的原因
        assert!(is_cjk_wide('，') && !is_cjk_han('，'));
        assert!(is_cjk_wide('あ') && !is_cjk_han('あ'));
    }

    /// 复现文档里那段：宽口径做分词会产垃圾 bigram。
    #[test]
    fn 宽口径_若误用于分词_会产垃圾_bigram() {
        let s = "支付，网关";
        let narrow: Vec<char> = s.chars().filter(|c| is_cjk_han(*c)).collect();
        let wide: Vec<char> = s.chars().filter(|c| is_cjk_wide(*c)).collect();
        let bg = |v: &[char]| -> Vec<String> {
            v.windows(2).map(|w| w.iter().collect()).collect()
        };
        assert_eq!(bg(&narrow), vec!["支付", "付网", "网关"]);
        let bad = bg(&wide);
        assert!(bad.contains(&"付，".to_string()), "宽口径确实产垃圾");
        assert!(bad.contains(&"，网".to_string()), "宽口径确实产垃圾");
    }
}
