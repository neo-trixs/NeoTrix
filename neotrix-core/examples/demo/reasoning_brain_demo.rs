//! NeoTrix ReasoningBrain 能力进化演示
//!
//! 展示吸收不同知识源（HeroUI, BaseUI 等）后能力向量的变化
//! 使用 ReasoningBrain::absorb() 和 evaluate_capability()

use neotrix::reasoning_brain::{
    ReasoningBrain, KnowledgeSource, CapabilityVector,
};
use neotrix::world_model::TaskType;
use colored::Colorize;

/// 打印能力向量（只显示关键维度）
fn print_capability_vector(brain: &ReasoningBrain, label: &str) {
    println!("\n{}", label);
    println!("{}", "-".repeat(50));
    println!("Typography:       {:.3}", brain.capability.typography());
    println!("Grid:              {:.3}", brain.capability.grid());
    println!("Color:             {:.3}", brain.capability.color());
    println!("Accessibility:     {:.3}", brain.capability.accessibility());
    println!("Compound Comp:     {:.3}", brain.capability.compound_composition());
    println!("Tailwind:          {:.3}", brain.capability.tailwind_proficiency());
    println!("React ARIA:        {:.3}", brain.capability.react_aria_usage());
    println!("AI Native:         {:.3}", brain.capability.ai_native_states());
    println!("Semantic Layer:    {:.3}", brain.capability.semantic_layer());
    println!("Quality Gates:     {:.3}", brain.capability.quality_gates());
    println!("Inference Depth:   {:.3}", brain.capability.inference_depth());
    println!("Creativity:        {:.3}", brain.capability.creativity());
    println!("Analysis:          {:.3}", brain.capability.analysis());
    println!("Synthesis:         {:.3}", brain.capability.synthesis());
}

/// 评估并打印各任务类型的能力分数
fn evaluate_and_print(brain: &ReasoningBrain) {
    let task_types = vec![
        TaskType::Design,
        TaskType::UIDesign,
        TaskType::CodeAnalysis,
        TaskType::CodeGeneration,
        TaskType::Security,
        TaskType::Planning,
    ];

    println!("\n{}", "能力评估:".bold());
    println!("{}", "-".repeat(50));
    for task_type in task_types {
        let score = brain.evaluate_capability(task_type);
        println!("  {:15} => {:.3}", format!("{:?}", task_type), score);
    }
}

/// 演示 1: 初始状态
pub fn demo_initial_state() {
    println!("\n{}", "=== 演示 1: 初始状态 ===".green().bold());

    let brain = ReasoningBrain::new();
    print_capability_vector(&brain, "初始能力向量 (全为 0)");
    evaluate_and_print(&brain);

    let stats = brain.get_statistics();
    println!("\n统计信息:");
    println!("  总吸收次数: {}", stats.total_absorbed);
    println!("  唯一知识源: {:?}", stats.unique_sources);
}

/// 演示 2: 吸收单个知识源
pub fn demo_absorb_single() {
    println!("\n{}", "=== 演示 2: 吸收 HeroUI ===".green().bold());

    let mut brain = ReasoningBrain::new();
    println!("吸收前: UI 设计能力 = {:.3}", brain.evaluate_capability(TaskType::UIDesign));

    brain.absorb(KnowledgeSource::HeroUI);

    print_capability_vector(&brain, "吸收 HeroUI 后");
    println!("\n吸收后: UI 设计能力 = {:.3}", brain.evaluate_capability(TaskType::UIDesign));

    let stats = brain.get_statistics();
    println!("\n统计信息:");
    println!("  总吸收次数: {}", stats.total_absorbed);
}

/// 演示 3: 批量吸收多个知识源
pub fn demo_absorb_batch() {
    println!("\n{}", "=== 演示 3: 批量吸收多个知识源 ===".green().bold());

    let mut brain = ReasoningBrain::new();

    let sources = vec![
        KnowledgeSource::HeroUI,
        KnowledgeSource::BaseUI,
        KnowledgeSource::ArcUI,
        KnowledgeSource::CortexUI,
        KnowledgeSource::AgenticDS,
    ];

    println!("依次吸收: HeroUI, BaseUI, ArcUI, CortexUI, AgenticDS");

    for (i, &source) in sources.iter().enumerate() {
        let score_before = brain.evaluate_capability(TaskType::UIDesign);
        brain.absorb(source);
        let score_after = brain.evaluate_capability(TaskType::UIDesign);
        println!("  {}. 吸收 {:15} => UI能力: {:.3} -> {:.3} (Δ {:.3})",
                 i + 1,
                 format!("{:?}", source),
                 score_before,
                 score_after,
                 score_after - score_before);
    }

    print_capability_vector(&brain, "批量吸收后的能力向量");
    evaluate_and_print(&brain);
}

