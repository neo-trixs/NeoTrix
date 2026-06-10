use std::time::Instant;
use neotrix::neotrix::nt_memory_kb::KnowledgeBase;
use neotrix::neotrix::nt_world_jepa::JepaWorldModel;
use neotrix::neotrix::nt_mind::prediction_trainer::PredictionTrainer;
use neotrix::neotrix::nt_world_crawl::data_connector::{
    ExternalDataConnector, DataSourceType,
};

const FEATURE_DIM: usize = 32;
const TOTAL_ITERATIONS: usize = 50;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  NeoTrix 外部数据采集 + JEPA 预测训练 — 50 轮循环          ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    let kb = KnowledgeBase::open(None).expect("Failed to open knowledge base");
    let mut jepa = JepaWorldModel::new(FEATURE_DIM);
    let mut trainer = PredictionTrainer::new();

    let mut total_ingested = 0usize;
    let mut round_times: Vec<f64> = Vec::new();
    let mut accuracy_history: Vec<f64> = Vec::new();
    let mut loss_history: Vec<f64> = Vec::new();
    let mut ingested_per_round: Vec<usize> = Vec::new();
    let mut pairs_history: Vec<usize> = Vec::new();

    let kb_start = kb.stats().unwrap_or_default();
    println!("\n  📊 Initial KB: {} nodes, {} edges\n",
        kb_start.total_nodes, kb_start.total_edges);

    // Seed foundational knowledge first if KB is empty
    if kb_start.total_nodes < 10 {
        println!("  🌱 Seeding foundational knowledge...");
        match kb.seed_foundational() {
            Ok(n) => println!("  ✅ Seeded {} nodes", n),
            Err(e) => println!("  ⚠ Seed error: {}", e),
        }
    }

    for round in 1..=TOTAL_ITERATIONS {
        let round_start = Instant::now();
        println!("━━━ Round {}/{} ━━━", round, TOTAL_ITERATIONS);

        // Phase 1: Collect data from diverse external sources
        let sources = select_sources(round, &trainer);
        let mut round_records = 0usize;
        for source in &sources {
            let start = Instant::now();
            let records = ExternalDataConnector::collect_from(*source);
            let elapsed = start.elapsed().as_secs_f64();
            print!("  📡 {}: {} recs [{:.1}s]", source.name(), records.len(), elapsed);
            if !records.is_empty() {
                match ExternalDataConnector::ingest_to_kb(&records, &kb) {
                    Ok(n) => {
                        total_ingested += n;
                        round_records += n;
                        println!(" → ingested {}", n);
                    }
                    Err(e) => println!(" → {}", e),
                }
            } else {
                println!(" → (empty)");
            }
        }
        ingested_per_round.push(round_records);

        // Phase 2: Train JEPA on KB graph relationships
        let train_start = Instant::now();
        let metrics = trainer.train_on_kb(&mut jepa, &kb);
        let train_elapsed = train_start.elapsed().as_secs_f64();
        let round_elapsed = round_start.elapsed().as_secs_f64();
        round_times.push(round_elapsed);
        accuracy_history.push(metrics.accuracy);
        loss_history.push(metrics.avg_loss);
        pairs_history.push(metrics.total_pairs);

        // Phase 3: Report
        let quality = trainer.prediction_quality();
        let trend = trainer.accuracy_trend_slope();
        let trend_char = if trend > 0.01 { '+' } else if trend < -0.01 { '-' } else { '=' };

        let kb_stats = kb.stats().unwrap_or_default();
        println!(
            "  🧠 JEPA: pairs={} loss={:.4} acc={:.1}% qual={:.2} {}trend={:.4} [{:.1}s]",
            metrics.total_pairs, metrics.avg_loss, metrics.accuracy * 100.0,
            quality, trend_char, trend, train_elapsed,
        );
        println!(
            "  📊 KB: {} nodes {} edges | +{} recs | round={:.1}s ∑={:.1}s",
            kb_stats.total_nodes, kb_stats.total_edges,
            round_records, round_elapsed, round_times.iter().sum::<f64>(),
        );

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // Final report
    let kb_end = kb.stats().unwrap_or_default();
    let avg_time = round_times.iter().sum::<f64>() / round_times.len() as f64;
    let final_quality = trainer.prediction_quality();
    let final_trend = trainer.accuracy_trend_slope();

    let best_acc = accuracy_history.iter().cloned().fold(0.0, f64::max);
    let final_acc = accuracy_history.last().copied().unwrap_or(0.0);

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  50 轮循环完成                                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("  📊 知识库增长:");
    println!("     Nodes:   {} → {} (+{})",
        kb_start.total_nodes, kb_end.total_nodes,
        kb_end.total_nodes - kb_start.total_nodes);
    println!("     Edges:   {} → {} (+{})",
        kb_start.total_edges, kb_end.total_edges,
        kb_end.total_edges - kb_start.total_edges);
    println!("     Total ingested records: {}", total_ingested);
    println!();
    println!("  🧠 预测能力:");
    println!("     Final accuracy:  {:.1}%", final_acc * 100.0);
    println!("     Best accuracy:   {:.1}%", best_acc * 100.0);
    println!("     Quality score:   {:.3}", final_quality);
    println!("     Trend slope:     {:.4} {}",
        final_trend,
        if final_trend > 0.01 { "(improving)" }
        else if final_trend < -0.01 { "(declining)" }
        else { "(stable)" });
    println!("     JEPA train steps: {}", jepa.training_steps);
    println!();
    println!("  ⏱ 性能:");
    println!("     Avg round: {:.1}s", avg_time);
    println!("     Total:     {:.1}s", round_times.iter().sum::<f64>());
    println!();

    // Save metrics as JSON
    let metrics_output = serde_json::json!({
        "kb_nodes_before": kb_start.total_nodes,
        "kb_nodes_after": kb_end.total_nodes,
        "kb_edges_before": kb_start.total_edges,
        "kb_edges_after": kb_end.total_edges,
        "total_ingested": total_ingested,
        "jepa_training_steps": jepa.training_steps,
        "final_accuracy": final_acc,
        "best_accuracy": best_acc,
        "final_quality": final_quality,
        "trend_slope": final_trend,
        "avg_round_time_secs": avg_time,
        "total_time_secs": round_times.iter().sum::<f64>(),
        "accuracy_history": accuracy_history,
        "loss_history": loss_history,
        "pairs_per_round": pairs_history,
        "ingested_per_round": ingested_per_round,
    });

    let metrics_path = dirs::home_dir()
        .map(|p| p.join(".neotrix").join("data_ingest_metrics.json"))
        .unwrap_or_else(|| std::path::PathBuf::from("data_ingest_metrics.json"));

    match std::fs::write(&metrics_path, serde_json::to_string_pretty(&metrics_output).unwrap()) {
        Ok(_) => println!("  💾 Metrics saved to: {:?}", metrics_path),
        Err(e) => println!("  ⚠ Failed to save metrics: {}", e),
    }

    // Print accuracy timeline
    println!("\n  📈 Accuracy timeline (every 5 rounds):");
    for chunk in accuracy_history.chunks(5) {
        let round_num = accuracy_history.len() - chunk.len() + 1;
        let avg = chunk.iter().sum::<f64>() / chunk.len() as f64;
        print!("     round {:>2}-{:>2}: {:5.1}%",
            round_num, round_num + chunk.len() - 1, avg * 100.0);
        let bar_len = (avg * 30.0) as usize;
        print!(" {}", "█".repeat(bar_len));
        println!();
    }
}

fn select_sources(round: usize, trainer: &PredictionTrainer) -> Vec<DataSourceType> {
    let quality = trainer.prediction_quality();
    let base = round % 4;

    let sources = match base {
        0 => vec![DataSourceType::HackerNews, DataSourceType::ArXiv],
        1 => vec![DataSourceType::GitHubTrending, DataSourceType::SemanticScholar],
        2 => vec![DataSourceType::Wikipedia, DataSourceType::HackerNews],
        _ => vec![DataSourceType::ArXiv, DataSourceType::GitHubTrending],
    };

    if quality < 0.3 {
        let mut expanded = sources;
        expanded.push(DataSourceType::Wikipedia);
        expanded
    } else if quality < 0.6 {
        let mut expanded = sources;
        expanded.push(DataSourceType::HackerNews);
        expanded
    } else {
        sources
    }
}
