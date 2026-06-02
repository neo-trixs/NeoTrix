//! 示例：多子agent同步执行 TODO 任务
//! 展示 ParallelExecutor 的并发执行能力

use neotrix::parallel::executor::{ParallelExecutor, ExecMode};
use neotrix::parallel::types::TodoTask;

#[tokio::main]
async fn main() {
    println!("=== NeoTrix 多子agent同步执行示例 ===");

    // 1. 创建 ParallelExecutor（最大并发数 4）
    let mut executor = ParallelExecutor::new(4);
    executor.set_mode(ExecMode::Parallel);

    // 2. 定义 TODO 任务列表
    let todo_tasks = vec![
        TodoTask::new("task1".to_string(), "修复编译错误".to_string(), "code".to_string())
            .with_priority(1)
            .with_complexity(0.8),
        TodoTask::new("task2".to_string(), "实现多子agent同步".to_string(), "parallel".to_string())
            .with_priority(2)
            .with_complexity(0.9),
        TodoTask::new("task3".to_string(), "完善Renderer渲染".to_string(), "render".to_string())
            .with_priority(1)
            .with_complexity(0.7),
        TodoTask::new("task4".to_string(), "集成ReasoningBrain到UI".to_string(), "ui".to_string())
            .with_priority(3)
            .with_complexity(0.6),
        TodoTask::new("task5".to_string(), "实现MCP Tools集成".to_string(), "mcp".to_string())
            .with_priority(2)
            .with_complexity(0.75),
    ];

    // 3. 将 TODO 任务添加到执行器
    // 每个任务分配一个 agent（这里简单用任务ID作为agent_id）
    for task in &todo_tasks {
        // 将复杂度作为向量输入（实际应用可能更复杂）
        let input = vec![task.estimated_complexity, task.priority as f64];
        executor.add_task(task.id.clone(), input, task.priority);
    }

    println!("已添加 {} 个任务到执行器", todo_tasks.len());

    // 4. 同步执行所有任务（多子agent并发）
    println!("开始执行任务（多子agent同步）...");
    let results = executor.execute().await;

    println!("所有任务执行完成，共 {} 个结果", results.len());
    for (i, output) in results.iter().enumerate() {
        println!("  任务 {} 输出: {:?}", i + 1, output);
    }

    println!("=== 示例结束 ===");
}
