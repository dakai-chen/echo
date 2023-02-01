use std::{future::Future, time::Duration};

use tokio::sync::mpsc::{self, Receiver, Sender};

pub(crate) struct GracefulShutdown {
    watcher_tx: Option<Sender<()>>,
    watcher_rx: Receiver<()>,
}

impl GracefulShutdown {
    pub(crate) fn new() -> Self {
        let (watcher_tx, watcher_rx) = mpsc::channel::<()>(1);
        Self {
            watcher_tx: Some(watcher_tx),
            watcher_rx,
        }
    }

    pub(crate) fn watch<T>(&self, task: T) -> impl Future<Output = T::Output>
    where
        T: Future,
    {
        let watcher_tx = self.watcher_tx.clone();
        async move {
            let _watcher_tx = watcher_tx;
            task.await
        }
    }

    pub(crate) async fn shutdown(mut self, timeout: Option<Duration>) {
        self.watcher_tx.take();
        tokio::select! {
            _ = self.watcher_rx.recv() => {},
            _ = sleep(timeout) => {},
        }
    }
}

async fn sleep(timeout: Option<Duration>) {
    if let Some(timeout) = timeout {
        tokio::time::sleep(timeout).await
    } else {
        std::future::pending().await
    }
}
