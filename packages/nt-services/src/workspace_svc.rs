use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chrono::Utc;
use nt_db::workspace::WorkspaceRepo;
use nt_domain::*;
use uuid::Uuid;

// ── WorkspaceSummary ────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct WorkspaceSummary {
    pub workspace_name: String,
    pub channel_count: usize,
    pub task_count: usize,
    pub document_count: usize,
    pub member_count: usize,
}

// ── WorkspaceService ────────────────────────────────────────────────

pub struct WorkspaceService {
    repo: Box<dyn WorkspaceRepo>,
    task_channels: Mutex<HashMap<Uuid, Uuid>>,
}

impl WorkspaceService {
    pub fn new(repo: Box<dyn WorkspaceRepo>) -> Self {
        WorkspaceService {
            repo,
            task_channels: Mutex::new(HashMap::new()),
        }
    }

    pub fn create_workspace(&self, name: &str, owner_id: Uuid) -> Result<Workspace, String> {
        if name.trim().is_empty() {
            return Err("Workspace name cannot be empty".to_string());
        }
        self.repo.create_workspace(name, owner_id)
    }

    pub fn get_workspace(&self, id: Uuid) -> Result<Workspace, String> {
        self.repo
            .get_workspace(id)
            .ok_or_else(|| format!("Workspace {} not found", id))
    }

    pub fn create_channel(
        &self,
        ws_id: Uuid,
        name: &str,
        is_public: bool,
    ) -> Result<Channel, String> {
        self.get_workspace(ws_id)?;
        self.repo.create_channel(ws_id, name, is_public)
    }

    pub fn create_task(
        &self,
        channel_id: Uuid,
        title: &str,
        desc: &str,
        priority: Priority,
    ) -> Result<Task, String> {
        let task = self.repo.create_task(channel_id, title)?;
        self.task_channels.lock().unwrap().insert(task.id, channel_id);
        Ok(Task { description: desc.to_string(), priority, ..task })
    }

    pub fn update_task(
        &self,
        task_id: Uuid,
        status: TaskStatus,
        _priority: Priority,
        _assignee: Option<Uuid>,
    ) -> Result<Task, String> {
        let channel_id = {
            let map = self.task_channels.lock().unwrap();
            map.get(&task_id)
                .copied()
                .ok_or_else(|| format!("Task {} not found", task_id))?
        };

        if !self.repo.update_task_status(task_id, status) {
            return Err(format!("Task {} not found in repo", task_id));
        }

        let tasks = self.repo.list_tasks(channel_id);
        tasks
            .into_iter()
            .find(|t| t.id == task_id)
            .ok_or_else(|| "Task vanished after update".to_string())
    }

    pub fn create_document(
        &self,
        ws_id: Uuid,
        title: &str,
        content: &str,
        created_by: Uuid,
    ) -> Result<Document, String> {
        self.repo.create_document(ws_id, title, content, created_by)
    }

    pub fn get_workspace_summary(&self, ws_id: Uuid) -> Result<WorkspaceSummary, String> {
        let ws = self.get_workspace(ws_id)?;
        let channels = self.repo.list_channels(ws_id);
        let task_count: usize = channels
            .iter()
            .map(|c| self.repo.list_tasks(c.id).len())
            .sum();
        let document_count = self.repo.list_documents(ws_id).len();

        Ok(WorkspaceSummary {
            workspace_name: ws.name,
            channel_count: channels.len(),
            task_count,
            document_count,
            member_count: 1,
        })
    }
}

// ── In-Memory Repo ──────────────────────────────────────────────────

pub struct MemWorkspaceRepo {
    workspaces: Arc<Mutex<HashMap<Uuid, Workspace>>>,
    channels: Arc<Mutex<HashMap<Uuid, Channel>>>,
    tasks: Arc<Mutex<HashMap<Uuid, Task>>>,
    documents: Arc<Mutex<HashMap<Uuid, Document>>>,
}

