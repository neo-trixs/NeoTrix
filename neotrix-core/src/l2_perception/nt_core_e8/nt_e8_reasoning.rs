//! Mythos / structured-reasoning parsing and E8 hexagram estimation.
//!
//! Moved out of `nt_core_e8/mod.rs` (facade slimming, behaviour-neutral).

use super::FUNCTION_TAG_BLOCKS;

/// Parse 9-stage Mythos structured reasoning text.
/// Returns [acknowledgment, restatement, decomposition, first_principles,
///          self_verification, alternative, deep_dive, synthesis, conclusion]
/// — empty strings for missing stages.
pub fn parse_mythos_reasoning(text: &str) -> [String; 9] {
    let markers: [(&[&str], usize); 9] = [
        (
            &[
                "acknowledg",
                "**acknowledg**",
                "## acknowledg",
                "scope:",
                "frame:",
            ],
            0,
        ),
        (
            &[
                "restatement",
                "**restatement**",
                "## restate",
                "rephrase:",
                "in other words",
            ],
            1,
        ),
        (
            &[
                "decompos",
                "**decompos**",
                "## decompos",
                "break down",
                "sub-problem",
                "subproblem",
            ],
            2,
        ),
        (
            &[
                "first-principle",
                "first principle",
                "**first-principle**",
                "## first-principle",
                "fundamental",
                "root cause",
            ],
            3,
        ),
        (
            &[
                "self-verif",
                "self verif",
                "**self-verif**",
                "## self-verif",
                "check my",
                "double-check",
            ],
            4,
        ),
        (
            &[
                "alternat",
                "**alternat**",
                "## alternat",
                "other approach",
                "different way",
            ],
            5,
        ),
        (
            &[
                "deep dive",
                "**deep dive**",
                "## deep dive",
                "detailed",
                "deep analysis",
            ],
            6,
        ),
        (
            &[
                "synthesis",
                "**synthesis**",
                "## synthesis",
                "integrat",
                "combine",
                "pull together",
            ],
            7,
        ),
        (
            &[
                "conclusion",
                "**conclusion**",
                "## conclusion",
                "summary:",
                "final:",
                "answer:",
            ],
            8,
        ),
    ];

    let mut stages = [
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
    ];
    let mut current = 9usize;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let low = trimmed.to_lowercase();

        let mut matched = false;
        for (kw_set, idx) in &markers {
            if kw_set.iter().any(|m| low.starts_with(m)) {
                current = *idx;
                if let Some(content) = trimmed.split_once(':').map(|(_, rest)| rest.trim()) {
                    if !content.is_empty() && content.len() > 1 {
                        stages[*idx].push_str(content);
                    }
                }
                matched = true;
                break;
            }
        }
        if matched {
            continue;
        }

        if current < 9 {
            if stages[current].is_empty() {
                stages[current].push_str(trimmed);
            } else {
                stages[current].push_str(&format!(" {}", trimmed));
            }
        }
    }

    for s in &mut stages {
        *s = s.trim().to_string();
    }
    stages
}

/// Legacy: parse 4-stage structured reasoning text.
pub fn parse_structured_reasoning(text: &str) -> [String; 4] {
    let goal_markers = ["goal:", "**goal:**", "## goal", "objective:", "aim:"];
    let reason_markers = ["reason:", "**reason:**", "## reason", "context:", "why:"];
    let boundary_markers = [
        "boundar",
        "**boundar",
        "## boundar",
        "constraint:",
        "don't:",
        "avoid:",
    ];
    let verify_markers = [
        "verif:",
        "**verif:",
        "## verif",
        "verification:",
        "check:",
        "validation:",
        "test:",
    ];

    let mut stages = [String::new(), String::new(), String::new(), String::new()];
    let mut current = 4usize;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let low = trimmed.to_lowercase();

        if goal_markers.iter().any(|m| low.starts_with(m)) {
            current = 0;
            if let Some(content) = trimmed.split(':').nth(1).map(|s| s.trim()) {
                if !content.is_empty() {
                    stages[0].push_str(content);
                }
            }
            continue;
        }
        if reason_markers.iter().any(|m| low.starts_with(m)) {
            current = 1;
            if let Some(content) = trimmed.split(':').nth(1).map(|s| s.trim()) {
                if !content.is_empty() {
                    stages[1].push_str(content);
                }
            }
            continue;
        }
        if boundary_markers.iter().any(|m| low.starts_with(m)) {
            current = 2;
            if let Some(content) = trimmed.split(':').nth(1).map(|s| s.trim()) {
                if !content.is_empty() {
                    stages[2].push_str(content);
                }
            }
            continue;
        }
        if verify_markers.iter().any(|m| low.starts_with(m)) {
            current = 3;
            if let Some(content) = trimmed.split(':').nth(1).map(|s| s.trim()) {
                if !content.is_empty() {
                    stages[3].push_str(content);
                }
            }
            continue;
        }

        if current < 4 {
            stages[current].push_str(&format!(" {}", trimmed));
        }
    }

    for s in &mut stages {
        *s = s.trim().to_string();
    }
    stages
}

