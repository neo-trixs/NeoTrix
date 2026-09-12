use super::GameDefinition;
use std::path::Path;

pub struct GameDefParser;

impl GameDefParser {
    /// Parse YAML game definition file
    pub fn parse_yaml(path: &Path) -> Result<GameDefinition, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
        serde_yaml::from_str(&content)
            .map_err(|e| format!("YAML parse error: {}", e))
    }

    /// Parse JSON game definition file
    pub fn parse_json(path: &Path) -> Result<GameDefinition, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("JSON parse error: {}", e))
    }

    /// Auto-detect format by extension
    pub fn parse(path: &Path) -> Result<GameDefinition, String> {
        match path.extension().and_then(|e| e.to_str()) {
            Some("yaml") | Some("yml") => Self::parse_yaml(path),
            Some("json") => Self::parse_json(path),
            _ => Err(format!("Unsupported format: {}", path.display())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_yaml() {
        let yaml = r#"
name: TestGame
version: "0.1.0"
engine:
  target: bevy
  features:
    - 2d
entities: {}
systems: {}
resources: {}
"#;
        let path = Path::new("/tmp/nt_world_sim_test_game.yaml");
        std::fs::write(path, yaml).unwrap();
        let def = GameDefParser::parse_yaml(path).unwrap();
        assert_eq!(def.name, "TestGame");
        assert_eq!(def.engine.target, "bevy");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_parse_json() {
        let json = r#"{
  "name": "JsonGame",
  "version": "1.0.0",
  "engine": { "target": "bevy", "features": ["3d"] },
  "entities": {},
  "systems": {},
  "resources": {}
}"#;
        let path = Path::new("/tmp/nt_world_sim_test_game.json");
        std::fs::write(path, json).unwrap();
        let def = GameDefParser::parse_json(path).unwrap();
        assert_eq!(def.name, "JsonGame");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_auto_detect() {
        let yaml = r#"
name: AutoGame
version: "0.1.0"
engine:
  target: bevy
  features: []
entities: {}
systems: {}
resources: {}
"#;
        let path = Path::new("/tmp/nt_world_sim_test_auto.yml");
        std::fs::write(path, yaml).unwrap();
        let def = GameDefParser::parse(path).unwrap();
        assert_eq!(def.name, "AutoGame");
        let _ = std::fs::remove_file(path);
    }
}
