use nt_domain::*;

pub trait OrgRepo: Send + Sync {
    fn create_org(&self, name: &str) -> Result<Org, String>;
    fn get_org(&self, id: uuid::Uuid) -> Option<Org>;
    fn create_team(&self, org_id: uuid::Uuid, name: &str, lead_id: uuid::Uuid) -> Result<Team, String>;
    fn list_teams(&self, org_id: uuid::Uuid) -> Vec<Team>;
    fn create_project(&self, team_id: uuid::Uuid, name: &str) -> Result<Project, String>;
    fn list_projects(&self, team_id: uuid::Uuid) -> Vec<Project>;
    fn add_member(&self, org_id: uuid::Uuid, user_id: uuid::Uuid, role: OrgRole) -> Result<Member, String>;
    fn list_members(&self, org_id: uuid::Uuid) -> Vec<Member>;
}
