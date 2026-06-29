use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chrono::Utc;
use nt_domain::*;
use uuid::Uuid;

pub struct OrgService {
    orgs: Arc<Mutex<HashMap<Uuid, Org>>>,
    teams: Arc<Mutex<HashMap<Uuid, Team>>>,
    projects: Arc<Mutex<HashMap<Uuid, Project>>>,
    members: Arc<Mutex<Vec<Member>>>,
}

impl OrgService {
    pub fn new() -> Self {
        Self {
            orgs: Arc::new(Mutex::new(HashMap::new())),
            teams: Arc::new(Mutex::new(HashMap::new())),
            projects: Arc::new(Mutex::new(HashMap::new())),
            members: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn create_org(&self, name: &str) -> Result<Org, String> {
        if name.is_empty() {
            return Err("Org name cannot be empty".to_string());
        }
        let org = Org {
            id: Uuid::new_v4(),
            name: name.to_string(),
            domain: None,
            created_at: Utc::now(),
        };
        self.orgs.lock().unwrap().insert(org.id, org.clone());
        Ok(org)
    }

    pub fn get_org(&self, id: Uuid) -> Result<Org, String> {
        self.orgs.lock().unwrap().get(&id).cloned().ok_or_else(|| "Org not found".to_string())
    }

    pub fn create_team(&self, org_id: Uuid, name: &str, lead_id: Uuid) -> Result<Team, String> {
        if name.is_empty() {
            return Err("Team name cannot be empty".to_string());
        }
        self.get_org(org_id)?;
        let team = Team {
            id: Uuid::new_v4(),
            org_id,
            name: name.to_string(),
            lead_id,
            created_at: Utc::now(),
        };
        self.teams.lock().unwrap().insert(team.id, team.clone());
        Ok(team)
    }

    pub fn list_teams(&self, org_id: Uuid) -> Vec<Team> {
        self.teams.lock().unwrap().values().filter(|t| t.org_id == org_id).cloned().collect()
    }

    pub fn create_project(&self, team_id: Uuid, name: &str) -> Result<Project, String> {
        if name.is_empty() {
            return Err("Project name cannot be empty".to_string());
        }
        {
            let teams = self.teams.lock().unwrap();
            if !teams.contains_key(&team_id) {
                return Err("Team not found".to_string());
            }
        }
        let project = Project {
            id: Uuid::new_v4(),
            team_id,
            name: name.to_string(),
            workspace_ids: vec![],
            created_at: Utc::now(),
        };
        self.projects.lock().unwrap().insert(project.id, project.clone());
        Ok(project)
    }

    pub fn list_projects(&self, team_id: Uuid) -> Vec<Project> {
        self.projects.lock().unwrap().values().filter(|p| p.team_id == team_id).cloned().collect()
    }

    pub fn add_member(&self, org_id: Uuid, user_id: Uuid, role: OrgRole) -> Result<Member, String> {
        self.get_org(org_id)?;
        let member = Member {
            id: Uuid::new_v4(),
            org_id,
            user_id,
            role,
            created_at: Utc::now(),
        };
        self.members.lock().unwrap().push(member.clone());
        Ok(member)
    }

    pub fn list_members(&self, org_id: Uuid) -> Vec<Member> {
        self.members.lock().unwrap().iter().filter(|m| m.org_id == org_id).cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_org() {
        let svc = OrgService::new();
        let org = svc.create_org("acme").unwrap();
        assert_eq!(org.name, "acme");
    }

    #[test]
    fn test_create_org_empty_name() {
        let svc = OrgService::new();
        assert!(svc.create_org("").is_err());
    }

    #[test]
    fn test_get_org() {
        let svc = OrgService::new();
        let org = svc.create_org("acme").unwrap();
        assert_eq!(svc.get_org(org.id).unwrap().name, "acme");
    }

    #[test]
    fn test_get_org_not_found() {
        let svc = OrgService::new();
        assert!(svc.get_org(Uuid::new_v4()).is_err());
    }

    #[test]
    fn test_create_team() {
        let svc = OrgService::new();
        let org = svc.create_org("acme").unwrap();
        let team = svc.create_team(org.id, "core", Uuid::new_v4()).unwrap();
        assert_eq!(team.name, "core");
    }

    #[test]
    fn test_create_team_invalid_org() {
        let svc = OrgService::new();
        assert!(svc.create_team(Uuid::new_v4(), "core", Uuid::new_v4()).is_err());
    }

    #[test]
    fn test_list_teams() {
        let svc = OrgService::new();
        let org = svc.create_org("acme").unwrap();
        svc.create_team(org.id, "t1", Uuid::new_v4()).unwrap();
        svc.create_team(org.id, "t2", Uuid::new_v4()).unwrap();
        assert_eq!(svc.list_teams(org.id).len(), 2);
    }

    #[test]
    fn test_project_lifecycle() {
        let svc = OrgService::new();
        let org = svc.create_org("acme").unwrap();
        let team = svc.create_team(org.id, "core", Uuid::new_v4()).unwrap();
        let proj = svc.create_project(team.id, "project-x").unwrap();
        assert_eq!(proj.name, "project-x");
        assert_eq!(svc.list_projects(team.id).len(), 1);
    }

    #[test]
    fn test_membership() {
        let svc = OrgService::new();
        let org = svc.create_org("acme").unwrap();
        let user = Uuid::new_v4();
        svc.add_member(org.id, user, OrgRole::Admin).unwrap();
        assert_eq!(svc.list_members(org.id).len(), 1);
        assert_eq!(svc.list_members(org.id)[0].role, OrgRole::Admin);
    }
}
