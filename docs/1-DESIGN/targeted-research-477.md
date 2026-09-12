# Targeted Research #477 — Internal Pain Points (Post-27-Fix Iteration)

**Date**: 2026-09-12
**Scope**: neotrix-core/src/ — production code with stub implementations
**Method**: TODO/FIXME grep → hardcoded return audit → dead code annotation scan

---

## Pain Point 1: Security Scanner Returns Fabricated Host Data

**File**: `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_impl/nt_shield_internal_scan.rs:173-207`

**What's wrong**: `discover_hosts()` and `enumerate_services()` return hardcoded fake network topology (192.168.1.1 gateway, 192.168.1.10 webserver, etc.) instead of performing real ARP/ICMP discovery or nmap scanning. Any downstream consumer (risk assessment, audit reports, GWT attention routing) operates on fabricated intelligence. A security scanner that returns fake data is worse than no scanner — it creates false confidence.

**Severity**: **P0** — Security integrity violation. The NT-SHIELD domain's core scanning capability is a lie.

**Fix sketch**:
```rust
pub async fn discover_hosts(&mut self) -> Result<Vec<_HostInfo>, String> {
    let output = tokio::process::Command::new("fscan")
        .args(["-t", &self.config.target_cidr, "-p", "all", "-o", "json"])
        .output()
        .await
        .map_err(|e| format!("fscan execution failed: {e}"))?;

    if !output.status.success() {
        return Err(format!("fscan exited with {}", output.status));
    }

    let hosts: Vec<_HostInfo> = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("fscan JSON parse failed: {e}"))?;

    self.internal_hosts = hosts.clone();
    Ok(hosts)
}

pub async fn enumerate_services(&mut self, host: &str) -> Result<Vec<ServiceInfo>, String> {
    let output = tokio::process::Command::new("nmap")
        .args(["-sV", "-oX", "-", host])
        .output()
        .await
        .map_err(|e| format!("nmap execution failed: {e}"))?;

    parse_nmap_xml(&output.stdout)
}
```

---

## Pain Point 2: Quality Gate Approves Everything Blindly

**File**: `neotrix-core/src/l6_meta/coordination/quality_control.rs:283-304`

**What's wrong**: The three-tier review flow (AI → Human → Platform) hardcodes `ReviewStatus::Approved` with `total_score: 0.90` for both Human and Platform levels. The quality gate is a rubber stamp — every piece of content passes regardless of actual quality. This defeats the entire purpose of the `QualityControlPipeline` (defined in CONTEXT.md as "AI 初检 → 人工复审 → 平台终审").

**Severity**: **P0** — Quality gate bypass. No content is ever rejected.

**Fix sketch**:
```rust
ReviewLevel::Human => {
    let review_url = format!("{}/api/review/submit", self.config.human_review_endpoint);
    let client = reqwest::Client::new();
    let response = client.post(&review_url)
        .json(&ReviewSubmission { content_id: content_id.to_string() })
        .send().await
        .map_err(|e| format!("Human review API unreachable: {e}"))?;

    let result: ReviewResult = response.json().await
        .map_err(|e| format!("Failed to parse human review response: {e}"))?;
    result
}
ReviewLevel::Platform => {
    // Return Pending instead of Approved — platform review is async
    ReviewResult {
        status: ReviewStatus::Pending,
        total_score: 0.0, // score set later by callback
        // ...
    }
}
```

---

## Pain Point 3: VLM Video Verification Replaced by String Matching

**File**: `neotrix-core/src/l6_meta/coordination/verifier_agent.rs:195-281`

**What's wrong**: `_verify_shot()` claims to verify video quality but actually calls `simulate_verification()` which is pure keyword-matching heuristics — checking if the description contains "character" or "action" and assigning integer scores (7, 8, 9) based on string length. This is the NT-META verification pipeline that gates video regeneration decisions. With this stub, every video passes verification and no regeneration is ever triggered, even for corrupted or off-spec output.

**Severity**: **P1** — Verification is a sham; regeneration loop is dead.

**Fix sketch**:
```rust
fn simulate_verification(&self, description: &str, context: Option<&str>) -> Vec<_VerificationScore> {
    let prompt = format!(
        "Analyze this video shot for consistency. \
         Description: {description}\nContext: {}",
        context.unwrap_or("none")
    );

    let vlm_response = self.vlm_provider
        .analyze_frames(&self.frame_sampler.sample(&_shot_path), &prompt)
        .await
        .unwrap_or_default();

    vec![
        _VerificationScore {
            dimension: _VerificationDimension::EntityConsistency,
            score: vlm_response.entity_consistency_score,
            explanation: vlm_response.entity_explanation,
        },
        // ... other dimensions from VLM response
    ]
}
```

---

## Summary

| # | File | Problem | Severity |
|---|------|---------|----------|
| 1 | `nt_shield_internal_scan.rs:173` | Hardcoded fake network topology in security scanner | P0 |
| 2 | `quality_control.rs:283` | Quality gate auto-approves everything with 0.90 score | P0 |
| 3 | `verifier_agent.rs:195` | VLM verification stub uses string matching instead of vision model | P1 |

All three represent the same anti-pattern: **production API surface with mock internals**. The modules compile, register, and are callable — but return fabricated confidence scores. Downstream systems (GWT attention routing, SEAL regeneration loop, risk assessment) make decisions on hallucinated data.
