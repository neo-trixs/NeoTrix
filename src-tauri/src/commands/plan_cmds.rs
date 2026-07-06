use tauri::command;

#[command]
pub fn plan_list() -> Result<String, String> {
    Ok("[]".to_string())
}

#[command]
pub fn plan_create(name: String) -> Result<String, String> {
    Ok(format!("plan_{}", name.len()))
}

#[command]
pub fn plan_steps(_plan_id: String) -> Result<String, String> {
    Ok("[]".to_string())
}

#[command]
pub fn plan_step(_plan_id: String) -> Result<String, String> {
    Ok("step detail".to_string())
}

#[command]
pub fn plan_complete(_plan_id: String) -> Result<String, String> {
    Ok("completed".to_string())
}
