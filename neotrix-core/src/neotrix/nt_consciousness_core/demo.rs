//! 迭代验证演示
//! 
//! 展示迭代验证引擎的完整运行流程

use crate::nt_consciousness_core::{
    IterationAgent,
    probes::*,
    patches::*,
    state::StateSnapshot,
};

/// 运行迭代验证演示
pub fn run_iteration_demo() {
    println!("╔═══════════════════════════════════════════════════════════════════════╗");
    println!("║         NeoTrix 意识核心 — 迭代验证引擎 (Iteration Verification)     ║");
    println!("╚═══════════════════════════════════════════════════════════════════════╝");
    println!();

    // 创建迭代验证 Agent
    let mut agent = IterationAgent::new(100, 5)  // 100周期，连续5次无漏洞收敛
        .with_probe_engine(Box::new(LogicProbe))
        .with_probe_engine(Box::new(ImplProbe))
        .with_probe_engine(Box::new(BoundaryProbe))
        .with_probe_engine(Box::new(ConsistencyProbe))
        .with_probe_engine(Box::new(PerformanceProbe))
        .with_probe_engine(Box::new(SecurityProbe))
        .with_probe_engine(Box::new(EvolutionProbe))
        .with_probe_engine(Box::new(ConsciousnessProbe))
        .with_probe_engine(Box::new(IntegrationProbe))
        .with_patch_generator(Box::new(LogicPatchGenerator))
        .with_patch_generator(Box::new(ImplPatchGenerator))
        .with_patch_generator(Box::new(BoundaryPatchGenerator))
        .with_patch_generator(Box::new(ConsistencyPatchGenerator))
        .with_patch_generator(Box::new(PerformancePatchGenerator))
        .with_patch_generator(Box::new(SecurityPatchGenerator))
        .with_patch_generator(Box::new(EvolutionPatchGenerator))
        .with_patch_generator(Box::new(ConsciousnessPatchGenerator))
        .with_patch_generator(Box::new(IntegrationPatchGenerator));

    // 运行迭代验证
    println!("🚀 开始迭代验证...");
    println!();

    let report = agent.run();

    // 打印报告
    println!("{}", report.stats);
    println!();
    println!("{}", report.convergence_proof);
    println!();

    // 打印元模式
    if !report.meta_patterns.is_empty() {
        println!("📊 发现的元模式:");
        for pattern in &report.meta_patterns {
            println!("  - {}: {}", pattern.id, pattern.description);
        }
    }
}

/// 运行单次迭代演示
pub fn run_single_iteration_demo(cycle: u32) {
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("  Cycle {} - 单次迭代演示", cycle);
    println!("═══════════════════════════════════════════════════════════════════════");
    println!();

    // 创建状态快照
    let state = StateSnapshot::new(cycle, 0.5 + (cycle as f64 * 0.001).min(0.5), 0.5 + (cycle as f64 * 0.0005).min(0.5));
    println!("{}", state);
    println!();

    // 创建探测器
    let probes: Vec<Box<dyn ProbeEngine>> = vec![
        Box::new(LogicProbe),
        Box::new(ImplProbe),
        Box::new(BoundaryProbe),
        Box::new(ConsistencyProbe),
        Box::new(PerformanceProbe),
        Box::new(SecurityProbe),
        Box::new(EvolutionProbe),
        Box::new(ConsciousnessProbe),
        Box::new(IntegrationProbe),
    ];

    // 探测漏洞
    let mut all_gaps = Vec::new();
    for probe in &probes {
        let gaps = probe.probe(&state, &GapRegistry::new());
        all_gaps.extend(gaps);
    }

    println!("🔍 探测到 {} 个漏洞:", all_gaps.len());
    for gap in &all_gaps {
        println!("  - [{}] {} ({}): {}", gap.severity, gap.dimension, gap.gap_type as u8, gap.description);
    }
    println!();

    // 生成补丁
    let patch_generators: Vec<Box<dyn PatchGenerator>> = vec![
        Box::new(LogicPatchGenerator),
        Box::new(ImplPatchGenerator),
        Box::new(BoundaryPatchGenerator),
        Box::new(ConsistencyPatchGenerator),
        Box::new(PerformancePatchGenerator),
        Box::new(SecurityPatchGenerator),
        Box::new(EvolutionPatchGenerator),
        Box::new(ConsciousnessPatchGenerator),
        Box::new(IntegrationPatchGenerator),
    ];

    let mut patches = Vec::new();
    for gap in &all_gaps {
        for generator in &patch_generators {
            if let Some(patch) = generator.generate(gap) {
                patches.push(patch);
            }
        }
    }

    println!("🛠️  生成 {} 个补丁:", patches.len());
    for patch in &patches {
        println!("  - [{}] {} -> {}", patch.confidence, patch.dimension, patch.content);
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_iteration() {
        run_single_iteration_demo(0);
    }
}
