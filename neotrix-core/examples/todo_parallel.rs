//! 示例：多任务并行执行
//! 使用 tokio 任务实现并发
//!
//! 注意: neotrix::parallel 模块已不存在，
//! 这里直接用 tokio::spawn 演示并发模式

async fn simulate_task(id: &str, desc: &str, complexity: f64) -> String {
    let work = (complexity * 100.0) as u64;
    tokio::time::sleep(tokio::time::Duration::from_millis(work)).await;
    format!("[{}] {} — 复杂度: {:.1} ✅", id, desc, complexity)
}

#[tokio::main]
async fn main() {
    log::info!("=== NeoTrix 多任务并行执行示例 ===\n");

    let task_data: Vec<(String, String, f64)> = vec![
        ("task1".into(), "修复编译错误".into(), 0.8),
        ("task2".into(), "实现并行功能".into(), 0.9),
        ("task3".into(), "完善渲染组件".into(), 0.7),
        ("task4".into(), "集成推理能力".into(), 0.6),
        ("task5".into(), "实现工具集成".into(), 0.75),
    ];

    log::info!("已添加 {} 个任务\n", task_data.len());
    log::info!("开始并行执行...");

    let handles: Vec<_> = task_data
        .into_iter()
        .map(|(id, desc, complexity)| {
            tokio::spawn(async move { simulate_task(&id, &desc, complexity).await })
        })
        .collect();

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap());
    }

    log::info!("\n所有任务执行完成，共 {} 个结果:", results.len());
    for (i, output) in results.iter().enumerate() {
        log::info!("  {}. {}", i + 1, output);
    }

    log::info!("\n=== 示例结束 ===");
}
