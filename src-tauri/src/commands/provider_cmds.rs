use std::sync::{Arc, OnceLock};
use tauri::command;
use neotrix::neotrix::nt_core_error::NeoTrixError;
use neotrix::neotrix::nt_io_provider::GatewayV2;

static GATEWAY: OnceLock<Arc<GatewayV2>> = OnceLock::new();

pub fn init_gateway() -> Arc<GatewayV2> {
    let gw = Arc::new(neotrix::neotrix::nt_io_provider::create_gateway());
    let _ = GATEWAY.set(gw.clone());
    gw
}

fn get_gateway() -> Result<&'static Arc<GatewayV2>, NeoTrixError> {
    GATEWAY.get().ok_or_else(|| NeoTrixError::Config("Gateway not initialized".into()))
}

#[command]
pub fn provider_status() -> Result<Vec<serde_json::Value>, NeoTrixError> {
    Ok(get_gateway()?.provider_status())
}

/* ══════════════════════════════════════════
   Phase 3 M2/M3 (吸收 grok-bot Usage & Billing + LM Studio health)
   ══════════════════════════════════════════ */

/// M2: per-provider 用量账本快照 — 活动记录, 非权威账单。
#[derive(serde::Serialize)]
pub struct ProviderUsageRow {
    pub provider: String,
    pub request_count: u64,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    /// 恒 true — 诚实标注 (grok-bot 吸收纪律)
    pub activity_record: bool,
}

#[command]
pub fn provider_usage_snapshot() -> Result<Vec<ProviderUsageRow>, NeoTrixError> {
    let rows = neotrix::core::nt_core_telemetry::global_provider_usage_ledger()
        .snapshot()
        .into_iter()
        .map(|r| ProviderUsageRow {
            provider: r.provider,
            request_count: r.request_count,
            prompt_tokens: r.prompt_tokens,
            completion_tokens: r.completion_tokens,
            activity_record: r.activity_record,
        })
        .collect();
    Ok(rows)
}

/// M3: 提供商连通性探测 — GET base_url (任意 HTTP 状态=存活), 5s 超时 fail-closed。
#[command]
pub async fn provider_test(base_url: String) -> Result<serde_json::Value, String> {
    let url = base_url.trim().trim_end_matches('/').to_string();
    if url.is_empty() || !url.starts_with("http") {
        return Err(format!("invalid base_url: `{}`", base_url));
    }
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .connect_timeout(std::time::Duration::from_secs(3))
        .build()
        .map_err(|e| format!("client build: {}", e))?;
    let start = std::time::Instant::now();
    match client.get(&url).send().await {
        Ok(resp) => {
            let latency_ms = start.elapsed().as_millis() as u64;
            let code = resp.status().as_u16();
            log::info!("[provider_test] {} → {} ({}ms)", url, code, latency_ms);
            Ok(serde_json::json!({
                "ok": true, "status_code": code, "latency_ms": latency_ms,
                "activity_record": false,
            }))
        }
        Err(e) => Err(format!("unreachable: {}", e)),
    }
}
