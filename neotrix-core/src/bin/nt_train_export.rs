//! NT-TRAIN-EXPORT — 从 cocoons.json 导出 MiniMind JSONL 训练数据
//!
//! 用法：
//! - 全量：`cargo run -p neotrix --bin nt-train-export`
//! - 增量：`cargo run -p neotrix --bin nt-train-export -- --incremental`
//!   （读水位文件，只导新记忆并 append，完事更新水位；cron 可直接调）
//! - 定点：`... -- --since-ts 1758000000 --append`
//! - 入库：`... -- --ingest models/training/repo_cards.jsonl`
//!   （crawl_queue 格式 {url,title,content,domain} → 记忆灌入 cocoons，加法合并）
//! - 熔炼：`cargo run -p neotrix --bin nt-train-export -- --refine`
//!   （knowledge.db experience → 晶体 episodes → evolve → crystal.json + cocoons 落盘；
//!   opencode 蒸馏行入库后跑此模式即完成“蒸馏到晶体核心”）
//!
//! 产出到 `models/training/`：
//! - pretrain.jsonl（全部记忆 → {"text": ...}）
//! - sft.jsonl（推理链 → 对话格式，需 archive_train 后才有内容）
//! - think.jsonl（推理链 → <think> 痕迹，需 archive_train 后才有内容）
//! - dpo.jsonl（成功 vs 失败配对，需 experience 数据）

use neotrix::neotrix::nt_crystal_core::cocoons::CocoonStore;
use neotrix::neotrix::nt_crystal_core::consciousness::CrystalConsciousness;
use neotrix::neotrix::nt_crystal_core::{AwakenBudget, CrystalCore, CrystalEngine, NtTrainExport};
use std::path::PathBuf;

/// 缺陷 #8 修复：增量水位文件（上次导出的最大 created_at）
const STATE_FILE: &str = ".export_state";

fn read_state(dir: &std::path::Path) -> u64 {
    std::fs::read_to_string(dir.join(STATE_FILE))
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

fn write_state(dir: &std::path::Path, ts: u64) {
    let _ = std::fs::write(dir.join(STATE_FILE), ts.to_string());
}

/// 入库：crawl_queue 格式 JSONL → 新记忆 → 加法合并进 cocoons。
/// 每行 {url, title, content, domain}；content 为空则用 title 兜底；
/// 去重靠 sync（同 id 跳过）+ 内容哈希（同 content 不同源跳过）。
fn ingest_file(path: &std::path::Path) {
    use neotrix::neotrix::nt_crystal_core::consciousness::MemoryType;
    use std::collections::HashSet;

    let data = std::fs::read_to_string(path).expect("read ingest jsonl");
    // 现有内容指纹（防重复入库）
    let store = CocoonStore::load();
    let mut seen: HashSet<String> = HashSet::new();
    for cocoon in store.cocoons.values() {
        for m in &cocoon.memories {
            seen.insert(content_hash(&m.content));
        }
    }
    eprintln!("[ingest] existing fingerprints: {}", seen.len());

    let mut consciousness = CrystalConsciousness::new("SmeltIngest");
    let mut n_new = 0usize;
    let mut n_dup = 0usize;
    for line in data.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let v: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let title = v.get("title").and_then(|x| x.as_str()).unwrap_or("").trim();
        let content = v.get("content").and_then(|x| x.as_str()).unwrap_or("").trim();
        let domain = v.get("domain").and_then(|x| x.as_str()).unwrap_or("smelt").trim();
        let url = v.get("url").and_then(|x| x.as_str()).unwrap_or("");
        let body = if content.is_empty() { title } else { content };
        if body.is_empty() {
            continue;
        }
        let tagged = if url.is_empty() {
            body.to_string()
        } else {
            format!("[ingest:{url}] {body}")
        };
        if !seen.insert(content_hash(&tagged)) {
            n_dup += 1;
            continue;
        }
        consciousness.remember(tagged, MemoryType::Fact, domain, 0.6);
        n_new += 1;
    }
    eprintln!("[ingest] new={n_new} dup-skipped={n_dup}");

    if n_new > 0 {
        let mut store = CocoonStore::load();
        store.sync_from_consciousness(&consciousness);
        store.save().expect("save cocoons");
        eprintln!("[ingest] cocoons merged + saved");
    }
}

/// 内容指纹（与 store 比对用；稳定哈希，非密码学）
fn content_hash(s: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    format!("{:016x}", h.finish())
}

