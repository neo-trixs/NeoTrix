use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkerAgent {
    pub id: String,
    pub task: String,
    pub status: String,
    pub progress: f64,
    pub created_at: u64,
}

#[derive(Serialize, Clone, Debug)]
pub struct CoordinatorStatus {
    pub active_workers: usize,
    pub max_workers: usize,
    pub strategy: String,
    pub workers: Vec<WorkerAgent>,
}

#[derive(Serialize, Clone, Debug)]
pub struct CoordinatorResult {
    pub worker_id: String,
    pub status: String,
    pub output: String,
}

static COORDINATOR: std::sync::LazyLock<Mutex<CoordinatorState>> =
    std::sync::LazyLock::new(|| Mutex::new(CoordinatorState::default()));

#[derive(Default)]
struct CoordinatorState {
    workers: Vec<WorkerAgent>,
    max_workers: usize,
    strategy: String,
    counter: u64,
}

#[tauri::command]
pub fn coordinator_spawn(task: String) -> Result<CoordinatorResult, String> {
    let mut state = COORDINATOR.lock().map_err(|e| e.to_string())?;

    if state.workers.len() >= state.max_workers {
        return Err("Max workers reached. Wait for some to complete.".into());
    }

    state.counter += 1;
    let id = format!("worker-{:04}", state.counter);
    let worker = WorkerAgent {
        id: id.clone(),
        task: task.clone(),
        status: "running".into(),
        progress: 0.0,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
    };
    state.workers.push(worker);

    Ok(CoordinatorResult {
        worker_id: id,
        status: "spawned".into(),
        output: format!("Worker spawned for task: {}", task),
    })
}

#[tauri::command]
pub fn coordinator_list() -> Result<CoordinatorStatus, String> {
    let state = COORDINATOR.lock().map_err(|e| e.to_string())?;
    Ok(CoordinatorStatus {
        active_workers: state.workers.iter().filter(|w| w.status == "running").count(),
        max_workers: state.max_workers,
        strategy: state.strategy.clone(),
        workers: state.workers.clone(),
    })
}

#[tauri::command]
pub fn coordinator_update(worker_id: String, progress: f64, status: String) -> Result<(), String> {
    let mut state = COORDINATOR.lock().map_err(|e| e.to_string())?;
    if let Some(w) = state.workers.iter_mut().find(|w| w.id == worker_id) {
        w.progress = progress;
        w.status = status;
        Ok(())
    } else {
        Err(format!("Worker {} not found", worker_id))
    }
}

#[tauri::command]
pub fn coordinator_remove(worker_id: String) -> Result<(), String> {
    let mut state = COORDINATOR.lock().map_err(|e| e.to_string())?;
    let len = state.workers.len();
    state.workers.retain(|w| w.id != worker_id);
    if state.workers.len() < len {
        Ok(())
    } else {
        Err(format!("Worker {} not found", worker_id))
    }
}

#[tauri::command]
pub fn coordinator_set_max_workers(max: usize) -> Result<(), String> {
    let mut state = COORDINATOR.lock().map_err(|e| e.to_string())?;
    state.max_workers = max;
    Ok(())
}

#[tauri::command]
pub fn coordinator_set_strategy(strategy: String) -> Result<(), String> {
    let mut state = COORDINATOR.lock().map_err(|e| e.to_string())?;
    state.strategy = strategy;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_and_list() {
        let _ = coordinator_remove("test".into());

        let result = coordinator_spawn("refactor core".into());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, "spawned");

        let list = coordinator_list().unwrap();
        assert!(list.active_workers >= 1);
    }

    #[test]
    fn test_update_progress() {
        let result = coordinator_spawn("test-task".into()).unwrap();
        assert!(coordinator_update(result.worker_id.clone(), 0.5, "running".into()).is_ok());

        let list = coordinator_list().unwrap();
        let w = list.workers.iter().find(|w| w.id == result.worker_id).unwrap();
        assert!((w.progress - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_set_max_workers() {
        assert!(coordinator_set_max_workers(5).is_ok());
        let list = coordinator_list().unwrap();
        assert_eq!(list.max_workers, 5);
    }

    #[test]
    fn test_remove_worker() {
        let result = coordinator_spawn("removable".into()).unwrap();
        assert!(coordinator_remove(result.worker_id.clone()).is_ok());
        assert!(coordinator_remove("nonexistent".into()).is_err());
    }
}
