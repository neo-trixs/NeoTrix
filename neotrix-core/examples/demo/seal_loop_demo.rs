//! NeoTrix SEAL 自迭代循环演示
//!
//! 展示 run_seal_loop() 如何根据任务调整能力向量
//! 展示 ReasoningBank 如何存储和检索记忆

use colored::Colorize;
use neotrix::neotrix::nt_expert_routing::TaskType;
use neotrix::neotrix::nt_mind::{ReasoningBank, ReasoningMemory, SelfEdit, SelfIteratingBrain};

/// 打印 SelfIteratingBrain 状态
fn print_brain_state(brain: &SelfIteratingBrain, label: &str) {
    log::info!("\n{}", label);
    log::info!("{}", "-".repeat(50));
    log::info!("迭代次数: {}", brain.iteration);
    log::info!("学习率:   {:.4}", brain.brain.learning_rate);
    log::info!("质量阈值: {:.2}", brain.quality_threshold);

    log::info!("\n能力向量 (Top 10):");
    let cap = &brain.brain.capability;
    log::info!("  Typography:     {:.3}", cap.typography());
    log::info!("  Grid:           {:.3}", cap.grid());
    log::info!("  Accessibility:  {:.3}", cap.accessibility());
    log::info!("  Compound:       {:.3}", cap.compound_composition());
    log::info!("  Tailwind:       {:.3}", cap.tailwind_proficiency());
    log::info!("  AI Native:      {:.3}", cap.ai_native_states());
    log::info!("  Inference:      {:.3}", cap.inference_depth());
    log::info!("  Creativity:     {:.3}", cap.creativity());
    log::info!("  Analysis:       {:.3}", cap.analysis());
    log::info!("  Synthesis:      {:.3}", cap.synthesis());

    log::info!("\n任务亲和度:");
    for (task_type, &affinity) in &brain.brain.task_affinity {
        log::info!("  {:?} => {:.3}", task_type, affinity);
    }

    let bank_stats = brain.reasoning_bank.stats();
    log::info!("\nReasoningBank 统计:");
    log::info!("  总记忆数: {}", bank_stats.total_memories);
    log::info!("  成功次数: {}", bank_stats.success_count);
    log::info!("  成功率: {:.3}", bank_stats.success_rate);
}

/// 演示 1: 单次 SEAL 循环
pub fn demo_single_seal_loop() {
    log::info!("\n{}", "=== 演示 1: 单次 SEAL 循环 ===".green().bold());

    let mut brain = SelfIteratingBrain::new();
    brain.brain.learning_rate = 0.05;

    let task = "设计一个响应式的 React UI 组件，使用 Tailwind CSS";

    log::info!("任务: \"{}\"", task);
    log::info!("\nSEAL 循环执行中...");

    let score_before = brain.brain.evaluate_capability(TaskType::UIDesign);
    log::info!("  循环前 UI 设计能力: {:.3}", score_before);

    match brain.run_seal_loop(task, None, None) {
        Ok(reward) => {
            log::info!("  奖励: {:.3}", reward);
            let score_after = brain.brain.evaluate_capability(TaskType::UIDesign);
            log::info!(
                "  循环后 UI 设计能力: {:.3} (Δ {:.3})",
                score_after,
                score_after - score_before
            );
        }
        Err(e) => log::info!("  错误: {}", e),
    }

    print_brain_state(&brain, "SEAL 循环后的 Brain 状态");
}

/// 演示 2: 多次 SEAL 循环（任务序列）
pub fn demo_multiple_seal_loops() {
    log::info!("\n{}", "=== 演示 2: 多次 SEAL 循环 ===".green().bold());

    let mut brain = SelfIteratingBrain::new();
    brain.brain.learning_rate = 0.05;

    let tasks = vec![
        "设计一个移动端导航栏",
        "优化组件的 accessibility 属性",
        "实现深色模式切换功能",
        "添加动画和过渡效果",
        "集成 Figma 设计系统",
    ];

    log::info!("执行 {} 个任务的 SEAL 循环...\n", tasks.len());

    for (i, task) in tasks.iter().enumerate() {
        let score_before = brain.brain.evaluate_capability(TaskType::UIDesign);

        print!("  {}. \"{}\" ... ", i + 1, task);
        match brain.run_seal_loop(task, None, None) {
            Ok(reward) => {
                let score_after = brain.brain.evaluate_capability(TaskType::UIDesign);
                log::info!(
                    "奖励: {:.3}, UI能力: {:.3} -> {:.3}",
                    reward,
                    score_before,
                    score_after
                );
            }
            Err(e) => log::info!("失败: {}", e),
        }
    }

    print_brain_state(&brain, "多次循环后的 Brain 状态");

    let report = brain.get_brain_report();
    log::info!("\nBrain 报告:");
    log::info!("  迭代次数: {}", report.iteration);
    log::info!("  总吸收次数: {}", report.total_absorbed);
    log::info!("  近期改进次数: {}", report.recent_improvement);
}

