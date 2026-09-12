use super::GameDefinition;

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

        for (name, _system_def) in &self.game_def.systems {
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

        for (name, _) in &self.game_def.entities {
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
