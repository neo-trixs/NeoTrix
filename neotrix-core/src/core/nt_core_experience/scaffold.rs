use log;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ScaffoldType {
    Handler,
    Subsystem,
    Agent,
}

impl ScaffoldType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "handler" => Some(Self::Handler),
            "subsystem" => Some(Self::Subsystem),
            "agent" => Some(Self::Agent),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Handler => "handler",
            Self::Subsystem => "subsystem",
            Self::Agent => "agent",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScaffoldConfig {
    pub name: String,
    pub type_: ScaffoldType,
    pub description: String,
}

pub struct ScaffoldGenerator;

impl ScaffoldGenerator {
    pub fn generate_and_print(type_str: &str, name: &str, description: &str) {
        let type_ = match ScaffoldType::from_str(type_str) {
            Some(t) => t,
            None => {
                log::error!(
                    "error: unknown type '{}'. Use handler, subsystem, or agent.",
                    type_str
                );
                std::process::exit(1);
            }
        };
        let cfg = ScaffoldConfig {
            name: name.to_string(),
            type_,
            description: if description.is_empty() {
                format!("A new {} component", type_.name())
            } else {
                description.to_string()
            },
        };

        log::info!("╭──────────────────────────────────────────────╮");
        log::info!("│  NeoTrix Scaffold Generator                  │");
        log::info!("│  Type: {:<33} │", cfg.type_.name());
        log::info!("│  Name: {:<33} │", cfg.name);
        log::info!("╰──────────────────────────────────────────────╯");
        log::info!("scaffold");

        match cfg.type_ {
            ScaffoldType::Handler => Self::print_handler_template(&cfg),
            ScaffoldType::Subsystem => Self::print_subsystem_template(&cfg),
            ScaffoldType::Agent => Self::print_agent_template(&cfg),
        }
    }

