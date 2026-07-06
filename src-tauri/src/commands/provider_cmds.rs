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
