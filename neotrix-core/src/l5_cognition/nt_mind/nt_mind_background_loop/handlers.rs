use super::*;
use std::time::Duration;

impl BackgroundLoop {
    /// Spawn an ad-hoc task outside the coordinated handler system.
    /// Such tasks will be aborted during shutdown without grace period.
    pub fn spawn<F>(&mut self, task: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        self.handles.push(tokio::spawn(task));
    }

    /// Gracefully shut down all background handler tasks.
    ///
    /// 1. Broadcasts shutdown signal via the coordinator watch channel.
    /// 2. Waits up to 5 seconds for all handlers to complete their current
    ///    tick and exit.
    /// 3. Any handler that hasn't exited after 5s is aborted.
    ///
    /// Handlers spawned via `spawn()` (not `spawn_handler!`) are
    /// immediately aborted if they haven't finished by the deadline.
    pub async fn shutdown(&mut self) {
        if !self.started {
            log::warn!("[bg] shutdown called but not started");
            return;
        }

        let coordinator = match self.shutdown_coordinator.take() {
            Some(c) => c,
            None => return,
        };

        log::info!(
            "[bg] shutdown: signaling {} coordinated handlers",
            self.handles.len()
        );

        // 1. Broadcast shutdown signal to all watch receivers
        let _ = coordinator.sender.send(true);
        drop(coordinator.sender);

        // 2. Try graceful join with 5-second deadline
        //    Each handler gets up to 5s to observe the shutdown signal
        //    and exit gracefully. After the deadline, remaining tasks
        //    are aborted via AbortHandle (which works independently of
        //    the JoinHandle).
        let deadline = tokio::time::sleep(Duration::from_secs(5));
        tokio::pin!(deadline);

        for handle in self.handles.drain(..) {
            let abort = handle.abort_handle();
            tokio::select! {
                biased;
                _ = &mut deadline => {
                    abort.abort();
                    log::warn!("[bg] shutdown deadline reached — aborting task");
                }
                _ = handle => {}
            }
        }

        log::info!("[bg] shutdown complete");
    }
}
