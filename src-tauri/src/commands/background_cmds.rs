use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

const TASKS_FILE: &str = ".neotrix/background-tasks.json";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BackgroundTask {
    pub id: String,
    pub name: String,
    pub prompt: String,
    pub schedule: String,
    pub last_run: Option<i64>,
    pub next_run: Option<i64>,
    pub status: TaskStatus,
    pub runs: Vec<TaskRun>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TaskRun {
    pub timestamp: i64,
    pub summary: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[derive(Default)]
pub enum TaskStatus {
    #[default]
    Idle,
    Running,
    Paused,
    Error,
}


impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Idle => write!(f, "idle"),
            TaskStatus::Running => write!(f, "running"),
            TaskStatus::Paused => write!(f, "paused"),
            TaskStatus::Error => write!(f, "error"),
        }
    }
}

fn tasks_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    format!("{}/{}", home, TASKS_FILE)
}

fn load_tasks() -> Vec<BackgroundTask> {
    let path = tasks_path();
    if !Path::new(&path).exists() {
        return Vec::new();
    }
    fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

fn save_tasks(tasks: &[BackgroundTask]) {
    let path = tasks_path();
    if let Some(parent) = Path::new(&path).parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(tasks) {
        let _ = fs::write(&path, json);
    }
}

#[tauri::command]
pub fn list_background_tasks() -> Vec<BackgroundTask> {
    load_tasks()
}

#[tauri::command]
pub fn create_background_task(name: String, prompt: String, schedule: String) -> BackgroundTask {
    let tasks = load_tasks();
    let new_task = BackgroundTask {
        id: format!("task-{}", tasks.len() + 1),
        name,
        prompt,
        schedule,
        last_run: None,
        next_run: None,
        status: TaskStatus::Idle,
        runs: Vec::new(),
    };
    let mut tasks = tasks;
    tasks.push(new_task.clone());
    save_tasks(&tasks);
    new_task
}

#[tauri::command]
pub fn pause_background_task(id: String) -> Result<(), String> {
    let mut tasks = load_tasks();
    if let Some(task) = tasks.iter_mut().find(|t| t.id == id) {
        task.status = TaskStatus::Paused;
        save_tasks(&tasks);
        Ok(())
    } else {
        Err(format!("Task {} not found", id))
    }
}

#[tauri::command]
pub fn resume_background_task(id: String) -> Result<(), String> {
    let mut tasks = load_tasks();
    if let Some(task) = tasks.iter_mut().find(|t| t.id == id) {
        task.status = TaskStatus::Idle;
        save_tasks(&tasks);
        Ok(())
    } else {
        Err(format!("Task {} not found", id))
    }
}

#[tauri::command]
pub fn delete_background_task(id: String) -> Result<(), String> {
    let mut tasks = load_tasks();
    tasks.retain(|t| t.id != id);
    save_tasks(&tasks);
    Ok(())
}

#[tauri::command]
pub fn run_background_task_now(id: String) -> Result<String, String> {
    let mut tasks = load_tasks();
    if let Some(task) = tasks.iter_mut().find(|t| t.id == id) {
        task.status = TaskStatus::Running;
        let result = format!("Executed: {} at scheduled time", task.name);
        task.runs.push(TaskRun {
            timestamp: chrono::offset::Local::now().timestamp(),
            summary: result.clone(),
        });
        task.status = TaskStatus::Idle;
        task.last_run = Some(chrono::offset::Local::now().timestamp());
        save_tasks(&tasks);
        Ok(result)
    } else {
        Err(format!("Task {} not found", id))
    }
}

#[tauri::command]
pub fn get_background_task_log(id: String) -> Result<Vec<TaskRun>, String> {
    let tasks = load_tasks();
    if let Some(task) = tasks.iter().find(|t| t.id == id) {
        Ok(task.runs.clone())
    } else {
        Err(format!("Task {} not found", id))
    }
}