/// Map 9-stage Mythos reasoning to E8 hexagram sequence.
/// Each stage selects within its designated 8-entry block by keyword overlap.
pub fn estimate_e8_from_mythos(stages: &[String; 9]) -> Vec<u8> {
    const MYTHOS_KEYWORDS: [[&[&str]; 8]; 9] = [
        // Block 56-63 (Acknowledgment): scope/intent
        [
            &["system", "overview"],
            &["scope", "range"],
            &["intent", "purpose"],
            &["boundary", "limit"],
            &["stakeholder", "actor"],
            &["context", "background"],
            &["problem", "issue"],
            &["question", "query"],
        ],
        // Block 48-55 (Restatement): rephrase
        [
            &["rephrase", "restate"],
            &["simplify", "condense"],
            &["clarify", "elaborate"],
            &["paraphrase", "rewrite"],
            &["translate", "convert"],
            &["extract", "distill"],
            &["identify", "highlight"],
            &["define", "specify"],
        ],
        // Block 40-47 (Decomposition): sub-problems
        [
            &["decompose", "break"],
            &["sub-problem", "subproblem"],
            &["component", "module"],
            &["dependency", "depends"],
            &["hierarchy", "nested"],
            &["layer", "tier"],
            &["step", "phase"],
            &["separate", "isolate"],
        ],
        // Block 32-39 (First-Principles): fundamental
        [
            &["fundamental", "first"],
            &["principle", "axiom", "law"],
            &["reason", "cause"],
            &["root", "origin"],
            &["essence", "core"],
            &["invariant", "immutable"],
            &["derive", "deduce"],
            &["assumption", "premise"],
        ],
        // Block 24-31 (Self-Verification): check
        [
            &["verify", "check"],
            &["confirm", "ensure"],
            &["validate", "prove"],
            &["review", "audit"],
            &["test", "assert"],
            &["trace", "walk through"],
            &["gap", "missing"],
            &["consist", "coherent"],
        ],
        // Block 16-23 (Alternative): alternate
        [
            &["alternative", "alternate"],
            &["option", "choice"],
            &["trade-off", "tradeoff"],
            &["compare", "contrast"],
            &["pros", "cons"],
            &["scenario", "what if"],
            &["fallback", "backup"],
            &["plan b", "second"],
        ],
        // Block 8-15 (Deep Dive): detailed
        [
            &["detail", "depth"],
            &["analyze", "examine"],
            &["explore", "investigate"],
            &["compute", "calculate"],
            &["evidence", "data"],
            &["example", "instance"],
            &["mechanism", "how"],
            &["trace", "timeline"],
        ],
        // Block 0-7 (Synthesis): integrate
        [
            &["synthesize", "integrate"],
            &["combine", "merge"],
            &["summary", "overview"],
            &["pattern", "theme"],
            &["insight", "finding"],
            &["conclude", "infer"],
            &["framework", "model"],
            &["map", "bridge"],
        ],
        // Block 4-11 (Conclusion): final answer (uses blocks 4-11, accessible via offset wrapping)
        [
            &["answer", "result"],
            &["final", "conclusion"],
            &["recommend", "suggest"],
            &["next step", "follow up"],
            &["action", "decision"],
            &["outcome", "output"],
            &["implication", "impact"],
            &["summary", "tl;dr"],
        ],
    ];

    let blocks: [usize; 9] = [56, 48, 40, 32, 24, 16, 8, 0, 4];
    let mut seq = Vec::with_capacity(9);

    for (stage_idx, text) in stages.iter().enumerate() {
        let lower = text.to_lowercase();
        let base = blocks[stage_idx];
        let keywords = &MYTHOS_KEYWORDS[stage_idx];

        let mut best_offset = 0usize;
        let mut best_score = 0usize;
        for (offset, kw_set) in keywords.iter().enumerate() {
            let score = kw_set.iter().filter(|kw| lower.contains(*kw)).count();
            if score > best_score {
                best_score = score;
                best_offset = offset;
            }
        }

        let e8_val = if stage_idx == 8 {
            // Conclusion wraps within block 4-11
            (4 + best_offset % 8) as u8
        } else {
            (base + best_offset) as u8
        };
        seq.push(e8_val.min(63));
    }

    seq
}

