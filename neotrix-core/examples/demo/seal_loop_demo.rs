//! NeoTrix SEAL 自迭代循环演示
//!
//! 展示 run_seal_loop() 如何根据任务调整能力向量
//! 展示 ReasoningBank 如何存储和检索记忆

use neotrix::neotrix::nt_mind::{
    SelfIteratingBrain, ReasoningBank, ReasoningMemory,
    KnowledgeSource, SelfEdit,
};
use neotrix::neotrix::nt_world_model::TaskType;
use colored::Colorize;

/// 打印 SelfIteratingBrain 状态
fn print_brain_state(brain: &SelfIteratingBrain, label: &str) {
    println!("\n{}", label);
    println!("{}", "-".repeat(50));
    println!("迭代次数: {}", brain.iteration);
    println!("学习率:   {:.4}", brain.brain.learning_rate);
    println!("质量阈值: {:.2}", brain.quality_threshold);

    println!("\n能力向量 (Top 10):");
    let cap = &brain.brain.capability;
    println!("  Typography:     {:.3}", cap.typography);
    println!("  Grid:           {:.3}", cap.grid);
    println!("  Accessibility:  {:.3}", cap.accessibility);
    println!("  Compound:       {:.3}", cap.compound_composition);
    println!("  Tailwind:       {:.3}", cap.tailwind_proficiency);
    println!("  AI Native:      {:.3}", cap.ai_native_states);
    println!("  Inference:      {:.3}", cap.inference_depth);
    println!("  Creativity:     {:.3}", cap.creativity);
    println!("  Analysis:       {:.3}", cap.analysis);
    println!("  Synthesis:      {:.3}", cap.synthesis);

    println!("\n任务亲和度:");
    for (task_type, &affinity) in &brain.brain.task_affinity {
        println!("  {:?} => {:.3}", task_type, affinity);
    }

    let bank_stats = brain.reasoning_bank.stats();
    println!("\nReasoningBank 统计:");
    println!("  总记忆数: {}", bank_stats.total_memories);
    println!("  成功次数: {}", bank_stats.success_count);
    println!("  成功率: {:.3}", bank_stats.success_rate);
}

/// 演示 1: 单次 SEAL 循环
pub fn demo_single_seal_loop() {
    println!("\n{}", "=== 演示 1: 单次 SEAL 循环 ===".green().bold());

    let mut brain = SelfIteratingBrain::new();
    brain.brain.learning_rate = 0.05;

    let task = "设计一个响应式的 React UI 组件，使用 Tailwind CSS";

    println!("任务: \"{}\"", task);
    println!("\nSEAL 循环执行中...");

    let score_before = brain.brain.evaluate_capability(TaskType::UIDesign);
    println!("  循环前 UI 设计能力: {:.3}", score_before);

    match brain.run_seal_loop(task, None, None) {
        Ok(reward) => {
            println!("  奖励: {:.3}", reward);
            let score_after = brain.brain.evaluate_capability(TaskType::UIDesign);
            println!("  循环后 UI 设计能力: {:.3} (Δ {:.3})",
                     score_after, score_after - score_before);
        }
        Err(e) => println!("  错误: {}", e),
    }

    print_brain_state(&brain, "SEAL 循环后的 Brain 状态");
}

/// 演示 2: 多次 SEAL 循环（任务序列）
pub fn demo_multiple_seal_loops() {
    println!("\n{}", "=== 演示 2: 多次 SEAL 循环 ===".green().bold());

    let mut brain = SelfIteratingBrain::new();
    brain.brain.learning_rate = 0.05;

    let tasks = vec![
        "设计一个移动端导航栏",
        "优化组件的 accessibility 属性",
        "实现深色模式切换功能",
        "添加动画和过渡效果",
        "集成 Figma 设计系统",
    ];

    println!("执行 {} 个任务的 SEAL 循环...\n", tasks.len());

    for (i, task) in tasks.iter().enumerate() {
        let score_before = brain.brain.evaluate_capability(TaskType::UIDesign);

        print!("  {}. \"{}\" ... ", i + 1, task);
        match brain.run_seal_loop(task, None, None) {
            Ok(reward) => {
                let score_after = brain.brain.evaluate_capability(TaskType::UIDesign);
                println!("奖励: {:.3}, UI能力: {:.3} -> {:.3}",
                         reward, score_before, score_after);
            }
            Err(e) => println!("失败: {}", e),
        }
    }

    print_brain_state(&brain, "多次循环后的 Brain 状态");

    let report = brain.get_brain_report();
    println!("\nBrain 报告:");
    println!("  迭代次数: {}", report.iteration);
    println!("  总吸收次数: {}", report.total_absorbed);
    println!("  近期改进次数: {}", report.recent_improvement);
}