/// 熔炼：knowledge.db → 晶体核心全链路（觉醒→镜像→进化→巩固→双落盘）。
/// 经验预算提到 20000，确保蒸馏新行（rowid 尾部）能被 mirror 到；
/// mirror 按 (title, action) 去重，可重复跑。
fn refine_crystal() {
    use neotrix::neotrix::nt_crystal_core::consciousness::CrystalConsciousness;
    let db_path = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".neotrix")
        .join("knowledge.db");
    let mut core = CrystalCore::load().unwrap_or_else(|_| CrystalCore::new("Refine"));
    let before = core.experience.stats().total_episodes;
    let mut consciousness = CrystalConsciousness::new("Refine");
    let budget = AwakenBudget {
        experience: 20000,
        verbose: true,
        ..AwakenBudget::default()
    };
    match CrystalEngine::full_refine(&mut core, &mut consciousness, &db_path, &budget) {
        Ok(rep) => {
            let after = core.experience.stats().total_episodes;
            eprintln!("[refine] report: {rep:?}");
            eprintln!("[refine] episodes before={before} after={after} (mirrored this run={})", rep.mirrored);
        }
        Err(e) => {
            eprintln!("[refine] FAILED: {e}");
            std::process::exit(1);
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    // 熔炼模式优先：蒸馏行入库后，一键炼入晶体核心（不碰导出逻辑）
    if args.iter().any(|a| a == "--refine") {
        refine_crystal();
        return;
    }
    // 入库模式优先：不碰导出逻辑
    if let Some(pos) = args.iter().position(|a| a == "--ingest") {
        let file = args.get(pos + 1).cloned().unwrap_or_default();
        if file.is_empty() {
            eprintln!("[ingest] usage: --ingest <jsonl>");
            std::process::exit(2);
        }
        ingest_file(&std::path::PathBuf::from(file));
        return;
    }
    let incremental = args.iter().any(|a| a == "--incremental");
    let append = incremental || args.iter().any(|a| a == "--append");
    let since_ts = args
        .windows(2)
        .find(|w| w[0] == "--since-ts")
        .and_then(|w| w[1].parse().ok());

    let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("models")
        .join("training");
    std::fs::create_dir_all(&out_dir).expect("create training dir");

    // 1. 加载 cocoons → consciousness
    eprintln!("[export] loading cocoons...");
    let cocoons = CocoonStore::load();
    let stats = cocoons.stats();
    eprintln!(
        "[export] {} domains, {} memories",
        stats.cocoon_count, stats.total_memories
    );

    let mut consciousness = CrystalConsciousness::new("Export");
    cocoons.sync_to_consciousness(&mut consciousness);
    eprintln!("[export] consciousness: {} memories", consciousness.memories.len());

    // 2. 加载 crystal core（经验数据）
    let core = CrystalCore::load().unwrap_or_else(|_| CrystalCore::new("Export"));

    // 3. 水位：--incremental 读 state；--since-ts 定点；缺省全量
    let since = if incremental {
        read_state(&out_dir)
    } else {
        since_ts.unwrap_or(0)
    };
    eprintln!("[export] mode: {} since_ts={since}", if append { "append" } else { "overwrite" });

    let emit = |path: PathBuf, lines: &[String], label: &str| {
        let n = if append {
            NtTrainExport::append_jsonl(&path, lines).expect("append jsonl")
        } else {
            NtTrainExport::write_jsonl(&path, lines).expect("write jsonl")
        };
        eprintln!("[export] {label}: {n} lines");
        n
    };

    // 4. 导出（增量按时间戳过滤；dpo 无时间戳，全量语义保持 overwrite/append 原样）
    eprintln!("[export] generating pretrain lines...");
    let pretrain = if since > 0 {
        NtTrainExport::pretrain_lines_since(&consciousness, since)
    } else {
        NtTrainExport::pretrain_lines(&consciousness)
    };
    emit(out_dir.join("pretrain.jsonl"), &pretrain, "pretrain.jsonl");

    eprintln!("[export] generating sft lines...");
    let sft = if since > 0 {
        NtTrainExport::sft_lines_since(&consciousness, since)
    } else {
        NtTrainExport::sft_lines(&consciousness)
    };
    emit(out_dir.join("sft.jsonl"), &sft, "sft.jsonl");

    eprintln!("[export] generating think lines...");
    let think = if since > 0 {
        NtTrainExport::think_lines_since(&consciousness, since)
    } else {
        NtTrainExport::think_lines(&consciousness)
    };
    emit(out_dir.join("think.jsonl"), &think, "think.jsonl");

    eprintln!("[export] generating dpo lines...");
    let dpo = NtTrainExport::dpo_lines(&core);
    emit(out_dir.join("dpo.jsonl"), &dpo, "dpo.jsonl");

    // 5. 增量水位前移
    if incremental {
        let high = NtTrainExport::max_created_at(&consciousness);
        write_state(&out_dir, high);
        eprintln!("[export] state watermark -> {high}");
    }

    eprintln!("[export] done. files in {}", out_dir.display());
}