/// Map sentence-level function tags to E8 hexagram sequence.
/// Takes a slice of tag strings (e.g. &["problem_setup", "plan_generation"])
/// and returns hexagrams [block, block+1, ...] for each tag in order.
pub fn function_tags_to_e8(tags: &[&str]) -> Vec<u8> {
    let mut seq = Vec::with_capacity(tags.len());
    for &tag in tags {
        let mut found = false;
        for &(known, block) in &FUNCTION_TAG_BLOCKS {
            if tag == known {
                let offset = if tag.contains("generation") || tag.contains("emission") {
                    1
                } else {
                    0
                };
                seq.push((block as u8 + offset).min(63));
                found = true;
                break;
            }
        }
        if !found {
            seq.push(32); // default: Syntax Check
        }
    }
    seq
}

/// Map 4-stage structured reasoning to E8 hexagram sequence (legacy).
pub fn estimate_e8_from_structured(stages: &[String; 4]) -> Vec<u8> {
    const E8_KEYWORDS: [[&[&str]; 8]; 4] = [
        [
            &["system design", "architecture", "overview"],
            &["reduce", "eliminate", "fix"],
            &["create", "build", "implement"],
            &["design", "plan", "blueprint"],
            &["optimize", "improve", "enhance"],
            &["understand", "analyze", "investigate"],
            &["integrate", "connect", "bridge"],
            &["migrate", "upgrade", "transform"],
        ],
        [
            &["pattern", "template", "recurring"],
            &["generate", "create", "produce"],
            &["analyze", "examine", "study"],
            &["reason", "why", "because"],
            &["compare", "contrast", "evaluate"],
            &["principle", "fundamental", "theory"],
            &["context", "background", "motivation"],
            &["trade-off", "balance", "cost"],
        ],
        [
            &["constraint", "limit", "boundary"],
            &["avoid", "don't", "never"],
            &["assume", "presume", "given"],
            &["scope", "range", "coverage"],
            &["safety", "secure", "protect"],
            &["privacy", "confidential", "permission"],
            &["resource", "memory", "time"],
            &["compatible", "support", "platform"],
        ],
        [
            &["test", "verify", "check"],
            &["validate", "confirm", "ensure"],
            &["benchmark", "measure", "metric"],
            &["review", "audit", "inspect"],
            &["coverage", "complete", "exhaustive"],
            &["correct", "accuracy", "precision"],
            &["monitor", "observe", "track"],
            &["regression", "stability", "consistent"],
        ],
    ];

    let blocks: [usize; 4] = [56, 48, 24, 40];
    let mut seq = Vec::with_capacity(4);

    for (stage_idx, text) in stages.iter().enumerate() {
        let lower = text.to_lowercase();
        let base = blocks[stage_idx];
        let keywords = &E8_KEYWORDS[stage_idx];

        let mut best_offset = 0usize;
        let mut best_score = 0usize;
        for (offset, kw_set) in keywords.iter().enumerate() {
            let score = kw_set.iter().filter(|kw| lower.contains(*kw)).count();
            if score > best_score {
                best_score = score;
                best_offset = offset;
            }
        }

        let e8_val = (base + best_offset) as u8;
        seq.push(e8_val.min(63));
    }

    seq
}

/// Convenience: parse + estimate in one call (legacy).
pub fn structured_reasoning_to_e8(text: &str) -> Vec<u8> {
    let stages = parse_structured_reasoning(text);
    estimate_e8_from_structured(&stages)
}

/// Convenience: parse 9-stage mythos + estimate in one call.
pub fn mythos_reasoning_to_e8(text: &str) -> Vec<u8> {
    let stages = parse_mythos_reasoning(text);
    estimate_e8_from_mythos(&stages)
}
