use std::marker::PhantomData;
use std::time::Duration;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::time::timeout;

pub struct BackpressurePipeline<T: Send + 'static> {
    pub name: &'static str,
    pub step_timeout: Duration,
    pub buffer_size: usize,
    _phantom: PhantomData<T>,
}

impl<T: Send + 'static> BackpressurePipeline<T> {
    pub fn new(
        _name: &'static str,
        buffer_size: usize,
        _step_timeout: Duration,
    ) -> (Sender<T>, Receiver<T>) {
        let (tx, rx) = mpsc::channel(buffer_size);
        (tx, rx)
    }

    pub async fn process<F, Fut>(
        rx: &mut Receiver<T>,
        step_timeout: Duration,
        handler_name: &str,
        process_fn: F,
    ) where
        F: Fn(T) -> Fut,
        Fut: std::future::Future<Output = Result<(), String>> + Send,
    {
        while let Some(item) = rx.recv().await {
            match timeout(step_timeout, process_fn(item)).await {
                Ok(Ok(())) => {}
                Ok(Err(e)) => log::warn!("[backpressure] {} step failed: {}", handler_name, e),
                Err(_) => log::warn!(
                    "[backpressure] {} timed out after {:?}",
                    handler_name,
                    step_timeout
                ),
            };
        }
    }
}
