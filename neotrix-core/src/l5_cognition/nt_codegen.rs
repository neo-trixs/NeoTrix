//! Multi-Engine Code Generation — absorbed from nt-world-sim
//!
//! YAML/JSON game definitions → Bevy/Unity/Godot/Unreal code.
//!
//! Source: archive/nt-world-sim/src/codegen/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameDefinition {
    pub name: String,
    pub version: String,
    pub engine: EngineConfig,
    pub entities: HashMap<String, EntityDef>,
    pub systems: HashMap<String, SystemDef>,
    pub resources: HashMap<String, ResourceDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub target: String,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityDef {
    pub components: Vec<String>,
    pub systems: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemDef {
    pub priority: i32,
    pub read: Vec<String>,
    pub write: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDef {
    pub fields: HashMap<String, String>,
}

// GameDefinition defined above
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
        let dir = tempfile::Builder::new()
            .prefix("nt_world_sim_test_game")
            .tempdir()
            .unwrap();
        let path = dir.path().join("game.yaml");
        std::fs::write(&path, yaml).unwrap();
        let def = GameDefParser::parse_yaml(&path).unwrap();
        assert_eq!(def.name, "TestGame");
        assert_eq!(def.engine.target, "bevy");
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
        let dir = tempfile::Builder::new()
            .prefix("nt_world_sim_test_json")
            .tempdir()
            .unwrap();
        let path = dir.path().join("game.json");
        std::fs::write(&path, json).unwrap();
        let def = GameDefParser::parse_json(&path).unwrap();
        assert_eq!(def.name, "JsonGame");
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
        let dir = tempfile::Builder::new()
            .prefix("nt_world_sim_test_auto")
            .tempdir()
            .unwrap();
        let path = dir.path().join("auto.yml");
        std::fs::write(&path, yaml).unwrap();
        let def = GameDefParser::parse(&path).unwrap();
        assert_eq!(def.name, "AutoGame");
    }
}

// Types defined above

pub struct CodeGenerator {
    game_def: GameDefinition,
}

impl CodeGenerator {
    pub fn new(game_def: GameDefinition) -> Self {
        Self { game_def }
    }

    pub fn generate_bevy(&self) -> String {
        let mut code = String::new();
        code.push_str("// Auto-generated Bevy code\n");
        code.push_str("use bevy::prelude::*;\n\n");

        for (name, entity_def) in &self.game_def.entities {
            code.push_str(&format!("#[derive(Component)]\nstruct {} {{\n", name));
            for component in &entity_def.components {
                code.push_str(&format!("    {}: {},\n", component, "f32"));
            }
            code.push_str("}\n\n");
        }

        for name in self.game_def.systems.keys() {
            code.push_str(&format!("fn {}_system(", name));
            code.push_str(") {\n");
            code.push_str("    // TODO: implement system logic\n");
            code.push_str("}\n\n");
        }

        code
    }

    pub fn generate_unity(&self) -> String {
        let mut code = String::new();
        code.push_str("// Auto-generated Unity DOTS code\n");
        code.push_str("using Unity.Entities;\n\n");

        for (name, entity_def) in &self.game_def.entities {
            code.push_str(&format!("public struct {} : IComponentData {{\n", name));
            for component in &entity_def.components {
                code.push_str(&format!("    public {} {};\n", "float", component));
            }
            code.push_str("}\n\n");
        }

        code
    }

    pub fn generate_godot(&self) -> String {
        let mut code = String::new();
        code.push_str("# Auto-generated Godot GDScript code\n\n");

        for name in self.game_def.entities.keys() {
            code.push_str("extends Node\n\n");
            code.push_str(&format!("class_name {}\n\n", name));
            code.push_str("func _ready():\n");
            code.push_str("    pass\n\n");
        }

        code
    }

    pub fn generate(&self) -> String {
        match self.game_def.engine.target.as_str() {
            "bevy" => self.generate_bevy(),
            "unity" => self.generate_unity(),
            "godot" => self.generate_godot(),
            _ => self.generate_bevy(),
        }
    }
}

impl Default for GameDefinition {
    fn default() -> Self {
        Self {
            name: "MyGame".to_string(),
            version: "0.1.0".to_string(),
            engine: EngineConfig {
                target: "bevy".to_string(),
                features: vec!["2d".to_string()],
            },
            entities: std::collections::HashMap::new(),
            systems: std::collections::HashMap::new(),
            resources: std::collections::HashMap::new(),
        }
    }
}