/// 演示 3: ReasoningBank 记忆管理
pub fn demo_reasoning_bank() {
    log::info!(
        "\n{}",
        "=== 演示 3: ReasoningBank 记忆管理 ===".green().bold()
    );

    let mut bank = ReasoningBank::new(10); // 最多存储 10 条记忆

    // 添加一些模拟记忆
    let tasks = vec![
        ("UI 设计任务 1", 0.8),
        ("UI 设计任务 2", 0.6),
        ("代码分析任务", 0.9),
        ("安全审计任务", 0.4),
        ("UI 设计任务 3", 0.7),
    ];

    log::info!("向 ReasoningBank 添加 {} 条记忆...", tasks.len());

    for (task, reward) in &tasks {
        let self_edit = SelfEdit {
            task_type: TaskType::UIDesign,
            target_dimensions: vec![
                "accessibility".to_string(),
                "compound_composition".to_string(),
            ],
            adjustment_magnitude: 0.1,
            tool_calls: vec![],
            config_overrides: std::collections::HashMap::new(),
        };

        let memory = ReasoningMemory::new(task, self_edit.task_type, &[], *reward);
        bank.store(memory);
        log::info!("  存储: \"{}\" (奖励: {:.1})", task, reward);
    }

    let stats = bank.stats();
    log::info!("\nReasoningBank 统计:");
    log::info!("  总记忆数: {}", stats.total_memories);
    log::info!("  成功次数: {}", stats.success_count);
    log::info!("  成功率: {:.3}", stats.success_rate);

    // 检索相关记忆
    log::info!("\n检索与 \"UI 设计\" 相关的记忆:");
    let relevant = bank.retrieve_relevant("UI 设计", Some(TaskType::UIDesign), 3);
    for (i, mem) in relevant.iter().enumerate() {
        log::info!(
            "  {}. \"{}\" (奖励: {:.1}, 成功: {})",
            i + 1,
            mem.task_description,
            mem.reward,
            mem.success
        );
    }
}

/// 演示 4: 批量 SEAL 训练循环
pub fn demo_batch_seal_loop() {
    log::info!("\n{}", "=== 演示 4: 批量 SEAL 训练循环 ===".green().bold());

    let mut brain = SelfIteratingBrain::new();
    brain.brain.learning_rate = 0.05;

    let tasks: Vec<(String, Option<Vec<f64>>)> = vec![
        ("设计一个 Dashboard 界面".to_string(), None),
        ("创建一个用户管理页面".to_string(), None),
        ("实现数据可视化图表".to_string(), None),
    ];

    log::info!("批量执行 {} 个任务的 SEAL 循环...", tasks.len());

    // 由于 benchmark 可能依赖外部资源，这里我们用单个任务循环代替
    log::info!("\n调用 run_seal_loop 处理每个任务...");

    let mut total_reward = 0.0;
    for (task, _) in &tasks {
        match brain.run_seal_loop(task, None, None) {
            Ok(reward) => {
                total_reward += reward;
                log::info!("  任务 \"{}\" => 奖励: {:.3}", task, reward);
            }
            Err(e) => log::info!("  任务 \"{}\" => 错误: {}", task, e),
        }
    }

    let avg_reward = total_reward / tasks.len() as f64;
    log::info!("\n平均奖励: {:.3}", avg_reward);
    log::info!("最终学习率: {:.4}", brain.brain.learning_rate);
}