    fn print_handler_template(cfg: &ScaffoldConfig) {
        let name = &cfg.name;
        let desc = &cfg.description;
        let handler_method = format!(
            r#"    /// {desc}
    pub fn handle_{name}_tick(&mut self) -> String {{
        log::debug!("MODULES: {name}_tick");
        // TODO: implement {name} logic
        format!("{name}:stub")
    }}"#,
        );

        let dispatch_arm = format!(r#"            "{name}" => self.handle_{name}_tick(),"#);

        log::info!(
            "{}",
            Self::section("STEP 1 — Handler method (add to modules.rs)")
        );
        log::info!("{handler_method}");
        log::info!("scaffold");

        log::info!(
            "{}",
            Self::section("STEP 2 — Dispatch arm (add to modules.rs dispatch match)")
        );
        log::info!("{dispatch_arm}");
        log::info!("scaffold");

        let tier_reg = format!(r#"        registry.register("{name}", LoadTier::Warm);"#);
        log::info!(
            "{}",
            Self::section("STEP 3 — Tier registration (add to handler_tier.rs)")
        );
        log::info!("{tier_reg}");
        log::info!("scaffold");

        let field_decl = format!(
            r#"    /// {desc}
    pub {name}_counter: u64,"#
        );
        log::info!(
            "{}",
            Self::section("STEP 4 — Optional field (add to types.rs ConsciousnessIntegration)")
        );
        log::info!("{field_decl}");
        log::info!("scaffold");

        let pipeline = format!(
            r#"        // {name}
        let _ = self.dispatch_handler("{name}");"#
        );
        log::info!(
            "{}",
            Self::section("STEP 5 — Pipeline call (add to core.rs phase_two or phase_three)")
        );
        log::info!("{pipeline}");
        log::info!("scaffold");

        let name_clone = name.clone();
        let handler_names = format!(
            r#"            HandlerNode {{ name: "{name_clone}", interval_secs: 30, call_count: 0 }},"#
        );
        log::info!(
            "{}",
            Self::section(
                "STEP 6 — SelfInspectable registration (add to self_inspect.rs handler_names)"
            )
        );
        log::info!("{handler_names}");
        log::info!("scaffold");

        Self::print_file_hint(
            "modules.rs",
            &format!("neotrix/nt_mind_background_loop/consciousness/modules.rs"),
        );
    }

    fn print_subsystem_template(cfg: &ScaffoldConfig) {
        let name = &cfg.name;
        let desc = &cfg.description;
        let mod_name = name.replace('-', "_");
        let name_pascal = Self::to_pascal(&mod_name);

        let module_content = format!(
            r#"/// {desc}
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct {name_pascal}Config {{
    pub max_items: usize,
    pub enabled: bool,
}}

impl Default for {name_pascal}Config {{
    fn default() -> Self {{
        Self {{
            max_items: 100,
            enabled: true,
        }}
    }}
}}

#[derive(Debug, Clone)]
pub struct {name_pascal} {{
    pub config: {name_pascal}Config,
    pub items: Vec<String>,
    pub tick_count: u64,
}}

impl {name_pascal} {{
    pub fn new() -> Self {{
        Self {{
            config: {name_pascal}Config::default(),
            items: Vec::new(),
            tick_count: 0,
        }}
    }}

    pub fn with_config(config: {name_pascal}Config) -> Self {{
        Self {{
            config,
            items: Vec::new(),
            tick_count: 0,
        }}
    }}

    pub fn tick(&mut self) -> String {{
        self.tick_count += 1;
        log::debug!("{mod_name}: tick #{{}}", self.tick_count);
        format!("{mod_name}:tick={{}}", self.tick_count)
    }}

    pub fn stats(&self) -> String {{
        format!("{mod_name}:items={{}}_ticks={{}}", self.items.len(), self.tick_count)
    }}
}}

impl Default for {name_pascal} {{
    fn default() -> Self {{
        Self::new()
    }}
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_{mod_name}_new() {{
        let engine = {name_pascal}::new();
        assert_eq!(engine.items.len(), 0);
        assert_eq!(engine.tick_count, 0);
    }}

    #[test]
    fn test_{mod_name}_tick() {{
        let mut engine = {name_pascal}::new();
        let result = engine.tick();
        assert!(result.contains("tick=1"));
        assert_eq!(engine.tick_count, 1);
    }}

    #[test]
    fn test_{mod_name}_with_config() {{
        let config = {name_pascal}Config {{
            max_items: 50,
            enabled: false,
        }};
        let engine = {name_pascal}::with_config(config);
        assert_eq!(engine.config.max_items, 50);
        assert!(!engine.config.enabled);
    }}
}}
"#,
            name_pascal = &name_pascal,
        );

        let handler_method = format!(
            r#"    /// {desc}
    pub fn handle_{mod_name}_tick(&mut self) -> String {{
        if self.{mod_name}.is_none() {{
            self.{mod_name} = Some({name_pascal}::new());
            return format!("{mod_name}:init");
        }}
        self.{mod_name}.as_mut().unwrap().tick()
    }}"#,
            name_pascal = &name_pascal,
        );

        let dispatch_arm = format!(r#"            "{mod_name}" => self.handle_{mod_name}_tick(),"#);

        let field_decl = format!(
            r#"    /// {desc}
    pub {mod_name}: Option<{name_pascal}>,"#,
            name_pascal = &name_pascal,
        );

        let construct_init = format!(r#"{mod_name}: None,"#);

        let import = format!(
            r#"use crate::core::nt_core_experience::{name_pascal};"#,
            name_pascal = &name_pascal,
        );

        log::info!("{}", Self::section("MODULE — {mod_name}.rs (create file)"));
        log::info!("Path: core/nt_core_experience/{mod_name}.rs");
        log::info!("{module_content}");
        log::info!("scaffold");

        log::info!(
            "{}",
            Self::section("STEP 1 — mod.rs declaration (add to core/nt_core_experience/mod.rs)")
        );
        log::info!("pub mod {mod_name};");
        log::info!("pub use {mod_name}::{};", name_pascal);
        log::info!("scaffold");

        log::info!(
            "{}",
            Self::section("STEP 2 — Import (add to types.rs or modules.rs)")
        );
        log::info!("{import}");
        log::info!("scaffold");

        log::info!(
            "{}",
            Self::section(
                "STEP 3 — Field declaration (add to types.rs ConsciousnessIntegration struct)"
            )
        );
        log::info!("{field_decl}");
        log::info!("scaffold");

        log::info!(
            "{}",
            Self::section(
                "STEP 4 — Constructor init (add to types.rs ConsciousnessIntegration::new())"
            )
        );
        log::info!("{construct_init}");
        log::info!("scaffold");

        log::info!(
            "{}",
            Self::section("STEP 5 — Handler method (add to modules.rs)")
        );
        log::info!("{handler_method}");
        log::info!("scaffold");

        log::info!(
            "{}",
            Self::section("STEP 6 — Dispatch arm (add to modules.rs dispatch match)")
        );
        log::info!("{dispatch_arm}");
        log::info!("scaffold");

        let tier_reg = format!(r#"        registry.register("{mod_name}", LoadTier::Warm);"#);
        log::info!(
            "{}",
            Self::section("STEP 7 — Tier registration (add to handler_tier.rs)")
        );
        log::info!("{tier_reg}");
        log::info!("scaffold");

        let pipeline = format!(
            r#"        // {mod_name}
        let _ = self.dispatch_handler("{mod_name}");"#
        );
        log::info!(
            "{}",
            Self::section("STEP 8 — Pipeline call (add to core.rs phase_two or phase_three)")
        );
        log::info!("{pipeline}");
        log::info!("scaffold");

        let name_clone = mod_name.clone();
        let handler_names = format!(
            r#"            HandlerNode {{ name: "{name_clone}", interval_secs: 60, call_count: 0 }},"#
        );
        log::info!(
            "{}",
            Self::section("STEP 9 — SelfInspectable registration (add to self_inspect.rs)")
        );
        log::info!("{handler_names}");
        log::info!("scaffold");

        Self::print_file_hint(
            "mod.rs declaration",
            &format!("core/nt_core_experience/mod.rs"),
        );
    }

    fn print_agent_template(cfg: &ScaffoldConfig) {
        let name = &cfg.name;
        let desc = &cfg.description;
        let mod_name = name.replace('-', "_");
        let name_pascal = Self::to_pascal(&mod_name);

        let agent_content = format!(
            r#"/// {desc}
use std::collections::HashMap;
use serde::{{Deserialize, Serialize}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {name_pascal}Message {{
    pub role: String,
    pub content: String,
}}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {name_pascal}Config {{
    pub max_history: usize,
    pub system_prompt: String,
}}

impl Default for {name_pascal}Config {{
    fn default() -> Self {{
        Self {{
            max_history: 50,
            system_prompt: format!("You are {{}}, a NeoTrix A2A agent.", "{name}"),
        }}
    }}
}}

#[derive(Debug, Clone)]
pub struct {name_pascal}Agent {{
    pub config: {name_pascal}Config,
    pub history: Vec<{name_pascal}Message>,
    pub session_id: String,
}}

impl {name_pascal}Agent {{
    pub fn new() -> Self {{
        Self {{
            config: {name_pascal}Config::default(),
            history: Vec::new(),
            session_id: uuid_or_default(),
        }}
    }}

    pub fn with_config(config: {name_pascal}Config) -> Self {{
        Self {{
            config,
            history: Vec::new(),
            session_id: uuid_or_default(),
        }}
    }}

    pub fn process_message(&mut self, content: &str) -> String {{
        self.history.push({name_pascal}Message {{
            role: "user".to_string(),
            content: content.to_string(),
        }});
        if self.history.len() > self.config.max_history {{
            self.history.remove(0);
        }}
        // TODO: implement actual {name} agent logic
        format!("{{agent}}:ack_{{}}", self.history.len())
    }}

    pub fn stats(&self) -> String {{
        format!("{{}}:session={{}}_history={{}}", self.session_id, self.history.len())
    }}
}}

impl Default for {name_pascal}Agent {{
    fn default() -> Self {{
        Self::new()
    }}
}}

fn uuid_or_default() -> String {{
    // Placeholder — replace with actual UUID generation
    "agent-default-session".to_string()
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_{mod_name}_new() {{
        let agent = {name_pascal}Agent::new();
        assert_eq!(agent.history.len(), 0);
    }}

    #[test]
    fn test_{mod_name}_process_message() {{
        let mut agent = {name_pascal}Agent::new();
        let resp = agent.process_message("hello");
        assert!(resp.contains("ack_1"));
        assert_eq!(agent.history.len(), 1);
    }}

    #[test]
    fn test_{mod_name}_history_cap() {{
        let mut agent = {name_pascal}Agent {{
            config: {name_pascal}Config {{
                max_history: 2,
                ..Default::default()
            }},
            history: Vec::new(),
            session_id: "test".to_string(),
        }};
        agent.process_message("a");
        agent.process_message("b");
        agent.process_message("c");
        assert_eq!(agent.history.len(), 2);
        assert_eq!(agent.history[0].content, "b");
    }}
}}
"#,
            name_pascal = &name_pascal,
        );

        let handler_method = format!(
            r#"    /// {desc}
    pub fn handle_{mod_name}_tick(&mut self) -> String {{
        if self.{mod_name}.is_none() {{
            self.{mod_name} = Some({name_pascal}Agent::new());
            return format!("{mod_name}:init");
        }}
        let agent = self.{mod_name}.as_mut().unwrap();
        format!("{mod_name}:history={{}}", agent.history.len())
    }}"#,
            name_pascal = &name_pascal,
        );

        let dispatch_arm = format!(r#"            "{mod_name}" => self.handle_{mod_name}_tick(),"#);

        let field_decl = format!(
            r#"    /// {desc}
    pub {mod_name}: Option<{name_pascal}Agent>,"#,
            name_pascal = &name_pascal,
        );

        let construct_init = format!(r#"{mod_name}: None,"#);

        let import = format!(
            r#"use crate::core::nt_core_experience::{name_pascal}Agent;"#,
            name_pascal = &name_pascal,
        );

        log::info!(
            "{}",
            Self::section("AGENT MODULE — {mod_name}.rs (create file)")
        );
        log::info!("Path: core/nt_core_experience/{mod_name}.rs");
        log::info!("{agent_content}");
        log::info!("scaffold");

        log::info!(
            "{}",
            Self::section("STEP 1 — mod.rs declaration (add to core/nt_core_experience/mod.rs)")
        );
        log::info!("pub mod {mod_name};");
        log::info!("pub use {mod_name}::{}Agent;", name_pascal);
        log::info!("scaffold");

        log::info!("{}", Self::section("STEP 2 — Import (add to types.rs)"));
        log::info!("{import}");
        log::info!("scaffold");

        log::info!(
            "{}",
            Self::section(
                "STEP 3 — Field declaration (add to types.rs ConsciousnessIntegration struct)"
            )
        );
        log::info!("{field_decl}");
        log::info!("scaffold");

        log::info!(
            "{}",
            Self::section(
                "STEP 4 — Constructor init (add to types.rs ConsciousnessIntegration::new())"
            )
        );
        log::info!("{construct_init}");
        log::info!("scaffold");

        log::info!(
            "{}",
            Self::section("STEP 5 — Handler method (add to modules.rs)")
        );
        log::info!("{handler_method}");
        log::info!("scaffold");

        log::info!(
            "{}",
            Self::section("STEP 6 — Dispatch arm (add to modules.rs dispatch match)")
        );
        log::info!("{dispatch_arm}");
        log::info!("scaffold");

        let tier_reg = format!(r#"        registry.register("{mod_name}", LoadTier::Warm);"#);
        log::info!(
            "{}",
            Self::section("STEP 7 — Tier registration (add to handler_tier.rs)")
        );
        log::info!("{tier_reg}");
        log::info!("scaffold");

        let pipeline = format!(
            r#"        // {mod_name}
        let _ = self.dispatch_handler("{mod_name}");"#
        );
        log::info!(
            "{}",
            Self::section("STEP 8 — Pipeline call (add to core.rs phase_two or phase_three)")
        );
        log::info!("{pipeline}");
        log::info!("scaffold");

        let name_clone = mod_name.clone();
        let handler_names = format!(
            r#"            HandlerNode {{ name: "{name_clone}", interval_secs: 60, call_count: 0 }},"#
        );
        log::info!(
            "{}",
            Self::section("STEP 9 — SelfInspectable registration (add to self_inspect.rs)")
        );
        log::info!("{handler_names}");
        log::info!("scaffold");

        Self::print_file_hint(
            "Agent module",
            &format!("core/nt_core_experience/{mod_name}.rs"),
        );
    }

    fn to_pascal(s: &str) -> String {
        s.split('_')
            .map(|part| {
                let mut c = part.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().to_string() + c.as_str(),
                }
            })
            .collect()
    }

    fn section(label: &str) -> String {
        format!("\n━━━ {} ━━━", label)
    }

    fn print_file_hint(label: &str, path: &str) {
        log::info!("{}", Self::section("File location"));
        log::info!("  {label}: {path}");
    }
}
