use serde_json::Value;

use super::ROUND_SECONDS;
use super::Reporter;
use crate::edges::{Clock, EdgeFailure, SeatProbe};
use crate::local::LocalSeat;

pub trait SeatKeeper {
    fn look(&mut self, seat: Option<&Value>) -> bool;
    fn end(&mut self);
}

pub trait TunnelProcess {
    fn stays_open_for(&mut self, seconds: u64) -> bool;
    fn exit_code(&mut self) -> Option<i32>;
    fn terminate(&mut self);
}

pub struct HoldEdges<'edges> {
    pub clock: &'edges dyn Clock,
    pub probe: &'edges dyn SeatProbe,
    pub reporter: &'edges dyn Reporter,
    pub local: &'edges LocalSeat<'edges>,
}

fn report_tunnel_end(tunnel: &mut impl TunnelProcess, reporter: &dyn Reporter) {
    let ending = tunnel
        .exit_code()
        .map_or_else(|| "None".to_owned(), |code| code.to_string());
    reporter.report(&format!("This seat ends; its tunnel ended with {ending}"));
}

pub fn hold_seat<Keeper: SeatKeeper, Tunnel: TunnelProcess>(
    keeper: &mut Keeper,
    tunnel: &mut Tunnel,
    edges: &HoldEdges<'_>,
    longest_seat_seconds: u32,
) -> Result<(), EdgeFailure> {
    let deadline = edges.clock.monotonic() + f64::from(longest_seat_seconds);
    watch_until_over(keeper, tunnel, edges, deadline);
    report_tunnel_end(tunnel, edges.reporter);
    tunnel.terminate();
    keeper.end();
    edges.local.close_browsers()
}

fn seat_is_kept(keeper: &mut impl SeatKeeper, edges: &HoldEdges<'_>) -> bool {
    let kept = keeper.look(edges.probe.health().as_ref());
    if !kept {
        edges
            .reporter
            .report("The meeting ended on the host, so this seat ends too");
    }
    kept
}

fn watch_until_over(
    keeper: &mut impl SeatKeeper,
    tunnel: &mut impl TunnelProcess,
    edges: &HoldEdges<'_>,
    deadline: f64,
) {
    while edges.clock.monotonic() < deadline
        && tunnel.stays_open_for(ROUND_SECONDS)
        && seat_is_kept(keeper, edges)
    {}
}
