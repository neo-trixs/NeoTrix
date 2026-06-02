//! NeoTrix 快速入门示例
//!
//! 运行: cargo run --example quick_start

use neotrix::{
    ReasoningBrain, SelfIteratingBrain, ReasoningBank,
    CapabilityVector, KnowledgeSource,
};

fn main() {
    // 1. 创建 ReasoningBrain
    let mut brain = ReasoningBrain::new();
    println!("=== 初始能力 ===");
    print_capability(&brain.capability);

    // 2. 吸收知识
    println!("\n=== 吸收 HeroUI 知识 ===");
    brain.absorb(KnowledgeSource::HeroUI);
    brain.absorb(KnowledgeSource::BaseUI);
    print_capability(&brain.capability);

    // 3. 创建 SEAL 自迭代系统
    let mut system = SelfIteratingBrain::new();
    println!("\n=== 运行 SEAL 循环 ===");
    let reward = system.run_seal_loop("设计一个 UI 组件", None, None);
    match reward {
        Ok(r) => println!("迭代奖励: {:.3}", r),
        Err(e) => println!("迭代失败: {}", e),
    }
    println!("记忆数: {}", system.reasoning_bank.stats().total_memories);

    // 4. 能力向量操作
    println!("\n=== 能力向量 ===");
    let mut cv = CapabilityVector::default();
    cv.set_typography(0.9);
    cv.set_grid(0.8);
    cv.set_accessibility(0.95);
    cv.add_extension_dim("custom_skill", 0.7);
    println!("总维度: {}", cv.total_dim());
    println!("扩展维度名: {:?}", cv.extension_names());
    println!("余弦相似度: {:.3}", cv.similarity(&CapabilityVector::default()));

    // 5. 代码审查
    println!("\n=== 代码审查 ===");
    let reviewer = neotrix::reasoning_brain::code_review::CodeReviewEngine::new(cv);
    let code = "fn main() { let x = foo.unwrap(); unsafe { *p = 1; } }";
    let report = reviewer.review("example.rs", code);
    println!("审查评分: {:.3}, 问题数: {}", report.score, report.total());
    for issue in &report.issues {
        println!("  [{:?}] {:?}: {}", issue.severity, issue.category, issue.message);
    }
}

fn print_capability(cv: &CapabilityVector) {
    println!("  typography={:.2}, grid={:.2}, color={:.2}",
        cv.typography(), cv.grid(), cv.color());
    println!("  accessibility={:.2}, analysis={:.2}",
        cv.accessibility(), cv.analysis());
}
