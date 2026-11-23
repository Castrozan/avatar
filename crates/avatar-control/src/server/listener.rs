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
use crate::server::probe::ServerProbe;
use crate::server::speech::{SpeechPort, forward_events};

const COMMAND_CAPACITY: usize = 256;

pub struct ShutdownHandle(watch::Sender<bool>);

impl ShutdownHandle {
    pub fn shutdown(self) {
        self.0.send_replace(true);
    }
}

pub struct RunningServer {
    pub address: SocketAddr,
    pub shutdown: ShutdownHandle,
    pub probe: ServerProbe,
}

impl RunningServer {
    fn new(address: SocketAddr, stop: watch::Sender<bool>, commands: Sender<Command>) -> Self {
        Self {
            address,
            shutdown: ShutdownHandle(stop),
            probe: ServerProbe::new(commands),
        }
    }
}

pub async fn serve<Port: SpeechPort>(
    address: SocketAddr,
    speech_port: Port,
    origins: Vec<String>,
) -> io::Result<(SocketAddr, ShutdownHandle)> {
    let running = serve_probed(address, speech_port, origins).await?;
    Ok((running.address, running.shutdown))
}

pub async fn serve_probed<Port: SpeechPort>(
    address: SocketAddr,
    speech_port: Port,
    origins: Vec<String>,
) -> io::Result<RunningServer> {
    let listener = TcpListener::bind(address).await?;
    let (stop, stopped) = watch::channel(false);
    let address = listener.local_addr()?;
    let commands = start_actor(speech_port, stopped.clone());
    tokio::spawn(run(listener, commands.clone(), origins, stopped));
    Ok(RunningServer::new(address, stop, commands))
}

async fn run(
    listener: TcpListener,
    commands: Sender<Command>,
    origins: Vec<String>,
    stopped: watch::Receiver<bool>,
) {
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
