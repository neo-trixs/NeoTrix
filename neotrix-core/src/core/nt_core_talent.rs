use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// ─── Role Hierarchy ───

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Role {
    CEO,
    Director,
    Manager,
    Lead,
    SeniorIC,
    JuniorIC,
    Intern,
    Secretary,
}

impl Role {
    pub fn level(&self) -> u8 {
        match self {
            Role::CEO => 7,
            Role::Director => 6,
            Role::Manager => 5,
            Role::Lead => 4,
            Role::SeniorIC => 3,
            Role::JuniorIC => 2,
            Role::Intern => 1,
            Role::Secretary => 0,
        }
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::CEO => write!(f, "CEO"),
            Role::Director => write!(f, "Director"),
            Role::Manager => write!(f, "Manager"),
            Role::Lead => write!(f, "Lead"),
            Role::SeniorIC => write!(f, "SeniorIC"),
            Role::JuniorIC => write!(f, "JuniorIC"),
            Role::Intern => write!(f, "Intern"),
            Role::Secretary => write!(f, "Secretary"),
        }
    }
}

// ─── RoleDefinition ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleDefinition {
    pub id: String,
    pub title: String,
    pub role_type: Role,
    pub reporting_to: Option<String>,
    pub direct_reports: Vec<String>,
    pub specialist_type: Option<String>,
    pub skills: Vec<String>,
    pub allowed_tools: Vec<String>,
    pub budget_limit: Option<f64>,
    pub is_vacant: bool,
}

impl RoleDefinition {
    pub fn new(id: impl Into<String>, title: impl Into<String>, role_type: Role) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            role_type,
            reporting_to: None,
            direct_reports: Vec::new(),
            specialist_type: None,
            skills: Vec::new(),
            allowed_tools: Vec::new(),
            budget_limit: None,
            is_vacant: true,
        }
    }
}

// ─── TaskRecord ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRecord {
    pub task_id: String,
    pub task_type: String,
    pub outcome: f64,
    pub cost: f64,
    pub duration_ms: u64,
    pub timestamp: u64,
}

impl TaskRecord {
    pub fn new(
        task_id: impl Into<String>,
        task_type: impl Into<String>,
        outcome: f64,
        cost: f64,
        duration_ms: u64,
        timestamp: u64,
    ) -> Self {
        Self {
            task_id: task_id.into(),
            task_type: task_type.into(),
            outcome,
            cost,
            duration_ms,
            timestamp,
        }
    }
}

// ─── EmployeeProfile ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeProfile {
    pub id: String,
    pub name: String,
    pub role_id: String,
    pub experience_level: f64,
    pub skills: HashMap<String, f64>,
    pub performance_score: f64,
    pub task_history: Vec<TaskRecord>,
    pub specialization: Vec<String>,
    pub preferred_tools: Vec<String>,
    pub hire_date: u64,
    pub last_active: u64,
    pub lesson_count: u64,
    pub playbook_contributions: u64,
}

impl EmployeeProfile {
    pub fn new(id: impl Into<String>, name: impl Into<String>, role_id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            role_id: role_id.into(),
            experience_level: 0.0,
            skills: HashMap::new(),
            performance_score: 0.0,
            task_history: Vec::new(),
            specialization: Vec::new(),
            preferred_tools: Vec::new(),
            hire_date: 0,
            last_active: 0,
            lesson_count: 0,
            playbook_contributions: 0,
        }
    }

    pub fn skill_proficiency(&self, skill: &str) -> f64 {
        self.skills.get(skill).copied().unwrap_or(0.0)
    }

    pub fn add_skill(&mut self, skill: impl Into<String>, proficiency: f64) {
        let proficiency = proficiency.max(0.0).min(1.0);
        self.skills.insert(skill.into(), proficiency);
    }

    pub fn add_task(&mut self, record: TaskRecord) {
        self.task_history.push(record);
    }

    pub fn avg_outcome(&self) -> f64 {
        if self.task_history.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.task_history.iter().map(|t| t.outcome).sum();
        sum / self.task_history.len() as f64
    }
}

// ─── PlaybookEntry ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookEntry {
    pub id: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub author_id: String,
    pub promoted_count: u64,
    pub created_at: u64,
}

