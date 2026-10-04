//! brain — 从 `entry/mod.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。

use std::path::PathBuf;

use super::{info, warn};
use neotrix::l1_action::nt_core_bank::bank::ReasoningBank;
use neotrix::l5_cognition::nt_mind::nt_mind::self_iterating::{ReasoningBrain, SelfIteratingBrain};

/// 打印 brain 状态面板。
///
/// ⚠️ 2026-10-04 迁移到 `nt_term_viz::panel`（TERM-VIZ 清单最后一项）。
///
/// **首版的三处手画缺陷**（逐条实测，非观感问题）：
/// 1. `│ {:<5}` 用**字符数**补足，而标签 `Capability Sum:` 比 `Iteration:`
///    长 5 列 ⇒ 两行右边框落在**不同列**。
/// 2. 顶边 `╭─ NeoTrix V2 Brain Status ──…──╮` 与内容行**各自硬编码**，
///    宽度互不相关 ⇒ 框线参差。
/// 3. 底边是**另一串** `─`，与顶边长度无任何契约。
///
/// 现全部交给 `render_panel`：内容宽 = 最长内容的**可见列宽**，
/// 顶/底/内容行共享同一个 `content_width`。
pub(crate) fn print_brain_stats(brain: &SelfIteratingBrain) {
    use nt_term_viz::panel::render_panel;

    let stats = brain.brain.get_statistics();
    let rows = render_panel(
        "NeoTrix V2 Brain Status",
        &[
            &format!("{} {}", info("Iteration:"), brain.iteration),
            &format!("{} {}", info("Absorbed:"), brain.brain.total_absorb_count),
            &format!("{} {:.3}", info("Capability Sum:"), stats.capability_sum),
            &format!("{} {}", info("Memory:"), brain.reasoning_bank.memories().len()),
        ],
    );
    println!();
    for r in rows {
        // 染色整行（含边框）⇒ 宽度不受影响，视觉统一。
        println!("{}", info(r));
    }
}

/// 构造 brain 面板行 —— 测试与生产共用同一条渲染路径。
///
/// ⚠️ 刻意**复用** `render_panel` 而非在测试里另写一份格式串：
/// 否则测试测的是「测试自己」，生产改动它不会变红（自证循环）。
fn render_brain_stats_for_test(
    iteration: u64,
    absorbed: u64,
    capability_sum: f64,
    memory: usize,
) -> Vec<String> {
    use nt_term_viz::panel::render_panel;
    let rows = render_panel(
        "NeoTrix V2 Brain Status",
        &[
            &format!("{} {}", super::info("Iteration:"), iteration),
            &format!("{} {}", super::info("Absorbed:"), absorbed),
            &format!("{} {:.3}", super::info("Capability Sum:"), capability_sum),
            &format!("{} {}", super::info("Memory:"), memory),
        ],
    );
    rows.into_iter().map(|r| super::info(r)).collect()
}

fn strip_ansi(s: &str) -> String {
    nt_term_viz::table::strip_ansi(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nt_term_viz::display_width;

    /// 反向锁：首版的两行内容宽度**不等**，右边框参差。
    ///
    /// 复算首版格式串（ANSI 已剥）：
    /// · `│ Iteration: 7  Absorbed: 12` + 13 空格 ⇒ `│ `2 + `Iteration:`10 + ` `
    ///   1 + `7`1 + `  `2 + `Absorbed:`9 + ` `1 + `12`2 + 13 = **41**
    /// · `│ Capability Sum: 3.500  Memory: 9` + 7 空格 ⇒ `│ `2 + 16 + 1 + 5
    ///   + `  `2 + `Memory:`7 + ` `1 + `9`1 + 7 = **42**
    /// ⇒ 差 1 列。这正是手画 `{:<5}` + 硬编码空格必然踩的坑。
    #[test]
    fn brain_status_panel_lines_share_one_width() {
        let rows = render_brain_stats_for_test(7, 12, 3.5, 9);
        let widths: Vec<usize> = rows.iter().map(|r| display_width(&strip_ansi(r))).collect();
        assert!(
            widths.windows(2).all(|w| w[0] == w[1]),
            "brain 面板每行可见宽度应一致（右边框对齐），实测 {widths:?}"
        );
        assert_eq!(widths[0], display_width(&strip_ansi(&rows[0])));
    }

    /// 数值位数变化不得改变框宽（首版 `{:<5}` 在 ≥5 位时会撑开）。
    #[test]
    fn brain_panel_width_is_stable_across_magnitudes() {
        let small = render_brain_stats_for_test(1, 2, 0.5, 0);
        let big = render_brain_stats_for_test(123456789, 987654321, 999999.5, 1234567);
        let w = |rows: &[String]| display_width(&strip_ansi(&rows[1]));
        // 只断言「每组内部自洽」——跨组宽度可变（内容更长就该更宽），
        // 但**每组内所有行必须同宽**，这才是右边框对齐的充要条件。
        for rows in [&small, &big] {
            let ws: Vec<usize> = rows.iter().map(|r| display_width(&strip_ansi(r))).collect();
            assert!(
                ws.windows(2).all(|x| x[0] == x[1]),
                "组内行宽应一致，实测 {ws:?}"
            );
        }
        assert!(w(&big) > w(&small), "内容更长时框应变宽（反之说明宽度没跟上）");
    }
}

pub(crate) fn brain_dir(profile: &str) -> PathBuf {
    let base = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".neotrix");
    if profile.is_empty() || profile == "default" {
        base
    } else {
        base.join("profiles").join(profile)
    }
}

