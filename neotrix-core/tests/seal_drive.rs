//! 意识核心预测凝结验证驱动
//! 运行 SelfIteratingBrain seal_loop 5 轮, 观察 E8 预测模型是否开始凝结
#![forbid(unsafe_code)]

use neotrix::SelfIteratingBrain;

#[test]
fn test_seal_predictive_condensation() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();
    let mut brain = SelfIteratingBrain::new();

    let tasks = [
        "分析这段Rust代码的错误处理是否健壮，给出改进建议",
        "设计一个分布式缓存的一致性方案",
        "解释知识图谱中节点重要性如何计算",
        "对一个并发任务的死锁场景做根因分析",
        "评估这个机器学习模型过拟合的应对策略",
        "为订单系统设计幂等的支付接口",
        "分析微服务链路追踪中如何定位慢查询",
        "设计一个支持冲突合并的分布式存储方案",
    ];

    let mut rewards = Vec::new();
    for (i, task) in tasks.iter().enumerate() {
        let preview: String = task.chars().take(20).collect();
        println!("\n[seal-drive] === 轮次 {}: {} ===", i + 1, preview);
        match brain.run_seal_loop(task, None, None) {
            Ok(r) => {
                rewards.push(r);
                println!("[seal-drive] 轮次 {} 完成, reward={:.4}", i + 1, r);
            }
            Err(e) => {
                println!("[seal-drive] 轮次 {} 错误: {}", i + 1, e);
            }
        }
        // 每轮后观察凝结进程
        if let Some(ref engine) = brain.reasoning_engine {
            let top = engine
                .last_e8_attention_weights
                .as_ref()
                .map(|aw: &Vec<f64>| aw.iter().fold(0.0f64, |m, &x| m.max(x)))
                .unwrap_or(0.0);
            println!(
                "[seal-drive]   ⟪轮{} 轨迹={}步 注意力峰值={:.4} confidence={:.4} mode={}⟫",
                i + 1,
                engine.state_trajectory.len(),
                top,
                engine.last_e8_confidence,
                engine.current_state.mode.0
            );
        }
    }

    // 打印 E8 状态
    if let Some(ref engine) = brain.reasoning_engine {
        println!("\n[seal-drive] === E8 预测状态 ===");
        println!("  current_mode: {}", engine.current_state.mode.0);
        println!("  trajectory_modes: {:?}", engine.state_trajectory);
        println!("  轨迹展开长度: {}", engine.state_trajectory.len());
        if let Some(ref aw) = engine.last_e8_attention_weights {
            let top: f64 = aw.iter().fold(0.0f64, |m, &x| m.max(x));
            let actives = aw.iter().filter(|x| **x > 0.01).count();
            println!(
                "  注意力: {}维, 激活(>0.01)={}, 峰值={:.4}",
                aw.len(),
                actives,
                top
            );
        }
        println!("  last_confidence: {:.4}", engine.last_e8_confidence);
    }

    assert!(!rewards.is_empty(), "seal_loop 应产生至少一轮 reward");
    println!("\n[seal-drive] 预测凝结验证完成");
}
