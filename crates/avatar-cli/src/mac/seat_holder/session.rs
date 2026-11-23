use std::io;
use std::path::Path;
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use avatar_seat::edges::{EdgeFailure, HttpProbe, SystemClock, SystemProcesses};
use avatar_seat::holder::{ConversationSeat, HoldEdges, Reporter, hold_seat};
use avatar_seat::local::{LocalSeat, MEETING_PAGE_QUERY, SEAT_PORT, SeatConfig, tunnel_command};
use tokio::runtime::Builder;
use tokio::signal::unix::{SignalKind, signal};

use super::super::environment::seat_config;
use super::keeping::{hold_meeting, longest_seat_seconds};
use super::log::SeatLog;
use super::tunnel::ChildTunnel;

struct Holding<'run> {
    config: &'run SeatConfig,
    query: &'run str,
    terminated: &'run AtomicBool,
    log: &'run SeatLog,
}

fn hold_over(
    holding: &Holding<'_>,
    tunnel: &mut ChildTunnel<'_>,
    edges: &HoldEdges<'_>,
) -> Result<(), EdgeFailure> {
    if holding.query == MEETING_PAGE_QUERY {
        return hold_meeting(holding.config, tunnel, edges);
    }
    hold_seat(&mut ConversationSeat, tunnel, edges, longest_seat_seconds())
}

fn hold_edges<'edges>(
    reporter: &'edges SeatLog,
    clock: &'edges SystemClock,
    probe: &'edges HttpProbe,
    local: &'edges LocalSeat<'edges>,
) -> HoldEdges<'edges> {
    HoldEdges {
        clock,
        probe,
        reporter,
        local,
    }
}

fn hold_session(holding: &Holding<'_>) -> Result<(), EdgeFailure> {
    let (clock, probe) = (SystemClock::new(), HttpProbe { port: SEAT_PORT });
    let local = LocalSeat::new(holding.config, &SystemProcesses, &clock, &probe);
    let edges = hold_edges(holding.log, &clock, &probe, &local);
    let command = tunnel_command(holding.config);
    let mut tunnel = ChildTunnel::spawn(&command, holding.log, holding.terminated)?;
    hold_over(holding, &mut tunnel, &edges)
}

fn exit_of(outcome: Result<(), EdgeFailure>, log: &SeatLog) -> ExitCode {
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => {
            log.report(&failure.to_string());
            ExitCode::FAILURE
        }
    }
}

fn hold(config: &SeatConfig, query: &str, terminated: &AtomicBool) -> ExitCode {
    let log = SeatLog::open(Path::new(&config.home));
    let holding = Holding {
        config,
        query,
        terminated,
        log: &log,
    };
    exit_of(hold_session(&holding), &log)
}

fn watch_for_termination(terminated: Arc<AtomicBool>) -> io::Result<()> {
    let mut termination = signal(SignalKind::terminate())?;
    tokio::spawn(async move {
        termination.recv().await;
        terminated.store(true, Ordering::SeqCst);
    });
    Ok(())
}

async fn hold_until_terminated(config: SeatConfig, query: String) -> io::Result<ExitCode> {
    let terminated = Arc::new(AtomicBool::new(false));
    watch_for_termination(Arc::clone(&terminated))?;
    let holding = tokio::task::spawn_blocking(move || hold(&config, &query, &terminated));
    holding.await.map_err(io::Error::other)
}

pub(super) fn run(query: &str) -> io::Result<ExitCode> {
    let config = seat_config()?;
    let runtime = Builder::new_multi_thread().enable_all().build()?;
    runtime.block_on(hold_until_terminated(config, query.to_owned()))
}
