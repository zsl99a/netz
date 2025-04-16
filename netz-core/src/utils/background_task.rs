use std::{future::Future, sync::Arc};

use tokio::task::AbortHandle;

#[derive(Debug, Clone)]
pub struct BackgroundTask(#[allow(dead_code)] Arc<AutoAbort>);

impl BackgroundTask {
    pub fn run<Fut, T>(fut: Fut) -> Self
    where
        Fut: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        Self(Arc::new(AutoAbort(tokio::spawn(fut).abort_handle())))
    }
}

#[derive(Debug)]
struct AutoAbort(AbortHandle);

impl Drop for AutoAbort {
    fn drop(&mut self) {
        self.0.abort();
    }
}
