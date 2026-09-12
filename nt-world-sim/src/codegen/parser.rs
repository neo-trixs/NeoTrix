use super::GameDefinition;
use std::path::Path;

pub struct GameDefParser;

impl GameDefParser {
    pub fn parse_yaml(path: &Path) -> Result<GameDefinition, String> {
        let _content =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
        Ok(GameDefinition::default())
    }

    pub fn parse_json(path: &Path) -> Result<GameDefinition, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse JSON: {}", e))
    }
}