/// 演示 4: 知识源特性对比
pub fn demo_knowledge_comparison() {
    println!("\n{}", "=== 演示 4: 知识源特性对比 ===".green().bold());

    let sources = vec![
        KnowledgeSource::HeroUI,
        KnowledgeSource::BaseUI,
        KnowledgeSource::ArcUI,
        KnowledgeSource::CortexUI,
        KnowledgeSource::AgenticDS,
    ];

    println!("\n{:15} | {:10} | {:10} | {:10} | {:10} | {:10}",
             "知识源", "Compound", "Tailwind", "ReactARIA", "Access", "AINative");
    println!("{}", "-".repeat(75));

    for source in sources {
        let vec: CapabilityVector = source.capability_vector();
        println!("{:15} | {:10.3} | {:10.3} | {:10.3} | {:10.3} | {:10.3}",
                 source.name(),
                 vec.compound_composition,
                 vec.tailwind_proficiency,
                 vec.react_aria_usage,
                 vec.accessibility,
                 vec.ai_native_states);
    }
}

/// 演示 5: 任务亲和度学习
pub fn demo_task_affinity() {
    println!("\n{}", "=== 演示 5: 任务亲和度学习 ===".green().bold());

    let mut brain = ReasoningBrain::new();

    // 模拟多次 UI 设计任务
    println!("\n模拟 5 次 UI 设计任务...");
    for i in 0..5 {
        brain.absorb(KnowledgeSource::HeroUI);
        brain.update_task_affinity(TaskType::UIDesign, 0.8 + i as f64 * 0.02);
    }

    println!("\n任务亲和度:");
    for (task_type, &affinity) in &brain.task_affinity {
        println!("  {:?} => {:.3}", task_type, affinity);
    }

    println!("\n最终 UI 设计能力: {:.3}", brain.evaluate_capability(TaskType::UIDesign));
}

/// 演示 6: 能力向量归一化
pub fn demo_normalization() {
    println!("\n{}", "=== 演示 6: 能力向量归一化 ===".green().bold());

    let mut brain = ReasoningBrain::new();
    brain.absorb(KnowledgeSource::HeroUI);
    brain.absorb(KnowledgeSource::BaseUI);

    println!("\n归一化前的能力向量:");
    let max_before = get_max_value(&brain.capability);
    println!("  Max value: {:.3}", max_before);

    brain.capability.normalize();

    println!("\n归一化后的能力向量:");
    let max_after = get_max_value(&brain.capability);
    println!("  Max value: {:.3}", max_after);
    print_capability_vector(&brain, "归一化后");
}

fn get_max_value(cv: &CapabilityVector) -> f64 {
    let values: Vec<f64> = vec![
        cv.typography, cv.grid, cv.color, cv.whitespace,
        cv.data_viz, cv.emotion, cv.minimalism, cv.experimental,
        cv.inference_depth, cv.creativity, cv.analysis, cv.synthesis,
        cv.domain_specificity,
        cv.accessibility, cv.compound_composition, cv.tailwind_proficiency,
        cv.react_aria_usage, cv.bem_naming, cv.figma_integration,
        cv.ai_native_states, cv.semantic_layer, cv.quality_gates,
        cv.verification,
    ];
    values.into_iter().fold(0.0f64, |a, b| a.max(b))
}

/// 演示 7: 生成自编辑
pub fn demo_generate_self_edit() {
    println!("\n{}", "=== 演示 7: 生成自编辑 ===".green().bold());

    let brain = ReasoningBrain::new();

    let tasks = vec![
        "设计一个响应式 UI 界面",
        "分析这段代码的性能问题",
        "生成一个安全的用户认证模块",
        "规划一个分布式系统架构",
    ];

    for task in tasks {
        let self_edit = brain.generate_self_edit(task);
        println!("\n任务: \"{}\"", task);
        println!("  任务类型: {:?}", self_edit.task_type);
        println!("  目标维度: {:?}", self_edit.target_dimensions);
        println!("  调整幅度: {:.3}", self_edit.adjustment_magnitude);
        println!("  工具调用: {} 个", self_edit.tool_calls.len());
    }
}

pub fn run_all_demos() {
    println!("{}", "╔══════════════════════════════════════════════════════════════╗".cyan());
    println!("{}", "║     NeoTrix ReasoningBrain 能力进化演示                      ║".cyan());
    println!("{}", "╚══════════════════════════════════════════════════════════════╝".cyan());

    demo_initial_state();
    demo_absorb_single();
    demo_absorb_batch();
    demo_knowledge_comparison();
    demo_task_affinity();
    demo_normalization();
    demo_generate_self_edit();

    println!("\n{}", "══════════════════════════════════════════════════════════".green());
    println!("演示完成!");
    println!("════════════════════════════════════════════════════════════\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_runs() {
        // 简单验证演示可以运行而不崩溃
        let brain = ReasoningBrain::new();
        assert_eq!(brain.capability.typography, 0.0);
        assert_eq!(brain.total_absorb_count, 0);
    }
}
