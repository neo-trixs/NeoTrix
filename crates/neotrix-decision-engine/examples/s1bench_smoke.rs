//! S1Bench-style smoke baseline for the local LayaEngine (ModernBERT port).
//!
//! Loads the hand-labelled golden pack (`tests/data/s1_smoke.jsonl`), runs each
//! item through either the real local model (`LayaEngine::try_new`) or — when
//! model files are absent/unloadable — the heuristic `SimulationBackend`
//! (`LayaEngine::simulate_answers`), and reports accuracy + ECE.
//!
//! Run: `cargo run -p neotrix-decision-engine --example s1bench_smoke [model_dir]`
//!
//! Report-only: exit code is always 0 on a completed run (never gates on
//! accuracy). Non-zero exit means the harness itself failed (pack unreadable,
//! etc.).
//!
//! ECE reference (equal-mass bins, `B` bins, `n` items):
//!   ECE = sum_b (|B_b| / n) * |acc(B_b) - conf(B_b)|
//!
//! Hand-verified toy cases (kept so a reader can re-check the math by hand):
//! - Toy 1: pairs (conf, correct) = (0.9,1),(0.8,1),(0.6,0),(0.4,0),(0.2,0),
//!   5 bins -> one item per bin -> |acc-conf| = 0.1,0.2,0.4,0.4,0.2 ->
//!   ECE = (0.1+0.2+0.4+0.4+0.2)/5 = 0.26.
//! - Toy 2: five (1.0,1) + five (0.0,0), 5 equal-mass bins of 2 ->
//!   sorted confs 0,0,0,0,0,1,1,1,1,1 -> bins [0,0],[0,0],[0,1],[1,1],[1,1] ->
//!   middle bin acc=0.5 conf=0.5 -> 0, all others 0 -> ECE = 0.0
//!   (perfectly calibrated at the extremes; the straddling bin is exact).

use neotrix_decision_engine::{Answer, LayaEngine, Question, State};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

/// One golden-pack row (JSONL). Exactly one of `gold_bool` / `gold_choice` is
/// set, matching `kind`.
#[derive(Debug, serde::Deserialize)]
struct GoldenItem {
    id: String,
    kind: String,
    state: String,
    question: String,
    options: Option<Vec<String>>,
    gold_bool: Option<bool>,
    gold_choice: Option<String>,
}

/// Per-item outcome fed into accuracy / ECE.
struct ItemResult {
    id: String,
    kind: String,
    gold: String,
    pred: String,
    confidence: f64,
    correct: bool,
    latency_ms: f64,
    note: String,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("s1bench_smoke harness error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let default_model_dir = manifest
        .join("../../models/modernbert-large")
        .canonicalize()
        .unwrap_or_else(|_| manifest.join("../../models/modernbert-large"));
    let model_dir: PathBuf = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or(default_model_dir);
    let pack_path = manifest.join("tests/data/s1_smoke.jsonl");

    let pack = load_pack(&pack_path)?;

    // Backend selection: real local inference when the weights load,
    // heuristic simulation otherwise. The label below is the source of truth
    // for which path actually ran.
    let engine = LayaEngine::try_new(&model_dir);
    let backend_label = match &engine {
        Some(_) => format!("laya-local (model_dir={})", model_dir.display()),
        None => {
            "simulation (agent_simulation heuristics via LayaEngine::simulate_answers)".to_string()
        }
    };

    println!("pack:    {} ({} items)", pack_path.display(), pack.len());
    println!("backend: {backend_label}");

    let mut results: Vec<ItemResult> = Vec::with_capacity(pack.len());
    for item in &pack {
        results.push(eval_item(item, engine.as_ref()));
    }

    println!();
    println!(
        "{:<6} {:<6} {:<10} {:<10} {:<6} {:<7} {:>9}  note",
        "id", "kind", "gold", "pred", "correct", "conf", "ms"
    );
    for r in &results {
        println!(
            "{:<6} {:<6} {:<10} {:<10} {:<6}  {:<7.3} {:>9.1}  {}",
            r.id,
            r.kind,
            truncate(&r.gold, 10),
            truncate(&r.pred, 10),
            if r.correct { "Y" } else { "N" },
            r.confidence,
            r.latency_ms,
            r.note,
        );
    }

    let n = results.len() as f64;
    let n_correct = results.iter().filter(|r| r.correct).count();
    let accuracy = if n > 0.0 { n_correct as f64 / n } else { 0.0 };
    let pairs: Vec<(f64, bool)> = results.iter().map(|r| (r.confidence, r.correct)).collect();
    let ece = ece_equal_mass(&pairs, 5);
    let mean_ms = if n > 0.0 {
        results.iter().map(|r| r.latency_ms).sum::<f64>() / n
    } else {
        0.0
    };

    println!();
    println!("summary: backend={backend_label}");
    println!(
        "summary: n={} correct={n_correct} accuracy={accuracy:.4}",
        results.len()
    );
    println!("summary: ECE(5 equal-mass bins)={ece:.4}");
    println!("summary: mean_latency_ms={mean_ms:.1}");
    if let Some(e) = engine.as_ref() {
        let errs = e.drain_errors();
        println!("summary: engine_per_question_errors={}", errs.len());
        for (qid, msg) in errs.iter().take(10) {
            println!("  error: {qid}: {msg}");
        }
    }
    println!("note: smoke only — no accuracy gate (temperature-fitting signal, not a pass/fail).");
    Ok(())
}

fn load_pack(path: &std::path::Path) -> Result<Vec<GoldenItem>, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read pack {}: {e}", path.display()))?;
    let mut items = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let item: GoldenItem =
            serde_json::from_str(line).map_err(|e| format!("pack line {}: {e}", i + 1))?;
        items.push(item);
    }
    if items.is_empty() {
        return Err(format!("pack {} has no items", path.display()));
    }
    Ok(items)
}

