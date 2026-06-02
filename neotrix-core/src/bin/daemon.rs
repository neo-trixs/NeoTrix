/// NeoTrix Autonomous Daemon
///
/// Starts the full pipeline: background loop + KS activation + metacognition bridge + validation.
/// Self-healing: auto-detects knowledge gaps and generates stub modules.
/// Writes health status to /tmp/neotrix_daemon.health for external monitoring.
///
/// Usage: cargo run --bin daemon [--features full]
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

use neotrix::neotrix::nt_mind_background_loop::BackgroundLoop;
use neotrix::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
use neotrix::core::nt_core_knowledge::{KSActivationEngine, TaskType};
use neotrix::neotrix::nt_mind::self_iterating::validation::{cargo_check_validation, ValidationResult};

fn health_path() -> String {
    std::env::var("NEOTRIX_HEALTH_FILE").unwrap_or_else(|_| "/tmp/neotrix_daemon.health".to_string())
}

fn write_health(msg: &str) {
    let content = format!("[{}] {}", chrono_now(), msg);
    let _ = std::fs::write(health_path(), &content);
}

fn chrono_now() -> String {
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let h = (dur.as_secs() / 3600) % 24;
    let m = (dur.as_secs() / 60) % 60;
    let s = dur.as_secs() % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

#[tokio::main]
async fn main() {
    println!("╭─ NeoTrix Autonomous Daemon ─────────────────────────╮");
    println!("│ Starting full pipeline: BG loop + KS + Val + Meta   │");
    println!("│ Self-healing: gap detection → auto-fix → verify     │");

    // 1. Initialize brain
    let dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".neotrix");
    std::env::set_var("NEOTRIX_HOME", &dir);

    let (brain, bank) = if neotrix::neotrix::nt_mind::self_iterating::ReasoningBrain::has_saved_state() {
        match neotrix::neotrix::nt_mind::self_iterating::ReasoningBrain::load() {
            Ok(b) => {
                println!("[daemon] Loaded brain from {}/brain.json", dir.display());
                (b, neotrix::neotrix::nt_mind::memory::ReasoningBank::new(100))
            }
            Err(e) => {
                eprintln!("[daemon] Load failed ({}), creating new brain", e);
                (neotrix::neotrix::nt_mind::self_iterating::ReasoningBrain::new(),
                 neotrix::neotrix::nt_mind::memory::ReasoningBank::new(100))
            }
        }
    } else {
        println!("[daemon] New brain at {}/brain.json", dir.display());
        (neotrix::neotrix::nt_mind::self_iterating::ReasoningBrain::new(),
         neotrix::neotrix::nt_mind::memory::ReasoningBank::new(100))
    };

    let mut agent = SelfIteratingBrain::new();
    agent.brain = brain;
    agent.reasoning_bank = bank;
    agent.auto_absorb = true;
    agent.auto_memory_iteration = true;

    let brain_arc = Arc::new(RwLock::new(agent));

    // 2. Initialize KS Activation Engine
    let mut ks_engine = KSActivationEngine::new();
    println!("[daemon] KS engine: {} sources", ks_engine.engine_size());

    // 3. Initialize MetaGoalBridge
    let _meta_bridge = neotrix::core::nt_core_meta::MetaGoalBridge::new();
    println!("[daemon] MetaGoalBridge ready");

    // 4. Initialize self-healing gap detector
    let kb_path = dir.join("knowledge.db");
    let kb = if kb_path.exists() {
        match neotrix::neotrix::nt_memory_kb::KnowledgeBase::open(Some(kb_path.clone())) {
            Ok(k) => {
                let node_count = match k.conn.lock() {
                    Ok(conn) => match neotrix::neotrix::nt_memory_kb::nt_memory_store::get_stats(&conn) {
                        Ok(stats) => stats.total_nodes,
                        Err(_) => -1,
                    },
                    Err(_) => -1,
                };
                println!("[daemon] KB loaded ({} nodes)", node_count);
                Some(Arc::new(Mutex::new(k)))
            }
            Err(e) => {
                eprintln!("[daemon] KB open failed: {}", e);
                None
            }
        }
    } else {
        println!("[daemon] No KB at {}, healing without persistence", kb_path.display());
        None
    };

    let mut heal_engine = neotrix::neotrix::nt_core_knowledge_gap::NeotrixGapDetector::new(kb);
    println!("[daemon] Self-healing engine ready");

    // 5. Initial KS activation
    {
        let b = brain_arc.read().await;
        let cv = b.brain.capability.clone();
        let activated = ks_engine.select(TaskType::General, &cv);
        if !activated.is_empty() {
            println!("[daemon] Initial KS activation: {} sources selected", activated.len());
        }
    }

    // 6. Start background loop
    let bg_brain = brain_arc.clone();
    let _bg_handle = tokio::spawn(async move {
        let mut bg = BackgroundLoop::new(bg_brain);
        bg.start().await;
    });
    println!("[daemon] Background loop spawned");

    // 7. Daemon's own ticker loop: KS activation, validation, health, self-heal
    let mut ks_ticker = interval(Duration::from_secs(300));
    let mut val_ticker = interval(Duration::from_secs(900));
    let mut health_ticker = interval(Duration::from_secs(60));
    let mut heal_ticker = interval(Duration::from_secs(1800));

    write_health("daemon started");
    let mut cycle = 0u64;

    loop {
        tokio::select! {
            _ = ks_ticker.tick() => {
                cycle += 1;
                let b = brain_arc.read().await;
                let cv = b.brain.capability.clone();
                let activated = ks_engine.select(TaskType::General, &cv);
                let n = activated.len();
                drop(b);
                for ks in &activated {
                    ks_engine.record_hit(*ks);
                }
                if n > 0 {
                    println!("[daemon] KS #{}: {} active", cycle, n);
                }
            }

            _ = val_ticker.tick() => {
                let result = cargo_check_validation();
                match &result {
                    ValidationResult::Pass(_) => println!("[daemon] ✅ cargo check pass"),
                    ValidationResult::Fail(_, reason) => println!("[daemon] ❌ cargo check: {}", reason),
                    ValidationResult::Skipped => {}
                }
            }

            _ = heal_ticker.tick() => {
                println!("[daemon] 🔧 Self-healing cycle...");
                match heal_engine.heal_cycle() {
                    Ok(cycle_result) => {
                        let applied = cycle_result.applied_count;
                        let gaps = cycle_result.gap_count;
                        let fixes = cycle_result.fix_count;
                        if applied > 0 {
                            println!("[daemon] ✅ Healing: {} gaps → {} fixes, {} applied",
                                gaps, fixes, applied);
                            println!("[daemon] Verifying with cargo check...");
                            match heal_engine.verify() {
                                Ok(results) => {
                                    let ok = results.iter().filter(|r| r.1).count();
                                    let total = results.len();
                                    println!("[daemon] Verification: {}/{} modules OK", ok, total);
                                }
                                Err(e) => eprintln!("[daemon] Verification error: {}", e),
                            }
                        } else {
                            println!("[daemon] No new fixes needed ({} gaps, {} pending fixes)",
                                gaps, heal_engine.pending_fixes().len());
                        }
                        let health = format!("heal | gaps={} | fixes={} | applied={} | coherence={:.2}",
                            gaps, fixes, applied, cycle_result.coherence);
                        write_health(&health);
                    }
                    Err(e) => {
                        eprintln!("[daemon] ❌ Healing cycle failed: {}", e);
                        write_health(&format!("heal_error: {}", e));
                    }
                }
            }

            _ = health_ticker.tick() => {
                let b = brain_arc.read().await;
                let stats = b.brain.get_statistics();
                let mem = b.reasoning_bank.memories().len();
                let pending = heal_engine.pending_fixes().len();
                let health = format!(
                    "up | cycle={} | cap={:.3} | mem={} | absorbed={} | pending_fixes={}",
                    cycle, stats.capability_sum, mem, stats.total_absorbed, pending
                );
                drop(b);
                write_health(&health);
                if cycle % 10 == 0 {
                    println!("[daemon] 💓 {}", health);
                }
            }
        }
    }
}
