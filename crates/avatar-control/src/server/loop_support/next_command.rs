use tokio::sync::mpsc::Receiver;
use tokio::sync::watch;
use tokio::time::Interval;

use crate::server::command::Command;

pub(crate) async fn next_command(
    commands: &mut Receiver<Command>,
    ticker: &mut Interval,
    stopped: &mut watch::Receiver<bool>,
) -> Option<Command> {
    tokio::select! {
        command = command_or_tick(commands, ticker) => command,
        _ = stopped.wait_for(|requested| *requested) => None,
    }
}

async fn command_or_tick(
    commands: &mut Receiver<Command>,
    ticker: &mut Interval,
) -> Option<Command> {
    tokio::select! {
        command = commands.recv() => command,
        _ = ticker.tick() => Some(Command::Tick),
    }
}
