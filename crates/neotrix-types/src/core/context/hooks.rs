use std::time::Duration;

type HookFn = Box<dyn Fn(&LifecycleEvent) + Send + Sync>;

#[derive(Debug, Clone)]
pub enum LifecycleEvent {
    SessionStarted,
    SessionEnded,
    ToolCalled(String),
    ToolCompleted(String, Duration),
    Error(String),
}

pub struct HookRegistry {
    hooks: Vec<HookFn>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self { hooks: Vec::new() }
    }

    pub fn register<F>(&mut self, hook: F)
    where
        F: Fn(&LifecycleEvent) + Send + Sync + 'static,
    {
        self.hooks.push(Box::new(hook));
    }

    pub fn dispatch(&self, event: &LifecycleEvent) {
        for hook in &self.hooks {
            hook(event);
        }
    }

    pub fn len(&self) -> usize {
        self.hooks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.hooks.is_empty()
    }
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_hook_registry_empty() {
        let reg = HookRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
    }

    #[test]
    fn test_hook_registry_register_and_dispatch() {
        let counter = Arc::new(AtomicUsize::new(0));
        let c = counter.clone();
        let mut reg = HookRegistry::new();
        reg.register(move |_| {
            c.fetch_add(1, Ordering::SeqCst);
        });
        assert_eq!(reg.len(), 1);
        reg.dispatch(&LifecycleEvent::SessionStarted);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_multiple_hooks() {
        let counter = Arc::new(AtomicUsize::new(0));
        let c1 = counter.clone();
        let c2 = counter.clone();
        let mut reg = HookRegistry::new();
        reg.register(move |_| { c1.fetch_add(1, Ordering::SeqCst); });
        reg.register(move |_| { c2.fetch_add(1, Ordering::SeqCst); });
        reg.dispatch(&LifecycleEvent::SessionEnded);
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_dispatch_tool_called() {
        let last_tool = Arc::new(std::sync::Mutex::new(String::new()));
        let t = last_tool.clone();
        let mut reg = HookRegistry::new();
        reg.register(move |ev| {
            if let LifecycleEvent::ToolCalled(name) = ev {
                *t.lock().expect("mutex not poisoned") = name.clone();
            }
        });
        reg.dispatch(&LifecycleEvent::ToolCalled("git".into()));
        assert_eq!(*last_tool.lock().expect("mutex not poisoned"), "git");
    }

    #[test]
    fn test_dispatch_tool_completed() {
        let last_dur = Arc::new(std::sync::Mutex::new(Duration::ZERO));
        let d = last_dur.clone();
        let mut reg = HookRegistry::new();
        reg.register(move |ev| {
            if let LifecycleEvent::ToolCompleted(_, dur) = ev {
                *d.lock().expect("mutex not poisoned") = *dur;
            }
        });
        reg.dispatch(&LifecycleEvent::ToolCompleted("test".into(), Duration::from_secs(5)));
        assert_eq!(last_dur.lock().expect("mutex not poisoned").as_secs(), 5);
    }

    #[test]
    fn test_dispatch_error() {
        let last_err = Arc::new(std::sync::Mutex::new(String::new()));
        let e = last_err.clone();
        let mut reg = HookRegistry::new();
        reg.register(move |ev| {
            if let LifecycleEvent::Error(msg) = ev {
                *e.lock().expect("mutex not poisoned") = msg.clone();
            }
        });
        reg.dispatch(&LifecycleEvent::Error("something failed".into()));
        assert_eq!(*last_err.lock().expect("mutex not poisoned"), "something failed");
    }

    #[test]
    fn test_default_hook_registry() {
        let reg: HookRegistry = Default::default();
        assert!(reg.is_empty());
    }
}
