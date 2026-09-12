# Targeted Research 518 — Internal Pain Points (WISER Iteration #29)

> **Date**: 2026-09-13
> **Scope**: 3 internal pain points found via TODO/hardcoded/stub scan of `neotrix-core/src/`
> **Prior fixes**: 27 issues resolved in previous iterations

---

## Pain Point 1 — FscanModule: `discover_hosts()` and `enumerate_services()` Return Fabricated Network Data

**File**: `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_impl/nt_shield_internal_scan.rs:173-259`
**Severity**: P1 (security scanner returns fake topology — any downstream audit/decision is based on fiction)

**What's wrong**:
`FscanModule` is the internal network scanner (absorbed from fscan 14K★). Both core methods:
1. `discover_hosts()` (line 173) — returns a hardcoded vec of 3 fake `_HostInfo` entries with fabricated IPs, hostnames, OS, and MAC addresses
2. `enumerate_services()` (line 210) — pattern-matches on IP strings to return hardcoded `ServiceInfo` with fake banners and versions

The `config.target_cidr` is completely ignored. Any caller trusting these results will make security decisions based on invented network topology. The method signatures accept `&mut self` but never actually scan anything.

**Concrete fix (10 lines)**:

```rust
// nt_shield_internal_scan.rs:173
pub async fn discover_hosts(&mut self) -> Result<Vec<_HostInfo>, String> {
    let output = tokio::process::Command::new("fscan")
        .args(["-t", &self.config.target_cidr, "-p", "all", "-oJ", "/dev/stdout"])
        .output()
        .await
        .map_err(|e| format!("fscan not found or failed: {e}"))?;
    if !output.status.success() {
        return Err(format!("fscan exited with {}", output.status));
    }
    let hosts = parse_fscan_json(&output.stdout)?;
    self.internal_hosts = hosts.clone();
    Ok(hosts)
}
```

---

## Pain Point 2 — GraphifyBackend: Entity Extraction Is Naive Token Splitting, Not Real NER

**File**: `neotrix-core/src/l1_action/nt_memory/nt_memory_graphify.rs:68-91`
**Severity**: P1 (knowledge graph gets populated with junk tokens — every downstream query/index is corrupted)

**What's wrong**:
`GraphifyBackend::extract()` is the primary entry point for knowledge graph construction. Instead of calling the Graphify service or running NER, it:
1. Splits text on non-alphanumeric characters (line 73)
2. Filters tokens ≥3 chars (line 74)
3. Labels every token as `"term"` entity type (line 80)
4. Creates `RelatedTo` edges between consecutive tokens (line 86)

For `"The quick brown fox"`, this produces 4 entities (`"The"`, `"quick"`, `"brown"`, `"fox"`) with 3 `RelatedTo` edges — garbage that pollutes the KB. Every downstream use of the graph (shortest_path, subgraph, community_detection) operates on noise.

**Concrete fix (8 lines)**:

```rust
// nt_memory_graphify.rs:68
fn extract(&self, text: &str) -> Result<(Vec<ExtractedEntity>, Vec<ExtractedRelation>), String> {
    if text.trim().is_empty() {
        return Err("text must not be empty".to_string());
    }
    if !self.is_available() {
        return Err("graphify backend not reachable".to_string());
    }
    let body = reqwest::blocking::get(format!("{}/extract?text={}", self.endpoint, urlencoding::encode(text)))
        .map_err(|e| format!("graphify request failed: {e}"))?;
    let parsed: ExtractResponse = body.json().map_err(|e| format!("parse error: {e}"))?;
    Ok((parsed.entities, parsed.relations))
}
```

---

## Pain Point 3 — VerifierAgent: `_verify_shot()` Uses Keyword Matching Instead of VLM Verification

**File**: `neotrix-core/src/l6_meta/coordination/verifier_agent.rs:186-217`
**Severity**: P1 (video quality verification is fabricated — `passed: true` on any input corrupts the regeneration loop)

**What's wrong**:
`_verify_shot()` is supposed to verify video shot quality using a VLM (Vision-Language Model). Instead it:
1. Calls `simulate_verification()` (line 196) which is keyword matching on `spec_description` and `memory_context`
2. Checks if the description contains "character"/"face" → score 7, else 9 (lines 227-232)
3. Checks for "lighting"/"scene" → score 6, else 8 (lines 235-240)
4. Returns `passed: true` for any description without entity/environment keywords

The `_auto_correct_prompt()` (line 336) just appends suggested corrections as raw strings instead of using an LLM to rewrite. The entire verification→regeneration loop is a no-op that always passes, meaning bad video output is never caught.

**Concrete fix (10 lines)**:

```rust
// verifier_agent.rs:195
pub(crate) fn _verify_shot(
    &mut self,
    _shot_id: &str,
    video_path: &str,
    spec_description: &str,
    memory_context: Option<&str>,
) -> VerificationResult {
    let start = std::time::Instant::now();
    let scores = self.verify_with_vlm(video_path, spec_description, memory_context).await;
    let total_score = self.calculate_total_score(&scores);
    let passed = total_score >= self.config.pass_threshold;
    let needs_regeneration = !passed && self.history.len() < self.config.max_regeneration_attempts as usize;
    let result = VerificationResult { passed, total_score, scores, error_types: vec![], suggested_corrections: vec![],
        needs_regeneration, verification_time_ms: start.elapsed().as_millis() as u64 };
    self.history.push(result.clone());
    result
}
```

---

## Summary

| # | File:Line | Severity | Issue | Impact |
|---|-----------|----------|-------|--------|
| 1 | `nt_shield_internal_scan.rs:173` | P1 | Hardcoded fake host/service data | Security audit decisions based on fiction |
| 2 | `nt_memory_graphify.rs:68` | P1 | Token-splitting代替NER | KB populated with junk, corrupts all graph queries |
| 3 | `verifier_agent.rs:195` | P1 | Keyword matching代替VLM | Bad video output never caught, regeneration loop is a no-op |