pub(crate) fn init_brain(profile: &str) -> (ReasoningBrain, ReasoningBank) {
    let dir = brain_dir(profile);
    std::env::set_var("NEOTRIX_HOME", &dir);

    if ReasoningBrain::has_saved_state() {
        match ReasoningBrain::load() {
            Ok(b) => {
                println!(
                    "{}",
                    info(format!("Loaded brain from {}/brain.json", dir.display()))
                );
                (b, ReasoningBank::new(100))
            }
            Err(e) => {
                eprintln!(
                    "{}",
                    warn(format!("Load failed ({}), creating new brain", e))
                );
                (ReasoningBrain::new(), ReasoningBank::new(100))
            }
        }
    } else {
        println!(
            "{}",
            info(format!("New brain at {}/brain.json", dir.display()))
        );
        (ReasoningBrain::new(), ReasoningBank::new(100))
    }
}

pub(crate) fn set_default_model_from_config(agent: &mut SelfIteratingBrain) {
    let cfg = neotrix::config::NeoTrixConfig::load();
    if let Some(ref model) = cfg.default_model {
        if !model.is_empty() {
            agent.default_model = model.clone();
        }
    }
}

/// 构建自进化 brain — 抽取 7 处重复初始化样板 (审计 R-P99 去重)。
///
/// 核心 5 步: init_brain → SelfIteratingBrain::new → 挂载 brain/reasoning_bank
/// → set_default_model_from_config → ensure_provider_env_from_config → init_reasoning_engine。
/// 带 load_cortex 的变体 (run_daemon/evolution) 不共用, 因顺序不同。
pub(crate) fn build_brain(profile: &str) -> SelfIteratingBrain {
    let (brain, bank) = init_brain(profile);
    let mut agent = SelfIteratingBrain::new();
    agent.brain = brain;
    agent.reasoning_bank = bank;
    set_default_model_from_config(&mut agent);
    ensure_provider_env_from_config();
    agent.init_reasoning_engine();
    attach_eval_harness(&mut agent);
    agent
}

/// 把 L6 的评测闸门工厂装配进 brain（B5b，2026-09-29）。
///
/// ## 为什么在 `entry` 层接线
///
/// `entry` 是**顶层编排层** —— 它本就可以看见 L0–L6 全部。
/// 而 L5（`SelfIteratingBrain`）与 L6（`EvalHarness`）**都不该**做这次接线：
/// - L5 不能 `use crate::l6_meta::*`（`l5_cognition/traits.rs:133-139` 明令）
/// - L6 若自己往 L5 的 brain 字段里塞东西，等于绕过 trait 依赖倒置
///
/// ⇒ 装配点必须在**同时认识两侧**的地方。`entry` 正是那个位置。
///
/// ## ⛔ 为什么这件事重要（不是「锦上添花」）
///
/// A9 之前，`seal_loop` 的自进化闭环钩子硬传 `None`，
/// 而 `EvalHarnessApi` 的 impl 早已存在 ⇒ **实现了但永远收不到调用**。
/// 效果上：**候选变更从不接受回归检验就直接持久化**。
///
/// 这与本轮根治的 `check-doc-drift` 恒红是**同一个病**：
/// 一个「存在但不起作用」的东西，比不存在更危险 ——
/// 它让人以为防护在位。
fn attach_eval_harness(agent: &mut SelfIteratingBrain) {
    use neotrix::l5_cognition::traits::EvalHarnessFactory;

    let factory = neotrix::l6_meta::healing::nt_mind_eval_harness::DefaultEvalHarnessFactory;
    match factory.make_eval_harness() {
        Some(h) => {
            agent.eval_harness = Some(h);
        }
        None => {
            // ⛔ 显式告警：闸门缺失会让自进化闭环**静默退化为无验证**。
            //   `seal_loop` 会打 debug 日志，但 debug 在生产常被关掉 ⇒ 这里升到 warn。
            super::warn(
                "评测闸门未能装配 ⇒ 自进化闭环的回归检验**不会运行**（候选变更将直接持久化）",
            );
        }
    }
}

/// 将 config.toml 中的 provider/api_key 提升为环境变量，使 GatewayV2 能发现
pub(crate) fn ensure_provider_env_from_config() {
    let cfg = neotrix::config::NeoTrixConfig::load();
    if let (Some(provider), Some(api_key)) = (&cfg.provider, &cfg.api_key) {
        if !api_key.is_empty() {
            match provider.as_str() {
                "openai" => {
                    if std::env::var("OPENAI_API_KEY").is_err() {
                        std::env::set_var("OPENAI_API_KEY", api_key);
                    }
                }
                "anthropic" => {
                    if std::env::var("ANTHROPIC_API_KEY").is_err() {
                        std::env::var("ANTHROPIC_API_KEY").ok();
                        std::env::set_var("ANTHROPIC_API_KEY", api_key);
                    }
                }
                "custom" => {
                    if std::env::var("NEOTRIX_API_KEY").is_err() {
                        std::env::set_var("NEOTRIX_API_KEY", api_key);
                    }
                    if let Some(ref endpoint) = cfg.custom_endpoint {
                        if std::env::var("NEOTRIX_BASE_URL").is_err() {
                            std::env::set_var("NEOTRIX_BASE_URL", endpoint);
                        }
                    }
                    if let Some(ref model) = cfg.default_model {
                        if std::env::var("NEOTRIX_MODEL").is_err() {
                            std::env::set_var("NEOTRIX_MODEL", model);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
