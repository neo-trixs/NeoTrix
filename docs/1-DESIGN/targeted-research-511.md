# Targeted Research #511 — Internal Pain Points

> WISER iteration loop: 3 pain points from `neotrix-core/src/`
> Date: 2026-09-13

---

## P1: Model Adapter Functions Are Hardcoded Stubs Returning Fake Success

**File:** `l1_action/nt_io/model_adapter.rs:157-228`

**What's wrong:** `apply_lora()`, `_apply_ip_adapter()`, and `_apply_controlnet()` all return `AdapterResult { success: true, ... }` with fabricated metrics (`application_time_ms: 1000`, `similarity_score: 0.95`) without invoking any actual model. Callers believe adapters were applied successfully when nothing happened. The `history` vector records fake entries, corrupting any downstream analytics.

**Severity:** P1 — Production pipeline silently produces fake results; users trust adapter output that was never generated.

**Fix sketch:**

```rust
pub fn apply_lora(&mut self, lora_id: &str, input_path: &str, strength: f32) -> AdapterResult {
    let adapter = match self.adapters.get(lora_id) {
        Some(a) => a,
        None => return AdapterResult {
            success: false, output_path: String::new(), adapter_name: String::new(),
            application_time_ms: 0, similarity_score: 0.0,
            error: Some(format!("adapter '{}' not registered", lora_id)),
        },
    };
    let start = std::time::Instant::now();
    // Delegate to actual inference backend (ONNX / Triton / local GPU)
    let output = match self.backend.apply_lora(adapter, input_path, strength) {
        Ok(path) => path,
        Err(e) => return AdapterResult {
            success: false, output_path: String::new(), adapter_name: adapter.name.clone(),
            application_time_ms: start.elapsed().as_millis() as u64, similarity_score: 0.0,
            error: Some(e.to_string()),
        },
    };
    let elapsed = start.elapsed().as_millis() as u64;
    let result = AdapterResult {
        success: true, output_path: output, adapter_name: adapter.name.clone(),
        application_time_ms: elapsed, similarity_score: -1.0, // computed by backend
        error: None,
    };
    self.history.push(result.clone());
    result
}
```

---

## P1: Internal Network Scanner Returns Hardcoded Fake Host Data

**File:** `l3_embodiment/nt_shield/nt_shield_impl/nt_shield_internal_scan.rs:173-207`

**What's wrong:** `discover_hosts()` returns a hardcoded `vec!["192.168.1.1", "192.168.1.10", "192.168.1.25"]` with fabricated hostnames, OS info, and open ports. `enumerate_services()` (line 210) does the same — matching against hardcoded IP strings and returning fake service banners. A security tool returning invented network topology is actively dangerous: users may believe their network was scanned when it wasn't, or miss real vulnerable hosts.

**Severity:** P1 — Security scanner returns fabricated data; false sense of coverage.

**Fix sketch:**

```rust
pub async fn discover_hosts(&mut self) -> Result<Vec<_HostInfo>, String> {
    let start = std::time::Instant::now();
    // Try real ARP/ICMP scan via system commands
    let output = tokio::process::Command::new("arp")
        .args(["-a"])
        .output()
        .await
        .map_err(|e| format!("arp failed: {e}"))?;

    let hosts = parse_arp_output(&String::from_utf8_lossy(&output.stdout));
    if hosts.is_empty() {
        tracing::warn!(elapsed_ms = start.elapsed().as_millis(), "ARP scan returned 0 hosts; fallback to ICMP");
        // TODO(nt-shield): ICMP sweep fallback via ping -c1 -W1 <subnet>
    }
    self.internal_hosts = hosts.clone();
    Ok(hosts)
}
```

---

## P1: Resource Budget Cost Estimation Uses Hardcoded Price Table

**File:** `l1_action/nt_act/resource_budget.rs:288-296`

**What's wrong:** `estimate_cost()` uses a hardcoded `match` on 3 model names (`"gpt-4"`, `"gpt-3.5-turbo"`, `"claude-3"`) with static per-1k-token prices. Any other model (Gemini, Llama, Mistral, local) falls through to `$0.001/1k` — a meaningless default. Prices are never updated from provider APIs. The function is the single cost estimation point for the entire budget system, so all budget checks are wrong for 95% of models.

**Severity:** P1 — Budget enforcement is based on invented numbers; cost alerts are unreliable.

**Fix sketch:**

```rust
pub fn estimate_cost(&self, token_count: u64, model: &str) -> f64 {
    // Load from KB or config; fallback to last-known prices
    let prices = self.price_cache.get(model).copied().unwrap_or_else(|| {
        tracing::warn!(model, "no cached price; using fallback $0.001/1k");
        0.001
    });
    (token_count as f64 / 1000.0) * prices
}

/// Refresh price cache from provider pricing endpoints (call on init / schedule).
pub async fn refresh_prices(&mut self) -> Result<(), String> {
    let providers = vec![
        ("openai", "https://api.openai.com/v1/models"),
        ("anthropic", "https://api.anthropic.com/v1/models"),
    ];
    for (name, url) in providers {
        let resp = reqwest::get(url).await.map_err(|e| format!("{name}: {e}"))?;
        // Parse response and insert into self.price_cache
        // ...
    }
    Ok(())
}
```
