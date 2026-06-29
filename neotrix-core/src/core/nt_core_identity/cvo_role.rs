#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HumanRole {
    Developer,
    Designer,
    Manager,
    Analyst,
    Viewer,
}

#[derive(Debug, Clone)]
pub struct CvoRoleModel {
    pub current_role: HumanRole,
    pub interaction_mode: String,
}

impl CvoRoleModel {
    pub fn new(role: HumanRole) -> Self {
        Self {
            current_role: role,
            interaction_mode: "direct".into(),
        }
    }
    pub fn switch_role(&mut self, role: HumanRole) {
        self.current_role = role;
    }
    pub fn mode_for_role(&self) -> &str {
        match self.current_role {
            HumanRole::Developer => "technical",
            HumanRole::Designer => "visual",
            HumanRole::Manager => "strategic",
            HumanRole::Analyst => "analytical",
            HumanRole::Viewer => "passive",
        }
    }
}
