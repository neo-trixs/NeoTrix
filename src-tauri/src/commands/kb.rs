use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct KbEntry {
    pub key: String,
    pub namespace: String,
    pub value: String,
    pub updated_at: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct KbQueryResult {
    pub entries: Vec<KbEntry>,
    pub total: usize,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct ExperienceEntry {
    pub cycle_id: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub created_at: String,
}

#[tauri::command]
pub async fn query_kb(namespace: String, keyword: Option<String>) -> Result<KbQueryResult, String> {
    let mut args = vec!["kb".into(), "query".into(), namespace];
    if let Some(kw) = keyword {
        args.push("--kw".into());
        args.push(kw);
    }
    let output = super::neotrix_cli::run_cli(args).await?;
    if output.success {
        serde_json::from_str(&output.stdout)
            .map_err(|e| format!("Failed to parse KB query result: {e}"))
    } else {
        Ok(KbQueryResult::default())
    }
}

#[tauri::command]
pub async fn list_experiences() -> Result<Vec<ExperienceEntry>, String> {
    let output = super::neotrix_cli::run_cli(vec![
        "experience".into(),
        "hub".into(),
    ])
    .await?;
    if output.success {
        serde_json::from_str(&output.stdout)
            .map_err(|e| format!("Failed to parse experiences: {e}"))
    } else {
        Ok(vec![])
    }
}
