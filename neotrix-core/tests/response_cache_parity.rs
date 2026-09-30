// Behavioural parity: our ResponseCache vs the reference LRU implementation.
//
// Why this file exists (DECOMPOSE-PARITY-2026-09-30 §4):
//   The atom-level `parity` matrix is **name-level** — it reported `get` as
//   "theirs only" purely because our equivalent method is called `cache`. A
//   name-level matrix cannot answer "can we reproduce their product?". This
//   test is the behavioural leg: the same op script, the same expected
//   observations, both sides executed for real.
//
// Inputs (both in-repo, no network at test time):
//   .neotrix/parity/response-cache.vectors.json          op script (source of truth)
//   .neotrix/parity/response-cache.vectors.reference.json observations captured
//                                                            from lru@0.18.5 (MIT)
//                                                            by scripts/ops/nt_parity_ref.py
//
// Discipline:
//   - The reference file is produced by **running the other implementation**, not
//     by hand ⇒ this test cannot prove itself right by agreeing with a wish.
//   - If the reference file is missing this test **fails**; it never degrades to
//     a vacuous green (the failure mode this repo keeps re-learning).
//   - Only the **common** op subset is compared (put/get/len/contains). Pinning
//     and hit/miss counters are our extensions and are deliberately not compared:
//     they would diverge by construction, and that is a feature, not a defect.

use neotrix::l1_action::nt_io::nt_io_provider::gateway::resilience::ResponseCache;
use std::path::PathBuf;

fn parity_file(name: &str) -> PathBuf {
    // CARGO_MANIFEST_DIR = <repo>/neotrix-core
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("neotrix-core has a parent (repo root)")
        .to_path_buf();
    root.join(".neotrix").join("parity").join(name)
}

fn read_json(path: &PathBuf) -> serde_json::Value {
    let raw = std::fs::read_to_string(path).unwrap_or_else(|e| {
        panic!(
            "cannot read {}: {}\n\
             A behavioural parity test with no reference oracle is a vacuous green.\n\
             Regenerate it (offline; uses the vendored copy in ~/.cargo/registry):\n  \
             python3 scripts/ops/nt_parity_ref.py --vectors {}",
            path.display(),
            e,
            path.file_name()
                .map(|f| format!(".neotrix/parity/{}", f.to_string_lossy()))
                .unwrap_or_default()
        )
    });
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("{}: {}", path.display(), e))
}

#[derive(Debug)]
struct Op {
    op: String,
    k: String,
    v: String,
}

fn observe_ours(capacity: usize, ops: &[Op]) -> Vec<String> {
    let mut cache = ResponseCache::new(capacity);
    let mut out = Vec::new();
    for step in ops {
        match step.op.as_str() {
            "put" => {
                cache.insert(&step.k, step.v.clone());
                out.push("put:none".to_string());
            }
            "get" => match cache.cache(&step.k) {
                Some(v) => out.push(format!("get:Some({})", v)),
                None => out.push("get:None".to_string()),
            },
            "len" => out.push(format!("len:{}", cache.len())),
            "contains" => out.push(format!("contains:{}", cache.contains(&step.k))),
            other => out.push(format!("unsupported:{}", other)),
        }
    }
    out
}

#[test]
fn response_cache_behaviour_matches_reference_lru() {
    let vectors = read_json(&parity_file("response-cache.vectors.json"));
    let reference = read_json(&parity_file("response-cache.vectors.reference.json"));

    assert_eq!(
        reference["schema"], "nt-parity/reference/1",
        "unexpected reference schema — regenerate it"
    );

    let expected: Vec<String> = reference["observations"]
        .as_array()
        .expect("reference observations is an array")
        .iter()
        .map(|v| v.as_str().unwrap_or_default().to_string())
        .collect();

    let cases = vectors["cases"]
        .as_array()
        .expect("vectors cases is an array")
        .clone();
    assert!(!cases.is_empty(), "vectors has no cases");

    let mut ours_all: Vec<String> = Vec::new();
    for case in &cases {
        let name = case["name"].as_str().expect("case name").to_string();
        let capacity = case["capacity"].as_u64().expect("case capacity") as usize;
        let ops: Vec<Op> = case["ops"]
            .as_array()
            .expect("case ops is an array")
            .iter()
            .map(|o| Op {
                op: o["op"].as_str().expect("op name").to_string(),
                k: o["k"].as_str().unwrap_or_default().to_string(),
                v: o["v"].as_str().unwrap_or_default().to_string(),
            })
            .collect();
        ours_all.push(name);
        ours_all.extend(observe_ours(capacity, &ops));
    }

    assert_eq!(
        ours_all.len(),
        expected.len(),
        "observation count differs: ours {} vs reference {} — the vectors and the \
         reference are out of sync (regenerate the reference)",
        ours_all.len(),
        expected.len()
    );

    // Diff all mismatches before asserting, so one run reports every divergence
    // instead of only the first.
    let diffs: Vec<String> = ours_all
        .iter()
        .zip(expected.iter())
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(i, (a, b))| format!("  step {}: ours={} reference={}", i + 1, a, b))
        .collect();
    assert!(
        diffs.is_empty(),
        "behavioural divergence from the reference LRU ({} step(s)):\n{}\n\
         The step number maps to the vectors file (1 = first case name). \
         Our extras (pinning, hit/miss counters) are not compared by design.",
        diffs.len(),
        diffs.join("\n")
    );
}

#[test]
fn contains_does_not_disturb_lru_order() {
    // The reason `contains` exists at all: a membership probe must not promote.
    //
    // Discriminator: with capacity 2, after `insert a; insert b`, `a` is the
    // least-recently-used. If `contains("a")` promoted, the next insert would
    // evict `b`; if it does not, it evicts `a`. So the surviving key *is* the
    // measurement.
    //
    // (First draft of this test asserted the opposite and failed. The
    // implementation was right and the hand-derivation was wrong: with
    // insertion order a,b the LRU is `a`, not `b`. The reference-oracle test
    // above independently agrees — c2 evicts `a`. Kept as a reminder that
    // 手推 ≠ 实证.)
    let mut cache = ResponseCache::new(2);
    cache.insert("a", "1".to_string());
    cache.insert("b", "2".to_string());
    assert!(cache.contains("a"), "precondition: 'a' is cached");
    cache.insert("c", "3".to_string());
    assert!(
        !cache.contains("a") && cache.contains("b") && cache.contains("c"),
        "contains() promoted 'a': expected the non-promoting probe to leave 'a' \
         as the eviction victim"
    );
}
