# Targeted Pain Points — Iteration #481

> Found via TODO/FIXME scan + hardcoded-return + dead-code analysis.
> Each is a **concrete, fixable** internal pain point.

---

## P1: VerifierAgent Uses Keyword Heuristics Instead of VLM

**File:** `neotrix-core/src/l6_meta/coordination/verifier_agent.rs:186-217`

**What's wrong:** `_verify_shot()` calls `simulate_verification()` which is a keyword-matching heuristic — checks if description contains "character"/"face" and returns hardcoded scores. This means the video verification pipeline **always passes** if the description doesn't mention entities. Any user trusting the verification gate gets a false-positive.

**Severity:** P0 — The verification gate is the quality control bottleneck; fake scores bypass it entirely.

**Fix sketch:**
```rust
// verifier_agent.rs — replace simulate_verification with real VLM call
async fn verify_shot(&mut self, shot_id: &str, video_path: &str, spec: &str) -> VerificationResult {
    let start = Instant::now();
    
    // Extract a representative frame from the video
    let frame = self.extract_key_frame(video_path, shot_id).await?;
    
    // Call VLM with the frame + spec prompt
    let prompt = format!(
        "Compare this video frame against the specification: {}. \
         Score entity_consistency, temporal_quality, and spec_alignment each 0-10.",
        spec
    );
    let vlm_response = self.vlm_provider.complete(&frame, &prompt).await?;
    
    // Parse structured scores from VLM response
    let scores = self.parse_vlm_scores(&vlm_response)?;
    let total_score = self.calculate_total_score(&scores);
    
    VerificationResult {
        passed: total_score >= self.config.pass_threshold,
        total_score,
        scores,
        error_types: self.detect_errors(&vlm_response),
        suggested_corrections: self.extract_corrections(&vlm_response),
        needs_regeneration: total_score < self.config.pass_threshold
            && self.history.len() < self.config.max_regeneration_attempts as usize,
        verification_time_ms: start.elapsed().as_millis() as u64,
    }
}
```

---

## P2: ReferenceGeneration Returns Fabricated Success + Fake Scores

**File:** `neotrix-core/src/l1_action/nt_io/reference_generation.rs:224-272`

**What's wrong:** `video_to_video()`, `image_to_video()`, and `style_transfer()` all return `success: true` with fabricated `quality_score` (0.83-0.90) and `reference_similarity` (0.85-0.89) without calling any model. Downstream consumers (e.g., `QualityControlPipeline`) trust these scores and mark the generation as "high quality".

**Severity:** P0 — Fabricated quality metrics poison the entire production pipeline; `ProductionOrchestrator` uses these scores to decide pass/fail.

**Fix sketch:**
```rust
// reference_generation.rs — delegate to platform gateway, propagate errors
pub fn video_to_video(&mut self, input_path: &str) -> GenerationResult {
    let start = Instant::now();
    
    match self.platform_gateway.generate_video(input_path, &self.config) {
        Ok(output_path) => {
            let result = GenerationResult {
                success: true,
                output_paths: vec![output_path],
                generation_time_ms: start.elapsed().as_millis() as u64,
                model_used: self.config.model_name.clone(),
                reference_similarity: 0.0, // filled by post-hoc VLM scoring
                quality_score: 0.0,        // filled by post-hoc VLM scoring
                error: None,
            };
            self.history.push(result.clone());
            result
        }
        Err(e) => GenerationResult {
            success: false,
            output_paths: vec![],
            generation_time_ms: start.elapsed().as_millis() as u64,
            model_used: self.config.model_name.clone(),
            reference_similarity: 0.0,
            quality_score: 0.0,
            error: Some(e.to_string()),
        }
    }
}
```

---

## P3: NT-ACT Cache L2 Disk Read Is a Dead No-Op

**File:** `neotrix-core/src/l1_action/nt_act/actions/nt_act_cache.rs:194-201`

**What's wrong:** The `l2_cache` field is structurally present (disk cache tier), but the `get()` method at line 194 does nothing — it's an empty `if let` block. When L1 misses, it falls straight through to `CacheResult::Miss`. Additionally, the bloom filter penetration protection at line 200 is also a no-op. This means the cache is effectively L1-only; any hot data exceeding `l1_max_entries` gets evicted permanently, and the L2/bloom filter infrastructure is wasted allocations.

**Severity:** P1 — Silent performance degradation under load; data is lost when L1 fills, not persisted to disk.

**Fix sketch:**
```rust
// nt_act_cache.rs — implement L2 disk read + bloom filter
fn get(&mut self, key: &str) -> CacheResult {
    // L1 check (existing)
    if let Some(entry) = self.l1_cache.get(key) {
        self.stats.hits += 1;
        self.update_hit_rate();
        return CacheResult::Hit(entry.clone());
    }

    // L2 disk read
    if let Some(ref disk_cache) = self.l2_cache {
        if let Some(entry) = disk_cache.read_entry(key) {
            // Promote to L1
            self.l1_cache.insert(key.to_string(), entry.clone());
            self.stats.hits += 1;
            self.update_hit_rate();
            return CacheResult::Hit(entry);
        }
    }

    // Bloom filter skip — avoid redundant lookups for known-missing keys
    if self.config.penetration_protection {
        if let Some(ref bloom) = self.bloom_filter {
            if !bloom.might_contain(key.as_bytes()) {
                self.stats.bloom_filter_skips += 1;
                self.stats.misses += 1;
                self.update_hit_rate();
                return CacheResult::Miss;
            }
        }
    }

    self.stats.misses += 1;
    self.update_hit_rate();
    CacheResult::Miss
}
```

---

## Summary

| # | Pain Point | File | Severity | Category |
|---|-----------|------|----------|----------|
| P1 | VerifierAgent uses keyword heuristics, not VLM | `verifier_agent.rs:186` | **P0** | Dead code path |
| P2 | ReferenceGeneration returns fabricated quality scores | `reference_generation.rs:224` | **P0** | Hardcoded values |
| P3 | Cache L2 disk read + bloom filter are empty no-ops | `nt_act_cache.rs:194` | **P1** | Incomplete implementation |

**Pattern:** All 3 follow the same anti-pattern — structural scaffolding exists (fields, methods, config) but the core logic is a no-op returning fabricated results. The "plumbing" looks real, so consumers trust it, but zero actual work happens.
