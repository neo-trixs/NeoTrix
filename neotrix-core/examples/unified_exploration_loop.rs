use std::path::PathBuf;
use std::time::{Duration, Instant};
use neotrix::neotrix::nt_mind::{
    ReasoningBrain, ReasoningBank,
    exploration_pipeline::ExplorationPipeline,
};
fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  🌐 统一探索管道 — 3 小时自治循环                          ║");
    println!("║  超心理学 · 神学 · 玄学 · Wiki · 论文 · GitHub             ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let work_dir = PathBuf::from(&home).join(".neotrix");
    std::fs::create_dir_all(&work_dir).ok();

    let mut brain = ReasoningBrain::load().unwrap_or_default();
    let mut bank = ReasoningBank::new(10000);
    let mut pipeline = ExplorationPipeline::new(work_dir.clone());

    let start = Instant::now();
    let total_duration = Duration::from_secs(3 * 3600);
    let mut round = 0u64;
    let mut total_mined = 0usize;

    while start.elapsed() < total_duration {
        round += 1;
        let elapsed = start.elapsed();
        let remaining = total_duration.saturating_sub(elapsed);
        println!("\n── Round #{} | elapsed={:?} | remaining={:?} ──", round, elapsed, remaining);

        let result = pipeline.run_round(&mut brain, &mut bank);
        total_mined += result.total_mined;

        if result.total_mined > 0 {
            println!("  ✅ +{} mined, +{} KE entries, reward={:.3}",
                result.total_mined, result.ke_entries_added, result.total_reward);
        }
        for d in &result.details {
            println!("  {}", d);
        }

        let stats = pipeline.stats();
        println!("  📊 stats: rounds={}, web_mined={}, gh_mined={}, KE={}, queue={}",
            stats.rounds, stats.web_mined, stats.gh_mined,
            stats.ke_entries, stats.queued);

        let _ = brain.save();

        if remaining.as_secs() > 0 {
            let sleep_secs = remaining.as_secs().min(600).max(60);
            println!("  😴 sleeping {}s until next round...", sleep_secs);
            std::thread::sleep(Duration::from_secs(sleep_secs));
        }
    }

    let _ = brain.save();
    let stats = pipeline.stats();
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  📊 探索管道 — 3 小时执行报告                               ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  总轮次: {:>4}                                              ║", round);
    println!("║  总挖掘: {:>4} web + {:>4} GitHub = {:>4}                  ║",
        stats.web_mined, stats.gh_mined, stats.web_mined + stats.gh_mined);
    println!("║  KE 条目: {:>4}                                            ║", stats.ke_entries);
    println!("║  队列中: {:>4}                                              ║", stats.queued);
    println!("║  已处理: {:>4}                                              ║", stats.processed);
    println!("╚══════════════════════════════════════════════════════════════╝");
}
