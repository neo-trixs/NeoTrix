//! Runtime - TUI 应用运行时

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, watch};

/// 应用运行器
pub struct AppRunner {
    state: Arc<Mutex<crate::cli::tui::app::state::AppState>>,
    action_tx: tokio::sync::mpsc::Sender<crate::cli::tui::app::actions::Action>,
    action_rx: tokio::sync::mpsc::Receiver<crate::cli::tui::app::actions::Action>,
    effect_tx: tokio::sync::mpsc::Sender<crate::cli::tui::app::effects::Effect>,
    effect_rx: tokio::sync::mpsc::Receiver<crate::cli::tui::app::effects::Effect>,
    event_bus: Arc<crate::cli::tui::app::events::EventBus>,
    handles: Vec<tokio::task::JoinHandle<()>>,
    shutdown_tx: watch::Sender<bool>,
    shutdown_rx: watch::Receiver<bool>,
}

impl AppRunner {
    pub fn new(_initial_state: crate::cli::tui::app::state::AppState) -> Self {
        let (action_tx, action_rx) = tokio::sync::mpsc::channel(1000);
        let (effect_tx, effect_rx) = tokio::sync::mpsc::channel(1000);
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        
        Self {
            state: Arc::new(tokio::sync::Mutex::new(
                crate::cli::tui::app::state::AppState::default()
            )),
            action_tx,
            action_rx,
            effect_tx,
            effect_rx,
            event_bus: Arc::new(crate::cli::tui::app::events::EventBus::new()),
            handles: Vec::new(),
            shutdown_tx,
        }
    }

    /// 启动应用
    pub async fn run(mut self) -> Result<(), String> {
        // 启动动作处理循环
        let state = self.state.clone();
        let mut action_rx = self.action_rx;
        let _effect_tx = self.effect_tx.clone();
        let effect_rx = self.effect_rx;
        
        let _action_handle = tokio::spawn(async move {
            while let Some(action) = action_rx.recv().await {
                let mut state = state.lock().await;
                let new_state = crate::cli::tui::app::actions::reducer::reduce(state.clone(), action);
                *state = new_state;
            }
        });
        
        // 启动 effect 处理循环
        let _effect_handle = tokio::spawn(async move {
            let mut rx = effect_rx;
            while let Some(effect) = rx.recv().await {
                // Handle effects here
                let _ = effect;
            }
        });
        
        // 等待关闭信号
        self.shutdown_rx.changed().await.ok();
        
        Ok(())
    }

    /// 发送动作
    pub fn dispatch(&self, action: crate::cli::tui::app::actions::Action) -> Result<(), String> {
        self.action_tx.try_send(action).map_err(|e| e.to_string())
    }

    /// 发送副作用
    pub fn send_effect(&self, effect: crate::cli::tui::app::effects::Effect) {
        let _ = self.effect_tx.try_send(effect);
    }

    /// 获取状态快照
    pub async fn state_snapshot(&self) -> crate::cli::tui::app::state::AppState {
        self.state.lock().await.clone()
    }

    /// 关闭运行时
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(true);
    }
}

impl Default for crate::cli::tui::app::runtime::AppRunner {
    fn default() -> Self {
        Self::new(crate::cli::tui::app::state::AppState::default())
    }
}

/// 调度器 - 任务调度器
pub struct Scheduler {
    tasks: Vec<tokio::task::JoinHandle<()>>,
    interval_tasks: std::collections::HashMap<String, tokio::task::JoinHandle<()>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            interval_tasks: HashMap::new(),
        }
    }

    pub fn spawn<F>(&mut self, future: F) -> tokio::task::JoinHandle<()>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let handle = tokio::spawn(future);
        self.tasks.push(handle);
        handle
    }

    pub fn spawn_interval<F, Fut>(&mut self, name: String, interval: std::time::Duration, f: F)
    where
        F: Fn() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let handle = tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            loop {
                tokio::select! {
                    _ = interval_timer.tick() => {
                        f().await;
                    }
                }
            }
        });
        self.interval_tasks.insert(name, handle);
    }

    pub async fn shutdown(&mut self) {
        for handle in self.tasks.drain(..) {
            handle.abort();
        }
        for (_, handle) in self.interval_tasks.drain() {
            handle.abort();
        }
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// 生命周期钩子
pub struct LifecycleHooks {
    on_mount: Vec<Box<dyn FnOnce() + Send + 'static>>,
    on_unmount: Vec<Box<dyn FnOnce() + Send + 'static>>,
    on_update: Vec<Box<dyn Fn() + Send + 'static>>,
}

impl LifecycleHooks {
    pub fn new() -> Self {
        Self {
            on_mount: Vec::new(),
            on_unmount: Vec::new(),
            on_update: Vec::new(),
        }
    }

    pub fn on_mount<F: FnOnce() + Send + 'static>(&mut self, f: F) {
        self.on_mount.push(Box::new(f));
    }

    pub fn on_unmount<F: FnOnce() + Send + 'static>(&mut self, f: F) {
        self.on_unmount.push(Box::new(f));
    }

    pub fn on_update<F: Fn() + Send + 'static>(&mut self, f: F) {
        self.on_update.push(Box::new(f));
    }

    pub fn run_on_mount(&mut self) {
        for hook in self.on_mount.drain(..) {
            hook();
        }
    }

    pub fn run_on_unmount(&mut self) {
        for hook in self.on_unmount.drain(..) {
            hook();
        }
    }

    pub fn run_on_update(&mut self) {
        for hook in &self.on_update {
            hook();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler() {
        let mut scheduler = Scheduler::new();
        let handle = scheduler.spawn(async {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        });
        handle.await.unwrap();
    }
}
