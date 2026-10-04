//! Todo 入口（kanban `/board todo` 继任者）。
//!
//! 架构：任务管理已迁移到 AutoOrchestrator（意图→路由→实例）。
//! - `todo status`：编排器状态观测（实例/路由统计/最近路由）
//! - `sync|allocate|import`：旧管理命令已删除，改用 headless 自由文本意图。

use neotrix::l6_meta::nt_auto_orchestrator::AutoOrchestrator;

/// `todo` 入口：意图分类 → status 实装 / 管理命令指引。
pub fn run_todo(args: &[String]) -> Result<(), String> {
    let orchestrator = AutoOrchestrator::new();
    let classification = orchestrator.classify_intent(&format!("todo {}", args.join(" ")));
    println!(
        "🔍 意图识别: {:?} (conf={:.2})",
        classification.task_type, classification.confidence
    );
    let sub = args.first().map(|s| s.as_str()).unwrap_or("status");
    match sub {
        "status" => {
            let st = orchestrator.status();
            let ls = &st.lifecycle_stats;
            let rs = &st.routing_stats;
            // ⚠️ 2026-10-05 迁移到 nt_term_viz::panel。
            //
            // 首版缺陷：手画框 + 每行硬编码结尾 `│`，
            // 而 `instances:` / `tasks:` / `routes:` 三行**长度各不相同**、
            // 末尾空格数也不同 ⇒ 右边框参差。逐条实测（可见宽）：
            //   │ - [TaskKind 92%] 帮我实现一个解析器 → agent-1 (ok) │
            // 该行含 `truncate(&r.input, 24)`（≤24）+ task_type + agent_name
            // ⇒ 长度随数据浮动，而框宽固定 48 列 ⇒ 长路由必然冲出框外。
            //
            // 现两段式：先收集全部行，再 render_panel 一次算出内容宽。
            let mut lines: Vec<String> = vec![
                format!(
                    "instances: {} total ({} idle, {} running, {} paused)",
                    ls.total_instances, ls.idle, ls.running, ls.paused
                ),
                format!(
                    "tasks: {} total | cost: ${:.2} / ${:.2} budget",
                    ls.total_tasks, ls.total_cost, ls.cost_budget
                ),
                format!(
                    "routes: {} total, {} ok ({:.0}%), avg {}ms",
                    rs.total_routes, rs.successful_routes, rs.success_rate * 100.0, rs.avg_duration_ms
                ),
            ];
            if st.recent_routes.is_empty() {
                lines.push("(no routes yet — speak an intent in headless)".to_string());
            } else {
                for r in st.recent_routes.iter().take(10) {
                    lines.push(format!(
                        "- [{:?} {:.0}%] {} → {} ({})",
                        r.task_type,
                        r.confidence * 100.0,
                        truncate(&r.input, 24),
                        r.agent_name,
                        if r.success { "ok" } else { "FAIL" }
                    ));
                }
            }
            let borrowed: Vec<&str> = lines.iter().map(String::as_str).collect();
            for r in nt_term_viz::panel::render_panel("Orchestrator Status", &borrowed) {
                println!("{}", r);
            }
            Ok(())
        }
        "sync" | "allocate" | "import" => Err(format!(
            "todo {sub} 已随 kanban 移除：任务分配由 AutoOrchestrator 自动完成。\n\
             请在 headless 直接说出意图（例：\"帮我实现一个解析器\"），系统自动路由。\n\
             状态观测用: todo status"
        )),
        other => Err(format!("Unknown todo subcommand: {other}. Try: status")),
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let t: String = s.chars().take(max_len).collect();
        format!("{t}...")
    }
}
