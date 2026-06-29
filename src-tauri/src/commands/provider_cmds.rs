use tauri::command;
use neotrix::neotrix::nt_io_provider::factory::create_provider;
use neotrix::neotrix::nt_io_provider::types::LlmRequest;
use super::{payload_to_provider_config, read_provider_config, ProviderConfigPayload};

#[command]
pub async fn save_provider_config(payload: ProviderConfigPayload) -> Result<(), String> {
    let path = dirs::config_dir()
        .ok_or("Cannot find config directory")?
        .join("neotrix")
        .join("provider.json");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config dir: {}", e))?;
    }
    let json = serde_json::to_string_pretty(&payload)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    std::fs::write(&path, json).map_err(|e| format!("Failed to write config: {}", e))?;
    Ok(())
}

#[command]
pub async fn get_current_provider() -> Result<ProviderConfigPayload, String> {
    read_provider_config()
}

#[command]
pub async fn test_provider(payload: ProviderConfigPayload) -> Result<bool, String> {
    let config = payload_to_provider_config(&payload);
    let provider = create_provider(config);
    let request = LlmRequest::new(&payload.model, "Reply with just: OK");
    match provider.complete(&request).await {
        Ok(_) => Ok(true),
        Err(e) => {
            log::warn!("Provider test failed: {}", e);
            Ok(false)
        }
    }
}
