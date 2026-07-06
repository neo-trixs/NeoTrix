use super::super::{Plugin, PluginEvent};

/// A built-in plugin that logs every dispatched event.
pub struct LoggingPlugin;

impl Plugin for LoggingPlugin {
    fn name(&self) -> &'static str {
        "logging"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn on_load(&self) -> Result<(), String> {
        log::info!("[plugin/logging] loaded");
        Ok(())
    }

    fn on_unload(&self) -> Result<(), String> {
        log::info!("[plugin/logging] unloaded");
        Ok(())
    }

    fn on_event(&self, event: &PluginEvent) -> Result<(), String> {
        match event {
            PluginEvent::ConfigChanged => {
                log::info!("[plugin/logging] config changed");
            }
            PluginEvent::SessionStarted => {
                log::info!("[plugin/logging] session started");
            }
            PluginEvent::SessionEnded => {
                log::info!("[plugin/logging] session ended");
            }
            PluginEvent::TaskReceived(task) => {
                log::info!("[plugin/logging] task received: {}", task);
            }
            PluginEvent::TaskCompleted(task) => {
                log::info!("[plugin/logging] task completed: {}", task);
            }
            PluginEvent::BrainTick => {
                log::info!("[plugin/logging] brain tick");
            }
            PluginEvent::Shutdown => {
                log::info!("[plugin/logging] shutdown");
            }
        }
        Ok(())
    }
}
