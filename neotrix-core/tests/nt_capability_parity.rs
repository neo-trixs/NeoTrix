// Behavioural parity harness — our implementations vs reference products.
//
// What this replaces: a hand-written test per capability. The first target
// (ResponseCache) had its own test file; adding a second target would have meant
// a second hand-written file, i.e. the "reproduce their product" loop would not
// scale. So the ours-side is now **data-driven**: a vectors file declares the
// capability, and a registry arm below drives the real implementation.
//
// Files (all in-repo, no network at test time):
//   .neotrix/parity/<name>.vectors.json           op script — single source of truth
//   .neotrix/parity/<name>.vectors.reference.json observations captured by
//                                                   scripts/ops/nt_parity_ref.py
//                                                   from the reference implementation
//   TARGETS (below)                               ours-side adapter per capability
//
// Discipline:
//   - Oracles are produced by **running the reference**, never hand-written, so a
//     matching test cannot prove itself right by agreeing with a wish.
//   - A missing reference file is a **hard failure**. A parity test that degrades
//     to "nothing to compare" is a vacuous green.
//   - Only the **common** op subset is compared. Our extensions (cache pinning,
//     hit/miss counters) are deliberately excluded: they diverge by construction,
//     and that is a feature, not a defect.
//   - A FAIL here is not a broken test, it is a **finding**: our implementation
//     and the reference disagree on observable behaviour. The assertion message
//     prints every diverging step at once, not just the first.

use neotrix::l1_action::nt_io::nt_io_provider::gateway::resilience::ResponseCache;
use neotrix::l4_emotion::nt_memory::entity_linking::linker;
use std::path::PathBuf;

/// One op from a vectors file. `k`/`v` are the operands; for a stateful
/// capability `k` is the key and `v` the value, for a pure function they are
/// the two arguments.
#[derive(Debug)]
struct Op {
    op: String,
    k: String,
    v: String,
}

#[derive(Debug)]
struct Case {
    name: String,
    capacity: Option<usize>,
    ops: Vec<Op>,
}

/// Drive our real implementation. Returns one observation line per step, in the
/// exact shape the reference collector emits, or Err if the vectors ask for an
/// op this capability does not implement (never a silent "unsupported" line —
/// that would make both sides agree on garbage).
fn run_ours(capability: &str, cases: &[Case]) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for c in cases {
        out.push(c.name.clone());
        match capability {
            "lru_core" => {
                let cap = c
                    .capacity
                    .ok_or_else(|| format!("{}: case {} needs a capacity", capability, c.name))?;
                let mut cache = ResponseCache::new(cap);
                for step in &c.ops {
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
                        other => return Err(format!("unimplemented op {:?}", other)),
                    }
                }
            }
            "levenshtein" => {
                for step in &c.ops {
                    match step.op.as_str() {
                        "dist" => {
                            out.push(format!("dist:{}", linker::levenshtein(&step.k, &step.v)))
                        }
                        other => return Err(format!("unimplemented op {:?}", other)),
                    }
                }
            }
            other => return Err(format!("no ours-side adapter for capability {:?}", other)),
        }
    }
    Ok(out)
}

fn parity_dir() -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("neotrix-core has a parent (repo root)")
        .to_path_buf();
    root.join(".neotrix").join("parity")
}

fn read_json(path: &PathBuf) -> serde_json::Value {
    let raw = std::fs::read_to_string(path).unwrap_or_else(|e| {
        panic!(
            "cannot read {}: {}\n\
             A behavioural parity test with no reference oracle is a vacuous green.\n\
             Regenerate it (offline; uses the vendored copy in ~/.cargo/registry):\n  \
             python3 scripts/ops/nt_parity_ref.py --vectors .neotrix/parity/{}",
            path.display(),
            e,
            path.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default()
        )
    });
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("{}: {}", path.display(), e))
}