/// 演示 3: ReasoningBank 记忆管理
pub fn demo_reasoning_bank() {
    println!("\n{}", "=== 演示 3: ReasoningBank 记忆管理 ===".green().bold());

    let mut bank = ReasoningBank::new(10); // 最多存储 10 条记忆

    // 添加一些模拟记忆
    let tasks = vec![
        ("UI 设计任务 1", 0.8),
        ("UI 设计任务 2", 0.6),
        ("代码分析任务", 0.9),
        ("安全审计任务", 0.4),
        ("UI 设计任务 3", 0.7),
    ];

    println!("向 ReasoningBank 添加 {} 条记忆...", tasks.len());

    for (task, reward) in &tasks {
        let self_edit = SelfEdit {
            task_type: TaskType::UIDesign,
            target_dimensions: vec!["accessibility".to_string(), "compound_composition".to_string()],
            adjustment_magnitude: 0.1,
            tool_calls: vec![],
            config_overrides: std::collections::HashMap::new(),
        };

        let memory = ReasoningMemory::new(task, &self_edit, *reward);
        bank.store(memory);
        println!("  存储: \"{}\" (奖励: {:.1})", task, reward);
    }

    let stats = bank.stats();
    println!("\nReasoningBank 统计:");
    println!("  总记忆数: {}", stats.total_memories);
    println!("  成功次数: {}", stats.success_count);
    println!("  成功率: {:.3}", stats.success_rate);

    // 检索相关记忆
    println!("\n检索与 \"UI 设计\" 相关的记忆:");
    let relevant = bank.retrieve_relevant("UI 设计", 3);
    for (i, mem) in relevant.iter().enumerate() {
        println!("  {}. \"{}\" (奖励: {:.1}, 成功: {})",
                 i + 1, mem.task_description, mem.reward, mem.success);
    }
}

/// 演示 4: 批量 SEAL 训练循环
pub fn demo_batch_seal_loop() {
    println!("\n{}", "=== 演示 4: 批量 SEAL 训练循环 ===".green().bold());

    let mut brain = SelfIteratingBrain::new();
    brain.brain.learning_rate = 0.05;

    let tasks: Vec<(String, Option<Vec<f64>>)> = vec![
        ("设计一个 Dashboard 界面".to_string(), None),
        ("创建一个用户管理页面".to_string(), None),
        ("实现数据可视化图表".to_string(), None),
    ];

    println!("批量执行 {} 个任务的 SEAL 循环...", tasks.len());

    // 由于 benchmark 可能依赖外部资源，这里我们用单个任务循环代替
    println!("\n调用 run_seal_loop 处理每个任务...");

    let mut total_reward = 0.0;
    for (task, _) in &tasks {
        match brain.run_seal_loop(task, None, None) {
            Ok(reward) => {
                total_reward += reward;
                println!("  任务 \"{}\" => 奖励: {:.3}", task, reward);
            }
            Err(e) => println!("  任务 \"{}\" => 错误: {}", task, e),
        }
    }

    let avg_reward = total_reward / tasks.len() as f64;
    println!("\n平均奖励: {:.3}", avg_reward);
    println!("最终学习率: {:.4}", brain.brain.learning_rate);
}