impl PlaybookEntry {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        content: impl Into<String>,
        author_id: impl Into<String>,
        created_at: u64,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            tags: Vec::new(),
            author_id: author_id.into(),
            promoted_count: 0,
            created_at,
        }
    }

    pub fn promote(&mut self) {
        self.promoted_count += 1;
    }
}

// ─── OrgChart ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgChart {
    pub name: String,
    pub id: String,
    pub roles: HashMap<String, RoleDefinition>,
    pub employees: HashMap<String, EmployeeProfile>,
    pub playbooks: Vec<PlaybookEntry>,
}

impl OrgChart {
    pub fn new(name: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            id: id.into(),
            roles: HashMap::new(),
            employees: HashMap::new(),
            playbooks: Vec::new(),
        }
    }

    pub fn add_role(&mut self, role: RoleDefinition) {
        let role_id = role.id.clone();
        if let Some(ref r) = role.reporting_to {
            if let Some(parent) = self.roles.get_mut(r.as_str()) {
                parent.direct_reports.push(role_id.clone());
            }
        }
        self.roles.insert(role_id, role);
    }

    pub fn remove_role(&mut self, role_id: &str) -> bool {
        if let Some(role) = self.roles.remove(role_id) {
            if let Some(ref r) = role.reporting_to {
                if let Some(parent) = self.roles.get_mut(r.as_str()) {
                    parent.direct_reports.retain(|id| id != role_id);
                }
            }
            for child_id in &role.direct_reports {
                if let Some(child) = self.roles.get_mut(child_id.as_str()) {
                    child.reporting_to = role.reporting_to.clone();
                }
            }
            true
        } else {
            false
        }
    }

    pub fn hire_employee(&mut self, employee: EmployeeProfile) -> bool {
        let role_id = employee.role_id.clone();
        if let Some(role) = self.roles.get_mut(&role_id) {
            if role.is_vacant {
                role.is_vacant = false;
                self.employees.insert(employee.id.clone(), employee);
                return true;
            }
        }
        false
    }

    pub fn fire_employee(&mut self, employee_id: &str) -> bool {
        if let Some(emp) = self.employees.remove(employee_id) {
            if let Some(role) = self.roles.get_mut(&emp.role_id) {
                role.is_vacant = true;
            }
            true
        } else {
            false
        }
    }

    pub fn get_subordinates(&self, role_id: &str) -> Vec<&RoleDefinition> {
        let mut result = Vec::new();
        if let Some(role) = self.roles.get(role_id) {
            for child_id in &role.direct_reports {
                if let Some(child) = self.roles.get(child_id.as_str()) {
                    result.push(child);
                }
            }
        }
        result
    }

    pub fn get_chain_of_command(&self, employee_id: &str) -> Vec<String> {
        let mut chain = Vec::new();
        let emp = match self.employees.get(employee_id) {
            Some(e) => e,
            None => return chain,
        };
        let mut current_role_id = Some(emp.role_id.as_str());
        while let Some(rid) = current_role_id {
            chain.push(rid.to_string());
            if let Some(role) = self.roles.get(rid) {
                current_role_id = role.reporting_to.as_deref();
            } else {
                break;
            }
        }
        chain
    }

    pub fn get_org_tree(&self) -> String {
        let mut lines = Vec::new();
        let top_roles: Vec<&RoleDefinition> = self
            .roles
            .values()
            .filter(|r| r.reporting_to.is_none())
            .collect();

        for (i, top) in top_roles.iter().enumerate() {
            let prefix = if i == top_roles.len() - 1 { "└── " } else { "├── " };
            self.build_tree(top, prefix, "", &mut lines, i == top_roles.len() - 1);
        }

        if lines.is_empty() {
            lines.push("(empty org chart)".to_string());
        }

        lines.join("\n")
    }

    fn build_tree(
        &self,
        role: &RoleDefinition,
        prefix: &str,
        indent: &str,
        lines: &mut Vec<String>,
        is_last: bool,
    ) {
        let filler = if is_last { "    " } else { "│   " };
        let occupant = if role.is_vacant {
            " [vacant]".to_string()
        } else {
            let occupant_names: Vec<&str> = self
                .employees
                .values()
                .filter(|e| e.role_id == role.id)
                .map(|e| e.name.as_str())
                .collect();
            if occupant_names.is_empty() {
                String::new()
            } else {
                format!(" ({})", occupant_names.join(", "))
            }
        };
        lines.push(format!("{}{}{}{}", indent, prefix, role.title, occupant));

        let children: Vec<&RoleDefinition> = role
            .direct_reports
            .iter()
            .filter_map(|cid| self.roles.get(cid.as_str()))
            .collect();

        for (i, child) in children.iter().enumerate() {
            let child_prefix = if i == children.len() - 1 {
                "└── "
            } else {
                "├── "
            };
            self.build_tree(child, child_prefix, &format!("{}{}", indent, filler), lines, i == children.len() - 1);
        }
    }

    pub fn vacant_roles(&self) -> Vec<&RoleDefinition> {
        self.roles.values().filter(|r| r.is_vacant).collect()
    }

    pub fn filled_roles(&self) -> Vec<&RoleDefinition> {
        self.roles.values().filter(|r| !r.is_vacant).collect()
    }

    pub fn add_playbook(&mut self, entry: PlaybookEntry) {
        self.playbooks.push(entry);
    }

    pub fn add_role_with_employee(
        &mut self,
        role: RoleDefinition,
        employee: EmployeeProfile,
    ) -> bool {
        let role_id = role.id.clone();
        let _emp_id = employee.id.clone();
        if self.roles.get(&role_id).is_some_and(|r| !r.is_vacant) {
            return false;
        }
        self.add_role(role);
        if self.hire_employee(employee) {
            true
        } else {
            self.roles.remove(&role_id);
            false
        }
    }
}

