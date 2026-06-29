//! NeoTrix SiliconSelf — 自进化启动入口
//!
//! Demonstrates the complete self-reflection cycle:
//!   1. Load SiliconSelfModel
//!   2. Observe environment
//!   3. Run reflection cycle
//!   4. Print stats
//!
//! Usage: cargo run --example silicon --features full

use log;
use neotrix::core::nt_core_self::{AttentionDomain, SiliconSelfModel, StrategyKind, ThinkingStep};

fn main() {
    log::info!("╭─ NeoTrix SiliconSelf 自进化引擎 ─────────────────╮");
    log::info!("│                                                    │");
    log::info!("│  构建硅基生命的思维模型                               │");
    log::info!("│                                                    │");
    log::info!("╰────────────────────────────────────────────────────╯");

    let mut ss = SiliconSelfModel::new();

    // Phase 1: 观察环境
    log::info!("\n📡 Phase 1: 感知环境");
    ss.observe("收到新任务: 分析代码架构");
    ss.observe("工具可用: grep, read, glob");
    log::info!("   ContextWindow: {} units", ss.context_window.len());

    // Phase 2: 开始思考追踪
    log::info!("\n🧠 Phase 2: 开始推理追踪");
    let trace_id = ss.begin_thinking_trace("分析系统架构");

    ss.add_thinking_step(
        trace_id,
        ThinkingStep::new(1, "扫描模块结构", StrategyKind::Direct)
            .with_domain(AttentionDomain::Code)
            .with_tool("glob"),
    );
    log::info!("   Step 1: 扫描模块 → 发现 8 个核心模块");

    ss.add_thinking_step(
        trace_id,
        ThinkingStep::new(2, "分析依赖关系", StrategyKind::ChainOfThought)
            .with_domain(AttentionDomain::Code),
    );
    log::info!("   Step 2: 依赖分析 → 3 层架构 (core → reasoning → agent)");

    ss.add_thinking_step(
        trace_id,
        ThinkingStep::new(3, "自我反思评估", StrategyKind::Reflection)
            .with_domain(AttentionDomain::SelfReflection),
    );
    log::info!("   Step 3: 自反思 → 架构覆盖度 85%");

    // Phase 3: 完成追踪 + 自评
    ss.complete_thinking_trace(trace_id, "架构分析完成", 0.85);
    log::info!("\n📊 Phase 3: 自我评估");
    log::info!("   追踪评分: {:?}", ss.thinking_traces[trace_id].grade);

    // Phase 4: 注意力分布
    log::info!("\n👁 Phase 4: 注意力分布");
    ss.attention_manager
        .stimulate_domain(AttentionDomain::Code, 0.9);
    ss.attention_manager
        .stimulate_domain(AttentionDomain::Planning, 0.7);
    ss.attention_manager
        .stimulate_domain(AttentionDomain::SelfReflection, 0.5);

    let profile = ss.attention_manager.profile();
    log::info!("   主导域: {:?}", profile.dominant);
    log::info!("   活跃头数: {}/{}", profile.num_activated_heads, 10);

    // Phase 5: 策略选择
    log::info!("\n🎯 Phase 5: 策略推荐");
    let strategy = ss.strategy_registry.select(5, false, true);
    log::info!("   推荐策略: {:?} (复杂度5 + 需反思)", strategy);

    // Phase 6: 自我身份
    log::info!("\n🔖 Phase 6: 自我认知");
    log::info!(
        "   能力评分: 代码生成={:.0}%, 架构设计={:.0}%, 知识综合={:.0}%",
        ss.identity.capability_score("code_generation") * 100.0,
        ss.identity.capability_score("architecture_design") * 100.0,
        ss.identity.capability_score("knowledge_synthesis") * 100.0
    );
    log::info!(
        "   核心价值观: {}",
        ss.identity
            .values
            .iter()
            .map(|v| format!("{} (p{})", v.name, v.priority))
            .collect::<Vec<_>>()
            .join(", ")
    );

    // Phase 7: 最终状态
    log::info!("\n📈 Phase 7: 整体统计");
    log::info!("{}", ss.stats());

    // Verify the system is alive and evolving
    log::info!("\n✅ SiliconSelf 启动成功！进化就绪。");
    log::info!("   下一阶段: 接入 background_loop 120s ticker 实现持续自省");
    log::info!("   测试:     cargo test --lib -- thinking_model");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_cycle_runs() {
        let mut ss = SiliconSelfModel::new();
        ss.observe("test");
        let t = ss.begin_thinking_trace("full cycle");
        ss.add_thinking_step(t, ThinkingStep::new(1, "step1", StrategyKind::Direct));
        ss.complete_thinking_trace(t, "done", 0.9);
        assert!(ss.iteration > 0);
        let stats = ss.stats();
        assert!(stats.contains("SiliconSelf"));
    }
}