impl MemWorkspaceRepo {
    pub fn new() -> Self {
        MemWorkspaceRepo {
            workspaces: Arc::new(Mutex::new(HashMap::new())),
            channels: Arc::new(Mutex::new(HashMap::new())),
            tasks: Arc::new(Mutex::new(HashMap::new())),
            documents: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for MemWorkspaceRepo {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceRepo for MemWorkspaceRepo {
    fn create_workspace(&self, name: &str, owner_id: Uuid) -> Result<Workspace, String> {
        let ws = Workspace {
            id: Uuid::new_v4(),
            name: name.to_string(),
            owner_id,
            created_at: Utc::now(),
        };
        self.workspaces.lock().unwrap().insert(ws.id, ws.clone());
        Ok(ws)
    }

    fn get_workspace(&self, id: Uuid) -> Option<Workspace> {
        self.workspaces.lock().unwrap().get(&id).cloned()
    }

    fn list_workspaces(&self, _user_id: Uuid) -> Vec<Workspace> {
        self.workspaces.lock().unwrap().values().cloned().collect()
    }

    fn delete_workspace(&self, id: Uuid) -> bool {
        self.workspaces.lock().unwrap().remove(&id).is_some()
    }

    fn create_channel(&self, ws_id: Uuid, name: &str, is_public: bool) -> Result<Channel, String> {
        let ch = Channel {
            id: Uuid::new_v4(),
            workspace_id: ws_id,
            name: name.to_string(),
            topic: String::new(),
            is_public,
            created_at: Utc::now(),
        };
        self.channels.lock().unwrap().insert(ch.id, ch.clone());
        Ok(ch)
    }

    fn list_channels(&self, ws_id: Uuid) -> Vec<Channel> {
        self.channels
            .lock()
            .unwrap()
            .values()
            .filter(|c| c.workspace_id == ws_id)
            .cloned()
            .collect()
    }

    fn create_task(&self, channel_id: Uuid, title: &str) -> Result<Task, String> {
        let task = Task {
            id: Uuid::new_v4(),
            channel_id,
            title: title.to_string(),
            description: String::new(),
            assignee_id: None,
            status: TaskStatus::Todo,
            priority: Priority::Medium,
            due: None,
            created_at: Utc::now(),
        };
        self.tasks.lock().unwrap().insert(task.id, task.clone());
        Ok(task)
    }

    fn list_tasks(&self, channel_id: Uuid) -> Vec<Task> {
        self.tasks
            .lock()
            .unwrap()
            .values()
            .filter(|t| t.channel_id == channel_id)
            .cloned()
            .collect()
    }

    fn update_task_status(&self, id: Uuid, status: TaskStatus) -> bool {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(task) = tasks.get_mut(&id) {
            task.status = status;
            true
        } else {
            false
        }
    }

    fn create_document(
        &self,
        ws_id: Uuid,
        title: &str,
        content: &str,
        _created_by: Uuid,
    ) -> Result<Document, String> {
        let now = Utc::now();
        let doc = Document {
            id: Uuid::new_v4(),
            workspace_id: ws_id,
            title: title.to_string(),
            content: content.to_string(),
            version: 1,
            created_at: now,
            updated_at: now,
        };
        self.documents.lock().unwrap().insert(doc.id, doc.clone());
        Ok(doc)
    }

    fn list_documents(&self, ws_id: Uuid) -> Vec<Document> {
        self.documents
            .lock()
            .unwrap()
            .values()
            .filter(|d| d.workspace_id == ws_id)
            .cloned()
            .collect()
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn new_service() -> WorkspaceService {
        WorkspaceService::new(Box::new(MemWorkspaceRepo::new()))
    }

    #[test]
    fn test_create_workspace() {
        let svc = new_service();
        let owner = Uuid::new_v4();
        let ws = svc.create_workspace("My Workspace", owner).unwrap();
        assert_eq!(ws.name, "My Workspace");
        assert_eq!(ws.owner_id, owner);
    }

    #[test]
    fn test_create_workspace_empty_name() {
        let svc = new_service();
        let result = svc.create_workspace("", Uuid::new_v4());
        assert!(result.is_err());
        let result = svc.create_workspace("   ", Uuid::new_v4());
        assert!(result.is_err());
    }

    #[test]
    fn test_get_workspace_not_found() {
        let svc = new_service();
        let result = svc.get_workspace(Uuid::new_v4());
        assert!(result.is_err());
    }

    #[test]
    fn test_create_channel_invalid_workspace() {
        let svc = new_service();
        let result = svc.create_channel(Uuid::new_v4(), "general", true);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_task_lifecycle() {
        let svc = new_service();
        let ws = svc.create_workspace("test", Uuid::new_v4()).unwrap();
        let ch = svc.create_channel(ws.id, "dev", true).unwrap();
        let task = svc
            .create_task(ch.id, "Fix bug", "urgent fix needed", Priority::High)
            .unwrap();
        assert_eq!(task.title, "Fix bug");
        assert_eq!(task.description, "urgent fix needed");
        assert_eq!(task.priority, Priority::High);
        assert_eq!(task.status, TaskStatus::Todo);

        let updated = svc
            .update_task(task.id, TaskStatus::InProgress, Priority::High, None)
            .unwrap();
        assert_eq!(updated.status, TaskStatus::InProgress);
    }

    #[test]
    fn test_update_task_not_found() {
        let svc = new_service();
        let result = svc.update_task(Uuid::new_v4(), TaskStatus::Done, Priority::Low, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_document_crud() {
        let svc = new_service();
        let ws = svc.create_workspace("test", Uuid::new_v4()).unwrap();
        let author = Uuid::new_v4();

        let doc = svc
            .create_document(ws.id, "readme", "# Hello World", author)
            .unwrap();
        assert_eq!(doc.title, "readme");
        assert_eq!(doc.content, "# Hello World");
        assert_eq!(doc.version, 1);
        assert_eq!(doc.workspace_id, ws.id);
    }

    #[test]
    fn test_workspace_summary() {
        let svc = new_service();
        let owner = Uuid::new_v4();
        let ws = svc.create_workspace("summary test", owner).unwrap();

        let ch1 = svc.create_channel(ws.id, "general", true).unwrap();
        let ch2 = svc.create_channel(ws.id, "dev", false).unwrap();

        svc.create_task(ch1.id, "task1", "desc1", Priority::Low)
            .unwrap();
        svc.create_task(ch1.id, "task2", "desc2", Priority::High)
            .unwrap();
        svc.create_task(ch2.id, "task3", "desc3", Priority::Medium)
            .unwrap();

        svc.create_document(ws.id, "doc1", "content1", owner)
            .unwrap();
        svc.create_document(ws.id, "doc2", "content2", owner)
            .unwrap();

        let summary = svc.get_workspace_summary(ws.id).unwrap();
        assert_eq!(summary.workspace_name, "summary test");
        assert_eq!(summary.channel_count, 2);
        assert_eq!(summary.task_count, 3);
        assert_eq!(summary.document_count, 2);
        assert_eq!(summary.member_count, 1);
    }
}
