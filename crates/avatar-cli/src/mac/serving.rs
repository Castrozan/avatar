use std::future::Future;
use std::io::{self, Write};
use std::process::ExitCode;

use avatar_mac::edge::browser::HeadlessBrowser;
use avatar_mac::edge::server::{ServerProgram, hold_seat, relay_microphone};
use avatar_mac::edge::stage::BrowserProcesses;
use avatar_mac::edge::terminate::until_terminated;
use avatar_mac::failure::{Failure, FailureKind};
use tokio::runtime::Builder;

use super::hosting::{Hosting, Kind};

fn report(failure: &Failure, no_page_message: &str) -> ExitCode {
    let timed_out = failure.kind == FailureKind::Timeout;
    let text = if timed_out {
        no_page_message
    } else {
        &failure.message
    };
    let _ = writeln!(io::stderr(), "{text}");
    ExitCode::FAILURE
}

fn exit_of(outcome: Option<Result<(), Failure>>, no_page_message: &str) -> ExitCode {
    match outcome {
        None | Some(Ok(())) => ExitCode::SUCCESS,
        Some(Err(failure)) => report(&failure, no_page_message),
    }
}

async fn serve(
    kind: Kind,
    program: ServerProgram,
    open_page: Box<dyn FnOnce() -> Result<(), Failure> + Send>,
) -> Result<(), Failure> {
    let output = Box::new(io::stdout());
    match kind {
        Kind::Seat => hold_seat(program, output, open_page).await,
        Kind::Microphone => relay_microphone(program, output, open_page).await,
    }
}

fn opening(
    browser: HeadlessBrowser,
    query: &'static str,
    autoplay: bool,
) -> Box<dyn FnOnce() -> Result<(), Failure> + Send> {
    Box::new(move || {
        BrowserProcesses::system().end_matching(&browser.profile_flag)?;
        browser.open(query, autoplay)
    })
}

fn run_until_terminated(
    work: impl Future<Output = Result<(), Failure>>,
) -> io::Result<Option<Result<(), Failure>>> {
    let runtime = Builder::new_multi_thread().enable_all().build()?;
    runtime.block_on(until_terminated(work))
}

pub(super) fn host(hosting: Hosting) -> io::Result<ExitCode> {
    let flag = hosting.browser.profile_flag.clone();
    let message = hosting.program.no_page_message.clone();
    let autoplay = matches!(hosting.kind, Kind::Seat);
    let open_page = opening(hosting.browser, hosting.query, autoplay);
    let outcome = run_until_terminated(serve(hosting.kind, hosting.program, open_page))?;
    let _ = BrowserProcesses::system().end_matching(&flag);
    Ok(exit_of(outcome, &message))
}