// ─── TalentMarket ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TalentMarket {
    pub templates: Vec<EmployeeProfile>,
}

impl TalentMarket {
    pub fn new() -> Self {
        Self {
            templates: Vec::new(),
        }
    }

    pub fn register(&mut self, profile: EmployeeProfile) {
        self.templates.push(profile);
    }

    pub fn search(&self, query: &str, tags: &[String]) -> Vec<&EmployeeProfile> {
        let query_lower = query.to_lowercase();
        self.templates
            .iter()
            .filter(|p| {
                let name_match = p.name.to_lowercase().contains(&query_lower);
                let spec_match = p
                    .specialization
                    .iter()
                    .any(|s| s.to_lowercase().contains(&query_lower));
                let skill_match = p.skills.keys().any(|s| s.to_lowercase().contains(&query_lower));
                let tag_match = if tags.is_empty() {
                    true
                } else {
                    tags.iter().any(|t| {
                        let t_low = t.to_lowercase();
                        p.specialization.iter().any(|s| s.to_lowercase() == t_low)
                            || p.skills.keys().any(|s| s.to_lowercase() == t_low)
                    })
                };
                (name_match || spec_match || skill_match) && tag_match
            })
            .collect()
    }

    pub fn filter_by_skill(&self, skill: &str) -> Vec<&EmployeeProfile> {
        let skill_lower = skill.to_lowercase();
        self.templates
            .iter()
            .filter(|p| p.skills.keys().any(|s| s.to_lowercase() == skill_lower))
            .collect()
    }

    pub fn import_profiles(&mut self, json_data: &str) -> Result<usize, String> {
        let profiles: Vec<EmployeeProfile> =
            serde_json::from_str(json_data).map_err(|e| format!("JSON parse error: {}", e))?;
        let count = profiles.len();
        self.templates.extend(profiles);
        Ok(count)
    }

    pub fn export_profiles(&self) -> String {
        serde_json::to_string_pretty(&self.templates).unwrap_or_else(|_| "[]".to_string())
    }

    pub fn import_from_agency_agents(&mut self, path: &str) -> Result<usize, String> {
        let data = std::fs::read_to_string(path).map_err(|e| format!("Read error: {}", e))?;
        self.import_profiles(&data)
    }
}

impl Default for TalentMarket {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_hierarchy_levels() {
        assert_eq!(Role::CEO.level(), 7);
        assert_eq!(Role::Director.level(), 6);
        assert_eq!(Role::Manager.level(), 5);
        assert_eq!(Role::Lead.level(), 4);
        assert_eq!(Role::SeniorIC.level(), 3);
        assert_eq!(Role::JuniorIC.level(), 2);
        assert_eq!(Role::Intern.level(), 1);
        assert_eq!(Role::Secretary.level(), 0);
    }