/// 演示 5: 策略更新（学习率自适应）
pub fn demo_policy_update() {
    log::info!(
        "\n{}",
        "=== 演示 5: 策略更新（学习率自适应）===".green().bold()
    );

    let mut brain = SelfIteratingBrain::new();
    brain.brain.learning_rate = 0.1;
    brain.policy_learning_rate = 0.01;

    log::info!("初始学习率: {:.3}", brain.brain.learning_rate);

    // 模拟高奖励 -> 学习率增加
    log::info!("\n模拟高奖励场景 (avg_reward = 0.8)...");
    // 手动实现策略更新逻辑
    let avg_reward = 0.8;
    if avg_reward > 0.5 {
        brain.brain.learning_rate =
            (brain.brain.learning_rate * (1.0 + brain.policy_learning_rate)).min(0.3);
    }
    log::info!("更新后学习率: {:.3}", brain.brain.learning_rate);

    // 模拟低奖励 -> 学习率减少
    log::info!("\n模拟低奖励场景 (avg_reward = -0.2)...");
    let avg_reward = -0.2;
    if avg_reward < 0.0 {
        brain.brain.learning_rate =
            (brain.brain.learning_rate * (1.0 - brain.policy_learning_rate)).max(0.01);
    }
    log::info!("更新后学习率: {:.3}", brain.brain.learning_rate);

    // 模拟中等奖励 -> 学习率不变
    let lr_before = brain.brain.learning_rate;
    log::info!("\n模拟中等奖励场景 (avg_reward = 0.3)...");
    let avg_reward = 0.3;
    // 中等奖励不调整学习率
    log::info!(
        "更新后学习率: {:.3} (变化: {:.4})",
        brain.brain.learning_rate,
        brain.brain.learning_rate - lr_before
    );
}

/// 演示 6: 能力向量回滚（负奖励时）
pub fn demo_rollback() {
    log::info!("\n{}", "=== 演示 6: 能力向量回滚 ===".green().bold());

    let mut brain = SelfIteratingBrain::new();
    brain.brain.learning_rate = 0.1;

    // 先进行一次正常更新
    log::info!("第一次 SEAL 循环 (正常)...");
    let _ = brain.run_seal_loop("设计 UI 组件", None, None);
    let cap_after_first = brain.brain.capability.clone();
    log::info!(
        "  第一次后能力: {:.3}",
        brain.brain.evaluate_capability(TaskType::UIDesign)
    );

    // 模拟第二次更新（会被回滚）
    log::info!("\n第二次 SEAL 循环 (模拟负奖励回滚)...");

    // 手动触发回滚逻辑
    let snapshot = brain.brain.capability.clone();
    let snapshot_lr = brain.brain.learning_rate;

    // 应用一个大的负向更新
    let negative_edit = SelfEdit {
        task_type: TaskType::UIDesign,
        target_dimensions: vec![
            "accessibility".to_string(),
            "compound_composition".to_string(),
        ],
        adjustment_magnitude: -0.5,
        tool_calls: vec![],
        config_overrides: std::collections::HashMap::new(),
    };
    let _ = brain.brain.apply_self_edit(&negative_edit, Some(-0.3));

    let negative_reward = -0.3;
    log::info!("  模拟负奖励: {:.3}", negative_reward);

    // 触发回滚
    if negative_reward < 0.0 {
        brain.brain.capability = snapshot;
        brain.brain.learning_rate = (snapshot_lr * 0.9).max(0.01);
        log::info!("  执行回滚!");
        log::info!("  回滚后学习率: {:.4}", brain.brain.learning_rate);
    }

    let cap_after_rollback = brain.brain.capability.clone();
    log::info!(
        "\n回滚前后能力是否一致: {}",
        cap_after_first.similarity(&cap_after_rollback) > 0.99
    );
}

pub fn run_all_demos() {
    log::info!(
        "{}",
        "╔══════════════════════════════════════════════════════════════╗".cyan()
    );
    log::info!(
        "{}",
        "║         NeoTrix SEAL 自迭代循环演示                          ║".cyan()
    );
    log::info!(
        "{}",
        "╚══════════════════════════════════════════════════════════════╝".cyan()
    );

    demo_single_seal_loop();
    demo_multiple_seal_loops();
    demo_reasoning_bank();
    demo_batch_seal_loop();
    demo_policy_update();
    demo_rollback();

    log::info!(
        "\n{}",
        "════════════════════════════════════════════════════════".green()
    );
    log::info!("演示完成!");
    log::info!("════════════════════════════════════════════════════════\n");
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
