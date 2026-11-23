use std::future::Future;

use tokio::sync::watch;

pub(crate) async fn until_first_finishes(
    first: impl Future<Output = ()>,
    second: impl Future<Output = ()>,
) {
    tokio::select! {
        () = first => {}
        () = second => {}
    }
}

async fn stop_requested(mut stopping: watch::Receiver<bool>) {
    let _ = stopping.wait_for(|requested| *requested).await;
}

pub(crate) async fn until_stopped(stopping: watch::Receiver<bool>, work: impl Future<Output = ()>) {
    until_first_finishes(work, stop_requested(stopping)).await;
}