/// 演示 5: 策略更新（学习率自适应）
pub fn demo_policy_update() {
    println!("\n{}", "=== 演示 5: 策略更新（学习率自适应）===".green().bold());

    let mut brain = SelfIteratingBrain::new();
    brain.brain.learning_rate = 0.1;
    brain.policy_learning_rate = 0.01;

    println!("初始学习率: {:.3}", brain.brain.learning_rate);

    // 模拟高奖励 -> 学习率增加
    println!("\n模拟高奖励场景 (avg_reward = 0.8)...");
    // 手动实现策略更新逻辑
    let avg_reward = 0.8;
    if avg_reward > 0.5 {
        brain.brain.learning_rate = (brain.brain.learning_rate * (1.0 + brain.policy_learning_rate)).min(0.3);
    }
    println!("更新后学习率: {:.3}", brain.brain.learning_rate);

    // 模拟低奖励 -> 学习率减少
    println!("\n模拟低奖励场景 (avg_reward = -0.2)...");
    let avg_reward = -0.2;
    if avg_reward < 0.0 {
        brain.brain.learning_rate = (brain.brain.learning_rate * (1.0 - brain.policy_learning_rate)).max(0.01);
    }
    println!("更新后学习率: {:.3}", brain.brain.learning_rate);

    // 模拟中等奖励 -> 学习率不变
    let lr_before = brain.brain.learning_rate;
    println!("\n模拟中等奖励场景 (avg_reward = 0.3)...");
    let avg_reward = 0.3;
    // 中等奖励不调整学习率
    println!("更新后学习率: {:.3} (变化: {:.4})",
             brain.brain.learning_rate,
             brain.brain.learning_rate - lr_before);
}

/// 演示 6: 能力向量回滚（负奖励时）
pub fn demo_rollback() {
    println!("\n{}", "=== 演示 6: 能力向量回滚 ===".green().bold());

    let mut brain = SelfIteratingBrain::new();
    brain.brain.learning_rate = 0.1;

    // 先进行一次正常更新
    println!("第一次 SEAL 循环 (正常)...");
    let _ = brain.run_seal_loop("设计 UI 组件", None, None);
    let cap_after_first = brain.brain.capability.clone();
    println!("  第一次后能力: {:.3}", brain.brain.evaluate_capability(TaskType::UIDesign));

    // 模拟第二次更新（会被回滚）
    println!("\n第二次 SEAL 循环 (模拟负奖励回滚)...");

    // 手动触发回滚逻辑
    let snapshot = brain.brain.capability.clone();
    let snapshot_lr = brain.brain.learning_rate;

    // 应用一个大的负向更新
    let mut negative_edit = brain.brain.generate_self_edit("糟糕的任务");
    negative_edit.adjustment_magnitude = -0.5;
    brain.brain.apply_self_edit(&negative_edit);

    let negative_reward = -0.3;
    println!("  模拟负奖励: {:.3}", negative_reward);

    // 触发回滚
    if negative_reward < 0.0 {
        brain.brain.capability = snapshot;
        brain.brain.learning_rate = (snapshot_lr * 0.9).max(0.01);
        println!("  执行回滚!");
        println!("  回滚后学习率: {:.4}", brain.brain.learning_rate);
    }

    let cap_after_rollback = brain.brain.capability.clone();
    println!("\n回滚前后能力是否一致: {}",
             cap_after_first.similarity(&cap_after_rollback) > 0.99);
}

pub fn run_all_demos() {
    println!("{}", "╔══════════════════════════════════════════════════════════════╗".cyan());
    println!("{}", "║         NeoTrix SEAL 自迭代循环演示                          ║".cyan());
    println!("{}", "╚══════════════════════════════════════════════════════════════╝".cyan());

    demo_single_seal_loop();
    demo_multiple_seal_loops();
    demo_reasoning_bank();
    demo_batch_seal_loop();
    demo_policy_update();
    demo_rollback();

    println!("\n{}", "════════════════════════════════════════════════════════".green());
    println!("演示完成!");
    println!("════════════════════════════════════════════════════════\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_runs() {
        let brain = SelfIteratingBrain::new();
        assert_eq!(brain.iteration, 0);
        assert_eq!(brain.brain.total_absorb_count, 0);
    }
}
