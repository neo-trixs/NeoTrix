use keyring::Entry;

pub fn get_api_key() -> Result<String, String> {
    let entry = Entry::new("novachat", "anthropic_api_key")
        .map_err(|e| format!("keyring error: {}", e))?;
    entry.get_password().map_err(|e| format!("no api key: {}", e))
}

pub fn save_api_key(key: &str) -> Result<(), String> {
    let entry = Entry::new("novachat", "anthropic_api_key")
        .map_err(|e| format!("keyring error: {}", e))?;
    entry.set_password(key).map_err(|e| format!("failed to save key: {}", e))
}

pub fn delete_api_key() -> Result<(), String> {
    let entry = Entry::new("novachat", "anthropic_api_key")
        .map_err(|e| format!("keyring error: {}", e))?;
    entry.delete_credential().map_err(|e| format!("failed to delete key: {}", e))
}

pub fn has_api_key() -> bool {
    get_api_key().map(|k| !k.is_empty()).unwrap_or(false)
}
