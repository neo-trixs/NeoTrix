use nt_domain::*;

pub trait WorkspaceRepo: Send + Sync {
    fn create_workspace(&self, name: &str, owner_id: uuid::Uuid) -> Result<Workspace, String>;
    fn get_workspace(&self, id: uuid::Uuid) -> Option<Workspace>;
    fn list_workspaces(&self, user_id: uuid::Uuid) -> Vec<Workspace>;
    fn delete_workspace(&self, id: uuid::Uuid) -> bool;

    fn create_channel(&self, ws_id: uuid::Uuid, name: &str, is_public: bool) -> Result<Channel, String>;
    fn list_channels(&self, ws_id: uuid::Uuid) -> Vec<Channel>;

    fn create_task(&self, channel_id: uuid::Uuid, title: &str) -> Result<Task, String>;
    fn list_tasks(&self, channel_id: uuid::Uuid) -> Vec<Task>;
    fn update_task_status(&self, id: uuid::Uuid, status: TaskStatus) -> bool;

    fn create_document(&self, ws_id: uuid::Uuid, title: &str, content: &str, created_by: uuid::Uuid) -> Result<Document, String>;
    fn list_documents(&self, ws_id: uuid::Uuid) -> Vec<Document>;
}
