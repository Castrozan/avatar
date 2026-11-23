use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use axum::Router;
use axum::routing::get;
use tokio::net::TcpListener;
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::watch;

use crate::server::actor::{Actor, drive};
use crate::server::command::Command;
use crate::server::connection::{ConnectionState, upgrade};
use crate::server::speech::{SpeechPort, forward_events};

const COMMAND_CAPACITY: usize = 256;

pub struct ShutdownHandle(watch::Sender<bool>);

impl ShutdownHandle {
    pub fn shutdown(self) {
        self.0.send_replace(true);
    }
}

pub async fn serve<Port: SpeechPort>(
    address: SocketAddr,
    speech_port: Port,
    origins: Vec<String>,
) -> io::Result<(SocketAddr, ShutdownHandle)> {
    let listener = TcpListener::bind(address).await?;
    let (stop, stopped) = watch::channel(false);
    let local_address = listener.local_addr()?;
    tokio::spawn(run(listener, speech_port, origins, stopped));
    Ok((local_address, ShutdownHandle(stop)))
}

async fn run<Port: SpeechPort>(
    listener: TcpListener,
    port: Port,
    origins: Vec<String>,
    stopped: watch::Receiver<bool>,
) {
    let commands = start_actor(port, stopped.clone());
    let router = build_router(commands, origins);
    let _ = axum::serve(listener, router)
        .with_graceful_shutdown(await_stop(stopped))
        .await;
}

fn start_actor<Port: SpeechPort>(
    mut port: Port,
    stopped: watch::Receiver<bool>,
) -> Sender<Command> {
    let (commands, receiver) = mpsc::channel(COMMAND_CAPACITY);
    if let Some(events) = port.take_events() {
        tokio::spawn(forward_events(events, commands.clone()));
    }
    tokio::spawn(drive(Actor::new(port, receiver, stopped)));
    commands
}

fn build_router(commands: Sender<Command>, origins: Vec<String>) -> Router {
    let state = ConnectionState {
        commands,
        client_ids: Arc::new(AtomicU64::new(1)),
        origins: origins.into(),
    };
    Router::new().route("/", get(upgrade)).with_state(state)
}

async fn await_stop(mut stopped: watch::Receiver<bool>) {
    let _ = stopped.wait_for(|requested| *requested).await;
}
