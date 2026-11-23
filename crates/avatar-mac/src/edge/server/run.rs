use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use tokio::net::TcpListener;
use tokio::time::timeout;

use super::state::SeatState;
use crate::failure::{Failure, FailureKind};

fn joined(outcome: Result<Result<(), Failure>, tokio::task::JoinError>) -> Result<(), Failure> {
    outcome.map_err(|error| Failure::new(FailureKind::Os, &error.to_string()))?
}

async fn await_page(state: &SeatState, attach: Duration, message: &str) -> Result<(), Failure> {
    let mut presence = state.subscribe();
    let attached = timeout(attach, presence.wait_for(|seen| seen.attached)).await;
    attached
        .map_err(|_| Failure::timeout(message))?
        .map_err(|_| Failure::runtime(message))?;
    let _ = presence.wait_for(|seen| seen.detached).await;
    Ok(())
}

async fn hold(
    state: &SeatState,
    open_page: impl FnOnce() -> Result<(), Failure> + Send + 'static,
    attach: (Duration, &str),
) -> Result<(), Failure> {
    joined(tokio::task::spawn_blocking(open_page).await)?;
    await_page(state, attach.0, attach.1).await
}

pub async fn serve_until_the_page_leaves(
    listener: TcpListener,
    router: Router,
    state: &Arc<SeatState>,
    open_page: impl FnOnce() -> Result<(), Failure> + Send + 'static,
    attach: (Duration, &str),
) -> Result<(), Failure> {
    let server = tokio::spawn(axum::serve(listener, router).into_future());
    let outcome = hold(state, open_page, attach).await;
    server.abort();
    outcome
}
