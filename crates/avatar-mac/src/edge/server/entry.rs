use std::io::Write;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use tokio::net::TcpListener;

use super::allowed_origins;
use super::files::PageFiles;
use super::routes::{microphone_router, seat_router};
use super::run::serve_until_the_page_leaves;
use super::state::SeatState;
use crate::edge::failure::os_failure;
use crate::failure::Failure;

pub struct ServerProgram {
    pub port: u16,
    pub files: PageFiles,
    pub attach: Duration,
    pub no_page_message: String,
}

type OpenPage = Box<dyn FnOnce() -> Result<(), Failure> + Send>;

async fn serve(
    program: ServerProgram,
    output: Box<dyn Write + Send>,
    open_page: OpenPage,
    router_for: fn(Arc<SeatState>) -> Router,
) -> Result<(), Failure> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, program.port)).await;
    let listener = listener.map_err(|error| os_failure(&error))?;
    let origins = allowed_origins(program.port);
    let state = Arc::new(SeatState::new(output, program.files, origins));
    let router = router_for(Arc::clone(&state));
    let attach = (program.attach, program.no_page_message.as_str());
    serve_until_the_page_leaves(listener, router, &state, open_page, attach).await
}

pub async fn hold_seat(
    program: ServerProgram,
    output: Box<dyn Write + Send>,
    open_page: OpenPage,
) -> Result<(), Failure> {
    serve(program, output, open_page, seat_router).await
}

pub async fn relay_microphone(
    program: ServerProgram,
    output: Box<dyn Write + Send>,
    open_page: OpenPage,
) -> Result<(), Failure> {
    serve(program, output, open_page, microphone_router).await
}
