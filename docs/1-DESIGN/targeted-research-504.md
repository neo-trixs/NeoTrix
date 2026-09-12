# Targeted Research #504 — Internal Pain Points

**Date**: 2026-09-12
**Scope**: neotrix-core/src/ — hardcoded stubs, dead code, incomplete implementations
**Method**: grep-based sweep for TODO/FIXME/HACK, `Ok(vec![])`/`Ok(0)` returns, `#[allow(dead_code)]`

---

## Pain Point 1: Fake Network Scan Results (P0 — Security Critical)

**File**: `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_impl/nt_shield_internal_scan.rs:173-206`

**What's wrong**: `discover_hosts()` and `enumerate_services()` return **hardcoded fake hosts** (192.168.1.1, .10, .25) with fake open ports and service banners. A security scanner that invents fake results is **actively dangerous** — operators will trust the output and miss real vulnerabilities. The scan is registered as a real capability but lies about results.

**Severity**: **P0** — Security tool returning fabricated data is worse than no tool.

**Code sketch (10 lines)**:
```rust
pub async fn discover_hosts(&mut self) -> Result<Vec<_HostInfo>, String> {
    // Attempt real ARP/ICMP scan, fallback to error
    let output = tokio::process::Command::new("arp")
        .args(["-a"])
        .output()
        .await
        .map_err(|e| format!("arp failed: {e}"))?;
    let hosts = parse_arp_output(&String::from_utf8_lossy(&output.stdout))
        .ok_or("failed to parse ARP table")?;
    self.internal_hosts = hosts.clone();
    Ok(hosts)
}
```

---

## Pain Point 2: Model Adapter Always Fakes Success (P1 — Data Integrity)

**File**: `neotrix-core/src/l1_action/nt_io/model_adapter.rs:156-178`

**What's wrong**: `apply_lora()` and `_apply_ip_adapter()` return `success: true` with fabricated `similarity_score: 0.95` without ever calling any model. The `output_path` points to a file that doesn't exist. Downstream code trusting `success == true` will attempt to read a phantom file, causing silent failures or data corruption.

**Severity**: **P1** — Users get false confidence; pipeline silently breaks.

**Code sketch (8 lines)**:
```rust
pub fn apply_lora(&mut self, lora_id: &str, input_path: &str, strength: f32) -> AdapterResult {
    let Some(adapter) = self.adapters.get(lora_id) else {
        return AdapterResult { success: false, output_path: String::new(),
            adapter_name: String::new(), application_time_ms: 0,
            similarity_score: 0.0, error: Some("adapter not found".into()) };
    };
    // Delegate to actual backend (ComfyUI / SD-WebUI API)
    match self.call_backend(&adapter.backend_url, input_path, strength).await {
        Ok(path) => AdapterResult { success: true, output_path: path, .. },
        Err(e)   => AdapterResult { success: false, error: Some(e), .. },
    }
}
```

---

## Pain Point 3: Recon Scanner Registered but Non-Functional (P1 — C1 Lie)

**File**: `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_recon.rs:31-34`

**What's wrong**: `_ReconScanner::scan()` returns `Err("not yet implemented (fresh bud)")` but is **registered as a C1-certified capability** in the CapabilityRegistry. The `wiring_evidence` metadata says "not yet wired to production" — contradicting its C1 status (which requires unit tests + functional behavior). This is a **constellation honesty violation**.

**Severity**: **P1** — Capability tree lies about maturity; callers trust C1 means functional.

**Code sketch (7 lines)**:
```rust
pub fn register_capability(tree: &mut CapabilityRegistry) -> Result<(), RegistryError> {
    let mut node = CapabilityNode::new_primitive(
        "nt_shield::recon::scan".into(), Domain::Shield,
        vec!["shield.recon.scan".into(), "shield.recon.enumerate".into()],
    );
    node.layer = NodeLayer::L0Seed; // Downgrade from L1Composite — no real logic
    node.constellation = ConstellationLevel::C0Compiles; // Honest: only compiles
    tree.register(node)
}
```

---

## Bonus: Quick Wins (P2)

| File:Line | Issue | Action |
|-----------|-------|--------|
| `visual/face_consistency.rs:173` | `_fix_faces` returns hardcoded `consistency_score: 0.95` | Downgrade to C0 or implement |
| `visual/style_harmonizer.rs:114-157` | `_analyze_style`/`_harmonize`/`_match_colors` all fake | Remove from production path |
| `video_post_processor.rs:342-369` | `denoise`/`sharpen` return fabricated scores | Either implement or expose as "config preview" |
| `speculative_decoding.rs:166` | `generate()` returns empty `output_tokens` with fake throughput | Mark as experimental, not default |

## Statistics

| Category | Count |
|----------|-------|
| `TODO`/`FIXME` in non-test code | ~100+ |
| `#[allow(dead_code)]` | ~100+ |
| Hardcoded return stubs (security) | 3 critical |
| Hardcoded return stubs (visual) | 6 moderate |
| `todo!()`/`unimplemented!()` in production | 0 (clean) |