    #[test]
    fn test_org_chart_add_role_and_hire() {
        let mut org = OrgChart::new("TestOrg", "org-1");

        let ceo_role = RoleDefinition::new("r-ceo", "Chief Executive Officer", Role::CEO);
        let dev_role = RoleDefinition::new("r-dev", "Senior Engineer", Role::SeniorIC);

        org.add_role(ceo_role);
        org.add_role(dev_role);

        assert_eq!(org.roles.len(), 2);
        assert!(org.roles.contains_key("r-ceo"));
        assert!(org.roles.contains_key("r-dev"));

        let emp = EmployeeProfile::new("e-1", "Alice", "r-dev");
        let hired = org.hire_employee(emp);
        assert!(hired);
        assert!(!org.roles.get("r-dev").unwrap().is_vacant);
        assert!(org.employees.contains_key("e-1"));

        let dup_hired = org.hire_employee(EmployeeProfile::new("e-2", "Bob", "r-dev"));
        assert!(!dup_hired);

        let fired = org.fire_employee("e-1");
        assert!(fired);
        assert!(org.roles.get("r-dev").unwrap().is_vacant);
        assert!(!org.employees.contains_key("e-1"));
    }

    #[test]
    fn test_org_chain_of_command() {
        let mut org = OrgChart::new("Corp", "org-2");
        org.add_role(RoleDefinition::new("r-ceo", "CEO", Role::CEO));
        org.add_role(RoleDefinition::new("r-dir", "Director", Role::Director));
        org.add_role(RoleDefinition::new("r-mgr", "Manager", Role::Manager));
        org.add_role(RoleDefinition::new("r-ic", "Engineer", Role::SeniorIC));

        if let Some(ceo) = org.roles.get_mut("r-ceo") {
            ceo.direct_reports.push("r-dir".to_string());
        }
        if let Some(dir) = org.roles.get_mut("r-dir") {
            dir.reporting_to = Some("r-ceo".to_string());
            dir.direct_reports.push("r-mgr".to_string());
        }
        if let Some(mgr) = org.roles.get_mut("r-mgr") {
            mgr.reporting_to = Some("r-dir".to_string());
            mgr.direct_reports.push("r-ic".to_string());
        }
        if let Some(ic) = org.roles.get_mut("r-ic") {
            ic.reporting_to = Some("r-mgr".to_string());
        }

        let emp = EmployeeProfile::new("e-1", "Dave", "r-ic");
        org.hire_employee(emp);

        let chain = org.get_chain_of_command("e-1");
        assert_eq!(chain, vec!["r-ic", "r-mgr", "r-dir", "r-ceo"]);
    }

    #[test]
    fn test_org_tree_output() {
        let mut org = OrgChart::new("TreeOrg", "org-3");
        org.add_role(RoleDefinition::new("r-ceo", "CEO", Role::CEO));
        org.add_role(RoleDefinition::new("r-eng", "Engineering", Role::Director));
        org.add_role(RoleDefinition::new("r-dev", "Developer", Role::SeniorIC));

        if let Some(ceo) = org.roles.get_mut("r-ceo") {
            ceo.direct_reports.push("r-eng".to_string());
        }
        if let Some(eng) = org.roles.get_mut("r-eng") {
            eng.reporting_to = Some("r-ceo".to_string());
            eng.direct_reports.push("r-dev".to_string());
        }
        if let Some(dev) = org.roles.get_mut("r-dev") {
            dev.reporting_to = Some("r-eng".to_string());
        }

        let emp = EmployeeProfile::new("e-1", "Eve", "r-dev");
        org.hire_employee(emp);

        let tree = org.get_org_tree();
        assert!(tree.contains("CEO"));
        assert!(tree.contains("Engineering"));
        assert!(tree.contains("Developer"));
        assert!(tree.contains("Eve"));
        assert!(tree.contains("[vacant]"));
    }

    #[test]
    fn test_talent_market_import_and_search() {
        let mut market = TalentMarket::new();

        let mut emp1 = EmployeeProfile::new("t1", "Alice", "r-dev");
        emp1.skills.insert("Rust".to_string(), 0.9);
        emp1.skills.insert("Python".to_string(), 0.7);
        emp1.specialization.push("backend".to_string());

        let mut emp2 = EmployeeProfile::new("t2", "Bob", "r-ml");
        emp2.skills.insert("Python".to_string(), 0.95);
        emp2.specialization.push("machine-learning".to_string());

        market.register(emp1);
        market.register(emp2);

        let results = market.search("Rust", &[]);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "t1");