fn parse_cases(vectors: &serde_json::Value) -> Vec<Case> {
    vectors["cases"]
        .as_array()
        .expect("vectors cases is an array")
        .iter()
        .map(|c| Case {
            name: c["name"].as_str().expect("case name").to_string(),
            capacity: c.get("capacity").and_then(|v| v.as_u64()).map(|v| v as usize),
            ops: c["ops"]
                .as_array()
                .expect("case ops is an array")
                .iter()
                .map(|o| Op {
                    op: o["op"].as_str().expect("op name").to_string(),
                    k: o["k"].as_str().unwrap_or_default().to_string(),
                    v: o["v"].as_str().unwrap_or_default().to_string(),
                })
                .collect(),
        })
        .collect()
}

/// The shared body: run our side, compare against the captured reference, and
/// report **all** diverging steps.
fn assert_parity(stem: &str) {
    let vectors = read_json(&parity_dir().join(format!("{}.vectors.json", stem)));
    let reference = read_json(&parity_dir().join(format!("{}.vectors.reference.json", stem)));

    let capability = vectors["capability"]
        .as_str()
        .unwrap_or_else(|| panic!("{}: vectors has no `capability` field", stem))
        .to_string();
    assert_eq!(
        reference["schema"], "nt-parity/reference/1",
        "{}: unexpected reference schema — regenerate it", stem
    );
    assert_eq!(
        reference["capability"], capability.as_str(),
        "{}: reference was captured for a different capability — the vectors and \
         the oracle disagree about what is being compared", stem
    );

    let expected: Vec<String> = reference["observations"]
        .as_array()
        .expect("reference observations is an array")
        .iter()
        .map(|v| v.as_str().unwrap_or_default().to_string())
        .collect();

    let cases = parse_cases(&vectors);
    assert!(!cases.is_empty(), "{}: vectors has no cases", stem);
    let ours = run_ours(&capability, &cases)
        .unwrap_or_else(|e| panic!("{}: {}", stem, e));

    assert_eq!(
        ours.len(),
        expected.len(),
        "{}: observation count differs: ours {} vs reference {} — vectors and \
         reference are out of sync (regenerate the reference)",
        stem,
        ours.len(),
        expected.len()
    );

    // Map a flat observation index back to (case name, step within case) so the
    // failure message points at a case, not at a line number in a flattened list.
    let mut starts = Vec::new();
    let mut acc = 0usize;
    for c in &cases {
        starts.push((acc, c.name.clone()));
        acc += 1 + c.ops.len();
    }
    let label = |i: usize| -> String {
        for (start, name) in &starts {
            if i >= *start {
                return format!("{} step {}", name, i - start + 1);
            }
        }
        format!("step {}", i + 1)
    };

    let diffs: Vec<String> = ours
        .iter()
        .zip(expected.iter())
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(i, (a, b))| {
            format!("  [{}] ours={} reference={}", label(i), a, b)
        })
        .collect();

    assert!(
        diffs.is_empty(),
        "{}: behavioural divergence from the reference implementation ({} step(s)):\n{}\n\
         Each line names the case and the step within it. Read the vectors file to \
         see the operands. Our extensions (pinning, hit/miss counters) are not \
         compared by design.",
        stem,
        diffs.len(),
        diffs.join("\n")
    );
}

#[test]
fn lru_core_matches_reference() {
    assert_parity("response-cache");
}

#[test]
fn levenshtein_matches_reference() {
    assert_parity("levenshtein");
}

#[test]
fn contains_does_not_disturb_lru_order() {
    // The reason `ResponseCache::contains` is not `cache().is_some()`: a membership
    // probe must not promote, or "is it cached?" silently changes what gets evicted
    // next. Discriminator: with capacity 2, after `insert a; insert b`, `a` is the
    // least-recently-used — so if `contains("a")` promoted, the next insert would
    // evict `b`; if it does not, it evicts `a`. The surviving key is the measurement.
    //
    // (First draft asserted the opposite and failed: the implementation was right
    // and the hand-derivation was wrong. The reference oracle agrees — case c2
    // evicts `a`. Kept as a reminder that 手推 ≠ 实证.)
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
