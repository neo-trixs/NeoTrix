use super::*;

impl BackgroundLoop {
    pub fn spawn<F>(&mut self, task: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        self.handles.push(tokio::spawn(task));
    }

    pub async fn shutdown(&mut self) {
        for handle in self.handles.drain(..) {
            handle.abort();
        }
    }
}