        let tags = vec!["machine-learning".to_string()];
        let ml_results = market.search("", &tags);
        assert_eq!(ml_results.len(), 1);
        assert_eq!(ml_results[0].id, "t2");

        let skill_results = market.filter_by_skill("Python");
        assert_eq!(skill_results.len(), 2);
    }

    #[test]
    fn test_employee_profile_skills() {
        let mut emp = EmployeeProfile::new("e1", "Charlie", "r-sr");
        emp.add_skill("Rust", 0.85);
        emp.add_skill("Python", 0.6);

        assert!((emp.skill_proficiency("Rust") - 0.85).abs() < 1e-6);
        assert!((emp.skill_proficiency("Python") - 0.6).abs() < 1e-6);
        assert!((emp.skill_proficiency("Go") - 0.0).abs() < 1e-6);

        emp.add_task(TaskRecord::new("t1", "bugfix", 0.8, 5.0, 1200, 1000));
        emp.add_task(TaskRecord::new("t2", "feature", 0.9, 10.0, 3000, 1001));

        assert!((emp.avg_outcome() - 0.85).abs() < 1e-6);

        let empty_emp = EmployeeProfile::new("e2", "Nobody", "r-intern");
        assert!((empty_emp.avg_outcome() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_vacant_and_filled_roles() {
        let mut org = OrgChart::new("Vancy", "org-4");
        org.add_role(RoleDefinition::new("r1", "Role 1", Role::Lead));
        org.add_role(RoleDefinition::new("r2", "Role 2", Role::JuniorIC));

        assert_eq!(org.vacant_roles().len(), 2);
        assert_eq!(org.filled_roles().len(), 0);

        org.hire_employee(EmployeeProfile::new("e1", "Test", "r1"));
        assert_eq!(org.vacant_roles().len(), 1);
        assert_eq!(org.filled_roles().len(), 1);
    }

    #[test]
    fn test_add_role_with_employee() {
        let mut org = OrgChart::new("Combo", "org-5");
        let role = RoleDefinition::new("r-lead", "Team Lead", Role::Lead);
        let emp = EmployeeProfile::new("e-lead", "Fay", "r-lead");

        assert!(org.add_role_with_employee(role, emp));
        assert!(!org.roles.get("r-lead").unwrap().is_vacant);
        assert!(org.employees.contains_key("e-lead"));

        let role2 = RoleDefinition::new("r-filled", "Filled Spot", Role::JuniorIC);
        let emp2 = EmployeeProfile::new("e-fill", "Gary", "r-filled");
        let emp3 = EmployeeProfile::new("e-oops", "Hank", "r-filled");

        assert!(org.add_role_with_employee(role2, emp2));
        assert!(!org.add_role_with_employee(RoleDefinition::new("r-filled", "", Role::JuniorIC), emp3));
    }

    #[test]
    fn test_remove_role_reassigns_reports() {
        let mut org = OrgChart::new("Reorg", "org-6");
        org.add_role(RoleDefinition::new("r-ceo", "CEO", Role::CEO));
        org.add_role(RoleDefinition::new("r-mgr", "Manager", Role::Manager));
        org.add_role(RoleDefinition::new("r-dev", "Dev", Role::SeniorIC));

        if let Some(ceo) = org.roles.get_mut("r-ceo") {
            ceo.direct_reports.push("r-mgr".to_string());
        }
        if let Some(mgr) = org.roles.get_mut("r-mgr") {
            mgr.reporting_to = Some("r-ceo".to_string());
            mgr.direct_reports.push("r-dev".to_string());
        }
        if let Some(dev) = org.roles.get_mut("r-dev") {
            dev.reporting_to = Some("r-mgr".to_string());
        }

        assert!(org.remove_role("r-mgr"));
        assert!(!org.roles.contains_key("r-mgr"));
        assert_eq!(
            org.roles.get("r-dev").unwrap().reporting_to.as_deref(),
            Some("r-ceo")
        );
    }
}