fn eval_item(item: &GoldenItem, engine: Option<&LayaEngine>) -> ItemResult {
    let fail = |note: String| ItemResult {
        id: item.id.clone(),
        kind: item.kind.clone(),
        gold: gold_str(item),
        pred: "-".to_string(),
        confidence: 0.0,
        correct: false,
        latency_ms: 0.0,
        note,
    };

    let question = match build_question(item) {
        Ok(q) => q,
        Err(e) => return fail(format!("build_question: {e}")),
    };
    let state: State = item.state.as_str().into();

    let t0 = Instant::now();
    let answer: Option<Answer> = match engine {
        Some(e) => match e.evaluate(&state, std::slice::from_ref(&question)) {
            Ok(res) => res.answers.get(&item.id).cloned(),
            Err(e) => return fail(format!("evaluate: {e}")),
        },
        None => LayaEngine::simulate_answers(std::slice::from_ref(&question))
            .get(&item.id)
            .cloned(),
    };
    let latency_ms = t0.elapsed().as_secs_f64() * 1000.0;

    let answer = match answer {
        Some(a) => a,
        None => return fail("missing answer for question id".to_string()),
    };

    // Confidence extraction (correctness proxy for ECE):
    // - Noul: max(p, 1-p) where p = P(true).
    // - Choice: max over the probability distribution (P(selected)). The
    //   `confidence` field is distribution concentration, NOT a correctness
    //   probability, so it must not feed ECE directly; it is only the
    //   fallback when the distribution is empty (simulation backend).
    let (pred, confidence, correct, note) = match &answer {
        Answer::Noul(n) => {
            let p = n.noul.clamp(0.0, 1.0);
            let pred_bool = p >= 0.5;
            let gold = item.gold_bool.unwrap_or(false);
            (
                pred_bool.to_string(),
                p.max(1.0 - p),
                pred_bool == gold,
                String::new(),
            )
        }
        Answer::Choice(c) => {
            let conf = if c.probabilities.is_empty() {
                c.confidence.clamp(0.0, 1.0)
            } else {
                c.probabilities
                    .values()
                    .cloned()
                    .fold(f64::NEG_INFINITY, f64::max)
                    .clamp(0.0, 1.0)
            };
            let gold = item.gold_choice.clone().unwrap_or_default();
            let ok = c.choice == gold;
            let note = if c.probabilities.is_empty() {
                "sim: empty distribution, concentration fallback".to_string()
            } else {
                String::new()
            };
            (c.choice.clone(), conf, ok, note)
        }
        Answer::Score(s) => (
            format!("{:.2}", s.score),
            0.0,
            false,
            "unexpected Score answer for smoke pack".to_string(),
        ),
    };

    ItemResult {
        id: item.id.clone(),
        kind: item.kind.clone(),
        gold: gold_str(item),
        pred,
        confidence,
        correct,
        latency_ms,
        note,
    }
}

fn build_question(item: &GoldenItem) -> Result<Question, String> {
    match item.kind.as_str() {
        "noul" => Ok(Question::noul(item.id.clone(), item.question.clone())),
        "choice" => {
            let opts = item.options.clone().unwrap_or_default();
            if opts.is_empty() {
                return Err("choice item needs non-empty options".to_string());
            }
            let map: HashMap<String, Option<String>> =
                opts.into_iter().map(|o| (o, None)).collect();
            Question::choice(item.id.clone(), item.question.clone(), map).map_err(|e| e.to_string())
        }
        other => Err(format!("unknown kind '{other}'")),
    }
}

fn gold_str(item: &GoldenItem) -> String {
    match item.kind.as_str() {
        "noul" => item.gold_bool.map(|b| b.to_string()).unwrap_or_default(),
        _ => item.gold_choice.clone().unwrap_or_default(),
    }
}

/// Equal-mass ECE: sort by confidence, split into `bins` groups as evenly as
/// possible (first `n % bins` groups take one extra), then
/// sum_b (|B_b|/n) * |acc - conf|.
fn ece_equal_mass(pairs: &[(f64, bool)], bins: usize) -> f64 {
    let n = pairs.len();
    if n == 0 || bins == 0 {
        return 0.0;
    }
    let mut sorted: Vec<(f64, bool)> = pairs.to_vec();
    sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let b = bins.min(n);
    let base = n / b;
    let rem = n % b;
    let mut ece = 0.0;
    let mut start = 0;
    for i in 0..b {
        let size = base + if i < rem { 1 } else { 0 };
        let slice = &sorted[start..start + size];
        start += size;
        let acc = slice.iter().filter(|(_, c)| *c).count() as f64 / slice.len() as f64;
        let conf = slice.iter().map(|(c, _)| c).sum::<f64>() / slice.len() as f64;
        ece += (slice.len() as f64 / n as f64) * (acc - conf).abs();
    }
    ece
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}
