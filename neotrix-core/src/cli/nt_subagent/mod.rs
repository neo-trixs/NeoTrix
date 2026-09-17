//! Stub — deleted module, kept for CLI compilation only.

pub struct SubAgentDef { pub name: String, pub e8_mode: u8 }

pub struct SubAgentRegistry { agents: Vec<SubAgentDef> }
impl SubAgentRegistry {
    pub fn new() -> Self { Self { agents: Vec::new() } }
    pub fn scan_all(&mut self) {}
    pub fn get(&self, _name: &str) -> Option<&SubAgentDef> { None }
    pub fn list(&self) -> &[SubAgentDef] { &self.agents }
}

pub struct SubAgentDefParser;
pub struct PermissionMatrix;
